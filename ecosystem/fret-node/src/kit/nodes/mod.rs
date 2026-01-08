//! Built-in recipe node kinds for the kit layer.
//!
//! These are *not* core substrate contracts. They exist as convenience building blocks and demos.

/// A variadic merge node that grows/shrinks its input slots under `DataflowProfile` concretization.
pub const VARIADIC_MERGE_KIND: &str = "fret.variadic_merge";
