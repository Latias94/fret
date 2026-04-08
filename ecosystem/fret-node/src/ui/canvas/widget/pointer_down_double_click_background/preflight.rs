use super::super::*;

pub(super) fn can_zoom_background_double_click<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    click_count: u8,
) -> bool {
    click_count == 2
        && snapshot.interaction.zoom_on_double_click
        && !super::super::menu_session::has_active_menu_session(&canvas.interaction)
}
