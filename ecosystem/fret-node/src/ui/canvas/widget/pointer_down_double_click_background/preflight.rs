use super::super::*;

pub(super) fn can_zoom_background_double_click<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    click_count: u8,
) -> bool {
    click_count == 2
        && snapshot.interaction.zoom_on_double_click
        && canvas.interaction.searcher.is_none()
        && canvas.interaction.context_menu.is_none()
}
