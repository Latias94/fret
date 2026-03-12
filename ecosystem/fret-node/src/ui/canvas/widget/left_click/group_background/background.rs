use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn begin_background_interaction<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
) {
    if snapshot.interaction.elements_selectable {
        crate::ui::canvas::widget::marquee::begin_background_marquee(
            canvas, cx, snapshot, position, modifiers, true,
        );
    } else if snapshot.interaction.pan_on_drag.left {
        let _ = crate::ui::canvas::widget::pan_zoom::begin_panning(
            canvas,
            cx,
            snapshot,
            position,
            fret_core::MouseButton::Left,
        );
    }
}
