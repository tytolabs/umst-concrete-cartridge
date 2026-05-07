pub mod core;
pub mod physics;

// Expose the core cartridge interface
pub use umst_manifold::core::{IScienceCartridge, MixTensor, PhysicalResult};
