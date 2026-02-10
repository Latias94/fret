//! Shared interaction building blocks for editor-grade UI.
//!
//! This crate intentionally lives under `ecosystem/`:
//! - it is policy-heavy and iteration-friendly,
//! - it is meant to be reused by multiple ecosystem surfaces (`imui`, node graphs, docking UX),
//! - it should not become a hard mechanism-layer contract in `crates/fret-ui` prematurely.
//!
//! Design goals:
//! - Provide small, testable state machines and math helpers.
//! - Avoid domain semantics (no node/edge graph types; no app-specific command wiring).
//! - Support multiple coordinate spaces via explicit transform helpers (screen-space vs canvas-space).

pub mod dpi;
pub mod drag;

#[cfg(feature = "runtime")]
pub mod runtime_drag;
