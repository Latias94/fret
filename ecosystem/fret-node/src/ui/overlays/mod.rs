//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0126) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).
mod blackboard_declarative;
mod blackboard_interaction_policy;
mod blackboard_layout;
mod blackboard_paint_plan;
mod blackboard_policy;
mod controls_declarative;
mod controls_host_policy;
mod controls_interaction_policy;
mod controls_layout;
mod controls_paint_plan;
mod controls_policy;
mod group_rename;
mod minimap_declarative;
mod minimap_drag_policy;
mod minimap_interaction_policy;
mod minimap_navigation_policy;
mod minimap_policy;
mod minimap_projection;
#[cfg(feature = "compat-retained-canvas")]
mod panel_button_paint;
mod panel_item_state;
mod panel_navigation_policy;
mod panel_pointer_policy;
mod rename_command;
mod rename_declarative;
mod rename_host_layout;
mod rename_lifecycle;
mod rename_policy;
mod toolbar_layout_policy;
mod toolbar_policy;
mod toolbars_declarative;

#[cfg(feature = "compat-retained-canvas")]
pub use group_rename::{GroupRenameOverlay, NodeGraphOverlayState};
#[cfg(feature = "compat-retained-canvas")]
pub(in crate::ui) use rename_policy::open_group_rename_session;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPlacement {
    /// Positions itself within the canvas bounds (legacy / backwards-compatible).
    FloatingInCanvas,
    /// Treats `cx.bounds` as the overlay's own window-space panel bounds.
    PanelBounds,
}
