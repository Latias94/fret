use fret_app::App;
use fret_app::CommandId;
use fret_app::Effect;
use fret_app::Model;
use fret_core::{
    AppWindowId, Color, Corners, Edges, Event, NodeId, Px, TextOverflow, TextWrap, UiServices,
    ViewportInputEvent,
};
use fret_launch::{
    EngineFrameUpdate, FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext,
    WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};
use fret_ui::declarative::RenderRootContext;
use fret_ui::element::Elements;
use fret_ui::overlay_placement::LayoutDirection;
use fret_ui::{ElementContext, Invalidation, Theme, UiFrameCx, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_kit::primitives::dialog as dialog_prim;
use fret_ui_kit::primitives::direction as direction_prim;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

use fret_core::time::Instant;

#[cfg(feature = "diagnostics")]
use crate::ui_diagnostics::UiDiagnosticsService;

pub type ViewElements = Elements;

type ViewFn<S> = for<'a> fn(&mut ElementContext<'a, App>, &mut S) -> ViewElements;

type EventHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &Event);

type CommandHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &CommandId);

type PreferencesHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S);

type HotReloadHookFn<S> = fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S);

type ModelChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[fret_app::ModelId]);
type GlobalChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[std::any::TypeId]);

type RecordEngineFrameHookFn<S> = fn(
    &mut App,
    AppWindowId,
    &mut UiTree<App>,
    &mut S,
    &WgpuContext,
    &mut Renderer,
    f32,
    TickId,
    FrameId,
) -> EngineFrameUpdate;

/// A minimal, hotpatch-friendly “golden path” app driver.
///
/// This wraps `fret-launch::FnDriver` and centralizes common boilerplate:
/// - declarative root mounting (`RenderRootContext`)
/// - `UiTree` event/command routing
/// - model/global change propagation
/// - layout/paint submission via `UiFrameCx`
/// - accessibility snapshot + actions
/// - conservative hot reload reset (Subsecond-friendly)
///
/// This driver intentionally uses `fn` pointers (not captured closures) to keep dev hotpatch behavior
/// predictable (ADR 0107).
pub struct UiAppDriver<S> {
    root_name: &'static str,
    init_window: fn(&mut App, AppWindowId) -> S,
    view: ViewFn<S>,
    close_on_window_close_requested: bool,
    #[cfg(feature = "ui-assets")]
    drive_ui_assets: bool,

    on_event: Option<EventHookFn<S>>,
    on_command: Option<CommandHookFn<S>>,
    on_preferences: Option<PreferencesHookFn<S>>,
    on_hot_reload_window: Option<HotReloadHookFn<S>>,
    on_model_changes: Option<ModelChangesHookFn<S>>,
    on_global_changes: Option<GlobalChangesHookFn<S>>,

    window_create_spec:
        Option<fn(&mut App, &fret_app::CreateWindowRequest) -> Option<WindowCreateSpec>>,
    window_created: Option<fn(&mut App, &fret_app::CreateWindowRequest, AppWindowId)>,
    before_close_window: Option<fn(&mut App, AppWindowId) -> bool>,

    handle_global_command: Option<fn(&mut App, &mut dyn UiServices, CommandId)>,

    viewport_input: Option<fn(&mut App, ViewportInputEvent)>,
    dock_op: Option<fn(&mut App, fret_core::DockOp)>,
    record_engine_frame: Option<RecordEngineFrameHookFn<S>>,

    #[cfg(feature = "ui-app-command-palette")]
    command_palette_enabled: bool,
}

impl<S> UiAppDriver<S> {
    pub fn new(
        root_name: &'static str,
        init_window: fn(&mut App, AppWindowId) -> S,
        view: ViewFn<S>,
    ) -> Self {
        Self {
            root_name,
            init_window,
            view,
            close_on_window_close_requested: true,
            #[cfg(feature = "ui-assets")]
            drive_ui_assets: true,
            on_event: None,
            on_command: None,
            on_preferences: None,
            on_hot_reload_window: None,
            on_model_changes: None,
            on_global_changes: None,
            window_create_spec: None,
            window_created: None,
            before_close_window: None,
            handle_global_command: None,
            viewport_input: None,
            dock_op: None,
            record_engine_frame: None,

            #[cfg(feature = "ui-app-command-palette")]
            command_palette_enabled: true,
        }
    }

    #[cfg(feature = "ui-app-command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.command_palette_enabled = enabled;
        self
    }

    pub fn on_event(mut self, f: EventHookFn<S>) -> Self {
        self.on_event = Some(f);
        self
    }

    /// When `true` (default, with the `ui-assets` feature enabled), drives `fret-ui-assets`
    /// caches from the event pipeline.
    ///
    /// This makes `ImageAssetCache` work out-of-the-box in golden-path apps without additional
    /// boilerplate (ADR 0108 / ADR 0112).
    #[cfg(feature = "ui-assets")]
    pub fn drive_ui_assets(mut self, enabled: bool) -> Self {
        self.drive_ui_assets = enabled;
        self
    }

    /// When `true` (default), receiving `Event::WindowCloseRequested` emits
    /// `Effect::Window(WindowRequest::Close(window))` for the active window.
    ///
    /// This keeps the “golden path” behavior intuitive for small apps, while advanced apps can
    /// disable it and implement custom close flows (e.g. unsaved-changes prompts) in `on_event`.
    pub fn close_on_window_close_requested(mut self, enabled: bool) -> Self {
        self.close_on_window_close_requested = enabled;
        self
    }

    pub fn on_command(mut self, f: CommandHookFn<S>) -> Self {
        self.on_command = Some(f);
        self
    }

    /// Register a handler for the standard `app.preferences` command.
    ///
    /// This is intentionally app-owned (no OS-native default beyond menu wiring).
    pub fn on_preferences(mut self, f: PreferencesHookFn<S>) -> Self {
        self.on_preferences = Some(f);
        self
    }

    pub fn on_hot_reload_window(mut self, f: HotReloadHookFn<S>) -> Self {
        self.on_hot_reload_window = Some(f);
        self
    }

    pub fn on_model_changes(mut self, f: ModelChangesHookFn<S>) -> Self {
        self.on_model_changes = Some(f);
        self
    }

    pub fn on_global_changes(mut self, f: GlobalChangesHookFn<S>) -> Self {
        self.on_global_changes = Some(f);
        self
    }

    pub fn window_create_spec(
        mut self,
        f: fn(&mut App, &fret_app::CreateWindowRequest) -> Option<WindowCreateSpec>,
    ) -> Self {
        self.window_create_spec = Some(f);
        self
    }

    pub fn window_created(
        mut self,
        f: fn(&mut App, &fret_app::CreateWindowRequest, AppWindowId),
    ) -> Self {
        self.window_created = Some(f);
        self
    }

    pub fn before_close_window(mut self, f: fn(&mut App, AppWindowId) -> bool) -> Self {
        self.before_close_window = Some(f);
        self
    }

    pub fn handle_global_command(
        mut self,
        f: fn(&mut App, &mut dyn UiServices, CommandId),
    ) -> Self {
        self.handle_global_command = Some(f);
        self
    }

    pub fn viewport_input(mut self, f: fn(&mut App, ViewportInputEvent)) -> Self {
        self.viewport_input = Some(f);
        self
    }

    pub fn dock_op(mut self, f: fn(&mut App, fret_core::DockOp)) -> Self {
        self.dock_op = Some(f);
        self
    }

    pub fn record_engine_frame(mut self, f: RecordEngineFrameHookFn<S>) -> Self {
        self.record_engine_frame = Some(f);
        self
    }

    pub fn into_fn_driver(self) -> FnDriver<Self, UiAppWindowState<S>> {
        FnDriver::new(
            self,
            ui_app_create_window_state::<S>,
            ui_app_handle_event::<S>,
            ui_app_render::<S>,
        )
        .with_hooks(|hooks| {
            hooks.handle_command = Some(ui_app_handle_command::<S>);
            hooks.handle_global_command = Some(ui_app_handle_global_command::<S>);
            hooks.handle_model_changes = Some(ui_app_handle_model_changes::<S>);
            hooks.handle_global_changes = Some(ui_app_handle_global_changes::<S>);

            hooks.hot_reload_window = Some(ui_app_hot_reload_window::<S>);

            hooks.window_create_spec = Some(ui_app_window_create_spec::<S>);
            hooks.window_created = Some(ui_app_window_created::<S>);
            hooks.before_close_window = Some(ui_app_before_close_window::<S>);

            hooks.accessibility_snapshot = Some(ui_app_accessibility_snapshot::<S>);
            hooks.accessibility_focus = Some(ui_app_accessibility_focus::<S>);
            hooks.accessibility_invoke = Some(ui_app_accessibility_invoke::<S>);
            hooks.accessibility_set_value_text = Some(ui_app_accessibility_set_value_text::<S>);

            hooks.viewport_input = Some(ui_app_viewport_input::<S>);
            hooks.dock_op = Some(ui_app_dock_op::<S>);
            hooks.record_engine_frame = Some(ui_app_record_engine_frame::<S>);
        })
    }
}

pub struct UiAppWindowState<S> {
    pub ui: UiTree<App>,
    pub root: Option<NodeId>,
    pub state: S,
    pending_invalidation: PendingInvalidationBatch,
}

#[derive(Debug, Default)]
struct PendingInvalidationBatch {
    models: Vec<fret_app::ModelId>,
    models_seen: HashSet<fret_app::ModelId>,
    globals: Vec<std::any::TypeId>,
    globals_seen: HashSet<std::any::TypeId>,
}

impl PendingInvalidationBatch {
    fn push_models(&mut self, changed: &[fret_app::ModelId]) {
        for &id in changed {
            if self.models_seen.insert(id) {
                self.models.push(id);
            }
        }
    }

    fn push_globals(&mut self, changed: &[std::any::TypeId]) {
        for &id in changed {
            if self.globals_seen.insert(id) {
                self.globals.push(id);
            }
        }
    }

    fn flush(&mut self, app: &mut App, ui: &mut UiTree<App>) {
        if !self.models.is_empty() {
            ui.propagate_model_changes(app, &self.models);
            self.models.clear();
            self.models_seen.clear();
        }
        if !self.globals.is_empty() {
            ui.propagate_global_changes(app, &self.globals);
            self.globals.clear();
            self.globals_seen.clear();
        }
    }
}

#[cfg(feature = "ui-app-command-palette")]
#[derive(Debug, Clone)]
pub struct CommandPaletteModels {
    pub open: fret_app::Model<bool>,
    pub query: fret_app::Model<String>,
    gating_handle: Option<fret_runtime::WindowCommandGatingHandle>,
}

#[cfg(feature = "ui-app-command-palette")]
#[derive(Debug, Default)]
pub struct CommandPaletteService {
    by_window: HashMap<AppWindowId, CommandPaletteModels>,
}

#[cfg(feature = "ui-app-command-palette")]
impl CommandPaletteService {
    pub fn models(&self, window: AppWindowId) -> Option<CommandPaletteModels> {
        self.by_window.get(&window).cloned()
    }

    fn set_gating_handle(
        &mut self,
        window: AppWindowId,
        handle: Option<fret_runtime::WindowCommandGatingHandle>,
    ) {
        if let Some(models) = self.by_window.get_mut(&window) {
            models.gating_handle = handle;
        }
    }

    fn take_gating_handle(
        &mut self,
        window: AppWindowId,
    ) -> Option<fret_runtime::WindowCommandGatingHandle> {
        self.by_window
            .get_mut(&window)
            .and_then(|models| models.gating_handle.take())
    }

    fn ensure_window(&mut self, app: &mut App, window: AppWindowId) -> CommandPaletteModels {
        if let Some(existing) = self.by_window.get(&window) {
            return existing.clone();
        }

        let models = CommandPaletteModels {
            open: app.models_mut().insert(false),
            query: app.models_mut().insert(String::new()),
            gating_handle: None,
        };
        self.by_window.insert(window, models.clone());
        models
    }
}

#[cfg(feature = "ui-app-command-palette")]
fn command_palette_toggle(app: &mut App, window: AppWindowId) -> bool {
    let (next_open, prev_gating_handle) =
        app.with_global_mut(CommandPaletteService::default, |svc, app| {
            let models = svc.ensure_window(app, window);
            let is_open = app.models().get_copied(&models.open).unwrap_or(false);
            let next_open = !is_open;
            let _ = app.models_mut().update(&models.open, |v| *v = next_open);
            let _ = app.models_mut().update(&models.query, |v| v.clear());
            let prev_gating_handle = svc.take_gating_handle(window);
            (next_open, prev_gating_handle)
        });

    if let Some(handle) = prev_gating_handle {
        app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                let _ = svc.pop_snapshot(handle);
            },
        );
    }

    if next_open {
        let fallback_input_ctx = fret_ui_shadcn::command::command_palette_input_context(app);
        let snapshot = fret_runtime::best_effort_snapshot_for_window_with_input_ctx_fallback(
            app,
            window,
            fallback_input_ctx,
        );

        let mut input_ctx = snapshot.input_ctx().clone();
        input_ctx.ui_has_modal = true;
        input_ctx.focus_is_text_input = false;
        input_ctx.dispatch_phase = fret_runtime::InputDispatchPhase::Bubble;

        let handle = app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| svc.push_snapshot(window, snapshot.with_input_ctx(input_ctx)),
        );

        app.with_global_mut(CommandPaletteService::default, |svc, _app| {
            svc.set_gating_handle(window, Some(handle));
        });
    }

    app.request_redraw(window);
    next_open
}

#[cfg(feature = "ui-app-command-palette")]
fn command_palette_cleanup_gating_if_closed(app: &mut App, window: AppWindowId, open_now: bool) {
    if open_now {
        return;
    }

    let handle = app.with_global_mut(CommandPaletteService::default, |svc, _app| {
        svc.take_gating_handle(window)
    });
    if let Some(handle) = handle {
        app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                let _ = svc.pop_snapshot(handle);
            },
        );
    }
}

#[cfg(all(test, feature = "ui-app-command-palette"))]
mod command_palette_gating_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn command_palette_toggle_pushes_snapshot_and_cleanup_pops_when_closed() {
        let window = AppWindowId::default();
        let mut app = App::new();

        assert_eq!(command_palette_toggle(&mut app, window), true);
        assert!(
            app.global::<fret_runtime::WindowCommandGatingService>()
                .and_then(|svc| svc.snapshot(window))
                .is_some(),
            "expected command palette open to publish a gating snapshot"
        );

        let models = app
            .global::<CommandPaletteService>()
            .and_then(|svc| svc.models(window))
            .expect("command palette models");
        let _ = app.models_mut().update(&models.open, |v| *v = false);
        command_palette_cleanup_gating_if_closed(&mut app, window, false);

        assert!(
            app.global::<fret_runtime::WindowCommandGatingService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none(),
            "expected command palette close to pop its gating snapshot"
        );
    }

    #[test]
    fn command_palette_close_does_not_clear_other_pushed_overrides() {
        let window = AppWindowId::default();
        let mut app = App::new();

        assert_eq!(command_palette_toggle(&mut app, window), true);

        let other = app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                svc.push_snapshot(
                    window,
                    fret_runtime::WindowCommandGatingSnapshot::new(
                        fret_runtime::InputContext::default(),
                        HashMap::new(),
                    ),
                )
            },
        );

        assert_eq!(command_palette_toggle(&mut app, window), false);
        assert!(
            app.global::<fret_runtime::WindowCommandGatingService>()
                .and_then(|svc| svc.snapshot(window))
                .is_some(),
            "expected other pushed override to remain after command palette closes"
        );

        app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                let _ = svc.pop_snapshot(other);
            },
        );
        assert!(
            app.global::<fret_runtime::WindowCommandGatingService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none(),
            "expected window snapshot to be cleared after removing last override"
        );
    }
}

#[derive(Debug, Clone)]
struct PreferencesOverlayModels {
    open: Model<bool>,
}

#[derive(Debug, Default)]
struct PreferencesOverlayService {
    by_window: HashMap<AppWindowId, PreferencesOverlayModels>,
}

impl PreferencesOverlayService {
    fn models(&self, window: AppWindowId) -> Option<PreferencesOverlayModels> {
        self.by_window.get(&window).cloned()
    }

    fn ensure_window(&mut self, app: &mut App, window: AppWindowId) -> PreferencesOverlayModels {
        if let Some(existing) = self.by_window.get(&window) {
            return existing.clone();
        }

        let models = PreferencesOverlayModels {
            open: app.models_mut().insert(false),
        };
        self.by_window.insert(window, models.clone());
        models
    }
}

pub fn default_on_preferences<S>(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    _state: &mut S,
) {
    app.with_global_mut(PreferencesOverlayService::default, |svc, app| {
        let models = svc.ensure_window(app, window);
        let _ = app.models_mut().update(&models.open, |v| *v = true);
    });
    app.request_redraw(window);
}

fn drive_preferences_overlay(cx: &mut ElementContext<'_, App>) {
    let Some(models) = cx
        .app
        .global::<PreferencesOverlayService>()
        .and_then(|svc| svc.models(cx.window))
    else {
        return;
    };

    cx.observe_model(&models.open, Invalidation::Layout);
    let open_now = cx.app.models().get_copied(&models.open).unwrap_or(false);
    if !open_now {
        return;
    }

    let theme = Theme::global(&*cx.app).clone();
    let pad = theme.metric_by_key("metric.padding.md").unwrap_or(Px(16.0));
    let pad_sm = theme.metric_by_key("metric.padding.sm").unwrap_or(Px(12.0));
    let radius = theme.metric_by_key("metric.radius.md").unwrap_or(Px(8.0));
    let radius_sm = theme.metric_by_key("metric.radius.sm").unwrap_or(Px(6.0));

    let fg = theme.color_required("foreground");
    let muted_fg = theme.color_by_key("muted-foreground").unwrap_or(fg);
    let card = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or(fg);
    let muted = theme.color_by_key("muted").unwrap_or(card);
    let border = theme.color_by_key("border").unwrap_or(muted_fg);

    let config_paths = fret_app::config_files::LayeredConfigPaths::for_project_root(".");

    let file_rows = [
        (
            "Project settings.json",
            Some(config_paths.project_settings_json().display().to_string()),
        ),
        (
            "User settings.json",
            config_paths
                .user_settings_json()
                .map(|p| p.display().to_string()),
        ),
        (
            "Project keymap.json",
            Some(config_paths.project_keymap_json().display().to_string()),
        ),
        (
            "User keymap.json",
            config_paths
                .user_keymap_json()
                .map(|p| p.display().to_string()),
        ),
        (
            "Project menubar.json",
            Some(config_paths.project_menubar_json().display().to_string()),
        ),
        (
            "User menubar.json",
            config_paths
                .user_menubar_json()
                .map(|p| p.display().to_string()),
        ),
    ];

    let close_button = {
        let open = models.open.clone();
        cx.pressable(
            fret_ui::element::PressableProps {
                focusable: true,
                a11y: fret_ui::element::PressableA11y {
                    label: Some(std::sync::Arc::from("Close preferences")),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st| {
                cx.pressable_add_on_activate(std::sync::Arc::new(move |host, action_cx, _| {
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(action_cx.window);
                }));

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        padding: Edges::all(pad_sm),
                        background: Some(muted),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(border),
                        corner_radii: Corners::all(radius_sm),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.text_props(fret_ui::element::TextProps {
                            layout: Default::default(),
                            text: std::sync::Arc::from("Close"),
                            style: None,
                            color: Some(fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })]
                    },
                )]
            },
        )
    };

    let watcher_status = cx
        .app
        .global::<fret_app::ConfigFilesWatcherStatus>()
        .cloned();
    let watcher_text = watcher_status
        .as_ref()
        .and_then(|s| s.last_tick().map(|t| (s.seq(), t)))
        .map(|(seq, tick)| {
            format!(
                "Watcher seq={} reloaded: settings={} keymap={} menubar={}",
                seq, tick.reloaded_settings, tick.reloaded_keymap, tick.reloaded_menu_bar
            )
        })
        .unwrap_or_else(|| "Watcher not installed (or no ticks yet).".to_string());

    let barrier_bg = cx.container(
        fret_ui::element::ContainerProps {
            layout: dialog_prim::modal_barrier_layout(),
            background: Some(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            }),
            ..Default::default()
        },
        |_cx| Vec::new(),
    );

    let content = cx.flex(
        fret_ui::element::FlexProps {
            layout: dialog_prim::modal_barrier_layout(),
            direction: fret_core::Axis::Vertical,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: fret_ui::element::MainAlign::Center,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        |cx| {
            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Px(Px(640.0));
                        layout.size.height = fret_ui::element::Length::Auto;
                        layout
                    },
                    padding: Edges::all(pad),
                    background: Some(card),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |cx| {
                    let header = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: fret_ui::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(12.0),
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::SpaceBetween,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: std::sync::Arc::from("Preferences"),
                                    style: None,
                                    color: Some(fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                close_button,
                            ]
                        },
                    );

                    let project_dir = config_paths.project_dir.display().to_string();
                    let user_dir = config_paths
                        .user_dir
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<none>".to_string());

                    let dirs = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: fret_ui::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Vertical,
                            gap: Px(6.0),
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: std::sync::Arc::from(format!(
                                        "Project config dir: {project_dir}"
                                    )),
                                    style: None,
                                    color: Some(muted_fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: std::sync::Arc::from(format!(
                                        "User config dir: {user_dir}"
                                    )),
                                    style: None,
                                    color: Some(muted_fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                            ]
                        },
                    );

                    let files = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: fret_ui::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Vertical,
                            gap: Px(10.0),
                            padding: Edges::all(Px(0.0)),
                            justify: fret_ui::element::MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            let mut out = Vec::new();
                            for (label, path) in file_rows {
                                let Some(path) = path else {
                                    continue;
                                };
                                let text_for_copy = path.clone();
                                let row = cx.flex(
                                    fret_ui::element::FlexProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(12.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: fret_ui::element::MainAlign::SpaceBetween,
                                        align: fret_ui::element::CrossAlign::Center,
                                        wrap: false,
                                    },
                                    |cx| {
                                        let left = cx.flex(
                                            fret_ui::element::FlexProps {
                                                layout: {
                                                    let mut layout =
                                                        fret_ui::element::LayoutStyle::default();
                                                    layout.flex.grow = 1.0;
                                                    layout.flex.shrink = 1.0;
                                                    layout.flex.basis =
                                                        fret_ui::element::Length::Px(Px(0.0));
                                                    layout
                                                },
                                                direction: fret_core::Axis::Vertical,
                                                gap: Px(2.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: fret_ui::element::MainAlign::Start,
                                                align: fret_ui::element::CrossAlign::Stretch,
                                                wrap: false,
                                            },
                                            |cx| {
                                                vec![
                                                    cx.text_props(fret_ui::element::TextProps {
                                                        layout: Default::default(),
                                                        text: std::sync::Arc::from(label),
                                                        style: None,
                                                        color: Some(fg),
                                                        wrap: TextWrap::None,
                                                        overflow: TextOverflow::Clip,
                                                    }),
                                                    cx.text_props(fret_ui::element::TextProps {
                                                        layout: Default::default(),
                                                        text: std::sync::Arc::from(path),
                                                        style: None,
                                                        color: Some(muted_fg),
                                                        wrap: TextWrap::None,
                                                        overflow: TextOverflow::Clip,
                                                    }),
                                                ]
                                            },
                                        );
                                        let copy = cx.pressable(
                                            fret_ui::element::PressableProps {
                                                focusable: true,
                                                a11y: fret_ui::element::PressableA11y {
                                                    label: Some(std::sync::Arc::from(
                                                        "Copy config path",
                                                    )),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            move |cx, _st| {
                                                cx.pressable_add_on_activate(std::sync::Arc::new(
                                                    move |host, _action_cx, _| {
                                                        host.push_effect(
                                                            Effect::ClipboardSetText {
                                                                text: text_for_copy.clone(),
                                                            },
                                                        );
                                                    },
                                                ));
                                                vec![cx.container(
                                                    fret_ui::element::ContainerProps {
                                                        padding: Edges::all(pad_sm),
                                                        background: Some(muted),
                                                        border: Edges::all(Px(1.0)),
                                                        border_color: Some(border),
                                                        corner_radii: Corners::all(radius_sm),
                                                        ..Default::default()
                                                    },
                                                    move |cx| {
                                                        vec![cx.text_props(
                                                            fret_ui::element::TextProps {
                                                                layout: Default::default(),
                                                                text: std::sync::Arc::from("Copy"),
                                                                style: None,
                                                                color: Some(fg),
                                                                wrap: TextWrap::None,
                                                                overflow: TextOverflow::Clip,
                                                            },
                                                        )]
                                                    },
                                                )]
                                            },
                                        );

                                        vec![left, copy]
                                    },
                                );
                                out.push(row);
                            }
                            out
                        },
                    );

                    let watcher = cx.container(
                        fret_ui::element::ContainerProps {
                            padding: Edges::all(pad_sm),
                            background: Some(muted),
                            border: Edges::all(Px(1.0)),
                            border_color: Some(border),
                            corner_radii: Corners::all(radius_sm),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.text_props(fret_ui::element::TextProps {
                                layout: Default::default(),
                                text: std::sync::Arc::from(watcher_text.clone()),
                                style: None,
                                color: Some(muted_fg),
                                wrap: TextWrap::Word,
                                overflow: TextOverflow::Clip,
                            })]
                        },
                    );

                    vec![
                        header,
                        cx.spacer(fret_ui::element::SpacerProps {
                            min: Px(12.0),
                            ..Default::default()
                        }),
                        dirs,
                        cx.spacer(fret_ui::element::SpacerProps {
                            min: Px(12.0),
                            ..Default::default()
                        }),
                        files,
                        cx.spacer(fret_ui::element::SpacerProps {
                            min: Px(12.0),
                            ..Default::default()
                        }),
                        watcher,
                    ]
                },
            )]
        },
    );

    let open = models.open.clone();
    let children = dialog_prim::modal_dialog_layer_elements(
        cx,
        open.clone(),
        dialog_prim::DialogOptions::default(),
        [barrier_bg],
        content,
    );

    let mut req = fret_ui_kit::OverlayRequest::modal(
        fret_ui::elements::GlobalElementId(0x8f31_7a1f_4b27_1d01),
        None,
        open,
        fret_ui_kit::OverlayPresence::instant(true),
        children.into_vec(),
    );
    req.root_name = Some("bootstrap.preferences".to_string());
    OverlayController::request(cx, req);
}

fn hotpatch_trace_enabled() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }

    std::env::var_os("FRET_HOTPATCH_DIAG").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_HOTPATCH").is_some_and(|v| !v.is_empty())
        || std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty())
}

fn hotpatch_trace_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();
    paths.push(std::path::Path::new(".fret").join("hotpatch_bootstrap.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("hotpatch_bootstrap.log"));
    }
    paths.into_iter()
}

fn hotpatch_trace_log(line: &str) {
    if !hotpatch_trace_enabled() {
        return;
    }

    use std::io::Write as _;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    for path in hotpatch_trace_paths() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(msg.as_bytes());
            let _ = file.flush();
        }
    }
}

#[cfg(all(windows, feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_module_path_for_address(addr: usize) -> Option<std::path::PathBuf> {
    if addr == 0 {
        return None;
    }

    unsafe {
        use std::ffi::c_void;

        #[allow(non_snake_case)]
        unsafe extern "system" {
            fn GetModuleHandleExA(
                dwFlags: u32,
                lpModuleName: *const i8,
                phModule: *mut *mut c_void,
            ) -> i32;
            fn GetModuleFileNameA(hModule: *mut c_void, lpFilename: *mut u8, nSize: u32) -> u32;
        }

        const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x0000_0002;
        const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x0000_0004;

        let mut module: *mut c_void = std::ptr::null_mut();
        let ok = GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            addr as *const i8,
            &mut module as *mut _,
        );
        if ok == 0 || module.is_null() {
            return None;
        }

        let mut buf = vec![0u8; 4096];
        let len = GetModuleFileNameA(module, buf.as_mut_ptr(), buf.len() as u32);
        if len == 0 {
            return None;
        }
        buf.truncate(len as usize);
        Some(std::path::PathBuf::from(
            String::from_utf8_lossy(&buf).to_string(),
        ))
    }
}

#[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_head16(addr: usize) -> Option<[u8; 16]> {
    if addr == 0 {
        return None;
    }

    unsafe {
        let bytes = std::slice::from_raw_parts(addr as *const u8, 16);
        let mut out = [0u8; 16];
        out.copy_from_slice(bytes);
        Some(out)
    }
}

#[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_head_bytes(addr: usize, len: usize) -> Option<String> {
    if addr == 0 || len == 0 {
        return None;
    }

    unsafe {
        let bytes = std::slice::from_raw_parts(addr as *const u8, len);
        let mut out = String::new();
        for (i, b) in bytes.iter().copied().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            use std::fmt::Write as _;
            let _ = write!(out, "{:02x}", b);
        }
        Some(out)
    }
}

#[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_call_target_from_head16(addr: usize, head16: &[u8; 16]) -> Option<usize> {
    if addr == 0 {
        return None;
    }
    if head16[0] != 0x55 || head16[1] != 0xB8 || head16[6] != 0xE8 {
        return None;
    }

    let rel = i32::from_le_bytes([head16[7], head16[8], head16[9], head16[10]]) as isize;
    let next = (addr as isize).checked_add(11)?;
    let target = next.checked_add(rel)?;
    if target <= 0 {
        return None;
    }
    Some(target as usize)
}

#[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_abs_jmp_target_from_head16(head16: &[u8; 16]) -> Option<usize> {
    if head16[0] != 0x48 || head16[1] != 0xB8 || head16[10] != 0xFF || head16[11] != 0xE0 {
        return None;
    }
    let imm = u64::from_le_bytes([
        head16[2], head16[3], head16[4], head16[5], head16[6], head16[7], head16[8], head16[9],
    ]);
    if imm == 0 {
        return None;
    }
    Some(imm as usize)
}

fn ui_app_create_window_state<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
) -> UiAppWindowState<S> {
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(
        std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()),
    );

    #[cfg(feature = "ui-app-command-palette")]
    if driver.command_palette_enabled {
        app.with_global_mut(CommandPaletteService::default, |svc, app| {
            svc.ensure_window(app, window);
        });
    }

    let state = {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(driver.init_window);
            hot.call((app, window))
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            (driver.init_window)(app, window)
        }
    };
    UiAppWindowState {
        ui,
        root: None,
        state,
        pending_invalidation: PendingInvalidationBatch::default(),
    }
}

fn ui_app_handle_event<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitEventContext<'_, UiAppWindowState<S>>,
    event: &Event,
) {
    let WinitEventContext {
        app,
        services,
        window,
        state,
    } = context;

    if let Event::Timer { token } = event
        && let Some(tick) = fret_app::handle_config_files_watcher_timer(app, window, *token)
    {
        let actionable = tick.reloaded_settings
            || tick.reloaded_keymap
            || tick.reloaded_menu_bar
            || tick.settings_error.is_some()
            || tick.keymap_error.is_some()
            || tick.menu_bar_error.is_some()
            || tick.actionable_keymap_conflicts > 0;

        if actionable {
            app.with_global_mut(fret_app::ConfigFilesWatcherStatus::default, |svc, _app| {
                svc.note(tick.clone());
            });
            app.request_redraw(window);

            hotpatch_trace_log(&format!(
                "config_watcher: window={window:?} settings_reload={} keymap_reload={} menubar_reload={} settings_err={:?} keymap_err={:?} menubar_err={:?} conflicts={} samples={:?}",
                tick.reloaded_settings,
                tick.reloaded_keymap,
                tick.reloaded_menu_bar,
                tick.settings_error,
                tick.keymap_error,
                tick.menu_bar_error,
                tick.actionable_keymap_conflicts,
                tick.keymap_conflict_samples,
            ));
        }
        return;
    }

    #[cfg(feature = "diagnostics")]
    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
        svc.record_event(app, window, event);
    });

    #[cfg(feature = "diagnostics")]
    if app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
        svc.maybe_intercept_event_for_picking(app, window, event)
    }) {
        return;
    }

    #[cfg(feature = "diagnostics")]
    if app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
        svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event)
    }) {
        return;
    }

    state.ui.dispatch_event(app, services, event);

    #[cfg(feature = "ui-assets")]
    if driver.drive_ui_assets {
        let _ = fret_ui_assets::UiAssets::handle_event(app, window, event);
    }

    if let Some(on_event) = driver.on_event {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(on_event);
            hot.call((
                app,
                services,
                window,
                &mut state.ui,
                &mut state.state,
                event,
            ));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            on_event(
                app,
                services,
                window,
                &mut state.ui,
                &mut state.state,
                event,
            );
        }
    }

    if driver.close_on_window_close_requested && matches!(event, Event::WindowCloseRequested) {
        app.push_effect(Effect::Window(fret_app::WindowRequest::Close(window)));
    }
}

fn ui_app_handle_command<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitCommandContext<'_, UiAppWindowState<S>>,
    command: CommandId,
) {
    let WinitCommandContext {
        app,
        services,
        window,
        state,
    } = context;

    #[cfg(feature = "ui-app-command-palette")]
    if driver.command_palette_enabled
        && matches!(
            command.as_str(),
            "app.command_palette" | "command_palette.toggle"
        )
    {
        let _ = command_palette_toggle(app, window);
        return;
    }

    if state.ui.dispatch_command(app, services, &command) {
        return;
    }

    match command.as_str() {
        fret_app::core_commands::APP_ABOUT => {
            #[cfg(target_os = "macos")]
            {
                app.push_effect(Effect::ShowAboutPanel);
                return;
            }
        }
        fret_app::core_commands::APP_PREFERENCES => {
            if let Some(f) = driver.on_preferences {
                f(app, services, window, &mut state.ui, &mut state.state);
                return;
            }
        }
        fret_app::core_commands::APP_QUIT => {
            app.push_effect(Effect::QuitApp);
            return;
        }
        fret_app::core_commands::APP_HIDE => {
            app.push_effect(Effect::HideApp);
            return;
        }
        fret_app::core_commands::APP_HIDE_OTHERS => {
            app.push_effect(Effect::HideOtherApps);
            return;
        }
        fret_app::core_commands::APP_SHOW_ALL => {
            app.push_effect(Effect::UnhideAllApps);
            return;
        }
        _ => {}
    }

    if fret_ui_kit::try_handle_window_overlays_command(&mut state.ui, app, window, &command) {
        return;
    }

    if let Some(on_command) = driver.on_command {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(on_command);
            hot.call((
                app,
                services,
                window,
                &mut state.ui,
                &mut state.state,
                &command,
            ));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            on_command(
                app,
                services,
                window,
                &mut state.ui,
                &mut state.state,
                &command,
            );
        }
    }
}

fn ui_app_handle_global_command<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitGlobalContext<'_>,
    command: CommandId,
) {
    let WinitGlobalContext { app, services } = context;

    match command.as_str() {
        fret_app::core_commands::APP_ABOUT => {
            #[cfg(target_os = "macos")]
            {
                app.push_effect(Effect::ShowAboutPanel);
                return;
            }
        }
        fret_app::core_commands::APP_QUIT => {
            app.push_effect(Effect::QuitApp);
            return;
        }
        fret_app::core_commands::APP_HIDE => {
            app.push_effect(Effect::HideApp);
            return;
        }
        fret_app::core_commands::APP_HIDE_OTHERS => {
            app.push_effect(Effect::HideOtherApps);
            return;
        }
        fret_app::core_commands::APP_SHOW_ALL => {
            app.push_effect(Effect::UnhideAllApps);
            return;
        }
        _ => {}
    }

    if let Some(f) = driver.handle_global_command {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, services, command));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, services, command);
        }
    }
}

fn ui_app_handle_model_changes<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitWindowContext<'_, UiAppWindowState<S>>,
    changed: &[fret_app::ModelId],
) {
    let WinitWindowContext {
        app, window, state, ..
    } = context;

    #[cfg(feature = "diagnostics")]
    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        svc.record_model_changes(window, changed);
    });

    state.pending_invalidation.push_models(changed);
    if !changed.is_empty() {
        app.request_redraw(window);
    }
    if let Some(f) = driver.on_model_changes {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, window, &mut state.ui, &mut state.state, changed));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, window, &mut state.ui, &mut state.state, changed);
        }
    }
}

fn ui_app_handle_global_changes<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitWindowContext<'_, UiAppWindowState<S>>,
    changed: &[std::any::TypeId],
) {
    let WinitWindowContext {
        app, window, state, ..
    } = context;

    #[cfg(feature = "diagnostics")]
    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
        svc.record_global_changes(app, window, changed);
    });

    state.pending_invalidation.push_globals(changed);
    if !changed.is_empty() {
        app.request_redraw(window);
    }
    if let Some(f) = driver.on_global_changes {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, window, &mut state.ui, &mut state.state, changed));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, window, &mut state.ui, &mut state.state, changed);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FrameHitchConfig {
    hitch_ms: u64,
}

fn frame_hitch_config() -> Option<FrameHitchConfig> {
    static CONFIG: OnceLock<Option<FrameHitchConfig>> = OnceLock::new();
    *CONFIG.get_or_init(|| {
        let enabled = std::env::var_os("FRET_FRAME_HITCH_LOG").is_some_and(|v| !v.is_empty());
        if !enabled {
            return None;
        }

        let hitch_ms = std::env::var("FRET_FRAME_HITCH_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        Some(FrameHitchConfig { hitch_ms })
    })
}

fn frame_hitch_log_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();
    paths.push(std::path::Path::new(".fret").join("frame_hitches.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("frame_hitches.log"));
    }
    paths.into_iter()
}

fn write_frame_hitch_log(line: &str) {
    use std::io::Write as _;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    for path in frame_hitch_log_paths() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(msg.as_bytes());
            let _ = file.flush();
        }
    }
}

fn ui_app_render<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitRenderContext<'_, UiAppWindowState<S>>,
) {
    thread_local! {
        static RENDER_DEPTH: Cell<u32> = const { Cell::new(0) };
        static VIEW_DEPTH: Cell<u32> = const { Cell::new(0) };
    }

    let WinitRenderContext {
        app,
        services,
        window,
        state,
        bounds,
        scale_factor,
        scene,
    } = context;

    #[cfg(feature = "tracing")]
    let frame_span = tracing::info_span!(
        "fret.frame",
        window = ?window,
        tick_id = app.tick_id().0,
        frame_id = app.frame_id().0,
        bounds = ?bounds,
        scale_factor = scale_factor,
    );
    #[cfg(feature = "tracing")]
    let _frame_guard = frame_span.enter();

    let hitch_config = frame_hitch_config();
    let hitch_total_started = hitch_config.map(|_| Instant::now());
    let mut hitch_view_ms: Option<u64> = None;
    let mut hitch_overlay_ms: Option<u64> = None;
    let mut hitch_paint_ms: Option<u64> = None;

    // Note: diagnostics may enable inspection mode (disables caching) on demand.

    let render_depth = RENDER_DEPTH.with(|d| {
        let next = d.get().saturating_add(1);
        d.set(next);
        next
    });
    hotpatch_trace_log(&format!(
        "ui_app_render: begin window={window:?} depth={render_depth}"
    ));

    OverlayController::begin_frame(app, window);
    hotpatch_trace_log(&format!(
        "ui_app_render: after begin_frame window={window:?}"
    ));

    #[cfg(feature = "diagnostics")]
    {
        // Ensure optional diagnostics stores exist before layout/paint so ecosystem crates can
        // publish frame-local records without allocating globals in production runs.
        let enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if enabled {
            app.with_global_mut_untracked(
                fret_runtime::WindowInteractionDiagnosticsStore::default,
                |store, app| store.begin_frame(window, app.frame_id()),
            );
        }
    }

    let view_started = hitch_config.map(|_| Instant::now());
    #[cfg(feature = "tracing")]
    let view_span = tracing::info_span!("fret.ui.view");
    #[cfg(feature = "tracing")]
    let _view_guard = view_span.enter();
    let root = RenderRootContext::new(&mut state.ui, app, services, window, bounds).render_root(
        driver.root_name,
        |cx| {
            let view_depth = VIEW_DEPTH.with(|d| {
                let next = d.get().saturating_add(1);
                d.set(next);
                next
            });
            if view_depth >= 8 {
                hotpatch_trace_log(&format!(
                    "ui_app_render: entering view window={window:?} depth={view_depth}"
                ));
            }
            hotpatch_trace_log(&format!(
                "ui_app_render: view begin window={window:?} depth={view_depth}"
            ));

            // Install a Radix-style direction provider for the whole app subtree.
            //
            // Apps may override this by setting `LayoutDirection` as a global; otherwise we
            // default to LTR (matching Radix `useDirection` default).
            let dir = cx
                .app
                .global::<LayoutDirection>()
                .copied()
                .unwrap_or_default();

            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let view_ptr = driver.view as usize as u64;
                let mapped = unsafe {
                    subsecond::get_jump_table().and_then(|table| table.map.get(&view_ptr).cloned())
                };
                hotpatch_trace_log(&format!(
                    "ui_app_render: view ptr=0x{view_ptr:x} mapped={mapped:?}"
                ));
                #[cfg(windows)]
                {
                    let view_module =
                        hotpatch_module_path_for_address(view_ptr as usize).map(|p| p.display().to_string());
                    let mapped_module = mapped
                        .and_then(|p| hotpatch_module_path_for_address(p as usize))
                        .map(|p| p.display().to_string());
                    hotpatch_trace_log(&format!(
                        "ui_app_render: view module={view_module:?} mapped_module={mapped_module:?}"
                    ));
                }
                let byte_diag = std::env::var_os("FRET_HOTPATCH_DIAG_BYTES")
                    .is_some_and(|v| !v.is_empty());
                if byte_diag {
                    let view_head = hotpatch_head_bytes(view_ptr as usize, 16);
                    let mapped_head = mapped.and_then(|p| hotpatch_head_bytes(p as usize, 16));
                    hotpatch_trace_log(&format!(
                        "ui_app_render: view head16={view_head:?} mapped_head16={mapped_head:?}"
                    ));

                    #[cfg(windows)]
                    if let Some(mapped_addr) = mapped {
                        if let Some(head) = hotpatch_head16(mapped_addr as usize) {
                            if let Some(target) =
                                hotpatch_call_target_from_head16(mapped_addr as usize, &head)
                            {
                                let target_module = hotpatch_module_path_for_address(target)
                                    .map(|p| p.display().to_string());
                                let target_head16 = hotpatch_head_bytes(target, 16);
                                hotpatch_trace_log(&format!(
                                    "ui_app_render: mapped prologue call_target=0x{target:x} target_module={target_module:?} target_head16={target_head16:?}"
                                ));

                                if let Some(target_head) = hotpatch_head16(target) {
                                    if let Some(abs) =
                                        hotpatch_abs_jmp_target_from_head16(&target_head)
                                    {
                                        let abs_module = hotpatch_module_path_for_address(abs)
                                            .map(|p| p.display().to_string());
                                        let abs_head16 = hotpatch_head_bytes(abs, 16);
                                        hotpatch_trace_log(&format!(
                                            "ui_app_render: call_target abs_jmp=0x{abs:x} abs_module={abs_module:?} abs_head16={abs_head16:?}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                let use_direct = std::env::var_os("FRET_HOTPATCH_VIEW_CALL_DIRECT")
                    .is_some_and(|v| !v.is_empty());
                hotpatch_trace_log(&format!(
                    "ui_app_render: view call strategy={}",
                    if use_direct { "direct" } else { "hotfn" }
                ));

                let out = direction_prim::with_direction_provider(cx, dir, |cx| {
                    let mut out = if use_direct {
                        (driver.view)(cx, &mut state.state)
                    } else {
                        let mut hot = subsecond::HotFn::current(driver.view);
                        hot.call((cx, &mut state.state))
                    };

                    #[cfg(feature = "ui-app-command-palette")]
                    if driver.command_palette_enabled
                        && let Some(models) = cx
                            .app
                            .global::<CommandPaletteService>()
                            .and_then(|svc| svc.models(cx.window))
                    {
                        let open_now = cx
                            .app
                            .models()
                            .get_copied(&models.open)
                            .unwrap_or(false);
                        command_palette_cleanup_gating_if_closed(cx.app, cx.window, open_now);
                        let entries = if open_now {
                            fret_ui_shadcn::command::command_entries_from_host_commands_with_options(
                                cx,
                                fret_ui_shadcn::command::CommandCatalogOptions::default(),
                            )
                        } else {
                            Vec::new()
                        };

                        let dialog = fret_ui_shadcn::CommandDialog::new(
                            models.open,
                            models.query,
                            Vec::new(),
                        )
                        .entries(entries)
                        .a11y_label("Command palette")
                        .into_element(cx, |cx| {
                            cx.interactivity_gate_props(
                                fret_ui::element::InteractivityGateProps {
                                    present: false,
                                    interactive: false,
                                    ..Default::default()
                                },
                                |_| vec![],
                            )
                        });
                        out.push(dialog);
                    }

                    drive_preferences_overlay(cx);
                    out
                });
                hotpatch_trace_log(&format!(
                    "ui_app_render: view end window={window:?} depth={view_depth}"
                ));
                VIEW_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                out
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                let out = direction_prim::with_direction_provider(cx, dir, |cx| {
                    let out = (driver.view)(cx, &mut state.state);

                    #[cfg(feature = "ui-app-command-palette")]
                    let mut out = out;

                    #[cfg(feature = "ui-app-command-palette")]
                    if driver.command_palette_enabled
                        && let Some(models) = cx
                            .app
                            .global::<CommandPaletteService>()
                            .and_then(|svc| svc.models(cx.window))
                    {
                        let open_now = cx
                            .app
                            .models()
                            .get_copied(&models.open)
                            .unwrap_or(false);
                        command_palette_cleanup_gating_if_closed(cx.app, cx.window, open_now);
                        let entries = if open_now {
                            fret_ui_shadcn::command::command_entries_from_host_commands_with_options(
                                cx,
                                fret_ui_shadcn::command::CommandCatalogOptions::default(),
                            )
                        } else {
                            Vec::new()
                        };

                        let dialog = fret_ui_shadcn::CommandDialog::new(
                            models.open,
                            models.query,
                            Vec::new(),
                        )
                        .entries(entries)
                        .a11y_label("Command palette")
                        .into_element(cx, |cx| {
                            cx.interactivity_gate_props(
                                fret_ui::element::InteractivityGateProps {
                                    present: false,
                                    interactive: false,
                                    ..Default::default()
                                },
                                |_| vec![],
                            )
                        });
                        out.push(dialog);
                    }

                    drive_preferences_overlay(cx);
                    out
                });
                hotpatch_trace_log(&format!(
                    "ui_app_render: view end window={window:?} depth={view_depth}"
                ));
                VIEW_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                out
            }
        },
    );
    if let Some(started) = view_started {
        hitch_view_ms = Some(started.elapsed().as_millis() as u64);
    }
    hotpatch_trace_log(&format!(
        "ui_app_render: after render_root window={window:?} root={root:?}"
    ));
    state.ui.set_root(root);
    hotpatch_trace_log(&format!("ui_app_render: after set_root window={window:?}"));

    let overlay_started = hitch_config.map(|_| Instant::now());
    #[cfg(feature = "tracing")]
    let overlay_span = tracing::info_span!("fret.ui.overlay");
    #[cfg(feature = "tracing")]
    let _overlay_guard = overlay_span.enter();
    OverlayController::render(&mut state.ui, app, services, window, bounds);
    if let Some(started) = overlay_started {
        hitch_overlay_ms = Some(started.elapsed().as_millis() as u64);
    }
    hotpatch_trace_log(&format!(
        "ui_app_render: after overlay render window={window:?}"
    ));

    state.pending_invalidation.flush(app, &mut state.ui);

    #[cfg(feature = "diagnostics")]
    {
        let diag_inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(diag_inspection_active);
        render_diag_inspect_overlay(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            diag_inspection_active,
        );
    }
    state.root = Some(root);

    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();
    let layout_started = hitch_config.map(|_| Instant::now());
    {
        #[cfg(feature = "tracing")]
        let layout_span = tracing::info_span!("fret.ui.layout");
        #[cfg(feature = "tracing")]
        let _layout_guard = layout_span.enter();
        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
    }
    let mut layout_total_ms: Option<u64> = layout_started.map(|s| s.elapsed().as_millis() as u64);
    hotpatch_trace_log(&format!(
        "ui_app_render: after layout_all window={window:?}"
    ));

    #[cfg(feature = "diagnostics")]
    {
        let semantics_snapshot = state.ui.semantics_snapshot();
        #[cfg(feature = "tracing")]
        let diag_span = tracing::info_span!("fret.ui.diagnostics.drive_script");
        #[cfg(feature = "tracing")]
        let _diag_guard = diag_span.enter();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                semantics_snapshot,
                element_runtime,
            )
        });
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
            ui_app_handle_event(
                driver,
                WinitEventContext {
                    app,
                    services,
                    window,
                    state,
                },
                &event,
            );
        }

        if injected_any {
            state.ui.request_semantics_snapshot();

            let relayout_started = hitch_config.map(|_| Instant::now());
            {
                #[cfg(feature = "tracing")]
                let relayout_span = tracing::info_span!("fret.ui.layout.relayout_after_script");
                #[cfg(feature = "tracing")]
                let _relayout_guard = relayout_span.enter();
                let mut frame =
                    UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
                frame.layout_all();
            }
            if let Some(started) = relayout_started {
                layout_total_ms =
                    Some(layout_total_ms.unwrap_or(0) + started.elapsed().as_millis() as u64);
            }
        }
    }

    let hitch_layout_ms = layout_total_ms;

    let paint_started = hitch_config.map(|_| Instant::now());
    {
        #[cfg(feature = "tracing")]
        let paint_span = tracing::info_span!("fret.ui.paint");
        #[cfg(feature = "tracing")]
        let _paint_guard = paint_span.enter();
        let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);
    }
    if let Some(started) = paint_started {
        hitch_paint_ms = Some(started.elapsed().as_millis() as u64);
    }
    hotpatch_trace_log(&format!("ui_app_render: after paint_all window={window:?}"));

    #[cfg(feature = "diagnostics")]
    {
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
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
            if let Some(dir) = svc.maybe_dump_if_triggered() {
                #[cfg(feature = "tracing")]
                tracing::info!(window = ?window, out_dir = %dir.display(), "ui diagnostics dumped");
            }
            if svc.poll_exit_trigger() {
                app.push_effect(Effect::QuitApp);
            } else if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
    }

    if let (Some(cfg), Some(started)) = (hitch_config, hitch_total_started) {
        let total = started.elapsed();
        let total_ms = total.as_millis() as u64;
        if total_ms >= cfg.hitch_ms {
            write_frame_hitch_log(&format!(
                "frame hitch window={window:?} total_ms={total_ms} view_ms={view_ms:?} overlay_ms={overlay_ms:?} layout_ms={layout_ms:?} paint_ms={paint_ms:?} scene_ops={ops} bounds={bounds:?} scale_factor={scale_factor}",
                view_ms = hitch_view_ms,
                overlay_ms = hitch_overlay_ms,
                layout_ms = hitch_layout_ms,
                paint_ms = hitch_paint_ms,
                ops = scene.ops_len(),
            ));

            #[cfg(feature = "tracing")]
            tracing::warn!(
                window = ?window,
                total_ms,
                view_ms = hitch_view_ms,
                overlay_ms = hitch_overlay_ms,
                layout_ms = hitch_layout_ms,
                paint_ms = hitch_paint_ms,
                scene_ops = scene.ops_len(),
                bounds = ?bounds,
                scale_factor,
                "frame hitch"
            );
        }
    }

    hotpatch_trace_log(&format!(
        "ui_app_render: end window={window:?} depth={render_depth}"
    ));
    RENDER_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
}

#[cfg(feature = "diagnostics")]
fn render_diag_inspect_overlay(
    ui: &mut fret_ui::UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: fret_core::Rect,
    inspection_active: bool,
) {
    use slotmap::Key as _;

    if !inspection_active {
        return;
    }

    const ROOT_NAME: &str = "__diag_inspect";

    let (
        pointer_pos,
        picked_node_id,
        focus_node_id,
        redact_text,
        pick_armed,
        inspect_enabled,
        consume_clicks,
        locked,
    ) = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        (
            svc.last_pointer_position(window),
            svc.last_picked_node_id(window),
            svc.inspect_focus_node_id(window),
            svc.redact_text(),
            svc.pick_is_armed(),
            svc.inspect_is_enabled(),
            svc.inspect_consume_clicks(),
            svc.inspect_is_locked(window),
        )
    });

    struct InspectNodeInfo {
        bounds: fret_core::Rect,
        role: fret_core::SemanticsRole,
        node_id: u64,
        test_id: Option<String>,
        label: Option<String>,
    }

    let snapshot = ui.semantics_snapshot();
    let hovered = pointer_pos
        .and_then(|pos| {
            snapshot
                .and_then(|snap| crate::ui_diagnostics::pick_semantics_node_by_bounds(snap, pos))
        })
        .map(|node| InspectNodeInfo {
            bounds: node.bounds,
            role: node.role,
            node_id: node.id.data().as_ffi(),
            test_id: node.test_id.clone(),
            label: node.label.clone(),
        });

    let picked = picked_node_id
        .and_then(|id| {
            snapshot.and_then(|snap| snap.nodes.iter().find(|n| n.id.data().as_ffi() == id))
        })
        .map(|node| InspectNodeInfo {
            bounds: node.bounds,
            role: node.role,
            node_id: node.id.data().as_ffi(),
            test_id: node.test_id.clone(),
            label: node.label.clone(),
        });

    let focus = focus_node_id
        .and_then(|id| {
            snapshot.and_then(|snap| snap.nodes.iter().find(|n| n.id.data().as_ffi() == id))
        })
        .map(|node| InspectNodeInfo {
            bounds: node.bounds,
            role: node.role,
            node_id: node.id.data().as_ffi(),
            test_id: node.test_id.clone(),
            label: node.label.clone(),
        });

    let hovered = if locked { None } else { hovered };

    let (toast, best_selector) =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            (
                svc.inspect_toast_message(window).map(|s| s.to_string()),
                svc.inspect_best_selector_json(window)
                    .map(|s| s.to_string()),
            )
        });

    let (focus_summary, focus_path) =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            (
                svc.inspect_focus_summary_line(window)
                    .map(|s| s.to_string()),
                svc.inspect_focus_path_line(window).map(|s| s.to_string()),
            )
        });

    let present = pick_armed
        || inspect_enabled
        || toast.is_some()
        || best_selector.is_some()
        || focus.is_some()
        || hovered.is_some()
        || picked.is_some();

    let root_node = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        ROOT_NAME,
        move |cx| {
            if !present {
                return Vec::new();
            }

            use fret_core::{Color, Corners, Edges, Px};
            use fret_ui::element::{
                ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, SizeStyle,
            };

            let mut children = Vec::new();

            if pick_armed || inspect_enabled {
                let mut layout = LayoutStyle::default();
                layout.position = PositionStyle::Absolute;
                layout.inset = InsetStyle {
                    top: Some(Px(8.0)),
                    left: Some(Px(8.0)),
                    ..Default::default()
                };

                let mut props = ContainerProps::default();
                props.layout = layout;
                props.padding = Edges::all(Px(6.0));
                props.background = Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.65,
                });
                props.corner_radii = Corners::all(Px(6.0));
                props.border = Edges::all(Px(1.0));
                props.border_color = Some(Color {
                    r: 0.2,
                    g: 0.8,
                    b: 1.0,
                    a: 1.0,
                });

                let mut lines: Vec<String> = Vec::new();
                if pick_armed {
                    lines.push("INSPECT: click to pick a target (Esc to cancel)".to_string());
                } else {
                    lines.push(format!(
                        "INSPECT: Esc exit | Ctrl+C copy selector | Ctrl+Shift+C copy details | F focus | L lock | Alt+Up/Down nav (consume_clicks={consume_clicks}, locked={locked})"
                    ));
                }
                if let Some(t) = toast.as_deref() {
                    lines.push(format!("status: {t}"));
                }
                if let Some(summary) = focus_summary.as_deref() {
                    lines.push(summary.to_string());
                }
                if let Some(path) = focus_path.as_deref() {
                    lines.push(path.to_string());
                }
                if let Some(sel) = best_selector.as_deref() {
                    let trimmed = if sel.len() > 180 {
                        format!("{}…", &sel[..180])
                    } else {
                        sel.to_string()
                    };
                    lines.push(format!("selector: {trimmed}"));
                }

                children.push(cx.container(props, |cx| {
                    lines.into_iter().map(|t| cx.text(t)).collect::<Vec<_>>()
                }));
            }

            let show_focus = focus.as_ref().is_some_and(|f| {
                picked.as_ref().is_none_or(|p| p.node_id != f.node_id)
                    && hovered.as_ref().is_none_or(|h| h.node_id != f.node_id)
            });
            let focus_outline = if show_focus { focus } else { None };

            let outlines = [
                (
                    focus_outline,
                    Color {
                        r: 0.2,
                        g: 0.8,
                        b: 1.0,
                        a: 1.0,
                    },
                    "focus",
                ),
                (
                    picked,
                    Color {
                        r: 1.0,
                        g: 0.2,
                        b: 1.0,
                        a: 1.0,
                    },
                    "picked",
                ),
                (
                    hovered,
                    Color {
                        r: 0.2,
                        g: 1.0,
                        b: 0.4,
                        a: 1.0,
                    },
                    "hovered",
                ),
            ];

            for (node, color, tag) in outlines {
                let Some(node) = node else {
                    continue;
                };

                let rect = node.bounds;
                let mut layout = LayoutStyle::default();
                layout.position = PositionStyle::Absolute;
                layout.inset = InsetStyle {
                    top: Some(rect.origin.y),
                    left: Some(rect.origin.x),
                    ..Default::default()
                };
                layout.size = SizeStyle {
                    width: Length::Px(rect.size.width),
                    height: Length::Px(rect.size.height),
                    ..Default::default()
                };

                let mut props = ContainerProps::default();
                props.layout = layout;
                props.border = Edges::all(Px(1.0));
                props.border_color = Some(color);
                props.corner_radii = Corners::all(Px(2.0));

                children.push(cx.container(props, |_cx| Vec::new()));

                let mut label_layout = LayoutStyle::default();
                label_layout.position = PositionStyle::Absolute;
                label_layout.inset = InsetStyle {
                    top: Some(rect.origin.y),
                    left: Some(rect.origin.x),
                    ..Default::default()
                };

                let mut label_props = ContainerProps::default();
                label_props.layout = label_layout;
                label_props.padding = Edges::all(Px(4.0));
                label_props.background = Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.75,
                });
                label_props.corner_radii = Corners::all(Px(4.0));

                let role = crate::ui_diagnostics::semantics_role_label(node.role);
                let mut label = format!("[{tag}] {role} node={}", node.node_id);
                if let Some(test_id) = node.test_id.as_deref() {
                    label.push_str(&format!(" test_id={test_id}"));
                }
                if !redact_text && let Some(name) = node.label.as_deref() {
                    label.push_str(&format!(" label={name}"));
                }

                children.push(cx.container(label_props, |cx| vec![cx.text(label)]));
            }

            let mut root_layout = LayoutStyle::default();
            root_layout.position = PositionStyle::Relative;
            root_layout.size = SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            };

            let mut root_props = ContainerProps::default();
            root_props.layout = root_layout;
            root_props.background = None;
            root_props.border = Edges::all(Px(0.0));
            root_props.border_color = None;

            vec![cx.container(root_props, |_cx| children)]
        },
    );

    let layer = ui
        .node_layer(root_node)
        .unwrap_or_else(|| ui.push_overlay_root_ex(root_node, false, false));
    ui.set_layer_visible(layer, present);
    ui.set_layer_hit_testable(layer, false);
    ui.set_layer_wants_pointer_down_outside_events(layer, false);
    ui.set_layer_wants_pointer_move_events(layer, false);
    ui.set_layer_wants_timer_events(layer, false);
}

fn ui_app_hot_reload_window<S>(
    driver: &mut UiAppDriver<S>,
    context: WinitHotReloadContext<'_, UiAppWindowState<S>>,
) {
    let WinitHotReloadContext {
        app,
        services,
        window,
        state,
    } = context;

    reset_ui_tree_for_hotpatch(app, window, &mut state.ui);
    state.root = None;

    #[cfg(feature = "diagnostics")]
    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        svc.clear_window(window);
    });

    if let Some(f) = driver.on_hot_reload_window {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, services, window, &mut state.ui, &mut state.state));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, services, window, &mut state.ui, &mut state.state);
        }
    }
}

fn ui_app_window_create_spec<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    let Some(f) = driver.window_create_spec else {
        return None;
    };

    #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
    {
        let mut hot = subsecond::HotFn::current(f);
        return hot.call((app, request));
    }

    #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
    {
        f(app, request)
    }
}

fn ui_app_window_created<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
    if let Some(f) = driver.window_created {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, request, new_window));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, request, new_window);
        }
    }
}

fn ui_app_before_close_window<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
) -> bool {
    let Some(f) = driver.before_close_window else {
        return true;
    };

    #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
    {
        let mut hot = subsecond::HotFn::current(f);
        return hot.call((app, window));
    }

    #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
    {
        f(app, window)
    }
}

fn ui_app_accessibility_snapshot<S>(
    _driver: &mut UiAppDriver<S>,
    _app: &mut App,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
) -> Option<std::sync::Arc<fret_core::SemanticsSnapshot>> {
    state.ui.semantics_snapshot_arc()
}

fn ui_app_accessibility_focus<S>(
    _driver: &mut UiAppDriver<S>,
    _app: &mut App,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
) {
    state.ui.set_focus(Some(target));
}

fn ui_app_accessibility_invoke<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
) {
    fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
}

fn ui_app_accessibility_set_value_text<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
    value: &str,
) {
    fret_ui_app::accessibility_actions::set_value_text(&mut state.ui, app, services, target, value);
}

fn ui_app_viewport_input<S>(driver: &mut UiAppDriver<S>, app: &mut App, event: ViewportInputEvent) {
    #[cfg(feature = "diagnostics")]
    {
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_viewport_input(event);
        });
    }

    if let Some(f) = driver.viewport_input {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, event));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, event);
        }
    }
}

fn ui_app_dock_op<S>(driver: &mut UiAppDriver<S>, app: &mut App, op: fret_core::DockOp) {
    if let Some(f) = driver.dock_op {
        #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
        {
            let mut hot = subsecond::HotFn::current(f);
            hot.call((app, op));
        }

        #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
        {
            f(app, op);
        }
    }
}

fn ui_app_record_engine_frame<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    let Some(f) = driver.record_engine_frame else {
        return EngineFrameUpdate::default();
    };

    #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
    {
        let mut hot = subsecond::HotFn::current(f);
        hot.call((
            app,
            window,
            &mut state.ui,
            &mut state.state,
            context,
            renderer,
            scale_factor,
            tick_id,
            frame_id,
        ))
    }

    #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
    {
        f(
            app,
            window,
            &mut state.ui,
            &mut state.state,
            context,
            renderer,
            scale_factor,
            tick_id,
            frame_id,
        )
    }
}

fn reset_ui_tree_for_hotpatch(app: &mut App, window: AppWindowId, ui: &mut UiTree<App>) {
    let mut new_ui: UiTree<App> = UiTree::new();
    new_ui.set_window(window);

    let old = std::mem::replace(ui, new_ui);
    if hotpatch_drop_old_state() {
        drop(old);
    } else {
        std::mem::forget(old);
    }

    fret_ui::internal_drag::clear_window(app, window);
}

fn hotpatch_drop_old_state() -> bool {
    std::env::var_os("FRET_HOTPATCH_DROP_OLD_STATE").is_some_and(|v| !v.is_empty())
}
