// lib.rs - Main library file
// This file makes your crate available as a library

pub mod diff_eq;
pub mod gpu;
pub mod scene_layout;
pub mod sim;
pub mod utils;

// Re-export commonly used items for convenience
pub use gpu::*;
// pub use scene_layout::*;
// pub use sim::*;

// /// A small prelude of commonly used traits/types for convenience in binaries.
// pub mod prelude {
//     pub use crate::scene_layout::{FillWithDust, SetupObject};
//     pub use crate::sim::{
//         sun_planet_binary_ccw, sun_planet_binary_cw, Attractor, Body, Dust, System,
//     };
//     pub use crate::utils::InteractionHandler;
// }

pub mod prelude {
    pub use crate::{
        diff_eq::{
            gpuable::VV,
            not_gpuable::{DOP853, EULER, RK4, SSPRK3},
            AllowedMethod,
        },
        scene_layout::{Disc, Quad, Setup, SetupObject},
        sim::{sun_planet_binary_ccw, Attractor, Body, Dust, System},
        utils::InteractionHandler,
    };
}
