//! What differs between the desktop and the web.

#[cfg_attr(not(target_arch = "wasm32"), path = "desktop.rs")]
#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
mod imp;

pub(crate) use imp::*;
