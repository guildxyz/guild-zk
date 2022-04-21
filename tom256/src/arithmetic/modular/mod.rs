#[cfg_attr(target_pointer_width = "64", path = "pointer_u64.rs")]
#[cfg_attr(not(target_pointer_width = "64"), path = "pointer_u32.rs")]
mod modular_trait;

pub use modular_trait::Modular;
