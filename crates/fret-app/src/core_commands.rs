use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord, OsAction,
    PlatformFilter, WhenExpr,
};

pub const COMMAND_PALETTE: &str = "app.command_palette";
pub const COMMAND_PALETTE_LEGACY: &str = "command_palette.toggle";
pub const APP_ABOUT: &str = "app.about";
pub const APP_PREFERENCES: &str = "app.preferences";
pub const APP_LOCALE_SWITCH_NEXT: &str = "app.locale.switch_next";
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
        CommandId::new(APP_LOCALE_SWITCH_NEXT),
        CommandMeta::new("Switch Language")
            .with_category("App")
            .with_keywords(["locale", "language", "i18n", "translation"])
            .with_default_keybindings([
                DefaultKeybinding {
                    platform: PlatformFilter::Windows,
                    sequence: vec![KeyChord::new(
                        KeyCode::KeyL,
                        Modifiers {
                            ctrl: true,
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Linux,
                    sequence: vec![KeyChord::new(
                        KeyCode::KeyL,
                        Modifiers {
                            ctrl: true,
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
                DefaultKeybinding {
                    platform: PlatformFilter::Macos,
                    sequence: vec![KeyChord::new(
                        KeyCode::KeyL,
                        Modifiers {
                            meta: true,
                            alt: true,
                            ..Default::default()
                        },
                    )],
                    when: None,
                },
            ]),
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

const CORE_COMMAND_CATEGORY_APP_KEY: &str = "core-command-category-app";

const CORE_COMMAND_LOCALIZATION_SPECS: &[(&str, &str, Option<&str>)] = &[
    (
        COMMAND_PALETTE,
        "core-command-title-app-command-palette",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_ABOUT,
        "core-command-title-app-about",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_PREFERENCES,
        "core-command-title-app-preferences",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_LOCALE_SWITCH_NEXT,
        "core-command-title-app-locale-switch-next",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_HIDE,
        "core-command-title-app-hide",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_HIDE_OTHERS,
        "core-command-title-app-hide-others",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_SHOW_ALL,
        "core-command-title-app-show-all",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
    (
        APP_QUIT,
        "core-command-title-app-quit",
        Some(CORE_COMMAND_CATEGORY_APP_KEY),
    ),
];

pub fn apply_core_command_localization(app: &mut crate::App) {
    let Some(service) = app
        .global::<fret_runtime::fret_i18n::I18nService>()
        .cloned()
    else {
        return;
    };

    let mut updates: Vec<(CommandId, CommandMeta)> = Vec::new();
    for (command, title_key, category_key) in CORE_COMMAND_LOCALIZATION_SPECS {
        let id = CommandId::new(*command);
        let Some(mut meta) = app.commands().get(id.clone()).cloned() else {
            continue;
        };

        meta.title = localized_or_fallback(&service, title_key, meta.title.clone());

        if let Some(category_key) = category_key {
            let fallback = meta.category.clone().unwrap_or_else(|| Arc::from("App"));
            meta.category = Some(localized_or_fallback(&service, category_key, fallback));
        }

        updates.push((id, meta));
    }

    for (id, meta) in updates {
        app.commands_mut().register(id, meta);
    }
}

pub fn handle_locale_cycle_command(app: &mut crate::App, command: &CommandId) -> bool {
    if command.as_str() != APP_LOCALE_SWITCH_NEXT {
        return false;
    }

    let mut service = app
        .global::<fret_runtime::fret_i18n::I18nService>()
        .cloned()
        .unwrap_or_default();
    let mut locales = service.preferred_locales().to_vec();
    if locales.is_empty() {
        locales.push(fret_runtime::fret_i18n::LocaleId::default());
    }
    if locales.len() == 1 {
        let zh_cn = fret_runtime::fret_i18n::LocaleId::parse("zh-CN")
            .expect("hardcoded locale zh-CN must parse");
        let en_us = fret_runtime::fret_i18n::LocaleId::parse("en-US")
            .expect("hardcoded locale en-US must parse");
        let alt = if locales[0] == zh_cn { en_us } else { zh_cn };
        locales.push(alt);
    }

    locales.rotate_left(1);
    service.set_preferred_locales(locales.clone());
    app.set_global(service);

    if let Some(mut settings) = app.global::<crate::SettingsFileV1>().cloned()
        && let Some((primary, fallbacks)) = locales.split_first()
    {
        settings.locale.primary = primary.to_string();
        settings.locale.fallbacks = fallbacks.iter().map(ToString::to_string).collect();
        app.set_global(settings);
    }

    apply_core_command_localization(app);
    true
}

fn localized_or_fallback(
    service: &fret_runtime::fret_i18n::I18nService,
    key: &str,
    fallback: Arc<str>,
) -> Arc<str> {
    let value = service.t(key.to_string());
    if value == key {
        fallback
    } else {
        Arc::from(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;

    use fret_runtime::fret_i18n::{I18nLookup, I18nLookupError, LocalizedMessage, MessageKey};

    struct TestLookup;

    impl I18nLookup for TestLookup {
        fn format(
            &self,
            preferred_locales: &[fret_runtime::fret_i18n::LocaleId],
            key: &MessageKey,
            _args: Option<&fret_runtime::fret_i18n::MessageArgs>,
        ) -> Result<LocalizedMessage, I18nLookupError> {
            for (depth, locale) in preferred_locales.iter().enumerate() {
                let value = match (locale.to_string().as_str(), key.as_str()) {
                    ("zh-CN", "core-command-title-app-about") => Some("关于"),
                    ("zh-CN", "core-command-title-app-locale-switch-next") => Some("切换语言"),
                    ("en-US", "core-command-title-app-about") => Some("About"),
                    ("en-US", "core-command-title-app-locale-switch-next") => {
                        Some("Switch Language")
                    }
                    _ => None,
                };
                if let Some(text) = value {
                    return Ok(LocalizedMessage {
                        text: text.to_string(),
                        locale: locale.clone(),
                        fallback_depth: depth,
                    });
                }
            }

            Err(I18nLookupError::MissingKey { key: key.clone() })
        }
    }

    #[test]
    fn core_command_localization_uses_current_locale() {
        let mut app = crate::App::new();
        let mut service = app
            .global::<fret_runtime::fret_i18n::I18nService>()
            .cloned()
            .unwrap_or_default();
        service.set_lookup(Some(Rc::new(TestLookup)));
        app.set_global(service);

        let mut settings = crate::SettingsFileV1::default();
        settings.locale.primary = "zh-CN".to_string();
        settings.locale.fallbacks = vec!["en-US".to_string()];
        crate::settings::apply_settings_globals(&mut app, &settings);

        let about = app
            .commands()
            .get(CommandId::new(APP_ABOUT))
            .expect("app.about should be registered");
        assert_eq!(about.title.as_ref(), "关于");

        let switch = app
            .commands()
            .get(CommandId::new(APP_LOCALE_SWITCH_NEXT))
            .expect("app.locale.switch_next should be registered");
        assert_eq!(switch.title.as_ref(), "切换语言");
    }

    #[test]
    fn locale_cycle_command_rotates_locale_and_relocalizes() {
        let mut app = crate::App::new();
        let mut service = app
            .global::<fret_runtime::fret_i18n::I18nService>()
            .cloned()
            .unwrap_or_default();
        service.set_lookup(Some(Rc::new(TestLookup)));
        app.set_global(service);

        let mut settings = crate::SettingsFileV1::default();
        settings.locale.primary = "en-US".to_string();
        settings.locale.fallbacks = vec!["zh-CN".to_string()];
        crate::settings::apply_settings_globals(&mut app, &settings);

        let before = app
            .commands()
            .get(CommandId::new(APP_ABOUT))
            .expect("app.about should be registered");
        assert_eq!(before.title.as_ref(), "About");

        assert!(handle_locale_cycle_command(
            &mut app,
            &CommandId::new(APP_LOCALE_SWITCH_NEXT)
        ));

        let locales = app
            .global::<fret_runtime::fret_i18n::I18nService>()
            .expect("i18n service should exist")
            .preferred_locales()
            .to_vec();
        assert_eq!(
            locales[0],
            fret_runtime::fret_i18n::LocaleId::parse("zh-CN").expect("valid locale")
        );

        let after = app
            .commands()
            .get(CommandId::new(APP_ABOUT))
            .expect("app.about should be registered");
        assert_eq!(after.title.as_ref(), "关于");
    }
}
