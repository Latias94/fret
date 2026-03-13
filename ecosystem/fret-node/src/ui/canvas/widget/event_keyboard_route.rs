use super::*;

pub(super) fn route_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) {
    if keyboard_shortcuts::handle_escape_key(canvas, cx, key) {
        return;
    }

    if keyboard_shortcuts::handle_overlay_key_down(canvas, cx, key, modifiers) {
        return;
    }

    if keyboard_shortcuts::handle_modifier_shortcuts(cx, snapshot, key, modifiers) {
        return;
    }

    if keyboard_shortcuts::handle_tab_navigation(canvas, cx, snapshot, key, modifiers) {
        return;
    }

    if keyboard_pan_activation::handle_pan_activation_key_down(canvas, cx, snapshot, key, modifiers)
    {
        return;
    }

    if keyboard_shortcuts::handle_arrow_nudging(cx, snapshot, key, modifiers) {
        return;
    }

    let _ = keyboard_shortcuts::handle_delete_shortcut(cx, snapshot, key);
}

pub(super) fn route_key_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) {
    let _ = keyboard_pan_activation::handle_pan_activation_key_up(canvas, cx, snapshot, key);
}
