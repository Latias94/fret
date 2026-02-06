use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord, OsAction,
    PlatformFilter, WhenExpr,
};

pub const COMMAND_PALETTE: &str = "app.command_palette";
pub const COMMAND_PALETTE_LEGACY: &str = "command_palette.toggle";
pub const APP_ABOUT: &str = "app.about";
pub const APP_PREFERENCES: &str = "app.preferences";
pub const APP_QUIT: &str = "app.quit";
pub const APP_HIDE: &str = "app.hide";
pub const APP_HIDE_OTHERS: &str = "app.hide_others";
pub const APP_SHOW_ALL: &str = "app.show_all";
pub const FOCUS_NEXT: &str = "focus.next";
pub const FOCUS_PREVIOUS: &str = "focus.previous";
pub const FOCUS_MENU_BAR: &str = "focus.menu_bar";

pub const EDIT_COPY: &str = "edit.copy";
pub const EDIT_CUT: &str = "edit.cut";
pub const EDIT_PASTE: &str = "edit.paste";
pub const EDIT_SELECT_ALL: &str = "edit.select_all";

pub const TEXT_COPY: &str = "text.copy";
pub const TEXT_CUT: &str = "text.cut";
pub const TEXT_PASTE: &str = "text.paste";
pub const TEXT_SELECT_ALL: &str = "text.select_all";
pub const EDIT_UNDO: &str = "edit.undo";
pub const EDIT_REDO: &str = "edit.redo";

pub fn register_core_commands(registry: &mut CommandRegistry) {
    register_command_palette(registry);
    register_legacy_command_palette_alias(registry);
    register_app_commands(registry);
    register_focus_commands(registry);
    register_text_edit_commands(registry);
}

pub fn register_app_commands(registry: &mut CommandRegistry) {
    let ctrl_mods = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let meta_mods = Modifiers {
        meta: true,
        ..Default::default()
    };
    let meta_alt_mods = Modifiers {
        meta: true,
        alt: true,
        ..Default::default()
    };

    registry.register(
        CommandId::new(APP_ABOUT),
        CommandMeta::new("About")
            .with_category("App")
            .with_keywords(["about", "version"]),
    );

    registry.register(
        CommandId::new(APP_PREFERENCES),
        CommandMeta::new("Preferences…")
            .with_category("App")
            .with_keywords(["preferences", "settings"])
            .with_default_keybindings([DefaultKeybinding {
                platform: PlatformFilter::Macos,
                sequence: vec![KeyChord::new(KeyCode::Comma, meta_mods)],
                when: None,
            }]),
    );

    registry.register(
        CommandId::new(APP_HIDE),
        CommandMeta::new("Hide")
            .with_category("App")
            .with_keywords(["hide", "minimize"])
            .with_default_keybindings([DefaultKeybinding {
                platform: PlatformFilter::Macos,
                sequence: vec![KeyChord::new(KeyCode::KeyH, meta_mods)],
                when: None,
            }]),
    );

    registry.register(
        CommandId::new(APP_HIDE_OTHERS),
        CommandMeta::new("Hide Others")
            .with_category("App")
            .with_keywords(["hide", "others"])
            .with_default_keybindings([DefaultKeybinding {
                platform: PlatformFilter::Macos,
                sequence: vec![KeyChord::new(KeyCode::KeyH, meta_alt_mods)],
                when: None,
            }]),
    );

    registry.register(
        CommandId::new(APP_SHOW_ALL),
        CommandMeta::new("Show All")
            .with_category("App")
            .with_keywords(["show", "unhide", "all"]),
    );

    registry.register(
        CommandId::new(APP_QUIT),
        CommandMeta::new("Quit")
            .with_category("App")
            .with_keywords(["quit", "exit"])
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::KeyQ, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::KeyQ, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(KeyCode::KeyQ, meta_mods)],
                    when: None,
                },
            ]),
    );
}

pub fn register_command_palette(registry: &mut CommandRegistry) {
    let ctrl_mods = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let meta_mods = Modifiers {
        meta: true,
        ..Default::default()
    };

    let meta = CommandMeta::new("Command Palette")
        .with_category("App")
        .with_keywords(["command palette", "commands", "palette", "search"])
        .with_default_keybindings([
            DefaultKeybinding {
                platform: PlatformFilter::Windows,
                sequence: vec![KeyChord::new(KeyCode::KeyP, ctrl_mods)],
                when: None,
            },
            DefaultKeybinding {
                platform: PlatformFilter::Linux,
                sequence: vec![KeyChord::new(KeyCode::KeyP, ctrl_mods)],
                when: None,
            },
            DefaultKeybinding {
                platform: PlatformFilter::Macos,
                sequence: vec![KeyChord::new(KeyCode::KeyP, meta_mods)],
                when: None,
            },
        ]);

    registry.register(CommandId::new(COMMAND_PALETTE), meta);
}

pub fn register_legacy_command_palette_alias(registry: &mut CommandRegistry) {
    registry.register(
        CommandId::new(COMMAND_PALETTE_LEGACY),
        CommandMeta::new("Command Palette").hidden(),
    );
}

pub fn register_focus_commands(registry: &mut CommandRegistry) {
    registry.register(
        CommandId::new(FOCUS_NEXT),
        CommandMeta::new("Focus Next")
            .with_category("Focus")
            .with_keywords(["focus", "tab", "next"])
            .with_scope(CommandScope::Widget),
    );
    registry.register(
        CommandId::new(FOCUS_PREVIOUS),
        CommandMeta::new("Focus Previous")
            .with_category("Focus")
            .with_keywords(["focus", "tab", "previous"])
            .with_scope(CommandScope::Widget),
    );

    let when_not_text = WhenExpr::parse("!focus.is_text_input").expect("valid when expression");

    registry.register(
        CommandId::new(FOCUS_MENU_BAR),
        CommandMeta::new("Focus Menu Bar")
            .with_category("Focus")
            .with_keywords(["focus", "menu", "menubar"])
            .with_scope(CommandScope::Widget)
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::F10, Modifiers::default())],
                    when: Some(when_not_text.clone()),
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::F10, Modifiers::default())],
                    when: Some(when_not_text),
                },
            ]),
    );
}

pub fn register_text_edit_commands(registry: &mut CommandRegistry) {
    let ctrl_mods = Modifiers {
        ctrl: true,
        ..Default::default()
    };
    let meta_mods = Modifiers {
        meta: true,
        ..Default::default()
    };

    let when_text = WhenExpr::parse("focus.is_text_input").expect("valid when expression");

    registry.register(
        CommandId::new(EDIT_COPY),
        CommandMeta::new("Copy")
            .with_category("Edit")
            .with_keywords(["copy", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Copy)
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::KeyC, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::KeyC, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(KeyCode::KeyC, meta_mods)],
                    when: None,
                },
            ]),
    );

    registry.register(
        CommandId::new(EDIT_CUT),
        CommandMeta::new("Cut")
            .with_category("Edit")
            .with_keywords(["cut", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Cut)
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::KeyX, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::KeyX, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(KeyCode::KeyX, meta_mods)],
                    when: None,
                },
            ]),
    );

    registry.register(
        CommandId::new(EDIT_PASTE),
        CommandMeta::new("Paste")
            .with_category("Edit")
            .with_keywords(["paste", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Paste)
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::KeyV, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::KeyV, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(KeyCode::KeyV, meta_mods)],
                    when: None,
                },
            ]),
    );

    registry.register(
        CommandId::new(EDIT_SELECT_ALL),
        CommandMeta::new("Select All")
            .with_category("Edit")
            .with_keywords(["select", "all"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::SelectAll)
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(KeyCode::KeyA, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(KeyCode::KeyA, ctrl_mods)],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(KeyCode::KeyA, meta_mods)],
                    when: None,
                },
            ]),
    );

    // Legacy `text.*` commands: keep for compatibility, but prefer `edit.*` for cross-surface copy.
    registry.register(
        CommandId::new(TEXT_COPY),
        CommandMeta::new("Copy")
            .with_category("Edit")
            .with_keywords(["copy", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Copy)
            .hidden(),
    );

    registry.register(
        CommandId::new(TEXT_CUT),
        CommandMeta::new("Cut")
            .with_category("Edit")
            .with_keywords(["cut", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Cut)
            .with_when(when_text.clone())
            .hidden(),
    );

    registry.register(
        CommandId::new(TEXT_PASTE),
        CommandMeta::new("Paste")
            .with_category("Edit")
            .with_keywords(["paste", "clipboard"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::Paste)
            .with_when(when_text.clone())
            .hidden(),
    );

    registry.register(
        CommandId::new(TEXT_SELECT_ALL),
        CommandMeta::new("Select All")
            .with_category("Edit")
            .with_keywords(["select", "all"])
            .with_scope(CommandScope::Widget)
            .with_os_action(OsAction::SelectAll)
            .hidden(),
    );

    registry.register(
        CommandId::new(EDIT_UNDO),
        CommandMeta::new("Undo")
            .with_category("Edit")
            .with_keywords(["undo", "history"])
            .with_when(WhenExpr::parse("edit.can_undo").expect("valid when expression"))
            .with_os_action(OsAction::Undo),
    );

    registry.register(
        CommandId::new(EDIT_REDO),
        CommandMeta::new("Redo")
            .with_category("Edit")
            .with_keywords(["redo", "history"])
            .with_when(WhenExpr::parse("edit.can_redo").expect("valid when expression"))
            .with_os_action(OsAction::Redo),
    );
}
