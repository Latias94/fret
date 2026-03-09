use fret_core::MouseButton;
use fret_ui::UiHost;

use super::searcher_ui::{dismiss_searcher_event, finish_searcher_event};
use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SearcherPointerHit {
    pub(super) inside: bool,
    pub(super) row_ix: Option<usize>,
}

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

    let hit = super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
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

pub(super) fn handle_searcher_pointer_up_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }
    if canvas.interaction.searcher.is_none() {
        super::searcher_activation_state::clear_pending_searcher_row_drag(&mut canvas.interaction);
        return false;
    }

    let hit = super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
    super::searcher_activation_state::finish_searcher_row_drag_release(canvas, cx, hit)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn arm_searcher_row_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
        position: Point,
    ) -> bool {
        super::searcher_activation_state::arm_searcher_row_drag(self, cx, row_ix, position)
    }

    pub(super) fn activate_searcher_hit_or_dismiss<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        hit: SearcherPointerHit,
    ) {
        super::searcher_activation_state::activate_searcher_hit_or_dismiss(self, cx, hit)
    }
}
