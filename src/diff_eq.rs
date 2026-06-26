use differential_equations::{
    methods::{ExplicitRungeKutta, Fixed, Ordinary, SymplecticIntegrator},
    ode::{OrdinaryNumericalMethod, ODE},
    traits::Real,
};
use nannou::prelude::*;

use crate::sim::{Body, System};

pub type VV = SymplecticIntegrator<Ordinary, Fixed, f64, Vec<f64>, 2>;
pub type RK4 = ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 4, 4, 4>;

pub trait MethodFn<M, T>
where
    M: OrdinaryNumericalMethod<T, Vec<T>>,
    T: Real,
{
    fn method_fn() -> fn(T) -> M;
}

impl<T> MethodFn<SymplecticIntegrator<Ordinary, Fixed, T, Vec<T>, 2>, T>
    for SymplecticIntegrator<Ordinary, Fixed, T, Vec<T>, 2>
where
    T: Real,
{
    fn method_fn() -> fn(T) -> SymplecticIntegrator<Ordinary, Fixed, T, Vec<T>, 2> {
        |dt| SymplecticIntegrator::velocity_verlet(dt)
    }
}

impl<T> MethodFn<ExplicitRungeKutta<Ordinary, Fixed, T, Vec<T>, 4, 4, 4>, T>
    for ExplicitRungeKutta<Ordinary, Fixed, T, Vec<T>, 4, 4, 4>
where
    T: Real,
{
    fn method_fn() -> fn(T) -> ExplicitRungeKutta<Ordinary, Fixed, T, Vec<T>, 4, 4, 4> {
        |dt| ExplicitRungeKutta::rk4(dt)
    }
}

// Define the gravitational system for the differential equations solver
#[derive(Clone)]
pub struct GravitationalODE {
    masses: Vec<f64>,
}

impl GravitationalODE {
    pub fn new(masses: Vec<f64>) -> Self {
        Self { masses }
    }
}

// Implements the ODE for the gravitational system
// -> vec![x1, y1, x2, y2, ... , vx1, vy1, vx2, vy2, ...] for each body in the system
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

#[derive(Clone)]
pub struct State {
    positions: Vec<f64>,
    velocities: Vec<f64>,
}

impl State {
    pub fn empty() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    pub fn from_system<M>(system: &System<M>) -> Self
    where
        M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M, f64>,
    {
        let mut positions = Vec::new();
        let mut velocities = Vec::new();

        for body in system.get_bodies().iter() {
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

    /// -> vec![x1, y1, x2, y2, ... , vx1, vy1, vx2, vy2, ...] for each body in the system
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

    fn from_vec(vec: &Vec<f64>) -> Self {
        Self {
            positions: vec[..vec.len() / 2].to_vec(),
            velocities: vec[vec.len() / 2..].to_vec(),
        }
    }

    fn out(&self) -> Vec<(Vec2, Vec2)> {
        let mut out = Vec::new();
        for i in 0..self.positions.len() / 2 {
            out.push((
                vec2(
                    self.positions[i * 2] as f32,
                    self.positions[i * 2 + 1] as f32,
                ),
                vec2(
                    self.velocities[i * 2] as f32,
                    self.velocities[i * 2 + 1] as f32,
                ),
            ));
        }
        out
    }
}

/// Handles numerical integration of the ODE by adapting a crate::System to work with differential_equations.
/// Responsible for integrating the system by a single timestep
pub struct DiffEq<M>
where
    M: OrdinaryNumericalMethod<f64, Vec<f64>>,
{
    method: Option<M>,
    stages: Vec<Vec<f64>>,
}

impl<M> DiffEq<M>
where
    M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M, f64>,
{
    pub fn new() -> Self {
        Self {
            method: None,
            stages: Vec::new(),
        }
    }

    // pub fn init(&mut self, dt: f64, state: &State, ode: &GravitationalODE) {

    // }

    pub fn update(
        &mut self,
        state: &State,
        ode: &GravitationalODE,
        dt: f64,
        sub_steps: u32,
    ) -> Vec<(Vec2, Vec2)> {
        let mut method = (M::method_fn())(dt / sub_steps as f64);

        let y0 = &state.to_vec();
        method.init(ode, 0.0, dt, y0).unwrap();
        // self.method = Some(method);

        for _ in 0..sub_steps {
            method.step(&ode).unwrap();
            self.stages = method.stage_states().unwrap().into();
        }

        let state = State::from_vec(&method.y());
        self.method = Some(method);
        state.out()
    }

    // fn sub_update(&mut self, &mut method: M, ode: &GravitationalODE) -M {

    // if let Some(ref mut method) = self.method {
    //     method.step(&ode).unwrap();
    //     self.stages = method.stage_states().unwrap().into();
    // }
    // }
}
