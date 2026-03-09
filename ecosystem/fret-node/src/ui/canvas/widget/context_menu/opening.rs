use crate::ui::canvas::widget::*;

use super::item_builders;

pub(in crate::ui::canvas::widget) fn handle_right_click_context_menu_event<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_canvas_pos = Some(crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });

    if let Some(group_id) = canvas.hit_group_context_target(cx.app, snapshot, position, zoom) {
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

    if let Some(edge) = canvas.hit_edge_context_target(cx.app, snapshot, position, zoom) {
        let items = canvas.build_edge_context_menu_items(cx.app, edge);
        canvas.select_edge_context_target(cx.app, edge);
        return canvas.show_context_menu(
            cx,
            snapshot,
            position,
            ContextMenuTarget::Edge(edge),
            items,
            Vec::new(),
            true,
        );
    }

    let has_selection = !snapshot.selected_nodes.is_empty()
        || !snapshot.selected_edges.is_empty()
        || !snapshot.selected_groups.is_empty();
    let items =
        item_builders::build_background_context_menu_items(cx.window.is_some(), has_selection);

    canvas.show_context_menu(
        cx,
        snapshot,
        position,
        ContextMenuTarget::Background,
        items,
        Vec::new(),
        false,
    )
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn show_context_menu<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        target: ContextMenuTarget,
        items: Vec<NodeGraphContextMenuItem>,
        candidates: Vec<InsertNodeCandidate>,
        clear_hover_edge: bool,
    ) -> bool {
        let menu = build_context_menu_state(
            self, position, cx.bounds, snapshot, target, items, candidates,
        );
        super::restore_context_menu(&mut self.interaction, menu);
        if clear_hover_edge {
            self.interaction.hover_edge = None;
        }
        cx.request_focus(cx.node);
        super::ui::finish_context_menu_event(cx)
    }

    pub(in crate::ui::canvas::widget) fn build_edge_context_menu_items<H: UiHost>(
        &mut self,
        host: &mut H,
        edge: EdgeId,
    ) -> Vec<NodeGraphContextMenuItem> {
        let presenter = &mut *self.presenter;
        let style = &self.style;
        self.graph
            .read_ref(host, |graph| {
                let mut items = Vec::new();
                presenter.fill_edge_context_menu(graph, edge, style, &mut items);
                item_builders::append_builtin_edge_context_menu_items(&mut items);
                items
            })
            .ok()
            .unwrap_or_default()
    }
}
