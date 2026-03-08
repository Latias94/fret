use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingMarqueeDrag, ViewSnapshot};

pub(super) fn begin_background_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
    clear_selection_on_up: bool,
) {
    let _ = snapshot;
    let _ = modifiers;
    canvas.interaction.pending_marquee = Some(PendingMarqueeDrag {
        start_pos: pos,
        clear_selection_on_up,
    });
    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}
