use super::*;

pub(super) fn handle_escape_key<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::keyboard_shortcuts_overlay::KeyboardOverlayCx<H, M>,
    key: fret_core::KeyCode,
) -> bool {
    super::keyboard_shortcuts_overlay::handle_escape_key(canvas, cx, key)
}

pub(super) fn handle_overlay_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::keyboard_shortcuts_overlay::KeyboardOverlayCx<H, M>,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_overlay::handle_overlay_key_down(canvas, cx, key, modifiers)
}

pub(in crate::ui::canvas::widget) trait KeyboardShortcutCommandSink {
    fn dispatch_keyboard_command(&mut self, command: &'static str);
}

pub(super) fn handle_modifier_shortcuts(
    cx: &mut impl KeyboardShortcutCommandSink,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_modifier_shortcuts(cx, snapshot, key, modifiers)
}

pub(super) fn handle_tab_navigation<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl KeyboardShortcutCommandSink,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_tab_navigation(canvas, cx, snapshot, key, modifiers)
}

pub(super) fn handle_arrow_nudging(
    cx: &mut impl KeyboardShortcutCommandSink,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_arrow_nudging(cx, snapshot, key, modifiers)
}

pub(super) fn handle_delete_shortcut(
    cx: &mut impl KeyboardShortcutCommandSink,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    super::keyboard_shortcuts_commands::handle_delete_shortcut(cx, snapshot, key)
}
