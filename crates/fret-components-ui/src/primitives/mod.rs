//! Radix-aligned primitives (headless building blocks).
//!
//! This module exists to keep the `fret-components-ui` foundation surface organized around
//! Radix UI Primitives concepts, while remaining Rust-native and renderer-agnostic.
//!
//! - We port behavior outcomes, not React/DOM APIs.
//! - Runtime mechanisms live in `fret-ui`; these primitives provide component-layer composition.

pub mod dismissable_layer;
pub mod popper;
pub mod visually_hidden;
