//! Material Design 3 (and Expressive) component surface for Fret.
//!
//! This crate targets **visual + interaction outcome alignment** with the Material 3 design
//! system, while keeping `crates/fret-ui` focused on mechanisms (not Material-specific policy).

#![forbid(unsafe_code)]

pub mod button;
pub mod icon_button;
pub mod interaction;
pub mod motion;
pub mod theme;
pub mod tokens;

pub use button::{Button, ButtonVariant};
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
