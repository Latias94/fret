use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    ActionId, CommandId, CommandMeta, CommandRegistry, DefaultKeybinding, KeyChord, PlatformFilter,
    TypedAction,
};
use std::sync::Arc;

pub const CMD_WORKSPACE_TAB_NEXT: &str = "workspace.tab.next";
pub const CMD_WORKSPACE_TAB_PREV: &str = "workspace.tab.prev";
pub const CMD_WORKSPACE_TAB_CLOSE: &str = "workspace.tab.close";
pub const CMD_WORKSPACE_TAB_CLOSE_OTHERS: &str = "workspace.tab.close.others";
pub const CMD_WORKSPACE_TAB_CLOSE_LEFT: &str = "workspace.tab.close.left";
pub const CMD_WORKSPACE_TAB_CLOSE_RIGHT: &str = "workspace.tab.close.right";

pub const CMD_WORKSPACE_TAB_MOVE_LEFT: &str = "workspace.tab.move.left";
pub const CMD_WORKSPACE_TAB_MOVE_RIGHT: &str = "workspace.tab.move.right";

pub const CMD_WORKSPACE_TAB_TOGGLE_PIN: &str = "workspace.tab.toggle_pin";

/// Prefix for "open a tab as preview and activate it" commands.
///
/// Shape: `workspace.tab.open_preview.<tab_id>`
pub const CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX: &str = "workspace.tab.open_preview.";

/// Commit the active preview tab (if any), converting it into a normal tab.
pub const CMD_WORKSPACE_TAB_COMMIT_PREVIEW: &str = "workspace.tab.commit_preview";

/// Prefix for "move the active tab before another tab" commands.
///
/// Shape: `workspace.tab.move_before.<target_tab_id>`
pub const CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX: &str = "workspace.tab.move_before.";

/// Prefix for "move the active tab after another tab" commands.
///
/// Shape: `workspace.tab.move_after.<target_tab_id>`
pub const CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX: &str = "workspace.tab.move_after.";

/// Prefix for "move a specific tab before another tab" commands.
///
/// Shape: `workspace.tab.move_before_id.<dragged_len>:<dragged_tab_id><target_tab_id>`
///
/// The length prefix keeps the command family deterministic for app-owned tab IDs that may contain
/// dots or other separators.
pub const CMD_WORKSPACE_TAB_MOVE_BEFORE_ID_PREFIX: &str = "workspace.tab.move_before_id.";

/// Prefix for "move a specific tab after another tab" commands.
///
/// Shape: `workspace.tab.move_after_id.<dragged_len>:<dragged_tab_id><target_tab_id>`
///
/// The length prefix keeps the command family deterministic for app-owned tab IDs that may contain
/// dots or other separators.
pub const CMD_WORKSPACE_TAB_MOVE_AFTER_ID_PREFIX: &str = "workspace.tab.move_after_id.";

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

pub const CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP: &str = "workspace.pane.focus_tab_strip";
pub const CMD_WORKSPACE_PANE_FOCUS_CONTENT: &str = "workspace.pane.focus_content";
pub const CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS: &str = "workspace.pane.toggle_tab_strip_focus";

pub const CMD_WORKSPACE_DIRTY_CLOSE_CANCEL: &str = "workspace.dirty_close.cancel";
pub const CMD_WORKSPACE_DIRTY_CLOSE_DISCARD: &str = "workspace.dirty_close.discard";
pub const CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE: &str = "workspace.dirty_close.save_and_close";

/// Typed unit actions for action-first authoring.
///
/// `ActionId` is an alias over `CommandId` (ADR 0307), so these marker types provide a
/// typed authoring surface without introducing new runtime schemas.
pub mod act {
    use super::*;

    macro_rules! workspace_unit_action {
        ($name:ident, $id:ident) => {
            pub struct $name;

            impl $name {
                pub const ID: &'static str = $id;

                pub fn command_id() -> CommandId {
                    CommandId::from(Self::ID)
                }
            }

            impl TypedAction for $name {
                fn action_id() -> ActionId {
                    Self::command_id()
                }
            }
        };
    }

    workspace_unit_action!(WorkspaceTabNext, CMD_WORKSPACE_TAB_NEXT);
    workspace_unit_action!(WorkspaceTabPrev, CMD_WORKSPACE_TAB_PREV);
    workspace_unit_action!(WorkspaceTabClose, CMD_WORKSPACE_TAB_CLOSE);
    workspace_unit_action!(WorkspaceTabCloseOthers, CMD_WORKSPACE_TAB_CLOSE_OTHERS);
    workspace_unit_action!(WorkspaceTabCloseLeft, CMD_WORKSPACE_TAB_CLOSE_LEFT);
    workspace_unit_action!(WorkspaceTabCloseRight, CMD_WORKSPACE_TAB_CLOSE_RIGHT);
    workspace_unit_action!(WorkspaceTabMoveLeft, CMD_WORKSPACE_TAB_MOVE_LEFT);
    workspace_unit_action!(WorkspaceTabMoveRight, CMD_WORKSPACE_TAB_MOVE_RIGHT);
    workspace_unit_action!(WorkspaceTabTogglePin, CMD_WORKSPACE_TAB_TOGGLE_PIN);
    workspace_unit_action!(WorkspaceTabCommitPreview, CMD_WORKSPACE_TAB_COMMIT_PREVIEW);

    workspace_unit_action!(WorkspacePaneNext, CMD_WORKSPACE_PANE_NEXT);
    workspace_unit_action!(WorkspacePanePrev, CMD_WORKSPACE_PANE_PREV);
    workspace_unit_action!(
        WorkspacePaneMoveActiveTabNext,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT
    );
    workspace_unit_action!(
        WorkspacePaneMoveActiveTabPrev,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV
    );

    workspace_unit_action!(WorkspacePaneResizeRight, CMD_WORKSPACE_PANE_RESIZE_RIGHT);
    workspace_unit_action!(WorkspacePaneResizeLeft, CMD_WORKSPACE_PANE_RESIZE_LEFT);
    workspace_unit_action!(WorkspacePaneResizeUp, CMD_WORKSPACE_PANE_RESIZE_UP);
    workspace_unit_action!(WorkspacePaneResizeDown, CMD_WORKSPACE_PANE_RESIZE_DOWN);

    workspace_unit_action!(WorkspacePaneSplitRight, CMD_WORKSPACE_PANE_SPLIT_RIGHT);
    workspace_unit_action!(WorkspacePaneSplitLeft, CMD_WORKSPACE_PANE_SPLIT_LEFT);
    workspace_unit_action!(WorkspacePaneSplitUp, CMD_WORKSPACE_PANE_SPLIT_UP);
    workspace_unit_action!(WorkspacePaneSplitDown, CMD_WORKSPACE_PANE_SPLIT_DOWN);

    workspace_unit_action!(WorkspacePaneFocusRight, CMD_WORKSPACE_PANE_FOCUS_RIGHT);
    workspace_unit_action!(WorkspacePaneFocusLeft, CMD_WORKSPACE_PANE_FOCUS_LEFT);
    workspace_unit_action!(WorkspacePaneFocusUp, CMD_WORKSPACE_PANE_FOCUS_UP);
    workspace_unit_action!(WorkspacePaneFocusDown, CMD_WORKSPACE_PANE_FOCUS_DOWN);

    workspace_unit_action!(
        WorkspacePaneMoveActiveTabRight,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT
    );
    workspace_unit_action!(
        WorkspacePaneMoveActiveTabLeft,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT
    );
    workspace_unit_action!(
        WorkspacePaneMoveActiveTabUp,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP
    );
    workspace_unit_action!(
        WorkspacePaneMoveActiveTabDown,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN
    );

    workspace_unit_action!(
        WorkspacePaneFocusTabStrip,
        CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP
    );
    workspace_unit_action!(WorkspacePaneFocusContent, CMD_WORKSPACE_PANE_FOCUS_CONTENT);
    workspace_unit_action!(
        WorkspacePaneToggleTabStripFocus,
        CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS
    );
    workspace_unit_action!(WorkspaceDirtyCloseCancel, CMD_WORKSPACE_DIRTY_CLOSE_CANCEL);
    workspace_unit_action!(
        WorkspaceDirtyCloseDiscard,
        CMD_WORKSPACE_DIRTY_CLOSE_DISCARD
    );
    workspace_unit_action!(
        WorkspaceDirtyCloseSaveAndClose,
        CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE
    );
}

pub fn typed_command_id<A: TypedAction>() -> CommandId {
    A::action_id()
}

pub const TYPED_WORKSPACE_COMMAND_IDS: &[&str] = &[
    CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
    CMD_WORKSPACE_TAB_CLOSE,
    CMD_WORKSPACE_TAB_CLOSE_OTHERS,
    CMD_WORKSPACE_TAB_CLOSE_LEFT,
    CMD_WORKSPACE_TAB_CLOSE_RIGHT,
    CMD_WORKSPACE_TAB_MOVE_LEFT,
    CMD_WORKSPACE_TAB_MOVE_RIGHT,
    CMD_WORKSPACE_TAB_TOGGLE_PIN,
    CMD_WORKSPACE_TAB_COMMIT_PREVIEW,
    CMD_WORKSPACE_PANE_NEXT,
    CMD_WORKSPACE_PANE_PREV,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV,
    CMD_WORKSPACE_PANE_RESIZE_RIGHT,
    CMD_WORKSPACE_PANE_RESIZE_LEFT,
    CMD_WORKSPACE_PANE_RESIZE_UP,
    CMD_WORKSPACE_PANE_RESIZE_DOWN,
    CMD_WORKSPACE_PANE_SPLIT_RIGHT,
    CMD_WORKSPACE_PANE_SPLIT_LEFT,
    CMD_WORKSPACE_PANE_SPLIT_UP,
    CMD_WORKSPACE_PANE_SPLIT_DOWN,
    CMD_WORKSPACE_PANE_FOCUS_RIGHT,
    CMD_WORKSPACE_PANE_FOCUS_LEFT,
    CMD_WORKSPACE_PANE_FOCUS_UP,
    CMD_WORKSPACE_PANE_FOCUS_DOWN,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP,
    CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN,
    CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP,
    CMD_WORKSPACE_PANE_FOCUS_CONTENT,
    CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS,
    CMD_WORKSPACE_DIRTY_CLOSE_CANCEL,
    CMD_WORKSPACE_DIRTY_CLOSE_DISCARD,
    CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE,
];

pub fn is_typed_workspace_command(command: &CommandId) -> bool {
    TYPED_WORKSPACE_COMMAND_IDS.contains(&command.as_str())
}

pub fn is_workspace_ui_command(command: &CommandId) -> bool {
    matches!(
        command.as_str(),
        CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP
            | CMD_WORKSPACE_PANE_FOCUS_CONTENT
            | CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS
    )
}

pub fn is_workspace_dirty_close_resolution(command: &CommandId) -> bool {
    matches!(
        command.as_str(),
        CMD_WORKSPACE_DIRTY_CLOSE_CANCEL
            | CMD_WORKSPACE_DIRTY_CLOSE_DISCARD
            | CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE
    )
}

pub fn is_workspace_model_command(command: &CommandId) -> bool {
    if is_typed_workspace_command(command) {
        return !is_workspace_ui_command(command) && !is_workspace_dirty_close_resolution(command);
    }

    [
        CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX,
        CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX,
        CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX,
        CMD_WORKSPACE_TAB_MOVE_BEFORE_ID_PREFIX,
        CMD_WORKSPACE_TAB_MOVE_AFTER_ID_PREFIX,
        CMD_WORKSPACE_TAB_ACTIVATE_PREFIX,
        CMD_WORKSPACE_TAB_CLOSE_PREFIX,
        CMD_WORKSPACE_TAB_PIN_PREFIX,
        CMD_WORKSPACE_TAB_UNPIN_PREFIX,
        CMD_WORKSPACE_PANE_ACTIVATE_PREFIX,
        CMD_WORKSPACE_PANE_SPLIT_PREFIX,
        CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_TO_PREFIX,
    ]
    .iter()
    .any(|prefix| {
        command
            .as_str()
            .strip_prefix(prefix)
            .is_some_and(|payload| !payload.trim().is_empty())
    })
}

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

/// Prefix for "pin a specific tab" commands.
///
/// Shape: `workspace.tab.pin.<tab_id>`
pub const CMD_WORKSPACE_TAB_PIN_PREFIX: &str = "workspace.tab.pin.";

/// Prefix for "unpin a specific tab" commands.
///
/// Shape: `workspace.tab.unpin.<tab_id>`
pub const CMD_WORKSPACE_TAB_UNPIN_PREFIX: &str = "workspace.tab.unpin.";

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

pub fn tab_pin_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_PIN_PREFIX}{id}"
    ))))
}

pub fn tab_unpin_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_UNPIN_PREFIX}{id}"
    ))))
}

pub fn tab_open_preview_command(id: &str) -> Option<CommandId> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX}{id}"
    ))))
}

pub fn tab_move_active_before_command(target_id: &str) -> Option<CommandId> {
    let id = target_id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_MOVE_BEFORE_PREFIX}{id}"
    ))))
}

pub fn tab_move_active_after_command(target_id: &str) -> Option<CommandId> {
    let id = target_id.trim();
    if id.is_empty() {
        return None;
    }
    Some(CommandId::new(Arc::<str>::from(format!(
        "{CMD_WORKSPACE_TAB_MOVE_AFTER_PREFIX}{id}"
    ))))
}

fn tab_move_specific_command(prefix: &str, dragged_id: &str, target_id: &str) -> Option<CommandId> {
    let dragged = dragged_id.trim();
    let target = target_id.trim();
    if dragged.is_empty() || target.is_empty() {
        return None;
    }

    Some(CommandId::new(Arc::<str>::from(format!(
        "{prefix}{}:{dragged}{target}",
        dragged.len()
    ))))
}

pub fn tab_move_before_tab_command(dragged_id: &str, target_id: &str) -> Option<CommandId> {
    tab_move_specific_command(
        CMD_WORKSPACE_TAB_MOVE_BEFORE_ID_PREFIX,
        dragged_id,
        target_id,
    )
}

pub fn tab_move_after_tab_command(dragged_id: &str, target_id: &str) -> Option<CommandId> {
    tab_move_specific_command(
        CMD_WORKSPACE_TAB_MOVE_AFTER_ID_PREFIX,
        dragged_id,
        target_id,
    )
}

pub fn parse_tab_move_specific_payload(payload: &str) -> Option<(&str, &str)> {
    let (len, rest) = payload.split_once(':')?;
    let dragged_len = len.parse::<usize>().ok()?;
    if dragged_len == 0 || dragged_len >= rest.len() || !rest.is_char_boundary(dragged_len) {
        return None;
    }

    let (dragged, target) = rest.split_at(dragged_len);
    let dragged = dragged.trim();
    let target = target.trim();
    if dragged.is_empty() || target.is_empty() {
        return None;
    }

    Some((dragged, target))
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
        sequence: vec![KeyChord::new(key, mods)],
        when: None,
    }
}

fn seq(platform: PlatformFilter, sequence: Vec<KeyChord>) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        sequence,
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
    let mac_cmd_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        )
    };

    let win_ctrl_k = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let linux_ctrl_k = win_ctrl_k;
    let mac_cmd_k = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabNext>(),
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
        typed_command_id::<act::WorkspaceTabPrev>(),
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
        typed_command_id::<act::WorkspaceTabClose>(),
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
        typed_command_id::<act::WorkspaceTabCloseOthers>(),
        CommandMeta::new("Close Other Tabs")
            .with_category("Workspace")
            .with_keywords(["tab", "close", "others", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabCloseLeft>(),
        CommandMeta::new("Close Tabs to the Left")
            .with_category("Workspace")
            .with_keywords(["tab", "close", "left", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabCloseRight>(),
        CommandMeta::new("Close Tabs to the Right")
            .with_category("Workspace")
            .with_keywords(["tab", "close", "right", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabCommitPreview>(),
        CommandMeta::new("Commit Preview Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "preview", "commit", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabMoveLeft>(),
        CommandMeta::new("Move Tab Left")
            .with_category("Workspace")
            .with_keywords(["tab", "move", "left", "reorder", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                ctrl: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                ctrl: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                kb(
                    PlatformFilter::Macos,
                    KeyCode::ArrowLeft,
                    Modifiers {
                        meta: true,
                        alt: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabMoveRight>(),
        CommandMeta::new("Move Tab Right")
            .with_category("Workspace")
            .with_keywords(["tab", "move", "right", "reorder", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                ctrl: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                ctrl: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                kb(
                    PlatformFilter::Macos,
                    KeyCode::ArrowRight,
                    Modifiers {
                        meta: true,
                        alt: true,
                        shift: true,
                        ..Default::default()
                    },
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceTabTogglePin>(),
        CommandMeta::new("Toggle Tab Pin")
            .with_category("Workspace")
            .with_keywords(["tab", "pin", "unpin", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneNext>(),
        CommandMeta::new("Next Pane")
            .with_category("Workspace")
            .with_keywords(["pane", "next", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePanePrev>(),
        CommandMeta::new("Previous Pane")
            .with_category("Workspace")
            .with_keywords(["pane", "previous", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabNext>(),
        CommandMeta::new("Move Active Tab to Next Pane")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "next", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabPrev>(),
        CommandMeta::new("Move Active Tab to Previous Pane")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "previous", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusTabStrip>(),
        CommandMeta::new("Focus Tab Strip")
            .with_category("Workspace")
            .with_keywords(["focus", "tab", "tabstrip", "pane", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusContent>(),
        CommandMeta::new("Focus Pane Content")
            .with_category("Workspace")
            .with_keywords(["focus", "content", "pane", "workspace"]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneToggleTabStripFocus>(),
        CommandMeta::new("Toggle Tab Strip Focus")
            .with_category("Workspace")
            .with_keywords(["toggle", "focus", "tab", "tabstrip", "pane", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::F6, false),
                linux_ctrl(KeyCode::F6, false),
                mac_ctrl(KeyCode::F6, false),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneResizeRight>(),
        CommandMeta::new("Resize Pane Right")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "right", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                alt: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneResizeLeft>(),
        CommandMeta::new("Resize Pane Left")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "left", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                alt: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneResizeUp>(),
        CommandMeta::new("Resize Pane Up")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "up", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                alt: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneResizeDown>(),
        CommandMeta::new("Resize Pane Down")
            .with_category("Workspace")
            .with_keywords(["resize", "pane", "down", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                ctrl: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                alt: true,
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneSplitRight>(),
        CommandMeta::new("Split Pane Right")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "right", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Backslash, false),
                linux_ctrl(KeyCode::Backslash, false),
                mac_cmd(KeyCode::Backslash),
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(KeyCode::ArrowRight, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(KeyCode::ArrowRight, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(KeyCode::ArrowRight, Modifiers::default()),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneSplitLeft>(),
        CommandMeta::new("Split Pane Left")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "left", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(KeyCode::ArrowLeft, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(KeyCode::ArrowLeft, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(KeyCode::ArrowLeft, Modifiers::default()),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneSplitUp>(),
        CommandMeta::new("Split Pane Up")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "up", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(KeyCode::ArrowUp, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(KeyCode::ArrowUp, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(KeyCode::ArrowUp, Modifiers::default()),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneSplitDown>(),
        CommandMeta::new("Split Pane Down")
            .with_category("Workspace")
            .with_keywords(["split", "pane", "down", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Backslash, true),
                linux_ctrl(KeyCode::Backslash, true),
                mac_cmd_shift(KeyCode::Backslash),
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(KeyCode::ArrowDown, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(KeyCode::ArrowDown, Modifiers::default()),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(KeyCode::ArrowDown, Modifiers::default()),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusRight>(),
        CommandMeta::new("Focus Pane Right")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "right", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusLeft>(),
        CommandMeta::new("Focus Pane Left")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "left", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusUp>(),
        CommandMeta::new("Focus Pane Up")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "up", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneFocusDown>(),
        CommandMeta::new("Focus Pane Down")
            .with_category("Workspace")
            .with_keywords(["focus", "pane", "down", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                shift: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabRight>(),
        CommandMeta::new("Move Active Tab Right")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "right", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowRight,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabLeft>(),
        CommandMeta::new("Move Active Tab Left")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "left", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowLeft,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabUp>(),
        CommandMeta::new("Move Active Tab Up")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "up", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowUp,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspacePaneMoveActiveTabDown>(),
        CommandMeta::new("Move Active Tab Down")
            .with_category("Workspace")
            .with_keywords(["move", "tab", "pane", "down", "workspace"])
            .with_default_keybindings([
                seq(
                    PlatformFilter::Windows,
                    vec![
                        win_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Linux,
                    vec![
                        linux_ctrl_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
                seq(
                    PlatformFilter::Macos,
                    vec![
                        mac_cmd_k,
                        KeyChord::new(
                            KeyCode::ArrowDown,
                            Modifiers {
                                alt: true,
                                ..Default::default()
                            },
                        ),
                    ],
                ),
            ]),
    );

    registry.register(
        typed_command_id::<act::WorkspaceDirtyCloseCancel>(),
        CommandMeta::new("Cancel Dirty Close")
            .with_category("Workspace")
            .with_keywords(["dirty", "close", "cancel", "workspace"]),
    );
    registry.register(
        typed_command_id::<act::WorkspaceDirtyCloseDiscard>(),
        CommandMeta::new("Discard and Close")
            .with_category("Workspace")
            .with_keywords(["dirty", "close", "discard", "workspace"]),
    );
    registry.register(
        typed_command_id::<act::WorkspaceDirtyCloseSaveAndClose>(),
        CommandMeta::new("Save and Close")
            .with_category("Workspace")
            .with_keywords(["dirty", "close", "save", "workspace"]),
    );
}
