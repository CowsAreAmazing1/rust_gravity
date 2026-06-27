use differential_equations::{
    methods::{Fixed, Ordinary, SymplecticIntegrator},
    ode::OrdinaryNumericalMethod,
};
use nannou::glam::Vec2;

use crate::{
    diff_eq::{AllowedMethod, GravitationalODE, MethodFn, State},
    GpuState,
};

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

        #[allow(clippy::never_loop)]
        for _ in 0..sub_steps {
            // Main update step
            method.step(&ode).expect("step failed");

            unimplemented!("get this working + figure out how to make GpuAttractors / how to handle states of type Vec<f64> with the gpu");

            // let stages = method
            //     .stage_states()
            //     .unwrap()
            //     .iter()
            //     .map(|vec| State::from_vec(vec))
            //     .collect::<Vec<State>>();

            // println!(
            //     "y:      {:?}",
            //     method.y().iter().take(2).collect::<Vec<&f64>>()
            // );
            // println!(
            //     "stages: {:?}",
            //     stages[0].positions.iter().take(2).collect::<Vec<&f64>>()
            // );

            // println!("{:?}", method.stage_states().unwrap().len());

            // // Run compute shader for dust
            // if let (Some(device), Some(queue)) = (device, queue) {
            //     let attractors = State::vec_to_attractors(&method.y(), &ode.masses);
            //     if let Some(ref mut gpu_state) = gpu_state {
            //         gpu_state.update(sub_dt as f32, device, queue, &attractors);
            //     }
            // }
        }

        let state = State::from_vec(method.y());
        state.out()
    }
}
