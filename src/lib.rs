// lib.rs - Main library file
// This file makes your crate available as a library

pub mod sim;
pub mod gpu;
pub mod scene_layout;

// Re-export commonly used items for convenience
pub use sim::*;
pub use gpu::*;
pub use scene_layout::*;

/// A small prelude of commonly used traits/types for convenience in binaries.
pub mod prelude {
	pub use crate::scene_layout::{SetupObject, FillWithDust};
}