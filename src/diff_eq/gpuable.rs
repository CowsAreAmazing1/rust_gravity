use differential_equations::{
    methods::{Fixed, Ordinary, SymplecticIntegrator},
    ode::OrdinaryNumericalMethod,
};
use nannou::glam::Vec2;

use crate::{
    diff_eq::{AllowedMethod, GravitationalODE, MethodFn, State},
    GpuAttractor, GpuState,
};

// Velocity Verlet
pub type VV = SymplecticIntegrator<Ordinary, Fixed, f64, Vec<f64>, 2>;

impl MethodFn<VV> for VV {
    fn method_fn() -> fn(f64) -> VV {
        |dt| SymplecticIntegrator::velocity_verlet(dt)
    }
}

impl AllowedMethod<VV> for VV {
    /// Velocity Verlet update function for the system state using the SymplecticIntegrator method.
    /// Can update the GPU, if provided
    fn update(
        state: &State,
        ode: &GravitationalODE,
        dt: f64,
        sub_steps: u32,
        device: Option<&nannou::wgpu::Device>,
        queue: Option<&nannou::wgpu::Queue>,
        mut gpu_state: Option<&mut GpuState>,
    ) -> Vec<(Vec2, Vec2)> {
        let sub_dt = dt / sub_steps as f64;
        let mut method = (VV::method_fn())(sub_dt);

        let y0 = &state.to_vec();
        method.init(ode, 0.0, dt, y0).unwrap();

        for _ in 0..sub_steps {
            // Main update step
            method.step(&ode).expect("step failed");

            // Run compute shader for dust
            if let (Some(device), Some(queue)) = (device, queue) {
                if let Some(ref mut gpu_state) = gpu_state {
                    let positions = method
                        .stage_states()
                        .unwrap()
                        .iter()
                        .map(|vec| State::from_vec(vec).positions)
                        .collect::<Vec<Vec<f64>>>();

                    let attractors = (0..method.y().len() / 4) // y.len() / 4 = num bodies
                        .map(|body_idx| {
                            let stages = std::array::from_fn(|i| {
                                if let Some(pos) = positions.get(i) {
                                    let idx = body_idx * 2;
                                    [pos[idx] as f32, pos[idx + 1] as f32]
                                } else {
                                    [0.0, 0.0]
                                }
                            });

                            GpuAttractor::new(stages, ode.masses[body_idx] as f32)
                        })
                        .collect::<Vec<GpuAttractor>>();

                    gpu_state.update(sub_dt as f32, device, queue, &attractors);
                }
            }
        }

        let state = State::from_vec(method.y());
        state.out()
    }
}
