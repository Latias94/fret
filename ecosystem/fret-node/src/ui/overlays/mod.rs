//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0126) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).
#[cfg(feature = "compat-retained-canvas")]
mod blackboard;
mod blackboard_layout;
#[cfg(feature = "compat-retained-canvas")]
mod blackboard_paint;
mod blackboard_policy;
#[cfg(feature = "compat-retained-canvas")]
mod controls;
mod controls_declarative;
mod controls_layout;
mod controls_policy;
mod group_rename;
#[cfg(feature = "compat-retained-canvas")]
mod minimap;
mod minimap_drag_policy;
mod minimap_navigation_policy;
mod minimap_policy;
mod minimap_projection;
#[cfg(feature = "compat-retained-canvas")]
mod panel_button_paint;
mod panel_item_state;
mod panel_navigation_policy;
mod panel_pointer_policy;
mod rename_host_event;
mod rename_host_layout;
mod rename_policy;
mod toolbar_policy;
#[cfg(feature = "compat-retained-canvas")]
mod toolbars;

#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use blackboard::NodeGraphBlackboardOverlay;
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use controls::NodeGraphControlsOverlay;
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use controls_policy::{NodeGraphControlsBindings, NodeGraphControlsCommandBinding};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use group_rename::NodeGraphOverlayHost;
#[cfg(any(test, feature = "compat-retained-canvas"))]
pub use group_rename::{GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use minimap::NodeGraphMiniMapOverlay;
#[cfg(feature = "compat-retained-canvas")]
pub(in crate::ui) use rename_policy::{open_group_rename_session, open_symbol_rename_session};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use toolbar_policy::{NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarSize};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub use toolbars::{NodeGraphEdgeToolbar, NodeGraphNodeToolbar};

#[cfg(feature = "compat-retained-canvas")]
use fret_core::{Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPlacement {
    /// Positions itself within the canvas bounds (legacy / backwards-compatible).
    FloatingInCanvas,
    /// Treats `cx.bounds` as the overlay's own panel bounds (for `NodeGraphPanel` composition).
    PanelBounds,
}

#[cfg(feature = "compat-retained-canvas")]
fn layout_hidden_child_and_release_focus<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::LayoutCx<'_, H>,
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

#[cfg(feature = "compat-retained-canvas")]
use fret_ui::UiHost;
