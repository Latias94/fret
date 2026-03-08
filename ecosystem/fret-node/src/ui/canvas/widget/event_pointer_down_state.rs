use super::*;

pub(super) fn prepare_pointer_down_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
) {
    if canvas.interaction.viewport_animation.is_some() {
        canvas.stop_viewport_animation_timer(cx.app);
    }
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Ended,
        );
    }
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.multi_selection_active = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(modifiers);
    canvas.interaction.last_canvas_pos = Some(canvas_point_from_pointer_position(position));
}

fn canvas_point_from_pointer_position(position: Point) -> CanvasPoint {
    CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_point_from_pointer_position_transfers_axes() {
        let position = Point::new(Px(12.5), Px(-8.0));
        assert_eq!(
            canvas_point_from_pointer_position(position),
            CanvasPoint { x: 12.5, y: -8.0 }
        );
    }
}
