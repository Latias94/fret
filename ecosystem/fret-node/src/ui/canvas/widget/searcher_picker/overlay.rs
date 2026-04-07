use super::super::*;

pub(super) fn open_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    request: super::request::SearcherPickerRequest,
) {
    let snapshot = canvas.sync_view_state(host);
    let bounds = canvas.interaction.last_bounds.unwrap_or_default();
    canvas.open_searcher_overlay(
        request.invoked_at,
        bounds,
        &snapshot,
        request.target,
        request.candidates,
        request.rows_mode,
    );
}
