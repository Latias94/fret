use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, marquee_cx::MarqueeCx};
use crate::ui::canvas::state::{PendingMarqueeDrag, ViewSnapshot};

pub(super) fn begin_background_marquee<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    pos: Point,
    modifiers: Modifiers,
    clear_selection_on_up: bool,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    let _ = snapshot;
    let _ = modifiers;
    canvas.interaction.pending_marquee = Some(PendingMarqueeDrag {
        start_pos: pos,
        clear_selection_on_up,
    });
    cx.capture_self_pointer();
    super::widget_tail::invalidate_widget_paint(cx);
}
