use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn sync_pointer_up_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: Modifiers,
) {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.last_canvas_pos = Some(canvas_point_from_pointer_position(position));
}

pub(super) fn handle_sticky_wire_ignored_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    button: MouseButton,
) -> bool {
    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        super::paint_invalidation::invalidate_paint(cx);
        return true;
    }

    false
}

pub(super) fn handle_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    button: MouseButton,
) -> bool {
    if !super::cancel_session::matches_pan_release(&canvas.interaction, button) {
        return false;
    }

    super::cancel_session::clear_pan_drag_state(&mut canvas.interaction);
    canvas.stop_auto_pan_timer(cx.app);
    let started_inertia = canvas.maybe_start_pan_inertia_timer(cx.app, cx.window, snapshot);
    canvas.emit_move_end(
        snapshot,
        ViewportMoveKind::PanDrag,
        ViewportMoveEndOutcome::Ended,
    );
    if started_inertia {
        canvas.emit_move_start(snapshot, ViewportMoveKind::PanInertia);
    }
    cx.release_pointer_capture();
    super::paint_invalidation::invalidate_paint(cx);
    true
}

fn canvas_point_from_pointer_position(position: Point) -> crate::core::CanvasPoint {
    crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn canvas_point_from_pointer_position_transfers_axes() {
        let position = Point::new(Px(18.0), Px(-4.5));
        assert_eq!(
            canvas_point_from_pointer_position(position),
            crate::core::CanvasPoint { x: 18.0, y: -4.5 }
        );
    }
}
