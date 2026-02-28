//! Diagnostics-only semantics anchors for node graph UIs.
//!
//! These widgets are intentionally paint-free and hit-test-free. They exist to provide stable
//! `test_id` selectors for `fretboard diag` scripts that need to drive pointer interactions
//! without relying on pixel coordinates.

use std::sync::Arc;

use fret_core::{Rect, SemanticsRole, Size};
use fret_ui::{UiHost, retained_bridge::*};

use super::internals::NodeGraphInternalsStore;

/// A semantics-only anchor that contributes a stable `test_id`.
///
/// Caller is responsible for laying out this node to the desired bounds (typically a port bounds
/// from `NodeGraphInternalsStore`).
pub struct NodeGraphDiagAnchor {
    test_id: Arc<str>,
}

impl NodeGraphDiagAnchor {
    pub fn new(test_id: impl Into<Arc<str>>) -> Self {
        Self {
            test_id: test_id.into(),
        }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphDiagAnchor {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
        cx.set_focusable(false);
        cx.set_invokable(false);
        cx.set_test_id(self.test_id.as_ref());
        cx.set_label("Node Graph Diagnostics Anchor");
    }
}

/// Diagnostics-only semantics flag that exists only while a wire drag session is active.
///
/// This is intentionally value-free so scripts can gate on `exists(test_id)` even when text
/// redaction is enabled.
pub struct NodeGraphDiagConnectingFlag {
    internals: Arc<NodeGraphInternalsStore>,
    test_id: Arc<str>,
}

impl NodeGraphDiagConnectingFlag {
    pub fn new(internals: Arc<NodeGraphInternalsStore>, test_id: impl Into<Arc<str>>) -> Self {
        Self {
            internals,
            test_id: test_id.into(),
        }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphDiagConnectingFlag {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn semantics_present(&self) -> bool {
        self.internals.a11y_snapshot().connecting
    }

    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        // Semantics-only: consume no space.
        Size::new(fret_core::Px(0.0), fret_core::Px(0.0))
    }

    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
        cx.set_focusable(false);
        cx.set_invokable(false);
        cx.set_test_id(self.test_id.as_ref());
        cx.set_label("Node Graph Connecting");
    }
}
