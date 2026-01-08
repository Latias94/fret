use fret_core::Point;
use fret_ui::UiHost;

use super::NodeGraphCanvas;

pub(super) fn handle_panning_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    delta: Point,
) -> bool {
    if !canvas.interaction.panning {
        return false;
    }

    canvas.update_view_state(cx.app, |s| {
        s.pan.x += delta.x.0;
        s.pan.y += delta.y.0;
    });
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
