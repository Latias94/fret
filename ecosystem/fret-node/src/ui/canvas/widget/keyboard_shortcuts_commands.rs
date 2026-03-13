use super::*;

pub(super) fn handle_modifier_shortcuts<H: UiHost>(
    cx: &mut EventCx<'_, H>,
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
        super::keyboard_shortcuts::dispatch_command(cx, command);
        return true;
    }

    let Some(command) = super::keyboard_shortcuts_map::modifier_command(key, modifiers) else {
        return false;
    };
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_tab_navigation<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
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

    if canvas.interaction.searcher.is_some() || canvas.interaction.context_menu.is_some() {
        return true;
    }

    let command = super::keyboard_shortcuts_map::plain_tab_focus_command(modifiers);
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_arrow_nudging<H: UiHost>(
    cx: &mut EventCx<'_, H>,
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
    super::keyboard_shortcuts::dispatch_command(cx, command);
    true
}

pub(super) fn handle_delete_shortcut<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    if !super::keyboard_shortcuts_gate::matches_delete_shortcut(
        snapshot.interaction.delete_key,
        key,
    ) {
        return false;
    }

    super::keyboard_shortcuts::dispatch_command(cx, CMD_NODE_GRAPH_DELETE_SELECTION);
    true
}
