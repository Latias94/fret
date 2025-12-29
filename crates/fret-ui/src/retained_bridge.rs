//! Unstable retained-widget bridge for policy-heavy UI (e.g. docking migration).
//!
//! This module is intentionally feature-gated (`unstable-retained-bridge`) and is **not** part of
//! the stable `fret-ui` runtime contract surface (ADR 0066).

use crate::{UiHost, UiTree};
use fret_core::NodeId;

pub use crate::resize_handle::ResizeHandle;
pub use crate::widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};

/// Extension trait that exposes a feature-gated node creation API for retained widgets.
pub trait UiTreeRetainedExt<H: UiHost> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId;
}

impl<H: UiHost> UiTreeRetainedExt<H> for UiTree<H> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.create_node(widget)
    }
}
