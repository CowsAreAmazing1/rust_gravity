use differential_equations::{
    methods::{Adaptive, DormandPrince, Fixed, Ordinary},
    ode::OrdinaryNumericalMethod,
    prelude::*,
};
use main_gravity::prelude::*;
use nannou::{
    color,
    prelude::*,
    rand::{self, seq::SliceRandom},
};

macro_rules! make_state {
    ($n:literal) => {
        #[derive(State)]
        struct StateVec {
            positions: [f64; $n],
            velocities: [f64; $n],
        }
    };
}

// positions:  [x1,  y1,  x2,  y2,  ...]
// velocities: [vx1, vy1, vx2, vy2, ...]
const STATE_SIZE: usize = 6;
make_state!(6);

impl StateVec {
    fn from_system<M>(system: &System<M>) -> Self
    where
        M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M, f64>,
    {
        let mut positions = [0.0; STATE_SIZE];
        let mut velocities = [0.0; STATE_SIZE];

        for (i, body) in system.get_bodies().iter().enumerate() {
            positions[i * 2] = body.position().x as f64;
            positions[i * 2 + 1] = body.position().y as f64;
            velocities[i * 2] = body.velocity().x as f64;
            velocities[i * 2 + 1] = body.velocity().y as f64;
        }

        Self {
            positions,
            velocities,
        }
    }

    fn _draw(&self, draw: &Draw, masses: &[f64]) {
        for (i, mass) in masses.iter().enumerate() {
            let pos_idx = i * 2;
            draw.ellipse()
                .xy(vec2(
                    self.positions[pos_idx] as f32,
                    self.positions[pos_idx + 1] as f32,
                ))
                .radius((*mass as f32).sqrt() * 0.5)
                .color(if i == 0 { BLUE } else { GREEN });
        }
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
impl ODE<f64, StateVec> for GravitationalODE {
    fn diff(&self, _t: f64, y: &StateVec, dydt: &mut StateVec) {
        // Position derivatives = velocity
        for i in 0..STATE_SIZE {
            dydt.positions[i] = y.velocities[i];
        }

        // Initialize accelerations to zero
        let mut accelerations = [0.0; STATE_SIZE];

        // Compute pairwise gravitational accelerations for all bodies
        for body_i in 0..STATE_SIZE / 2 {
            // Extract position and velocity indices (2D: x, y for each body)
            let pos_i_idx = body_i * 2;
            let pos_i = (y.positions[pos_i_idx], y.positions[pos_i_idx + 1]);

            for body_j in 0..STATE_SIZE / 2 {
                if body_i == body_j {
                    continue; // Skip self-interaction
                }

                let pos_j_idx = body_j * 2;
                let pos_j = (y.positions[pos_j_idx], y.positions[pos_j_idx + 1]);

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
                let accel_magnitude = self.masses[body_j] / r_sq;
                accelerations[pos_i_idx] += accel_magnitude * force_unit_x;
                accelerations[pos_i_idx + 1] += accel_magnitude * force_unit_y;
            }
        }

        // Velocity derivatives = acceleration
        dydt.velocities[..STATE_SIZE].copy_from_slice(&accelerations[..STATE_SIZE]);
    }
}

struct MethodTester<M: OrdinaryNumericalMethod<f64, StateVec>> {
    name: String,
    state: StateVec,
    method_fn: fn(f64) -> M,
    method: M,
    data: [Vec<Vec2>; STATE_SIZE / 2],
}

impl<M> MethodTester<M>
where
    M: OrdinaryNumericalMethod<f64, StateVec>,
{
    fn new(name: String, method_fn: fn(f64) -> M, state: StateVec) -> Self {
        Self {
            name,
            state,
            method_fn,
            method: (method_fn)(1.0),
            data: Default::default(),
        }
    }

    fn update(&mut self, ode_system: &GravitationalODE, dt: f64, max_trail_length: usize) {
        self.method = (self.method_fn)(dt);
        self.method.init(&ode_system, 0.0, dt, &self.state).unwrap();
        self.method.step(&ode_system).unwrap();

        let new_state = *self.method.y();
        self.state = new_state;

        for i in 0..STATE_SIZE / 2 {
            let pos_idx = i * 2;
            self.data[i].push(vec2(
                self.state.positions[pos_idx] as f32,
                self.state.positions[pos_idx + 1] as f32,
            ));
            if self.data[i].len() > max_trail_length {
                self.data[i].remove(0);
            }
        }

        if self.name == "RK4" {
            println!("Rk4 stages: {:#?}", self.method.stage_states().unwrap());
        }
    }

    fn draw<C>(&self, draw: &Draw, scale: f32, color: C, offset: Vec2)
    where
        C: color::IntoLinSrgba<nannou::draw::properties::ColorScalar>,
    {
        if !self.data[0].is_empty() {
            let color = color.into_lin_srgba();

            for i in 0..STATE_SIZE / 2 {
                for t in 1..self.data[i].len() {
                    let alpha = t as f32 / self.data[i].len() as f32;
                    let color = srgba(color.red, color.green, color.blue, alpha);

                    draw.translate(offset.extend(0.0))
                        .line()
                        .start(self.data[i][t - 1])
                        .end(self.data[i][t])
                        .weight(1.0 / scale)
                        .color(color);
                }
            }

            let com = (0..STATE_SIZE / 2)
                .map(|i| self.data[i].last().unwrap())
                .sum::<Vec2>()
                / (STATE_SIZE as f32 / 2.0);
            draw.translate(offset.extend(0.0))
                .translate(com.extend(0.0))
                .text(&self.name);
        }
    }
}

struct Everything {
    colors: Vec<rgb::Rgb<color::encoding::Srgb, u8>>,
    euler: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 1, 1, 1>>,
    heun: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 2, 2, 2>>,
    midpoint: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 2, 2, 2>>,
    ralston: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 2, 2, 2>>,
    ssp_rk3: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 3, 3, 3>>,
    three_eighths: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 4, 4, 4>>,
    rk4: MethodTester<ExplicitRungeKutta<Ordinary, Fixed, f64, StateVec, 4, 4, 4>>,
    cash_karp: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 5, 6, 6>>,
    dop853: MethodTester<ExplicitRungeKutta<Ordinary, DormandPrince, f64, StateVec, 8, 12, 16>>,
    dopri5: MethodTester<ExplicitRungeKutta<Ordinary, DormandPrince, f64, StateVec, 5, 7, 7>>,
    rkf45: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 5, 6, 6>>,
    rkv655e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 6, 9, 10>>,
    rkv656e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 6, 9, 12>>,
    rkv766e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 7, 10, 13>>,
    rkv767e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 7, 10, 16>>,
    rkv877e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 8, 13, 17>>,
    rkv878e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 8, 13, 21>>,
    rkv988e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 9, 16, 21>>,
    rkv989e: MethodTester<ExplicitRungeKutta<Ordinary, Adaptive, f64, StateVec, 9, 16, 26>>,
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
            GREEN,
            GREENYELLOW,
            HONEYDEW,
            HOTPINK,
            INDIANRED,
            INDIGO,
            IVORY,
            KHAKI,
            LAVENDER,
            LAVENDERBLUSH,
            LAWNGREEN,
        ];
        colors.shuffle(&mut rand::thread_rng());
        colors.truncate(19);

        let init_state = StateVec::from_system(system);

        let euler = MethodTester::new("Euler".into(), ExplicitRungeKutta::euler, init_state);
        let heun = MethodTester::new("Heun".into(), ExplicitRungeKutta::heun, init_state);
        let midpoint =
            MethodTester::new("Midpoint".into(), ExplicitRungeKutta::midpoint, init_state);
        let ralston = MethodTester::new("Ralston".into(), ExplicitRungeKutta::ralston, init_state);
        let ssp_rk3 = MethodTester::new("SSP-RK3".into(), ExplicitRungeKutta::ssp_rk3, init_state);
        let three_eighths = MethodTester::new(
            "Three-Eighths".into(),
            ExplicitRungeKutta::three_eighths,
            init_state,
        );
        let rk4 = MethodTester::new("RK4".into(), ExplicitRungeKutta::rk4, init_state);
        let cash_karp = MethodTester::new(
            "Cash-Karp".into(),
            |dt: f64| ExplicitRungeKutta::cash_karp().h_min(dt).h_max(dt),
            init_state,
        );
        let dop853 = MethodTester::new(
            "Dop853".into(),
            |dt: f64| ExplicitRungeKutta::dop853().h_min(dt).h_max(dt),
            init_state,
        );
        let dopri5 = MethodTester::new(
            "Dopri5".into(),
            |dt: f64| ExplicitRungeKutta::dopri5().h_min(dt).h_max(dt),
            init_state,
        );
        let rkf45 = MethodTester::new(
            "RKF45".into(),
            |dt: f64| ExplicitRungeKutta::rkf45().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv655e = MethodTester::new(
            "RKV655E".into(),
            |dt: f64| ExplicitRungeKutta::rkv655e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv656e = MethodTester::new(
            "RKV656E".into(),
            |dt: f64| ExplicitRungeKutta::rkv656e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv766e = MethodTester::new(
            "RKV766E".into(),
            |dt: f64| ExplicitRungeKutta::rkv766e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv767e = MethodTester::new(
            "RKV767E".into(),
            |dt: f64| ExplicitRungeKutta::rkv767e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv877e = MethodTester::new(
            "RKV877E".into(),
            |dt: f64| ExplicitRungeKutta::rkv877e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv878e = MethodTester::new(
            "RKV878E".into(),
            |dt: f64| ExplicitRungeKutta::rkv878e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv988e = MethodTester::new(
            "RKV988E".into(),
            |dt: f64| ExplicitRungeKutta::rkv988e().h_min(dt).h_max(dt),
            init_state,
        );
        let rkv989e = MethodTester::new(
            "RKV989E".into(),
            |dt: f64| ExplicitRungeKutta::rkv989e().h_min(dt).h_max(dt),
            init_state,
        );

        Self {
            colors,
            euler,
            heun,
            midpoint,
            ralston,
            ssp_rk3,
            three_eighths,
            rk4,
            cash_karp,
            dop853,
            dopri5,
            rkf45,
            rkv655e,
            rkv656e,
            rkv766e,
            rkv767e,
            rkv877e,
            rkv878e,
            rkv988e,
            rkv989e,
        }
    }

    fn update(&mut self, ode_system: &GravitationalODE, dt: f64, max_trail_length: usize) {
        self.euler.update(ode_system, dt, max_trail_length);
        self.heun.update(ode_system, dt, max_trail_length);
        self.midpoint.update(ode_system, dt, max_trail_length);
        self.ralston.update(ode_system, dt, max_trail_length);
        self.ssp_rk3.update(ode_system, dt, max_trail_length);
        self.three_eighths.update(ode_system, dt, max_trail_length);
        self.rk4.update(ode_system, dt, max_trail_length);
        self.cash_karp.update(ode_system, dt, max_trail_length);
        self.dop853.update(ode_system, dt, max_trail_length);
        self.dopri5.update(ode_system, dt, max_trail_length);
        self.rkf45.update(ode_system, dt, max_trail_length);
        self.rkv655e.update(ode_system, dt, max_trail_length);
        self.rkv656e.update(ode_system, dt, max_trail_length);
        self.rkv766e.update(ode_system, dt, max_trail_length);
        self.rkv767e.update(ode_system, dt, max_trail_length);
        self.rkv877e.update(ode_system, dt, max_trail_length);
        self.rkv878e.update(ode_system, dt, max_trail_length);
        self.rkv988e.update(ode_system, dt, max_trail_length);
        self.rkv989e.update(ode_system, dt, max_trail_length);
    }

    fn draw(&self, draw: &Draw, scale: f32, offset_scale: f32) {
        self.euler
            .draw(draw, scale, self.colors[0], offset_scale * vec2(2.0, 0.0));
        self.heun
            .draw(draw, scale, self.colors[1], offset_scale * vec2(4.0, 0.0));
        self.midpoint
            .draw(draw, scale, self.colors[2], offset_scale * vec2(6.0, 0.0));
        self.ralston
            .draw(draw, scale, self.colors[3], offset_scale * vec2(8.0, 0.0));
        self.ssp_rk3
            .draw(draw, scale, self.colors[4], offset_scale * vec2(2.0, -1.0));
        self.three_eighths
            .draw(draw, scale, self.colors[5], offset_scale * vec2(4.0, -1.0));
        self.rk4
            .draw(draw, scale, self.colors[6], offset_scale * vec2(6.0, -1.0));
        self.cash_karp
            .draw(draw, scale, self.colors[7], offset_scale * vec2(8.0, -1.0));
        self.dop853
            .draw(draw, scale, self.colors[8], offset_scale * vec2(2.0, -2.0));
        self.dopri5
            .draw(draw, scale, self.colors[9], offset_scale * vec2(4.0, -2.0));
        self.rkf45
            .draw(draw, scale, self.colors[10], offset_scale * vec2(6.0, -2.0));
        self.rkv655e
            .draw(draw, scale, self.colors[11], offset_scale * vec2(8.0, -2.0));
        self.rkv656e
            .draw(draw, scale, self.colors[12], offset_scale * vec2(2.0, -3.0));
        self.rkv766e
            .draw(draw, scale, self.colors[13], offset_scale * vec2(4.0, -3.0));
        self.rkv767e
            .draw(draw, scale, self.colors[14], offset_scale * vec2(6.0, -3.0));
        self.rkv877e
            .draw(draw, scale, self.colors[15], offset_scale * vec2(8.0, -3.0));
        self.rkv878e
            .draw(draw, scale, self.colors[16], offset_scale * vec2(2.0, -4.0));
        self.rkv988e
            .draw(draw, scale, self.colors[17], offset_scale * vec2(4.0, -4.0));
        self.rkv989e
            .draw(draw, scale, self.colors[18], offset_scale * vec2(6.0, -4.0));
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
    ih: InteractionHandler,            // For user interaction
    system: System<VV>,                // Your original system
    everything: Everything,            // All the methods and data
    data: [Vec<Vec2>; STATE_SIZE / 2], // Trail data for original system
    current_time: f64,                 // Current simulation time
    max_trail_length: usize,           // Limit trail length for performance
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

    Model {
        ih,
        system,
        everything,
        data: std::array::from_fn(|_| Vec::new()),
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

    for i in 0..STATE_SIZE / 2 {
        if let Some(moon) = model.system.get_bodies().get(i) {
            model.data[i].push(vec2(moon.position().x, moon.position().y));
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
        for i in 0..STATE_SIZE / 2 {
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
        .get_bodies()
        .iter()
        .map(|b| b.position())
        .fold(Vec2::ZERO, |acc, p| acc + p)
        / model.system.get_bodies().len() as f32;
    draw.translate(com.extend(0.0)).text("My RK4");

    model.everything.draw(&draw, model.ih.scale, 100.0);

    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: nannou::prelude::Event) {
    model.ih.custom_event_handler(app, event);
}
