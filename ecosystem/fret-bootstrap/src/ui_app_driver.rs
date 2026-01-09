use fret_app::App;
use fret_app::CommandId;
use fret_app::Effect;
use fret_core::{AppWindowId, Event, NodeId, UiServices, ViewportInputEvent};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitGlobalContext,
    WinitHotReloadContext, WinitRenderContext, WinitWindowContext,
};
use fret_ui::declarative::RenderRootContext;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiFrameCx, UiTree};
use fret_ui_kit::OverlayController;
use std::cell::Cell;

type ViewFn<S> = for<'a> fn(&mut ElementContext<'a, App>, &mut S) -> Vec<AnyElement>;

type EventHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &Event);

type CommandHookFn<S> =
    fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S, &CommandId);

type HotReloadHookFn<S> = fn(&mut App, &mut dyn UiServices, AppWindowId, &mut UiTree<App>, &mut S);

type ModelChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[fret_app::ModelId]);
type GlobalChangesHookFn<S> =
    fn(&mut App, AppWindowId, &mut UiTree<App>, &mut S, &[std::any::TypeId]);

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
            on_hot_reload_window: None,
            on_model_changes: None,
            on_global_changes: None,
            window_create_spec: None,
            window_created: None,
            before_close_window: None,
            handle_global_command: None,
            viewport_input: None,
            dock_op: None,
        }
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
        })
    }
}

pub struct UiAppWindowState<S> {
    pub ui: UiTree<App>,
    pub root: Option<NodeId>,
    pub state: S,
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

    if state.ui.dispatch_command(app, services, &command) {
        return;
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
    state.ui.propagate_model_changes(app, changed);
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
    state.ui.propagate_global_changes(app, changed);
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

                let out = if use_direct {
                    (driver.view)(cx, &mut state.state)
                } else {
                    let mut hot = subsecond::HotFn::current(driver.view);
                    hot.call((cx, &mut state.state))
                };
                hotpatch_trace_log(&format!(
                    "ui_app_render: view end window={window:?} depth={view_depth}"
                ));
                VIEW_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                out
            }

            #[cfg(not(all(feature = "hotpatch-subsecond", not(target_arch = "wasm32"))))]
            {
                let out = (driver.view)(cx, &mut state.state);
                hotpatch_trace_log(&format!(
                    "ui_app_render: view end window={window:?} depth={view_depth}"
                ));
                VIEW_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                out
            }
        },
    );
    hotpatch_trace_log(&format!(
        "ui_app_render: after render_root window={window:?} root={root:?}"
    ));
    state.ui.set_root(root);
    hotpatch_trace_log(&format!("ui_app_render: after set_root window={window:?}"));
    OverlayController::render(&mut state.ui, app, services, window, bounds);
    hotpatch_trace_log(&format!(
        "ui_app_render: after overlay render window={window:?}"
    ));
    state.root = Some(root);

    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();

    let mut frame = UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    hotpatch_trace_log(&format!(
        "ui_app_render: after layout_all window={window:?}"
    ));
    frame.paint_all(scene);
    hotpatch_trace_log(&format!("ui_app_render: after paint_all window={window:?}"));

    hotpatch_trace_log(&format!(
        "ui_app_render: end window={window:?} depth={render_depth}"
    ));
    RENDER_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
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

fn reset_ui_tree_for_hotpatch(app: &mut App, window: AppWindowId, ui: &mut UiTree<App>) {
    let mut new_ui: UiTree<App> = UiTree::new();
    new_ui.set_window(window);

    let old = std::mem::replace(ui, new_ui);
    if hotpatch_drop_old_state() {
        drop(old);
    } else {
        std::mem::forget(old);
    }

    app.with_global_mut(fret_ui::InternalDragRouteService::default, |svc, _app| {
        svc.clear_window(window);
    });
}

fn hotpatch_drop_old_state() -> bool {
    std::env::var_os("FRET_HOTPATCH_DROP_OLD_STATE").is_some_and(|v| !v.is_empty())
}
