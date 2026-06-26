use differential_equations::{
    methods::{Adaptive, Fixed, Ordinary, SymplecticIntegrator},
    ode::OrdinaryNumericalMethod,
    prelude::*,
};
use main_gravity::prelude::*;
use nannou::{
    color,
    prelude::*,
    rand::{self, seq::SliceRandom},
};

#[derive(Clone)]
struct StateVec {
    positions: Vec<f64>,
    velocities: Vec<f64>,
}
// -> vec![x1, y1, x2, y2, ... , vx1, vy1, vx2, vy2, ...] for each body in the system

impl StateVec {
    fn from_system<M>(system: &System<M>) -> Self
    where
        M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M, f64>,
    {
        let mut positions = Vec::new();
        let mut velocities = Vec::new();

        for body in system.get_attractors().iter() {
            positions.push(body.position().x as f64);
            positions.push(body.position().y as f64);
            velocities.push(body.velocity().x as f64);
            velocities.push(body.velocity().y as f64);
        }

        Self {
            positions,
            velocities,
        }
    }

    fn to_vec(&self) -> Vec<f64> {
        let mut vec = Vec::new();
        for i in 0..self.positions.len() / 2 {
            vec.push(self.positions[i * 2]);
            vec.push(self.positions[i * 2 + 1]);
        }
        for i in 0..self.velocities.len() / 2 {
            vec.push(self.velocities[i * 2]);
            vec.push(self.velocities[i * 2 + 1]);
        }

        vec
    }
}

// Define the gravitational system for the differential equations solver
#[derive(Clone)]
struct GravitationalODE {
    masses: Vec<f64>,
}

impl GravitationalODE {
    fn new(masses: Vec<f64>) -> Self {
        Self { masses }
    }
}

// Implement the ODE trait for our gravitational system
// State vector format: positions and velocities for each body in 2D
impl ODE<f64, Vec<f64>> for GravitationalODE {
    fn diff(&self, _t: f64, y: &Vec<f64>, dydt: &mut Vec<f64>) {
        let n = y.len();
        let half_n = n / 2;
        let num_bodies = half_n / 2;

        // Position derivatives = velocity
        for i in 0..num_bodies {
            let idx = i * 2;
            dydt[idx] = y[half_n + idx]; // dx/dt = vx
            dydt[idx + 1] = y[half_n + idx + 1]; // dy/dt = vy
        }

        // Initialize accelerations to zero
        let mut accelerations = vec![0.0; half_n];

        // Compute pairwise gravitational accelerations for all bodies
        for i in 0..num_bodies {
            // Extract position and velocity indices (2D: x, y for each body)
            let body_i = i * 2;
            let pos_i = (y[body_i], y[body_i + 1]);

            for j in 0..num_bodies {
                let body_j = j * 2;
                if body_i == body_j {
                    continue; // Skip self-interaction
                }

                let pos_j = (y[body_j], y[body_j + 1]);

                // Calculate relative position vector from body i to body j
                let dx = pos_j.0 - pos_i.0;
                let dy = pos_j.1 - pos_i.1;

                // Distance squared with softening parameter to avoid singularities
                let r_sq = dx * dx + dy * dy + 1e-6;
                let r = r_sq.sqrt();

                // Unit vector from body i to body j
                let force_unit_x = dx / r;
                let force_unit_y = dy / r;

                // Acceleration on body i due to body j (G = 1 for simplicity)
                let accel_magnitude: f64 = self.masses[j] / r_sq;
                accelerations[body_i] += accel_magnitude * force_unit_x;
                accelerations[body_i + 1] += accel_magnitude * force_unit_y;
            }
        }

        // Velocity derivatives = acceleration
        (0..num_bodies).for_each(|i| {
            dydt[half_n + i * 2] = accelerations[i * 2];
            dydt[half_n + i * 2 + 1] = accelerations[i * 2 + 1];
        });
    }
}

struct MethodTester<M: OrdinaryNumericalMethod<f64, Vec<f64>>> {
    name: String,
    state: Vec<f64>,
    method_fn: fn(f64) -> M,
    method: M,
    data: Vec<Vec<Vec2>>,
}

impl<M> MethodTester<M>
where
    M: OrdinaryNumericalMethod<f64, Vec<f64>>,
{
    fn new(name: String, method_fn: fn(f64) -> M, state_vec: StateVec) -> Self {
        // Flatten the StateVec into a single Vec<f64> for the ODE solver
        let state = state_vec.to_vec();

        // Initialize the method to set it's initial state
        let mut method = (method_fn)(1.0);
        method
            .init(
                &GravitationalODE::new(vec![1.0; state_vec.positions.len() / 2]),
                0.0,
                1.0,
                &state,
            )
            .unwrap();

        Self {
            name,
            state,
            method_fn,
            method,
            data: vec![Vec::new(); state_vec.positions.len() / 2],
        }
    }

    fn update(&mut self, ode_system: &GravitationalODE, dt: f64, max_trail_length: usize) {
        let state = self.method.y().clone();

        self.method = (self.method_fn)(dt);
        self.method.init(&ode_system, 0.0, dt, &state).unwrap();
        self.method.step(&ode_system).unwrap();

        for i in 0..self.state.len() / 4 {
            let body_idx = i * 2;
            if self.method.y().is_empty() {
                continue;
            }
            self.data[i].push(vec2(
                self.method.y()[body_idx] as f32,
                self.method.y()[body_idx + 1] as f32,
            ));
            if self.data[i].len() > max_trail_length {
                self.data[i].remove(0);
            }
        }

        // if self.name == "RK4" {
        //     println!("Rk4 stages: {:#?}", self.method.stage_states().unwrap());
        // }
    }

    fn draw<C>(&self, draw: &Draw, scale: f32, color: C, offset: Vec2)
    where
        C: color::IntoLinSrgba<nannou::draw::properties::ColorScalar>,
    {
        if !self.data[0].is_empty() {
            let color = color.into_lin_srgba();

            for body_data in self.data.iter() {
                for t in 1..body_data.len() {
                    let alpha = t as f32 / body_data.len() as f32;
                    let color = srgba(color.red, color.green, color.blue, alpha);

                    draw.translate(offset.extend(0.0))
                        .line()
                        .start(body_data[t - 1])
                        .end(body_data[t])
                        .weight(1.0 / scale)
                        .color(color);
                }
            }

            let positions = self
                .method
                .y()
                .chunks(4)
                .map(|body| vec2(body[0] as f32, body[1] as f32))
                .collect::<Vec<Vec2>>();
            let com = positions.iter().sum::<Vec2>() / (self.method.y().len() as f32 / 4.0);

            draw.translate(offset.extend(0.0))
                .translate(com.extend(0.0))
                .text(&self.name);
        }
    }
}

struct Everything {
    colors: Vec<rgb::Rgb<color::encoding::Srgb, u8>>,
    euler: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 1, 1, 1>>,
    heun: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 2, 2, 2>>,
    ssp_rk3: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 3, 3, 3>>,
    rk4: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 4, 4, 4>>,
    rkv989e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, Vec<f64>, 9, 16, 26>>,
    vv: MethodTester<SymplecticIntegrator<Ordinary, Fixed, f64, Vec<f64>, 2>>,
    rf: MethodTester<SymplecticIntegrator<Ordinary, Fixed, f64, Vec<f64>, 4>>,
}

impl Everything {
    fn new<M>(system: &System<M>) -> Self
    where
        M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M, f64>,
    {
        let mut colors = vec![
            DARKVIOLET,
            DEEPPINK,
            DEEPSKYBLUE,
            DODGERBLUE,
            FIREBRICK,
            FLORALWHITE,
            FORESTGREEN,
            FUCHSIA,
            GAINSBORO,
            GHOSTWHITE,
            GOLD,
            GOLDENROD,
            GRAY,
            GREY,
            GREENYELLOW,
            HONEYDEW,
            HOTPINK,
            INDIANRED,
            IVORY,
            KHAKI,
            LAVENDER,
            LAVENDERBLUSH,
            LAWNGREEN,
        ];
        colors.shuffle(&mut rand::thread_rng());
        colors.truncate(19);

        let init_state = StateVec::from_system(system);

        let euler = MethodTester::new(
            "Euler".into(),
            ExplicitRungeKutta::euler,
            init_state.clone(),
        );
        let heun = MethodTester::new("Heun".into(), ExplicitRungeKutta::heun, init_state.clone());
        let ssp_rk3 = MethodTester::new(
            "SSP-RK3".into(),
            ExplicitRungeKutta::ssp_rk3,
            init_state.clone(),
        );
        let rk4 = MethodTester::new("RK4".into(), ExplicitRungeKutta::rk4, init_state.clone());
        let rkv989e = MethodTester::new(
            "RKV989E".into(),
            |dt: f64| {
                ExplicitRungeKutta::rkv989e()
                    .h_min(dt)
                    .h_max(dt)
                    .rtol(1e10)
                    .atol(1e10)
            },
            init_state.clone(),
        );
        let vv = MethodTester::new(
            "Velocity Verlet".into(),
            |dt: f64| SymplecticIntegrator::velocity_verlet(dt),
            init_state.clone(),
        );
        let rf = MethodTester::new(
            "Ruth Forest".into(),
            |dt: f64| SymplecticIntegrator::ruth_forest(dt),
            init_state.clone(),
        );

        Self {
            colors,
            euler,
            heun,
            ssp_rk3,
            rk4,
            rkv989e,
            vv,
            rf,
        }
    }

    fn update(&mut self, ode_system: &GravitationalODE, dt: f64, max_trail_length: usize) {
        self.euler.update(ode_system, dt, max_trail_length);
        self.heun.update(ode_system, dt, max_trail_length);
        self.ssp_rk3.update(ode_system, dt, max_trail_length);
        self.rk4.update(ode_system, dt, max_trail_length);
        self.rkv989e.update(ode_system, dt, max_trail_length);

        self.vv.update(ode_system, dt, max_trail_length);
        self.rf.update(ode_system, dt, max_trail_length);
    }

    fn draw(&self, draw: &Draw, scale: f32, offset_scale: f32) {
        self.euler
            .draw(draw, scale, self.colors[0], offset_scale * vec2(2.0, 0.0));
        self.heun
            .draw(draw, scale, self.colors[1], offset_scale * vec2(4.0, 0.0));
        self.ssp_rk3
            .draw(draw, scale, self.colors[4], offset_scale * vec2(2.0, -1.0));
        self.rk4
            .draw(draw, scale, self.colors[6], offset_scale * vec2(6.0, -1.0));
        self.rkv989e
            .draw(draw, scale, self.colors[18], offset_scale * vec2(6.0, -4.0));
        self.vv
            .draw(draw, scale, self.colors[2], offset_scale * vec2(2.0, -2.0));
        self.rf
            .draw(draw, scale, self.colors[3], offset_scale * vec2(4.0, -2.0));
    }
}

fn main() {
    nannou::app(model).update(update).event(event).run();
}

#[allow(dead_code)]
fn orbit_system() -> System<VV> {
    let mut system = System::new();

    let mut planet = Attractor::new(Vec2::ZERO, Vec2::ZERO, 500.0, 120.0);
    let mut moon = Attractor::new(vec2(50.0, 0.0), Vec2::ZERO, 200.0, 250.0);
    let annoyance = Attractor::new(vec2(-100.0, 0.0), vec2(0.0, 0.1), 500.0, 250.0);

    planet.orbit_pair(&mut moon, false);

    system.add_attractor(planet);
    system.add_attractor(moon);
    system.add_attractor(annoyance);

    system
}

#[allow(dead_code)]
fn figure_8() -> System<VV> {
    // (-1.0, 0.0), ( 0.3471168881,  0.5327249454)
    // ( 1.0, 0.0), ( 0.3471168881,  0.5327249454)
    // ( 0.0, 0.0), (-0.6942337762, -1.0654498908)

    let mut system = System::new();

    let b1 = Attractor::new(
        100.0 * vec2(-1.0, 0.0),
        0.1 * vec2(0.347_116_9, 0.532_724_9),
        1.0,
        0.0,
    );
    let b2 = Attractor::new(
        100.0 * vec2(1.0, 0.0),
        0.1 * vec2(0.347_116_9, 0.532_724_9),
        1.0,
        120.0,
    );
    let b3 = Attractor::new(
        100.0 * vec2(0.0, 0.0),
        0.1 * vec2(-0.694_233_8, -1.065_449_8),
        1.0,
        240.0,
    );
    system.add_attractor(b1);
    system.add_attractor(b2);
    system.add_attractor(b3);

    system
}

struct Model {
    ih: InteractionHandler,  // For user interaction
    system: System<VV>,      // Your original system
    everything: Everything,  // All the methods and data
    data: Vec<Vec<Vec2>>,    // Trail data for original system
    current_time: f64,       // Current simulation time
    max_trail_length: usize, // Limit trail length for performance
}

fn model(app: &App) -> Model {
    app.new_window().size(1200, 600).view(view).build().unwrap();

    let ih = InteractionHandler::from_rect(&app.window_rect())
        .set_dt(6.0)
        .set_scale(4.0);

    // Create system to be solved manually and by differential-equations crate
    let system = figure_8();

    // Figure 8

    // Set up differential-equations solver
    // let masses = system.get_masses();
    // let ode_system = GravitationalODE::new(masses);

    // // Solve ODE problem once for the entire time span
    // let ode_problem = IVP::ode(&ode_system, 0.0, 100000.0, initial_state);
    // let solver = ExplicitRungeKutta::rkv655e()
    //     // .h_max(model.dt)
    //     // .h_min(model.dt)
    //     .atol(1e-8)
    //     .rtol(1e-8)
    //     .max_steps(100000);
    // let ode_solution = ode_problem.method(solver).solve().unwrap();
    // println!("{:?}", ode_solution.status);
    // println!("{:?}", ode_solution.steps);
    // println!("{:?}", ode_solution.evals);
    // println!("{:?}", ode_solution.timer);

    let everything = Everything::new(&system);
    let num = system.get_attractors().len();

    Model {
        ih,
        system,
        everything,
        data: vec![Vec::new(); num],
        current_time: 0.0,
        max_trail_length: 100,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let queue = window.queue();
    let device = window.device();

    model
        .system
        .update(model.ih.dt, 5, Some(device), Some(queue));

    let masses = model
        .system
        .get_masses()
        .iter()
        .map(|&m| m as f64)
        .collect();
    let ode_system = GravitationalODE::new(masses);

    model
        .everything
        .update(&ode_system, model.ih.dt as f64, model.max_trail_length);

    for i in 0..model.data.len() {
        if let Some(body) = model.system.get_attractors().get(i) {
            model.data[i].push(vec2(body.position().x, body.position().y));
            if model.data[i].len() > model.max_trail_length {
                model.data[i].remove(0);
            }
        }
    }

    model.current_time += model.ih.dt;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = model.ih.draw(app.draw());
    draw.background().color(BLACK);

    // Draw trail from System
    if !model.data[0].is_empty() {
        for i in 0..model.data.len() {
            for t in 1..model.data[i].len() {
                let alpha = t as f32 / model.data[i].len() as f32;
                draw.line()
                    .start(model.data[i][t - 1])
                    .end(model.data[i][t])
                    .weight(1.0 / model.ih.scale)
                    .color(srgba(1.0, 0.0, 0.0, alpha));
            }
        }
    }
    let com = model
        .system
        .get_attractors()
        .iter()
        .map(|b| b.position())
        .fold(Vec2::ZERO, |acc, p| acc + p)
        / model.system.get_attractors().len() as f32;
    draw.translate(com.extend(0.0)).text("My RK4");

    model.everything.draw(&draw, model.ih.scale, 100.0);

    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: nannou::prelude::Event) {
    model.ih.custom_event_handler(app, event);
}
