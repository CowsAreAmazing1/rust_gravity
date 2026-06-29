use differential_equations::methods::{DormandPrince, ExplicitRungeKutta, Fixed, Ordinary};

use crate::diff_eq::{AllowedMethod, MethodFn};

// Euler
pub type EULER = ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 1, 1, 1>;

impl MethodFn<EULER> for EULER {
    fn method_fn() -> fn(f64) -> EULER {
        |dt| ExplicitRungeKutta::euler(dt)
        // |dt| ExplicitRungeKutta::dop853().h_min(dt).h_max(dt)
    }
}

impl AllowedMethod<EULER> for EULER {}

// Strong Stability Preserving RK3
pub type SSPRK3 = ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 3, 3, 3>;

impl MethodFn<SSPRK3> for SSPRK3 {
    fn method_fn() -> fn(f64) -> SSPRK3 {
        |dt| ExplicitRungeKutta::ssp_rk3(dt)
    }
}

impl AllowedMethod<SSPRK3> for SSPRK3 {}

// RK4
pub type RK4 = ExplicitRungeKutta<Ordinary, Fixed, f64, Vec<f64>, 4, 4, 4>;

impl MethodFn<RK4> for RK4 {
    fn method_fn() -> fn(f64) -> RK4 {
        |dt| ExplicitRungeKutta::rk4(dt)
    }
}

impl AllowedMethod<RK4> for RK4 {}

// DOP853
pub type DOP853 = ExplicitRungeKutta<Ordinary, DormandPrince, f64, Vec<f64>, 8, 12, 16>;

impl MethodFn<DOP853> for DOP853 {
    fn method_fn() -> fn(f64) -> DOP853 {
        |dt| ExplicitRungeKutta::dop853().h_max(dt).atol(1e10).rtol(1e10)
    }
}
impl AllowedMethod<DOP853> for DOP853 {}
