use super::super::*;

pub(super) fn first_enabled_context_menu_item(items: &[NodeGraphContextMenuItem]) -> usize {
    items.iter().position(|item| item.enabled).unwrap_or(0)
}

pub(super) fn build_context_menu_state<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    desired_origin: Point,
    bounds: Rect,
    snapshot: &ViewSnapshot,
    target: ContextMenuTarget,
    items: Vec<NodeGraphContextMenuItem>,
    candidates: Vec<InsertNodeCandidate>,
) -> ContextMenuState {
    let origin = canvas.clamp_context_menu_origin(desired_origin, items.len(), bounds, snapshot);
    let active_item = first_enabled_context_menu_item(&items);
    ContextMenuState {
        origin,
        invoked_at: desired_origin,
        target,
        items,
        candidates,
        hovered_item: None,
        active_item,
        typeahead: String::new(),
    }
}
