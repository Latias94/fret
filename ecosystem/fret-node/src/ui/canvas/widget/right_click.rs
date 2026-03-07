use fret_core::Point;
use fret_ui::UiHost;

use super::context_menu::item_builders;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
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

    let hit_group = canvas.hit_group_context_target(cx.app, snapshot, position, zoom);

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

    let hit_edge = canvas.hit_edge_context_target(cx.app, snapshot, position, zoom);

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
