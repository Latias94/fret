// Included from `apps/fret-ui-gallery/src/driver.rs` to keep the module entrypoint small.

use fret_app::{
    ActivationPolicy, App, CommandId, CommandMeta, CreateWindowKind, CreateWindowRequest, Effect,
    LayeredConfigPaths, Menu, MenuBar, MenuBarIntegrationModeV1, MenuItem, MenuRole, Model,
    Platform, SettingsFileV1, WindowRequest, WindowRole, WindowStyleRequest, load_layered_settings,
};
use fret_core::{
    AlphaMode, AppWindowId, Event, ExternalDropReadLimits, FileDialogFilter, FileDialogOptions,
    ImageColorInfo, ImageId, ImageUploadToken, RectPx, TimerToken, UiServices,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_router::{NavigationAction, Router};
use fret_runtime::{
    ImageUpdateToken, MenuItemToggle, MenuItemToggleKind, PlatformCapabilities,
    WindowCommandAvailabilityService, WindowCommandEnabledService,
};
use fret_ui::UiTree;
use fret_ui::action::{UiActionHost, UiActionHostAdapter};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_shadcn as shadcn;
use fret_undo::{CoalesceKey, DocumentId, UndoRecord, UndoService, ValueTx};
use fret_workspace::commands::{
    CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::Date;

use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;

#[cfg(not(target_arch = "wasm32"))]
use crate::harness::UiGalleryCodeEditorHandlesStore;
use crate::spec::*;
use crate::ui;

mod app_bootstrap;
mod chrome;
mod debug_hud;
mod debug_stats;
mod diag_snapshot;
mod inspector;
mod menubar;
mod render_flow;
mod router;
mod settings_sheet;
mod shell;
mod status_bar;
mod toaster;
use router::{
    UiGalleryHistory, UiGalleryRouteId, apply_page_route_side_effects_via_router,
    apply_page_router_update_side_effects, build_ui_gallery_page_router,
    page_from_gallery_location,
};

#[derive(Default)]
struct DebugHudState {
    last_tick: Option<fret_core::time::Instant>,
    ema_frame_time_us: Option<f64>,
}

#[derive(Clone, Debug)]
struct PendingTaffyDumpRequest {
    root_label_filter: Option<Arc<str>>,
    filename_tag: Arc<str>,
}

#[derive(Default)]
struct UiGalleryHarnessDiagnosticsStore {
    per_window: HashMap<AppWindowId, UiGalleryHarnessModelIds>,
    focused_window: Option<AppWindowId>,
}

#[derive(Default)]
struct UiGalleryRecentItemsService {
    next_id: u64,
    items: Vec<Arc<str>>,
}

#[derive(Default)]
struct UiGalleryDebugWindowService {
    next_logical_window_id: u64,
    script_keepalive_window: Option<AppWindowId>,
    script_keepalive_frames: u32,
}

const DEBUG_WINDOW_OPEN_KEEPALIVE_TIMER: TimerToken = TimerToken(0x7569_6761_6c6c_6572);

#[derive(Clone)]
struct UiGalleryHarnessModelIds {
    selected_page: Model<Arc<str>>,
    code_editor_syntax_rust: Model<bool>,
    code_editor_boundary_identifier: Model<bool>,
    code_editor_soft_wrap: Model<bool>,
    code_editor_folds: Model<bool>,
    code_editor_inlays: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
}

impl DebugHudState {
    fn tick(&mut self, now: fret_core::time::Instant) -> Option<Duration> {
        let dt = self.last_tick.map(|prev| now.duration_since(prev));
        self.last_tick = Some(now);

        if let Some(dt) = dt {
            let sample = dt.as_micros() as f64;
            let alpha = 0.1;
            self.ema_frame_time_us = Some(match self.ema_frame_time_us {
                Some(prev) => prev * (1.0 - alpha) + sample * alpha,
                None => sample,
            });
        }

        dt
    }

    fn ema_fps(&self) -> Option<f64> {
        let us = self.ema_frame_time_us?;
        if us <= 0.0 {
            return None;
        }
        Some(1_000_000.0 / us)
    }
}

struct UiGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    debug_hud: DebugHudState,
    pending_taffy_dump: Option<PendingTaffyDumpRequest>,
    page_router: Router<UiGalleryRouteId, UiGalleryHistory>,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    workspace_tab_close_by_command: HashMap<Arc<str>, Arc<str>>,
    nav_query: Model<String>,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    applied_theme_preset: Option<Arc<str>>,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    inspector_enabled: Model<bool>,
    inspector_last_pointer: Model<Option<fret_core::Point>>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,

    settings_open: Model<bool>,
    settings_menu_bar_os: Model<Option<Arc<str>>>,
    settings_menu_bar_os_open: Model<bool>,
    settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    settings_menu_bar_in_window_open: Model<bool>,
    settings_edit_can_undo: Model<bool>,
    settings_edit_can_redo: Model<bool>,
    undo_doc: DocumentId,

    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    time_picker_open: Model<bool>,
    time_picker_selected: Model<time::Time>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    avatar_demo_image: Model<Option<ImageId>>,
    avatar_demo_image_token: Option<ImageUploadToken>,
    avatar_demo_image_retry_count: u8,
    image_fit_demo_wide_image: Model<Option<ImageId>>,
    image_fit_demo_wide_token: Option<ImageUploadToken>,
    image_fit_demo_tall_image: Model<Option<ImageId>>,
    image_fit_demo_tall_token: Option<ImageUploadToken>,
    image_fit_demo_streaming_image: Model<Option<ImageId>>,
    image_fit_demo_streaming_token: Option<ImageUploadToken>,
    image_fit_demo_streaming_frame: u64,
    image_fit_demo_streaming_size: (u32, u32),
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    code_editor_syntax_rust: Model<bool>,
    code_editor_boundary_identifier: Model<bool>,
    code_editor_soft_wrap: Model<bool>,
    code_editor_folds: Model<bool>,
    code_editor_inlays: Model<bool>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_expressive: Model<bool>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_navigation_rail_value: Model<Arc<str>>,
    material3_navigation_drawer_value: Model<Arc<str>>,
    material3_modal_navigation_drawer_open: Model<bool>,
    material3_dialog_open: Model<bool>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_autocomplete_value: Model<String>,
    material3_autocomplete_disabled: Model<bool>,
    material3_autocomplete_error: Model<bool>,
    material3_autocomplete_dialog_open: Model<bool>,
    material3_menu_open: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
    menu_bar_seq: Model<u64>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
    last_config_files_status_seq: u64,
}

impl UiGalleryWindowState {
    fn content_models(&self) -> ui::UiGalleryModels {
        ui::UiGalleryModels {
            content_tab: self.content_tab.clone(),
            theme_preset: self.theme_preset.clone(),
            theme_preset_open: self.theme_preset_open.clone(),
            view_cache_enabled: self.view_cache_enabled.clone(),
            view_cache_cache_shell: self.view_cache_cache_shell.clone(),
            view_cache_inner_enabled: self.view_cache_inner_enabled.clone(),
            view_cache_popover_open: self.view_cache_popover_open.clone(),
            view_cache_continuous: self.view_cache_continuous.clone(),
            view_cache_counter: self.view_cache_counter.clone(),
            popover_open: self.popover_open.clone(),
            dialog_open: self.dialog_open.clone(),
            alert_dialog_open: self.alert_dialog_open.clone(),
            sheet_open: self.sheet_open.clone(),
            portal_geometry_popover_open: self.portal_geometry_popover_open.clone(),
            select_value: self.select_value.clone(),
            select_open: self.select_open.clone(),
            combobox_value: self.combobox_value.clone(),
            combobox_open: self.combobox_open.clone(),
            combobox_query: self.combobox_query.clone(),
            date_picker_open: self.date_picker_open.clone(),
            date_picker_month: self.date_picker_month.clone(),
            date_picker_selected: self.date_picker_selected.clone(),
            time_picker_open: self.time_picker_open.clone(),
            time_picker_selected: self.time_picker_selected.clone(),
            resizable_h_fractions: self.resizable_h_fractions.clone(),
            resizable_v_fractions: self.resizable_v_fractions.clone(),
            data_table_state: self.data_table_state.clone(),
            data_grid_selected_row: self.data_grid_selected_row.clone(),
            tabs_value: self.tabs_value.clone(),
            accordion_value: self.accordion_value.clone(),
            avatar_demo_image: self.avatar_demo_image.clone(),
            image_fit_demo_wide_image: self.image_fit_demo_wide_image.clone(),
            image_fit_demo_tall_image: self.image_fit_demo_tall_image.clone(),
            image_fit_demo_streaming_image: self.image_fit_demo_streaming_image.clone(),
            progress: self.progress.clone(),
            checkbox: self.checkbox.clone(),
            switch: self.switch.clone(),
            material3_checkbox: self.material3_checkbox.clone(),
            material3_switch: self.material3_switch.clone(),
            material3_radio_value: self.material3_radio_value.clone(),
            material3_tabs_value: self.material3_tabs_value.clone(),
            material3_list_value: self.material3_list_value.clone(),
            material3_expressive: self.material3_expressive.clone(),
            material3_navigation_bar_value: self.material3_navigation_bar_value.clone(),
            material3_navigation_rail_value: self.material3_navigation_rail_value.clone(),
            material3_navigation_drawer_value: self.material3_navigation_drawer_value.clone(),
            material3_modal_navigation_drawer_open: self
                .material3_modal_navigation_drawer_open
                .clone(),
            material3_dialog_open: self.material3_dialog_open.clone(),
            material3_text_field_value: self.material3_text_field_value.clone(),
            material3_text_field_disabled: self.material3_text_field_disabled.clone(),
            material3_text_field_error: self.material3_text_field_error.clone(),
            material3_autocomplete_value: self.material3_autocomplete_value.clone(),
            material3_autocomplete_disabled: self.material3_autocomplete_disabled.clone(),
            material3_autocomplete_error: self.material3_autocomplete_error.clone(),
            material3_autocomplete_dialog_open: self.material3_autocomplete_dialog_open.clone(),
            material3_menu_open: self.material3_menu_open.clone(),
            text_input: self.text_input.clone(),
            text_area: self.text_area.clone(),
            dropdown_open: self.dropdown_open.clone(),
            context_menu_open: self.context_menu_open.clone(),
            context_menu_edge_open: self.context_menu_edge_open.clone(),
            cmdk_open: self.cmdk_open.clone(),
            cmdk_query: self.cmdk_query.clone(),
            last_action: self.last_action.clone(),
            sonner_position: self.sonner_position.clone(),
            virtual_list_torture_jump: self.virtual_list_torture_jump.clone(),
            virtual_list_torture_edit_row: self.virtual_list_torture_edit_row.clone(),
            virtual_list_torture_edit_text: self.virtual_list_torture_edit_text.clone(),
            virtual_list_torture_scroll: self.virtual_list_torture_scroll.clone(),
            code_editor_syntax_rust: self.code_editor_syntax_rust.clone(),
            code_editor_boundary_identifier: self.code_editor_boundary_identifier.clone(),
            code_editor_soft_wrap: self.code_editor_soft_wrap.clone(),
            code_editor_folds: self.code_editor_folds.clone(),
            code_editor_inlays: self.code_editor_inlays.clone(),
        }
    }
}

#[derive(Default)]
struct UiGalleryDriver;

#[derive(Debug, Clone)]
pub(crate) struct UiGalleryImageSourceDemoAssets {
    pub wide_png: fret_ui_assets::ImageSource,
    pub tall_png: fret_ui_assets::ImageSource,
    pub square_png: fret_ui_assets::ImageSource,
}

impl UiGalleryDriver {
    const AVATAR_DEMO_IMAGE_WIDTH: u32 = 96;
    const AVATAR_DEMO_IMAGE_HEIGHT: u32 = 96;
    const AVATAR_DEMO_IMAGE_RETRY_MAX: u8 = 8;

    const IMAGE_FIT_DEMO_WIDE_SIZE: (u32, u32) = (320, 180);
    const IMAGE_FIT_DEMO_TALL_SIZE: (u32, u32) = (180, 320);
    const IMAGE_FIT_DEMO_STREAMING_SIZE: (u32, u32) = (320, 200);

    fn ensure_image_source_demo_assets_installed(app: &mut App) {
        if app.global::<UiGalleryImageSourceDemoAssets>().is_some() {
            return;
        }

        // Encode a few tiny demo images to PNG and load them through the ecosystem `ImageSource`
        // path. This exercises the decode/load + `ImageAssetCache` integration without requiring
        // external files.
        let wide_rgba = Self::generate_fit_demo_image_rgba8(
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.0,
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.1,
            (120, 190, 255),
        );
        let tall_rgba = Self::generate_fit_demo_image_rgba8(
            Self::IMAGE_FIT_DEMO_TALL_SIZE.0,
            Self::IMAGE_FIT_DEMO_TALL_SIZE.1,
            (255, 160, 120),
        );
        let square_rgba = Self::generate_avatar_demo_image_rgba8(
            Self::AVATAR_DEMO_IMAGE_WIDTH,
            Self::AVATAR_DEMO_IMAGE_HEIGHT,
        );

        let wide_png = Self::encode_rgba8_png_bytes(
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.0,
            Self::IMAGE_FIT_DEMO_WIDE_SIZE.1,
            &wide_rgba,
        );
        let tall_png = Self::encode_rgba8_png_bytes(
            Self::IMAGE_FIT_DEMO_TALL_SIZE.0,
            Self::IMAGE_FIT_DEMO_TALL_SIZE.1,
            &tall_rgba,
        );
        let square_png = Self::encode_rgba8_png_bytes(
            Self::AVATAR_DEMO_IMAGE_WIDTH,
            Self::AVATAR_DEMO_IMAGE_HEIGHT,
            &square_rgba,
        );

        app.set_global(UiGalleryImageSourceDemoAssets {
            wide_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(wide_png)),
            tall_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(tall_png)),
            square_png: fret_ui_assets::ImageSource::from_bytes(Arc::<[u8]>::from(square_png)),
        });
    }

    fn enqueue_avatar_demo_image_register(
        app: &mut App,
        window: AppWindowId,
        token: ImageUploadToken,
    ) {
        app.push_effect(Effect::ImageRegisterRgba8 {
            window,
            token,
            width: Self::AVATAR_DEMO_IMAGE_WIDTH,
            height: Self::AVATAR_DEMO_IMAGE_HEIGHT,
            bytes: Self::generate_avatar_demo_image_rgba8(
                Self::AVATAR_DEMO_IMAGE_WIDTH,
                Self::AVATAR_DEMO_IMAGE_HEIGHT,
            ),
            color_info: ImageColorInfo::srgb_rgba(),
            alpha_mode: AlphaMode::Opaque,
        });
    }

    fn enqueue_image_fit_demo_image_register(
        app: &mut App,
        window: AppWindowId,
        token: ImageUploadToken,
        size: (u32, u32),
        accent: (u8, u8, u8),
    ) {
        app.push_effect(Effect::ImageRegisterRgba8 {
            window,
            token,
            width: size.0,
            height: size.1,
            bytes: Self::generate_fit_demo_image_rgba8(size.0, size.1, accent),
            color_info: ImageColorInfo::srgb_rgba(),
            alpha_mode: AlphaMode::Opaque,
        });
    }

    fn build_workspace_menu_commands(app: &App) -> fret_workspace::menu::WorkspaceMenuCommands {
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

    fn recent_menu_items(app: &App) -> Vec<Arc<str>> {
        app.global::<UiGalleryRecentItemsService>()
            .map(|svc| svc.items.clone())
            .unwrap_or_default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn window_menu_items(app: &App) -> Vec<AppWindowId> {
        let Some(store) = app.global::<UiGalleryHarnessDiagnosticsStore>() else {
            return Vec::new();
        };
        let mut windows: Vec<AppWindowId> = store.per_window.keys().copied().collect();
        windows.sort_by_key(|window| format!("{window:?}"));
        windows
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn focused_window_menu_item(app: &App) -> Option<AppWindowId> {
        let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
        let focused = store.focused_window?;
        store.per_window.contains_key(&focused).then_some(focused)
    }

    fn recent_open_command(index: usize) -> CommandId {
        CommandId::new(format!("{CMD_GALLERY_RECENT_OPEN_PREFIX}{}", index + 1))
    }

    fn window_activate_command(index: usize) -> CommandId {
        CommandId::new(format!("{CMD_GALLERY_WINDOW_ACTIVATE_PREFIX}{}", index + 1))
    }

    fn parse_dynamic_command_index(command: &CommandId, prefix: &str) -> Option<usize> {
        command
            .as_str()
            .strip_prefix(prefix)?
            .parse::<usize>()
            .ok()?
            .checked_sub(1)
    }

    fn sync_dynamic_menu_command_metadata(app: &mut App) {
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

    fn build_menu_bar(app: &App) -> MenuBar {
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
                MenuItem::Separator,
                MenuItem::Command {
                    command: CommandId::new(CMD_CLIPBOARD_COPY_LINK),
                    when: None,
                    toggle: None,
                },
                MenuItem::Command {
                    command: CommandId::new(CMD_CLIPBOARD_COPY_USAGE),
                    when: None,
                    toggle: None,
                },
                MenuItem::Command {
                    command: CommandId::new(CMD_CLIPBOARD_COPY_NOTES),
                    when: None,
                    toggle: None,
                },
            ],
        });

        menu_bar
    }

    fn sync_undo_availability(app: &mut App, window: AppWindowId, doc: &DocumentId) {
        let mut edit_can_undo = false;
        let mut edit_can_redo = false;

        app.with_global_mut(
            || UndoService::<ValueTx<f32>>::with_limit(256),
            |undo_svc, _app| {
                undo_svc.set_active_document(window, doc.clone());
                if let Some(history) = undo_svc.history_mut_active(window) {
                    edit_can_undo = history.can_undo();
                    edit_can_redo = history.can_redo();
                }
            },
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(handle) = app
                .global::<UiGalleryCodeEditorHandlesStore>()
                .and_then(|store| store.per_window.get(&window).cloned())
            {
                edit_can_undo |= handle.can_undo();
                edit_can_redo |= handle.can_redo();
            }
        }

        app.with_global_mut(WindowCommandAvailabilityService::default, |svc, _app| {
            svc.set_edit_availability(window, edit_can_undo, edit_can_redo);
        });
    }

    fn generate_avatar_demo_image_rgba8(width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        let w = (width.saturating_sub(1)).max(1) as f32;
        let h = (height.saturating_sub(1)).max(1) as f32;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let fx = x as f32 / w;
                let fy = y as f32 / h;

                let cx = fx - 0.5;
                let cy = fy - 0.5;
                let d = (cx * cx + cy * cy).sqrt().min(1.0);
                let highlight = (1.0 - d).powf(1.6);

                let r = (40.0 + 140.0 * fx + 60.0 * highlight).min(255.0) as u8;
                let g = (55.0 + 110.0 * (1.0 - fy) + 70.0 * highlight).min(255.0) as u8;
                let b = (90.0 + 110.0 * (0.5 + 0.5 * (fx - fy)).abs() + 80.0 * highlight).min(255.0)
                    as u8;

                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }

        out
    }

    fn generate_fit_demo_image_rgba8(width: u32, height: u32, accent: (u8, u8, u8)) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        let w = (width.saturating_sub(1)).max(1) as f32;
        let h = (height.saturating_sub(1)).max(1) as f32;

        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                let fx = x as f32 / w;
                let fy = y as f32 / h;

                let mut r = (20.0 + (accent.0 as f32) * (0.25 + 0.75 * fx)) as u8;
                let mut g = (20.0 + (accent.1 as f32) * (0.25 + 0.75 * (1.0 - fy))) as u8;
                let mut b =
                    (20.0 + (accent.2 as f32) * (0.25 + 0.75 * (0.5 + 0.5 * (fx - fy)))) as u8;

                let border = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;
                if border {
                    r = 245;
                    g = 245;
                    b = 245;
                }

                let dx = (x as i32 - cx).abs();
                let dy = (y as i32 - cy).abs();
                if dx <= 1 || dy <= 1 {
                    r = 10;
                    g = 10;
                    b = 10;
                }

                out[idx] = r;
                out[idx + 1] = g;
                out[idx + 2] = b;
                out[idx + 3] = 255;
            }
        }

        out
    }

    fn encode_rgba8_png_bytes(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::{ColorType, ImageEncoder as _};

        let mut out: Vec<u8> = Vec::new();
        PngEncoder::new(&mut out)
            .write_image(rgba, width, height, ColorType::Rgba8.into())
            .expect("png encode must succeed for demo bytes");
        out
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> UiGalleryWindowState {
        let page_router = build_ui_gallery_page_router();

        let start_page = ui_gallery_start_page().unwrap_or_else(|| {
            if std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
            {
                Arc::<str>::from(PAGE_OVERLAY)
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
            Arc::<str>::from(PAGE_ICONS),
            Arc::<str>::from(PAGE_FIELD),
            Arc::<str>::from(PAGE_OVERLAY),
            Arc::<str>::from(PAGE_COMMAND),
        ];
        if !workspace_tabs_init
            .iter()
            .any(|page| page.as_ref() == start_page.as_ref())
        {
            workspace_tabs_init.push(start_page);
        }

        let mut workspace_tab_close_by_command: HashMap<Arc<str>, Arc<str>> = HashMap::new();
        for tab_id in workspace_tabs_init.iter() {
            let cmd: Arc<str> = Arc::from(format!(
                "{}{}",
                CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                tab_id.as_ref()
            ));
            workspace_tab_close_by_command.insert(cmd, tab_id.clone());
        }
        let workspace_tabs = app.models_mut().insert(workspace_tabs_init);
        let workspace_dirty_tabs = app
            .models_mut()
            .insert(vec![Arc::<str>::from(PAGE_OVERLAY)]);
        let nav_query = app.models_mut().insert(String::new());
        let content_tab = app.models_mut().insert(Some(Arc::<str>::from("preview")));
        let theme_preset = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("zinc/light")));
        let theme_preset_open = app.models_mut().insert(false);
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let alert_dialog_open = app.models_mut().insert(false);
        let sheet_open = app.models_mut().insert(false);
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
        let undo_doc: DocumentId = "ui_gallery.window".into();
        let select_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("apple")));
        let select_open = app.models_mut().insert(false);
        let combobox_value = app.models_mut().insert(None::<Arc<str>>);
        let combobox_open = app.models_mut().insert(false);
        let combobox_query = app.models_mut().insert(String::new());

        let date_picker_open = app.models_mut().insert(false);
        let today = time::OffsetDateTime::now_utc().date();
        let date_picker_month = app
            .models_mut()
            .insert(fret_ui_headless::calendar::CalendarMonth::from_date(today));
        let date_picker_selected = app.models_mut().insert(None::<Date>);
        let time_picker_open = app.models_mut().insert(false);
        let time_picker_selected = app
            .models_mut()
            .insert(time::Time::from_hms(9, 41, 0).expect("valid time"));

        let resizable_h_fractions = app.models_mut().insert(vec![0.5, 0.5]);
        let resizable_v_fractions = app.models_mut().insert(vec![0.25, 0.75]);

        let data_table_state = app
            .models_mut()
            .insert(fret_ui_headless::table::TableState::default());
        let data_grid_selected_row = app.models_mut().insert(None::<u64>);
        let tabs_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("overview")));
        let accordion_value = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("item-1")));

        Self::ensure_image_source_demo_assets_installed(app);

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

        let image_fit_demo_streaming_image = app.models_mut().insert(None::<ImageId>);
        let image_fit_demo_streaming_token = app.next_image_upload_token();
        Self::enqueue_image_fit_demo_image_register(
            app,
            window,
            image_fit_demo_streaming_token,
            Self::IMAGE_FIT_DEMO_STREAMING_SIZE,
            (140, 230, 170),
        );

        let progress = app.models_mut().insert(35.0f32);
        let checkbox = app.models_mut().insert(false);
        let switch = app.models_mut().insert(true);
        let code_editor_syntax_rust = app.models_mut().insert(true);
        let code_editor_boundary_identifier = app.models_mut().insert(true);
        let code_editor_soft_wrap = app.models_mut().insert(false);
        let code_editor_folds = app.models_mut().insert(false);
        let code_editor_inlays = app.models_mut().insert(false);
        let material3_checkbox = app.models_mut().insert(false);
        let material3_switch = app.models_mut().insert(false);
        let material3_radio_value = app.models_mut().insert(None::<Arc<str>>);
        let material3_tabs_value = app.models_mut().insert(Arc::<str>::from("overview"));
        let material3_list_value = app.models_mut().insert(Arc::<str>::from("alpha"));
        let material3_expressive = app.models_mut().insert(false);
        let material3_navigation_bar_value = app.models_mut().insert(Arc::<str>::from("search"));
        let material3_navigation_rail_value = app.models_mut().insert(Arc::<str>::from("search"));
        let material3_navigation_drawer_value = app.models_mut().insert(Arc::<str>::from("search"));
        let material3_modal_navigation_drawer_open = app.models_mut().insert(false);
        let material3_dialog_open = app.models_mut().insert(false);
        let material3_text_field_value = app.models_mut().insert(String::new());
        let material3_text_field_disabled = app.models_mut().insert(false);
        let material3_text_field_error = app.models_mut().insert(false);
        let material3_autocomplete_value = app.models_mut().insert(String::new());
        let material3_autocomplete_disabled = app.models_mut().insert(false);
        let material3_autocomplete_error = app.models_mut().insert(false);
        let material3_autocomplete_dialog_open = app.models_mut().insert(false);
        let material3_menu_open = app.models_mut().insert(false);
        let text_input = app.models_mut().insert(String::new());
        let text_area = app.models_mut().insert(String::new());
        let dropdown_open = app.models_mut().insert(false);
        let context_menu_open = app.models_mut().insert(false);
        let context_menu_edge_open = app.models_mut().insert(false);
        let cmdk_open = app.models_mut().insert(false);
        let cmdk_query = app.models_mut().insert(String::new());
        let last_action = app.models_mut().insert(Arc::<str>::from("<none>"));
        let sonner_position = app.models_mut().insert(shadcn::ToastPosition::TopCenter);
        let menu_bar_seq = app.models_mut().insert(0_u64);
        let virtual_list_torture_jump = app.models_mut().insert(String::from("9000"));
        let virtual_list_torture_edit_row = app.models_mut().insert(None::<u64>);
        let virtual_list_torture_edit_text = app.models_mut().insert(String::new());
        let virtual_list_torture_scroll = VirtualListScrollHandle::new();

        let env_bool = |name: &str, default: bool| {
            let Some(v) = std::env::var_os(name).filter(|v| !v.is_empty()) else {
                return default;
            };
            let v = v.to_string_lossy().trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "no" || v == "off")
        };

        let view_cache_enabled = app
            .models_mut()
            .insert(env_bool("FRET_UI_GALLERY_VIEW_CACHE", false));
        let view_cache_cache_shell = app
            .models_mut()
            .insert(env_bool("FRET_UI_GALLERY_VIEW_CACHE_SHELL", false));
        let view_cache_inner_enabled = app
            .models_mut()
            .insert(env_bool("FRET_UI_GALLERY_VIEW_CACHE_INNER", true));
        let view_cache_popover_open = app.models_mut().insert(false);
        let view_cache_continuous = app
            .models_mut()
            .insert(env_bool("FRET_UI_GALLERY_VIEW_CACHE_CONTINUOUS", false));
        let view_cache_counter = app.models_mut().insert(0u64);

        let inspector_enabled = app.models_mut().insert(
            std::env::var_os("FRET_UI_GALLERY_INSPECTOR").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()),
        );
        let inspector_last_pointer = app.models_mut().insert(None::<fret_core::Point>);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(env_bool("FRET_UI_GALLERY_VIEW_CACHE", false));
        ui.set_debug_enabled(
            std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()),
        );

        Self::sync_undo_availability(app, window, &undo_doc);

        let mut state = UiGalleryWindowState {
            ui,
            root: None,
            debug_hud: DebugHudState::default(),
            pending_taffy_dump: None,
            page_router,
            selected_page,
            workspace_tabs,
            workspace_dirty_tabs,
            workspace_tab_close_by_command,
            nav_query,
            content_tab,
            theme_preset,
            theme_preset_open,
            applied_theme_preset: None,
            view_cache_enabled,
            view_cache_cache_shell,
            view_cache_inner_enabled,
            view_cache_popover_open,
            view_cache_continuous,
            view_cache_counter,
            inspector_enabled,
            inspector_last_pointer,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            portal_geometry_popover_open,
            settings_open,
            settings_menu_bar_os,
            settings_menu_bar_os_open,
            settings_menu_bar_in_window,
            settings_menu_bar_in_window_open,
            settings_edit_can_undo,
            settings_edit_can_redo,
            undo_doc,
            select_value,
            select_open,
            combobox_value,
            combobox_open,
            combobox_query,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
            time_picker_open,
            time_picker_selected,
            resizable_h_fractions,
            resizable_v_fractions,
            data_table_state,
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
            image_fit_demo_streaming_image,
            image_fit_demo_streaming_token: Some(image_fit_demo_streaming_token),
            image_fit_demo_streaming_frame: 0,
            image_fit_demo_streaming_size: Self::IMAGE_FIT_DEMO_STREAMING_SIZE,
            progress,
            checkbox,
            switch,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
            code_editor_soft_wrap,
            code_editor_folds,
            code_editor_inlays,
            material3_checkbox,
            material3_switch,
            material3_radio_value,
            material3_tabs_value,
            material3_list_value,
            material3_expressive,
            material3_navigation_bar_value,
            material3_navigation_rail_value,
            material3_navigation_drawer_value,
            material3_modal_navigation_drawer_open,
            material3_dialog_open,
            material3_text_field_value,
            material3_text_field_disabled,
            material3_text_field_error,
            material3_autocomplete_value,
            material3_autocomplete_disabled,
            material3_autocomplete_error,
            material3_autocomplete_dialog_open,
            material3_menu_open,
            text_input,
            text_area,
            dropdown_open,
            context_menu_open,
            context_menu_edge_open,
            cmdk_open,
            cmdk_query,
            last_action,
            sonner_position,
            menu_bar_seq,
            virtual_list_torture_jump,
            virtual_list_torture_edit_row,
            virtual_list_torture_edit_text,
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

        #[cfg(not(target_arch = "wasm32"))]
        app.with_global_mut(UiGalleryHarnessDiagnosticsStore::default, |store, _app| {
            store.per_window.insert(
                window,
                UiGalleryHarnessModelIds {
                    selected_page: state.selected_page.clone(),
                    code_editor_syntax_rust: state.code_editor_syntax_rust.clone(),
                    code_editor_boundary_identifier: state.code_editor_boundary_identifier.clone(),
                    code_editor_soft_wrap: state.code_editor_soft_wrap.clone(),
                    code_editor_folds: state.code_editor_folds.clone(),
                    code_editor_inlays: state.code_editor_inlays.clone(),
                    text_input: state.text_input.clone(),
                    text_area: state.text_area.clone(),
                },
            );
            if store.focused_window.is_none() {
                store.focused_window = Some(window);
            }
        });

        // Sync once after per-window state is registered so dynamic menu content (e.g. window list)
        // can be derived from the latest app globals.
        Self::sync_menu_bar_after_state_change(app, window);
        Self::bump_menu_bar_seq(app, &state.menu_bar_seq);

        state
    }

    #[cfg(target_arch = "wasm32")]
    fn sync_page_router_from_external_history(
        app: &mut App,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
    ) {
        let Ok(update) = state.page_router.sync_with_prefetch_intents() else {
            return;
        };

        if !update.update.changed() {
            return;
        }

        let next_page = page_from_gallery_location(&state.page_router.state().location)
            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        let next_page_for_selected = next_page.clone();
        let next_page_for_tabs = next_page.clone();

        let _ = app
            .models_mut()
            .update(&state.selected_page, |v| *v = next_page_for_selected);
        let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
            if !tabs
                .iter()
                .any(|t| t.as_ref() == next_page_for_tabs.as_ref())
            {
                tabs.push(next_page_for_tabs.clone());
            }
        });

        let cmd: Arc<str> = Arc::from(format!(
            "{}{}",
            CMD_WORKSPACE_TAB_CLOSE_PREFIX,
            next_page_for_tabs.as_ref()
        ));
        state
            .workspace_tab_close_by_command
            .insert(cmd, next_page_for_tabs);

        apply_page_router_update_side_effects(
            app,
            window,
            next_page.clone(),
            &mut state.page_router,
            Ok(update),
        );

        let _ = app.models_mut().update(&state.last_action, |v| {
            *v = Arc::<str>::from(format!("gallery.page_history.sync({})", next_page.as_ref()));
        });
    }

    fn handle_nav_command(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        if matches!(
            command.as_str(),
            CMD_GALLERY_PAGE_BACK | CMD_GALLERY_PAGE_FORWARD
        ) {
            let action = if command.as_str() == CMD_GALLERY_PAGE_BACK {
                NavigationAction::Back
            } else {
                NavigationAction::Forward
            };
            let update = state
                .page_router
                .navigate_with_prefetch_intents(action, None);

            let next_page = page_from_gallery_location(&state.page_router.state().location)
                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
            let next_page_for_selected = next_page.clone();
            let next_page_for_tabs = next_page.clone();

            let _ = app
                .models_mut()
                .update(&state.selected_page, |v| *v = next_page_for_selected);
            let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
                if !tabs
                    .iter()
                    .any(|t| t.as_ref() == next_page_for_tabs.as_ref())
                {
                    tabs.push(next_page_for_tabs.clone());
                }
            });

            let cmd: Arc<str> = Arc::from(format!(
                "{}{}",
                CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                next_page_for_tabs.as_ref()
            ));
            state
                .workspace_tab_close_by_command
                .insert(cmd, next_page_for_tabs);

            apply_page_router_update_side_effects(
                app,
                window,
                next_page.clone(),
                &mut state.page_router,
                update,
            );

            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from(format!(
                    "gallery.page_history.{}({})",
                    action,
                    next_page.as_ref()
                ));
            });

            return true;
        }

        let Some(page) = page_id_for_nav_command(command.as_str()) else {
            return false;
        };

        let page: Arc<str> = Arc::from(page);
        let page_for_selected = page.clone();
        let page_for_tabs = page.clone();
        let _ = app
            .models_mut()
            .update(&state.selected_page, |v| *v = page_for_selected);
        let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
            if !tabs.iter().any(|t| t.as_ref() == page_for_tabs.as_ref()) {
                tabs.push(page_for_tabs.clone());
            }
        });

        let cmd: Arc<str> = Arc::from(format!(
            "{}{}",
            CMD_WORKSPACE_TAB_CLOSE_PREFIX,
            page_for_tabs.as_ref()
        ));
        state
            .workspace_tab_close_by_command
            .insert(cmd, page_for_tabs);

        apply_page_route_side_effects_via_router(
            app,
            window,
            NavigationAction::Push,
            page.clone(),
            &mut state.page_router,
        );
        true
    }

    fn handle_workspace_tab_command(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        let close_tab_by_id =
            |app: &mut App, state: &mut UiGalleryWindowState, tab_id: Arc<str>| -> bool {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                let mut closed = false;
                let mut next_selected: Option<Arc<str>> = None;

                let _ = app.models_mut().update(&state.workspace_tabs, |tabs| {
                    let Some(index) = tabs.iter().position(|t| t.as_ref() == tab_id.as_ref())
                    else {
                        return;
                    };
                    if tabs.len() <= 1 {
                        return;
                    }

                    tabs.remove(index);
                    closed = true;

                    if selected.as_ref() == tab_id.as_ref() {
                        let next_index = index.min(tabs.len().saturating_sub(1));
                        next_selected = tabs.get(next_index).cloned();
                    }
                });

                if !closed {
                    return false;
                }

                let cmd: Arc<str> = Arc::from(format!(
                    "{}{}",
                    CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                    tab_id.as_ref()
                ));
                state.workspace_tab_close_by_command.remove(cmd.as_ref());

                let _ = app
                    .models_mut()
                    .update(&state.workspace_dirty_tabs, |dirty| {
                        dirty.retain(|t| t.as_ref() != tab_id.as_ref());
                    });

                if let Some(next) = next_selected {
                    let _ = app.models_mut().update(&state.selected_page, |v| *v = next);
                    let current_page = app
                        .models()
                        .get_cloned(&state.selected_page)
                        .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                    apply_page_route_side_effects_via_router(
                        app,
                        window,
                        NavigationAction::Replace,
                        current_page,
                        &mut state.page_router,
                    );
                }

                true
            };

        match command.as_str() {
            CMD_WORKSPACE_TAB_NEXT | CMD_WORKSPACE_TAB_PREV => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                let tabs = app
                    .models()
                    .get_cloned(&state.workspace_tabs)
                    .unwrap_or_default();
                if tabs.is_empty() {
                    return false;
                }
                let Some(index) = tabs.iter().position(|t| t.as_ref() == selected.as_ref()) else {
                    return false;
                };

                let next_index = if command.as_str() == CMD_WORKSPACE_TAB_NEXT {
                    (index + 1) % tabs.len()
                } else {
                    (index + tabs.len() - 1) % tabs.len()
                };
                if let Some(next) = tabs.get(next_index).cloned() {
                    let _ = app.models_mut().update(&state.selected_page, |v| *v = next);
                    let current_page = app
                        .models()
                        .get_cloned(&state.selected_page)
                        .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                    apply_page_route_side_effects_via_router(
                        app,
                        window,
                        NavigationAction::Replace,
                        current_page,
                        &mut state.page_router,
                    );
                    return true;
                }
                false
            }
            CMD_WORKSPACE_TAB_CLOSE => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                close_tab_by_id(app, state, selected)
            }
            _ => {
                if let Some(tab_id) = state
                    .workspace_tab_close_by_command
                    .get(command.as_str())
                    .cloned()
                {
                    return close_tab_by_id(app, state, tab_id);
                }
                false
            }
        }
    }

    fn handle_gallery_command(
        app: &mut App,
        state: &mut UiGalleryWindowState,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        if let Some(index) =
            Self::parse_dynamic_command_index(command, CMD_GALLERY_RECENT_OPEN_PREFIX)
        {
            let Some(title) = Self::recent_menu_items(app).into_iter().nth(index) else {
                return false;
            };
            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from(format!("recent.open({})", title.as_ref()));
            });
            return true;
        }

        if let Some(index) =
            Self::parse_dynamic_command_index(command, CMD_GALLERY_WINDOW_ACTIVATE_PREFIX)
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let Some(target_window) = Self::window_menu_items(app).into_iter().nth(index)
                else {
                    return false;
                };
                let is_diag_automation =
                    std::env::var_os("FRET_DIAG").is_some_and(|value| !value.is_empty());

                if is_diag_automation {
                    app.with_global_mut(
                        UiGalleryHarnessDiagnosticsStore::default,
                        |store, _app| {
                            if store.per_window.contains_key(&target_window) {
                                store.focused_window = Some(target_window);
                            }
                        },
                    );
                    Self::sync_menu_bar_after_state_change(app, window);
                    Self::bump_menu_bar_seq(app, &state.menu_bar_seq);
                } else {
                    app.push_effect(Effect::Window(WindowRequest::Raise {
                        window: target_window,
                        sender: Some(window),
                    }));
                }

                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from(format!("window.activate({})", index + 1));
                });
                return true;
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = index;
                return false;
            }
        }

        match command.as_str() {
            CMD_GALLERY_DEBUG_RECENT_ADD => {
                let mut label: Option<Arc<str>> = None;
                app.with_global_mut(UiGalleryRecentItemsService::default, |svc, _app| {
                    svc.next_id = svc.next_id.saturating_add(1);
                    let id = svc.next_id;
                    let next: Arc<str> = Arc::from(format!("Recent {id}"));
                    svc.items.insert(0, next.clone());
                    svc.items.truncate(10);
                    label = Some(next);
                });

                Self::sync_menu_bar_after_state_change(app, window);
                Self::bump_menu_bar_seq(app, &state.menu_bar_seq);

                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from(format!(
                        "debug.recent.add({})",
                        label.as_deref().unwrap_or("unknown")
                    ));
                });
            }
            CMD_GALLERY_DEBUG_RECENT_CLEAR => {
                app.with_global_mut(UiGalleryRecentItemsService::default, |svc, _app| {
                    svc.items.clear();
                });
                Self::sync_menu_bar_after_state_change(app, window);
                Self::bump_menu_bar_seq(app, &state.menu_bar_seq);
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("debug.recent.clear");
                });
            }
            CMD_GALLERY_DEBUG_WINDOW_OPEN => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let logical_window_id =
                        app.with_global_mut(UiGalleryDebugWindowService::default, |svc, _app| {
                            svc.next_logical_window_id =
                                svc.next_logical_window_id.saturating_add(1);
                            format!("ui-gallery-debug-window-{}", svc.next_logical_window_id)
                        });

                    app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                        kind: CreateWindowKind::DockRestore {
                            logical_window_id: logical_window_id.clone(),
                        },
                        anchor: None,
                        role: WindowRole::Auxiliary,
                        style: WindowStyleRequest {
                            activation: Some(ActivationPolicy::NonActivating),
                            ..Default::default()
                        },
                    })));
                    app.push_effect(Effect::Window(WindowRequest::Raise {
                        window,
                        sender: Some(window),
                    }));
                    app.with_global_mut(UiGalleryDebugWindowService::default, |svc, _app| {
                        svc.script_keepalive_window = Some(window);
                        svc.script_keepalive_frames = 180;
                    });
                    app.push_effect(Effect::SetTimer {
                        window: None,
                        token: DEBUG_WINDOW_OPEN_KEEPALIVE_TIMER,
                        after: Duration::from_millis(16),
                        repeat: Some(Duration::from_millis(16)),
                    });

                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from(format!("debug.window.open({logical_window_id})"));
                    });
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("debug.window.open.unsupported");
                    });
                }
            }
            CMD_CODE_EDITOR_LOAD_FONTS => {
                app.push_effect(Effect::FileDialogOpen {
                    window,
                    options: FileDialogOptions {
                        title: Some("Load fonts".to_string()),
                        multiple: true,
                        filters: vec![FileDialogFilter {
                            name: "Font files".to_string(),
                            extensions: vec![
                                "ttf".to_string(),
                                "otf".to_string(),
                                "ttc".to_string(),
                            ],
                        }],
                    },
                });

                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("code_editor.load_fonts");
                });
            }
            CMD_CODE_EDITOR_DUMP_TAFFY => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    state.pending_taffy_dump = Some(PendingTaffyDumpRequest {
                        // Prefer dumping the code editor subtree when present; fall back to the
                        // full UI root when the filter does not match anything.
                        root_label_filter: Some(Arc::<str>::from("ui-gallery-code-editor-root")),
                        filename_tag: Arc::<str>::from("ui_gallery.code_editor"),
                    });

                    let sonner = shadcn::Sonner::global(app);
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_message(
                        &mut host,
                        window,
                        "Layout dump queued",
                        shadcn::ToastMessageOptions::new().description(
                            "Will write a Taffy dump to .fret/taffy-dumps on the next frame.",
                        ),
                    );

                    let _ = host.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("code_editor.dump_taffy");
                    });
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let sonner = shadcn::Sonner::global(app);
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_error_message(
                        &mut host,
                        window,
                        "Layout dump unsupported",
                        shadcn::ToastMessageOptions::new()
                            .description("Writing debug dumps is not supported on wasm."),
                    );
                }
            }
            CMD_APP_TOGGLE_PREFERENCES_ENABLED => {
                let preferences = CommandId::new(fret_app::core_commands::APP_PREFERENCES);
                let is_disabled = app
                    .global::<WindowCommandEnabledService>()
                    .and_then(|svc| svc.enabled(window, &preferences))
                    == Some(false);

                app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
                    if is_disabled {
                        svc.clear_command(window, &preferences);
                    } else {
                        svc.set_enabled(window, preferences.clone(), false);
                    }
                });

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                if is_disabled {
                    sonner.toast_success_message(
                        &mut host,
                        window,
                        "Preferences enabled",
                        shadcn::ToastMessageOptions::new()
                            .description("Cleared WindowCommandEnabledService override."),
                    );
                    let _ = host.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("cmd.preferences_enabled");
                    });
                } else {
                    sonner.toast_error_message(
                        &mut host,
                        window,
                        "Preferences disabled",
                        shadcn::ToastMessageOptions::new()
                            .description("Set WindowCommandEnabledService override: disabled."),
                    );
                    let _ = host.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("cmd.preferences_disabled");
                    });
                }
            }
            CMD_PROGRESS_INC => {
                let before = app.models().get_copied(&state.progress).unwrap_or(0.0);
                let after = (before + 10.0).min(100.0);
                let _ = app.models_mut().update(&state.progress, |v| *v = after);
                app.with_global_mut(
                    || UndoService::<ValueTx<f32>>::with_limit(256),
                    |undo_svc, _app| {
                        undo_svc.set_active_document(window, state.undo_doc.clone());
                        undo_svc.record_or_coalesce_active(
                            window,
                            UndoRecord::new(ValueTx::new(before, after))
                                .label("Progress")
                                .coalesce_key(CoalesceKey::from("ui_gallery.progress")),
                        );
                    },
                );
                Self::sync_undo_availability(app, window, &state.undo_doc);
            }
            CMD_PROGRESS_DEC => {
                let before = app.models().get_copied(&state.progress).unwrap_or(0.0);
                let after = (before - 10.0).max(0.0);
                let _ = app.models_mut().update(&state.progress, |v| *v = after);
                app.with_global_mut(
                    || UndoService::<ValueTx<f32>>::with_limit(256),
                    |undo_svc, _app| {
                        undo_svc.set_active_document(window, state.undo_doc.clone());
                        undo_svc.record_or_coalesce_active(
                            window,
                            UndoRecord::new(ValueTx::new(before, after))
                                .label("Progress")
                                .coalesce_key(CoalesceKey::from("ui_gallery.progress")),
                        );
                    },
                );
                Self::sync_undo_availability(app, window, &state.undo_doc);
            }
            CMD_PROGRESS_RESET => {
                let before = app.models().get_copied(&state.progress).unwrap_or(0.0);
                let after = 35.0;
                let _ = app.models_mut().update(&state.progress, |v| *v = after);
                app.with_global_mut(
                    || UndoService::<ValueTx<f32>>::with_limit(256),
                    |undo_svc, _app| {
                        undo_svc.set_active_document(window, state.undo_doc.clone());
                        undo_svc.record_active(
                            window,
                            UndoRecord::new(ValueTx::new(before, after)).label("Reset progress"),
                        );
                    },
                );
                Self::sync_undo_availability(app, window, &state.undo_doc);
            }
            CMD_VIEW_CACHE_BUMP => {
                let _ = app
                    .models_mut()
                    .update(&state.view_cache_counter, |v| *v = v.saturating_add(1));
            }
            CMD_VIEW_CACHE_RESET => {
                let _ = app
                    .models_mut()
                    .update(&state.view_cache_counter, |v| *v = 0);
            }
            _ => return false,
        }
        true
    }

    fn menu_bar_mode_key(mode: MenuBarIntegrationModeV1) -> Arc<str> {
        match mode {
            MenuBarIntegrationModeV1::Auto => Arc::from("auto"),
            MenuBarIntegrationModeV1::On => Arc::from("on"),
            MenuBarIntegrationModeV1::Off => Arc::from("off"),
        }
    }

    fn menu_bar_mode_from_key(key: Option<&str>) -> MenuBarIntegrationModeV1 {
        match key.unwrap_or("auto") {
            "on" => MenuBarIntegrationModeV1::On,
            "off" => MenuBarIntegrationModeV1::Off,
            _ => MenuBarIntegrationModeV1::Auto,
        }
    }

    fn apply_menu_bar_settings(
        app: &mut App,
        os: MenuBarIntegrationModeV1,
        in_window: MenuBarIntegrationModeV1,
    ) {
        app.with_global_mut(SettingsFileV1::default, |settings, _app| {
            settings.menu_bar.os = os;
            settings.menu_bar.in_window = in_window;
        });
    }

    fn sync_menu_bar_after_state_change(app: &mut App, window: AppWindowId) {
        Self::sync_dynamic_menu_command_metadata(app);
        let menu_bar = Self::build_menu_bar(app);
        fret_app::set_menu_bar_baseline(app, menu_bar);
        fret_app::sync_os_menu_bar(app);
        app.request_redraw(window);
    }

    fn bump_menu_bar_seq(app: &mut App, seq: &Model<u64>) {
        let _ = app.models_mut().update(seq, |v| {
            *v = v.saturating_add(1);
        });
    }

    fn handle_menu_bar_mode_command(
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

        let _ = app.models_mut().update(&state.settings_menu_bar_os, |v| {
            *v = Some(Self::menu_bar_mode_key(os));
        });
        let _ = app
            .models_mut()
            .update(&state.settings_menu_bar_in_window, |v| {
                *v = Some(Self::menu_bar_mode_key(in_window));
            });

        let _ = app.models_mut().update(&state.last_action, |v| {
            *v = Arc::<str>::from(last_action);
        });

        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn write_project_settings_menu_bar(
        os: MenuBarIntegrationModeV1,
        in_window: MenuBarIntegrationModeV1,
    ) -> Result<(), std::io::Error> {
        let project_dir = std::path::Path::new(fret_app::PROJECT_CONFIG_DIR);
        std::fs::create_dir_all(project_dir)?;
        let path = project_dir.join(fret_app::SETTINGS_JSON);

        let payload = serde_json::json!({
            "settings_version": 1,
            "menu_bar": {
                "os": Self::menu_bar_mode_key(os).as_ref(),
                "in_window": Self::menu_bar_mode_key(in_window).as_ref(),
            }
        });

        let json = serde_json::to_string_pretty(&payload)
            .unwrap_or_else(|_| "{\"settings_version\":1}".to_string());
        std::fs::write(path, format!("{json}\n"))?;
        Ok(())
    }

    fn sync_shadcn_theme(app: &mut App, state: &mut UiGalleryWindowState) {
        let preset = app.models().get_cloned(&state.theme_preset).flatten();
        if preset.as_deref() == state.applied_theme_preset.as_deref() {
            return;
        }

        let Some(preset) = preset else {
            return;
        };

        let Some((base, scheme)) = preset.split_once('/') else {
            return;
        };

        let base = match base {
            "neutral" => shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            "zinc" => shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
            "slate" => shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            "stone" => shadcn::shadcn_themes::ShadcnBaseColor::Stone,
            "gray" => shadcn::shadcn_themes::ShadcnBaseColor::Gray,
            _ => return,
        };

        let scheme = match scheme {
            "light" => shadcn::shadcn_themes::ShadcnColorScheme::Light,
            "dark" => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            _ => return,
        };

        shadcn::shadcn_themes::apply_shadcn_new_york_v4(app, base, scheme);

        // Inject Material 3 v30 motion/state/typography tokens on top of the active theme preset.
        //
        // This keeps the gallery's base theme selection (shadcn light/dark) intact while enabling
        // Material components to query their extra token kinds via the shared theme system.
        fret_ui::Theme::with_global_mut(app, |theme| {
            let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
                fret_ui_material3::tokens::v30::TypographyOptions::default(),
                fret_ui_material3::tokens::v30::ColorSchemeOptions {
                    mode: match scheme {
                        shadcn::shadcn_themes::ShadcnColorScheme::Light => {
                            fret_ui_material3::tokens::v30::SchemeMode::Light
                        }
                        shadcn::shadcn_themes::ShadcnColorScheme::Dark => {
                            fret_ui_material3::tokens::v30::SchemeMode::Dark
                        }
                    },
                    ..Default::default()
                },
            );
            theme.extend_tokens_from_config(&cfg);
        });

        // Ensure the header theme select cannot remain visually open across a full theme swap.
        // In gallery flows this prevents stale modal barrier/content from overlapping the next
        // select interaction on the page content.
        let _ = app
            .models_mut()
            .update(&state.theme_preset_open, |open| *open = false);

        state.applied_theme_preset = Some(preset);
    }

    fn render_ui(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        bounds: fret_core::Rect,
    ) {
        let frame = render_flow::begin_frame(app, window, state);
        let root = render_flow::render_root(app, services, window, state, bounds, &frame);
        render_flow::end_frame(app, services, window, state, bounds, &frame, root);
    }
}

pub fn build_app() -> App {
    fn ui_gallery_project_root() -> std::path::PathBuf {
        let raw = std::env::var_os("FRET_UI_GALLERY_PROJECT_ROOT")
            .and_then(|v| (!v.is_empty()).then_some(v));
        let Some(raw) = raw else {
            return std::path::PathBuf::from(".");
        };

        let trimmed = raw.to_string_lossy();
        let trimmed = trimmed.trim();
        if trimmed.is_empty() {
            return std::path::PathBuf::from(".");
        }

        std::path::PathBuf::from(trimmed)
    }

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(UiGalleryRecentItemsService::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let project_root = ui_gallery_project_root();
    let config_paths = LayeredConfigPaths::for_project_root(&project_root);
    if let Ok((settings, _report)) = load_layered_settings(&config_paths) {
        fret_app::settings::apply_settings_globals(&mut app, &settings);
    }

    app_bootstrap::register_commands_and_menus(&mut app);

    diag_snapshot::install_ui_gallery_snapshot_provider(&mut app);

    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    fn parse_main_window_size_override() -> Option<winit::dpi::LogicalSize<f64>> {
        let raw = std::env::var("FRET_UI_GALLERY_MAIN_WINDOW_SIZE").ok()?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut parts = trimmed
            .split(|c: char| c == 'x' || c == 'X' || c == ',' || c.is_whitespace())
            .filter(|p| !p.trim().is_empty());

        let w = parts.next()?.trim().parse::<f64>().ok()?;
        let h = parts.next()?.trim().parse::<f64>().ok()?;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }

        Some(winit::dpi::LogicalSize::new(w, h))
    }

    let main_window_size = match parse_main_window_size_override() {
        Some(size) => {
            tracing::info!(
                w = size.width,
                h = size.height,
                "ui-gallery overriding main_window_size via FRET_UI_GALLERY_MAIN_WINDOW_SIZE"
            );
            size
        }
        None => winit::dpi::LogicalSize::new(1080.0, 720.0),
    };

    WinitRunnerConfig {
        main_window_title: "fret-ui-gallery".to_string(),
        main_window_size,
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    UiGalleryDriver
}

#[cfg(test)]
mod stack_overflow_tests;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let project_root = std::env::var_os("FRET_UI_GALLERY_PROJECT_ROOT")
        .and_then(|v| (!v.is_empty()).then_some(v))
        .map(|v| {
            let s = v.to_string_lossy();
            let trimmed = s.trim();
            if trimmed.is_empty() {
                std::path::PathBuf::from(".")
            } else {
                std::path::PathBuf::from(trimmed)
            }
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    fret_bootstrap::BootstrapBuilder::new(app, build_driver())
        .configure(move |c| {
            *c = config;
        })
        .with_default_diagnostics()
        .with_default_config_files_for_root(&project_root)?
        .with_config_files_watcher_for_root(Duration::from_millis(500), &project_root)
        .with_lucide_icons()
        .preload_icon_svgs_on_gpu_ready()
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_with_event_loop(event_loop: winit::event_loop::EventLoop) -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let project_root = std::env::var_os("FRET_UI_GALLERY_PROJECT_ROOT")
        .and_then(|v| (!v.is_empty()).then_some(v))
        .map(|v| {
            let s = v.to_string_lossy();
            let trimmed = s.trim();
            if trimmed.is_empty() {
                std::path::PathBuf::from(".")
            } else {
                std::path::PathBuf::from(trimmed)
            }
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    fret_bootstrap::BootstrapBuilder::new(app, build_driver())
        .configure(move |c| {
            *c = config;
        })
        .with_default_diagnostics()
        .with_default_config_files_for_root(&project_root)?
        .with_config_files_watcher_for_root(Duration::from_millis(500), &project_root)
        .with_lucide_icons()
        .preload_icon_svgs_on_gpu_ready()
        .into_inner()
        .with_event_loop(event_loop)
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl WinitAppDriver for UiGalleryDriver {
    type WindowState = UiGalleryWindowState;

    fn gpu_ready(
        &mut self,
        app: &mut App,
        _context: &fret_render::WgpuContext,
        renderer: &mut fret_render::Renderer,
    ) {
        let wants_bootstrap_fonts =
            std::env::var_os("FRET_UI_GALLERY_BOOTSTRAP_FONTS").is_some_and(|v| !v.is_empty());
        if !wants_bootstrap_fonts {
            return;
        }

        let fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        let _ = renderer.add_fonts(fonts);

        let update = fret_runtime::apply_font_catalog_update(
            app,
            renderer.all_font_names(),
            fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );
        let _ = renderer.set_text_font_families(&update.config);
        app.set_global::<fret_core::TextFontFamilyConfig>(update.config.clone());
        app.set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
            renderer.text_font_stack_key(),
        ));
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context.app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, _app| {
                svc.record_model_changes(context.window, changed);
            },
        );
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context.app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, app| {
                svc.record_global_changes(app, context.window, changed);
            },
        );
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);

        if changed.contains(&std::any::TypeId::of::<fret_app::ConfigFilesWatcherStatus>())
            && let Some((seq, tick)) = context
                .app
                .global::<fret_app::ConfigFilesWatcherStatus>()
                .map(|svc| (svc.seq(), svc.last_tick().cloned()))
            && seq != 0
            && context.state.last_config_files_status_seq != seq
        {
            context.state.last_config_files_status_seq = seq;

            if let Some(tick) = tick {
                let has_error = tick.settings_error.is_some()
                    || tick.keymap_error.is_some()
                    || tick.menu_bar_error.is_some()
                    || tick.actionable_keymap_conflicts > 0;

                let title = if has_error {
                    "Config reload failed"
                } else {
                    "Config reloaded"
                };

                let mut details: Vec<String> = Vec::new();
                if tick.reloaded_settings {
                    details.push("settings.json".to_string());
                }
                if tick.reloaded_keymap {
                    details.push("keymap.json".to_string());
                }
                if tick.reloaded_menu_bar {
                    details.push("menubar.json".to_string());
                }
                if let Some(err) = tick.settings_error.as_deref() {
                    details.push(format!("settings: {err}"));
                }
                if let Some(err) = tick.keymap_error.as_deref() {
                    details.push(format!("keymap: {err}"));
                }
                if let Some(err) = tick.menu_bar_error.as_deref() {
                    details.push(format!("menubar: {err}"));
                }
                if tick.actionable_keymap_conflicts > 0 {
                    details.push(format!(
                        "keymap conflicts: {}",
                        tick.actionable_keymap_conflicts
                    ));
                }

                let description = if details.is_empty() {
                    None
                } else {
                    Some(details.join(" | "))
                };

                let sonner = shadcn::Sonner::global(context.app);
                let mut host = UiActionHostAdapter { app: context.app };
                let opts = shadcn::ToastMessageOptions::new()
                    .description(description.unwrap_or_else(|| "OK".to_string()))
                    .duration(Duration::from_secs(6));

                if has_error {
                    sonner.toast_error_message(&mut host, context.window, title, opts);
                } else {
                    sonner.toast_success_message(&mut host, context.window, title, opts);
                }
            }
        }
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        if command.as_str() == fret_app::core_commands::COMMAND_PALETTE
            || command.as_str() == fret_app::core_commands::COMMAND_PALETTE_LEGACY
        {
            let _ = app.models_mut().update(&state.cmdk_open, |v| *v = true);
            let _ = app.models_mut().update(&state.cmdk_query, |v| v.clear());
            app.request_redraw(window);
            return;
        }

        if Self::handle_menu_bar_mode_command(app, window, state, command.as_str()) {
            app.request_redraw(window);
            return;
        }

        if state.ui.dispatch_command(app, services, &command) {
            app.request_redraw(window);
            return;
        }

        if command.as_str() == fret_app::core_commands::EDIT_UNDO {
            let mut did_apply = false;
            app.with_global_mut(
                || UndoService::<ValueTx<f32>>::with_limit(256),
                |undo_svc, app| {
                    undo_svc.set_active_document(window, state.undo_doc.clone());
                    did_apply = undo_svc
                        .undo_active_invertible(window, |rec| {
                            let _ = app
                                .models_mut()
                                .update(&state.progress, |v| *v = rec.tx.after);
                            Ok::<(), ()>(())
                        })
                        .unwrap_or(false);
                },
            );
            if did_apply {
                Self::sync_undo_availability(app, window, &state.undo_doc);
            }
            let _ = app
                .models_mut()
                .update(&state.last_action, |v| *v = Arc::from("edit.undo"));
            app.request_redraw(window);
            return;
        }

        if command.as_str() == fret_app::core_commands::EDIT_REDO {
            let mut did_apply = false;
            app.with_global_mut(
                || UndoService::<ValueTx<f32>>::with_limit(256),
                |undo_svc, app| {
                    undo_svc.set_active_document(window, state.undo_doc.clone());
                    did_apply = undo_svc
                        .redo_active_invertible(window, |rec| {
                            let _ = app
                                .models_mut()
                                .update(&state.progress, |v| *v = rec.tx.after);
                            Ok::<(), ()>(())
                        })
                        .unwrap_or(false);
                },
            );
            if did_apply {
                Self::sync_undo_availability(app, window, &state.undo_doc);
            }
            let _ = app
                .models_mut()
                .update(&state.last_action, |v| *v = Arc::from("edit.redo"));
            app.request_redraw(window);
            return;
        }

        if Self::handle_workspace_tab_command(app, state, window, &command) {
            app.request_redraw(window);
            return;
        }

        let did_nav = Self::handle_nav_command(app, state, window, &command);
        let did_gallery = Self::handle_gallery_command(app, state, window, &command);
        if did_nav || did_gallery {
            app.request_redraw(window);
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_JUMP {
            let raw = app
                .models()
                .get_cloned(&state.virtual_list_torture_jump)
                .unwrap_or_default();
            let index = raw.trim().parse::<usize>().unwrap_or(0);
            state
                .virtual_list_torture_scroll
                .scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_SCROLL_BOTTOM {
            state.virtual_list_torture_scroll.scroll_to_bottom();
            app.request_redraw(window);
            return;
        }

        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_CLEAR_EDIT {
            let _ = app
                .models_mut()
                .update(&state.virtual_list_torture_edit_row, |v| *v = None);
            let _ = app
                .models_mut()
                .update(&state.virtual_list_torture_edit_text, |v| v.clear());
            app.request_redraw(window);
            return;
        }

        if let Some(row) = data_grid_row_for_command(command.as_str()) {
            let _ = app.models_mut().update(&state.data_grid_selected_row, |v| {
                if *v == Some(row) {
                    *v = None;
                } else {
                    *v = Some(row);
                }
            });
            app.request_redraw(window);
            return;
        }

        match command.as_str() {
            CMD_CLIPBOARD_COPY_LINK | CMD_CLIPBOARD_COPY_USAGE | CMD_CLIPBOARD_COPY_NOTES => {
                let selected = app
                    .models()
                    .get_cloned(&state.selected_page)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                let (_title, _origin, notes_md, usage_md) =
                    crate::spec::page_meta(selected.as_ref());

                let text = match command.as_str() {
                    CMD_CLIPBOARD_COPY_LINK => format!("?page={}", selected.as_ref()),
                    CMD_CLIPBOARD_COPY_USAGE => usage_md.to_string(),
                    CMD_CLIPBOARD_COPY_NOTES => notes_md.to_string(),
                    _ => String::new(),
                };

                app.push_effect(Effect::ClipboardSetText { text });

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_success_message(
                    &mut host,
                    window,
                    "Copied",
                    shadcn::ToastMessageOptions::new().description("Copied to clipboard."),
                );

                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("clipboard.copy");
                });
            }
            crate::spec::CMD_SHELL_SHARE_SHEET_SMOKE => {
                let token = app.next_share_sheet_token();
                app.push_effect(Effect::ShareSheetShow {
                    window,
                    token,
                    items: vec![
                        fret_core::ShareItem::Text("Hello from Fret (share sheet)".to_string()),
                        fret_core::ShareItem::Url("https://example.com".to_string()),
                        fret_core::ShareItem::Bytes {
                            name: "hello.txt".to_string(),
                            mime: Some("text/plain".to_string()),
                            bytes: b"Hello from Fret!\n".to_vec(),
                        },
                    ],
                });

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Share sheet",
                    shadcn::ToastMessageOptions::new().description("Requested."),
                );

                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("shell.share_sheet");
                });
            }
            CMD_MENU_DROPDOWN_APPLE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.dropdown.apple");
                });
            }
            CMD_MENU_DROPDOWN_ORANGE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.dropdown.orange");
                });
            }
            CMD_MENU_CONTEXT_ACTION => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("menu.context.action");
                });
            }
            CMD_APP_OPEN => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.open");
                });
            }
            CMD_APP_SAVE => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.save");
                });
            }
            CMD_APP_SETTINGS => {
                let open_now = app
                    .models()
                    .get_copied(&state.settings_open)
                    .unwrap_or(false);
                if !open_now {
                    let settings = app.global::<SettingsFileV1>().cloned().unwrap_or_default();
                    let _ = app.models_mut().update(&state.settings_menu_bar_os, |v| {
                        *v = Some(Self::menu_bar_mode_key(settings.menu_bar.os));
                    });
                    let _ = app
                        .models_mut()
                        .update(&state.settings_menu_bar_in_window, |v| {
                            *v = Some(Self::menu_bar_mode_key(settings.menu_bar.in_window));
                        });
                }
                let _ = app
                    .models_mut()
                    .update(&state.settings_open, |v| *v = !open_now);
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.settings");
                });
                app.request_redraw(window);
            }
            CMD_APP_SETTINGS_APPLY => {
                let os = app
                    .models()
                    .get_cloned(&state.settings_menu_bar_os)
                    .flatten()
                    .as_deref()
                    .map(str::to_string);
                let in_window = app
                    .models()
                    .get_cloned(&state.settings_menu_bar_in_window)
                    .flatten()
                    .as_deref()
                    .map(str::to_string);

                let os = Self::menu_bar_mode_from_key(os.as_deref());
                let in_window = Self::menu_bar_mode_from_key(in_window.as_deref());
                Self::apply_menu_bar_settings(app, os, in_window);
                Self::sync_menu_bar_after_state_change(app, window);
                Self::bump_menu_bar_seq(app, &state.menu_bar_seq);

                let _ = app
                    .models_mut()
                    .update(&state.settings_open, |v| *v = false);

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_success_message(
                    &mut host,
                    window,
                    "Settings applied",
                    shadcn::ToastMessageOptions::new().description("Menu bar settings updated."),
                );

                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("settings.apply");
                });
            }
            CMD_APP_SETTINGS_WRITE_PROJECT => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let os = app
                        .models()
                        .get_cloned(&state.settings_menu_bar_os)
                        .flatten()
                        .as_deref()
                        .map(str::to_string);
                    let in_window = app
                        .models()
                        .get_cloned(&state.settings_menu_bar_in_window)
                        .flatten()
                        .as_deref()
                        .map(str::to_string);

                    let os = Self::menu_bar_mode_from_key(os.as_deref());
                    let in_window = Self::menu_bar_mode_from_key(in_window.as_deref());

                    let result =
                        Self::write_project_settings_menu_bar(os, in_window).and_then(|_| {
                            let paths = LayeredConfigPaths::for_project_root(".");
                            let (settings, _report) =
                                load_layered_settings(&paths).map_err(std::io::Error::other)?;
                            fret_app::settings::apply_settings_globals(app, &settings);
                            fret_app::sync_os_menu_bar(app);
                            Ok(())
                        });

                    let sonner = shadcn::Sonner::global(app);
                    let mut host = UiActionHostAdapter { app };
                    match result {
                        Ok(()) => {
                            sonner.toast_success_message(
                                &mut host,
                                window,
                                "Wrote settings.json",
                                shadcn::ToastMessageOptions::new()
                                    .description(".fret/settings.json updated."),
                            );
                        }
                        Err(e) => {
                            sonner.toast_error_message(
                                &mut host,
                                window,
                                "Write failed",
                                shadcn::ToastMessageOptions::new().description(format!("{e}")),
                            );
                        }
                    }

                    let _ = host
                        .models_mut()
                        .update(&state.settings_open, |v| *v = false);
                    let _ = host.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("settings.write_project");
                    });
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let sonner = shadcn::Sonner::global(app);
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_error_message(
                        &mut host,
                        window,
                        "Write failed",
                        shadcn::ToastMessageOptions::new()
                            .description("Writing settings.json is not supported on wasm."),
                    );
                }
            }
            fret_app::core_commands::APP_ABOUT => {
                if Platform::current() == Platform::Macos {
                    app.push_effect(Effect::ShowAboutPanel);
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("cmd.about");
                    });
                } else {
                    let sonner = shadcn::Sonner::global(app);
                    let mut host = UiActionHostAdapter { app };
                    sonner.toast_message(
                        &mut host,
                        window,
                        "About",
                        shadcn::ToastMessageOptions::new().description("Fret UI Gallery"),
                    );
                    let _ = host.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("cmd.about");
                    });
                }
            }
            fret_app::core_commands::APP_PREFERENCES => {
                app.push_effect(Effect::Command {
                    window: Some(window),
                    command: CommandId::new(CMD_APP_SETTINGS),
                });
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.preferences");
                });
            }
            fret_app::core_commands::APP_LOCALE_SWITCH_NEXT => {
                if fret_app::core_commands::handle_locale_cycle_command(app, &command) {
                    Self::sync_menu_bar_after_state_change(app, window);
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("cmd.locale.switch_next");
                    });
                }
            }
            fret_app::core_commands::APP_QUIT => {
                app.push_effect(Effect::QuitApp);
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("cmd.quit");
                });
            }
            CMD_TOAST_DEFAULT => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast(
                    &mut host,
                    window,
                    shadcn::ToastRequest::new("Default toast")
                        .id(shadcn::ToastId(100))
                        .description("Hello from fret-ui-gallery."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.default");
                });
            }
            CMD_TOAST_SUCCESS => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast(
                    &mut host,
                    window,
                    shadcn::ToastRequest::new("Success")
                        .id(shadcn::ToastId(101))
                        .variant(shadcn::ToastVariant::Success)
                        .description("Everything worked."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.success");
                });
            }
            CMD_TOAST_ERROR => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast(
                    &mut host,
                    window,
                    shadcn::ToastRequest::new("Error")
                        .id(shadcn::ToastId(102))
                        .variant(shadcn::ToastVariant::Error)
                        .description("Something failed."),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.error");
                });
            }
            CMD_TOAST_SHOW_ACTION_CANCEL => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast(
                    &mut host,
                    window,
                    shadcn::ToastRequest::new("Action toast")
                        .id(shadcn::ToastId(103))
                        .description("Try the action/cancel buttons.")
                        .action(shadcn::ToastAction::new(
                            "Undo",
                            CommandId::new(CMD_TOAST_ACTION),
                        ))
                        .cancel(shadcn::ToastAction::new(
                            "Cancel",
                            CommandId::new(CMD_TOAST_CANCEL),
                        ))
                        .duration(Some(Duration::from_secs(6))),
                );
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.action_cancel");
                });
            }
            CMD_TOAST_ACTION => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.action");
                });
            }
            CMD_TOAST_CANCEL => {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("toast.cancel");
                });
            }
            _ => {}
        }

        app.request_redraw(window);
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let consumed =
                app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                    if !svc.is_enabled() {
                        return false;
                    }
                    if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                        return true;
                    }
                    svc.maybe_intercept_event_for_picking(app, window, event)
                });
            if consumed {
                return;
            }
        }

        match event {
            Event::FileDialogSelection(selection) => {
                app.push_effect(Effect::FileDialogReadAllWithLimits {
                    window,
                    token: selection.token,
                    limits: ExternalDropReadLimits {
                        max_total_bytes: 128 * 1024 * 1024,
                        max_file_bytes: 128 * 1024 * 1024,
                        max_files: 8,
                    },
                });
            }
            Event::FileDialogData(data) => {
                let is_font_blob = |bytes: &[u8]| -> bool {
                    bytes.starts_with(b"OTTO")
                        || bytes.starts_with(b"ttcf")
                        || bytes
                            .get(0..4)
                            .is_some_and(|b| b == [0x00, 0x01, 0x00, 0x00])
                };

                let mut fonts: Vec<Vec<u8>> = Vec::new();
                for file in &data.files {
                    let name = file.name.to_ascii_lowercase();
                    let looks_like_font = name.ends_with(".ttf")
                        || name.ends_with(".otf")
                        || name.ends_with(".ttc")
                        || is_font_blob(&file.bytes);
                    if looks_like_font {
                        fonts.push(file.bytes.clone());
                    }
                }

                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };

                if fonts.is_empty() {
                    let description = if data.errors.is_empty() {
                        "No font files found in selection.".to_string()
                    } else {
                        format!("No fonts loaded ({} read errors).", data.errors.len())
                    };
                    sonner.toast_error_message(
                        &mut host,
                        window,
                        "Load fonts failed",
                        shadcn::ToastMessageOptions::new().description(description),
                    );
                } else {
                    host.push_effect(Effect::TextAddFonts { fonts });
                    let description = if data.errors.is_empty() {
                        "Fonts added to TextSystem.".to_string()
                    } else {
                        format!(
                            "Fonts added to TextSystem ({} read errors).",
                            data.errors.len()
                        )
                    };
                    sonner.toast_success_message(
                        &mut host,
                        window,
                        "Fonts loaded",
                        shadcn::ToastMessageOptions::new().description(description),
                    );
                }

                host.push_effect(Effect::FileDialogRelease { token: data.token });
                host.request_redraw(window);
            }
            Event::FileDialogCanceled => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                sonner.toast_message(
                    &mut host,
                    window,
                    "Load fonts canceled",
                    shadcn::ToastMessageOptions::new()
                        .description("The file dialog completed without a selection."),
                );
            }
            Event::ShareSheetCompleted { token: _, outcome } => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                match outcome {
                    fret_core::ShareSheetOutcome::Shared => {
                        sonner.toast_success_message(
                            &mut host,
                            window,
                            "Share sheet",
                            shadcn::ToastMessageOptions::new()
                                .description("Shared successfully."),
                        );
                    }
                    fret_core::ShareSheetOutcome::Canceled => {
                        sonner.toast_message(
                            &mut host,
                            window,
                            "Share sheet",
                            shadcn::ToastMessageOptions::new().description("Canceled."),
                        );
                    }
                    fret_core::ShareSheetOutcome::Unavailable => {
                        sonner.toast_error_message(
                            &mut host,
                            window,
                            "Share sheet",
                            shadcn::ToastMessageOptions::new().description("Unavailable."),
                        );
                    }
                    fret_core::ShareSheetOutcome::Failed { message } => {
                        sonner.toast_error_message(
                            &mut host,
                            window,
                            "Share sheet",
                            shadcn::ToastMessageOptions::new().description(message.clone()),
                        );
                    }
                }
                let _ = host.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("shell.share_sheet.completed");
                });
                host.request_redraw(window);
            }
            Event::ImageRegistered { token, image, .. } => {
                if state.avatar_demo_image_token == Some(*token) {
                    state.avatar_demo_image_token = None;
                    state.avatar_demo_image_retry_count = 0;
                    let _ = app
                        .models_mut()
                        .update(&state.avatar_demo_image, |v| *v = Some(*image));
                    fret_ui_kit::with_image_metadata_store_mut(app, |store| {
                        store.set_intrinsic_size_px(
                            *image,
                            (
                                Self::AVATAR_DEMO_IMAGE_WIDTH,
                                Self::AVATAR_DEMO_IMAGE_HEIGHT,
                            ),
                        );
                    });
                    app.request_redraw(window);
                }

                if state.image_fit_demo_wide_token == Some(*token) {
                    state.image_fit_demo_wide_token = None;
                    let _ = app
                        .models_mut()
                        .update(&state.image_fit_demo_wide_image, |v| *v = Some(*image));
                    fret_ui_kit::with_image_metadata_store_mut(app, |store| {
                        store.set_intrinsic_size_px(*image, Self::IMAGE_FIT_DEMO_WIDE_SIZE);
                    });
                    app.request_redraw(window);
                }

                if state.image_fit_demo_tall_token == Some(*token) {
                    state.image_fit_demo_tall_token = None;
                    let _ = app
                        .models_mut()
                        .update(&state.image_fit_demo_tall_image, |v| *v = Some(*image));
                    fret_ui_kit::with_image_metadata_store_mut(app, |store| {
                        store.set_intrinsic_size_px(*image, Self::IMAGE_FIT_DEMO_TALL_SIZE);
                    });
                    app.request_redraw(window);
                }

                if state.image_fit_demo_streaming_token == Some(*token) {
                    state.image_fit_demo_streaming_token = None;
                    let _ = app
                        .models_mut()
                        .update(&state.image_fit_demo_streaming_image, |v| {
                            *v = Some(*image);
                        });
                    fret_ui_kit::with_image_metadata_store_mut(app, |store| {
                        store.set_intrinsic_size_px(*image, Self::IMAGE_FIT_DEMO_STREAMING_SIZE);
                    });
                    app.request_redraw(window);
                }
            }
            Event::ImageRegisterFailed { token, message } => {
                if state.avatar_demo_image_token == Some(*token) {
                    let transient_not_ready = message.contains("not initialized");
                    if transient_not_ready
                        && state.avatar_demo_image_retry_count < Self::AVATAR_DEMO_IMAGE_RETRY_MAX
                    {
                        state.avatar_demo_image_retry_count =
                            state.avatar_demo_image_retry_count.saturating_add(1);
                        Self::enqueue_avatar_demo_image_register(app, window, *token);
                        app.request_redraw(window);
                    } else {
                        state.avatar_demo_image_token = None;
                        tracing::error!(message, "ui-gallery avatar demo image register failed");
                        app.request_redraw(window);
                    }
                }

                if state.image_fit_demo_wide_token == Some(*token) {
                    state.image_fit_demo_wide_token = None;
                    tracing::error!(message, "ui-gallery image fit wide image register failed");
                    app.request_redraw(window);
                }
                if state.image_fit_demo_tall_token == Some(*token) {
                    state.image_fit_demo_tall_token = None;
                    tracing::error!(message, "ui-gallery image fit tall image register failed");
                    app.request_redraw(window);
                }
                if state.image_fit_demo_streaming_token == Some(*token) {
                    state.image_fit_demo_streaming_token = None;
                    tracing::error!(
                        message,
                        "ui-gallery image fit streaming image register failed"
                    );
                    app.request_redraw(window);
                }
            }
            Event::Timer { token } if *token == DEBUG_WINDOW_OPEN_KEEPALIVE_TIMER => {
                let (target_window, keep_running) =
                    app.with_global_mut(UiGalleryDebugWindowService::default, |svc, _app| {
                        let target_window = svc.script_keepalive_window;
                        if svc.script_keepalive_frames == 0 {
                            svc.script_keepalive_window = None;
                            return (target_window, false);
                        }

                        svc.script_keepalive_frames = svc.script_keepalive_frames.saturating_sub(1);
                        if svc.script_keepalive_frames == 0 {
                            svc.script_keepalive_window = None;
                            return (target_window, false);
                        }

                        (target_window, true)
                    });

                if let Some(target_window) = target_window {
                    app.request_redraw(target_window);
                    app.push_effect(Effect::RequestAnimationFrame(target_window));
                }

                if !keep_running {
                    app.push_effect(Effect::CancelTimer {
                        token: DEBUG_WINDOW_OPEN_KEEPALIVE_TIMER,
                    });
                }
            }
            Event::WindowFocusChanged(focused) => {
                app.with_global_mut(UiGalleryHarnessDiagnosticsStore::default, |store, _app| {
                    if *focused {
                        store.focused_window = Some(window);
                    } else if store.focused_window == Some(window) {
                        store.focused_window = None;
                    }
                });
                Self::sync_menu_bar_after_state_change(app, window);
                Self::bump_menu_bar_seq(app, &state.menu_bar_seq);
            }
            Event::WindowCloseRequested => {
                app.with_global_mut(UiGalleryHarnessDiagnosticsStore::default, |store, _app| {
                    store.per_window.remove(&window);
                    if store.focused_window == Some(window) {
                        let next_focused = store
                            .per_window
                            .keys()
                            .copied()
                            .min_by_key(|window_id| format!("{window_id:?}"));
                        store.focused_window = next_focused;
                    }
                });
                Self::sync_menu_bar_after_state_change(app, window);
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
            }
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        Self::render_ui(app, services, window, state, bounds);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let (inspection_active, diag_enabled) = app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, _app| {
                (svc.wants_inspection_active(window), svc.is_enabled())
            },
        );
        state.ui.set_inspection_active(inspection_active);
        state.ui.set_debug_enabled(diag_enabled);

        scene.clear();

        if app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled())
        {
            // Diagnostics scripts select targets by semantics bounds. We must ensure we have a
            // fresh semantics snapshot for the current frame *before* we drive scripted input;
            // otherwise, scripts may act on a 1-frame-stale snapshot and mis-predict visibility
            // in virtualized lists (estimate -> measured jumps).
            state.ui.request_semantics_snapshot();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(request) = state.pending_taffy_dump.take() {
            let root = state.root.or_else(|| state.ui.base_root());
            let result = if let Some(root) = root {
                state.ui.debug_write_taffy_subtree_json(
                    app,
                    window,
                    root,
                    bounds,
                    scale_factor,
                    request.root_label_filter.as_deref(),
                    std::path::Path::new(".fret/taffy-dumps"),
                    request.filename_tag.as_ref(),
                )
            } else {
                Err(std::io::Error::other("missing UiTree root"))
            };

            let sonner = shadcn::Sonner::global(app);
            let mut host = UiActionHostAdapter { app };
            match result {
                Ok(path) => {
                    tracing::info!(path = %path.display(), "wrote taffy dump");
                    sonner.toast_success_message(
                        &mut host,
                        window,
                        "Layout dump written",
                        shadcn::ToastMessageOptions::new()
                            .description(format!("{}", path.display())),
                    );
                }
                Err(err) => {
                    tracing::warn!(error = %err, "failed to write taffy dump");
                    sonner.toast_error_message(
                        &mut host,
                        window,
                        "Layout dump failed",
                        shadcn::ToastMessageOptions::new().description(format!("{err}")),
                    );
                }
            }
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        if app
            .models()
            .get_cloned(&state.selected_page)
            .is_some_and(|page| page.as_ref() == PAGE_IMAGE_OBJECT_FIT)
            && let Some(image) = app
                .models()
                .get_cloned(&state.image_fit_demo_streaming_image)
                .flatten()
        {
            let (width, height) = state.image_fit_demo_streaming_size;

            let bar_w = 24u32;
            let max_x = width.saturating_sub(bar_w).max(1);
            let bar_x = (state.image_fit_demo_streaming_frame as u32) % max_x;

            let mut bytes = vec![0u8; (bar_w as usize) * (height as usize) * 4];
            for px in bytes.chunks_exact_mut(4) {
                px[0] = 240;
                px[1] = 90;
                px[2] = 80;
                px[3] = 255;
            }

            app.push_effect(Effect::ImageUpdateRgba8 {
                window: Some(window),
                token: ImageUpdateToken(state.image_fit_demo_streaming_frame),
                image,
                stream_generation: 0,
                width,
                height,
                update_rect_px: Some(RectPx::new(bar_x, 0, bar_w, height)),
                bytes_per_row: bar_w * 4,
                bytes,
                color_info: ImageColorInfo::srgb_rgba(),
                alpha_mode: AlphaMode::Opaque,
            });

            state.image_fit_demo_streaming_frame =
                state.image_fit_demo_streaming_frame.saturating_add(1);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        // Drive scripted input after `paint_all()` so virtualization-heavy trees (e.g.
        // VirtualList) have their realized item subtrees available for hit-testing.
        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, app| {
                let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
                svc.drive_script_for_window(
                    &*app,
                    window,
                    bounds,
                    scale_factor,
                    Some(&state.ui),
                    semantics_snapshot,
                    element_runtime,
                )
            },
        );

        for effect in drive.effects {
            app.push_effect(effect);
        }

        if drive.request_redraw {
            app.request_redraw(window);
            // Script-driven `wait_frames` needs a reliable way to advance frames even when the
            // scene is otherwise idle. Requesting an animation frame ensures the runner
            // schedules another render tick.
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            // Script-driven events bypass the winit event loop, so we must apply any generated
            // command effects (e.g. Tab => focus traversal) before we record snapshots.
            //
            // Keep non-command effects queued for the runner to handle after `render` returns.
            let mut deferred_effects: Vec<Effect> = Vec::new();
            loop {
                let effects = app.flush_effects();
                if effects.is_empty() {
                    break;
                }

                let mut applied_any_command = false;
                for effect in effects {
                    match effect {
                        Effect::Command { window: w, command } => {
                            if w.is_none() || w == Some(window) {
                                self.handle_command(
                                    WinitCommandContext {
                                        app,
                                        services,
                                        window,
                                        state,
                                    },
                                    command,
                                );
                                applied_any_command = true;
                            } else {
                                deferred_effects.push(Effect::Command { window: w, command });
                            }
                        }
                        other => deferred_effects.push(other),
                    }
                }

                if !applied_any_command {
                    break;
                }
            }
            for effect in deferred_effects {
                app.push_effect(effect);
            }
        }

        app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, app| {
                let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
                svc.record_snapshot(
                    app,
                    window,
                    bounds,
                    scale_factor,
                    &state.ui,
                    element_runtime,
                    scene,
                );
                let _ = svc.maybe_dump_if_triggered();
                if svc.is_enabled() {
                    app.push_effect(Effect::RequestAnimationFrame(window));
                }
            },
        );
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match &request.kind {
            CreateWindowKind::DockRestore { logical_window_id } => Some(WindowCreateSpec::new(
                format!("fret-ui-gallery - {logical_window_id}"),
                winit::dpi::LogicalSize::new(980.0, 720.0),
            )),
            CreateWindowKind::DockFloating { .. } => None,
        }
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }
}

#[cfg(test)]
mod stack_overflow_repro_tests;
