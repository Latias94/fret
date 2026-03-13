use super::*;

impl UiGalleryDriver {
    pub(crate) fn build_workspace_menu_commands(
        app: &App,
    ) -> fret_workspace::menu::WorkspaceMenuCommands {
        let resolve_menu_title = |key: &'static str, fallback: &'static str| -> Arc<str> {
            app.global::<fret_runtime::fret_i18n::I18nService>()
                .map(|i18n| {
                    let text = i18n.t(key.to_string());
                    if text == key {
                        Arc::from(fallback)
                    } else {
                        Arc::from(text)
                    }
                })
                .unwrap_or_else(|| Arc::from(fallback))
        };

        let platform = Platform::current();
        let mut cmds = fret_workspace::menu::WorkspaceMenuCommands {
            open: Some(CommandId::new(CMD_APP_OPEN)),
            save: Some(CommandId::new(CMD_APP_SAVE)),
            undo: Some(CommandId::new(fret_app::core_commands::EDIT_UNDO)),
            redo: Some(CommandId::new(fret_app::core_commands::EDIT_REDO)),
            cut: Some(CommandId::new(fret_app::core_commands::EDIT_CUT)),
            copy: Some(CommandId::new(fret_app::core_commands::EDIT_COPY)),
            paste: Some(CommandId::new(fret_app::core_commands::EDIT_PASTE)),
            select_all: Some(CommandId::new(fret_app::core_commands::EDIT_SELECT_ALL)),
            command_palette: Some(CommandId::new(fret_app::core_commands::COMMAND_PALETTE)),
            switch_locale: Some(CommandId::new(
                fret_app::core_commands::APP_LOCALE_SWITCH_NEXT,
            )),
            file_menu_title: Some(resolve_menu_title("workspace-menu-file", "File")),
            edit_menu_title: Some(resolve_menu_title("workspace-menu-edit", "Edit")),
            view_menu_title: Some(resolve_menu_title("workspace-menu-view", "View")),
            window_menu_title: Some(resolve_menu_title("workspace-menu-window", "Window")),
            ..Default::default()
        };

        if platform == Platform::Macos {
            cmds.app_menu_title = Some(Arc::from("Fret"));
            cmds.include_services_menu = true;
            cmds.about = Some(CommandId::new(fret_app::core_commands::APP_ABOUT));
            cmds.preferences = Some(CommandId::new(fret_app::core_commands::APP_PREFERENCES));
            cmds.hide = Some(CommandId::new(fret_app::core_commands::APP_HIDE));
            cmds.hide_others = Some(CommandId::new(fret_app::core_commands::APP_HIDE_OTHERS));
            cmds.show_all = Some(CommandId::new(fret_app::core_commands::APP_SHOW_ALL));
            cmds.quit_app = Some(CommandId::new(fret_app::core_commands::APP_QUIT));
        }

        cmds
    }

    pub(crate) fn recent_menu_items(app: &App) -> Vec<Arc<str>> {
        app.global::<UiGalleryRecentItemsService>()
            .map(|svc| svc.items.clone())
            .unwrap_or_default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn window_menu_items(app: &App) -> Vec<AppWindowId> {
        let Some(store) = app.global::<UiGalleryHarnessDiagnosticsStore>() else {
            return Vec::new();
        };
        let mut windows: Vec<AppWindowId> = store.per_window.keys().copied().collect();
        windows.sort_by_key(|window| format!("{window:?}"));
        windows
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn focused_window_menu_item(app: &App) -> Option<AppWindowId> {
        let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
        let focused = store.focused_window?;
        store.per_window.contains_key(&focused).then_some(focused)
    }

    pub(crate) fn recent_open_command(index: usize) -> CommandId {
        CommandId::new(format!("{CMD_GALLERY_RECENT_OPEN_PREFIX}{}", index + 1))
    }

    pub(crate) fn window_activate_command(index: usize) -> CommandId {
        CommandId::new(format!("{CMD_GALLERY_WINDOW_ACTIVATE_PREFIX}{}", index + 1))
    }

    pub(crate) fn parse_dynamic_command_index(command: &CommandId, prefix: &str) -> Option<usize> {
        command
            .as_str()
            .strip_prefix(prefix)?
            .parse::<usize>()
            .ok()?
            .checked_sub(1)
    }

    pub(crate) fn sync_dynamic_menu_command_metadata(app: &mut App) {
        for (index, title) in Self::recent_menu_items(app)
            .into_iter()
            .take(10)
            .enumerate()
        {
            app.commands_mut().register(
                Self::recent_open_command(index),
                CommandMeta::new(title)
                    .with_category("Menu")
                    .with_keywords(["menu", "recent", "open"])
                    .hidden(),
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        for (index, _window) in Self::window_menu_items(app).into_iter().enumerate() {
            app.commands_mut().register(
                Self::window_activate_command(index),
                CommandMeta::new(format!("Window {}", index + 1))
                    .with_category("Window")
                    .with_keywords(["menu", "window", "activate"])
                    .hidden(),
            );
        }
    }

    pub(crate) fn build_menu_bar(app: &App) -> MenuBar {
        let settings = app.global::<SettingsFileV1>().cloned().unwrap_or_default();
        let os = settings.menu_bar.os;
        let in_window = settings.menu_bar.in_window;

        let mut menu_bar = fret_workspace::menu::workspace_default_menu_bar(
            Self::build_workspace_menu_commands(app),
        );

        let recent_items = Self::recent_menu_items(app);

        if let Some(menu) = menu_bar
            .menus
            .iter_mut()
            .find(|m| m.role == Some(MenuRole::File) || m.title.as_ref() == "File")
            && let Some(MenuItem::Submenu {
                title: _, items, ..
            }) = menu.items.iter_mut().find(
                |i| matches!(i, MenuItem::Submenu { title, .. } if title.as_ref() == "Recent"),
            )
        {
            *items = if recent_items.is_empty() {
                vec![MenuItem::Label {
                    title: Arc::from("No recent items"),
                }]
            } else {
                recent_items
                    .iter()
                    .take(10)
                    .enumerate()
                    .map(|(index, _title)| MenuItem::Command {
                        command: Self::recent_open_command(index),
                        when: None,
                        toggle: None,
                    })
                    .collect()
            };
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let windows = Self::window_menu_items(app);
            let focused_window = Self::focused_window_menu_item(app);
            if !windows.is_empty()
                && let Some(menu) = menu_bar
                    .menus
                    .iter_mut()
                    .find(|m| m.role == Some(MenuRole::Window) || m.title.as_ref() == "Window")
                && let Some(MenuItem::Submenu {
                    title: _, items, ..
                }) = menu.items.iter_mut().find(
                    |i| matches!(i, MenuItem::Submenu { title, .. } if title.as_ref() == "Windows"),
                )
            {
                *items = windows
                    .into_iter()
                    .enumerate()
                    .map(|(index, window_id)| MenuItem::Command {
                        command: Self::window_activate_command(index),
                        when: None,
                        toggle: Some(MenuItemToggle {
                            kind: MenuItemToggleKind::Radio,
                            checked: Some(window_id) == focused_window,
                        }),
                    })
                    .collect();
            }
        }

        let radio = |checked: bool| {
            Some(MenuItemToggle {
                kind: MenuItemToggleKind::Radio,
                checked,
            })
        };

        menu_bar.menus.push(Menu {
            title: Arc::from("Gallery"),
            role: None,
            mnemonic: Some('g'),
            items: vec![
                MenuItem::Command {
                    command: CommandId::new(CMD_APP_SETTINGS),
                    when: None,
                    toggle: None,
                },
                MenuItem::Command {
                    command: CommandId::new(CMD_APP_TOGGLE_PREFERENCES_ENABLED),
                    when: None,
                    toggle: None,
                },
                MenuItem::Separator,
                MenuItem::Submenu {
                    title: Arc::from("Menu Bar"),
                    when: None,
                    items: vec![
                        MenuItem::Submenu {
                            title: Arc::from("OS"),
                            when: None,
                            items: vec![
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_OS_AUTO),
                                    when: None,
                                    toggle: radio(os == MenuBarIntegrationModeV1::Auto),
                                },
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_OS_ON),
                                    when: None,
                                    toggle: radio(os == MenuBarIntegrationModeV1::On),
                                },
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_OS_OFF),
                                    when: None,
                                    toggle: radio(os == MenuBarIntegrationModeV1::Off),
                                },
                            ],
                        },
                        MenuItem::Submenu {
                            title: Arc::from("In-window"),
                            when: None,
                            items: vec![
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_IN_WINDOW_AUTO),
                                    when: None,
                                    toggle: radio(in_window == MenuBarIntegrationModeV1::Auto),
                                },
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_IN_WINDOW_ON),
                                    when: None,
                                    toggle: radio(in_window == MenuBarIntegrationModeV1::On),
                                },
                                MenuItem::Command {
                                    command: CommandId::new(CMD_MENU_BAR_IN_WINDOW_OFF),
                                    when: None,
                                    toggle: radio(in_window == MenuBarIntegrationModeV1::Off),
                                },
                            ],
                        },
                    ],
                },
                MenuItem::Separator,
                MenuItem::Command {
                    command: CommandId::new(CMD_GALLERY_PAGE_BACK),
                    when: None,
                    toggle: None,
                },
                MenuItem::Command {
                    command: CommandId::new(CMD_GALLERY_PAGE_FORWARD),
                    when: None,
                    toggle: None,
                },
                MenuItem::Separator,
                MenuItem::Submenu {
                    title: Arc::from("Debug"),
                    when: None,
                    items: vec![
                        MenuItem::Command {
                            command: CommandId::new(CMD_GALLERY_DEBUG_RECENT_ADD),
                            when: None,
                            toggle: None,
                        },
                        MenuItem::Command {
                            command: CommandId::new(CMD_GALLERY_DEBUG_RECENT_CLEAR),
                            when: None,
                            toggle: None,
                        },
                        MenuItem::Command {
                            command: CommandId::new(CMD_GALLERY_DEBUG_WINDOW_OPEN),
                            when: None,
                            toggle: None,
                        },
                    ],
                },
            ],
        });

        menu_bar
    }

    pub(crate) fn menu_bar_mode_key(mode: MenuBarIntegrationModeV1) -> Arc<str> {
        match mode {
            MenuBarIntegrationModeV1::Auto => Arc::from("auto"),
            MenuBarIntegrationModeV1::On => Arc::from("on"),
            MenuBarIntegrationModeV1::Off => Arc::from("off"),
        }
    }

    pub(crate) fn menu_bar_mode_from_key(key: Option<&str>) -> MenuBarIntegrationModeV1 {
        match key.unwrap_or("auto") {
            "on" => MenuBarIntegrationModeV1::On,
            "off" => MenuBarIntegrationModeV1::Off,
            _ => MenuBarIntegrationModeV1::Auto,
        }
    }

    pub(crate) fn apply_menu_bar_settings(
        app: &mut App,
        os: MenuBarIntegrationModeV1,
        in_window: MenuBarIntegrationModeV1,
    ) {
        app.with_global_mut(SettingsFileV1::default, |settings, _app| {
            settings.menu_bar.os = os;
            settings.menu_bar.in_window = in_window;
        });
    }

    pub(crate) fn text_common_fallback_injection_key(
        injection: fret_core::TextCommonFallbackInjection,
    ) -> Arc<str> {
        match injection {
            fret_core::TextCommonFallbackInjection::PlatformDefault => {
                Arc::from("platform-default")
            }
            fret_core::TextCommonFallbackInjection::None => Arc::from("none"),
            fret_core::TextCommonFallbackInjection::CommonFallback => {
                Arc::from("common-fallback")
            }
        }
    }

    pub(crate) fn text_common_fallback_injection_from_key(
        key: Option<&str>,
    ) -> fret_core::TextCommonFallbackInjection {
        match key.unwrap_or("platform-default") {
            "none" => fret_core::TextCommonFallbackInjection::None,
            "common-fallback" => fret_core::TextCommonFallbackInjection::CommonFallback,
            _ => fret_core::TextCommonFallbackInjection::PlatformDefault,
        }
    }

    pub(crate) fn current_text_font_family_config(app: &App) -> fret_core::TextFontFamilyConfig {
        app.global::<fret_core::TextFontFamilyConfig>()
            .cloned()
            .or_else(|| app.global::<SettingsFileV1>().map(|settings| settings.fonts.clone()))
            .unwrap_or_default()
    }

    pub(crate) fn apply_settings_sheet_values(
        app: &mut App,
        os: MenuBarIntegrationModeV1,
        in_window: MenuBarIntegrationModeV1,
        common_fallback_injection: fret_core::TextCommonFallbackInjection,
    ) {
        let mut settings = app.global::<SettingsFileV1>().cloned().unwrap_or_default();
        settings.menu_bar.os = os;
        settings.menu_bar.in_window = in_window;
        settings.fonts.common_fallback_injection = common_fallback_injection;
        fret_app::settings::apply_settings_globals(app, &settings);

        let mut fonts = app
            .global::<fret_core::TextFontFamilyConfig>()
            .cloned()
            .unwrap_or_else(|| settings.fonts.clone());
        fonts.common_fallback_injection = common_fallback_injection;
        app.set_global::<fret_core::TextFontFamilyConfig>(fonts);
    }

    pub(crate) fn sync_menu_bar_after_state_change(app: &mut App, window: AppWindowId) {
        Self::sync_dynamic_menu_command_metadata(app);
        let menu_bar = Self::build_menu_bar(app);
        fret_app::set_menu_bar_baseline(app, menu_bar);
        fret_app::sync_os_menu_bar(app);
        app.request_redraw(window);
    }

    pub(crate) fn bump_menu_bar_seq(app: &mut App, seq: &Model<u64>) {
        let _ = app.models_mut().update(seq, |value| {
            *value = value.saturating_add(1);
        });
    }

    pub(crate) fn handle_menu_bar_mode_command(
        app: &mut App,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        command: &str,
    ) -> bool {
        let settings = app.global::<SettingsFileV1>().cloned().unwrap_or_default();
        let mut os = settings.menu_bar.os;
        let mut in_window = settings.menu_bar.in_window;

        let last_action: &'static str = match command {
            CMD_MENU_BAR_OS_AUTO => {
                os = MenuBarIntegrationModeV1::Auto;
                "settings.menu_bar.os.auto"
            }
            CMD_MENU_BAR_OS_ON => {
                os = MenuBarIntegrationModeV1::On;
                "settings.menu_bar.os.on"
            }
            CMD_MENU_BAR_OS_OFF => {
                os = MenuBarIntegrationModeV1::Off;
                "settings.menu_bar.os.off"
            }
            CMD_MENU_BAR_IN_WINDOW_AUTO => {
                in_window = MenuBarIntegrationModeV1::Auto;
                "settings.menu_bar.in_window.auto"
            }
            CMD_MENU_BAR_IN_WINDOW_ON => {
                in_window = MenuBarIntegrationModeV1::On;
                "settings.menu_bar.in_window.on"
            }
            CMD_MENU_BAR_IN_WINDOW_OFF => {
                in_window = MenuBarIntegrationModeV1::Off;
                "settings.menu_bar.in_window.off"
            }
            _ => return false,
        };

        Self::apply_menu_bar_settings(app, os, in_window);
        Self::sync_menu_bar_after_state_change(app, window);
        Self::bump_menu_bar_seq(app, &state.menu_bar_seq);

        let _ = app
            .models_mut()
            .update(&state.settings_menu_bar_os, |value| {
                *value = Some(Self::menu_bar_mode_key(os));
            });
        let _ = app
            .models_mut()
            .update(&state.settings_menu_bar_in_window, |value| {
                *value = Some(Self::menu_bar_mode_key(in_window));
            });

        let _ = app.models_mut().update(&state.last_action, |value| {
            *value = Arc::<str>::from(last_action);
        });

        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn write_project_settings_snapshot(
        os: MenuBarIntegrationModeV1,
        in_window: MenuBarIntegrationModeV1,
        common_fallback_injection: fret_core::TextCommonFallbackInjection,
    ) -> Result<(), std::io::Error> {
        let project_dir = std::path::Path::new(fret_app::PROJECT_CONFIG_DIR);
        std::fs::create_dir_all(project_dir)?;
        let path = project_dir.join(fret_app::SETTINGS_JSON);

        let mut settings = SettingsFileV1::load_json_if_exists(&path)
            .map_err(std::io::Error::other)?
            .unwrap_or_default();
        settings.menu_bar.os = os;
        settings.menu_bar.in_window = in_window;
        settings.fonts.common_fallback_injection = common_fallback_injection;

        let json = serde_json::to_string_pretty(&settings)
            .unwrap_or_else(|_| "{\"settings_version\":1}".to_string());
        std::fs::write(path, format!("{json}\n"))?;
        Ok(())
    }
}
