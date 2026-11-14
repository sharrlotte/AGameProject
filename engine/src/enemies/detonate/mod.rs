pub mod state;
pub mod system;

pub use state::{Detonated, Fuse};
pub use system::{blast_falloff, resolve_detonations, update_fuses};

#[cfg(test)]
mod tests;
