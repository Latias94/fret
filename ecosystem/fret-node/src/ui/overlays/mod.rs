//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0126) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).
mod blackboard;
mod controls;
mod group_rename;
mod minimap;
mod toolbars;

pub use blackboard::NodeGraphBlackboardOverlay;
pub use controls::NodeGraphControlsOverlay;
pub use controls::{NodeGraphControlsBindings, NodeGraphControlsCommandBinding};
pub use group_rename::{
    GroupRenameOverlay, NodeGraphOverlayHost, NodeGraphOverlayState, SymbolRenameOverlay,
};
pub use minimap::{
    NodeGraphMiniMapBindings, NodeGraphMiniMapNavigationBinding, NodeGraphMiniMapOverlay,
};
pub use toolbars::{
    NodeGraphEdgeToolbar, NodeGraphNodeToolbar, NodeGraphToolbarAlign, NodeGraphToolbarPosition,
    NodeGraphToolbarSize, NodeGraphToolbarVisibility,
};

use fret_core::{Px, Rect, Size};
use fret_ui::{UiHost, retained_bridge::LayoutCx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPlacement {
    /// Positions itself within the canvas bounds (legacy / backwards-compatible).
    FloatingInCanvas,
    /// Treats `cx.bounds` as the overlay's own panel bounds (for `NodeGraphPanel` composition).
    PanelBounds,
}

fn layout_hidden_child_and_release_focus<H: UiHost>(
    cx: &mut LayoutCx<'_, H>,
    child: fret_core::NodeId,
    canvas_node: fret_core::NodeId,
) {
    cx.layout_in(
        child,
        Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0))),
    );
    if cx.focus == Some(child) {
        cx.tree.set_focus(Some(canvas_node));
    }
}
