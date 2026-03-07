use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::commands::{
    CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_DELETE_SELECTION,
    CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT, CMD_NODE_GRAPH_GROUP_RENAME,
    CMD_NODE_GRAPH_GROUP_SEND_TO_BACK, CMD_NODE_GRAPH_OPEN_INSERT_NODE, CMD_NODE_GRAPH_PASTE,
    CMD_NODE_GRAPH_SELECT_ALL,
};
use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};

use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{ContextMenuTarget, ViewSnapshot};

pub(super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_canvas_pos = Some(crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });

    let hit_group = {
        let header_h = canvas.style.geometry.node_header_height;
        let pos = position;
        let this = &*canvas;
        this.graph
            .read_ref(cx.app, |graph| {
                let order =
                    crate::ui::canvas::geometry::group_order(graph, &snapshot.group_draw_order);
                for group_id in order.iter().rev() {
                    let Some(group) = graph.groups.get(group_id) else {
                        continue;
                    };

                    let rect0 = this.group_rect_with_preview(*group_id, group.rect);

                    let rect = super::group_resize::group_rect_to_px(rect0);
                    let handle = this.resize_handle_rect(rect, zoom);
                    if super::group_resize::group_resize_handle_hit(handle, pos, zoom, 6.0) {
                        return Some(*group_id);
                    }

                    if super::pending_group_drag::group_header_hit(rect0, header_h, zoom, pos) {
                        return Some(*group_id);
                    }
                }
                None
            })
            .ok()
            .flatten()
    };

    if let Some(group_id) = hit_group {
        let items: Vec<NodeGraphContextMenuItem> = vec![
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Bring to Front"),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Send to Back"),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_GROUP_SEND_TO_BACK,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Rename..."),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_GROUP_RENAME,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Delete"),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_DELETE_SELECTION,
                )),
            },
        ];

        canvas.select_group_context_target(cx.app, group_id);
        return canvas.show_context_menu(
            cx,
            snapshot,
            position,
            ContextMenuTarget::Group(group_id),
            items,
            Vec::new(),
            true,
        );
    }

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let hit_edge = {
        let this = &*canvas;
        let geom = geom.clone();
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                this.hit_edge(graph, snapshot, &mut ctx, position)
            })
            .ok()
            .flatten()
    };

    let Some(edge) = hit_edge else {
        let has_selection = !snapshot.selected_nodes.is_empty()
            || !snapshot.selected_edges.is_empty()
            || !snapshot.selected_groups.is_empty();
        let items: Vec<NodeGraphContextMenuItem> = vec![
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Insert Node..."),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_OPEN_INSERT_NODE,
                )),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("Create Group"),
                enabled: true,
                action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(
                    CMD_NODE_GRAPH_CREATE_GROUP,
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

        return canvas.show_context_menu(
            cx,
            snapshot,
            position,
            ContextMenuTarget::Background,
            items,
            Vec::new(),
            false,
        );
    };

    let items = canvas.build_edge_context_menu_items(cx.app, edge);
    canvas.select_edge_context_target(cx.app, edge);
    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Edge(edge),
        items,
        Vec::new(),
        true,
    )
}
