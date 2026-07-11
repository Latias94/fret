use fret_app::{App, install_command_default_keybindings_into_keymap};
use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandRegistry, InputContext, KeyChord, KeymapService, Platform, TypedAction,
};
use fret_workspace::commands::{self, act, register_workspace_commands, typed_command_id};
use fret_workspace::menu::{WorkspaceMenuCommands, workspace_default_menu_bar};

fn assert_action<A: TypedAction>(expected: &'static str) -> CommandId {
    let id = A::action_id();
    assert_eq!(id.as_str(), expected);
    id
}

fn static_workspace_action_ids() -> Vec<CommandId> {
    vec![
        assert_action::<act::WorkspaceTabNext>(commands::CMD_WORKSPACE_TAB_NEXT),
        assert_action::<act::WorkspaceTabPrev>(commands::CMD_WORKSPACE_TAB_PREV),
        assert_action::<act::WorkspaceTabClose>(commands::CMD_WORKSPACE_TAB_CLOSE),
        assert_action::<act::WorkspaceTabCloseOthers>(commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS),
        assert_action::<act::WorkspaceTabCloseLeft>(commands::CMD_WORKSPACE_TAB_CLOSE_LEFT),
        assert_action::<act::WorkspaceTabCloseRight>(commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT),
        assert_action::<act::WorkspaceTabMoveLeft>(commands::CMD_WORKSPACE_TAB_MOVE_LEFT),
        assert_action::<act::WorkspaceTabMoveRight>(commands::CMD_WORKSPACE_TAB_MOVE_RIGHT),
        assert_action::<act::WorkspaceTabTogglePin>(commands::CMD_WORKSPACE_TAB_TOGGLE_PIN),
        assert_action::<act::WorkspaceTabCommitPreview>(commands::CMD_WORKSPACE_TAB_COMMIT_PREVIEW),
        assert_action::<act::WorkspacePaneNext>(commands::CMD_WORKSPACE_PANE_NEXT),
        assert_action::<act::WorkspacePanePrev>(commands::CMD_WORKSPACE_PANE_PREV),
        assert_action::<act::WorkspacePaneMoveActiveTabNext>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT,
        ),
        assert_action::<act::WorkspacePaneMoveActiveTabPrev>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV,
        ),
        assert_action::<act::WorkspacePaneResizeRight>(commands::CMD_WORKSPACE_PANE_RESIZE_RIGHT),
        assert_action::<act::WorkspacePaneResizeLeft>(commands::CMD_WORKSPACE_PANE_RESIZE_LEFT),
        assert_action::<act::WorkspacePaneResizeUp>(commands::CMD_WORKSPACE_PANE_RESIZE_UP),
        assert_action::<act::WorkspacePaneResizeDown>(commands::CMD_WORKSPACE_PANE_RESIZE_DOWN),
        assert_action::<act::WorkspacePaneSplitRight>(commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT),
        assert_action::<act::WorkspacePaneSplitLeft>(commands::CMD_WORKSPACE_PANE_SPLIT_LEFT),
        assert_action::<act::WorkspacePaneSplitUp>(commands::CMD_WORKSPACE_PANE_SPLIT_UP),
        assert_action::<act::WorkspacePaneSplitDown>(commands::CMD_WORKSPACE_PANE_SPLIT_DOWN),
        assert_action::<act::WorkspacePaneFocusRight>(commands::CMD_WORKSPACE_PANE_FOCUS_RIGHT),
        assert_action::<act::WorkspacePaneFocusLeft>(commands::CMD_WORKSPACE_PANE_FOCUS_LEFT),
        assert_action::<act::WorkspacePaneFocusUp>(commands::CMD_WORKSPACE_PANE_FOCUS_UP),
        assert_action::<act::WorkspacePaneFocusDown>(commands::CMD_WORKSPACE_PANE_FOCUS_DOWN),
        assert_action::<act::WorkspacePaneMoveActiveTabRight>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT,
        ),
        assert_action::<act::WorkspacePaneMoveActiveTabLeft>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT,
        ),
        assert_action::<act::WorkspacePaneMoveActiveTabUp>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP,
        ),
        assert_action::<act::WorkspacePaneMoveActiveTabDown>(
            commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN,
        ),
        assert_action::<act::WorkspacePaneFocusTabStrip>(
            commands::CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP,
        ),
        assert_action::<act::WorkspacePaneFocusContent>(commands::CMD_WORKSPACE_PANE_FOCUS_CONTENT),
        assert_action::<act::WorkspacePaneToggleTabStripFocus>(
            commands::CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS,
        ),
        assert_action::<act::WorkspaceDirtyCloseCancel>(commands::CMD_WORKSPACE_DIRTY_CLOSE_CANCEL),
        assert_action::<act::WorkspaceDirtyCloseDiscard>(
            commands::CMD_WORKSPACE_DIRTY_CLOSE_DISCARD,
        ),
        assert_action::<act::WorkspaceDirtyCloseSaveAndClose>(
            commands::CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE,
        ),
    ]
}

fn collect_menu_commands(items: &[fret_runtime::MenuItem], out: &mut Vec<CommandId>) {
    for item in items {
        match item {
            fret_runtime::MenuItem::Command { command, .. } => out.push(command.clone()),
            fret_runtime::MenuItem::Submenu { items, .. } => collect_menu_commands(items, out),
            _ => {}
        }
    }
}

#[test]
fn typed_workspace_actions_match_canonical_command_ids() {
    let ids = static_workspace_action_ids();
    for id in ids {
        assert!(id.as_str().starts_with("workspace."));
    }
}

#[test]
fn dirty_close_actions_keep_their_resolution_lane_classification() {
    for id in [
        typed_command_id::<act::WorkspaceDirtyCloseCancel>(),
        typed_command_id::<act::WorkspaceDirtyCloseDiscard>(),
        typed_command_id::<act::WorkspaceDirtyCloseSaveAndClose>(),
    ] {
        assert!(commands::is_typed_workspace_command(&id));
        assert!(!commands::is_workspace_model_command(&id));
        assert!(!commands::is_workspace_ui_command(&id));
        assert!(commands::is_workspace_dirty_close_resolution(&id));
    }
}

#[test]
fn every_typed_workspace_command_has_one_routing_lane() {
    for id in static_workspace_action_ids() {
        let lane_count = [
            commands::is_workspace_model_command(&id),
            commands::is_workspace_ui_command(&id),
            commands::is_workspace_dirty_close_resolution(&id),
        ]
        .into_iter()
        .filter(|matches| *matches)
        .count();
        assert_eq!(
            lane_count,
            1,
            "workspace command `{}` must have one lane",
            id.as_str()
        );
    }
}

#[test]
fn typed_workspace_commands_round_trip_registry_keymap_and_menu() {
    let toggle = typed_command_id::<act::WorkspacePaneToggleTabStripFocus>();

    let mut registry = CommandRegistry::default();
    register_workspace_commands(&mut registry);
    for id in static_workspace_action_ids() {
        registry
            .get(id.clone())
            .unwrap_or_else(|| panic!("workspace command `{}` should be registered", id.as_str()));
    }

    let mut app = App::new();
    register_workspace_commands(app.commands_mut());
    install_command_default_keybindings_into_keymap(&mut app);
    let keymap = &app
        .global::<KeymapService>()
        .expect("default keymap service should be installed")
        .keymap;
    let ctrl_f6 = KeyChord::new(
        KeyCode::F6,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let ctx = InputContext {
        platform: Platform::Macos,
        ..Default::default()
    };
    assert_eq!(keymap.resolve(&ctx, ctrl_f6), Some(toggle.clone()));

    let menu_bar = workspace_default_menu_bar(WorkspaceMenuCommands::default());
    let mut menu_commands = Vec::new();
    for menu in &menu_bar.menus {
        collect_menu_commands(&menu.items, &mut menu_commands);
    }
    assert!(menu_commands.contains(&typed_command_id::<act::WorkspaceTabMoveLeft>()));
    assert!(menu_commands.contains(&typed_command_id::<act::WorkspacePaneMoveActiveTabRight>()));
    assert!(menu_commands.contains(&typed_command_id::<act::WorkspacePaneFocusDown>()));
}
