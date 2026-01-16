//! Retained-canvas resource caches (policy-light).
//!
//! These caches are intended for long-lived interactive surfaces (node graphs, plots, editors)
//! that:
//! - emit scene ops every frame, but
//! - want to avoid re-preparing renderer-owned resources (text blobs, paths, SVGs) unnecessarily.
//!
//! Unlike declarative hosted caches (ADR 0156), retained widgets own their caches directly and
//! must release resources deterministically via `Widget::cleanup_resources`.

mod path_cache;
mod svg_cache;

pub use path_cache::*;
pub use svg_cache::*;
