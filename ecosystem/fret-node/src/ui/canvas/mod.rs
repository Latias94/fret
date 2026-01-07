//! Node graph canvas widget and editor interaction policy.
//!
//! This module is split into submodules to keep the editor-grade canvas maintainable. The current
//! implementation lives in `legacy.rs` and will be incrementally migrated into the new modules.

mod context_menu;
mod conversion;
mod event;
mod legacy;
mod paint;

pub use legacy::NodeGraphCanvas;
