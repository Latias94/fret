//! Regression gates that rely on the real text shaping pipeline (Parley).
//!
//! This crate intentionally lives under `apps/` so ecosystem crates can keep their layering
//! constraints (no direct `fret-render-*` dependencies) while we still exercise end-to-end shaping
//! invariants in CI/dev workflows.
