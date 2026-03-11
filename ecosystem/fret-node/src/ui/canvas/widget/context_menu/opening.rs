mod background;
mod edge;
mod group;

use super::item_builders;
use crate::ui::canvas::widget::*;

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

    if group::try_show_group_context_menu(canvas, cx, snapshot, position, zoom) {
        return true;
    }

    if edge::try_show_edge_context_menu(canvas, cx, snapshot, position, zoom) {
        return true;
    }

    background::show_background_context_menu(canvas, cx, snapshot, position)
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
