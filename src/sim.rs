use std::time::Instant;

use nannou::{
    prelude::*,
    wgpu::{Device, Queue},
};

use crate::{
    diff_eq::{AllowedMethod, GravitationalODE, State},
    gpu, GpuAttractor, GpuColor, GpuDust, GpuState,
};

pub trait Body {
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity(&self) -> Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn mass(&self) -> f32;
    fn color(&self) -> Hsv;
    fn set_state(&mut self, pos: Vec2, vel: Vec2);

    fn draw(&self, draw: &Draw, scale: f32) {
        draw.ellipse()
            .xy(self.position())
            .radius(self.mass().sqrt() * 0.5 / scale)
            .color(self.color());
    }
}

#[derive(Clone, Copy)]
pub struct Attractor {
    position: Vec2,
    velocity: Vec2,
    mass: f32,
    color: Hsv,
}

impl Attractor {
    pub fn new(position: Vec2, velocity: Vec2, mass: f32, hue: f32) -> Self {
        Attractor {
            position,
            velocity,
            mass,
            color: Hsv::new(hue, 1.0, 1.0),
        }
    }

    pub fn set_orbit(&mut self, center: Vec2, center_mass: f32, clockwise: bool) {
        let r = self.position - center;
        let distance = r.length();
        let speed = (center_mass / distance).sqrt();
        let direction = if clockwise {
            vec2(r.y, -r.x).normalize()
        } else {
            vec2(-r.y, r.x).normalize()
        };
        self.velocity = direction * speed;
    }

    /// Modifies self and other to be in a stable orbit around their common center of mass, with self as the "planet" and other as the "sun". If `planet_clockwise` is true, the planet will orbit clockwise around the sun.
    pub fn orbit_pair(&mut self, other: &mut Attractor, planet_clockwise: bool) {
        let r = other.position - self.position;
        let distance = r.length();
        let mass_sum = self.mass + other.mass;

        self.position = -r * (other.mass / mass_sum);
        other.position = r * (self.mass / mass_sum);

        let denom = (mass_sum * distance).sqrt().max(1e-8);
        let direction = vec2(-r.y, r.x).normalize();
        let sign = if planet_clockwise { 1.0 } else { -1.0 };

        self.velocity = sign * direction * (other.mass / denom);
        other.velocity = sign * -direction * (self.mass / denom);
    }
}

impl Body for Attractor {
    fn position(&self) -> Vec2 {
        self.position
    }
    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }
    fn velocity(&self) -> Vec2 {
        self.velocity
    }
    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn color(&self) -> Hsv {
        self.color
    }
    fn set_state(&mut self, pos: Vec2, vel: Vec2) {
        self.position = pos;
        self.velocity = vel;
    }
}

#[derive(Clone)]
pub struct Dust {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Hsv,
}

impl Dust {
    pub fn new(position: Vec2, velocity: Vec2) -> Self {
        Dust {
            position,
            velocity,
            color: Hsv::new(random_range(0.0, 255.0), 1.0, 1.0),
        }
    }

    pub fn coord_swap(&self, transform: impl Fn(Vec2) -> Vec2) -> Dust {
        Dust {
            position: transform(self.position),
            velocity: transform(self.velocity),
            color: self.color,
        }
    }
}

impl Body for Dust {
    fn position(&self) -> Vec2 {
        self.position
    }
    fn position_mut(&mut self) -> &mut Vec2 {
        &mut self.position
    }
    fn velocity(&self) -> Vec2 {
        self.velocity
    }
    fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }
    fn mass(&self) -> f32 {
        panic!("Asked for the mass of a dust");
    }
    fn color(&self) -> Hsv {
        self.color
    }
    fn set_state(&mut self, pos: Vec2, vel: Vec2) {
        self.position = pos;
        self.velocity = vel;
    }

    // Manually implemented draw method for dust without calls to Body::mass()
    // Probably silly but idc rn
    fn draw(&self, draw: &Draw, scale: f32) {
        draw.ellipse()
            .xy(self.position())
            .radius(2.0 * scale)
            .color(self.color());
    }
}

/// Physics system that manages bodies and integration
pub struct System<M>
where
    M: AllowedMethod<M>,
{
    pub attractors: Vec<Attractor>,
    pub dust: Vec<Dust>,
    pub gpu_state: Option<GpuState>,
    pub steps: u32,
    _data: std::marker::PhantomData<M>,
}

impl<M> Clone for System<M>
where
    M: AllowedMethod<M>,
{
    fn clone(&self) -> Self {
        if self.gpu_state.is_some() {
            panic!("Cannot clone System with GPU state initialized");
        }

        System {
            attractors: self.attractors.to_vec(),
            dust: self.dust.clone(),
            gpu_state: None,
            steps: self.steps,
            _data: std::marker::PhantomData,
        }
    }
}

impl<M> System<M>
where
    M: AllowedMethod<M>,
{
    pub fn new() -> Self {
        System {
            attractors: Vec::new(),
            dust: Vec::new(),
            gpu_state: None,
            steps: 0,
            _data: std::marker::PhantomData,
        }
    }

    pub fn add_attractor(&mut self, body: Attractor) {
        self.attractors.push(body);
    }
    pub fn add_dust(&mut self, dust: Dust) {
        self.dust.push(dust);
    }

    pub fn include_setup(&mut self, setup: &crate::scene_layout::Setup, num_dust: u32) {
        self.dust = Vec::with_capacity(std::mem::size_of::<Dust>() * num_dust as usize);
        setup.build(num_dust, &mut self.dust);
    }
    pub fn include_setup_random(&mut self, setup: &crate::scene_layout::Setup, num_dust: u32) {
        self.dust = Vec::with_capacity(std::mem::size_of::<Dust>() * num_dust as usize);
        setup.build_random(num_dust, &mut self.dust);
    }

    pub fn init_gpu(&mut self, device: &Device) {
        let attractors = self.get_attractors_gpu();
        let dust_particles = self.get_dust_gpu();
        let colors = dust_particles
            .iter()
            .enumerate()
            .map(|(i, _)| GpuColor::new(i as f32 / dust_particles.len() as f32 * 255.0))
            .collect::<Vec<GpuColor>>();
        let gpu_state = GpuState::new(device, &attractors, &dust_particles, &colors);
        self.gpu_state = Some(gpu_state);
    }

    // pub fn update(
    //     &mut self,
    //     dt: f64,
    //     sub_steps: u32,
    //     device: Option<&Device>,
    //     queue: Option<&Queue>,
    // ) {
    //     self.update_diff_eq(dt, sub_steps, device, queue);
    //     self.steps += 1;
    // }

    // Main update
    pub fn update(
        &mut self,
        dt: f64,
        sub_steps: u32,
        device: Option<&Device>,
        queue: Option<&Queue>,
    ) {
        let ode = GravitationalODE::new(self.get_masses());
        let state = State::from_system(self);

        // self.integrator.init(dt, &state, &ode);
        let out = M::update(
            &state,
            &ode,
            dt,
            sub_steps,
            device,
            queue,
            self.gpu_state.as_mut(),
        );

        self.attractors
            .iter_mut()
            .zip(out)
            .for_each(|(body, (pos, vel))| {
                body.set_state(pos, vel);
            });
    }

    /// Updates the system until condition is met or max steps reached
    pub fn update_until(
        &mut self,
        mut condition: impl FnMut(&System<M>) -> bool,
        dt: f64,
        sub_steps: u32,
        max_steps: u32,
        device: Option<&Device>,
        queue: Option<&Queue>,
    ) {
        let mut steps = 0;
        while !condition(self) && steps < max_steps {
            self.update(dt, sub_steps, device, queue);
            steps += 1;
        }
    }

    pub fn get_attractors(&self) -> &[Attractor] {
        &self.attractors
    }
    pub fn get_attractors_gpu(&self) -> Vec<GpuAttractor> {
        self.attractors
            .iter()
            .map(|b| {
                let mut stages = [[0.0; 2]; 4];
                stages[0] = b.position().into();
                GpuAttractor::new(stages, b.mass())
            })
            .collect()
    }
    pub fn get_dusts(&self) -> &[Dust] {
        &self.dust
    }
    pub fn get_masses(&self) -> Vec<f64> {
        self.attractors.iter().map(|b| b.mass() as f64).collect()
    }

    /// Converts all `Dust`s in the system to `GpuDust`s
    pub fn get_dust_gpu(&mut self) -> Vec<GpuDust> {
        println!(
            "Converting {} dust particles to GPU format",
            self.dust.len()
        );

        let now = Instant::now();

        let dusts = self
            .dust
            .drain(..)
            .map(|d| GpuDust::new(d.position(), d.velocity()))
            .collect();

        println!("Done, took {}s", now.elapsed().as_secs_f64());

        dusts
    }

    pub fn get_attractor(&self, index: usize) -> Option<&Attractor> {
        self.attractors.get(index)
    }
    pub fn get_dust(&self, index: usize) -> Option<&Dust> {
        self.dust.get(index)
    }

    pub fn center_of_mass(&self) -> Vec2 {
        let mut total_mass = 0.0;
        let mut com = Vec2::ZERO;

        for body in self.attractors.iter() {
            com += body.position() * body.mass();
            total_mass += body.mass();
        }

        if total_mass > 0.0 {
            com / total_mass
        } else {
            Vec2::ZERO
        }
    }

    pub fn rotate_around(&self, body_index: usize) -> Option<impl Fn(Vec2) -> Vec2> {
        if let Some(body) = self.get_attractor(body_index) {
            let com = self.center_of_mass();
            let r = body.position() - com;
            let angle = r.angle();

            return Some(move |pos: Vec2| (pos - com).rotate(-angle));
        }

        None
    }

    pub fn draw(
        &self,
        draw: &Draw,
        device: &Device,
        queue: &Queue,
        texture_view: &wgpu::TextureView,
        scale: f32, // figure out how to sync gpu scale and draw zooming
    ) {
        for body in self.get_attractors() {
            body.draw(draw, scale);
        }

        if let Some(gpu_state) = &self.gpu_state {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };

            {
                let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
                render_pass.set_pipeline(&gpu_state.render_pipeline);

                render_pass.set_bind_group(0, &gpu_state.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, gpu_state.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, gpu_state.dust_buffer.slice(..));
                render_pass.set_vertex_buffer(2, gpu_state.color_buffer.slice(..));

                render_pass.draw(
                    0..gpu::QUAD_VERTICES.len() as u32,
                    0..gpu_state.num_particles,
                );
            }
            queue.submit(Some(encoder.finish()));
        }
    }
}

impl<M> Default for System<M>
where
    M: AllowedMethod<M>,
{
    fn default() -> Self {
        Self::new()
    }
}

pub fn sun_planet_binary<M>(sun_mass: f32, planet_mass: f32, planet_clockwise: bool) -> System<M>
where
    M: AllowedMethod<M>,
{
    let mut system = System::new();

    let mut sun = Attractor::new(Vec2::ZERO, Vec2::ZERO, sun_mass, 0.0);
    let mut planet = Attractor::new(vec2(200.0, 0.0), Vec2::ZERO, planet_mass, 100.0);
    sun.orbit_pair(&mut planet, planet_clockwise);

    system.add_attractor(sun);
    system.add_attractor(planet);

    system
}

pub fn sun_planet_binary_cw<M>(sun_mass: f32, planet_mass: f32) -> System<M>
where
    M: AllowedMethod<M>,
{
    sun_planet_binary(sun_mass, planet_mass, true)
}

pub fn sun_planet_binary_ccw<M>(sun_mass: f32, planet_mass: f32) -> System<M>
where
    M: AllowedMethod<M>,
{
    sun_planet_binary(sun_mass, planet_mass, false)
}
