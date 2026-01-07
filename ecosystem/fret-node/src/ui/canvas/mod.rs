//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The current
//! implementation lives in `legacy.rs` and will be incrementally migrated into the new modules.

mod context_menu;
mod conversion;
mod event;
mod geometry;
mod legacy;
mod paint;
mod spatial;

pub use legacy::NodeGraphCanvas;
