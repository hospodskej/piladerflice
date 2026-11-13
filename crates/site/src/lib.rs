//! The Pila Derflice website. Pages are rendered on the server; the
//! interactive parts are islands that hydrate in the browser as WASM.

#![recursion_limit = "512"]

pub mod app;
pub mod assets;
#[cfg(feature = "ssr")]
pub mod state;
