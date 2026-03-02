//! Internal registries for diag checks.
//!
//! This module is intentionally small in v1. The goal is to create a seam so we can move
//! ad-hoc check wiring (lint/post-run gates/perf hint gates) behind explicit registries without
//! growing more monolith-style match statements.
//!
//! NOTE: This is tooling-only; it is not a runtime contract.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CheckKind {
    Lint,
    Triage,
    Perf,
    Hotspots,
}
