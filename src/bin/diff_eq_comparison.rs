use differential_equations::prelude::*;
use main_gravity::prelude::*;
use nalgebra::SVector;
use nannou::prelude::*;

// For a 2-body system, we need 8 state variables: [x1, y1, vx1, vy1, x2, y2, vx2, vy2]
const STATE_SIZE: usize = 8;
// type StateVec = SVector<f64, STATE_SIZE>;

// Fiddle with the `State` derive to get const array sized fields working. Then generalize to n bodies
const OBJECTS: usize = 3;
#[derive(State)]
struct StateVec<T> {
    positions: [T; OBJECTS],
    velocities: [T; OBJECTS],
}

// Define the gravitational system for the differential equations solver
#[derive(Clone)]
struct GravitationalODE {
    masses: [f64; 2], // Fixed to 2 bodies for now (planet and moon)
}

impl GravitationalODE {
    fn new(mass1: f64, mass2: f64) -> Self {
        Self {
            masses: [mass1, mass2],
        }
    }
}

// Implement the ODE trait for our gravitational system
// State vector format: [x1, y1, vx1, vy1, x2, y2, vx2, vy2]
impl ODE<f64, StateVec<f64>> for GravitationalODE {
    fn diff(&self, _t: f64, y: &StateVec<f64>, dydt: &mut StateVec<f64>) {
        // Body 1 (index 0-3): x1, y1, vx1, vy1
        // Body 2 (index 4-7): x2, y2, vx2, vy2

        // Position derivatives = velocity
        dydt[0] = y[2]; // dx1/dt = vx1
        dydt[1] = y[3]; // dy1/dt = vy1
        dydt[4] = y[6]; // dx2/dt = vx2
        dydt[5] = y[7]; // dy2/dt = vy2

        // Calculate relative position vector from body 1 to body 2
        let dx = y[4] - y[0]; // x2 - x1
        let dy = y[5] - y[1]; // y2 - y1

        // Distance squared with softening parameter to avoid singularities
        let r_sq = dx * dx + dy * dy + 1e-6;
        let r = r_sq.sqrt();

        // Gravitational force magnitude (G = 1 for simplicity)
        let force_unit_x = dx / r;
        let force_unit_y = dy / r;

        // Acceleration on body 1 due to body 2
        let a1_x = self.masses[1] * force_unit_x / r_sq;
        let a1_y = self.masses[1] * force_unit_y / r_sq;

        // Acceleration on body 2 due to body 1 (Newton's 3rd law)
        let a2_x = -self.masses[0] * force_unit_x / r_sq;
        let a2_y = -self.masses[0] * force_unit_y / r_sq;

        // Velocity derivatives = acceleration
        dydt[2] = a1_x; // dvx1/dt = ax1
        dydt[3] = a1_y; // dvy1/dt = ay1
        dydt[6] = a2_x; // dvx2/dt = ax2
        dydt[7] = a2_y; // dvy2/dt = ay2
    }
}

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    system: System,                        // Your original system
    ode_solution: Solution<f64, StateVec>, // Pre-computed ODE solution
    current_time: f64,                     // Current simulation time
    dt: f64,                               // Time step for comparison
    data: Vec<Vec2>,                       // Trail for your RK4 system
    comparison_data: Vec<Vec2>,            // Trail for ODE solver
    show_comparison: bool,                 // Toggle between systems
    max_trail_length: usize,               // Limit trail length for performance
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 600)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let mut system = System::new();

    // Create bodies with same initial conditions for both systems
    let planet = Attractor::new(vec2(0.0, 0.0), vec2(0.0, 0.0), 500.0, 120.0);
    system.add_attractor(planet);

    let moon = Attractor::new(
        vec2(150.0, 0.0),
        vec2(0.0, (500.0 / 150.0).sqrt()),
        10.0,
        250.0,
    );
    system.add_attractor(moon);

    // Set up ODE system with same initial conditions
    let ode_system = GravitationalODE::new(500.0, 10.0); // planet mass, moon mass

    // Initial state: [x1, y1, vx1, vy1, x2, y2, vx2, vy2]
    let initial_state = StateVec::from_vec(vec![
        0.0,
        0.0,
        0.0,
        0.0, // planet
        150.0,
        0.0,
        0.0,
        (500.0 / 150.0).sqrt(), // moon
    ]);

    // Solve ODE problem once for the entire time span
    let ode_problem = ODEProblem::new(&ode_system, 0.0, 1000.0, initial_state);
    let mut solver = ExplicitRungeKutta::rkv655e()
        .atol(1e-8)
        .rtol(1e-8)
        .max_steps(100000);
    let ode_solution = ode_problem.solve(&mut solver).unwrap();

    Model {
        system,
        ode_solution,
        current_time: 0.0,
        dt: 0.5,
        data: Vec::new(),
        comparison_data: Vec::new(),
        show_comparison: false,
        max_trail_length: 1000,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => model.show_comparison = !model.show_comparison,
        Key::R => {
            // Reset both systems
            model.data.clear();
            model.comparison_data.clear();
            model.current_time = 0.0;

            // Reset your system
            model.system = System::new();
            let planet = Attractor::new(vec2(0.0, 0.0), vec2(0.0, 0.0), 500.0, 120.0);
            model.system.add_attractor(planet);
            let moon = Attractor::new(
                vec2(150.0, 0.0),
                vec2(0.0, (500.0 / 150.0).sqrt()),
                10.0,
                250.0,
            );
            model.system.add_attractor(moon);

            // Reset ODE solution
            let ode_system = GravitationalODE::new(500.0, 10.0);
            let initial_state = StateVec::from_vec(vec![
                0.0,
                0.0,
                0.0,
                0.0,
                150.0,
                0.0,
                0.0,
                (500.0 / 150.0).sqrt(),
            ]);
            let ode_problem = ODEProblem::new(&ode_system, 0.0, 1000.0, initial_state);
            let mut solver = ExplicitRungeKutta::rkv989e() // rkv655e
                .atol(1e-8)
                .rtol(1e-8)
                .max_steps(100000);
            model.ode_solution = ode_problem.solve(&mut solver).unwrap();
        }
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let queue = window.queue();
    let device = window.device();

    model
        .system
        .update(model.dt as f32, 5, Some(device), Some(queue));

    // Add current positions to trail (moon only for clarity)
    if let Some(moon) = model.system.get_bodies().get(1) {
        model.data.push(vec2(moon.position().x, moon.position().y));
        if model.data.len() > model.max_trail_length {
            model.data.remove(0);
        }
    }

    // Get corresponding position from ODE solution
    if let Some((_, state)) = model
        .ode_solution
        .iter()
        .find(|(t, _)| (*t - model.current_time).abs() < 1.0)
    {
        // Moon position is at indices 4,5
        model
            .comparison_data
            .push(vec2(state[4] as f32, state[5] as f32));
        if model.comparison_data.len() > model.max_trail_length {
            model.comparison_data.remove(0);
        }
    }

    model.current_time += model.dt;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    if model.show_comparison {
        // Show ODE solver results

        // Draw trail from ODE solver
        if model.comparison_data.len() > 1 {
            for i in 1..model.comparison_data.len() {
                let alpha = i as f32 / model.comparison_data.len() as f32;
                draw.line()
                    .start(model.comparison_data[i - 1])
                    .end(model.comparison_data[i])
                    .weight(1.0)
                    .color(srgba(0.0, 1.0, 0.0, alpha));
            }
        }

        // Get current positions from ODE solution
        if let Some((_, state)) = model
            .ode_solution
            .iter()
            .find(|(t, _)| (*t - model.current_time).abs() < 1.0)
        {
            // Draw planet (body 1)
            draw.ellipse()
                .xy(vec2(state[0] as f32, state[1] as f32))
                .radius((500.0_f32).sqrt() * 0.5)
                .color(BLUE);

            // Draw moon (body 2)
            draw.ellipse()
                .xy(vec2(state[4] as f32, state[5] as f32))
                .radius((10.0_f32).sqrt() * 0.5)
                .color(GREEN);
        }

        // Display text
        draw.text("ODE Solver (Green)")
            .xy(vec2(-350.0, 250.0))
            .color(GREEN)
            .font_size(20);
    } else {
        // Show your original system
        let window = app.main_window();
        let device = window.device();
        let queue = window.queue();
        let texture_view = frame.texture_view();
        model.system.draw(&draw, device, queue, texture_view, 1.0);

        // Draw trail from your RK4 system
        if model.data.len() > 1 {
            for i in 1..model.data.len() {
                let alpha = i as f32 / model.data.len() as f32;
                draw.line()
                    .start(model.data[i - 1])
                    .end(model.data[i])
                    .weight(1.0)
                    .color(srgba(1.0, 0.0, 0.0, alpha));
            }
        }

        // Display text
        draw.text("Your RK4 System (Red)")
            .xy(vec2(-350.0, 250.0))
            .color(RED)
            .font_size(20);
    }

    // Instructions
    draw.text("Press SPACE to toggle comparison")
        .xy(vec2(-350.0, -250.0))
        .color(WHITE)
        .font_size(16);

    draw.text("Press R to reset")
        .xy(vec2(-350.0, -270.0))
        .color(WHITE)
        .font_size(16);

    draw.to_frame(app, &frame).unwrap();
}
