use crate::ui::canvas::widget::*;

pub(super) fn scroll_pan_enabled<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
) -> bool {
    snapshot.interaction.pan_on_scroll
        || (snapshot.interaction.space_to_pan && canvas.interaction.pan_activation_key_held)
}
