use super::*;

use fret_runtime::CommandId;
use fret_ui::UiHost;

fn dispatch_keyboard_command<H: UiHost>(
    cx: &mut impl super::keyboard_shortcuts::KeyboardShortcutDispatchCx<H>,
    command: &'static str,
) {
    super::command_adapter::dispatch_canvas_command(cx, CommandId::from(command));
    cx.stop_propagation();
}

pub(super) fn handle_modifier_shortcuts<H: UiHost>(
    cx: &mut impl super::keyboard_shortcuts::KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !super::keyboard_shortcuts_gate::allow_modifier_shortcut(modifiers) {
        return false;
    }

    if let Some(command) = super::keyboard_shortcuts_map::modifier_tab_focus_edge_command(
        snapshot.interaction.disable_keyboard_a11y,
        key,
        modifiers,
    ) {
        dispatch_keyboard_command(cx, command);
        return true;
    }

    let Some(command) = super::keyboard_shortcuts_map::modifier_command(key, modifiers) else {
        return false;
    };
    dispatch_keyboard_command(cx, command);
    true
}

pub(super) fn handle_tab_navigation<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::keyboard_shortcuts::KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !super::keyboard_shortcuts_gate::allow_plain_tab_navigation(
        snapshot.interaction.disable_keyboard_a11y,
        key,
        modifiers,
    ) {
        return false;
    }

    if super::menu_session::has_active_menu_session(&canvas.interaction) {
        return true;
    }

    let command = super::keyboard_shortcuts_map::plain_tab_focus_command(modifiers);
    dispatch_keyboard_command(cx, command);
    true
}

pub(super) fn handle_arrow_nudging<H: UiHost>(
    cx: &mut impl super::keyboard_shortcuts::KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !super::keyboard_shortcuts_gate::allow_arrow_nudging(key, modifiers) {
        return false;
    }

    if snapshot.interaction.disable_keyboard_a11y {
        return true;
    }

    if snapshot.selected_nodes.is_empty() && snapshot.selected_groups.is_empty() {
        return true;
    }

    let Some(command) = super::keyboard_shortcuts_map::arrow_nudge_command(key, modifiers) else {
        return true;
    };
    dispatch_keyboard_command(cx, command);
    true
}

pub(super) fn handle_delete_shortcut<H: UiHost>(
    cx: &mut impl super::keyboard_shortcuts::KeyboardShortcutDispatchCx<H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    if !super::keyboard_shortcuts_gate::matches_delete_shortcut(
        snapshot.interaction.delete_key,
        key,
    ) {
        return false;
    }

    dispatch_keyboard_command(cx, CMD_NODE_GRAPH_DELETE_SELECTION);
    true
}
