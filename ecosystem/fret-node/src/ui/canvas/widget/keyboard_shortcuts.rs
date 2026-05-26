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

pub(in crate::ui::canvas::widget) trait KeyboardShortcutDispatchCx<H: UiHost>:
    super::command_adapter::CanvasCommandDispatchCx + super::low_level_adapter::CanvasHandledCx<H>
{
}

impl<H: UiHost, T> KeyboardShortcutDispatchCx<H> for T where
    T: super::command_adapter::CanvasCommandDispatchCx
        + super::low_level_adapter::CanvasHandledCx<H>
{
}

pub(super) fn handle_modifier_shortcuts<H: UiHost>(
    cx: &mut impl KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_modifier_shortcuts(cx, snapshot, key, modifiers)
}

pub(super) fn handle_tab_navigation<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_tab_navigation(canvas, cx, snapshot, key, modifiers)
}

pub(super) fn handle_arrow_nudging<H: UiHost>(
    cx: &mut impl KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::keyboard_shortcuts_commands::handle_arrow_nudging(cx, snapshot, key, modifiers)
}

pub(super) fn handle_delete_shortcut<H: UiHost>(
    cx: &mut impl KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    super::keyboard_shortcuts_commands::handle_delete_shortcut(cx, snapshot, key)
}
