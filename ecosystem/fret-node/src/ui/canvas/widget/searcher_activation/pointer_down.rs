use super::super::searcher_ui::{dismiss_searcher_event, finish_searcher_event};
use super::super::*;

pub(super) fn handle_searcher_pointer_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    let hit = super::super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
    match button {
        MouseButton::Left => {
            if let Some(row_ix) = hit.row_ix {
                let _ = canvas.arm_searcher_row_drag(cx, row_ix, position);
            } else if !hit.inside {
                canvas.dismiss_searcher_overlay(cx);
            }
            finish_searcher_event(cx)
        }
        _ => dismiss_searcher_event(canvas, cx),
    }
}
