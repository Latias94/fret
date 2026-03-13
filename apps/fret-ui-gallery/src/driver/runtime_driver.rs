// Included from `apps/fret-ui-gallery/src/driver.rs` to keep the module entrypoint small.

use fret::router::{NavigationAction, Router};
use fret_app::{
    ActivationPolicy, App, CommandId, CommandMeta, CreateWindowKind, CreateWindowRequest, Effect,
    LayeredConfigPaths, Menu, MenuBar, MenuBarIntegrationModeV1, MenuItem, MenuRole, Model,
    Platform, SettingsFileV1, WindowRequest, WindowRole, WindowStyleRequest, load_layered_settings,
};
use fret_core::{
    AlphaMode, AppWindowId, Event, ExternalDropReadLimits, FileDialogFilter, FileDialogOptions,
    ImageColorInfo, ImageId, ImageUploadToken, RectPx, TimerToken, UiServices,
};
use fret_icons::IconRegistry;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::{
    ImageUpdateToken, MenuItemToggle, MenuItemToggleKind, PlatformCapabilities,
    WindowCommandAvailabilityService, WindowCommandEnabledService,
};
use fret_ui::UiTree;
use fret_ui::action::{UiActionHost, UiActionHostAdapter};
#[cfg(feature = "gallery-dev")]
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_shadcn::facade as shadcn;
use fret_undo::{CoalesceKey, DocumentId, UndoRecord, UndoService, ValueTx};
use fret_workspace::commands::{
    CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
};
use fret_workspace::layout::WorkspaceWindowLayout;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::Date;

use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use crate::harness::UiGalleryCodeEditorHandlesStore;
use crate::spec::*;
use crate::ui;

mod app_bootstrap;
mod chrome;
mod debug_hud;
mod debug_stats;
mod demo_assets;
mod diag_snapshot;
mod inspector;
mod menu_runtime;
mod menubar;
mod render_flow;
mod router;
mod settings_sheet;
mod shell;
mod status_bar;
mod theme_runtime;
mod toaster;
mod window_bootstrap;
mod workspace_nav;
use router::{
    UiGalleryHistory, UiGalleryRouteId, apply_page_route_side_effects_via_router,
    apply_page_router_update_side_effects, build_ui_gallery_page_router,
    page_from_gallery_location,
};

#[cfg(target_os = "windows")]
fn ui_gallery_windows_common_fallback_override_disabled() -> bool {
    std::env::var("FRET_UI_GALLERY_DISABLE_WINDOWS_COMMON_FALLBACK_OVERRIDE")
        .ok()
        .is_some_and(|value| {
            let value = value.trim();
            !value.is_empty() && value != "0"
        })
}

fn apply_ui_gallery_text_font_fallback_overrides(config: &mut fret_render::TextFontFamilyConfig) {
    #[cfg(target_os = "windows")]
    {
        if ui_gallery_windows_common_fallback_override_disabled() {
            return;
        }

        if config.common_fallback_injection
            == fret_core::TextCommonFallbackInjection::PlatformDefault
        {
            // UI gallery demos intentionally include symbol glyphs (e.g. "⌘") to align with shadcn
            // docs. On Windows, relying on system fallback can yield tofu squares. Prefer injecting
            // the curated common fallback stack so missing glyphs can resolve via "Segoe UI Symbol"
            // / "Segoe UI Emoji" when available. Diagnostics can opt out to exercise the true
            // platform-default/system-fallback lane.
            config.common_fallback_injection =
                fret_core::TextCommonFallbackInjection::CommonFallback;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = config;
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiGalleryFileDialogKind {
    LoadFonts,
    InputPicture,
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
const UI_GALLERY_WORKSPACE_WINDOW_LAYOUT_ID: &str = "ui-gallery-window";
const UI_GALLERY_WORKSPACE_PANE_ID: &str = "ui-gallery-main-pane";

#[derive(Clone)]
struct UiGalleryHarnessModelIds {
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    nav_query: Model<String>,
    settings_menu_bar_os: Model<Option<Arc<str>>>,
    settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    chrome_show_workspace_tab_strip: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    input_file_value: Model<String>,
    #[cfg(feature = "gallery-dev")]
    code_editor_syntax_rust: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_boundary_identifier: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_soft_wrap: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_folds: Model<bool>,
    #[cfg(feature = "gallery-dev")]
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
    pending_file_dialog: Option<UiGalleryFileDialogKind>,
    page_router: Router<UiGalleryRouteId, UiGalleryHistory>,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: Model<Vec<Arc<str>>>,
    workspace_window_layout: Model<WorkspaceWindowLayout>,
    workspace_tab_close_by_command: HashMap<Arc<str>, Arc<str>>,
    nav_query: Model<String>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    applied_theme_preset: Option<Arc<str>>,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
    applied_motion_preset: Option<Arc<str>>,
    applied_motion_preset_theme_preset: Option<Arc<str>>,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_cache_content: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    inspector_enabled: Model<bool>,
    inspector_last_pointer: Model<Option<fret_core::Point>>,
    #[cfg(feature = "gallery-dev")]
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    dialog_glass_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    alert_dialog_open: Model<bool>,
    #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
    sheet_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    portal_geometry_popover_open: Model<bool>,

    settings_open: Model<bool>,
    settings_menu_bar_os: Model<Option<Arc<str>>>,
    settings_menu_bar_os_open: Model<bool>,
    settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    settings_menu_bar_in_window_open: Model<bool>,
    settings_edit_can_undo: Model<bool>,
    settings_edit_can_redo: Model<bool>,
    chrome_show_workspace_tab_strip: Model<bool>,
    undo_doc: DocumentId,

    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    #[cfg(feature = "gallery-dev")]
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
    #[cfg(feature = "gallery-dev")]
    checkbox: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    switch: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_syntax_rust: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_boundary_identifier: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_soft_wrap: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_folds: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    code_editor_inlays: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    markdown_link_gate_last_activation: Model<Option<Arc<str>>>,
    #[cfg(feature = "gallery-material3")]
    material3_checkbox: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_switch: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_slider_value: Model<f32>,
    #[cfg(feature = "gallery-material3")]
    material3_radio_value: Model<Option<Arc<str>>>,
    #[cfg(feature = "gallery-material3")]
    material3_expressive: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_text_field_disabled: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_text_field_error: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_autocomplete_disabled: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_autocomplete_error: Model<bool>,
    #[cfg(feature = "gallery-material3")]
    material3_menu_open: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    input_file_value: Model<String>,
    #[cfg(feature = "gallery-dev")]
    dropdown_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    context_menu_open: Model<bool>,
    #[cfg(feature = "gallery-dev")]
    context_menu_edge_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
    menu_bar_seq: Model<u64>,
    #[cfg(feature = "gallery-dev")]
    virtual_list_torture_jump: Model<String>,
    #[cfg(feature = "gallery-dev")]
    virtual_list_torture_edit_row: Model<Option<u64>>,
    #[cfg(feature = "gallery-dev")]
    virtual_list_torture_edit_text: Model<String>,
    #[cfg(feature = "gallery-dev")]
    virtual_list_torture_scroll: VirtualListScrollHandle,
    last_config_files_status_seq: u64,
}

impl UiGalleryWindowState {
    fn content_models(&self) -> ui::UiGalleryModels {
        ui::UiGalleryModels {
            theme_preset: self.theme_preset.clone(),
            theme_preset_open: self.theme_preset_open.clone(),
            motion_preset: self.motion_preset.clone(),
            motion_preset_open: self.motion_preset_open.clone(),
            view_cache_enabled: self.view_cache_enabled.clone(),
            view_cache_cache_shell: self.view_cache_cache_shell.clone(),
            view_cache_cache_content: self.view_cache_cache_content.clone(),
            view_cache_inner_enabled: self.view_cache_inner_enabled.clone(),
            view_cache_popover_open: self.view_cache_popover_open.clone(),
            view_cache_continuous: self.view_cache_continuous.clone(),
            view_cache_counter: self.view_cache_counter.clone(),
            #[cfg(feature = "gallery-dev")]
            popover_open: self.popover_open.clone(),
            dialog_open: self.dialog_open.clone(),
            #[cfg(feature = "gallery-dev")]
            dialog_glass_open: self.dialog_glass_open.clone(),
            #[cfg(feature = "gallery-dev")]
            alert_dialog_open: self.alert_dialog_open.clone(),
            #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
            sheet_open: self.sheet_open.clone(),
            #[cfg(feature = "gallery-dev")]
            portal_geometry_popover_open: self.portal_geometry_popover_open.clone(),
            combobox_value: self.combobox_value.clone(),
            combobox_open: self.combobox_open.clone(),
            combobox_query: self.combobox_query.clone(),
            date_picker_open: self.date_picker_open.clone(),
            date_picker_month: self.date_picker_month.clone(),
            date_picker_selected: self.date_picker_selected.clone(),
            data_table_state: self.data_table_state.clone(),
            #[cfg(feature = "gallery-dev")]
            data_grid_selected_row: self.data_grid_selected_row.clone(),
            tabs_value: self.tabs_value.clone(),
            accordion_value: self.accordion_value.clone(),
            avatar_demo_image: self.avatar_demo_image.clone(),
            image_fit_demo_wide_image: self.image_fit_demo_wide_image.clone(),
            image_fit_demo_tall_image: self.image_fit_demo_tall_image.clone(),
            image_fit_demo_streaming_image: self.image_fit_demo_streaming_image.clone(),
            progress: self.progress.clone(),
            #[cfg(feature = "gallery-dev")]
            checkbox: self.checkbox.clone(),
            #[cfg(feature = "gallery-dev")]
            switch: self.switch.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_checkbox: self.material3_checkbox.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_switch: self.material3_switch.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_slider_value: self.material3_slider_value.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_radio_value: self.material3_radio_value.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_expressive: self.material3_expressive.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_text_field_disabled: self.material3_text_field_disabled.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_text_field_error: self.material3_text_field_error.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_autocomplete_disabled: self.material3_autocomplete_disabled.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_autocomplete_error: self.material3_autocomplete_error.clone(),
            #[cfg(feature = "gallery-material3")]
            material3_menu_open: self.material3_menu_open.clone(),
            text_input: self.text_input.clone(),
            text_area: self.text_area.clone(),
            input_file_value: self.input_file_value.clone(),
            #[cfg(feature = "gallery-dev")]
            dropdown_open: self.dropdown_open.clone(),
            #[cfg(feature = "gallery-dev")]
            context_menu_open: self.context_menu_open.clone(),
            #[cfg(feature = "gallery-dev")]
            context_menu_edge_open: self.context_menu_edge_open.clone(),
            cmdk_open: self.cmdk_open.clone(),
            cmdk_query: self.cmdk_query.clone(),
            last_action: self.last_action.clone(),
            sonner_position: self.sonner_position.clone(),
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_jump: self.virtual_list_torture_jump.clone(),
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_edit_row: self.virtual_list_torture_edit_row.clone(),
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_edit_text: self.virtual_list_torture_edit_text.clone(),
            #[cfg(feature = "gallery-dev")]
            virtual_list_torture_scroll: self.virtual_list_torture_scroll.clone(),
            #[cfg(feature = "gallery-dev")]
            code_editor_syntax_rust: self.code_editor_syntax_rust.clone(),
            #[cfg(feature = "gallery-dev")]
            code_editor_boundary_identifier: self.code_editor_boundary_identifier.clone(),
            #[cfg(feature = "gallery-dev")]
            code_editor_soft_wrap: self.code_editor_soft_wrap.clone(),
            #[cfg(feature = "gallery-dev")]
            code_editor_folds: self.code_editor_folds.clone(),
            #[cfg(feature = "gallery-dev")]
            code_editor_inlays: self.code_editor_inlays.clone(),
            #[cfg(feature = "gallery-dev")]
            markdown_link_gate_last_activation: self.markdown_link_gate_last_activation.clone(),
        }
    }
}

#[derive(Default)]
struct UiGalleryDriver;

impl UiGalleryDriver {
    fn sync_undo_availability(app: &mut App, window: AppWindowId, doc: &DocumentId) {
        let mut edit_can_undo = false;
        let mut edit_can_redo = false;

        app.with_global_mut_untracked(
            || UndoService::<ValueTx<f32>>::with_limit(256),
            |undo_svc, _app| {
                undo_svc.set_active_document(window, doc.clone());
                if let Some(history) = undo_svc.history_mut_active(window) {
                    edit_can_undo = history.can_undo();
                    edit_can_redo = history.can_redo();
                }
            },
        );

        #[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
        {
            if let Some(handle) = app
                .global::<UiGalleryCodeEditorHandlesStore>()
                .and_then(|store| store.per_window.get(&window).cloned())
            {
                edit_can_undo |= handle.can_undo();
                edit_can_redo |= handle.can_redo();
            }
        }

        app.with_global_mut_untracked(WindowCommandAvailabilityService::default, |svc, _app| {
            svc.set_edit_availability(window, edit_can_undo, edit_can_redo);
        });
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
                state.pending_file_dialog = Some(UiGalleryFileDialogKind::LoadFonts);
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
            CMD_INPUT_PICTURE_BROWSE => {
                let diag_mode = std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty());

                if diag_mode {
                    let _ = app.models_mut().update(&state.input_file_value, |v| {
                        *v = String::from("avatar.png");
                    });
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("input.picture.browse.mocked");
                    });
                    app.request_redraw(window);
                    return true;
                }

                state.pending_file_dialog = Some(UiGalleryFileDialogKind::InputPicture);
                app.push_effect(Effect::FileDialogOpen {
                    window,
                    options: FileDialogOptions {
                        title: Some("Choose picture".to_string()),
                        multiple: false,
                        filters: vec![FileDialogFilter {
                            name: "Images".to_string(),
                            extensions: vec![
                                "png".to_string(),
                                "jpg".to_string(),
                                "jpeg".to_string(),
                                "gif".to_string(),
                                "webp".to_string(),
                            ],
                        }],
                    },
                });

                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("input.picture.browse");
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

#[cfg(target_arch = "wasm32")]
fn bool_from_window_query(key: &str) -> Option<bool> {
    let Some(window) = web_sys::window() else {
        return None;
    };

    let location = window.location();
    let search = location.search().unwrap_or_default();
    let hash = location.hash().unwrap_or_default();

    fn parse_query_params(query: &str) -> Option<web_sys::UrlSearchParams> {
        let query = query.trim();
        if query.is_empty() {
            return None;
        }
        let query = query.trim_start_matches('?');
        web_sys::UrlSearchParams::new_with_str(query).ok()
    }

    fn parse_hash_query_params(hash: &str) -> Option<web_sys::UrlSearchParams> {
        let hash = hash.trim();
        if hash.is_empty() {
            return None;
        }

        let hash = hash.trim_start_matches('#');
        let query = hash.split_once('?').map(|(_, q)| q).unwrap_or(hash);
        parse_query_params(query)
    }

    fn parse_bool(v: Option<String>) -> Option<bool> {
        let v = v?;
        let v = v.trim().to_ascii_lowercase();
        if v.is_empty() {
            return Some(true);
        }
        match v.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    if let Some(params) = parse_query_params(&search) {
        if let Some(v) = parse_bool(params.get(key)) {
            return Some(v);
        }
    }

    if let Some(params) = parse_hash_query_params(&hash) {
        if let Some(v) = parse_bool(params.get(key)) {
            return Some(v);
        }
    }

    let global_key = format!("__{}", key.to_ascii_uppercase());
    if let Ok(v) = js_sys::Reflect::get(
        window.as_ref(),
        &wasm_bindgen::JsValue::from_str(&global_key),
    ) {
        if let Some(b) = v.as_bool() {
            return Some(b);
        }
        if let Some(s) = v.as_string() {
            return parse_bool(Some(s));
        }
    }

    None
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
    let mut caps = PlatformCapabilities::default();
    caps.shell.share_sheet = true;
    caps.shell.incoming_open = true;
    app.set_global(caps);
    app.set_global(UiGalleryRecentItemsService::default());
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Zinc,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    app.with_global_mut(IconRegistry::default, |icons, app| {
        fret_icons_lucide::register_icons(icons);
        let frozen = icons.freeze_or_default_with_context("fret_ui_gallery.build_app");
        app.set_global(frozen);
    });

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
    fn parse_main_window_size_override() -> Option<fret_launch::WindowLogicalSize> {
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

        Some(fret_launch::WindowLogicalSize::new(w, h))
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
        None => fret_launch::WindowLogicalSize::new(1080.0, 720.0),
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
        .configure(|c| {
            apply_ui_gallery_text_font_fallback_overrides(&mut c.text_font_families);
        })
        .with_config_files_watcher_for_root(Duration::from_millis(500), &project_root)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
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
        .configure(|c| {
            apply_ui_gallery_text_font_fallback_overrides(&mut c.text_font_families);
        })
        .with_config_files_watcher_for_root(Duration::from_millis(500), &project_root)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
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
        if wants_bootstrap_fonts {
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

        // Ensure magic ecosystem components can use renderer-controlled Tier B materials.
        #[cfg(feature = "gallery-dev")]
        fret_ui_magic::advanced::ensure_materials(app, renderer);
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
            if command.as_str().starts_with("workspace.") {
                let _ = Self::sync_workspace_models_from_window_layout(app, state, window);
            }
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

        #[cfg(feature = "gallery-dev")]
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

        #[cfg(feature = "gallery-dev")]
        if command.as_str() == CMD_VIRTUAL_LIST_TORTURE_SCROLL_BOTTOM {
            state.virtual_list_torture_scroll.scroll_to_bottom();
            app.request_redraw(window);
            return;
        }

        #[cfg(feature = "gallery-dev")]
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

        #[cfg(feature = "gallery-dev")]
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
            let consumed = fret_bootstrap::maybe_consume_event(app, window, event);
            if consumed {
                return;
            }
        }

        match event {
            Event::FileDialogSelection(selection) => match state.pending_file_dialog {
                Some(UiGalleryFileDialogKind::LoadFonts) => {
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
                Some(UiGalleryFileDialogKind::InputPicture) => {
                    let name = selection.files.first().map(|f| f.name.clone());
                    let _ = app.models_mut().update(&state.input_file_value, |v| {
                        *v = name.unwrap_or_default();
                    });
                    app.push_effect(Effect::FileDialogRelease {
                        token: selection.token,
                    });
                    state.pending_file_dialog = None;
                    app.request_redraw(window);
                }
                None => {
                    app.push_effect(Effect::FileDialogRelease {
                        token: selection.token,
                    });
                }
            },
            Event::FileDialogData(data) => {
                if state.pending_file_dialog != Some(UiGalleryFileDialogKind::LoadFonts) {
                    state.pending_file_dialog = None;
                    let mut host = UiActionHostAdapter { app };
                    host.push_effect(Effect::FileDialogRelease { token: data.token });
                    host.request_redraw(window);
                    return;
                }

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
                state.pending_file_dialog = None;
                host.request_redraw(window);
            }
            Event::FileDialogCanceled => {
                let sonner = shadcn::Sonner::global(app);
                let mut host = UiActionHostAdapter { app };
                let title = match state.pending_file_dialog.take() {
                    Some(UiGalleryFileDialogKind::LoadFonts) => "Load fonts canceled",
                    Some(UiGalleryFileDialogKind::InputPicture) => "Choose file canceled",
                    None => "File dialog canceled",
                };
                sonner.toast_message(
                    &mut host,
                    window,
                    title,
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
                            shadcn::ToastMessageOptions::new().description("Shared successfully."),
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
                if Self::sync_workspace_models_from_window_layout(app, state, window) {
                    app.request_redraw(window);
                }
            }
        }

        let should_drive_ui_assets = match event {
            Event::ImageRegistered { token, .. } | Event::ImageRegisterFailed { token, .. } => {
                use fret_ui_assets::image_asset_cache::ImageAssetCacheHostExt as _;
                app.with_image_asset_cache(|cache, _app| cache.key_for_token(*token).is_some())
            }
            _ => false,
        };
        if should_drive_ui_assets {
            let _ = fret_ui_assets::UiAssets::handle_event(app, window, event);
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

        let diag_wants_semantics_snapshot = app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, _app| svc.wants_semantics_snapshot(window),
        );
        if diag_wants_semantics_snapshot {
            // Diagnostics scripts select targets by semantics bounds. Ensure we have a fresh
            // semantics snapshot for the current frame before we drive scripted input; otherwise,
            // scripts may act on a 1-frame-stale snapshot and mis-predict visibility in
            // virtualized lists (estimate -> measured jumps).
            state.ui.request_semantics_snapshot();
        }
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
        let semantics_snapshot = state.ui.semantics_snapshot_arc();
        let (drive, wants_quit) = app.with_global_mut_untracked(
            UiDiagnosticsService::default,
            |svc: &mut UiDiagnosticsService, app| {
                let wants_quit = svc.poll_exit_trigger();
                let drive = svc.drive_script_for_window(
                    app,
                    services,
                    window,
                    bounds,
                    scale_factor,
                    Some(&mut state.ui),
                    semantics_snapshot.as_deref(),
                );
                (drive, wants_quit)
            },
        );

        if wants_quit {
            app.push_effect(Effect::QuitApp);
            return;
        }

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
        if injected_any && Self::sync_workspace_models_from_window_layout(app, state, window) {
            app.request_redraw(window);
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
                    &mut state.ui,
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
                fret_launch::WindowLogicalSize::new(980.0, 720.0),
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

    fn semantics_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        // This is the primary semantics hook used by accessibility and other runner integrations.
        // Requesting semantics here ensures we start producing snapshots on the next frame without
        // forcing semantics on every frame.
        state.ui.request_semantics_snapshot();
        state.ui.semantics_snapshot_arc()
    }
}

#[cfg(test)]
mod stack_overflow_repro_tests;
