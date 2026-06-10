//! Control chrome recipe helpers built on shared theme/style resolution.
//!
//! This module owns the generic control fallback policy that higher-level
//! recipes compose for buttons, inputs, and list-like controls.

mod chrome;

pub use chrome::{
    ControlFallbacks, ControlTokenKeys, ResolvedControlChrome, resolve_control_chrome,
};
