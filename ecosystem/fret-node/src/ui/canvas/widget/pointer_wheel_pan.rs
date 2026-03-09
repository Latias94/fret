use super::*;

pub(super) fn handle_scroll_pan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !(snapshot.interaction.pan_on_scroll
        || (snapshot.interaction.space_to_pan && canvas.interaction.pan_activation_key_held))
    {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::PanScroll);
    let mode = snapshot.interaction.pan_on_scroll_mode;
    let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
    let dy_for_shift = delta.y.0;

    let mut dx = delta.x.0;
    let mut dy = delta.y.0;
    match mode {
        crate::io::NodeGraphPanOnScrollMode::Free => {}
        crate::io::NodeGraphPanOnScrollMode::Horizontal => {
            dy = 0.0;
        }
        crate::io::NodeGraphPanOnScrollMode::Vertical => {
            dx = 0.0;
        }
    }

    if cx.input_ctx.platform != fret_runtime::Platform::Macos
        && modifiers.shift
        && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
    {
        dx = dy_for_shift;
        dy = 0.0;
    }
    canvas.update_view_state(cx.app, |state| {
        state.pan.x += dx * speed;
        state.pan.y += dy * speed;
    });
    super::paint_invalidation::invalidate_paint(cx);
    true
}
