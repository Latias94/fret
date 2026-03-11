use super::super::*;

pub(super) struct SearcherPickerRequest {
    pub(super) invoked_at: Point,
    pub(super) target: ContextMenuTarget,
    pub(super) candidates: Vec<InsertNodeCandidate>,
}

pub(super) fn background_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    at: CanvasPoint,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at: Point::new(Px(at.x), Px(at.y)),
        target: ContextMenuTarget::BackgroundInsertNodePicker { at },
        candidates: canvas.list_background_insert_candidates(host),
    }
}

pub(super) fn connection_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
) -> SearcherPickerRequest {
    SearcherPickerRequest {
        invoked_at: Point::new(Px(at.x), Px(at.y)),
        target: ContextMenuTarget::ConnectionInsertNodePicker { from, at },
        candidates: canvas.list_connection_insert_candidates(host, from),
    }
}
