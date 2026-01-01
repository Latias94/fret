//! Integration convenience layer: `fret-ui` bound to `fret-app::App`.
//!
//! This crate exists to keep app/demo/editor code ergonomic (type aliases + re-exports)
//! while allowing `fret-ui` to remain host-generic and independent from `fret-app`.

pub use fret_app::App;

pub use fret_ui::declarative;
pub use fret_ui::element;
pub use fret_ui::elements;
pub use fret_ui::theme;
pub use fret_ui::tree;

pub use fret_ui::{
    GlobalElementId, Invalidation, PaintCachePolicy, Theme, ThemeConfig, ThemeSnapshot,
    UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiFrameCx, UiHost, UiLayerId,
};

pub type UiTree = fret_ui::UiTree<App>;

pub mod accessibility_actions;
