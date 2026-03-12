use crate::ui::canvas::widget::*;

pub(super) fn clear_for_group_resize<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    super::super::super::press_session::prepare_for_group_resize(&mut canvas.interaction);
}

pub(super) fn clear_for_group_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    super::super::super::press_session::prepare_for_group_drag(&mut canvas.interaction);
}

pub(super) fn clear_for_background_interaction<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    super::super::super::press_session::prepare_for_background_interaction(&mut canvas.interaction);
}
