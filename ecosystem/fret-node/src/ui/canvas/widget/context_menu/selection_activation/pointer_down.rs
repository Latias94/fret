use super::super::ui;
use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_pointer_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    let Some(menu) = super::super::take_context_menu(&mut canvas.interaction) else {
        return false;
    };

    match button {
        MouseButton::Left => {
            if let Some(index) = hit_context_menu_item(&canvas.style, &menu, position, zoom) {
                if matches!(
                    canvas.activate_context_menu_selection(cx, &menu, index),
                    super::ContextMenuSelectionActivationOutcome::KeepOpen
                ) {
                    super::super::restore_context_menu(&mut canvas.interaction, menu);
                }
            }
            ui::finish_context_menu_event(cx)
        }
        MouseButton::Right => false,
        _ => ui::finish_context_menu_event(cx),
    }
}
