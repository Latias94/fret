use fret_app::{
    ActivationPolicy, App, CommandId, CommandMeta, CreateWindowKind, CreateWindowRequest, Effect,
    LayeredConfigPaths, Menu, MenuBar, MenuBarIntegrationModeV1, MenuItem, MenuRole, Model,
    Platform, SettingsFileV1, WindowRequest, WindowRole, WindowStyleRequest, load_layered_settings,
};
use fret_core::{
    AlphaMode, AppWindowId, Event, ExternalDropReadLimits, FileDialogFilter, FileDialogOptions,
    ImageColorInfo, ImageId, ImageUploadToken, KeyCode, Modifiers, SemanticsRole, TimerToken,
    UiServices,
};
use fret_kit::prelude::{
    InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, menubar_from_runtime_with_focus_handle,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_query::{QueryPolicy, with_query_client};
use fret_router::{
    NamespaceInvalidationRule, NavigationAction, RouteChangePolicy, RouteHooks, RouteLocation,
    RouteNode, RouteSearchTable, RouteTree, Router, RouterUpdate, RouterUpdateWithPrefetchIntents,
    SearchValidationMode, collect_invalidated_namespaces, prefetch_intent_query_key,
};
use fret_runtime::{
    DefaultKeybinding, KeyChord, MenuItemToggle, MenuItemToggleKind, PlatformCapabilities,
    PlatformFilter, WindowCommandAvailabilityService, WindowCommandEnabledService,
};
use fret_ui::action::{UiActionHost, UiActionHostAdapter};
use fret_ui::declarative;
use fret_ui::element::{SemanticsDecoration, SemanticsProps};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{Invalidation, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use fret_undo::{CoalesceKey, DocumentId, UndoRecord, UndoService, ValueTx};
use fret_workspace::commands::{
    CMD_WORKSPACE_TAB_CLOSE, CMD_WORKSPACE_TAB_CLOSE_PREFIX, CMD_WORKSPACE_TAB_NEXT,
    CMD_WORKSPACE_TAB_PREV,
};
use fret_workspace::{
    WorkspaceFrame, WorkspaceStatusBar, WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::Date;

use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;

#[cfg(not(target_arch = "wasm32"))]
use crate::harness::{
    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER, UiGalleryCodeEditorHandlesStore,
    UiGalleryMarkdownEditorHandlesStore,
};
use crate::spec::*;
use crate::ui;

const UI_GALLERY_PAGE_CONTENT_NS: &str = "fret.ui_gallery.page_content.v1";
const UI_GALLERY_NAV_INDEX_NS: &str = "fret.ui_gallery.nav_index.v1";

#[cfg(target_arch = "wasm32")]
type UiGalleryHistory = fret_router::WebHistoryAdapter;

#[cfg(not(target_arch = "wasm32"))]
type UiGalleryHistory = fret_router::MemoryHistory;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum UiGalleryRouteId {
    Root,
    Gallery,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct UiGalleryPagePrefetchSeed {
    selected_page: Arc<str>,
}

fn route_location_for_page(from: &RouteLocation, page: &Arc<str>) -> RouteLocation {
    let location = RouteLocation::from_path("/gallery")
        .with_query_value("page", Some(page.to_string()))
        .with_query_value("source", Some("nav".to_string()));

    if let Some(demo) = from.query_value("demo") {
        location.with_query_value("demo", Some(demo.to_string()))
    } else {
        location
    }
}

fn page_from_gallery_location(location: &RouteLocation) -> Option<Arc<str>> {
    let page = location.query_value("page")?;
    page_spec(page).is_some().then_some(Arc::<str>::from(page))
}

fn build_ui_gallery_page_router() -> Router<UiGalleryRouteId, UiGalleryHistory> {
    let tree = Arc::new(RouteTree::new(
        RouteNode::new(UiGalleryRouteId::Root, "/")
            .expect("root route should build")
            .with_children(vec![
                RouteNode::new(UiGalleryRouteId::Gallery, "gallery")
                    .expect("gallery route should build"),
            ]),
    ));

    let search_table = Arc::new(RouteSearchTable::new());

    #[cfg(target_arch = "wasm32")]
    let history = UiGalleryHistory::new().expect("web history adapter should resolve a location");

    #[cfg(not(target_arch = "wasm32"))]
    let history = UiGalleryHistory::new(RouteLocation::parse("/"));

    let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
        .expect("ui gallery router should build");

    router.route_hooks_mut().insert(
        UiGalleryRouteId::Gallery,
        RouteHooks {
            before_load: None,
            loader: Some(Arc::new(|ctx| {
                vec![fret_router::RoutePrefetchIntent {
                    route: ctx.matched.route,
                    namespace: UI_GALLERY_PAGE_CONTENT_NS,
                    location: ctx.to.clone(),
                    extra: None,
                }]
            })),
        },
    );

    router
}

fn apply_page_route_side_effects_via_router(
    app: &mut App,
    window: AppWindowId,
    action: NavigationAction,
    current_page: Arc<str>,
    router: &mut Router<UiGalleryRouteId, UiGalleryHistory>,
) {
    let current_route = route_location_for_page(&router.state().location, &current_page);
    let update = router.navigate_with_prefetch_intents(action, Some(current_route.canonicalized()));
    apply_page_router_update_side_effects(app, window, current_page, router, update);
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_gallery_page_history_command_enabled(
    app: &mut App,
    window: AppWindowId,
    history: &UiGalleryHistory,
) {
    let can_back = history.can_back();
    let can_forward = history.can_forward();

    let cmd_back = CommandId::new(CMD_GALLERY_PAGE_BACK);
    let cmd_forward = CommandId::new(CMD_GALLERY_PAGE_FORWARD);

    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        if can_back {
            svc.clear_command(window, &cmd_back);
        } else {
            svc.set_enabled(window, cmd_back.clone(), false);
        }

        if can_forward {
            svc.clear_command(window, &cmd_forward);
        } else {
            svc.set_enabled(window, cmd_forward.clone(), false);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn sync_gallery_page_history_command_enabled(
    app: &mut App,
    window: AppWindowId,
    _history: &UiGalleryHistory,
) {
    let cmd_back = CommandId::new(CMD_GALLERY_PAGE_BACK);
    let cmd_forward = CommandId::new(CMD_GALLERY_PAGE_FORWARD);

    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.clear_command(window, &cmd_back);
        svc.clear_command(window, &cmd_forward);
    });
}

fn apply_page_router_update_side_effects(
    app: &mut App,
    window: AppWindowId,
    current_page: Arc<str>,
    router: &mut Router<UiGalleryRouteId, UiGalleryHistory>,
    update: Result<
        RouterUpdateWithPrefetchIntents<UiGalleryRouteId>,
        fret_router::RouteSearchValidationFailure,
    >,
) {
    sync_gallery_page_history_command_enabled(app, window, router.history());

    let Ok(update) = update else {
        return;
    };

    let RouterUpdateWithPrefetchIntents { update, intents } = update;

    let invalidated = if let RouterUpdate::Changed(transition) = &update {
        collect_invalidated_namespaces(
            &transition.from,
            &transition.to,
            &[
                NamespaceInvalidationRule::new(
                    UI_GALLERY_PAGE_CONTENT_NS,
                    RouteChangePolicy::PathOrQueryChanged,
                ),
                NamespaceInvalidationRule::new(
                    UI_GALLERY_NAV_INDEX_NS,
                    RouteChangePolicy::QueryChanged,
                ),
            ],
        )
    } else {
        Vec::new()
    };

    if invalidated.is_empty() && intents.is_empty() {
        return;
    }

    let _ = with_query_client(app, |client, app| {
        for namespace in invalidated {
            client.invalidate_namespace(namespace);
        }

        for intent in intents {
            if intent.namespace != UI_GALLERY_PAGE_CONTENT_NS {
                continue;
            }

            let seed = UiGalleryPagePrefetchSeed {
                selected_page: current_page.clone(),
            };
            let key = prefetch_intent_query_key::<String, _>(&intent);
            let policy = QueryPolicy::default();
            let _ = client.prefetch(app, window, key, policy, move |_token| {
                Ok::<String, fret_query::QueryError>(format!(
                    "ui_gallery.page_prefetch:{}",
                    seed.selected_page
                ))
            });
        }
    });
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

#[derive(Default)]
struct UiGalleryDriver;

impl UiGalleryDriver {
    const AVATAR_DEMO_IMAGE_WIDTH: u32 = 96;
    const AVATAR_DEMO_IMAGE_HEIGHT: u32 = 96;
    const AVATAR_DEMO_IMAGE_RETRY_MAX: u8 = 8;

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

    fn build_workspace_menu_commands(app: &App) -> fret_workspace::menu::WorkspaceMenuCommands {
        let mut cmds = fret_workspace::menu::WorkspaceMenuCommands::default();
        cmds.open = Some(CommandId::new(CMD_APP_OPEN));
        cmds.save = Some(CommandId::new(CMD_APP_SAVE));
        cmds.undo = Some(CommandId::new(fret_app::core_commands::EDIT_UNDO));
        cmds.redo = Some(CommandId::new(fret_app::core_commands::EDIT_REDO));
        cmds.cut = Some(CommandId::new(fret_app::core_commands::EDIT_CUT));
        cmds.copy = Some(CommandId::new(fret_app::core_commands::EDIT_COPY));
        cmds.paste = Some(CommandId::new(fret_app::core_commands::EDIT_PASTE));
        cmds.select_all = Some(CommandId::new(fret_app::core_commands::EDIT_SELECT_ALL));
        cmds.command_palette = Some(CommandId::new(fret_app::core_commands::COMMAND_PALETTE));
        cmds.switch_locale = Some(CommandId::new(
            fret_app::core_commands::APP_LOCALE_SWITCH_NEXT,
        ));

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
        cmds.file_menu_title = Some(resolve_menu_title("workspace-menu-file", "File"));
        cmds.edit_menu_title = Some(resolve_menu_title("workspace-menu-edit", "Edit"));
        cmds.view_menu_title = Some(resolve_menu_title("workspace-menu-view", "View"));
        cmds.window_menu_title = Some(resolve_menu_title("workspace-menu-window", "Window"));

        if Platform::current() == Platform::Macos {
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
        {
            if let Some(MenuItem::Submenu {
                title: _, items, ..
            }) = menu.items.iter_mut().find(
                |i| matches!(i, MenuItem::Submenu { title, .. } if title.as_ref() == "Recent"),
            ) {
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
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let windows = Self::window_menu_items(app);
            let focused_window = Self::focused_window_menu_item(app);
            if !windows.is_empty() {
                if let Some(menu) = menu_bar
                    .menus
                    .iter_mut()
                    .find(|m| m.role == Some(MenuRole::Window) || m.title.as_ref() == "Window")
                {
                    if let Some(MenuItem::Submenu { title: _, items, .. }) =
                        menu.items.iter_mut().find(|i| {
                            matches!(i, MenuItem::Submenu { title, .. } if title.as_ref() == "Windows")
                        })
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

        let _ = app.with_global_mut(
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

    fn compute_inspector_status(
        app: &mut App,
        ui: &UiTree<App>,
        window: AppWindowId,
        pointer: Option<fret_core::Point>,
    ) -> (Arc<str>, Arc<str>, Arc<str>, Arc<str>) {
        let hit = pointer.map(|p| ui.debug_hit_test(p));
        let hit_node = hit.as_ref().and_then(|h| h.hit);
        let hit_layers = hit
            .as_ref()
            .map(|h| h.active_layer_roots.len())
            .unwrap_or(0);
        let hit_barrier = hit.as_ref().and_then(|h| h.barrier_root);

        let (focused_node, focused_element, hovered_pressable, pressed_pressable) = app
            .with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                let state = runtime.diagnostics_snapshot(window);
                (
                    ui.focus(),
                    state.as_ref().and_then(|s| s.focused_element),
                    state.as_ref().and_then(|s| s.hovered_pressable),
                    state.as_ref().and_then(|s| s.pressed_pressable),
                )
            });

        let hit_element = hit_node.and_then(|node| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.element_for_node(window, node)
            })
        });

        let hit_path = hit_element.and_then(|element| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.debug_path_for_element(window, element)
            })
        });
        let focused_path = focused_element.and_then(|element| {
            app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                runtime.debug_path_for_element(window, element)
            })
        });

        let cursor = if let Some(pos) = pointer {
            Arc::<str>::from(format!("cursor=({:.1},{:.1})", pos.x.0, pos.y.0))
        } else {
            Arc::<str>::from("cursor=<none>")
        };

        let hit = Arc::<str>::from(format!(
            "hit={:?} el={} layers={} barrier={:?} {}",
            hit_node,
            hit_element
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            hit_layers,
            hit_barrier,
            hit_path.as_deref().unwrap_or(""),
        ));

        let focus = Arc::<str>::from(format!(
            "focus={:?} el={} hovered={} pressed={} {}",
            focused_node,
            focused_element
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            hovered_pressable
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            pressed_pressable
                .map(|id| format!("{:#x}", id.0))
                .unwrap_or_else(|| "<none>".to_string()),
            focused_path.as_deref().unwrap_or(""),
        ));

        let text = if let Some(node) = hit_node {
            let bounds = ui.debug_node_bounds(node);
            let constraints = ui.debug_text_constraints_snapshot(node);
            Arc::<str>::from(format!(
                "text node={:?} bounds={bounds:?} measured={:?} prepared={:?}",
                node, constraints.measured, constraints.prepared,
            ))
        } else {
            Arc::<str>::from("text node=<none>")
        };

        (cursor, hit, focus, text)
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

        let avatar_demo_image = app.models_mut().insert(None::<ImageId>);
        let avatar_demo_image_token = app.next_image_upload_token();
        Self::enqueue_avatar_demo_image_register(app, window, avatar_demo_image_token);

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
                let _ = app.with_global_mut(
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
                let _ = app.with_global_mut(
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
                let _ = app.with_global_mut(
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
        OverlayController::begin_frame(app, window);
        let bisect = ui_gallery_bisect_flags();

        Self::sync_undo_availability(app, window, &state.undo_doc);

        #[cfg(target_arch = "wasm32")]
        Self::sync_page_router_from_external_history(app, window, state);

        let availability = app
            .global::<WindowCommandAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .copied()
            .unwrap_or_default();
        let _ = app.models_mut().update(&state.settings_edit_can_undo, |v| {
            *v = availability.edit_can_undo
        });
        let _ = app.models_mut().update(&state.settings_edit_can_redo, |v| {
            *v = availability.edit_can_redo
        });

        let cache_enabled = app
            .models()
            .get_copied(&state.view_cache_enabled)
            .unwrap_or(false);
        let cache_shell = app
            .models()
            .get_copied(&state.view_cache_cache_shell)
            .unwrap_or(false);

        if state.ui.view_cache_enabled() != cache_enabled {
            state.ui.set_view_cache_enabled(cache_enabled);
            if let Some(root) = state.root {
                state.ui.invalidate(root, Invalidation::Layout);
            }
        }

        let selected_page = state.selected_page.clone();
        let workspace_tabs = state.workspace_tabs.clone();
        let workspace_dirty_tabs = state.workspace_dirty_tabs.clone();
        let nav_query = state.nav_query.clone();
        let content_tab = state.content_tab.clone();
        let theme_preset = state.theme_preset.clone();
        let theme_preset_open = state.theme_preset_open.clone();
        let view_cache_enabled = state.view_cache_enabled.clone();
        let view_cache_cache_shell = state.view_cache_cache_shell.clone();
        let view_cache_inner_enabled = state.view_cache_inner_enabled.clone();
        let view_cache_popover_open = state.view_cache_popover_open.clone();
        let view_cache_continuous = state.view_cache_continuous.clone();
        let view_cache_counter = state.view_cache_counter.clone();
        let popover_open = state.popover_open.clone();
        let dialog_open = state.dialog_open.clone();
        let alert_dialog_open = state.alert_dialog_open.clone();
        let sheet_open = state.sheet_open.clone();
        let portal_geometry_popover_open = state.portal_geometry_popover_open.clone();
        let settings_open = state.settings_open.clone();
        let settings_menu_bar_os = state.settings_menu_bar_os.clone();
        let settings_menu_bar_os_open = state.settings_menu_bar_os_open.clone();
        let settings_menu_bar_in_window = state.settings_menu_bar_in_window.clone();
        let settings_menu_bar_in_window_open = state.settings_menu_bar_in_window_open.clone();
        let settings_edit_can_undo = state.settings_edit_can_undo.clone();
        let settings_edit_can_redo = state.settings_edit_can_redo.clone();
        let select_value = state.select_value.clone();
        let select_open = state.select_open.clone();
        let combobox_value = state.combobox_value.clone();
        let combobox_open = state.combobox_open.clone();
        let combobox_query = state.combobox_query.clone();
        let date_picker_open = state.date_picker_open.clone();
        let date_picker_month = state.date_picker_month.clone();
        let date_picker_selected = state.date_picker_selected.clone();
        let time_picker_open = state.time_picker_open.clone();
        let time_picker_selected = state.time_picker_selected.clone();
        let resizable_h_fractions = state.resizable_h_fractions.clone();
        let resizable_v_fractions = state.resizable_v_fractions.clone();
        let data_table_state = state.data_table_state.clone();
        let data_grid_selected_row = state.data_grid_selected_row.clone();
        let tabs_value = state.tabs_value.clone();
        let accordion_value = state.accordion_value.clone();
        let avatar_demo_image = state.avatar_demo_image.clone();
        let progress = state.progress.clone();
        let checkbox = state.checkbox.clone();
        let switch = state.switch.clone();
        let code_editor_syntax_rust = state.code_editor_syntax_rust.clone();
        let code_editor_boundary_identifier = state.code_editor_boundary_identifier.clone();
        let code_editor_soft_wrap = state.code_editor_soft_wrap.clone();
        let code_editor_folds = state.code_editor_folds.clone();
        let code_editor_inlays = state.code_editor_inlays.clone();
        let material3_checkbox = state.material3_checkbox.clone();
        let material3_switch = state.material3_switch.clone();
        let material3_radio_value = state.material3_radio_value.clone();
        let material3_tabs_value = state.material3_tabs_value.clone();
        let material3_list_value = state.material3_list_value.clone();
        let material3_expressive = state.material3_expressive.clone();
        let material3_navigation_bar_value = state.material3_navigation_bar_value.clone();
        let material3_navigation_rail_value = state.material3_navigation_rail_value.clone();
        let material3_navigation_drawer_value = state.material3_navigation_drawer_value.clone();
        let material3_modal_navigation_drawer_open =
            state.material3_modal_navigation_drawer_open.clone();
        let material3_dialog_open = state.material3_dialog_open.clone();
        let material3_text_field_value = state.material3_text_field_value.clone();
        let material3_text_field_disabled = state.material3_text_field_disabled.clone();
        let material3_text_field_error = state.material3_text_field_error.clone();
        let material3_autocomplete_value = state.material3_autocomplete_value.clone();
        let material3_autocomplete_disabled = state.material3_autocomplete_disabled.clone();
        let material3_autocomplete_error = state.material3_autocomplete_error.clone();
        let material3_autocomplete_dialog_open = state.material3_autocomplete_dialog_open.clone();
        let material3_menu_open = state.material3_menu_open.clone();
        let text_input = state.text_input.clone();
        let text_area = state.text_area.clone();
        let dropdown_open = state.dropdown_open.clone();
        let context_menu_open = state.context_menu_open.clone();
        let context_menu_edge_open = state.context_menu_edge_open.clone();
        let cmdk_open = state.cmdk_open.clone();
        let cmdk_query = state.cmdk_query.clone();
        let last_action = state.last_action.clone();
        let sonner_position = state.sonner_position.clone();
        let menu_bar_seq = state.menu_bar_seq.clone();
        let virtual_list_torture_jump = state.virtual_list_torture_jump.clone();
        let virtual_list_torture_edit_row = state.virtual_list_torture_edit_row.clone();
        let virtual_list_torture_edit_text = state.virtual_list_torture_edit_text.clone();
        let virtual_list_torture_scroll = state.virtual_list_torture_scroll.clone();
        let inspector_enabled = state.inspector_enabled.clone();
        let inspector_last_pointer = state.inspector_last_pointer.clone();

        let inspector_on = app.models().get_copied(&inspector_enabled).unwrap_or(false);
        let debug_on = inspector_on
            || std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty());
        state.ui.set_debug_enabled(debug_on);
        if debug_on {
            app.request_redraw(window);
        }

        Self::sync_shadcn_theme(app, state);

        let last_debug_stats = state.ui.debug_stats();
        let frame_dt = if debug_on {
            state.debug_hud.tick(fret_core::time::Instant::now())
        } else {
            None
        };
        let fps = state.debug_hud.ema_fps();
        let last_cache_roots = state.ui.debug_cache_root_stats();
        let cache_root_breakdown: Option<Vec<Arc<str>>> = if !last_cache_roots.is_empty() {
            let total = last_cache_roots.len();
            let hits = last_cache_roots.iter().filter(|r| r.reused).count();
            let replayed_ops: u32 = last_cache_roots.iter().map(|r| r.paint_replayed_ops).sum();

            let mut lines: Vec<Arc<str>> = vec![Arc::from(format!(
                "cache_roots total={total} hits={hits} replayed_ops={replayed_ops}"
            ))];

            let max_items = 3usize;
            for (index, root) in last_cache_roots.iter().take(max_items).enumerate() {
                let element_path = root.element.and_then(|element| {
                    app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                        runtime.debug_path_for_element(window, element)
                    })
                });

                lines.push(Arc::from(format!(
                    "cache_root[{index}] node={:?} reused={} contained_layout={} replayed_ops={} el={} {}",
                    root.root,
                    root.reused as u8,
                    root.contained_layout as u8,
                    root.paint_replayed_ops,
                    root.element
                        .map(|id| format!("{:#x}", id.0))
                        .unwrap_or_else(|| "<none>".to_string()),
                    element_path.as_deref().unwrap_or(""),
                )));
            }

            Some(lines)
        } else {
            None
        };
        let hot_model_breakdown: Option<Arc<str>> = {
            let hotspots = state.ui.debug_model_change_hotspots();
            if hotspots.is_empty() {
                None
            } else {
                let mut line = String::from("hot_models");
                for hs in hotspots.iter().take(3) {
                    line.push(' ');
                    line.push_str(&format!("{:?}={}", hs.model, hs.observation_edges));
                }
                Some(Arc::from(line))
            }
        };
        let unobserved_model_breakdown: Option<Arc<str>> = {
            let unobserved = state.ui.debug_model_change_unobserved();
            if unobserved.is_empty() {
                None
            } else {
                let mut line = format!(
                    "unobs_models={}",
                    state.ui.debug_stats().model_change_unobserved_models
                );
                for entry in unobserved.iter().take(3) {
                    let type_name = entry.created.map(|c| c.type_name).unwrap_or("<unknown>");
                    let type_name = type_name.rsplit("::").next().unwrap_or(type_name);
                    line.push(' ');
                    line.push_str(&format!("{:?}={}", entry.model, type_name));
                }
                Some(Arc::from(line))
            }
        };

        let show_debug_hud = debug_on;
        let mut debug_hud_lines: Vec<Arc<str>> = if show_debug_hud {
            let mut lines: Vec<Arc<str>> = Vec::new();

            lines.push(Arc::from(format!(
                "fps={:.1} frame_dt_ms={:.2} solve_us={}",
                fps.unwrap_or(0.0),
                frame_dt.map(|dt| dt.as_secs_f64() * 1000.0).unwrap_or(0.0),
                last_debug_stats.layout_engine_solve_time.as_micros()
            )));
            lines.push(Arc::from(format!(
                "frame={:?} layout_us={} paint_us={} layout_nodes={}/{} paint_nodes={}/{}",
                last_debug_stats.frame_id,
                last_debug_stats.layout_time.as_micros(),
                last_debug_stats.paint_time.as_micros(),
                last_debug_stats.layout_nodes_performed,
                last_debug_stats.layout_nodes_visited,
                last_debug_stats.paint_nodes_performed,
                last_debug_stats.paint_nodes,
            )));
            lines.push(Arc::from(format!(
                "paint_cache hits={} misses={} replayed_ops={}",
                last_debug_stats.paint_cache_hits,
                last_debug_stats.paint_cache_misses,
                last_debug_stats.paint_cache_replayed_ops
            )));
            lines.push(Arc::from(format!(
                "view_cache active={} trunc={} relayouts={}",
                last_debug_stats.view_cache_active as u8,
                last_debug_stats.view_cache_invalidation_truncations,
                last_debug_stats.view_cache_contained_relayouts
            )));
            lines.push(Arc::from(format!(
                "changes models={} edges={} roots={} walks={} nodes={}",
                last_debug_stats.model_change_models,
                last_debug_stats.model_change_observation_edges,
                last_debug_stats.model_change_invalidation_roots,
                last_debug_stats.invalidation_walk_calls_model_change,
                last_debug_stats.invalidation_walk_nodes_model_change
            )));
            lines.push(Arc::from(format!(
                "globals count={} edges={} roots={} walks={} nodes={}",
                last_debug_stats.global_change_globals,
                last_debug_stats.global_change_observation_edges,
                last_debug_stats.global_change_invalidation_roots,
                last_debug_stats.invalidation_walk_calls_global_change,
                last_debug_stats.invalidation_walk_nodes_global_change
            )));
            lines.push(Arc::from(format!(
                "hover edges pressable={} region={} decl inst={} hit={} layout={} paint={}",
                last_debug_stats.hover_pressable_target_changes,
                last_debug_stats.hover_hover_region_target_changes,
                last_debug_stats.hover_declarative_instance_changes,
                last_debug_stats.hover_declarative_hit_test_invalidations,
                last_debug_stats.hover_declarative_layout_invalidations,
                last_debug_stats.hover_declarative_paint_invalidations,
            )));

            let hover_hotspots = state.ui.debug_hover_declarative_invalidation_hotspots(3);
            for (index, hs) in hover_hotspots.iter().enumerate() {
                let element_path = hs.element.and_then(|element| {
                    app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                        runtime.debug_path_for_element(window, element)
                    })
                });

                lines.push(Arc::from(format!(
                    "hover_decl[{index}] node={:?} hit={} layout={} paint={} el={} {}",
                    hs.node,
                    hs.hit_test,
                    hs.layout,
                    hs.paint,
                    hs.element
                        .map(|id| format!("{:#x}", id.0))
                        .unwrap_or_else(|| "<none>".to_string()),
                    element_path.as_deref().unwrap_or(""),
                )));
            }

            if let Some(extra) = cache_root_breakdown.as_ref() {
                lines.extend(extra.iter().cloned());
            }
            if let Some(line) = hot_model_breakdown.as_ref() {
                lines.push(line.clone());
            }
            if let Some(line) = unobserved_model_breakdown.as_ref() {
                lines.push(line.clone());
            }

            lines
        } else {
            Vec::new()
        };
        let inspector_status = if app.models().get_copied(&inspector_enabled).unwrap_or(false) {
            let pointer = app
                .models()
                .get_copied(&inspector_last_pointer)
                .unwrap_or(None);
            Some(Self::compute_inspector_status(
                app, &state.ui, window, pointer,
            ))
        } else {
            None
        };
        if show_debug_hud && let Some((cursor, hit, focus, text)) = inspector_status.as_ref() {
            debug_hud_lines.push(Arc::from("--- inspector ---"));
            debug_hud_lines.push(cursor.clone());
            debug_hud_lines.push(hit.clone());
            debug_hud_lines.push(focus.clone());
            debug_hud_lines.push(text.clone());
        }

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("fret-ui-gallery", |cx| {
                    if (bisect & BISECT_MINIMAL_ROOT) != 0 {
                        return vec![cx.text("Hello, fret-ui-gallery")];
                    }

                    let theme = cx.theme().clone();

                    let sidebar = if cache_shell {
                        cx.view_cache(
                            {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Px(Px(280.0));
                                layout.size.height = fret_ui::element::Length::Fill;
                                fret_ui::element::ViewCacheProps {
                                    layout,
                                    ..Default::default()
                                }
                            },
                            |cx| {
                                let selected = cx
                                    .get_model_cloned(&selected_page, Invalidation::Layout)
                                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                                let query = cx
                                    .get_model_cloned(&nav_query, Invalidation::Layout)
                                    .unwrap_or_default();

                                vec![if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                                    cx.container(
                                        decl_style::container_props(
                                            &theme,
                                            ChromeRefinement::default()
                                                .bg(ColorRef::Color(theme.color_required("muted")))
                                                .p(Space::N4),
                                            LayoutRefinement::default()
                                                .w_px(Px(280.0))
                                                .h_full(),
                                        ),
                                        |cx| vec![cx.text("Sidebar (disabled)")],
                                    )
                                } else {
                                    ui::sidebar_view(
                                        cx,
                                        &theme,
                                        selected.as_ref(),
                                        query.as_str(),
                                        nav_query.clone(),
                                        selected_page.clone(),
                                        state.workspace_tabs.clone(),
                                    )
                                }]
                            },
                        )
                    } else {
                        cx.keyed("ui_gallery.sidebar", |cx| {
                            let selected = cx
                                .get_model_cloned(&selected_page, Invalidation::Layout)
                                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                            let query = cx
                                .get_model_cloned(&nav_query, Invalidation::Layout)
                                .unwrap_or_default();

                            if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                                cx.container(
                                    decl_style::container_props(
                                        &theme,
                                        ChromeRefinement::default()
                                            .bg(ColorRef::Color(theme.color_required("muted")))
                                            .p(Space::N4),
                                        LayoutRefinement::default()
                                            .w_px(Px(280.0))
                                            .h_full(),
                                    ),
                                    |cx| vec![cx.text("Sidebar (disabled)")],
                                )
                            } else {
                                ui::sidebar_view(
                                    cx,
                                    &theme,
                                    selected.as_ref(),
                                    query.as_str(),
                                    nav_query.clone(),
                                    selected_page.clone(),
                                    state.workspace_tabs.clone(),
                                )
                            }
                        })
                    };

                    let content = if cache_shell {
                        cx.view_cache(
                            {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Fill;
                                layout.size.height = fret_ui::element::Length::Fill;
                                layout.flex.grow = 1.0;
                                fret_ui::element::ViewCacheProps {
                                    layout,
                                    ..Default::default()
                                }
                            },
                            |cx| {
                                let selected = cx
                                    .get_model_cloned(&selected_page, Invalidation::Layout)
                                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                                vec![cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                                    if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                                        cx.container(
                                            decl_style::container_props(
                                                &theme,
                                                ChromeRefinement::default()
                                                    .bg(ColorRef::Color(
                                                        theme.color_required("background"),
                                                    ))
                                                    .p(Space::N6),
                                                LayoutRefinement::default().w_full().h_full(),
                                            ),
                                            |cx| vec![cx.text("Content (disabled)")],
                                        )
                                    } else {
                                        ui::content_view(
                                            cx,
                                            &theme,
                                            selected.as_ref(),
                                            content_tab.clone(),
                                            theme_preset.clone(),
                                            theme_preset_open.clone(),
                                            view_cache_enabled.clone(),
                                            view_cache_cache_shell.clone(),
                                            view_cache_inner_enabled.clone(),
                                            view_cache_popover_open.clone(),
                                            view_cache_continuous.clone(),
                                            view_cache_counter.clone(),
                                            popover_open.clone(),
                                            dialog_open.clone(),
                                            alert_dialog_open.clone(),
                                            sheet_open.clone(),
                                            portal_geometry_popover_open.clone(),
                                            select_value.clone(),
                                            select_open.clone(),
                                            combobox_value.clone(),
                                            combobox_open.clone(),
                                            combobox_query.clone(),
                                            date_picker_open.clone(),
                                            date_picker_month.clone(),
                                            date_picker_selected.clone(),
                                            time_picker_open.clone(),
                                            time_picker_selected.clone(),
                                            resizable_h_fractions.clone(),
                                            resizable_v_fractions.clone(),
                                            data_table_state.clone(),
                                            data_grid_selected_row.clone(),
                                            tabs_value.clone(),
                                            accordion_value.clone(),
                                            avatar_demo_image.clone(),
                                            progress.clone(),
                                            checkbox.clone(),
                                            switch.clone(),
                                            material3_checkbox.clone(),
                                            material3_switch.clone(),
                                            material3_radio_value.clone(),
                                            material3_tabs_value.clone(),
                                            material3_list_value.clone(),
                                            material3_expressive.clone(),
                                            material3_navigation_bar_value.clone(),
                                            material3_navigation_rail_value.clone(),
                                            material3_navigation_drawer_value.clone(),
                                            material3_modal_navigation_drawer_open.clone(),
                                            material3_dialog_open.clone(),
                                            material3_text_field_value.clone(),
                                            material3_text_field_disabled.clone(),
                                            material3_text_field_error.clone(),
                                            material3_autocomplete_value.clone(),
                                            material3_autocomplete_disabled.clone(),
                                            material3_autocomplete_error.clone(),
                                            material3_autocomplete_dialog_open.clone(),
                                            material3_menu_open.clone(),
                                            text_input.clone(),
                                            text_area.clone(),
                                            dropdown_open.clone(),
                                            context_menu_open.clone(),
                                            context_menu_edge_open.clone(),
                                            cmdk_open.clone(),
                                            cmdk_query.clone(),
                                            last_action.clone(),
                                            sonner_position.clone(),
                                            virtual_list_torture_jump.clone(),
                                            virtual_list_torture_edit_row.clone(),
                                            virtual_list_torture_edit_text.clone(),
                                            virtual_list_torture_scroll.clone(),
                                            code_editor_syntax_rust.clone(),
                                            code_editor_boundary_identifier.clone(),
                                            code_editor_soft_wrap.clone(),
                                            code_editor_folds.clone(),
                                            code_editor_inlays.clone(),
                                        )
                                    }
                                })]
                            },
                        )
                    } else {
                        cx.keyed("ui_gallery.content_root", |cx| {
                            let selected = cx
                                .get_model_cloned(&selected_page, Invalidation::Layout)
                                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                            cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                                if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                                    cx.container(
                                        decl_style::container_props(
                                            &theme,
                                            ChromeRefinement::default()
                                                .bg(ColorRef::Color(
                                                    theme.color_required("background"),
                                                ))
                                                .p(Space::N6),
                                            LayoutRefinement::default().w_full().h_full(),
                                        ),
                                        |cx| vec![cx.text("Content (disabled)")],
                                    )
                                } else {
                                    ui::content_view(
                                        cx,
                                        &theme,
                                        selected.as_ref(),
                                        content_tab.clone(),
                                        theme_preset.clone(),
                                        theme_preset_open.clone(),
                                        view_cache_enabled.clone(),
                                        view_cache_cache_shell.clone(),
                                        view_cache_inner_enabled.clone(),
                                        view_cache_popover_open.clone(),
                                        view_cache_continuous.clone(),
                                        view_cache_counter.clone(),
                                        popover_open.clone(),
                                        dialog_open.clone(),
                                        alert_dialog_open.clone(),
                                        sheet_open.clone(),
                                        portal_geometry_popover_open.clone(),
                                        select_value.clone(),
                                        select_open.clone(),
                                        combobox_value.clone(),
                                        combobox_open.clone(),
                                        combobox_query.clone(),
                                        date_picker_open.clone(),
                                        date_picker_month.clone(),
                                        date_picker_selected.clone(),
                                        time_picker_open.clone(),
                                        time_picker_selected.clone(),
                                        resizable_h_fractions.clone(),
                                        resizable_v_fractions.clone(),
                                        data_table_state.clone(),
                                        data_grid_selected_row.clone(),
                                        tabs_value.clone(),
                                        accordion_value.clone(),
                                        avatar_demo_image.clone(),
                                        progress.clone(),
                                        checkbox.clone(),
                                        switch.clone(),
                                        material3_checkbox.clone(),
                                        material3_switch.clone(),
                                        material3_radio_value.clone(),
                                        material3_tabs_value.clone(),
                                        material3_list_value.clone(),
                                        material3_expressive.clone(),
                                        material3_navigation_bar_value.clone(),
                                        material3_navigation_rail_value.clone(),
                                        material3_navigation_drawer_value.clone(),
                                        material3_modal_navigation_drawer_open.clone(),
                                        material3_dialog_open.clone(),
                                        material3_text_field_value.clone(),
                                        material3_text_field_disabled.clone(),
                                        material3_text_field_error.clone(),
                                        material3_autocomplete_value.clone(),
                                        material3_autocomplete_disabled.clone(),
                                        material3_autocomplete_error.clone(),
                                        material3_autocomplete_dialog_open.clone(),
                                        material3_menu_open.clone(),
                                        text_input.clone(),
                                        text_area.clone(),
                                        dropdown_open.clone(),
                                        context_menu_open.clone(),
                                        context_menu_edge_open.clone(),
                                        cmdk_open.clone(),
                                        cmdk_query.clone(),
                                        last_action.clone(),
                                        sonner_position.clone(),
                                        virtual_list_torture_jump.clone(),
                                        virtual_list_torture_edit_row.clone(),
                                        virtual_list_torture_edit_text.clone(),
                                        virtual_list_torture_scroll.clone(),
                                        code_editor_syntax_rust.clone(),
                                        code_editor_boundary_identifier.clone(),
                                        code_editor_soft_wrap.clone(),
                                        code_editor_folds.clone(),
                                        code_editor_inlays.clone(),
                                    )
                                }
                            })
                        })
                    };

                    let tab_strip = cx.keyed("ui_gallery.tab_strip", |cx| {
                        if (bisect & BISECT_DISABLE_TAB_STRIP) != 0 {
                            return cx.text("Tabs (disabled)");
                        }

                        let selected = cx
                            .get_model_cloned(&selected_page, Invalidation::Layout)
                            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                        let workspace_tab_ids = cx
                            .get_model_cloned(&workspace_tabs, Invalidation::Layout)
                            .unwrap_or_default();
                        let workspace_dirty_ids = cx
                            .get_model_cloned(&workspace_dirty_tabs, Invalidation::Layout)
                            .unwrap_or_default();

                        WorkspaceTabStrip::new(selected.clone())
                            .tabs(workspace_tab_ids.iter().map(|tab_id| {
                                let (title, _origin, _docs, _usage) =
                                    crate::spec::page_meta(tab_id.as_ref());
                                let dirty = workspace_dirty_ids
                                    .iter()
                                    .any(|d| d.as_ref() == tab_id.as_ref());
                                WorkspaceTab::new(
                                    tab_id.clone(),
                                    title,
                                    page_spec(tab_id.as_ref())
                                        .map(|spec| CommandId::from(spec.command))
                                        .unwrap_or_else(|| {
                                            CommandId::new(format!(
                                                "ui_gallery.nav.select.{}",
                                                tab_id.as_ref()
                                            ))
                                        }),
                                )
                                .close_command(CommandId::new(format!(
                                    "{}{}",
                                    CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                                    tab_id.as_ref()
                                )))
                                .dirty(dirty)
                            }))
                            .into_element(cx)
                    });

                    let menu_bar_seq_value = cx
                        .get_model_copied(&menu_bar_seq, Invalidation::Layout)
                        .unwrap_or(0);
                    let menu_bar = fret_app::effective_menu_bar(cx.app);
                    let show_in_window_menu_bar = fret_app::should_render_in_window_menu_bar(
                        cx.app,
                        fret_app::Platform::current(),
                    );
                    cx.app.with_global_mut(
                        fret_runtime::WindowMenuBarFocusService::default,
                        |svc, _app| {
                            svc.set_present(cx.window, show_in_window_menu_bar && menu_bar.is_some());
                        },
                    );
                    let menubar_handle: std::cell::RefCell<Option<InWindowMenubarFocusHandle>> =
                        std::cell::RefCell::new(None);
                    let in_window_menu_bar = if show_in_window_menu_bar {
                        menu_bar.as_ref().map(|menu_bar| {
                            cx.keyed(format!("ui_gallery.menubar.{menu_bar_seq_value}"), |cx| {
                                let (menu, handle) = menubar_from_runtime_with_focus_handle(
                                    cx,
                                    menu_bar,
                                    MenubarFromRuntimeOptions::default(),
                                );
                                *menubar_handle.borrow_mut() = Some(handle);
                                menu
                            })
                        })
                    } else {
                        None
                    };

                    let top_bar = WorkspaceTopBar::new()
                        .left(in_window_menu_bar.into_iter().collect::<Vec<_>>())
                        .center(vec![tab_strip])
                        .right(vec![
                            shadcn::Button::new("Command palette")
                                .test_id("ui-gallery-command-palette")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(fret_app::core_commands::COMMAND_PALETTE)
                                .into_element(cx),
                        ])
                        .into_element(cx);

                    let status_bar = cx.keyed("ui_gallery.status_bar", |cx| {
                        let status_last_action = cx
                            .get_model_cloned(&last_action, Invalidation::Layout)
                            .unwrap_or_else(|| Arc::<str>::from("<none>"));
                        let status_theme = cx
                            .get_model_cloned(&theme_preset, Invalidation::Layout)
                            .flatten()
                            .unwrap_or_else(|| Arc::<str>::from("<default>"));
                        let status_view_cache = cx
                            .get_model_copied(&view_cache_enabled, Invalidation::Layout)
                            .unwrap_or(false);
                        let status_cache_shell = cx
                            .get_model_copied(&view_cache_cache_shell, Invalidation::Layout)
                            .unwrap_or(false);

                        let mut right_items: Vec<AnyElement> = vec![cx.text(format!(
                            "theme: {} view_cache={} shell_cache={} layout_us={} paint_us={}",
                            status_theme.as_ref(),
                            status_view_cache as u8,
                            status_cache_shell as u8,
                            last_debug_stats.layout_time.as_micros(),
                            last_debug_stats.paint_time.as_micros()
                        ))];
                        if let Some((cursor, hit, focus, text)) = inspector_status.as_ref() {
                            right_items.push(cx.text(format!("inspect: {}", cursor.as_ref())));
                            right_items.push(cx.text(format!("inspect: {}", hit.as_ref())));
                            right_items.push(cx.text(format!("inspect: {}", focus.as_ref())));
                            right_items.push(cx.text(format!("inspect: {}", text.as_ref())));
                        }

                        let status_last_action_label =
                            Arc::<str>::from(format!("last action: {}", status_last_action.as_ref()));
                        let status_last_action_text = status_last_action_label.clone();
                        let status_last_action_item = cx.semantics(
                            SemanticsProps {
                                role: SemanticsRole::Text,
                                label: Some(status_last_action_label),
                                test_id: Some(Arc::from("ui-gallery-status-last-action")),
                                ..Default::default()
                            },
                            move |cx| vec![cx.text(status_last_action_text.as_ref())],
                        );

                        WorkspaceStatusBar::new()
                            .left(vec![status_last_action_item])
                            .right(right_items)
                            .into_element(cx)
                    });

                    let mut center_layout = fret_ui::element::LayoutStyle::default();
                    center_layout.size.width = fret_ui::element::Length::Fill;
                    center_layout.size.height = fret_ui::element::Length::Fill;
                    center_layout.flex.grow = 1.0;

                    let center = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: center_layout,
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        |_cx| vec![sidebar, content],
                    );

                    let frame = WorkspaceFrame::new(center)
                        .top(top_bar)
                        .bottom(status_bar)
                        .into_element(cx);

                    let panel = frame.attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Panel)
                            .label("fret-ui-gallery"),
                    );
                    if let Some(handle) = menubar_handle.borrow().clone() {
                        let group_active = handle.group_active.clone();
                        let trigger_registry = handle.trigger_registry.clone();
                        let last_focus_before_menubar = handle.last_focus_before_menubar.clone();
                        let focus_is_trigger = handle.focus_is_trigger.clone();
                        let group_active_for_command = group_active.clone();
                        let trigger_registry_for_command = trigger_registry.clone();
                        let last_focus_for_command = last_focus_before_menubar.clone();
                        cx.command_add_on_command_for(
                            panel.id,
                            Arc::new(move |host, acx, command| {
                                if command.as_str() != fret_app::core_commands::FOCUS_MENU_BAR {
                                    return false;
                                }

                                let active = host
                                    .models_mut()
                                    .get_cloned(&group_active_for_command)
                                    .flatten();
                                if let Some(active) = active {
                                    let _ = host.models_mut().update(&active.open, |v| *v = false);
                                    let _ = host
                                        .models_mut()
                                        .update(&group_active_for_command, |v| *v = None);
                                    let restore =
                                        host.models_mut().get_cloned(&last_focus_for_command).flatten();
                                    host.request_focus(restore.unwrap_or(active.trigger));
                                    host.request_redraw(acx.window);
                                    return true;
                                }

                                let entries = host
                                    .models_mut()
                                    .get_cloned(&trigger_registry_for_command)
                                    .unwrap_or_default();
                                let target = entries.iter().find(|e| e.enabled).cloned();
                                let Some(target) = target else {
                                    return false;
                                };

                                let open_for_state = target.open.clone();
                                let _ = host
                                    .models_mut()
                                    .update(&group_active_for_command, |v| {
                                        *v = Some(
                                            fret_ui_kit::primitives::menubar::trigger_row::MenubarActiveTrigger {
                                                trigger: target.trigger,
                                                open: open_for_state,
                                            },
                                        );
                                    });

                                host.request_focus(target.trigger);
                                host.request_redraw(acx.window);
                                true
                            }),
                        );

                        cx.key_add_on_key_down_for(
                            panel.id,
                            fret_ui_kit::primitives::menubar::trigger_row::open_on_alt_mnemonic(
                                group_active.clone(),
                                trigger_registry.clone(),
                            ),
                        );
                        cx.key_add_on_key_down_for(
                            panel.id,
                            fret_ui_kit::primitives::menubar::trigger_row::open_on_mnemonic_when_active(
                                group_active.clone(),
                                trigger_registry.clone(),
                                focus_is_trigger.clone(),
                            ),
                        );
                        cx.key_add_on_key_down_for(
                            panel.id,
                            fret_ui_kit::primitives::menubar::trigger_row::exit_active_on_escape_when_closed(
                                group_active.clone(),
                                last_focus_before_menubar.clone(),
                                focus_is_trigger.clone(),
                            ),
                        );
                    }

                    let mut content: Vec<AnyElement> = vec![
                        panel,
                        if (bisect & BISECT_DISABLE_TOASTER) != 0 {
                            cx.text("")
                        } else {
                            {
                                let position = cx
                                    .get_model_copied(&sonner_position, Invalidation::Layout)
                                    .unwrap_or(shadcn::ToastPosition::TopCenter);
                                shadcn::Toaster::new().position(position).into_element(cx)
                            }
                        },
                    ];

                    content.push(cx.keyed("ui_gallery.settings_sheet", |cx| {
                        shadcn::Sheet::new(settings_open.clone())
                            .side(shadcn::SheetSide::Right)
                            .size(Px(420.0))
                            .into_element(
                                cx,
                                |cx| {
                                    let mut layout = fret_ui::element::LayoutStyle::default();
                                    layout.size.width = fret_ui::element::Length::Px(Px(0.0));
                                    layout.size.height = fret_ui::element::Length::Px(Px(0.0));
                                    cx.container(
                                        fret_ui::element::ContainerProps {
                                            layout,
                                            ..Default::default()
                                        },
                                        |_cx| Vec::new(),
                                    )
                                },
                                |cx| {
                                    let os_select = shadcn::Select::new(
                                        settings_menu_bar_os.clone(),
                                        settings_menu_bar_os_open.clone(),
                                    )
                                    .placeholder("OS menubar")
                                    .trigger_test_id("ui-gallery-settings-os-menubar")
                                    .items([
                                        shadcn::SelectItem::new(
                                            "auto",
                                            "Auto (Windows/macOS on; Linux/Web off)",
                                        )
                                        .test_id("ui-gallery-settings-os-menubar-auto"),
                                        shadcn::SelectItem::new("on", "On")
                                            .test_id("ui-gallery-settings-os-menubar-on"),
                                        shadcn::SelectItem::new("off", "Off")
                                            .test_id("ui-gallery-settings-os-menubar-off"),
                                    ])
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx);

                                    let in_window_select = shadcn::Select::new(
                                        settings_menu_bar_in_window.clone(),
                                        settings_menu_bar_in_window_open.clone(),
                                    )
                                    .placeholder("In-window menubar")
                                    .trigger_test_id("ui-gallery-settings-in-window-menubar")
                                    .items([
                                        shadcn::SelectItem::new(
                                            "auto",
                                            "Auto (Linux/Web on; Windows/macOS off)",
                                        )
                                        .test_id("ui-gallery-settings-in-window-menubar-auto"),
                                        shadcn::SelectItem::new("on", "On")
                                            .test_id("ui-gallery-settings-in-window-menubar-on"),
                                        shadcn::SelectItem::new("off", "Off")
                                            .test_id("ui-gallery-settings-in-window-menubar-off"),
                                    ])
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx);

                                    let body = stack::vstack(
                                        cx,
                                        stack::VStackProps::default()
                                            .layout(LayoutRefinement::default().w_full())
                                            .gap(Space::N4),
                                        |cx| {
                                            vec![
                                                stack::vstack(
                                                    cx,
                                                    stack::VStackProps::default()
                                                        .layout(LayoutRefinement::default().w_full())
                                                        .gap(Space::N2),
                                                    |cx| {
                                                        vec![
                                                            shadcn::SheetHeader::new(vec![
                                                                shadcn::SheetTitle::new("Settings")
                                                                    .into_element(cx),
                                                                shadcn::SheetDescription::new(
                                                                    "Menu bar presentation (OS vs in-window).",
                                                                )
                                                                .into_element(cx),
                                                            ])
                                                            .into_element(cx),
                                                            shadcn::Separator::new().into_element(cx),
                                                            cx.text("Menu bar surfaces"),
                                                            os_select,
                                                            in_window_select,
                                                            cx.text("Command availability (debug)"),
                                                            stack::hstack(
                                                                cx,
                                                                stack::HStackProps::default()
                                                                    .gap(Space::N2)
                                                                    .items_center(),
                                                                |cx| {
                                                                    vec![
                                                                        shadcn::Switch::new(
                                                                            settings_edit_can_undo
                                                                                .clone(),
                                                                        )
                                                                        .a11y_label("Can Undo")
                                                                        .disabled(true)
                                                                        .into_element(cx),
                                                                        cx.text(
                                                                            "edit.can_undo (enables OS/in-window Undo)",
                                                                        ),
                                                                    ]
                                                                },
                                                            ),
                                                            stack::hstack(
                                                                cx,
                                                                stack::HStackProps::default()
                                                                    .gap(Space::N2)
                                                                    .items_center(),
                                                                |cx| {
                                                                    vec![
                                                                        shadcn::Switch::new(
                                                                            settings_edit_can_redo
                                                                                .clone(),
                                                                        )
                                                                        .a11y_label("Can Redo")
                                                                        .disabled(true)
                                                                        .into_element(cx),
                                                                        cx.text(
                                                                            "edit.can_redo (enables OS/in-window Redo)",
                                                                        ),
                                                                    ]
                                                                },
                                                            ),
                                                        ]
                                                    },
                                                ),
                                                shadcn::SheetFooter::new(vec![
                                                    shadcn::Button::new("Apply (in memory)")
                                                        .variant(shadcn::ButtonVariant::Secondary)
                                                        .test_id("ui-gallery-settings-apply")
                                                        .on_click(CMD_APP_SETTINGS_APPLY)
                                                        .into_element(cx),
                                                    shadcn::Button::new(
                                                        "Write project .fret/settings.json",
                                                    )
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CMD_APP_SETTINGS_WRITE_PROJECT)
                                                    .into_element(cx),
                                                    shadcn::Button::new("Close")
                                                        .variant(shadcn::ButtonVariant::Ghost)
                                                        .toggle_model(settings_open.clone())
                                                        .into_element(cx),
                                                ])
                                                .into_element(cx),
                                            ]
                                        },
                                    );

                                    shadcn::SheetContent::new(vec![body]).into_element(cx)
                                },
                            )
                    }));

                    if show_debug_hud {
                        let debug_hud_lines = debug_hud_lines.clone();
                        content.push(cx.keyed("ui_gallery.debug_hud", |cx| {
                            let mut hud_layout = fret_ui::element::LayoutStyle::default();
                            hud_layout.position = fret_ui::element::PositionStyle::Absolute;
                            hud_layout.inset.top = Some(Px(8.0));
                            hud_layout.inset.right = Some(Px(8.0));
                            hud_layout.size.width = fret_ui::element::Length::Px(Px(520.0));
                            hud_layout.size.height = fret_ui::element::Length::Px(Px(220.0));

                            let mut gate = fret_ui::element::InteractivityGateProps::default();
                            gate.layout = hud_layout;
                            gate.present = true;
                            gate.interactive = false;

                            cx.interactivity_gate_props(gate, |cx| {
                                let mut container_props = decl_style::container_props(
                                    &theme,
                                    ChromeRefinement::default()
                                        .bg(ColorRef::Color(theme.color_required("background")))
                                        .border_1()
                                        .rounded(Radius::Md)
                                        .p(Space::N3),
                                    LayoutRefinement::default().w_full().h_full(),
                                );
                                container_props.layout.size.width = fret_ui::element::Length::Fill;
                                container_props.layout.size.height = fret_ui::element::Length::Fill;
                                container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                                let body = stack::vstack(
                                    cx,
                                    stack::VStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .gap(Space::N1),
                                    |cx| {
                                        debug_hud_lines
                                            .iter()
                                            .map(|line| {
                                                cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: line.clone(),
                                                    style: None,
                                                    color: Some(theme.color_required("foreground")),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                })
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                );

                                [cx.container(container_props, |cx| {
                                    [shadcn::ScrollArea::new([body])
                                        .refine_layout(LayoutRefinement::default().w_full().h_full())
                                        .into_element(cx)]
                                })]
                            })
                        }));
                    }

                    if cx
                        .get_model_copied(&inspector_enabled, Invalidation::Layout)
                        .unwrap_or(false)
                    {
                        cx.observe_model(&inspector_last_pointer, Invalidation::Paint);

                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = fret_ui::element::Length::Fill;
                        props.layout.size.height = fret_ui::element::Length::Fill;

                        let on_pointer_move = {
                            let inspector_last_pointer = inspector_last_pointer.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      cx: fret_ui::action::ActionCx,
                                      mv: fret_ui::action::PointerMoveCx| {
                                    let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                                        *v = Some(mv.position);
                                    });
                                    host.request_redraw(cx.window);
                                    false
                                },
                            )
                        };
                        let on_pointer_down = {
                            let inspector_last_pointer = inspector_last_pointer.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      cx: fret_ui::action::ActionCx,
                                      down: fret_ui::action::PointerDownCx| {
                                    let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                                        *v = Some(down.position);
                                    });
                                    host.request_redraw(cx.window);
                                    false
                                },
                            )
                        };

                        vec![cx.pointer_region(props, |cx| {
                            cx.pointer_region_on_pointer_move(on_pointer_move);
                            cx.pointer_region_on_pointer_down(on_pointer_down);
                            content
                        })]
                    } else {
                        content
                    }
                });

        state.ui.set_root(root);
        if (bisect & BISECT_DISABLE_OVERLAY_CONTROLLER) == 0 {
            OverlayController::render(&mut state.ui, app, services, window, bounds);
        }
        state.root = Some(root);
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(UiGalleryRecentItemsService::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let config_paths = LayeredConfigPaths::for_project_root(".");
    if let Ok((settings, _report)) = load_layered_settings(&config_paths) {
        fret_app::settings::apply_settings_globals(&mut app, &settings);
    }

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
    fret_app::install_command_default_keybindings_into_keymap(&mut app);
    UiGalleryDriver::sync_dynamic_menu_command_metadata(&mut app);
    app.push_effect(Effect::SetMenuBar {
        window: None,
        menu_bar: UiGalleryDriver::build_menu_bar(&app),
    });

    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc: &mut UiDiagnosticsService, _app| {
        svc.set_app_snapshot_provider(Some(Arc::new(|app, window| {
            let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
            let ids = store.per_window.get(&window)?;

            let selected_page = app.models().get_cloned(&ids.selected_page)?;
            let syntax_rust = app.models().get_cloned(&ids.code_editor_syntax_rust)?;
            let boundary_identifier = app.models().get_cloned(&ids.code_editor_boundary_identifier)?;
            let soft_wrap = app.models().get_cloned(&ids.code_editor_soft_wrap)?;
            let folds = app.models().get_cloned(&ids.code_editor_folds)?;
            let inlays = app.models().get_cloned(&ids.code_editor_inlays)?;
            let text_input = app.models().get_cloned(&ids.text_input)?;
            let text_area = app.models().get_cloned(&ids.text_area)?;

            let (torture, markdown_editor_source): (Option<serde_json::Value>, Option<serde_json::Value>) = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let torture = app
                        .global::<UiGalleryCodeEditorHandlesStore>()
                        .and_then(|store| store.per_window.get(&window))
                        .map(|handle| {
                            let text = handle.with_buffer(|b| b.text_string());
                            let selection = handle.selection();
                            let anchor = selection.anchor.min(text.len()) as u64;
                            let caret = selection.caret().min(text.len()) as u64;
                            let stats = handle.cache_stats();
                            let paint_perf = handle.paint_perf_frame().map(|frame| {
                                serde_json::json!({
                                    "schema_version": 1,
                                    "frame_seq": frame.frame_seq,
                                    "visible_start": frame.visible_start,
                                    "visible_end": frame.visible_end,
                                    "visible_rows": frame.visible_rows,
                                    "rows_painted": frame.rows_painted,
                                    "rows_drew_rich": frame.rows_drew_rich,
                                    "quads_background": frame.quads_background,
                                    "quads_selection": frame.quads_selection,
                                    "quads_caret": frame.quads_caret,
                                    "us_total": frame.us_total,
                                    "us_row_text": frame.us_row_text,
                                    "us_baseline_measure": frame.us_baseline_measure,
                                    "us_syntax_spans": frame.us_syntax_spans,
                                    "us_rich_materialize": frame.us_rich_materialize,
                                    "us_text_draw": frame.us_text_draw,
                                    "us_selection_rects": frame.us_selection_rects,
                                    "us_caret_x": frame.us_caret_x,
                                    "us_caret_stops": frame.us_caret_stops,
                                    "us_caret_rect": frame.us_caret_rect,
                                })
                            });
                            let preedit_active = handle.preedit_active();
                            let interaction = handle.interaction();
                            let buffer_revision = handle.buffer_revision().0 as u64;
                            let fold_placeholder_present = handle
                                .debug_decorated_line_text(0)
                                .is_some_and(|t| t.contains('…'));
                            let inlay_present = handle
                                .debug_decorated_line_text(0)
                                .is_some_and(|t| t.contains("<inlay>"));
                            serde_json::json!({
                                "schema_version": 1,
                                "marker_present": text.contains(UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER),
                                "preedit_active": preedit_active,
                                "interaction": {
                                    "enabled": interaction.enabled,
                                    "focusable": interaction.focusable,
                                    "selectable": interaction.selectable,
                                    "editable": interaction.editable,
                                },
                                "buffer_revision": buffer_revision,
                                "folds": { "enabled": folds, "line0_placeholder_present": fold_placeholder_present },
                                "inlays": { "enabled": inlays, "line0_present": inlay_present },
                                "text_len_bytes": text.len() as u64,
                                "selection": { "anchor": anchor, "caret": caret },
                                "cache_stats": {
                                    "row_text_get_calls": stats.row_text_get_calls,
                                    "row_text_hits": stats.row_text_hits,
                                    "row_text_misses": stats.row_text_misses,
                                    "row_text_evictions": stats.row_text_evictions,
                                    "row_text_resets": stats.row_text_resets,
                                    "row_rich_get_calls": stats.row_rich_get_calls(),
                                    "row_rich_hits": stats.row_rich_hits(),
                                    "row_rich_misses": stats.row_rich_misses(),
                                    "row_rich_evictions": stats.row_rich_evictions(),
                                    "row_rich_resets": stats.row_rich_resets(),
                                    "geom_pointer_hit_test_fallbacks": stats.geom_pointer_hit_test_fallbacks,
                                    "geom_caret_rect_fallbacks": stats.geom_caret_rect_fallbacks,
                                    "geom_vertical_move_fallbacks": stats.geom_vertical_move_fallbacks,
                                    "syntax_get_calls": stats.syntax_get_calls,
                                    "syntax_hits": stats.syntax_hits,
                                    "syntax_misses": stats.syntax_misses,
                                    "syntax_evictions": stats.syntax_evictions,
                                    "syntax_resets": stats.syntax_resets,
                                },
                                "paint_perf": paint_perf,
                            })
                        })
                        ;

                    let markdown_editor_source = app
                        .global::<UiGalleryMarkdownEditorHandlesStore>()
                        .and_then(|store| store.per_window.get(&window))
                        .map(|handle| {
                            let text = handle.with_buffer(|b| b.text_string());
                            let selection = handle.selection();
                            let anchor = selection.anchor.min(text.len()) as u64;
                            let caret = selection.caret().min(text.len()) as u64;
                            let interaction = handle.interaction();
                            let buffer_revision = handle.buffer_revision().0 as u64;
                            serde_json::json!({
                                "schema_version": 1,
                                "interaction": {
                                    "enabled": interaction.enabled,
                                    "focusable": interaction.focusable,
                                    "selectable": interaction.selectable,
                                    "editable": interaction.editable,
                                },
                                "buffer_revision": buffer_revision,
                                "text_len_bytes": text.len() as u64,
                                "selection": { "anchor": anchor, "caret": caret },
                            })
                        })
                        ;

                    (torture, markdown_editor_source)
                }

                #[cfg(target_arch = "wasm32")]
                {
                    (None, None)
                }
            };

            let mut out = serde_json::Map::new();
            out.insert("schema_version".to_string(), serde_json::json!(1));
            out.insert("kind".to_string(), serde_json::json!("fret_ui_gallery"));
            out.insert(
                "selected_page".to_string(),
                serde_json::Value::String(selected_page.to_string()),
            );
            out.insert(
                "code_editor".to_string(),
                serde_json::json!({
                    "syntax_rust": syntax_rust,
                    "text_boundary_mode": if boundary_identifier { "identifier" } else { "unicode_word" },
                    "soft_wrap_cols": if soft_wrap { Some(80u32) } else { None },
                    "folds_fixture": folds,
                    "inlays_fixture": inlays,
                    "torture": torture,
                    "markdown_editor_source": markdown_editor_source,
                }),
            );
            out.insert(
                "text_widgets".to_string(),
                serde_json::json!({
                    "text_input_chars": text_input.chars().count(),
                    "text_area_chars": text_area.chars().count(),
                }),
            );

            Some(serde_json::Value::Object(out))
        })));
    });

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
    UiGalleryDriver::default()
}

#[cfg(test)]
mod stack_overflow_tests;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();

    fret_bootstrap::BootstrapBuilder::new(app, build_driver())
        .configure(move |c| {
            *c = config;
        })
        .with_default_diagnostics()
        .with_default_config_files()?
        .with_config_files_watcher(Duration::from_millis(500))
        .with_lucide_icons()
        .preload_icon_svgs_on_gpu_ready()
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

impl WinitAppDriver for UiGalleryDriver {
    type WindowState = UiGalleryWindowState;

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
        {
            if seq != 0 && context.state.last_config_files_status_seq != seq {
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
            let _ = app.with_global_mut(
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
            let _ = app.with_global_mut(
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
                            let (settings, _report) = load_layered_settings(&paths)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
                        .action(shadcn::ToastAction {
                            label: Arc::from("Undo"),
                            command: CommandId::new(CMD_TOAST_ACTION),
                        })
                        .cancel(shadcn::ToastAction {
                            label: Arc::from("Cancel"),
                            command: CommandId::new(CMD_TOAST_CANCEL),
                        })
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
            Event::ImageRegistered { token, image, .. } => {
                if state.avatar_demo_image_token == Some(*token) {
                    state.avatar_demo_image_token = None;
                    state.avatar_demo_image_retry_count = 0;
                    let _ = app
                        .models_mut()
                        .update(&state.avatar_demo_image, |v| *v = Some(*image));
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

        #[cfg(not(target_arch = "wasm32"))]
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
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "missing UiTree root",
                ))
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

        #[cfg(not(target_arch = "wasm32"))]
        {
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
