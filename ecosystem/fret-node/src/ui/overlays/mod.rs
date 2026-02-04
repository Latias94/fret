//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0135) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).

mod controls;
mod group_rename;
mod minimap;
mod toolbars;

pub use controls::NodeGraphControlsOverlay;
pub use group_rename::{GroupRenameOverlay, NodeGraphOverlayHost, NodeGraphOverlayState};
pub use minimap::NodeGraphMiniMapOverlay;
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

fn clamp_rect_to_bounds(mut rect: Rect, bounds: Rect) -> Rect {
    let w = rect.size.width.0.max(0.0);
    let h = rect.size.height.0.max(0.0);

    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = bounds.origin.x.0 + (bounds.size.width.0 - w).max(0.0);
    let max_y = bounds.origin.y.0 + (bounds.size.height.0 - h).max(0.0);

    rect.origin.x.0 = rect.origin.x.0.clamp(min_x, max_x);
    rect.origin.y.0 = rect.origin.y.0.clamp(min_y, max_y);
    rect
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
