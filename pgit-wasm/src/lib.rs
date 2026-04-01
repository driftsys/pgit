#[cfg(target_arch = "wasm32")]
mod bindings;

#[cfg(target_arch = "wasm32")]
pub use bindings::*;
