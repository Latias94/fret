use crate::ui::canvas::widget::*;

pub(super) fn handle_missing_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool
where
    Cx: pointer_move_release::PointerMoveReleaseCx<H, M>,
{
    if !canvas.interaction.panning {
        return false;
    }

    if !should_end_pan_release(canvas.interaction.panning_button, buttons) {
        return false;
    }

    let snapshot = canvas.sync_view_state(pointer_up_release_cx::PointerUpReleaseCx::host(cx));
    let button = canvas
        .interaction
        .panning_button
        .unwrap_or(fret_core::MouseButton::Middle);
    let _ = pointer_up::handle_pointer_up(
        canvas,
        cx,
        &snapshot,
        position,
        button,
        1,
        modifiers,
        snapshot.zoom,
    );
    true
}

fn should_end_pan_release(
    panning_button: Option<fret_core::MouseButton>,
    buttons: fret_core::MouseButtons,
) -> bool {
    match panning_button {
        Some(fret_core::MouseButton::Middle) => !buttons.middle,
        Some(fret_core::MouseButton::Left) => !buttons.left,
        Some(fret_core::MouseButton::Right) => !buttons.right,
        _ => false,
    }
}
