//! What differs between the desktop and the web.

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use desktop::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use web::*;
