mod background;
mod edge;
mod group;

use super::item_builders;
use crate::core::GroupId;
use crate::ui::canvas::widget::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMenuOpeningRoute {
    Group(GroupId),
    Edge(EdgeId),
    Background,
}

fn context_menu_opening_route(
    group_hit: Option<GroupId>,
    edge_hit: Option<EdgeId>,
) -> ContextMenuOpeningRoute {
    match (group_hit, edge_hit) {
        (Some(group_id), _) => ContextMenuOpeningRoute::Group(group_id),
        (None, Some(edge_id)) => ContextMenuOpeningRoute::Edge(edge_id),
        (None, None) => ContextMenuOpeningRoute::Background,
    }
}

fn record_context_menu_invocation_position<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
) {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_canvas_pos = Some(crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });
}

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
    record_context_menu_invocation_position(canvas, position);

    let group_hit = canvas.hit_group_context_target(cx.app, snapshot, position, zoom);
    let edge_hit = canvas.hit_edge_context_target(cx.app, snapshot, position, zoom);

    match context_menu_opening_route(group_hit, edge_hit) {
        ContextMenuOpeningRoute::Group(group_id) => {
            group::show_group_context_menu(canvas, cx, snapshot, position, group_id)
        }
        ContextMenuOpeningRoute::Edge(edge_id) => {
            edge::show_edge_context_menu(canvas, cx, snapshot, position, edge_id)
        }
        ContextMenuOpeningRoute::Background => {
            background::show_background_context_menu(canvas, cx, snapshot, position)
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{ContextMenuOpeningRoute, context_menu_opening_route};
    use crate::core::{EdgeId, GroupId};

    #[test]
    fn group_hit_takes_priority_over_edge_hit() {
        let group_id = GroupId::new();
        let edge_id = EdgeId::new();

        let route = context_menu_opening_route(Some(group_id), Some(edge_id));

        assert_eq!(route, ContextMenuOpeningRoute::Group(group_id));
    }

    #[test]
    fn edge_hit_is_used_when_group_hit_is_absent() {
        let edge_id = EdgeId::new();

        let route = context_menu_opening_route(None, Some(edge_id));

        assert_eq!(route, ContextMenuOpeningRoute::Edge(edge_id));
    }

    #[test]
    fn background_route_is_used_when_no_specific_target_is_hit() {
        let route = context_menu_opening_route(None, None);

        assert_eq!(route, ContextMenuOpeningRoute::Background);
    }
}
