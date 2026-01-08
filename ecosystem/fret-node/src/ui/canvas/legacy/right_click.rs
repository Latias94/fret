use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::EdgeId;
use crate::ui::commands::{
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_INSERT_NODE, CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_SELECT_ALL,
};
use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};

use super::super::state::{ContextMenuState, ContextMenuTarget, ViewSnapshot};
use super::NodeGraphCanvas;

pub(super) fn handle_right_click_pointer_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let hit_edge = {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch: Vec<EdgeId> = Vec::new();
                this.hit_edge(
                    graph,
                    snapshot,
                    geom.as_ref(),
                    index.as_ref(),
                    position,
                    zoom,
                    &mut scratch,
                )
            })
            .ok()
            .flatten()
    };

    let Some(edge) = hit_edge else {
        let has_selection =
            !snapshot.selected_nodes.is_empty() || !snapshot.selected_edges.is_empty();
        let items: Vec<NodeGraphContextMenuItem> = vec![
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Insert Node..."),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_OPEN_INSERT_NODE,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Paste"),
                enabled: cx.window.is_some(),
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_PASTE,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Select All"),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_SELECT_ALL,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Delete Selection"),
                enabled: has_selection,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_DELETE_SELECTION,
                )),
            },
        ];

        let origin = canvas.clamp_context_menu_origin(position, items.len(), cx.bounds, snapshot);
        let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
        canvas.interaction.context_menu = Some(ContextMenuState {
            origin,
            invoked_at: position,
            target: ContextMenuTarget::Background,
            items,
            candidates: Vec::new(),
            hovered_item: None,
            active_item,
            typeahead: String::new(),
        });
        cx.request_focus(cx.node);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    };

    let items = {
        let presenter = &mut *canvas.presenter;
        let style = &canvas.style;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
                presenter.fill_edge_context_menu(graph, edge, style, &mut items);
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Insert Node..."),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
                    )),
                });
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Insert Reroute"),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_INSERT_REROUTE,
                    )),
                });
                items.push(NodeGraphContextMenuItem {
                    label: Arc::<str>::from("Delete"),
                    enabled: true,
                    action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                        CMD_NODE_GRAPH_DELETE_SELECTION,
                    )),
                });
                items
            })
            .ok()
            .unwrap_or_default()
    };

    let origin = canvas.clamp_context_menu_origin(position, items.len(), cx.bounds, snapshot);
    let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at: position,
        target: ContextMenuTarget::Edge(edge),
        items,
        candidates: Vec::new(),
        hovered_item: None,
        active_item,
        typeahead: String::new(),
    });
    canvas.interaction.hover_edge = None;
    cx.request_focus(cx.node);

    canvas.update_view_state(cx.app, |s| {
        s.selected_nodes.clear();
        if !s.selected_edges.iter().any(|id| *id == edge) {
            s.selected_edges.clear();
            s.selected_edges.push(edge);
        }
    });

    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
