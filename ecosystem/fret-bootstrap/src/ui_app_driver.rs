use fret_app::App;
use fret_app::CommandId;
use fret_app::Effect;
use fret_app::Model;
use fret_core::{
    AppWindowId, Color, Corners, Edges, Event, NodeId, Px, Rect, TextOverflow, TextWrap,
    UiServices, ViewportInputEvent,
};
use fret_launch::{
    EngineFrameUpdate, FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext,
    WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};
use fret_ui::element::Elements;
use fret_ui::overlay_placement::LayoutDirection;
use fret_ui::{ElementContext, Invalidation, UiFrameCx, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::primitives::dialog as dialog_prim;
use fret_ui_kit::primitives::direction as direction_prim;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use fret_core::time::{Duration, Instant};

#[cfg(feature = "diagnostics")]
use crate::ui_diagnostics::{UiDiagnosticsService, UiRealPerfSpanCaptureV1};

pub type ViewElements = Elements;

#[derive(Default)]
struct WindowPostFrameUiFocusService {
    by_window: HashMap<AppWindowId, PostFrameUiFocusQueue>,
}

#[derive(Default)]
struct PostFrameUiFocusQueue {
    pending: Vec<PostFrameUiFocusRequest>,
    ready: Vec<PostFrameUiFocusRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PostFrameUiFocusRequest {
    guard: PostFrameUiFocusGuard,
    target: Option<fret_ui::elements::GlobalElementId>,
    fallback_command: Option<CommandId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFrameUiFocusGuard {
    NoLiveFocus,
    Unchanged(fret_ui::elements::GlobalElementId),
    Authoritative,
}

fn post_frame_ui_focus_request_can_apply(
    request: &PostFrameUiFocusRequest,
    focused_node_present: bool,
    focused_element: Option<fret_ui::elements::GlobalElementId>,
) -> bool {
    match request.guard {
        PostFrameUiFocusGuard::NoLiveFocus => !focused_node_present,
        PostFrameUiFocusGuard::Unchanged(expected) => {
            !focused_node_present || focused_element == Some(expected)
        }
        PostFrameUiFocusGuard::Authoritative => true,
    }
}

/// Queue focus restoration until the retained tree has completed its next frame.
///
/// This is intended for follow-up policy such as focus restoration after an instant modal closes.
/// The driver waits until the authoritative input-context snapshot no longer reports a modal,
/// then focuses the original live element or dispatches the supplied UI-only fallback command.
/// `guard` makes the transaction's authority explicit instead of overloading a missing element ID.
pub fn defer_ui_focus_until_after_frame(
    app: &mut App,
    window: AppWindowId,
    guard: PostFrameUiFocusGuard,
    target: Option<fret_ui::elements::GlobalElementId>,
    fallback_command: Option<CommandId>,
) {
    app.with_global_mut_untracked(WindowPostFrameUiFocusService::default, |service, _app| {
        service
            .by_window
            .entry(window)
            .or_default()
            .pending
            .push(PostFrameUiFocusRequest {
                guard,
                target,
                fallback_command,
            });
    });
    app.request_redraw(window);
}

/// Advanced-driver lifecycle for focus requests queued with
/// [`defer_ui_focus_until_after_frame`].
///
/// `UiAppDriver` drives this automatically. Custom retained-tree drivers must call
/// [`Self::begin_frame`] before rebuilding the tree and [`Self::finish_frame`] after the frame is
/// complete so queued focus transactions observe the same ordering.
pub struct PostFrameUiFocusLifecycle;

impl PostFrameUiFocusLifecycle {
    pub fn begin_frame(app: &mut App, window: AppWindowId) {
        promote_post_frame_ui_focus_requests(app, window);
    }

    pub fn finish_frame(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        ui: &mut UiTree<App>,
    ) {
        if ui.has_active_input_barrier() {
            return;
        }

        let ready_requests = take_ready_post_frame_ui_focus_requests(app, window);
        if ready_requests.is_empty() {
            return;
        }

        for request in ready_requests {
            let focused_node = ui.focus();
            let focused_element = focused_node.and_then(|node| ui.debug_node_element(node));
            let live_focus_present = match (focused_node, focused_element) {
                (None, _) => false,
                (Some(_), None) => true,
                (Some(node), Some(element)) => ui
                    .live_attached_node_for_element(app, element)
                    .is_some_and(|live| live == node),
            };
            if !post_frame_ui_focus_request_can_apply(&request, live_focus_present, focused_element)
            {
                continue;
            }
            let restored_target = request.target.is_some_and(|target| {
                if ui.live_attached_node_for_element(app, target).is_none() {
                    return false;
                }
                ui.request_focus_element(app, target);
                true
            });
            if restored_target {
                app.request_redraw(window);
            } else if let Some(command) = request.fallback_command
                && ui.dispatch_command(app, services, &command)
            {
                app.request_redraw(window);
            }
        }
    }

    pub fn clear_window(app: &mut App, window: AppWindowId) {
        clear_post_frame_ui_focus_requests(app, window);
    }
}

/// Record a command handled by an app/runner integration rather than a retained UI element.
///
/// The caller owns pending dispatch-source routing. Any domain outcome previously recorded in
/// [`fret_runtime::WindowPendingCommandDispatchOutcomeService`] is consumed into the final trace.
pub fn record_driver_handled_command_dispatch(
    app: &mut App,
    window: AppWindowId,
    command: &CommandId,
    source: &fret_runtime::CommandDispatchSourceV1,
    started_from_focus: bool,
) {
    let handled_by_scope = app
        .commands()
        .get(command.clone())
        .map(|meta| meta.scope)
        .or(Some(fret_runtime::CommandScope::Window));
    let outcome = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchOutcomeService::default,
        |service, app| service.consume(window, app.tick_id(), command),
    );
    app.with_global_mut(
        fret_runtime::WindowCommandDispatchDiagnosticsStore::default,
        |store, app| {
            store.record(fret_runtime::CommandDispatchDecisionV1 {
                seq: 0,
                frame_id: app.frame_id(),
                tick_id: app.tick_id(),
                window,
                command: command.clone(),
                source: source.clone(),
                outcome,
                handled: true,
                handled_by_element: None,
                handled_by_scope,
                handled_by_driver: true,
                stopped: false,
                started_from_focus,
                used_default_root_fallback: false,
            });
        },
    );
}

fn promote_post_frame_ui_focus_requests(app: &mut App, window: AppWindowId) {
    app.with_global_mut_untracked(WindowPostFrameUiFocusService::default, |service, _app| {
        let Some(queue) = service.by_window.get_mut(&window) else {
            return;
        };
        if !queue.pending.is_empty() {
            queue.ready.append(&mut queue.pending);
        }
    });
}

fn take_ready_post_frame_ui_focus_requests(
    app: &mut App,
    window: AppWindowId,
) -> Vec<PostFrameUiFocusRequest> {
    app.with_global_mut_untracked(WindowPostFrameUiFocusService::default, |service, _app| {
        let Some(queue) = service.by_window.get_mut(&window) else {
            return Vec::new();
        };
        let ready = std::mem::take(&mut queue.ready);
        if queue.pending.is_empty() && queue.ready.is_empty() {
            service.by_window.remove(&window);
        }
        ready
    })
}

fn clear_post_frame_ui_focus_requests(app: &mut App, window: AppWindowId) {
    app.with_global_mut_untracked(WindowPostFrameUiFocusService::default, |service, _app| {
        service.by_window.remove(&window);
    });
}

type ViewFn<S> = for<'a> fn(&mut ElementContext<'a, App>, &mut S) -> ViewElements;

type EventHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &Event);

type AppEventHookFn<S> = fn(&mut App, AppWindowId, &mut S, &Event);

type CommandHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &CommandId);

type CommandBeforeUiHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &CommandId) -> bool;

/// Data-only routing context for app-owned command hooks that run before retained UI dispatch.
///
/// `source_is_within_active_input_barrier_scope` is computed against the live `UiTree`; callers
/// must not infer modal provenance from the best-effort source metadata alone.
#[derive(Debug, Clone, Copy)]
pub struct UiAppCommandBeforeUiContext<'a> {
    pub source: &'a fret_runtime::CommandDispatchSourceV1,
    pub ui_has_modal: bool,
    pub source_is_within_active_input_barrier_scope: bool,
}

type AppCommandBeforeUiHookFn<S> =
    for<'a> fn(&mut App, AppWindowId, &mut S, &CommandId, UiAppCommandBeforeUiContext<'a>) -> bool;

type PreferencesHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S);

type HotReloadHookFn<S> = fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S);

type ModelChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[fret_app::ModelId]);
type GlobalChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[std::any::TypeId]);
type AppGlobalChangesHookFn<S> = fn(&mut App, AppWindowId, &mut S, &[std::any::TypeId]);

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

type RendererPerfSampleHookFn<S> = fn(
    &mut App,
    AppWindowId,
    &mut UiTree<App>,
    &mut S,
    Option<fret_render::RendererPerfFrameSample>,
);

type FrameStageHookFn<S> = fn(&mut App, AppWindowId, &mut S, UiAppFrameObservation);

/// Ordered checkpoints emitted by the `UiAppDriver` frame path.
///
/// These stages are an app-facing summary of the real driver pipeline. They intentionally do not
/// expose `UiTree`, `UiFrameCx`, or runner contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAppFrameStage {
    Begin,
    View,
    Overlay,
    DiagnosticsOverlay,
    Semantics,
    Layout,
    Paint,
    DiagnosticsDriveScript,
    DiagnosticsSnapshot,
    End,
}

/// Lightweight frame observation passed to app-facing harnesses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiAppFrameObservation {
    pub stage: UiAppFrameStage,
    pub window: AppWindowId,
    pub bounds: Rect,
    pub scale_factor: f32,
    pub tick_id: TickId,
    pub frame_id: FrameId,
}

/// App state can implement this to collect frame-stage observations without owning raw UI staging.
pub trait UiAppFrameStageSink {
    fn record_frame_stage(&mut self, observation: UiAppFrameObservation);
}

fn record_frame_stage_into_sink<S: UiAppFrameStageSink>(
    _app: &mut App,
    _window: AppWindowId,
    state: &mut S,
    observation: UiAppFrameObservation,
) {
    state.record_frame_stage(observation);
}

#[cfg(feature = "ui-app-command-palette")]
pub type CommandPaletteOverlayFn =
    for<'a> fn(&mut ElementContext<'a, App>, CommandPaletteOverlayCx, &mut ViewElements);

#[cfg(feature = "ui-app-command-palette")]
#[derive(Debug, Clone)]
pub struct CommandPaletteOverlayCx {
    pub models: CommandPaletteModels,
    pub open: bool,
}

/// A minimal, hotpatch-friendly “golden path” app driver.
///
/// This wraps `fret-launch::FnDriver` and centralizes common boilerplate:
/// - declarative root mounting (`frame_pipeline`)
/// - `UiTree` event/command routing
/// - model/global change propagation
/// - layout/paint submission via `UiFrameCx`
/// - accessibility snapshot + actions
/// - conservative hot reload reset (Subsecond-friendly)
///
/// This driver intentionally uses `fn` pointers (not captured closures) to keep dev hotpatch behavior
/// predictable (ADR 0105).
pub struct UiAppDriver<S> {
    root_name: &'static str,
    init_window: fn(&mut App, AppWindowId) -> S,
    view: ViewFn<S>,
    close_on_window_close_requested: bool,
    #[cfg(feature = "ui-assets")]
    drive_ui_assets: bool,

    on_event: Option<EventHookFn<S>>,
    on_app_event: Option<AppEventHookFn<S>>,
    on_command_before_ui: Option<CommandBeforeUiHookFn<S>>,
    on_app_command_before_ui: Option<AppCommandBeforeUiHookFn<S>>,
    on_command: Option<CommandHookFn<S>>,
    on_preferences: Option<PreferencesHookFn<S>>,
    on_hot_reload_window: Option<HotReloadHookFn<S>>,
    on_model_changes: Option<ModelChangesHookFn<S>>,
    on_global_changes: Option<GlobalChangesHookFn<S>>,
    on_app_global_changes: Option<AppGlobalChangesHookFn<S>>,
    on_global_changes_middleware: Option<GlobalChangesHookFn<S>>,

    window_create_spec:
        Option<fn(&mut App, &fret_app::CreateWindowRequest) -> Option<WindowCreateSpec>>,
    window_created: Option<fn(&mut App, &fret_app::CreateWindowRequest, AppWindowId)>,
    before_close_window: Option<fn(&mut App, AppWindowId) -> bool>,

    handle_global_command: Option<fn(&mut App, &mut dyn UiServices, CommandId)>,

    viewport_input: Option<fn(&mut App, ViewportInputEvent)>,
    dock_op: Option<fn(&mut App, fret_core::DockOp)>,
    record_engine_frame: Option<RecordEngineFrameHookFn<S>>,
    renderer_perf_sample: Option<RendererPerfSampleHookFn<S>>,
    on_frame_stage: Option<FrameStageHookFn<S>>,

    #[cfg(feature = "ui-app-command-palette")]
    command_palette_enabled: bool,
    #[cfg(feature = "ui-app-command-palette")]
    command_palette_overlay: Option<CommandPaletteOverlayFn>,
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
            on_app_event: None,
            on_command_before_ui: None,
            on_app_command_before_ui: None,
            on_command: None,
            on_preferences: None,
            on_hot_reload_window: None,
            on_model_changes: None,
            on_global_changes: None,
            on_app_global_changes: None,
            on_global_changes_middleware: None,
            window_create_spec: None,
            window_created: None,
            before_close_window: None,
            handle_global_command: None,
            viewport_input: None,
            dock_op: None,
            record_engine_frame: None,
            renderer_perf_sample: None,
            on_frame_stage: None,

            #[cfg(feature = "ui-app-command-palette")]
            command_palette_enabled: true,
            #[cfg(feature = "ui-app-command-palette")]
            command_palette_overlay: None,
        }
    }

    #[cfg(feature = "ui-app-command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.command_palette_enabled = enabled;
        self
    }

    /// Register an app-owned command palette overlay renderer.
    ///
    /// `fret-bootstrap` keeps the command toggle/gating capability, while recipe crates own the
    /// default UI presentation.
    #[cfg(feature = "ui-app-command-palette")]
    pub fn command_palette_overlay(mut self, f: CommandPaletteOverlayFn) -> Self {
        self.command_palette_overlay = Some(f);
        self
    }

    pub fn on_event(mut self, f: EventHookFn<S>) -> Self {
        self.on_event = Some(f);
        self
    }

    /// Register an app-state event hook without exposing retained-tree or service seams.
    pub fn on_app_event(mut self, f: AppEventHookFn<S>) -> Self {
        self.on_app_event = Some(f);
        self
    }

    /// When `true` (default, with the `ui-assets` feature enabled), drives `fret-ui-assets`
    /// caches from the event pipeline.
    ///
    /// This makes `ImageAssetCache` work out-of-the-box in golden-path apps without additional
    /// boilerplate (ADR 0106 / ADR 0110).
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

    /// Run a command hook before the retained UI tree receives the command.
    ///
    /// This is for app-facing harnesses that own a model transaction first (for example a
    /// workspace dirty-close policy) while still using `UiAppDriver` for frame/diagnostics
    /// ownership. Return `true` when the hook handled the command; the driver records that
    /// outcome with the original dispatch source for diagnostics.
    pub fn on_command_before_ui(mut self, f: CommandBeforeUiHookFn<S>) -> Self {
        self.on_command_before_ui = Some(f);
        self
    }

    /// Register an app-state command hook before retained UI routing.
    ///
    /// The hook receives the preserved dispatch source plus live modal provenance, but no `UiTree`
    /// or `UiServices` access.
    pub fn on_app_command_before_ui(mut self, f: AppCommandBeforeUiHookFn<S>) -> Self {
        self.on_app_command_before_ui = Some(f);
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

    /// Register an app-state global-change hook without retained-tree access.
    pub fn on_app_global_changes(mut self, f: AppGlobalChangesHookFn<S>) -> Self {
        self.on_app_global_changes = Some(f);
        self
    }

    /// Register a global-changes middleware hook that runs before `on_global_changes`.
    ///
    /// This is intended for framework-level integration seams that should not override app-owned
    /// global-changes handling (e.g. ecosystem policy helpers that react to `WindowMetricsService`).
    pub fn on_global_changes_middleware(mut self, f: GlobalChangesHookFn<S>) -> Self {
        self.on_global_changes_middleware = Some(f);
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

    /// Observe the app-facing frame pipeline without taking ownership of raw UI staging.
    pub fn on_frame_stage(mut self, f: FrameStageHookFn<S>) -> Self {
        self.on_frame_stage = Some(f);
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

            hooks.semantics_snapshot = Some(ui_app_accessibility_snapshot::<S>);
            hooks.accessibility_focus = Some(ui_app_accessibility_focus::<S>);
            hooks.accessibility_invoke = Some(ui_app_accessibility_invoke::<S>);
            hooks.accessibility_set_value_text = Some(ui_app_accessibility_set_value_text::<S>);
            hooks.accessibility_set_value_numeric =
                Some(ui_app_accessibility_set_value_numeric::<S>);
            hooks.accessibility_decrement = Some(ui_app_accessibility_decrement::<S>);
            hooks.accessibility_increment = Some(ui_app_accessibility_increment::<S>);
            hooks.accessibility_scroll_by = Some(ui_app_accessibility_scroll_by::<S>);
            hooks.accessibility_set_text_selection =
                Some(ui_app_accessibility_set_text_selection::<S>);
            hooks.accessibility_replace_selected_text =
                Some(ui_app_accessibility_replace_selected_text::<S>);

            hooks.viewport_input = Some(ui_app_viewport_input::<S>);
            hooks.dock_op = Some(ui_app_dock_op::<S>);
            hooks.record_engine_frame = Some(ui_app_record_engine_frame::<S>);
            hooks.renderer_perf_sample = Some(ui_app_renderer_perf_sample::<S>);
            hooks.scene_chunk_manifest = Some(ui_app_scene_chunk_manifest::<S>);
        })
    }
}

impl<S: UiAppFrameStageSink> UiAppDriver<S> {
    /// Record driver frame stages into the app state.
    pub fn record_frame_stages(self) -> Self {
        self.on_frame_stage(record_frame_stage_into_sink::<S>)
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

    fn take(&mut self) -> (Vec<fret_app::ModelId>, Vec<std::any::TypeId>) {
        let models = std::mem::take(&mut self.models);
        let globals = std::mem::take(&mut self.globals);
        self.models_seen.clear();
        self.globals_seen.clear();
        (models, globals)
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
        let fallback_input_ctx = fret_ui_kit::command::command_palette_input_context(app);
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

        app.with_global_mut_untracked(CommandPaletteService::default, |svc, _app| {
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

    let handle = app.with_global_mut_untracked(CommandPaletteService::default, |svc, _app| {
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

#[cfg(feature = "ui-app-command-palette")]
fn render_command_palette_overlay_if_needed<S>(
    driver: &UiAppDriver<S>,
    cx: &mut ElementContext<'_, App>,
    out: &mut ViewElements,
) {
    if !driver.command_palette_enabled {
        return;
    }

    let Some(models) = cx
        .app
        .global::<CommandPaletteService>()
        .and_then(|svc| svc.models(cx.window))
    else {
        return;
    };

    let open_now = cx.app.models().get_copied(&models.open).unwrap_or(false);
    command_palette_cleanup_gating_if_closed(cx.app, cx.window, open_now);

    if let Some(render_overlay) = driver.command_palette_overlay {
        render_overlay(
            cx,
            CommandPaletteOverlayCx {
                models,
                open: open_now,
            },
            out,
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

    #[test]
    fn command_palette_cleanup_does_not_mark_service_changed_each_frame() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let _ = app.take_changed_globals();
        assert_eq!(command_palette_toggle(&mut app, window), true);
        let _ = app.take_changed_globals();

        let models = app
            .global::<CommandPaletteService>()
            .and_then(|svc| svc.models(window))
            .expect("command palette models");
        let _ = app.models_mut().update(&models.open, |v| *v = false);
        command_palette_cleanup_gating_if_closed(&mut app, window, false);

        let changed = app.take_changed_globals();
        assert!(
            !changed.contains(&std::any::TypeId::of::<CommandPaletteService>()),
            "command palette cleanup bookkeeping should stay untracked"
        );

        command_palette_cleanup_gating_if_closed(&mut app, window, false);
        assert!(app.take_changed_globals().is_empty());
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

    let theme = cx.theme_snapshot();
    let pad = theme.metric_by_key("metric.padding.md").unwrap_or(Px(16.0));
    let pad_sm = theme.metric_by_key("metric.padding.sm").unwrap_or(Px(12.0));
    let radius = theme.metric_by_key("metric.radius.md").unwrap_or(Px(8.0));
    let radius_sm = theme.metric_by_key("metric.radius.sm").unwrap_or(Px(6.0));

    let fg = theme.color_token("foreground");
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
                        padding: Edges::all(pad_sm).into(),
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
                            align: fret_core::TextAlign::Start,
                            ink_overflow: fret_ui::element::TextInkOverflow::None,
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
            gap: Px(0.0).into(),
            padding: Edges::all(Px(0.0)).into(),
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
                    padding: Edges::all(pad).into(),
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
                            gap: Px(12.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                                    align: fret_core::TextAlign::Start,
                                    ink_overflow: fret_ui::element::TextInkOverflow::None,
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
                            gap: Px(6.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                                    align: fret_core::TextAlign::Start,
                                    ink_overflow: fret_ui::element::TextInkOverflow::None,
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
                                    align: fret_core::TextAlign::Start,
                                    ink_overflow: fret_ui::element::TextInkOverflow::None,
                                }),
                            ]
                        },
                    );

                    let files = cx.flex(
                        fret_ui::element::FlexProps {
                            layout: fret_ui::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Vertical,
                            gap: Px(10.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                                        gap: Px(12.0).into(),
                                        padding: Edges::all(Px(0.0)).into(),
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
                                                gap: Px(2.0).into(),
                                                padding: Edges::all(Px(0.0)).into(),
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
                                                        align: fret_core::TextAlign::Start,
                                                        ink_overflow:
                                                            fret_ui::element::TextInkOverflow::None,
                                                    }),
                                                    cx.text_props(fret_ui::element::TextProps {
                                                        layout: Default::default(),
                                                        text: std::sync::Arc::from(path),
                                                        style: None,
                                                        color: Some(muted_fg),
                                                        wrap: TextWrap::None,
                                                        overflow: TextOverflow::Clip,
                                                        align: fret_core::TextAlign::Start,
                                                        ink_overflow:
                                                            fret_ui::element::TextInkOverflow::None,
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
                                                    move |host, action_cx, _| {
                                                        let token = host.next_clipboard_token();
                                                        host.push_effect(
                                                            Effect::ClipboardWriteText {
                                                                window: action_cx.window,
                                                                token,
                                                                text: text_for_copy.clone(),
                                                            },
                                                        );
                                                    },
                                                ));
                                                vec![cx.container(
                                                    fret_ui::element::ContainerProps {
                                                        padding: Edges::all(pad_sm).into(),
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
                                                                align: fret_core::TextAlign::Start,
                                                                ink_overflow:
                                                                    fret_ui::element::TextInkOverflow::None,
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
                            padding: Edges::all(pad_sm).into(),
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
                                align: fret_core::TextAlign::Start,
                                ink_overflow: fret_ui::element::TextInkOverflow::None,
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
    let ts = fret_core::time::SystemTime::now()
        .duration_since(fret_core::time::UNIX_EPOCH)
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

#[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
fn hotpatch_view_call_use_direct() -> bool {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Strategy {
        Auto,
        HotFn,
        Direct,
    }

    fn parse_strategy(raw: &str) -> Option<Strategy> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Strategy::Auto),
            "hotfn" => Some(Strategy::HotFn),
            "direct" => Some(Strategy::Direct),
            _ => None,
        }
    }

    // Legacy escape hatch. Prefer `FRET_HOTPATCH_VIEW_CALL_STRATEGY=direct` going forward.
    let legacy_direct =
        std::env::var_os("FRET_HOTPATCH_VIEW_CALL_DIRECT").is_some_and(|v| !v.is_empty());

    let strategy = if legacy_direct {
        Strategy::Direct
    } else if let Ok(raw) = std::env::var("FRET_HOTPATCH_VIEW_CALL_STRATEGY") {
        parse_strategy(&raw).unwrap_or(Strategy::Auto)
    } else {
        Strategy::Auto
    };

    let hotpatch_enabled = std::env::var_os("FRET_HOTPATCH").is_some_and(|v| !v.is_empty())
        || std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty());

    let use_direct = match strategy {
        Strategy::Direct => true,
        Strategy::HotFn => false,
        Strategy::Auto => {
            // Default posture: on Windows, prefer a safe boundary (direct view call + runner reload)
            // over view-level Subsecond hotpatching due to a known crash mode (ADR 0105).
            //
            // Non-Windows platforms default to `HotFn` for view-level hotpatching.
            cfg!(windows) && hotpatch_enabled
        }
    };

    static WARNED_DIRECT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if use_direct {
        WARNED_DIRECT.get_or_init(|| {
            let reason = if legacy_direct {
                "FRET_HOTPATCH_VIEW_CALL_DIRECT=1"
            } else if std::env::var_os("FRET_HOTPATCH_VIEW_CALL_STRATEGY")
                .is_some_and(|v| !v.is_empty())
            {
                "FRET_HOTPATCH_VIEW_CALL_STRATEGY=direct"
            } else if cfg!(windows) && hotpatch_enabled {
                "auto (Windows safety default; see ADR 0105)"
            } else {
                "direct"
            };

            hotpatch_trace_log(&format!(
                "warning: view call strategy=direct ({reason}) (view-level hotpatching disabled)"
            ));
            eprintln!(
                "warning: view call strategy=direct ({reason}) (view-level hotpatching disabled)"
            );
        });
    }

    use_direct
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
    #[cfg(not(target_arch = "wasm32"))]
    crate::dev_reload::DevReloadWatcher::install_if_enabled(app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(
        std::env::var_os("FRET_UI_DEBUG_STATS").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
            || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty()),
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

    #[cfg(not(target_arch = "wasm32"))]
    if let Event::Timer { token } = event
        && let Some(tick) = crate::dev_reload::handle_dev_reload_timer(app, window, *token)
    {
        let actionable = tick.reloaded_theme
            || tick.reloaded_literals
            || tick.bumped_asset_reload_epoch
            || tick.theme_error.is_some()
            || tick.literals_error.is_some();

        if actionable {
            app.request_redraw(window);
            hotpatch_trace_log(&format!(
                "dev_reload: window={window:?} theme_reload={} literals_reload={} assets_epoch={} fonts_reload={} theme_err={:?} literals_err={:?} fonts_err={:?}",
                tick.reloaded_theme,
                tick.reloaded_literals,
                tick.bumped_asset_reload_epoch,
                tick.reloaded_fonts,
                tick.theme_error,
                tick.literals_error,
                tick.fonts_error,
            ));
        }
        return;
    }

    #[cfg(feature = "diagnostics")]
    if crate::ui_diagnostics::maybe_consume_event(app, window, event) {
        return;
    }

    state.ui.dispatch_event(app, services, event);

    #[cfg(feature = "ui-assets")]
    if driver.drive_ui_assets {
        let _ = fret_ui_assets::UiAssets::handle_event(app, window, event);
    }

    if let Some(on_app_event) = driver.on_app_event {
        on_app_event(app, window, &mut state.state, event);
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
    let started_from_focus = state.ui.focus().is_some();

    // Capture the best-effort pending source up front so driver-handled commands can record the
    // same origin metadata as UI-tree-handled commands (ADR 0307).
    let pending_source = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.consume(window, app.tick_id(), &command)
                .unwrap_or_else(fret_runtime::CommandDispatchSourceV1::programmatic)
        },
    );

    #[cfg(feature = "ui-app-command-palette")]
    if driver.command_palette_enabled
        && command.as_str() == fret_app::core_commands::COMMAND_PALETTE
    {
        let _ = command_palette_toggle(app, window);
        record_driver_handled_command_dispatch(
            app,
            window,
            &command,
            &pending_source,
            started_from_focus,
        );
        return;
    }

    // Re-insert the pending source so the UI tree dispatch can consume it when it records its own
    // trace entry.
    let restored_source_ticket = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.restore_next(
                window,
                app.tick_id(),
                command.clone(),
                pending_source.clone(),
            )
        },
    );

    let ui_has_modal = state.ui.has_active_input_barrier();
    let source_is_within_active_input_barrier_scope = ui_has_modal
        && pending_source.element.is_some_and(|element| {
            state.ui.element_is_within_active_input_barrier_scope(
                app,
                fret_ui::GlobalElementId(element),
            )
        });
    let app_command_context = UiAppCommandBeforeUiContext {
        source: &pending_source,
        ui_has_modal,
        source_is_within_active_input_barrier_scope,
    };

    if let Some(on_app_command_before_ui) = driver.on_app_command_before_ui
        && on_app_command_before_ui(app, window, &mut state.state, &command, app_command_context)
    {
        app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, _app| {
                let _ = service.discard_restored(restored_source_ticket);
            },
        );
        record_driver_handled_command_dispatch(
            app,
            window,
            &command,
            &pending_source,
            started_from_focus,
        );
        return;
    }

    if let Some(on_command_before_ui) = driver.on_command_before_ui {
        let handled = {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(on_command_before_ui);
                hot.call((
                    app,
                    services,
                    window,
                    &mut state.ui,
                    &mut state.state,
                    &command,
                ))
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                on_command_before_ui(
                    app,
                    services,
                    window,
                    &mut state.ui,
                    &mut state.state,
                    &command,
                )
            }
        };

        if handled {
            // The hook may have dispatched into the UI tree and consumed this source already.
            // Remove any leftover copy before recording the app-owned handling outcome.
            app.with_global_mut(
                fret_runtime::WindowPendingCommandDispatchSourceService::default,
                |svc, _app| {
                    let _ = svc.discard_restored(restored_source_ticket);
                },
            );
            record_driver_handled_command_dispatch(
                app,
                window,
                &command,
                &pending_source,
                started_from_focus,
            );
            return;
        }
    }

    if state.ui.dispatch_command(app, services, &command) {
        return;
    }

    // `dispatch_command` normally consumes the restored source even when no widget handles the
    // command. Before the first UI root exists it returns early, so clean up this exact occurrence
    // before trying driver-owned fallbacks.
    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |service, _app| {
            let _ = service.discard_restored(restored_source_ticket);
        },
    );

    match command.as_str() {
        fret_app::core_commands::APP_ABOUT => {
            #[cfg(target_os = "macos")]
            {
                app.push_effect(Effect::ShowAboutPanel);
                record_driver_handled_command_dispatch(
                    app,
                    window,
                    &command,
                    &pending_source,
                    started_from_focus,
                );
                return;
            }
        }
        fret_app::core_commands::APP_PREFERENCES => {
            if let Some(f) = driver.on_preferences {
                f(app, services, window, &mut state.ui, &mut state.state);
                record_driver_handled_command_dispatch(
                    app,
                    window,
                    &command,
                    &pending_source,
                    started_from_focus,
                );
                return;
            }
        }
        fret_app::core_commands::APP_LOCALE_SWITCH_NEXT => {
            if fret_app::core_commands::handle_locale_cycle_command(app, &command) {
                app.request_redraw(window);
                record_driver_handled_command_dispatch(
                    app,
                    window,
                    &command,
                    &pending_source,
                    started_from_focus,
                );
                return;
            }
        }
        fret_app::core_commands::APP_QUIT => {
            app.push_effect(Effect::QuitApp);
            record_driver_handled_command_dispatch(
                app,
                window,
                &command,
                &pending_source,
                started_from_focus,
            );
            return;
        }
        fret_app::core_commands::APP_HIDE => {
            app.push_effect(Effect::HideApp);
            record_driver_handled_command_dispatch(
                app,
                window,
                &command,
                &pending_source,
                started_from_focus,
            );
            return;
        }
        fret_app::core_commands::APP_HIDE_OTHERS => {
            app.push_effect(Effect::HideOtherApps);
            record_driver_handled_command_dispatch(
                app,
                window,
                &command,
                &pending_source,
                started_from_focus,
            );
            return;
        }
        fret_app::core_commands::APP_SHOW_ALL => {
            app.push_effect(Effect::UnhideAllApps);
            record_driver_handled_command_dispatch(
                app,
                window,
                &command,
                &pending_source,
                started_from_focus,
            );
            return;
        }
        _ => {}
    }

    if fret_ui_kit::try_handle_window_overlays_command(&mut state.ui, app, window, &command) {
        record_driver_handled_command_dispatch(
            app,
            window,
            &command,
            &pending_source,
            started_from_focus,
        );
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
        fret_app::core_commands::TEXT_RESCAN_SYSTEM_FONTS => {
            app.push_effect(Effect::TextRescanSystemFonts);
            return;
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
        let changed_names = changed
            .iter()
            .map(|&t| {
                app.global_type_name(t)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{t:?}"))
            })
            .collect::<Vec<_>>();
        app.with_global_mut_untracked(
            fret_runtime::WindowGlobalChangeDiagnosticsStore::default,
            |store, app| {
                store.record_batch(window, app.frame_id(), changed_names.iter());
            },
        );
        app.request_redraw(window);
    }

    if let Some(f) = driver.on_global_changes_middleware {
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

    if let Some(f) = driver.on_app_global_changes {
        f(app, window, &mut state.state, changed);
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

#[cfg(target_arch = "wasm32")]
fn frame_hitch_config() -> Option<FrameHitchConfig> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn frame_hitch_log_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();
    paths.push(std::path::Path::new(".fret").join("frame_hitches.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("frame_hitches.log"));
    }
    paths.into_iter()
}

#[cfg(not(target_arch = "wasm32"))]
struct FrameHitchLogWriter {
    file: std::io::BufWriter<std::fs::File>,
}

#[cfg(not(target_arch = "wasm32"))]
struct FrameHitchLogState {
    writers: Vec<FrameHitchLogWriter>,
    writes_since_flush: u32,
    last_flush: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl FrameHitchLogState {
    fn new() -> Self {
        let mut writers = Vec::new();
        for path in frame_hitch_log_paths() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                writers.push(FrameHitchLogWriter {
                    file: std::io::BufWriter::with_capacity(16 * 1024, file),
                });
            }
        }

        Self {
            writers,
            writes_since_flush: 0,
            last_flush: Instant::now(),
        }
    }

    fn write_line(&mut self, msg: &str) {
        use std::io::Write as _;

        let mut i = 0;
        while i < self.writers.len() {
            let ok = self.writers[i].file.write_all(msg.as_bytes()).is_ok();
            if ok {
                i += 1;
            } else {
                self.writers.swap_remove(i);
            }
        }

        self.writes_since_flush = self.writes_since_flush.saturating_add(1);
        let should_flush =
            self.writes_since_flush >= 64 || self.last_flush.elapsed().as_millis() >= 250;
        if should_flush {
            for w in self.writers.iter_mut() {
                let _ = w.file.flush();
            }
            self.writes_since_flush = 0;
            self.last_flush = Instant::now();
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn write_frame_hitch_log(_line: &str) {}

#[cfg(not(target_arch = "wasm32"))]
fn write_frame_hitch_log(line: &str) {
    let ts = fret_core::time::SystemTime::now()
        .duration_since(fret_core::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    static STATE: OnceLock<Mutex<FrameHitchLogState>> = OnceLock::new();
    let state = STATE.get_or_init(|| Mutex::new(FrameHitchLogState::new()));
    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
    state.write_line(&msg);
}

#[derive(Debug, Clone, Copy)]
enum UiDriverPhase {
    View,
    #[cfg(all(feature = "diagnostics", feature = "ui-app-command-palette"))]
    ViewCommandPaletteOverlay,
    #[cfg(feature = "diagnostics")]
    ViewPreferencesOverlay,
    Overlay,
    Layout,
    Paint,
    #[cfg(feature = "diagnostics")]
    DiagnosticsDriveScript,
}

impl UiDriverPhase {
    #[cfg(feature = "diagnostics")]
    fn perf_span_name(self) -> &'static str {
        match self {
            Self::View => "fret.ui.view",
            #[cfg(all(feature = "diagnostics", feature = "ui-app-command-palette"))]
            Self::ViewCommandPaletteOverlay => "fret.ui.view.command_palette_overlay",
            #[cfg(feature = "diagnostics")]
            Self::ViewPreferencesOverlay => "fret.ui.view.preferences_overlay",
            Self::Overlay => "fret.ui.overlay",
            Self::Layout => "fret.ui.layout",
            Self::Paint => "fret.ui.paint",
            #[cfg(feature = "diagnostics")]
            Self::DiagnosticsDriveScript => "fret.ui.diagnostics.drive_script",
        }
    }

    #[cfg(feature = "diagnostics")]
    fn perf_span_phase(self) -> &'static str {
        match self {
            Self::View => "view",
            #[cfg(all(feature = "diagnostics", feature = "ui-app-command-palette"))]
            Self::ViewCommandPaletteOverlay => "view_command_palette_overlay",
            #[cfg(feature = "diagnostics")]
            Self::ViewPreferencesOverlay => "view_preferences_overlay",
            Self::Overlay => "overlay",
            Self::Layout => "layout",
            Self::Paint => "paint",
            #[cfg(feature = "diagnostics")]
            Self::DiagnosticsDriveScript => "diagnostics_drive_script",
        }
    }
}

#[cfg(feature = "tracing")]
impl UiDriverPhase {
    fn make_span(self) -> tracing::Span {
        match self {
            Self::View => tracing::info_span!("fret.ui.view"),
            #[cfg(all(feature = "diagnostics", feature = "ui-app-command-palette"))]
            Self::ViewCommandPaletteOverlay => {
                tracing::info_span!("fret.ui.view.command_palette_overlay")
            }
            #[cfg(feature = "diagnostics")]
            Self::ViewPreferencesOverlay => {
                tracing::info_span!("fret.ui.view.preferences_overlay")
            }
            Self::Overlay => tracing::info_span!("fret.ui.overlay"),
            Self::Layout => tracing::info_span!("fret.ui.layout"),
            Self::Paint => tracing::info_span!("fret.ui.paint"),
            #[cfg(feature = "diagnostics")]
            Self::DiagnosticsDriveScript => {
                tracing::info_span!("fret.ui.diagnostics.drive_script")
            }
        }
    }
}

#[cfg(feature = "diagnostics")]
type UiDriverPerfSpanCapture = UiRealPerfSpanCaptureV1;

fn measure_ui_driver_phase<T>(
    phase: UiDriverPhase,
    time_enabled: bool,
    f: impl FnOnce() -> T,
) -> (T, Option<Duration>) {
    #[cfg(feature = "tracing")]
    {
        fret_perf::measure_span(
            time_enabled,
            tracing::enabled!(tracing::Level::INFO),
            || phase.make_span(),
            f,
        )
    }

    #[cfg(not(feature = "tracing"))]
    {
        let _ = phase;
        fret_perf::measure(time_enabled, f)
    }
}

#[cfg(feature = "diagnostics")]
fn measure_ui_driver_phase_for_frame_with_capture<T>(
    capture: &mut Option<UiDriverPerfSpanCapture>,
    phase: UiDriverPhase,
    time_enabled: bool,
    f: impl FnOnce(&mut Option<UiDriverPerfSpanCapture>) -> T,
) -> (T, Option<Duration>) {
    let capture_start_us = capture
        .as_ref()
        .map(UiDriverPerfSpanCapture::frame_elapsed_us);
    let (out, elapsed) =
        measure_ui_driver_phase(phase, time_enabled || capture.is_some(), || f(capture));
    if let (Some(capture), Some(start_us), Some(elapsed)) =
        (capture.as_mut(), capture_start_us, elapsed)
    {
        capture.push_phase(
            phase.perf_span_name(),
            phase.perf_span_phase(),
            "ui_app_driver",
            start_us,
            elapsed,
        );
    }
    (out, elapsed)
}

#[cfg(feature = "diagnostics")]
fn measure_ui_driver_phase_for_frame<T>(
    capture: &mut Option<UiDriverPerfSpanCapture>,
    phase: UiDriverPhase,
    time_enabled: bool,
    f: impl FnOnce() -> T,
) -> (T, Option<Duration>) {
    measure_ui_driver_phase_for_frame_with_capture(capture, phase, time_enabled, |_| f())
}

#[cfg(not(feature = "diagnostics"))]
fn measure_ui_driver_phase_for_frame_with_capture<T>(
    capture: &mut (),
    phase: UiDriverPhase,
    time_enabled: bool,
    f: impl FnOnce(&mut ()) -> T,
) -> (T, Option<Duration>) {
    measure_ui_driver_phase(phase, time_enabled, || f(capture))
}

#[cfg(not(feature = "diagnostics"))]
fn measure_ui_driver_phase_for_frame<T>(
    capture: &mut (),
    phase: UiDriverPhase,
    time_enabled: bool,
    f: impl FnOnce() -> T,
) -> (T, Option<Duration>) {
    measure_ui_driver_phase_for_frame_with_capture(capture, phase, time_enabled, |_| f())
}

fn ui_app_observe_frame_stage<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    bounds: Rect,
    scale_factor: f32,
    stage: UiAppFrameStage,
) {
    let Some(f) = driver.on_frame_stage else {
        return;
    };
    let observation = UiAppFrameObservation {
        stage,
        window,
        bounds,
        scale_factor,
        tick_id: app.tick_id(),
        frame_id: app.frame_id(),
    };

    #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
    {
        let mut hot = subsecond::HotFn::current(f);
        hot.call((app, window, &mut state.state, observation));
    }

    #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
    {
        f(app, window, &mut state.state, observation);
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
    PostFrameUiFocusLifecycle::begin_frame(app, window);

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
    #[cfg(feature = "diagnostics")]
    let mut perf_span_capture = UiDriverPerfSpanCapture::new_if_enabled();
    #[cfg(not(feature = "diagnostics"))]
    let mut perf_span_capture = ();
    let hitch_total_started = hitch_config.map(|_| Instant::now());
    let mut hitch_view_ms: Option<u64> = None;
    let mut hitch_overlay_ms: Option<u64> = None;
    let mut hitch_paint_ms: Option<u64> = None;

    // Note: diagnostics may enable inspection mode (disables caching) on demand.
    #[cfg(feature = "diagnostics")]
    let diag_inspection_active = app
        .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.wants_inspection_active(window)
        });
    #[cfg(feature = "diagnostics")]
    state.ui.set_inspection_active(diag_inspection_active);

    let render_depth = RENDER_DEPTH.with(|d| {
        let next = d.get().saturating_add(1);
        d.set(next);
        next
    });
    hotpatch_trace_log(&format!(
        "ui_app_render: begin window={window:?} depth={render_depth}"
    ));
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::Begin,
    );

    OverlayController::begin_frame(app, window);
    hotpatch_trace_log(&format!(
        "ui_app_render: after begin_frame window={window:?}"
    ));

    // Apply invalidations before mounting the declarative root so view cache reuse sees the
    // correct dirty flags in the same frame.
    let (changed_models, changed_globals) = state.pending_invalidation.take();

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

    let (root, view_elapsed) = measure_ui_driver_phase_for_frame_with_capture(
        &mut perf_span_capture,
        UiDriverPhase::View,
        hitch_config.is_some(),
        |_frame_perf_span_capture| {
            fret_ui::frame_pipeline::render_base_root_with_changes(
                &mut state.ui,
                app,
                services,
                window,
                bounds,
                driver.root_name,
                &changed_models,
                &changed_globals,
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
                            subsecond::get_jump_table()
                                .and_then(|table| table.map.get(&view_ptr).cloned())
                        };
                        hotpatch_trace_log(&format!(
                            "ui_app_render: view ptr=0x{view_ptr:x} mapped={mapped:?}"
                        ));
                        #[cfg(windows)]
                        {
                            let view_module = hotpatch_module_path_for_address(view_ptr as usize)
                                .map(|p| p.display().to_string());
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
                            let mapped_head =
                                mapped.and_then(|p| hotpatch_head_bytes(p as usize, 16));
                            hotpatch_trace_log(&format!(
                                "ui_app_render: view head16={view_head:?} mapped_head16={mapped_head:?}"
                            ));

                            #[cfg(windows)]
                            if let Some(mapped_addr) = mapped {
                                if let Some(head) = hotpatch_head16(mapped_addr as usize) {
                                    if let Some(target) = hotpatch_call_target_from_head16(
                                        mapped_addr as usize,
                                        &head,
                                    ) {
                                        let target_module =
                                            hotpatch_module_path_for_address(target)
                                                .map(|p| p.display().to_string());
                                        let target_head16 = hotpatch_head_bytes(target, 16);
                                        hotpatch_trace_log(&format!(
                                            "ui_app_render: mapped prologue call_target=0x{target:x} target_module={target_module:?} target_head16={target_head16:?}"
                                        ));

                                        if let Some(target_head) = hotpatch_head16(target) {
                                            if let Some(abs) =
                                                hotpatch_abs_jmp_target_from_head16(&target_head)
                                            {
                                                let abs_module =
                                                    hotpatch_module_path_for_address(abs)
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

                        let use_direct = hotpatch_view_call_use_direct();
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

                            #[cfg(all(
                                feature = "ui-app-command-palette",
                                feature = "diagnostics"
                            ))]
                            let _ = measure_ui_driver_phase_for_frame(
                                _frame_perf_span_capture,
                                UiDriverPhase::ViewCommandPaletteOverlay,
                                false,
                                || render_command_palette_overlay_if_needed(driver, cx, &mut out),
                            );

                            #[cfg(all(
                                feature = "ui-app-command-palette",
                                not(feature = "diagnostics")
                            ))]
                            render_command_palette_overlay_if_needed(driver, cx, &mut out);

                            #[cfg(feature = "diagnostics")]
                            let _ = measure_ui_driver_phase_for_frame(
                                _frame_perf_span_capture,
                                UiDriverPhase::ViewPreferencesOverlay,
                                false,
                                || drive_preferences_overlay(cx),
                            );

                            #[cfg(not(feature = "diagnostics"))]
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

                            #[cfg(all(
                                feature = "ui-app-command-palette",
                                feature = "diagnostics"
                            ))]
                            let _ = measure_ui_driver_phase_for_frame(
                                _frame_perf_span_capture,
                                UiDriverPhase::ViewCommandPaletteOverlay,
                                false,
                                || render_command_palette_overlay_if_needed(driver, cx, &mut out),
                            );

                            #[cfg(all(
                                feature = "ui-app-command-palette",
                                not(feature = "diagnostics")
                            ))]
                            render_command_palette_overlay_if_needed(driver, cx, &mut out);

                            #[cfg(feature = "diagnostics")]
                            let _ = measure_ui_driver_phase_for_frame(
                                _frame_perf_span_capture,
                                UiDriverPhase::ViewPreferencesOverlay,
                                false,
                                || drive_preferences_overlay(cx),
                            );

                            #[cfg(not(feature = "diagnostics"))]
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
            )
        },
    );
    if let Some(elapsed) = view_elapsed {
        hitch_view_ms = Some(elapsed.as_millis() as u64);
    }
    hotpatch_trace_log(&format!(
        "ui_app_render: after render_root window={window:?} root={root:?}"
    ));
    hotpatch_trace_log(&format!("ui_app_render: after set_root window={window:?}"));
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::View,
    );

    let (_, overlay_elapsed) = measure_ui_driver_phase_for_frame(
        &mut perf_span_capture,
        UiDriverPhase::Overlay,
        hitch_config.is_some(),
        || {
            OverlayController::render(&mut state.ui, app, services, window, bounds);
        },
    );
    if let Some(elapsed) = overlay_elapsed {
        hitch_overlay_ms = Some(elapsed.as_millis() as u64);
    }
    hotpatch_trace_log(&format!(
        "ui_app_render: after overlay render window={window:?}"
    ));
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::Overlay,
    );

    #[cfg(feature = "diagnostics")]
    {
        crate::ui_diagnostics::render_diag_inspect_overlay(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            diag_inspection_active,
        );
        ui_app_observe_frame_stage(
            driver,
            app,
            window,
            state,
            bounds,
            scale_factor,
            UiAppFrameStage::DiagnosticsOverlay,
        );
    }
    state.root = Some(root);

    let diag_wants_semantics_snapshot = {
        #[cfg(feature = "diagnostics")]
        {
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_semantics_snapshot(window)
            })
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            false
        }
    };
    if diag_wants_semantics_snapshot {
        // Diagnostics scripts select targets by semantics bounds. Ensure we have a fresh semantics
        // snapshot for the current frame when the accessibility tree changed; paint-only animation
        // frames can keep using the existing snapshot.
        state.ui.request_semantics_snapshot_if_dirty();
    }
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::Semantics,
    );
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();

    let (_, layout_elapsed) = measure_ui_driver_phase_for_frame(
        &mut perf_span_capture,
        UiDriverPhase::Layout,
        hitch_config.is_some(),
        || {
            let mut frame =
                UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        },
    );
    let layout_total_ms: Option<u64> = layout_elapsed.map(|elapsed| elapsed.as_millis() as u64);
    hotpatch_trace_log(&format!(
        "ui_app_render: after layout_all window={window:?}"
    ));
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::Layout,
    );

    let hitch_layout_ms = layout_total_ms;

    let (_, paint_elapsed) = measure_ui_driver_phase_for_frame(
        &mut perf_span_capture,
        UiDriverPhase::Paint,
        hitch_config.is_some(),
        || {
            let mut frame =
                UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.paint_all(scene);
        },
    );
    if let Some(elapsed) = paint_elapsed {
        hitch_paint_ms = Some(elapsed.as_millis() as u64);
    }
    hotpatch_trace_log(&format!("ui_app_render: after paint_all window={window:?}"));
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::Paint,
    );

    #[cfg(feature = "diagnostics")]
    {
        let _ = measure_ui_driver_phase_for_frame(
            &mut perf_span_capture,
            UiDriverPhase::DiagnosticsDriveScript,
            false,
            || {
                // Drive scripted input after `paint_all()` so virtualization-heavy trees (e.g. VirtualList)
                // have their realized item subtrees available for hit-testing.
                //
                // The injected events will typically affect the *next* frame; the diagnostics recorder
                // below captures the current frame state.
                let semantics_snapshot = state.ui.semantics_snapshot_arc();
                let drive =
                    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                        svc.drive_script_for_window(
                            app,
                            services,
                            window,
                            bounds,
                            scale_factor,
                            Some(&mut state.ui),
                            semantics_snapshot.as_deref(),
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
                UiDiagnosticsService::with_script_injection_scope(|| {
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
                });

                // Scripted pointer steps often dispatch actions via `Effect::Command`. Flush those command
                // effects eagerly so:
                // - UI tree handlers run in the same render tick as the injected input, and
                // - command dispatch diagnostics are available to scripted `wait_command_dispatch_trace`
                //   without depending on runner-level effect timing.
                if injected_any {
                    UiDiagnosticsService::with_script_injection_scope(|| {
                        const MAX_SCRIPT_COMMAND_FLUSH_ROUNDS: usize = 8;
                        let mut deferred_effects: Vec<Effect> = Vec::new();
                        for _ in 0..MAX_SCRIPT_COMMAND_FLUSH_ROUNDS {
                            let effects = app.flush_effects();
                            if effects.is_empty() {
                                break;
                            }

                            let mut applied_any_command = false;
                            for effect in effects {
                                match effect {
                                    Effect::Command { window: w, command } => {
                                        if w.is_none() || w == Some(window) {
                                            applied_any_command = true;
                                            ui_app_handle_command(
                                                driver,
                                                WinitCommandContext {
                                                    app,
                                                    services,
                                                    window,
                                                    state,
                                                },
                                                command,
                                            );
                                        } else {
                                            deferred_effects
                                                .push(Effect::Command { window: w, command });
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
                    });
                    // Keep redraw requests from script-injected input alive after the eager
                    // command-only flush above has drained and re-queued deferred effects. Without
                    // this bridge, launched diagnostics can end a render tick with the script still
                    // waiting for geometry progression (for example `scroll_into_view`) but no
                    // guaranteed post-flush RAF wake-up.
                    app.request_redraw(window);
                    app.push_effect(Effect::RequestAnimationFrame(window));
                }
            },
        );
        ui_app_observe_frame_stage(
            driver,
            app,
            window,
            state,
            bounds,
            scale_factor,
            UiAppFrameStage::DiagnosticsDriveScript,
        );

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let real_perf_span_start_us = perf_span_capture
                .as_ref()
                .map(UiDriverPerfSpanCapture::frame_elapsed_us);
            if let Some(capture) = perf_span_capture.as_mut() {
                capture.record_for_window(svc, window);
            }
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &mut state.ui,
                element_runtime,
                real_perf_span_start_us,
                scene,
            );
            let defer_dump_until_renderer_perf =
                std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|v| !v.is_empty());
            if !defer_dump_until_renderer_perf && let Some(dir) = svc.maybe_dump_if_triggered() {
                #[cfg(feature = "tracing")]
                tracing::info!(window = ?window, out_dir = %dir.display(), "ui diagnostics dumped");
            }
            if svc.poll_exit_trigger() {
                app.push_effect(Effect::QuitApp);
            } else if svc.is_enabled() {
                // Diagnostics are driven per-window after paint, but multi-window scripts may
                // need a non-active window to continue ticking (e.g. tear-off creates a new
                // window and focus shifts). Keep all known windows in the RAF set so scripted
                // playback and timeouts remain deterministic.
                for w in svc.known_windows().iter().copied() {
                    app.request_redraw(w);
                    app.push_effect(Effect::RequestAnimationFrame(w));
                }
            }
        });
        ui_app_observe_frame_stage(
            driver,
            app,
            window,
            state,
            bounds,
            scale_factor,
            UiAppFrameStage::DiagnosticsSnapshot,
        );
    }

    PostFrameUiFocusLifecycle::finish_frame(app, services, window, &mut state.ui);

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
    ui_app_observe_frame_stage(
        driver,
        app,
        window,
        state,
        bounds,
        scale_factor,
        UiAppFrameStage::End,
    );
    RENDER_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
}

fn ui_app_scene_chunk_manifest<S>(
    _driver: &mut UiAppDriver<S>,
    state: &mut UiAppWindowState<S>,
) -> fret_core::SceneChunkManifest {
    state.ui.scene_chunk_manifest()
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

    // Ensure newly created windows get at least one frame. This is particularly important for
    // diagnostics/scripted playback, which discovers windows opportunistically during per-window
    // ticks (e.g. multi-window tear-off scripts waiting for `known_window_count_ge`).
    app.request_redraw(new_window);
    app.push_effect(Effect::RequestAnimationFrame(new_window));

    // Seed `WindowInputContextService` for diagnostics runs so `KnownWindowCount*` predicates can
    // observe window creation/closure without depending on a per-window input pass having run.
    //
    // The UI runtime will overwrite this placeholder snapshot on the first real dispatch.
    let diag_env_enabled = std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty());
    let diag_service_enabled = {
        #[cfg(feature = "diagnostics")]
        {
            app.global::<UiDiagnosticsService>()
                .is_some_and(|svc| svc.is_enabled())
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            false
        }
    };
    if diag_env_enabled || diag_service_enabled {
        app.with_global_mut(
            fret_runtime::WindowInputContextService::default,
            |svc, _app| {
                svc.set_snapshot(new_window, fret_runtime::InputContext::default());
            },
        );
    }
}

fn ui_app_before_close_window<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
) -> bool {
    let allow_close = match driver.before_close_window {
        None => true,
        Some(f) => {
            #[cfg(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32")))]
            {
                let mut hot = subsecond::HotFn::current(f);
                hot.call((app, window))
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                f(app, window)
            }
        }
    };
    if allow_close {
        PostFrameUiFocusLifecycle::clear_window(app, window);
    }
    allow_close
}

fn ui_app_accessibility_snapshot<S>(
    _driver: &mut UiAppDriver<S>,
    _app: &mut App,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
) -> Option<std::sync::Arc<fret_core::SemanticsSnapshot>> {
    // Accessibility snapshots are requested by the runner after layout. Request when the tree is
    // semantically dirty (or when no snapshot exists yet) so accessibility activation does not turn
    // paint-only animation frames into full semantics rebuilds.
    state.ui.request_semantics_snapshot_if_dirty();
    state.ui.semantics_snapshot_arc()
}

fn ui_app_accessibility_focus<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
) {
    fret_ui_app::accessibility_actions::focus(&mut state.ui, app, target);
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

fn ui_app_accessibility_set_value_numeric<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
    value: f64,
) {
    fret_ui_app::accessibility_actions::set_value_numeric(
        &mut state.ui,
        app,
        services,
        target,
        value,
    );
}

fn ui_app_accessibility_decrement<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
) {
    fret_ui_app::accessibility_actions::decrement(&mut state.ui, app, services, target);
}

fn ui_app_accessibility_increment<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
) {
    fret_ui_app::accessibility_actions::increment(&mut state.ui, app, services, target);
}

fn ui_app_accessibility_scroll_by<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
    dx: f64,
    dy: f64,
) {
    fret_ui_app::accessibility_actions::scroll_by(&mut state.ui, app, target, dx, dy);
}

fn ui_app_accessibility_set_text_selection<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
    anchor: u32,
    focus: u32,
) {
    fret_ui_app::accessibility_actions::set_text_selection(
        &mut state.ui,
        app,
        services,
        target,
        anchor,
        focus,
    );
}

fn ui_app_accessibility_replace_selected_text<S>(
    _driver: &mut UiAppDriver<S>,
    app: &mut App,
    services: &mut dyn UiServices,
    _window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    target: NodeId,
    value: &str,
) {
    fret_ui_app::accessibility_actions::replace_selected_text(
        &mut state.ui,
        app,
        services,
        target,
        value,
    );
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

// The explicit return keeps the default diagnostics sink from running after a custom hook.
#[allow(clippy::needless_return)]
fn ui_app_renderer_perf_sample<S>(
    driver: &mut UiAppDriver<S>,
    app: &mut App,
    window: AppWindowId,
    state: &mut UiAppWindowState<S>,
    sample: Option<fret_render::RendererPerfFrameSample>,
) {
    if let Some(f) = driver.renderer_perf_sample {
        f(app, window, &mut state.ui, &mut state.state, sample);
        return;
    }

    #[cfg(feature = "diagnostics")]
    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        if let Some(sample) = sample {
            svc.patch_latest_renderer_perf_sample(window, sample);
        }
        if let Some(dir) = svc.maybe_dump_if_triggered() {
            #[cfg(feature = "tracing")]
            tracing::info!(window = ?window, out_dir = %dir.display(), "ui diagnostics dumped");
        }
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::any::TypeId;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);
    static MIDDLEWARE_SEQ: AtomicUsize = AtomicUsize::new(0);
    static USER_SEQ: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn post_frame_ui_focus_requests_preserve_fifo_order() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let first = fret_ui::elements::GlobalElementId(1);
        let second = fret_ui::elements::GlobalElementId(2);
        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::Unchanged(first),
            Some(first),
            Some(CommandId::from("focus.first")),
        );
        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::Unchanged(second),
            Some(second),
            Some(CommandId::from("focus.second")),
        );

        assert!(take_ready_post_frame_ui_focus_requests(&mut app, window).is_empty());
        promote_post_frame_ui_focus_requests(&mut app, window);
        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::NoLiveFocus,
            None,
            Some(CommandId::from("focus.third")),
        );
        assert_eq!(
            take_ready_post_frame_ui_focus_requests(&mut app, window),
            vec![
                PostFrameUiFocusRequest {
                    guard: PostFrameUiFocusGuard::Unchanged(first),
                    target: Some(first),
                    fallback_command: Some(CommandId::from("focus.first")),
                },
                PostFrameUiFocusRequest {
                    guard: PostFrameUiFocusGuard::Unchanged(second),
                    target: Some(second),
                    fallback_command: Some(CommandId::from("focus.second")),
                },
            ]
        );
        assert!(take_ready_post_frame_ui_focus_requests(&mut app, window).is_empty());
        promote_post_frame_ui_focus_requests(&mut app, window);
        assert_eq!(
            take_ready_post_frame_ui_focus_requests(&mut app, window),
            vec![PostFrameUiFocusRequest {
                guard: PostFrameUiFocusGuard::NoLiveFocus,
                target: None,
                fallback_command: Some(CommandId::from("focus.third")),
            }]
        );
    }

    #[test]
    fn post_frame_ui_focus_requests_are_cleared_when_a_window_closes() {
        let mut app = App::new();
        let window = AppWindowId::default();
        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::NoLiveFocus,
            None,
            Some(CommandId::from("focus.active")),
        );

        PostFrameUiFocusLifecycle::clear_window(&mut app, window);
        promote_post_frame_ui_focus_requests(&mut app, window);

        assert!(take_ready_post_frame_ui_focus_requests(&mut app, window).is_empty());
    }

    #[test]
    fn post_frame_ui_focus_request_yields_to_newer_focus() {
        let original = fret_ui::elements::GlobalElementId(1);
        let newer = fret_ui::elements::GlobalElementId(2);
        let request = PostFrameUiFocusRequest {
            guard: PostFrameUiFocusGuard::Unchanged(original),
            target: Some(original),
            fallback_command: Some(CommandId::from("focus.original")),
        };

        assert!(post_frame_ui_focus_request_can_apply(&request, false, None));
        assert!(post_frame_ui_focus_request_can_apply(
            &request,
            true,
            Some(original)
        ));
        assert!(!post_frame_ui_focus_request_can_apply(
            &request,
            true,
            Some(newer)
        ));
        assert!(!post_frame_ui_focus_request_can_apply(&request, true, None));

        let fallback_request = PostFrameUiFocusRequest {
            guard: PostFrameUiFocusGuard::Unchanged(original),
            target: None,
            fallback_command: Some(CommandId::from("focus.fallback")),
        };
        assert!(post_frame_ui_focus_request_can_apply(
            &fallback_request,
            true,
            Some(original)
        ));
        assert!(!post_frame_ui_focus_request_can_apply(
            &fallback_request,
            true,
            Some(newer)
        ));

        let empty_focus_request = PostFrameUiFocusRequest {
            guard: PostFrameUiFocusGuard::NoLiveFocus,
            target: None,
            fallback_command: Some(CommandId::from("focus.empty")),
        };
        assert!(post_frame_ui_focus_request_can_apply(
            &empty_focus_request,
            false,
            None
        ));
        assert!(!post_frame_ui_focus_request_can_apply(
            &empty_focus_request,
            true,
            Some(newer)
        ));

        let authoritative_request = PostFrameUiFocusRequest {
            guard: PostFrameUiFocusGuard::Authoritative,
            target: None,
            fallback_command: Some(CommandId::from("focus.authoritative")),
        };
        assert!(post_frame_ui_focus_request_can_apply(
            &authoritative_request,
            true,
            Some(newer)
        ));
    }

    fn init_window(app: &mut App, window: AppWindowId) -> u8 {
        let _ = (app, window);
        0
    }

    fn view(_cx: &mut ElementContext<'_, App>, _st: &mut u8) -> ViewElements {
        ViewElements::default()
    }

    #[derive(Default)]
    struct FakeUiServices;

    impl fret_core::TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct FrameHarnessSmokeState {
        observations: Vec<UiAppFrameObservation>,
        rendered: bool,
        dirty: bool,
        dirty_close_blocked: bool,
        commands: Vec<String>,
        app_command_sources: Vec<Option<u64>>,
    }

    impl UiAppFrameStageSink for FrameHarnessSmokeState {
        fn record_frame_stage(&mut self, observation: UiAppFrameObservation) {
            self.observations.push(observation);
        }
    }

    fn frame_harness_init(_app: &mut App, _window: AppWindowId) -> FrameHarnessSmokeState {
        FrameHarnessSmokeState {
            dirty: true,
            ..Default::default()
        }
    }

    fn frame_harness_view(
        _cx: &mut ElementContext<'_, App>,
        state: &mut FrameHarnessSmokeState,
    ) -> ViewElements {
        state.rendered = true;
        ViewElements::default()
    }

    #[derive(Default)]
    struct FocusBatchHarnessState {
        first: Option<fret_ui::elements::GlobalElementId>,
        second: Option<fret_ui::elements::GlobalElementId>,
        third: Option<fret_ui::elements::GlobalElementId>,
    }

    fn focus_batch_init(_app: &mut App, _window: AppWindowId) -> FocusBatchHarnessState {
        FocusBatchHarnessState::default()
    }

    fn focus_batch_view(
        cx: &mut ElementContext<'_, App>,
        state: &mut FocusBatchHarnessState,
    ) -> ViewElements {
        let props = fret_ui::element::PressableProps {
            focusable: true,
            ..Default::default()
        };
        let first = cx.keyed("focus-batch-first", |cx| {
            cx.pressable(props.clone(), |_cx, _state| Vec::new())
        });
        let second = cx.keyed("focus-batch-second", |cx| {
            cx.pressable(props.clone(), |_cx, _state| Vec::new())
        });
        let third = cx.keyed("focus-batch-third", |cx| {
            cx.pressable(props, |_cx, _state| Vec::new())
        });
        state.first = Some(first.id);
        state.second = Some(second.id);
        state.third = Some(third.id);
        vec![first, second, third].into()
    }

    fn frame_harness_on_command(
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut UiTree<App>,
        state: &mut FrameHarnessSmokeState,
        command: &CommandId,
    ) {
        state.commands.push(command.as_str().to_owned());
    }

    fn frame_harness_on_command_before_ui(
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut UiTree<App>,
        state: &mut FrameHarnessSmokeState,
        command: &CommandId,
    ) -> bool {
        state.commands.push(command.as_str().to_owned());
        true
    }

    fn frame_harness_legacy_hook_consumes_source(
        app: &mut App,
        _services: &mut dyn UiServices,
        window: AppWindowId,
        _ui: &mut UiTree<App>,
        state: &mut FrameHarnessSmokeState,
        command: &CommandId,
    ) -> bool {
        let source = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| service.consume(window, app.tick_id(), command),
        );
        state
            .app_command_sources
            .push(source.and_then(|source| source.element));
        true
    }

    fn frame_harness_on_app_command_before_ui(
        _app: &mut App,
        _window: AppWindowId,
        state: &mut FrameHarnessSmokeState,
        command: &CommandId,
        context: UiAppCommandBeforeUiContext<'_>,
    ) -> bool {
        state.commands.push(command.as_str().to_owned());
        state.app_command_sources.push(context.source.element);
        assert!(!context.ui_has_modal);
        assert!(!context.source_is_within_active_input_barrier_scope);
        true
    }

    fn frame_harness_on_event(
        _app: &mut App,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut UiTree<App>,
        state: &mut FrameHarnessSmokeState,
        event: &Event,
    ) {
        if matches!(event, Event::WindowCloseRequested) && state.dirty {
            state.dirty_close_blocked = true;
        }
    }

    fn frame_harness_bounds() -> Rect {
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(640.0), Px(480.0)),
        )
    }

    fn expected_frame_stages() -> Vec<UiAppFrameStage> {
        let mut stages = vec![
            UiAppFrameStage::Begin,
            UiAppFrameStage::View,
            UiAppFrameStage::Overlay,
            UiAppFrameStage::Semantics,
            UiAppFrameStage::Layout,
            UiAppFrameStage::Paint,
        ];

        #[cfg(feature = "diagnostics")]
        {
            stages.insert(3, UiAppFrameStage::DiagnosticsOverlay);
            stages.push(UiAppFrameStage::DiagnosticsDriveScript);
            stages.push(UiAppFrameStage::DiagnosticsSnapshot);
        }

        stages.push(UiAppFrameStage::End);
        stages
    }

    fn render_test_frame<S>(
        driver: &mut UiAppDriver<S>,
        app: &mut App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        state: &mut UiAppWindowState<S>,
    ) {
        let mut scene = fret_core::Scene::default();
        ui_app_render(
            driver,
            WinitRenderContext {
                app,
                services,
                window,
                state,
                bounds: frame_harness_bounds(),
                scale_factor: 1.0,
                scene: &mut scene,
            },
        );
    }

    fn middleware(
        app: &mut App,
        window: AppWindowId,
        ui: &mut UiTree<App>,
        st: &mut u8,
        changed: &[TypeId],
    ) {
        let _ = (app, window, ui, st, changed);
        let idx = SEQ.fetch_add(1, Ordering::SeqCst) + 1;
        MIDDLEWARE_SEQ.store(idx, Ordering::SeqCst);
    }

    fn user_hook(
        app: &mut App,
        window: AppWindowId,
        ui: &mut UiTree<App>,
        st: &mut u8,
        changed: &[TypeId],
    ) {
        let _ = (app, window, ui, st, changed);
        let idx = SEQ.fetch_add(1, Ordering::SeqCst) + 1;
        USER_SEQ.store(idx, Ordering::SeqCst);
    }

    #[test]
    fn global_changes_middleware_runs_before_user_hook() {
        SEQ.store(0, Ordering::SeqCst);
        MIDDLEWARE_SEQ.store(0, Ordering::SeqCst);
        USER_SEQ.store(0, Ordering::SeqCst);

        let mut app = App::new();
        let window = AppWindowId::default();
        let mut state = UiAppWindowState {
            ui: UiTree::default(),
            root: None,
            state: 0,
            pending_invalidation: PendingInvalidationBatch::default(),
        };

        let mut driver = UiAppDriver::new("test", init_window, view)
            .on_global_changes_middleware(middleware)
            .on_global_changes(user_hook);

        let changed = [TypeId::of::<fret_core::WindowMetricsService>()];
        ui_app_handle_global_changes(
            &mut driver,
            WinitWindowContext {
                app: &mut app,
                window,
                state: &mut state,
            },
            &changed,
        );

        let middleware_seq = MIDDLEWARE_SEQ.load(Ordering::SeqCst);
        let user_seq = USER_SEQ.load(Ordering::SeqCst);
        assert_ne!(middleware_seq, 0);
        assert_ne!(user_seq, 0);
        assert!(middleware_seq < user_seq);
    }

    #[test]
    fn frame_stage_sink_observes_ui_app_render_order() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "frame-harness-smoke",
            frame_harness_init,
            frame_harness_view,
        )
        .record_frame_stages();
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);

        render_test_frame(&mut driver, &mut app, &mut services, window, &mut state);

        let stages = state
            .state
            .observations
            .iter()
            .map(|observation| observation.stage)
            .collect::<Vec<_>>();
        assert_eq!(stages, expected_frame_stages());
        assert!(state.state.rendered);
        for observation in &state.state.observations {
            assert_eq!(observation.window, window);
            assert_eq!(observation.bounds, frame_harness_bounds());
            assert_eq!(observation.scale_factor, 1.0);
            assert_eq!(observation.tick_id, app.tick_id());
            assert_eq!(observation.frame_id, app.frame_id());
        }
    }

    #[test]
    fn post_frame_focus_batch_rechecks_guard_after_each_restore() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new("focus-batch", focus_batch_init, focus_batch_view);
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);
        render_test_frame(&mut driver, &mut app, &mut services, window, &mut state);

        let first = state.state.first.expect("first focus target");
        let second = state.state.second.expect("second focus target");
        let third = state.state.third.expect("third focus target");
        let first_node = state
            .ui
            .live_attached_node_for_element(&mut app, first)
            .expect("first live node");
        let second_node = state
            .ui
            .live_attached_node_for_element(&mut app, second)
            .expect("second live node");
        state.ui.set_focus(Some(first_node));

        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::Authoritative,
            Some(second),
            None,
        );
        defer_ui_focus_until_after_frame(
            &mut app,
            window,
            PostFrameUiFocusGuard::Unchanged(first),
            Some(third),
            None,
        );
        PostFrameUiFocusLifecycle::begin_frame(&mut app, window);
        PostFrameUiFocusLifecycle::finish_frame(&mut app, &mut services, window, &mut state.ui);

        assert_eq!(state.ui.focus(), Some(second_node));
    }

    #[test]
    fn frame_harness_workspace_smoke_covers_command_close_diagnostics_and_sequence() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "workspace-frame-smoke",
            frame_harness_init,
            frame_harness_view,
        )
        .close_on_window_close_requested(false)
        .on_command(frame_harness_on_command)
        .on_event(frame_harness_on_event)
        .record_frame_stages();
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);

        ui_app_handle_command(
            &mut driver,
            WinitCommandContext {
                app: &mut app,
                services: &mut services,
                window,
                state: &mut state,
            },
            CommandId::new("workspace.smoke.toggle"),
        );
        ui_app_handle_event(
            &mut driver,
            WinitEventContext {
                app: &mut app,
                services: &mut services,
                window,
                state: &mut state,
            },
            &Event::WindowCloseRequested,
        );
        let effects_after_close = app.flush_effects();
        assert!(
            !effects_after_close.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Window(fret_app::WindowRequest::Close(close_window))
                        if *close_window == window
                )
            }),
            "dirty close smoke should not emit a close-window effect"
        );

        render_test_frame(&mut driver, &mut app, &mut services, window, &mut state);

        assert_eq!(
            state.state.commands,
            vec!["workspace.smoke.toggle".to_owned()]
        );
        assert!(state.state.dirty_close_blocked);
        assert!(state.state.rendered);
        let stages = state
            .state
            .observations
            .iter()
            .map(|observation| observation.stage)
            .collect::<Vec<_>>();
        assert_eq!(stages, expected_frame_stages());
        assert!(
            stages
                .windows(2)
                .any(|pair| pair[0] == UiAppFrameStage::Layout
                    && pair[1] == UiAppFrameStage::Paint)
        );

        #[cfg(feature = "diagnostics")]
        assert!(stages.contains(&UiAppFrameStage::DiagnosticsSnapshot));
    }

    #[test]
    fn command_before_ui_trace_preserves_source_and_entry_focus_state() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "command-before-ui-smoke",
            frame_harness_init,
            frame_harness_view,
        )
        .on_command_before_ui(frame_harness_on_command_before_ui);
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);
        let cases = [
            (
                "workspace.smoke.before_ui.keyboard",
                fret_runtime::CommandDispatchSourceKindV1::Keyboard,
                false,
            ),
            (
                "workspace.smoke.before_ui.shortcut",
                fret_runtime::CommandDispatchSourceKindV1::Shortcut,
                true,
            ),
        ];

        for (index, (command_name, source_kind, expected_started_from_focus)) in
            cases.into_iter().enumerate()
        {
            let command = CommandId::new(command_name);
            let source = fret_runtime::CommandDispatchSourceV1 {
                kind: source_kind,
                element: Some(42 + index as u64),
                test_id: Some(std::sync::Arc::from("workspace.smoke.trigger")),
            };
            state
                .ui
                .set_focus(expected_started_from_focus.then_some(fret_core::NodeId::default()));

            app.with_global_mut(
                fret_runtime::WindowPendingCommandDispatchSourceService::default,
                |service, app| {
                    service.record(window, app.tick_id(), command.clone(), source.clone());
                },
            );
            ui_app_handle_command(
                &mut driver,
                WinitCommandContext {
                    app: &mut app,
                    services: &mut services,
                    window,
                    state: &mut state,
                },
                command.clone(),
            );

            let decisions = app
                .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
                .expect("driver-handled command should record diagnostics")
                .snapshot_since(window, 0, 10);
            let decision = decisions
                .iter()
                .find(|decision| decision.command == command)
                .expect("expected a trace entry for the before-UI hook");
            assert!(decision.handled);
            assert!(decision.handled_by_driver);
            assert_eq!(
                decision.handled_by_scope,
                Some(fret_runtime::CommandScope::Window)
            );
            assert_eq!(decision.source, source);
            assert_eq!(
                decision.started_from_focus, expected_started_from_focus,
                "driver traces must use the focus state captured at dispatch entry"
            );

            let pending = app.with_global_mut(
                fret_runtime::WindowPendingCommandDispatchSourceService::default,
                |service, app| service.consume(window, app.tick_id(), &command),
            );
            assert_eq!(pending, None);
        }

        assert_eq!(
            state.state.commands,
            cases
                .into_iter()
                .map(|(command, _, _)| command.to_owned())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn repeated_same_command_preserves_fifo_sources_across_app_hook_cleanup() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "command-source-fifo-smoke",
            frame_harness_init,
            frame_harness_view,
        )
        .on_app_command_before_ui(frame_harness_on_app_command_before_ui);
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);
        let command = CommandId::new("workspace.smoke.before_ui.repeated");

        app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| {
                for element in [1, 2] {
                    service.record(
                        window,
                        app.tick_id(),
                        command.clone(),
                        fret_runtime::CommandDispatchSourceV1 {
                            kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                            element: Some(element),
                            test_id: None,
                        },
                    );
                }
            },
        );

        for _ in 0..2 {
            ui_app_handle_command(
                &mut driver,
                WinitCommandContext {
                    app: &mut app,
                    services: &mut services,
                    window,
                    state: &mut state,
                },
                command.clone(),
            );
        }

        let source_elements = app
            .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
            .expect("driver-handled commands should record diagnostics")
            .snapshot_since(window, 0, 10)
            .into_iter()
            .filter(|decision| decision.command == command)
            .map(|decision| decision.source.element)
            .collect::<Vec<_>>();
        assert_eq!(source_elements, vec![Some(1), Some(2)]);
        assert_eq!(state.state.app_command_sources, vec![Some(1), Some(2)]);
        let pending = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| service.consume(window, app.tick_id(), &command),
        );
        assert_eq!(pending, None);
    }

    #[test]
    fn repeated_same_command_preserves_fifo_when_legacy_hook_consumes_its_source() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "legacy-command-source-fifo-smoke",
            frame_harness_init,
            frame_harness_view,
        )
        .on_command_before_ui(frame_harness_legacy_hook_consumes_source);
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);
        let command = CommandId::new("workspace.smoke.before_ui.legacy-repeated");

        app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| {
                for element in [1, 2] {
                    service.record(
                        window,
                        app.tick_id(),
                        command.clone(),
                        fret_runtime::CommandDispatchSourceV1 {
                            kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                            element: Some(element),
                            test_id: None,
                        },
                    );
                }
            },
        );

        for _ in 0..2 {
            ui_app_handle_command(
                &mut driver,
                WinitCommandContext {
                    app: &mut app,
                    services: &mut services,
                    window,
                    state: &mut state,
                },
                command.clone(),
            );
        }

        assert_eq!(state.state.app_command_sources, vec![Some(1), Some(2)]);
        let traced_sources = app
            .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
            .expect("legacy hook commands should record diagnostics")
            .snapshot_since(window, 0, 10)
            .into_iter()
            .filter(|decision| decision.command == command)
            .map(|decision| decision.source.element)
            .collect::<Vec<_>>();
        assert_eq!(traced_sources, vec![Some(1), Some(2)]);
        let pending = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| service.consume(window, app.tick_id(), &command),
        );
        assert_eq!(pending, None);
    }

    #[test]
    fn command_before_first_frame_discards_only_its_restored_source() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut driver = UiAppDriver::new(
            "pre-frame-command-source-smoke",
            frame_harness_init,
            frame_harness_view,
        );
        let mut state = ui_app_create_window_state(&mut driver, &mut app, window);
        let command = CommandId::new("workspace.smoke.before_first_frame");

        app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| {
                for element in [1, 2] {
                    service.record(
                        window,
                        app.tick_id(),
                        command.clone(),
                        fret_runtime::CommandDispatchSourceV1 {
                            kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                            element: Some(element),
                            test_id: None,
                        },
                    );
                }
            },
        );

        ui_app_handle_command(
            &mut driver,
            WinitCommandContext {
                app: &mut app,
                services: &mut services,
                window,
                state: &mut state,
            },
            command.clone(),
        );

        let remaining = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| service.consume(window, app.tick_id(), &command),
        );
        assert_eq!(remaining.and_then(|source| source.element), Some(2));
        let exhausted = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchSourceService::default,
            |service, app| service.consume(window, app.tick_id(), &command),
        );
        assert_eq!(exhausted, None);
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_records_frame_relative_driver_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::View.perf_span_name(),
            UiDriverPhase::View.perf_span_phase(),
            "ui_app_driver",
            12,
            Duration::from_micros(34),
        );

        let spans = capture.take_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "fret.ui.view");
        assert_eq!(spans[0].cat, "ui.driver");
        assert_eq!(spans[0].start_us, 12);
        assert_eq!(spans[0].dur_us, 34);
        assert_eq!(
            spans[0]
                .args
                .as_ref()
                .and_then(|args| args.get("phase"))
                .and_then(|v| v.as_str()),
            Some("view")
        );
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_preserves_sub_microsecond_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::View.perf_span_name(),
            UiDriverPhase::View.perf_span_phase(),
            "ui_app_driver",
            0,
            Duration::from_nanos(1),
        );

        let spans = capture.take_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "fret.ui.view");
        assert_eq!(spans[0].dur_us, 1);
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_ignores_zero_duration_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::View.perf_span_name(),
            UiDriverPhase::View.perf_span_phase(),
            "ui_app_driver",
            0,
            Duration::ZERO,
        );

        let spans = capture.take_spans();
        assert!(spans.is_empty());
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_records_diagnostics_drive_script_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::DiagnosticsDriveScript.perf_span_name(),
            UiDriverPhase::DiagnosticsDriveScript.perf_span_phase(),
            "ui_app_driver",
            56,
            Duration::from_micros(78),
        );

        let spans = capture.take_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "fret.ui.diagnostics.drive_script");
        assert_eq!(spans[0].cat, "ui.driver");
        assert_eq!(spans[0].start_us, 56);
        assert_eq!(spans[0].dur_us, 78);
        assert_eq!(
            spans[0]
                .args
                .as_ref()
                .and_then(|args| args.get("phase"))
                .and_then(|v| v.as_str()),
            Some("diagnostics_drive_script")
        );
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_records_view_preferences_overlay_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::ViewPreferencesOverlay.perf_span_name(),
            UiDriverPhase::ViewPreferencesOverlay.perf_span_phase(),
            "ui_app_driver",
            89,
            Duration::from_micros(123),
        );

        let spans = capture.take_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "fret.ui.view.preferences_overlay");
        assert_eq!(spans[0].cat, "ui.driver");
        assert_eq!(spans[0].start_us, 89);
        assert_eq!(spans[0].dur_us, 123);
        assert_eq!(
            spans[0]
                .args
                .as_ref()
                .and_then(|args| args.get("phase"))
                .and_then(|v| v.as_str()),
            Some("view_preferences_overlay")
        );
    }

    #[cfg(all(feature = "diagnostics", feature = "ui-app-command-palette"))]
    #[test]
    fn perf_span_capture_records_view_command_palette_overlay_phase() {
        let mut capture = UiDriverPerfSpanCapture::new_for_test(Instant::now());

        capture.push_phase(
            UiDriverPhase::ViewCommandPaletteOverlay.perf_span_name(),
            UiDriverPhase::ViewCommandPaletteOverlay.perf_span_phase(),
            "ui_app_driver",
            177,
            Duration::from_micros(211),
        );

        let spans = capture.take_spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "fret.ui.view.command_palette_overlay");
        assert_eq!(spans[0].cat, "ui.driver");
        assert_eq!(spans[0].start_us, 177);
        assert_eq!(spans[0].dur_us, 211);
        assert_eq!(
            spans[0]
                .args
                .as_ref()
                .and_then(|args| args.get("phase"))
                .and_then(|v| v.as_str()),
            Some("view_command_palette_overlay")
        );
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn perf_span_capture_allows_nested_phase_recording() {
        let mut capture = Some(UiDriverPerfSpanCapture::new_for_test(Instant::now()));

        let (value, _) = measure_ui_driver_phase_for_frame_with_capture(
            &mut capture,
            UiDriverPhase::View,
            false,
            |capture| {
                capture.as_mut().expect("active capture").push_phase(
                    UiDriverPhase::ViewPreferencesOverlay.perf_span_name(),
                    UiDriverPhase::ViewPreferencesOverlay.perf_span_phase(),
                    "ui_app_driver",
                    144,
                    Duration::from_micros(55),
                );
                7
            },
        );

        assert_eq!(value, 7);
        let spans = capture.as_mut().expect("capture").take_spans();
        assert!(
            spans
                .iter()
                .any(|span| span.name == "fret.ui.view.preferences_overlay"
                    && span.start_us == 144
                    && span.dur_us == 55),
            "expected nested preferences overlay span in {spans:?}"
        );
    }
}
