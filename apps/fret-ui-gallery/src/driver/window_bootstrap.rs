use super::*;
use fret_ui::PaintCachePolicy;

fn parse_iso_date_ymd(raw: &str) -> Option<Date> {
    let raw = raw.trim();
    let (year, rest) = raw.split_once('-')?;
    let (month, day) = rest.split_once('-')?;

    let year: i32 = year.parse().ok()?;
    let month: u8 = month.parse().ok()?;
    let day: u8 = day.parse().ok()?;

    let month = time::Month::try_from(month).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

fn env_bool(name: &str, default: bool) -> bool {
    let Some(value) = std::env::var_os(name).filter(|value| !value.is_empty()) else {
        return default;
    };

    let value = value.to_string_lossy().trim().to_ascii_lowercase();
    !(value == "0" || value == "false" || value == "no" || value == "off")
}

fn config_bool(env_name: &str, query_name: &str, default: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(value) = bool_from_window_query(query_name) {
            return value;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = query_name;

    env_bool(env_name, default)
}

fn config_paint_cache_policy(env_name: &str, query_name: &str) -> PaintCachePolicy {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(value) = bool_from_window_query(query_name) {
            return if value {
                PaintCachePolicy::Enabled
            } else {
                PaintCachePolicy::Disabled
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = query_name;

    let Some(value) = std::env::var_os(env_name).filter(|value| !value.is_empty()) else {
        return PaintCachePolicy::Auto;
    };

    match value.to_string_lossy().trim().to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "off" | "disabled" => PaintCachePolicy::Disabled,
        "1" | "true" | "yes" | "on" | "enabled" => PaintCachePolicy::Enabled,
        "auto" => PaintCachePolicy::Auto,
        _ => PaintCachePolicy::Auto,
    }
}

fn register_harness_model_ids(app: &mut App, window: AppWindowId, state: &UiGalleryWindowState) {
    app.with_global_mut(UiGalleryHarnessDiagnosticsStore::default, |store, _app| {
        store.per_window.insert(
            window,
            UiGalleryHarnessModelIds {
                selected_page: state.selected_page.clone(),
                workspace_tabs: state.workspace_tabs.clone(),
                workspace_dirty_tabs: state.workspace_dirty_tabs.clone(),
                nav_query: state.nav_query.clone(),
                settings_menu_bar_os: state.settings_menu_bar_os.clone(),
                settings_menu_bar_in_window: state.settings_menu_bar_in_window.clone(),
                chrome_show_workspace_tab_strip: state.chrome_show_workspace_tab_strip.clone(),
                cmdk_query: state.cmdk_query.clone(),
                last_action: state.last_action.clone(),
                input_file_value: state.input_file_value.clone(),
                #[cfg(feature = "gallery-dev")]
                code_editor_syntax_rust: state.code_editor_syntax_rust.clone(),
                #[cfg(feature = "gallery-dev")]
                code_editor_boundary_identifier: state.code_editor_boundary_identifier.clone(),
                #[cfg(feature = "gallery-dev")]
                code_editor_soft_wrap: state.code_editor_soft_wrap.clone(),
                #[cfg(feature = "gallery-dev")]
                code_editor_folds: state.code_editor_folds.clone(),
                #[cfg(feature = "gallery-dev")]
                code_editor_inlays: state.code_editor_inlays.clone(),
                text_input: state.text_input.clone(),
                text_area: state.text_area.clone(),
            },
        );
        if store.focused_window.is_none() {
            store.focused_window = Some(window);
        }
    });
}

impl UiGalleryDriver {
    pub(crate) fn build_ui(app: &mut App, window: AppWindowId) -> UiGalleryWindowState {
        let page_router = build_ui_gallery_page_router();
        let diag_profile = ui_gallery_diag_profile();
        let workspace_shell_diag_profile =
            diag_profile.as_deref() == Some(UI_GALLERY_DIAG_PROFILE_WORKSPACE_SHELL);

        let start_page = ui_gallery_start_page().unwrap_or_else(|| {
            if workspace_shell_diag_profile
                || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
            {
                #[cfg(feature = "gallery-dev")]
                {
                    Arc::<str>::from(PAGE_OVERLAY)
                }

                #[cfg(not(feature = "gallery-dev"))]
                {
                    Arc::<str>::from(PAGE_DIALOG)
                }
            } else {
                Arc::<str>::from(PAGE_INTRO)
            }
        });

        #[cfg(target_arch = "wasm32")]
        let start_page = page_from_gallery_location(&page_router.state().location)
            .unwrap_or_else(|| start_page.clone());

        let selected_page = app.models_mut().insert(start_page.clone());

        let mut workspace_tabs_init = vec![
            Arc::<str>::from(PAGE_INTRO),
            Arc::<str>::from(PAGE_LAYOUT),
            Arc::<str>::from(PAGE_VIEW_CACHE),
            Arc::<str>::from(PAGE_BUTTON),
            Arc::<str>::from(PAGE_CARD),
            Arc::<str>::from(PAGE_BADGE),
            Arc::<str>::from(PAGE_AVATAR),
            Arc::<str>::from(PAGE_FIELD),
            Arc::<str>::from(PAGE_COMMAND),
        ];
        #[cfg(feature = "gallery-dev")]
        {
            workspace_tabs_init.push(Arc::<str>::from(PAGE_ICONS));
            workspace_tabs_init.push(Arc::<str>::from(PAGE_OVERLAY));
        }
        if !workspace_tabs_init
            .iter()
            .any(|page| page.as_ref() == start_page.as_ref())
        {
            workspace_tabs_init.push(start_page.clone());
        }

        let workspace_tab_close_by_command: HashMap<Arc<str>, Arc<str>> = workspace_tabs_init
            .iter()
            .cloned()
            .map(|tab_id| (Self::workspace_tab_close_command(tab_id.as_ref()), tab_id))
            .collect();
        let workspace_tabs_init_for_layout = workspace_tabs_init.clone();
        #[cfg(feature = "gallery-dev")]
        let workspace_dirty_tabs_init = vec![Arc::<str>::from(PAGE_OVERLAY)];
        #[cfg(not(feature = "gallery-dev"))]
        let workspace_dirty_tabs_init = vec![Arc::<str>::from(PAGE_COMMAND)];
        let workspace_window_layout_init = Self::build_workspace_window_layout(
            start_page.clone(),
            &workspace_tabs_init_for_layout,
            &workspace_dirty_tabs_init,
        );
        let workspace_tabs = app.models_mut().insert(workspace_tabs_init);
        let workspace_dirty_tabs = app.models_mut().insert(workspace_dirty_tabs_init);
        let workspace_window_layout = app.models_mut().insert(workspace_window_layout_init);
        let nav_query_default = std::env::var_os(ENV_UI_GALLERY_NAV_QUERY)
            .and_then(|value| (!value.is_empty()).then_some(value))
            .map(|value| value.to_string_lossy().trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_default();
        let nav_query = app.models_mut().insert(nav_query_default);
        let initial_theme_preset = if workspace_shell_diag_profile {
            Arc::<str>::from("zinc/dark")
        } else {
            Arc::<str>::from("zinc/light")
        };
        let theme_preset = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(initial_theme_preset));
        let theme_preset_open = app.models_mut().insert(false);
        let motion_preset = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("theme")));
        let motion_preset_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let dialog_glass_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let alert_dialog_open = app.models_mut().insert(false);
        #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
        let sheet_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let portal_geometry_popover_open = app.models_mut().insert(false);

        let mut settings = app.global::<SettingsFileV1>().cloned().unwrap_or_default();
        let is_diag = std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty());
        if is_diag {
            settings.menu_bar.os = MenuBarIntegrationModeV1::Off;
            settings.menu_bar.in_window = MenuBarIntegrationModeV1::On;
            Self::apply_menu_bar_settings(app, settings.menu_bar.os, settings.menu_bar.in_window);
        }
        let settings_open = app.models_mut().insert(false);
        let settings_menu_bar_os = app
            .models_mut()
            .insert(Some(Self::menu_bar_mode_key(settings.menu_bar.os)));
        let settings_menu_bar_os_open = app.models_mut().insert(false);
        let settings_menu_bar_in_window = app
            .models_mut()
            .insert(Some(Self::menu_bar_mode_key(settings.menu_bar.in_window)));
        let settings_menu_bar_in_window_open = app.models_mut().insert(false);
        let settings_edit_can_undo = app.models_mut().insert(true);
        let settings_edit_can_redo = app.models_mut().insert(true);
        let chrome_show_workspace_tab_strip = app.models_mut().insert(workspace_shell_diag_profile);
        let undo_doc: DocumentId = "ui_gallery.window".into();
        let combobox_value = app.models_mut().insert(None::<Arc<str>>);
        let combobox_open = app.models_mut().insert(false);
        let combobox_query = app.models_mut().insert(String::new());

        let date_picker_open = app.models_mut().insert(false);
        let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
            .ok()
            .and_then(|raw| parse_iso_date_ymd(&raw))
            .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());
        let date_picker_month = app
            .models_mut()
            .insert(fret_ui_headless::calendar::CalendarMonth::from_date(today));
        let date_picker_selected = app.models_mut().insert(None::<Date>);

        #[cfg(feature = "gallery-dev")]
        let data_grid_selected_row = app.models_mut().insert(None::<u64>);
        let tabs_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("overview")));
        let accordion_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("item-1")));

        let avatar_demo_image = app.models_mut().insert(None::<ImageId>);
        let avatar_demo_image_token = app.next_image_upload_token();
        Self::enqueue_avatar_demo_image_register(app, window, avatar_demo_image_token);

        let image_fit_demo_wide_image = app.models_mut().insert(None::<ImageId>);
        let image_fit_demo_wide_token = app.next_image_upload_token();
        Self::enqueue_image_fit_demo_image_register(
            app,
            window,
            image_fit_demo_wide_token,
            Self::IMAGE_FIT_DEMO_WIDE_SIZE,
            (120, 190, 255),
        );

        let image_fit_demo_tall_image = app.models_mut().insert(None::<ImageId>);
        let image_fit_demo_tall_token = app.next_image_upload_token();
        Self::enqueue_image_fit_demo_image_register(
            app,
            window,
            image_fit_demo_tall_token,
            Self::IMAGE_FIT_DEMO_TALL_SIZE,
            (255, 160, 120),
        );

        let progress = app.models_mut().insert(35.0f32);
        #[cfg(feature = "gallery-dev")]
        let checkbox = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let switch = app.models_mut().insert(true);
        #[cfg(feature = "gallery-dev")]
        let code_editor_syntax_rust = app.models_mut().insert(true);
        #[cfg(feature = "gallery-dev")]
        let code_editor_boundary_identifier = app.models_mut().insert(true);
        #[cfg(feature = "gallery-dev")]
        let code_editor_soft_wrap = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let code_editor_folds = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let code_editor_inlays = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let markdown_link_gate_last_activation = app.models_mut().insert(None::<Arc<str>>);
        #[cfg(feature = "gallery-material3")]
        let material3_expressive = app.models_mut().insert(false);
        let text_input = app.models_mut().insert(String::new());
        let text_area = app.models_mut().insert(String::new());
        let input_file_value = app.models_mut().insert(String::new());
        #[cfg(feature = "gallery-dev")]
        let dropdown_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let context_menu_open = app.models_mut().insert(false);
        #[cfg(feature = "gallery-dev")]
        let context_menu_edge_open = app.models_mut().insert(false);
        let cmdk_open = app.models_mut().insert(false);
        let cmdk_query = app.models_mut().insert(String::new());
        let last_action = app.models_mut().insert(Arc::<str>::from("ready"));
        let sonner_position = app.models_mut().insert(shadcn::ToastPosition::BottomRight);
        let menu_bar_seq = app.models_mut().insert(0u64);
        #[cfg(feature = "gallery-dev")]
        let virtual_list_torture_jump = app.models_mut().insert(String::new());
        #[cfg(feature = "gallery-dev")]
        let virtual_list_torture_edit_row = app.models_mut().insert(None::<u64>);
        #[cfg(feature = "gallery-dev")]
        let virtual_list_torture_edit_text = app.models_mut().insert(String::new());
        #[cfg(feature = "gallery-dev")]
        let virtual_list_torture_scroll = VirtualListScrollHandle::new();

        let view_cache_enabled = app.models_mut().insert(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE_ENABLE_INNER_CONTROL",
            "fret_ui_gallery_view_cache_enable_inner_control",
            false,
        ));
        let view_cache_shell_default = if workspace_shell_diag_profile {
            true
        } else {
            false
        };
        let view_cache_cache_shell = app.models_mut().insert(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL",
            "fret_ui_gallery_view_cache_shell",
            view_cache_shell_default,
        ));
        let view_cache_cache_content = app.models_mut().insert(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE_CONTENT",
            "fret_ui_gallery_view_cache_content",
            true,
        ));
        let view_cache_inner_enabled = app.models_mut().insert(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE_INNER",
            "fret_ui_gallery_view_cache_inner",
            true,
        ));
        let view_cache_popover_open = app.models_mut().insert(false);
        let view_cache_continuous = app.models_mut().insert(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE_CONTINUOUS",
            "fret_ui_gallery_view_cache_continuous",
            false,
        ));
        let view_cache_counter = app.models_mut().insert(0u64);

        let perf_mode = std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|v| !v.is_empty());
        let inspector_enabled = app.models_mut().insert(
            std::env::var_os("FRET_UI_GALLERY_INSPECTOR").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || (!perf_mode && std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())),
        );
        let inspector_last_pointer = app.models_mut().insert(None::<fret_core::Point>);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(config_bool(
            "FRET_UI_GALLERY_VIEW_CACHE",
            "fret_ui_gallery_view_cache",
            false,
        ));
        ui.set_paint_cache_policy(config_paint_cache_policy(
            "FRET_UI_GALLERY_PAINT_CACHE",
            "fret_ui_gallery_paint_cache",
        ));
        ui.set_debug_enabled(
            std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || (!perf_mode && std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())),
        );

        Self::sync_undo_availability(app, window, &undo_doc);

        let mut state = UiGalleryWindowState {
            ui,
            root: None,
            debug_hud: DebugHudState::default(),
            pending_taffy_dump: None,
            pending_file_dialog: None,
            page_router,
            selected_page,
            workspace_tabs,
            workspace_dirty_tabs,
            workspace_window_layout,
            workspace_tab_close_by_command,
            nav_query,
            theme_preset,
            theme_preset_open,
            applied_theme_preset: None,
            motion_preset,
            motion_preset_open,
            applied_motion_preset: None,
            applied_motion_preset_theme_preset: None,
            view_cache_enabled,
            view_cache_cache_shell,
            view_cache_cache_content,
            view_cache_inner_enabled,
            view_cache_popover_open,
            view_cache_continuous,
            view_cache_counter,
            inspector_enabled,
            inspector_last_pointer,
            #[cfg(feature = "gallery-dev")]
            popover_open,
            dialog_open,
            #[cfg(feature = "gallery-dev")]
            dialog_glass_open,
            #[cfg(feature = "gallery-dev")]
            alert_dialog_open,
            #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
            sheet_open,
            #[cfg(feature = "gallery-dev")]
            portal_geometry_popover_open,
            settings_open,
            settings_menu_bar_os,
            settings_menu_bar_os_open,
            settings_menu_bar_in_window,
            settings_menu_bar_in_window_open,
            settings_edit_can_undo,
            settings_edit_can_redo,
            chrome_show_workspace_tab_strip,
            undo_doc,
            combobox_value,
            combobox_open,
            combobox_query,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
            #[cfg(feature = "gallery-dev")]
            data_grid_selected_row,
            tabs_value,
            accordion_value,
            avatar_demo_image,
            avatar_demo_image_token: Some(avatar_demo_image_token),
            avatar_demo_image_retry_count: 0,
            image_fit_demo_wide_image,
            image_fit_demo_wide_token: Some(image_fit_demo_wide_token),
            image_fit_demo_tall_image,
            image_fit_demo_tall_token: Some(image_fit_demo_tall_token),
            progress,
            #[cfg(feature = "gallery-dev")]
            checkbox,
            #[cfg(feature = "gallery-dev")]
            switch,
            #[cfg(feature = "gallery-dev")]
            code_editor_syntax_rust,
            #[cfg(feature = "gallery-dev")]
            code_editor_boundary_identifier,
            #[cfg(feature = "gallery-dev")]
            code_editor_soft_wrap,
            #[cfg(feature = "gallery-dev")]
            code_editor_folds,
            #[cfg(feature = "gallery-dev")]
            code_editor_inlays,
            #[cfg(feature = "gallery-dev")]
            markdown_link_gate_last_activation,
            #[cfg(feature = "gallery-material3")]
            material3_expressive,
            text_input,
            text_area,
            input_file_value,
            #[cfg(feature = "gallery-dev")]
            dropdown_open,
            #[cfg(feature = "gallery-dev")]
            context_menu_open,
            #[cfg(feature = "gallery-dev")]
            context_menu_edge_open,
            cmdk_open,
            cmdk_query,
            last_action,
            sonner_position,
            menu_bar_seq,
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_jump,
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_edit_row,
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_edit_text,
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_scroll,
            last_config_files_status_seq: 0,
        };

        if let Some(selected) = app.models().get_cloned(&state.selected_page) {
            #[cfg(target_arch = "wasm32")]
            {
                let current_page = page_from_gallery_location(&state.page_router.state().location);
                if current_page.is_some_and(|page| page.as_ref() == selected.as_ref()) {
                    let update = state.page_router.init_with_prefetch_intents();
                    apply_page_router_update_side_effects(
                        app,
                        window,
                        selected,
                        &mut state.page_router,
                        update,
                    );
                } else {
                    apply_page_route_side_effects_via_router(
                        app,
                        window,
                        NavigationAction::Replace,
                        selected,
                        &mut state.page_router,
                    );
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            apply_page_route_side_effects_via_router(
                app,
                window,
                NavigationAction::Replace,
                selected,
                &mut state.page_router,
            );
        }

        register_harness_model_ids(app, window, &state);

        Self::sync_menu_bar_after_state_change(app, window);
        Self::bump_menu_bar_seq(app, &state.menu_bar_seq);

        state
    }
}
