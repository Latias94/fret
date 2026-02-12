//! Retained-canvas resource caches (policy-light).
//!
//! These caches are intended for long-lived interactive surfaces (node graphs, plots, editors)
//! that:
//! - emit scene ops every frame, but
//! - want to avoid re-preparing renderer-owned resources (text blobs, paths, SVGs) unnecessarily.
//!
//! Unlike declarative hosted caches (ADR 0141), retained widgets own their caches directly and
//! must release resources deterministically via `Widget::cleanup_resources`.

mod hosted_resource_touch;
mod path_cache;
mod scene_op_cache;
mod scene_op_tile_cache;
mod svg_cache;

pub use crate::text::TextCache;

/// Lightweight counters for cache observability.
///
/// These are intentionally backend-agnostic and may be wired into diagnostics tooling by
/// ecosystem/app code.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    pub get_calls: u64,
    pub get_hits: u64,
    pub get_misses: u64,
    pub prepare_calls: u64,
    pub prepare_hits: u64,
    pub prepare_misses: u64,
    pub prune_calls: u64,
    pub clear_calls: u64,
    pub evict_calls: u64,
    pub release_replaced: u64,
    pub release_prune_age: u64,
    pub release_prune_budget: u64,
    pub release_clear: u64,
    pub release_evict: u64,
}

pub use hosted_resource_touch::*;
pub use path_cache::*;
pub use scene_op_cache::*;
pub use scene_op_tile_cache::*;
pub use svg_cache::*;
