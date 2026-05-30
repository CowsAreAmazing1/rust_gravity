use nannou::{
    prelude::*,
    wgpu::{Device, Queue},
};

use crate::{gpu, GpuAttractor, GpuDust, GpuState};

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub u: Vec2,
    pub du: Vec2,
}

// Struct for both u and du in RK4
impl State {
    pub const ZERO: Self = Self {
        u: Vec2::ZERO,
        du: Vec2::ZERO,
    };

    pub fn new(u: Vec2, du: Vec2) -> Self {
        State { u, du }
    }

    pub fn scale(&self, factor: f32) -> State {
        State {
            u: self.u * factor,
            du: self.du * factor,
        }
    }

    pub fn combine(&self, other: &State, factor: f32) -> State {
        State {
            u: self.u + other.u * factor,
            du: self.du + other.du * factor,
        }
    }

    pub fn add(&self, other: &State) -> State {
        State {
            u: self.u + other.u,
            du: self.du + other.du,
        }
    }

    /// Calculate gravitational acceleration on self due to another state with given mass. Only makes sense when u: position, du: velocity
    fn grav_acc(&self, other: &dyn Body) -> Vec2 {
        let r = other.position() - self.u;
        let distance_sq = r.length_squared();

        let force_magnitude = other.mass() / (distance_sq + 5.0);
        r.normalize() * force_magnitude
    }
}

pub trait Body {
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity(&self) -> Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn mass(&self) -> f32;
    fn color(&self) -> Hsv;

    // To state
    fn get_state(&self) -> State {
        State::new(self.position(), self.velocity())
    }

    // From state
    fn set_state(&mut self, state: State) {
        *self.position_mut() = state.u;
        *self.velocity_mut() = state.du;
    }

    fn update_euler(&mut self, dt: f32) {
        let velocity = self.velocity();
        *self.position_mut() += velocity * dt;
    }

    fn grav_acc(&self, other: &dyn Body) -> Vec2 {
        let r = other.position() - self.position();
        let distance_sq = r.length_squared();

        let force_magnitude = other.mass() / distance_sq;
        r.normalize() * force_magnitude
    }

    fn draw(&self, draw: &Draw, scale: f32) {
        draw.ellipse()
            .xy(self.position())
            .radius(self.mass().sqrt() * 0.5 / scale)
            .color(self.color());
    }

    fn clone_box(&self) -> Box<dyn Body>;
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

    fn clone_box(&self) -> Box<dyn Body> {
        Box::new(Self {
            position: self.position,
            velocity: self.velocity,
            mass: self.mass,
            color: self.color,
        })
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
        panic!("Asked for the mass of a dust")
    }
    fn color(&self) -> Hsv {
        self.color
    }

    fn draw(&self, draw: &Draw, scale: f32) {
        draw.ellipse()
            .xy(self.position())
            .radius(2.0 * scale)
            .color(self.color());
    }

    fn clone_box(&self) -> Box<dyn Body> {
        Box::new(Self {
            position: self.position,
            velocity: self.velocity,
            color: self.color,
        })
    }
}

/// RK4 Integrator for gravitational systems
pub struct RK4Integrator;

impl RK4Integrator {
    /// Evaluate derivative at given state and time
    fn evaluate(
        bodies: &[Box<dyn Body>],
        body_index: usize,
        state: State,
        derivative: State,
        dt: f32,
    ) -> State {
        // Output a derivative state
        // Create temporary state
        let temp_state = state.combine(&State::new(derivative.u, derivative.du), dt);

        // Calculate acceleration due to all other bodies
        let mut acceleration = Vec2::ZERO;

        for (i, other_body) in bodies.iter().enumerate() {
            if i != body_index {
                acceleration += temp_state.grav_acc(other_body.as_ref());
            }
        }

        State::new(temp_state.du, acceleration)
    }

    fn evaluate_dust(bodies: &[Box<dyn Body>], state: State, derivative: State, dt: f32) -> State {
        // Output a derivative state
        // Create temporary state
        let temp_state = state.combine(&State::new(derivative.u, derivative.du), dt);

        // Calculate acceleration due to all other bodies
        let mut acceleration = Vec2::ZERO;

        for other_body in bodies.iter() {
            acceleration += temp_state.grav_acc(other_body.as_ref());
        }

        State::new(temp_state.du, acceleration)
    }

    pub fn integrate(
        bodies: &[Box<dyn Body>],
        body_index: usize,
        initial_state: State,
        dt: f32,
    ) -> State {
        // RK4 coefficients
        let k1 = Self::evaluate(bodies, body_index, initial_state, State::ZERO, 0.0);
        let k2 = Self::evaluate(bodies, body_index, initial_state, k1, dt * 0.5);
        let k3 = Self::evaluate(bodies, body_index, initial_state, k2, dt * 0.5);
        let k4 = Self::evaluate(bodies, body_index, initial_state, k3, dt);

        let final_state = k1
            .add(&k2.scale(2.0))
            .add(&k3.scale(2.0))
            .add(&k4)
            .scale(1.0 / 6.0);

        // Apply to state
        initial_state.combine(&final_state, dt)
    }

    pub fn integrate_dust(bodies: &[Box<dyn Body>], initial_state: State, dt: f32) -> State {
        // RK4 coefficients
        let k1 = Self::evaluate_dust(bodies, initial_state, State::ZERO, 0.0);
        let k2 = Self::evaluate_dust(bodies, initial_state, k1, dt * 0.5);
        let k3 = Self::evaluate_dust(bodies, initial_state, k2, dt * 0.5);
        let k4 = Self::evaluate_dust(bodies, initial_state, k3, dt);

        let final_state = k1
            .add(&k2.scale(2.0))
            .add(&k3.scale(2.0))
            .add(&k4)
            .scale(1.0 / 6.0);

        // Apply to state
        initial_state.combine(&final_state, dt)
    }
}

/// Physics system that manages bodies and integration
pub struct System {
    pub attractors: Vec<Box<dyn Body>>,
    pub dust: Vec<Dust>,
    pub gpu_state: Option<GpuState>,
    pub use_rk4: bool, // True: RK4, False: Euler
    pub steps: u32,
}

impl Clone for System {
    fn clone(&self) -> Self {
        if self.gpu_state.is_some() {
            panic!("Cannot clone System with GPU state initialized");
        }

        System {
            attractors: self.attractors.iter().map(|b| b.clone_box()).collect(),
            dust: self.dust.clone(),
            gpu_state: None,
            use_rk4: self.use_rk4,
            steps: self.steps,
        }
    }
}

impl System {
    pub fn new() -> Self {
        System {
            attractors: Vec::new(),
            dust: Vec::new(),
            gpu_state: None,
            use_rk4: true, // RK4 by default
            steps: 0,
        }
    }

    pub fn add_attractor<T: Body + 'static>(&mut self, body: T) {
        self.attractors.push(Box::new(body));
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

    pub fn set_integration_method(&mut self, use_rk4: bool) {
        self.use_rk4 = use_rk4;
    }

    pub fn init_gpu(&mut self, device: &Device) {
        let gpu_state = GpuState::new(device, &self.get_bodies_gpu(), &self.get_dust_gpu());
        self.gpu_state = Some(gpu_state);
    }

    // Main update
    pub fn update(
        &mut self,
        dt: f32,
        sub_steps: u32,
        device: Option<&Device>,
        queue: Option<&Queue>,
    ) {
        if self.use_rk4 {
            self.update_rk4(dt, sub_steps, device, queue);
        } else {
            self.update_euler(dt);
        }
        self.steps += 1;
    }

    fn update_rk4(
        &mut self,
        dt: f32,
        sub_steps: u32,
        device: Option<&Device>,
        queue: Option<&Queue>,
    ) {
        // Substeps
        let sub_dt = dt / sub_steps as f32;
        for _ in 0..sub_steps {
            // Get current attractor states
            let mut attractor_states: Vec<State> = self
                .attractors
                .iter()
                .map(|body| body.get_state())
                .collect();
            // Get current dust states
            let mut dust_states: Vec<State> =
                self.dust.iter().map(|body| body.get_state()).collect();

            // Update attractor states
            attractor_states = attractor_states
                .iter()
                .enumerate()
                .map(|(i, &state)| RK4Integrator::integrate(&self.attractors, i, state, sub_dt))
                .collect();
            // Update dust states
            dust_states = dust_states
                .iter()
                .map(|&state| RK4Integrator::integrate_dust(&self.attractors, state, sub_dt))
                .collect();

            // Apply new attractor states
            for (body, new_state) in self.attractors.iter_mut().zip(attractor_states.iter()) {
                body.set_state(*new_state);
            }
            // Apply new dust states
            for (body, new_state) in self.dust.iter_mut().zip(dust_states.iter()) {
                body.set_state(*new_state);
            }

            // Run compute shader for dust
            if let (Some(device), Some(queue)) = (device, queue) {
                let attractors = self.get_bodies_gpu();
                if let Some(gpu_state) = &mut self.gpu_state {
                    gpu_state.update(sub_dt, device, queue, &attractors);
                }
            }
        }
    }

    /// Update using simple Euler integration (for comparison)
    fn update_euler(&mut self, dt: f32) {
        // Calculate forces
        let mut accelerations = vec![Vec2::ZERO; self.attractors.len()];

        (0..self.attractors.len()).for_each(|i| {
            for j in 0..self.attractors.len() {
                if i != j {
                    let force = self.attractors[i].grav_acc(self.attractors[j].as_ref());
                    accelerations[i] += force / self.attractors[i].mass();
                }
            }
        });

        // Apply accelerations and update positions
        for (body, acceleration) in self.attractors.iter_mut().zip(accelerations.iter()) {
            let velocity = body.velocity();
            *body.velocity_mut() += *acceleration * dt;
            *body.position_mut() += velocity * dt;
        }
    }

    /// Updates the system until condition is met or max steps reached
    pub fn update_until(
        &mut self,
        mut condition: impl FnMut(&System) -> bool,
        dt: f32,
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

    pub fn get_bodies(&self) -> &[Box<dyn Body>] {
        &self.attractors
    }
    pub fn get_bodies_gpu(&self) -> Vec<GpuAttractor> {
        self.attractors
            .iter()
            .map(|b| GpuAttractor::new(b.position(), b.mass()))
            .collect()
    }
    pub fn get_dusts(&self) -> &[Dust] {
        &self.dust
    }

    /// Converts all `Dust`s in the system to `GpuDust`s and
    pub fn get_dust_gpu(&mut self) -> Vec<GpuDust> {
        self.dust
            .drain(..)
            .map(|d| GpuDust::new(d.position(), d.velocity()))
            .collect()
    }

    pub fn get_body(&self, index: usize) -> Option<&dyn Body> {
        self.attractors.get(index).map(|v| &**v)
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
        if let Some(body) = self.get_body(body_index) {
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
        for body in self.get_bodies() {
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

                render_pass.draw(
                    0..gpu::QUAD_VERTICES.len() as u32,
                    0..gpu_state.num_particles,
                );
            }
            queue.submit(Some(encoder.finish()));
        }
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sun_planet_binary(sun_mass: f32, planet_mass: f32, planet_clockwise: bool) -> System {
    let mut system = System::new();

    let mut sun = Attractor::new(Vec2::ZERO, Vec2::ZERO, sun_mass, 0.0);
    let mut planet = Attractor::new(vec2(200.0, 0.0), Vec2::ZERO, planet_mass, 100.0);
    sun.orbit_pair(&mut planet, planet_clockwise);

    system.add_attractor(sun);
    system.add_attractor(planet);

    system
}

pub fn sun_planet_binary_cw(sun_mass: f32, planet_mass: f32) -> System {
    sun_planet_binary(sun_mass, planet_mass, true)
}

pub fn sun_planet_binary_ccw(sun_mass: f32, planet_mass: f32) -> System {
    sun_planet_binary(sun_mass, planet_mass, false)
}
