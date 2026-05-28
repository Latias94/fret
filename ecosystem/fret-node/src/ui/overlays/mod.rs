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

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct NodeGraphEdgeToolbarInternalsHostTestProps {
    pub(crate) view_state: fret_runtime::Model<crate::io::NodeGraphViewState>,
    pub(crate) requested_edge: Option<crate::core::EdgeId>,
    pub(crate) internals: std::sync::Arc<crate::ui::NodeGraphInternalsStore>,
    pub(crate) bounds: fret_core::Rect,
    pub(crate) size: fret_core::Size,
    pub(crate) label: std::sync::Arc<str>,
    pub(crate) test_id: std::sync::Arc<str>,
}

#[cfg(test)]
pub(crate) fn node_graph_edge_toolbar_host_for_internals_test<H: fret_ui::UiHost + 'static>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    props: NodeGraphEdgeToolbarInternalsHostTestProps,
    children: impl FnOnce(&mut fret_ui::ElementContext<'_, H>) -> Vec<fret_ui::element::AnyElement>,
) -> fret_ui::element::AnyElement {
    let target = toolbars_declarative::resolve_edge_toolbar_declarative_target(
        &props.view_state,
        props.requested_edge,
        props.internals.as_ref(),
        cx.app,
    );

    toolbars_declarative::node_graph_edge_toolbar_host_element(
        cx,
        toolbars_declarative::NodeGraphEdgeToolbarHostElementProps {
            bounds: props.bounds,
            target,
            visibility: toolbar_policy::NodeGraphToolbarVisibility::WhenSelected,
            align_x: toolbar_policy::NodeGraphToolbarAlign::Center,
            align_y: toolbar_policy::NodeGraphToolbarAlign::Center,
            size: toolbar_policy::NodeGraphToolbarSize::Fixed(props.size),
            offset: fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            label: props.label,
            test_id: props.test_id,
            focus_fallback: None,
        },
        children,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayPlacement {
    /// Positions itself within the canvas bounds (legacy / backwards-compatible).
    FloatingInCanvas,
    /// Treats `cx.bounds` as the overlay's own window-space panel bounds.
    PanelBounds,
}
