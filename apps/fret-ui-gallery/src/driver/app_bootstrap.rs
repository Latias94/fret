use fret_app::{App, CommandId, CommandMeta, Effect};
use fret_core::{KeyCode, Modifiers};
use fret_runtime::{DefaultKeybinding, KeyChord, PlatformFilter};

use crate::spec::*;

use super::UiGalleryDriver;

pub(super) fn register_commands_and_menus(app: &mut App) {
    // Minimal command surface for `CommandDialog::new_with_host_commands`.
    fret_app::core_commands::register_core_commands(app.commands_mut());
    app.commands_mut().register(
        CommandId::new(CMD_APP_OPEN),
        CommandMeta::new("Open")
            .with_category("File")
            .with_keywords(["open", "file"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SAVE),
        CommandMeta::new("Save")
            .with_category("File")
            .with_keywords(["save", "file"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SETTINGS),
        CommandMeta::new("Settings")
            .with_category("View")
            .with_keywords(["settings", "preferences"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SETTINGS_APPLY),
        CommandMeta::new("Apply Settings")
            .with_category("Settings")
            .with_keywords(["settings", "menu", "menubar", "apply"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_SETTINGS_WRITE_PROJECT),
        CommandMeta::new("Write Project Settings")
            .with_category("Settings")
            .with_keywords(["settings", "menu", "menubar", "write", "project"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_APP_TOGGLE_PREFERENCES_ENABLED),
        CommandMeta::new("Toggle Preferences Enabled (debug)")
            .with_category("Settings")
            .with_keywords(["preferences", "menubar", "enabled", "disable", "debug"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_OS_AUTO),
        CommandMeta::new("Menu Bar (OS): Auto")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "os", "auto"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_OS_ON),
        CommandMeta::new("Menu Bar (OS): On")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "os", "on"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_OS_OFF),
        CommandMeta::new("Menu Bar (OS): Off")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "os", "off"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_IN_WINDOW_AUTO),
        CommandMeta::new("Menu Bar (In-window): Auto")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "in-window", "auto"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_IN_WINDOW_ON),
        CommandMeta::new("Menu Bar (In-window): On")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "in-window", "on"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_MENU_BAR_IN_WINDOW_OFF),
        CommandMeta::new("Menu Bar (In-window): Off")
            .with_category("Settings")
            .with_keywords(["menu", "menubar", "in-window", "off"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_GALLERY_PAGE_BACK),
        CommandMeta::new("Page Back")
            .with_category("Gallery")
            .with_keywords(["gallery", "page", "back", "history", "navigate"])
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(
                        KeyCode::ArrowLeft,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(
                        KeyCode::ArrowLeft,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(
                        KeyCode::BracketLeft,
                        Modifiers {
                            meta: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
            ]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_GALLERY_PAGE_FORWARD),
        CommandMeta::new("Page Forward")
            .with_category("Gallery")
            .with_keywords(["gallery", "page", "forward", "history", "navigate"])
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(
                        KeyCode::ArrowRight,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(
                        KeyCode::ArrowRight,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(
                        KeyCode::BracketRight,
                        Modifiers {
                            meta: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
            ]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_GALLERY_DEBUG_RECENT_ADD),
        CommandMeta::new("Debug: Recent (add item)")
            .with_category("Debug")
            .with_keywords(["debug", "menu", "menubar", "recent", "add"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_GALLERY_DEBUG_RECENT_CLEAR),
        CommandMeta::new("Debug: Recent (clear)")
            .with_category("Debug")
            .with_keywords(["debug", "menu", "menubar", "recent", "clear"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_GALLERY_DEBUG_WINDOW_OPEN),
        CommandMeta::new("Debug: Window (open)")
            .with_category("Debug")
            .with_keywords(["debug", "menu", "menubar", "window", "open"]),
    );

    for group in PAGE_GROUPS {
        for page in group.items {
            let mut keywords: Vec<&'static str> = Vec::new();
            keywords.push(page.id);
            keywords.push(page.origin);
            keywords.extend_from_slice(page.tags);

            app.commands_mut().register(
                CommandId::new(page.command),
                CommandMeta::new(page.label)
                    .with_category(format!("Gallery: {}", group.title))
                    .with_keywords(keywords),
            );
        }
    }

    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_LINK),
        CommandMeta::new("Copy page link")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "link", "page"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_USAGE),
        CommandMeta::new("Copy usage")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "usage", "code"]),
    );
    app.commands_mut().register(
        CommandId::new(CMD_CLIPBOARD_COPY_NOTES),
        CommandMeta::new("Copy notes")
            .with_category("Gallery")
            .with_keywords(["copy", "clipboard", "notes", "docs"]),
    );

    fret_workspace::commands::register_workspace_commands(app.commands_mut());
    fret_app::install_command_default_keybindings_into_keymap(app);
    UiGalleryDriver::sync_dynamic_menu_command_metadata(app);
    app.push_effect(Effect::SetMenuBar {
        window: None,
        menu_bar: UiGalleryDriver::build_menu_bar(app),
    });
}
