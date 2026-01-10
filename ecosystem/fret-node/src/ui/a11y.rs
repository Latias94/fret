//! Accessibility helpers for the node graph editor.
//!
//! This module is intentionally small: it provides a semantics-only widget that can be mounted as
//! a child of `NodeGraphCanvas` to enable `aria-activedescendant`-style semantics.

use std::sync::Arc;

use fret_core::{Rect, SemanticsRole, Size};
use fret_ui::{UiHost, retained_bridge::*};

use super::internals::NodeGraphInternalsStore;

/// Semantics-only node that mirrors the active descendant in the node graph.
///
/// Mount this widget as a child of `NodeGraphCanvas` to let the canvas set
/// `SemanticsNode.active_descendant` to an actual semantics node.
pub struct NodeGraphA11yActiveDescendant {
    internals: Arc<NodeGraphInternalsStore>,
}

impl NodeGraphA11yActiveDescendant {
    pub fn new(internals: Arc<NodeGraphInternalsStore>) -> Self {
        Self { internals }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphA11yActiveDescendant {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        // Semantics-only: consume no space.
        let _ = cx;
        Size::new(fret_core::Px(0.0), fret_core::Px(0.0))
    }

    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::ListItem);
        cx.set_focusable(false);
        cx.set_invokable(false);

        if let Some(label) = self.internals.snapshot().a11y_active_descendant_label {
            cx.set_label(label);
        } else {
            cx.set_label("Node Graph Active Item");
        }
    }
}
