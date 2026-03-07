use fret_core::Point;
use fret_ui::UiHost;

use super::context_menu::item_builders;
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
        let items = item_builders::build_group_context_menu_items();

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
        let items =
            item_builders::build_background_context_menu_items(cx.window.is_some(), has_selection);

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
