//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The retained
//! widget entry point is `NodeGraphCanvas`.

mod context_menu;
mod conversion;
mod event;
mod geometry;
pub mod middleware;
mod paint;
mod searcher;
mod snaplines;
mod spatial;
mod state;
mod widget;
mod workflow;

pub use widget::NodeGraphCanvas;
