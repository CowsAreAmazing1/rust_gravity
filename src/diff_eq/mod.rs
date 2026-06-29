pub mod gpuable;
pub mod not_gpuable;

use differential_equations::ode::{OrdinaryNumericalMethod, ODE};
use nannou::{
    prelude::*,
    wgpu::{Device, Queue},
};

use crate::{
    sim::{Body, System},
    GpuState,
};

pub trait MethodFn<M>
where
    M: OrdinaryNumericalMethod<f64, Vec<f64>>,
{
    fn method_fn() -> fn(f64) -> M;
}

pub trait AllowedMethod<M>: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M>
where
    M: OrdinaryNumericalMethod<f64, Vec<f64>> + MethodFn<M>,
{
    /// General, non-GPU update function for the system state using any specified numerical method.
    fn update(
        state: &State,
        ode: &GravitationalODE,
        dt: f64,
        sub_steps: u32,
        device: Option<&Device>,
        queue: Option<&Queue>,
        gpu_state: Option<&mut GpuState>,
    ) -> Vec<(Vec2, Vec2)> {
        let sub_dt = dt / sub_steps as f64;
        let mut method = (M::method_fn())(sub_dt);

        let y0 = &state.to_vec();
        method.init(ode, 0.0, dt, y0).unwrap();

        for _ in 0..sub_steps {
            // Main update step
            method.step(&ode).expect("step failed");

            if device.is_some() || queue.is_some() || gpu_state.is_some() {
                panic!("GPU state update not implemented for this integration method");
            }
        }

        let state = State::from_vec(method.y());
        state.out()
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

#[derive(Debug)]
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
        M: AllowedMethod<M>,
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

    fn from_vec(vec: &[f64]) -> Self {
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
