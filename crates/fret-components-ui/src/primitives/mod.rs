//! Radix-aligned primitive facades.
//!
//! This module exists to keep the `fret-components-ui` foundation surface organized around
//! Radix UI Primitives concepts, while remaining Rust-native and renderer-agnostic.
//!
//! # Where code should live (anti-duplication rules)
//!
//! This crate has three adjacent layers that can look similar if we are not strict:
//!
//! - `crate::headless`: pure logic / deterministic state machines / index math.
//! - `crate::declarative`: wiring helpers built on `ElementContext` + action hooks.
//! - `crate::primitives` (this module): **Radix-named stable entry points** (thin facades).
//!
//! To avoid drift and duplication:
//!
//! - If it is reusable **logic**, it belongs in `crate::headless` (and should be unit-testable).
//! - If it is reusable **wiring** (hooks, semantics stamping, overlay roots), it belongs in
//!   `crate::declarative` (and should have contract tests where appropriate).
//! - `crate::primitives` should stay thin: re-exports, small adapters, and stable naming aligned
//!   to <https://github.com/radix-ui/primitives> - not a second headless layer.
//!
//! Runtime mechanisms live in `fret-ui`; these facades intentionally port behavior outcomes, not
//! React/DOM APIs.

pub mod collection;
pub mod dismissable_layer;
pub mod focus_scope;
pub mod hover_intent;
pub mod menu;
pub mod popper;
pub mod presence;
pub mod roving_focus_group;
pub mod tooltip_delay_group;
pub mod tooltip_provider;
pub mod visually_hidden;
