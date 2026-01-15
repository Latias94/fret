use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, DefaultKeybinding, KeyChord, PlatformFilter,
};
use std::sync::Arc;

pub const CMD_WORKSPACE_TAB_NEXT: &str = "workspace.tab.next";
pub const CMD_WORKSPACE_TAB_PREV: &str = "workspace.tab.prev";
pub const CMD_WORKSPACE_TAB_CLOSE: &str = "workspace.tab.close";

pub const CMD_WORKSPACE_PANE_NEXT: &str = "workspace.pane.next";
pub const CMD_WORKSPACE_PANE_PREV: &str = "workspace.pane.prev";

pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT: &str = "workspace.pane.move_active_tab.next";
pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV: &str = "workspace.pane.move_active_tab.prev";

pub const CMD_WORKSPACE_PANE_RESIZE_RIGHT: &str = "workspace.pane.resize.right";
pub const CMD_WORKSPACE_PANE_RESIZE_LEFT: &str = "workspace.pane.resize.left";
pub const CMD_WORKSPACE_PANE_RESIZE_UP: &str = "workspace.pane.resize.up";
pub const CMD_WORKSPACE_PANE_RESIZE_DOWN: &str = "workspace.pane.resize.down";

pub const CMD_WORKSPACE_PANE_SPLIT_RIGHT: &str = "workspace.pane.split.right";
pub const CMD_WORKSPACE_PANE_SPLIT_LEFT: &str = "workspace.pane.split.left";
pub const CMD_WORKSPACE_PANE_SPLIT_UP: &str = "workspace.pane.split.up";
pub const CMD_WORKSPACE_PANE_SPLIT_DOWN: &str = "workspace.pane.split.down";

pub const CMD_WORKSPACE_PANE_FOCUS_RIGHT: &str = "workspace.pane.focus.right";
pub const CMD_WORKSPACE_PANE_FOCUS_LEFT: &str = "workspace.pane.focus.left";
pub const CMD_WORKSPACE_PANE_FOCUS_UP: &str = "workspace.pane.focus.up";
pub const CMD_WORKSPACE_PANE_FOCUS_DOWN: &str = "workspace.pane.focus.down";

pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT: &str = "workspace.pane.move_active_tab.right";
pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT: &str = "workspace.pane.move_active_tab.left";
pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP: &str = "workspace.pane.move_active_tab.up";
pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN: &str = "workspace.pane.move_active_tab.down";

/// Prefix for "activate a specific tab" commands.
///
/// This is intentionally a prefix-based command family so apps can implement their own tab models
/// without exposing internal IDs via generic enum payloads.
pub const CMD_WORKSPACE_TAB_ACTIVATE_PREFIX: &str = "workspace.tab.activate.";

/// Prefix for "close a specific tab" commands.
///
/// This is intentionally a prefix-based command family so apps can implement their own tab models
/// without exposing internal IDs via generic enum payloads.
pub const CMD_WORKSPACE_TAB_CLOSE_PREFIX: &str = "workspace.tab.close.";

/// Prefix for "activate a specific pane" commands.
///
/// This is prefix-based so apps can use their own stable pane IDs (strings) without adding a
/// dedicated runtime enum payload surface.
pub const CMD_WORKSPACE_PANE_ACTIVATE_PREFIX: &str = "workspace.pane.activate.";

/// Prefix for "split the active pane and create a new pane" commands.
///
/// Shape: `workspace.pane.split.<axis>.<side>.<new_pane_id>`
/// - `<axis>`: `horizontal` / `vertical`
/// - `<side>`: `first` / `second`
///
/// Notes:
/// - This command family is intentionally prefix-based so apps can pick their own pane ID scheme.
/// - `WorkspaceWindowLayout::apply_command` uses a default split fraction (0.5). Apps that need
///   custom split sizing should call `split_active_pane` directly.
pub const CMD_WORKSPACE_PANE_SPLIT_PREFIX: &str = "workspace.pane.split.";

/// Prefix for "move active tab to a specific pane" commands.
///
/// Shape: `workspace.pane.move_active_tab_to.<pane_id>`
pub const CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_TO_PREFIX: &str = "workspace.pane.move_active_tab_to.";

pub fn tab_activate_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_ACTIVATE_PREFIX}{id}"
    ))))
}

pub fn tab_close_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_CLOSE_PREFIX}{id}"
    ))))
}

pub fn pane_activate_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_PANE_ACTIVATE_PREFIX}{id}"
    ))))
}

pub fn pane_split_command(
    axis: fret_core::Axis,
    side: crate::layout::SplitSide,
    new_pane_id: &str,
) -> Option<CommandId> {
    let id = new_pane_id.trim();
    if id.is_empty() {
        return None;
    }

    let axis = match axis {
        fret_core::Axis::Horizontal => "horizontal",
        fret_core::Axis::Vertical => "vertical",
    };
    let side = match side {
        crate::layout::SplitSide::First => "first",
        crate::layout::SplitSide::Second => "second",
    };

    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_PANE_SPLIT_PREFIX}{axis}.{side}.{id}"
    ))))
}

pub fn pane_move_active_tab_to_command(pane_id: &str) -> Option<CommandId> {
    let id = pane_id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_TO_PREFIX}{id}"
    ))))
}

fn kb(platform: PlatformFilter, key: KeyCode, mods: Modifiers) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        chord: KeyChord::new(key, mods),
        when: None,
    }
}

pub fn register_workspace_commands(registry: &mut CommandRegistry) {
    let win_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let linux_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let mac_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let mac_cmd = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        )
    };

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_NEXT),
        CommandMeta::new("Next Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "next", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Tab, false),
                linux_ctrl(KeyCode::Tab, false),
                mac_ctrl(KeyCode::Tab, false),
            ]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_PREV),
        CommandMeta::new("Previous Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "previous", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Tab, true),
                linux_ctrl(KeyCode::Tab, true),
                mac_ctrl(KeyCode::Tab, true),
            ]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_CLOSE),
        CommandMeta::new("Close Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "close", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::KeyW, false),
                linux_ctrl(KeyCode::KeyW, false),
                mac_cmd(KeyCode::KeyW),
            ]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_NEXT),
        CommandMeta::new("Next Pane")
            .with_category("Workspace")
            .with_keywords(["pane", "next", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_PREV),
        CommandMeta::new("Previous Pane")
            .with_category("Workspace")
            .with_keywords(["pane", "previous", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT),
        CommandMeta::new("Move Active Tab to Next Pane")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "next", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV),
        CommandMeta::new("Move Active Tab to Previous Pane")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "previous", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_RESIZE_RIGHT),
        CommandMeta::new("Resize Pane Right")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "right", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_RESIZE_LEFT),
        CommandMeta::new("Resize Pane Left")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "left", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_RESIZE_UP),
        CommandMeta::new("Resize Pane Up")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "up", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_RESIZE_DOWN),
        CommandMeta::new("Resize Pane Down")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "down", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_SPLIT_RIGHT),
        CommandMeta::new("Split Pane Right")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "right", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_SPLIT_LEFT),
        CommandMeta::new("Split Pane Left")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "left", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_SPLIT_UP),
        CommandMeta::new("Split Pane Up")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "up", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_SPLIT_DOWN),
        CommandMeta::new("Split Pane Down")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "down", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_FOCUS_RIGHT),
        CommandMeta::new("Focus Pane Right")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "right", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_FOCUS_LEFT),
        CommandMeta::new("Focus Pane Left")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "left", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_FOCUS_UP),
        CommandMeta::new("Focus Pane Up")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "up", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_FOCUS_DOWN),
        CommandMeta::new("Focus Pane Down")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "down", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT),
        CommandMeta::new("Move Active Tab Right")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "right", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT),
        CommandMeta::new("Move Active Tab Left")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "left", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP),
        CommandMeta::new("Move Active Tab Up")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "up", "workspace"]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN),
        CommandMeta::new("Move Active Tab Down")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "down", "workspace"]),
    );
}
