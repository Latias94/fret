//! Interaction helpers for canvas-like widgets.
//!
//! This module is intentionally headless and policy-light:
//! - It does not depend on `fret-ui`.
//! - It provides reusable building blocks (e.g. selection math + hit-test hooks).
//!
//! Higher-level tool modes and gesture maps should live in ecosystem crates (or `crate::ui` when
//! integration wiring is needed).

pub mod selection;
