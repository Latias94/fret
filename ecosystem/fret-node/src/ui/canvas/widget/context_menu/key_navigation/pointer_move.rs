use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

use super::super::ui;
use super::hover;

pub(super) fn handle_context_menu_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(menu) = canvas.interaction.context_menu.as_mut() else {
        return false;
    };

    let hovered_item = hit_context_menu_item(&canvas.style, menu, position, zoom);
    if hover::sync_context_menu_hovered_item(menu, hovered_item) {
        ui::invalidate_context_menu_paint(cx);
    }
    true
}
