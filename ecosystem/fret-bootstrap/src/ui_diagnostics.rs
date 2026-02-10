use fret_app::{App, Effect, ModelId};
use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, MouseButton, MouseButtons, NodeId, Point, PointerEvent,
    PointerId, PointerType, Px, Rect, Scene, SemanticsRole,
};
#[cfg(feature = "diagnostics-ws")]
use fret_diag_protocol::{DevtoolsBundleDumpV1, DevtoolsBundleDumpedV1, DiagTransportMessageV1};
use fret_diag_protocol::{
    FilesystemCapabilitiesV1, UiActionScriptV1, UiActionScriptV2, UiActionStepV2,
    UiInspectConfigV1, UiKeyModifiersV1, UiMouseButtonV1, UiOptionalRootStateV1, UiPaddingInsetsV1,
    UiHitTestScopeRootEvidenceV1, UiHitTestTraceEntryV1, UiPointV1, UiPredicateV1, UiRectV1,
    UiRoleAndNameV1, UiScriptEvidenceV1, UiScriptResultV1, UiScriptStageV1,
    UiSelectorResolutionCandidateV1, UiSelectorResolutionTraceEntryV1, UiSelectorV1,
};
use fret_ui::elements::ElementRuntime;
use fret_ui::{Invalidation, UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiTree};
use serde::{Deserialize, Serialize};
use slotmap::{Key as _, KeyData};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(feature = "diagnostics-ws")]
use crate::ui_diagnostics_ws_bridge::UiDiagnosticsWsBridge;

#[derive(Debug, Clone)]
pub struct UiDiagnosticsConfig {
    pub enabled: bool,
    pub out_dir: PathBuf,
    pub trigger_path: PathBuf,
    pub ready_path: PathBuf,
    pub exit_path: PathBuf,
    pub max_events: usize,
    pub max_snapshots: usize,
    pub capture_semantics: bool,
    pub screenshots_enabled: bool,
    pub screenshot_request_path: PathBuf,
    pub screenshot_trigger_path: PathBuf,
    pub screenshot_result_path: PathBuf,
    pub screenshot_result_trigger_path: PathBuf,
    pub script_path: PathBuf,
    pub script_trigger_path: PathBuf,
    pub script_result_path: PathBuf,
    pub script_result_trigger_path: PathBuf,
    pub script_auto_dump: bool,
    pub pick_trigger_path: PathBuf,
    pub pick_result_path: PathBuf,
    pub pick_result_trigger_path: PathBuf,
    pub pick_auto_dump: bool,
    pub inspect_path: PathBuf,
    pub inspect_trigger_path: PathBuf,
    pub redact_text: bool,
    pub max_debug_string_bytes: usize,
    pub max_gating_trace_entries: usize,
    pub screenshot_on_dump: bool,

    /// Optional DevTools WebSocket endpoint for diagnostics control (script/pick/dump).
    ///
    /// When set (with `devtools_token`), diagnostics enablement is implied even if `FRET_DIAG` is
    /// not set. This is required for web runners that do not have filesystem access.
    pub devtools_ws_url: Option<String>,
    pub devtools_token: Option<String>,
    /// Whether `bundle.dumped` should embed the full `bundle.json` payload in the WS message.
    ///
    /// Web runners should enable this so tooling can materialize bundles without filesystem
    /// access inside the browser.
    pub devtools_embed_bundle: bool,
}

impl Default for UiDiagnosticsConfig {
    fn default() -> Self {
        let out_dir_env = std::env::var_os("FRET_DIAG_DIR").filter(|v| !v.is_empty());
        let diag_enabled =
            std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()) || out_dir_env.is_some();

        let (devtools_ws_url, devtools_token) = {
            #[cfg(all(feature = "diagnostics-ws", target_arch = "wasm32"))]
            {
                fret_diag_ws::client::devtools_ws_config_from_window_query()
            }
            #[cfg(all(feature = "diagnostics-ws", not(target_arch = "wasm32")))]
            {
                let ws_url = std::env::var("FRET_DEVTOOLS_WS")
                    .ok()
                    .filter(|v| !v.trim().is_empty());
                let token = std::env::var("FRET_DEVTOOLS_TOKEN")
                    .ok()
                    .filter(|v| !v.trim().is_empty());
                (ws_url, token)
            }
            #[cfg(not(feature = "diagnostics-ws"))]
            {
                (None, None)
            }
        };

        let enabled = diag_enabled || (devtools_ws_url.is_some() && devtools_token.is_some());
        let out_dir = out_dir_env
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));
        let trigger_path = std::env::var_os("FRET_DIAG_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("trigger.touch"));
        let ready_path = std::env::var_os("FRET_DIAG_READY_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("ready.touch"));
        let exit_path = std::env::var_os("FRET_DIAG_EXIT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("exit.touch"));

        let max_events = std::env::var("FRET_DIAG_MAX_EVENTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);
        let max_snapshots = std::env::var("FRET_DIAG_MAX_SNAPSHOTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        let capture_semantics = env_flag_default_true("FRET_DIAG_SEMANTICS");
        let screenshots_enabled = env_flag_default_false("FRET_DIAG_SCREENSHOTS");
        let screenshot_request_path = std::env::var_os("FRET_DIAG_SCREENSHOT_REQUEST_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("screenshots.request.json"));
        let screenshot_trigger_path = std::env::var_os("FRET_DIAG_SCREENSHOT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("screenshots.touch"));
        let screenshot_result_path = std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("screenshots.result.json"));
        let screenshot_result_trigger_path =
            std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| out_dir.join("screenshots.result.touch"));
        let script_path = std::env::var_os("FRET_DIAG_SCRIPT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("script.json"));
        let script_trigger_path = std::env::var_os("FRET_DIAG_SCRIPT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("script.touch"));
        let script_result_path = std::env::var_os("FRET_DIAG_SCRIPT_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("script.result.json"));
        let script_result_trigger_path = std::env::var_os("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("script.result.touch"));
        let script_auto_dump = env_flag_default_true("FRET_DIAG_SCRIPT_AUTO_DUMP");
        let pick_trigger_path = std::env::var_os("FRET_DIAG_PICK_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("pick.touch"));
        let pick_result_path = std::env::var_os("FRET_DIAG_PICK_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("pick.result.json"));
        let pick_result_trigger_path = std::env::var_os("FRET_DIAG_PICK_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("pick.result.touch"));
        let pick_auto_dump = env_flag_default_true("FRET_DIAG_PICK_AUTO_DUMP");
        let inspect_path = std::env::var_os("FRET_DIAG_INSPECT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("inspect.json"));
        let inspect_trigger_path = std::env::var_os("FRET_DIAG_INSPECT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| out_dir.join("inspect.touch"));
        let redact_text = env_flag_default_true("FRET_DIAG_REDACT_TEXT");
        let max_debug_string_bytes = std::env::var("FRET_DIAG_MAX_DEBUG_STRING_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4096);
        let max_gating_trace_entries = std::env::var("FRET_DIAG_MAX_GATING_TRACE_ENTRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(200)
            .clamp(0, 2000);
        let screenshot_on_dump = env_flag_default_false("FRET_DIAG_SCREENSHOT");

        Self {
            enabled,
            out_dir,
            trigger_path,
            ready_path,
            exit_path,
            max_events,
            max_snapshots,
            capture_semantics,
            screenshots_enabled,
            screenshot_request_path,
            screenshot_trigger_path,
            screenshot_result_path,
            screenshot_result_trigger_path,
            script_path,
            script_trigger_path,
            script_result_path,
            script_result_trigger_path,
            script_auto_dump,
            pick_trigger_path,
            pick_result_path,
            pick_result_trigger_path,
            pick_auto_dump,
            inspect_path,
            inspect_trigger_path,
            redact_text,
            max_debug_string_bytes,
            max_gating_trace_entries,
            screenshot_on_dump,
            devtools_ws_url,
            devtools_token,
            devtools_embed_bundle: cfg!(target_arch = "wasm32"),
        }
    }
}

#[derive(Default)]
pub struct UiDiagnosticsService {
    cfg: UiDiagnosticsConfig,
    per_window: HashMap<AppWindowId, WindowRing>,
    last_trigger_stamp: Option<u64>,
    last_script_trigger_stamp: Option<u64>,
    last_pick_trigger_mtime: Option<std::time::SystemTime>,
    last_inspect_trigger_mtime: Option<std::time::SystemTime>,
    exit_armed: bool,
    exit_last_mtime: Option<std::time::SystemTime>,
    ready_written: bool,
    capabilities_written: bool,
    inspect_enabled: bool,
    inspect_consume_clicks: bool,
    pending_script: Option<PendingScript>,
    pending_script_run_id: Option<u64>,
    active_scripts: HashMap<AppWindowId, ActiveScript>,
    pending_force_dump_label: Option<String>,
    last_dump_dir: Option<PathBuf>,
    last_script_run_id: u64,
    last_pick_run_id: u64,
    last_picked_node_id: HashMap<AppWindowId, u64>,
    last_picked_selector_json: HashMap<AppWindowId, String>,
    last_hovered_node_id: HashMap<AppWindowId, u64>,
    last_hovered_selector_json: HashMap<AppWindowId, String>,
    inspect_focus_node_id: HashMap<AppWindowId, u64>,
    inspect_focus_selector_json: HashMap<AppWindowId, String>,
    inspect_focus_down_stack: HashMap<AppWindowId, Vec<u64>>,
    inspect_pending_nav: HashMap<AppWindowId, InspectNavCommand>,
    inspect_focus_summary_line: HashMap<AppWindowId, String>,
    inspect_focus_path_line: HashMap<AppWindowId, String>,
    inspect_locked_windows: HashSet<AppWindowId>,
    inspect_toast: HashMap<AppWindowId, InspectToast>,
    pick_overlay_grace_frames: HashMap<AppWindowId, u32>,
    pick_armed_run_id: Option<u64>,
    pending_pick: Option<PendingPick>,
    app_snapshot_provider:
        Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    #[cfg(feature = "diagnostics-ws")]
    ws_bridge: UiDiagnosticsWsBridge,
}

#[derive(Debug, Clone, Copy)]
enum InspectNavCommand {
    Up,
    Down,
    Focus,
}

#[derive(Debug, Clone)]
struct InspectToast {
    message: String,
    remaining_frames: u32,
}

impl UiDiagnosticsService {
    pub fn is_enabled(&self) -> bool {
        self.cfg.enabled
    }

    pub fn set_app_snapshot_provider(
        &mut self,
        provider: Option<Arc<dyn Fn(&App, AppWindowId) -> Option<serde_json::Value> + 'static>>,
    ) {
        self.app_snapshot_provider = provider;
    }

    /// Returns `true` if the current diagnostics state would benefit from (or requires) a fresh
    /// semantics snapshot for `window` on this frame.
    ///
    /// This is a performance knob: semantics snapshots are expensive, and many scripted steps only
    /// need semantics for *initial target resolution*. Once a step has cached its target geometry
    /// (via `v2_step_state`), we can often skip requesting semantics until a selector-based step is
    /// about to run again.
    pub fn wants_semantics_snapshot(&mut self, window: AppWindowId) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();
        self.poll_script_trigger();

        if self.cfg.capture_semantics {
            return true;
        }

        if self.pick_armed_run_id.is_some()
            || self
                .pending_pick
                .as_ref()
                .is_some_and(|p| p.window == window)
            || self.inspect_enabled
            || self.inspect_locked_windows.contains(&window)
            || self.inspect_toast.contains_key(&window)
        {
            return true;
        }

        if self.pending_script.is_some() {
            return true;
        }

        self.active_scripts
            .get(&window)
            .is_some_and(active_script_needs_semantics_snapshot)
    }

    pub fn poll_exit_trigger(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        let current_mtime = std::fs::metadata(&self.cfg.exit_path)
            .and_then(|m| m.modified())
            .ok();

        if !self.exit_armed {
            self.exit_last_mtime = current_mtime;
            self.exit_armed = true;
            return false;
        }

        let Some(current_mtime) = current_mtime else {
            return false;
        };

        let triggered = match self.exit_last_mtime {
            Some(prev) => current_mtime > prev,
            None => true,
        };
        self.exit_last_mtime = Some(current_mtime);
        triggered
    }

    pub fn redact_text(&self) -> bool {
        self.cfg.redact_text
    }

    pub fn last_pointer_position(&self, window: AppWindowId) -> Option<Point> {
        self.per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position)
    }

    pub fn last_picked_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.last_picked_node_id.get(&window).copied()
    }

    pub fn pick_is_armed(&self) -> bool {
        self.pick_armed_run_id.is_some()
    }

    pub fn inspect_is_enabled(&self) -> bool {
        self.inspect_enabled
    }

    pub fn inspect_consume_clicks(&self) -> bool {
        self.inspect_consume_clicks
    }

    pub fn inspect_is_locked(&self, window: AppWindowId) -> bool {
        self.inspect_locked_windows.contains(&window)
    }

    pub fn inspect_focus_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.inspect_focus_node_id.get(&window).copied()
    }

    pub fn inspect_focus_summary_line(&self, window: AppWindowId) -> Option<&str> {
        self.inspect_focus_summary_line
            .get(&window)
            .map(|s| s.as_str())
    }

    pub fn inspect_focus_path_line(&self, window: AppWindowId) -> Option<&str> {
        self.inspect_focus_path_line
            .get(&window)
            .map(|s| s.as_str())
    }

    pub fn inspect_toast_message(&self, window: AppWindowId) -> Option<&str> {
        self.inspect_toast.get(&window).map(|t| t.message.as_str())
    }

    pub fn inspect_best_selector_json(&self, window: AppWindowId) -> Option<&str> {
        self.inspect_focus_selector_json
            .get(&window)
            .map(|s| s.as_str())
            .or_else(|| {
                self.last_picked_selector_json
                    .get(&window)
                    .map(|s| s.as_str())
            })
            .or_else(|| {
                self.last_hovered_selector_json
                    .get(&window)
                    .map(|s| s.as_str())
            })
    }

    pub fn wants_inspection_active(&mut self, window: AppWindowId) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let grace = self
            .pick_overlay_grace_frames
            .get(&window)
            .copied()
            .unwrap_or(0);
        if grace > 0 {
            let next = grace.saturating_sub(1);
            if next == 0 {
                self.pick_overlay_grace_frames.remove(&window);
            } else {
                self.pick_overlay_grace_frames.insert(window, next);
            }
        }

        if let Some(toast) = self.inspect_toast.get_mut(&window) {
            toast.remaining_frames = toast.remaining_frames.saturating_sub(1);
            if toast.remaining_frames == 0 {
                self.inspect_toast.remove(&window);
            }
        }

        self.pick_armed_run_id.is_some()
            || grace > 0
            || self.inspect_enabled
            || self.inspect_toast.contains_key(&window)
            || self
                .pending_pick
                .as_ref()
                .is_some_and(|p| p.window == window)
    }

    /// Returns `true` if the event was consumed by inspect-mode shortcuts.
    pub fn maybe_intercept_event_for_inspect_shortcuts(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
        else {
            return false;
        };
        if *repeat {
            return false;
        }

        let inspection_active = self.pick_armed_run_id.is_some() || self.inspect_enabled;
        if !inspection_active {
            return false;
        }

        match *key {
            KeyCode::Escape => {
                if self.pick_armed_run_id.take().is_some() {
                    self.push_inspect_toast(window, "inspect: pick disarmed".to_string());
                    app.request_redraw(window);
                    return true;
                }

                if self.inspect_enabled {
                    self.inspect_enabled = false;
                    self.inspect_locked_windows.clear();
                    self.last_hovered_selector_json.clear();
                    self.last_picked_selector_json.clear();
                    self.last_hovered_node_id.clear();
                    self.inspect_focus_node_id.clear();
                    self.inspect_focus_selector_json.clear();
                    self.inspect_focus_down_stack.clear();
                    self.inspect_pending_nav.clear();
                    self.inspect_focus_summary_line.clear();
                    self.inspect_focus_path_line.clear();

                    let _ = write_json(
                        self.cfg.inspect_path.clone(),
                        &UiInspectConfigV1 {
                            schema_version: 1,
                            enabled: false,
                            consume_clicks: self.inspect_consume_clicks,
                        },
                    );
                    let _ = touch_file(&self.cfg.inspect_trigger_path);

                    self.push_inspect_toast(window, "inspect: disabled".to_string());
                    app.request_redraw(window);
                    return true;
                }
                false
            }
            KeyCode::KeyL => {
                if self.inspect_locked_windows.remove(&window) {
                    self.inspect_focus_down_stack.remove(&window);
                    self.push_inspect_toast(window, "inspect: unlocked".to_string());
                } else if let Some(hovered) = self.last_hovered_node_id.get(&window).copied() {
                    self.last_picked_node_id.insert(window, hovered);
                    if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                        self.last_picked_selector_json.insert(window, sel);
                    }
                    self.inspect_focus_node_id.insert(window, hovered);
                    if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                        self.inspect_focus_selector_json.insert(window, sel);
                    }
                    self.inspect_focus_down_stack.insert(window, Vec::new());
                    self.inspect_locked_windows.insert(window);
                    self.push_inspect_toast(window, "inspect: locked selection".to_string());
                } else {
                    self.push_inspect_toast(window, "inspect: nothing to lock".to_string());
                }
                app.request_redraw(window);
                true
            }
            KeyCode::KeyC => {
                let wants_copy = modifiers.ctrl || modifiers.meta;
                if !wants_copy {
                    return false;
                }
                if modifiers.shift {
                    let payload = self.inspect_copy_details_payload(window);
                    if payload.is_empty() {
                        self.push_inspect_toast(
                            window,
                            "inspect: no details available to copy".to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                    app.push_effect(Effect::ClipboardSetText { text: payload });
                    self.push_inspect_toast(window, "inspect: copied inspect details".to_string());
                    app.request_redraw(window);
                    return true;
                }

                let Some(payload) = self
                    .inspect_best_selector_json(window)
                    .map(|s| s.to_string())
                else {
                    self.push_inspect_toast(window, "inspect: no selector to copy".to_string());
                    app.request_redraw(window);
                    return true;
                };
                app.push_effect(Effect::ClipboardSetText { text: payload });
                self.push_inspect_toast(window, "inspect: copied selector".to_string());
                app.request_redraw(window);
                true
            }
            KeyCode::KeyF => {
                if !self.inspect_enabled {
                    return false;
                }
                self.inspect_pending_nav
                    .insert(window, InspectNavCommand::Focus);
                self.push_inspect_toast(window, "inspect: select focused node".to_string());
                app.request_redraw(window);
                true
            }
            KeyCode::ArrowUp => {
                if !modifiers.alt {
                    return false;
                }
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
                self.inspect_pending_nav
                    .insert(window, InspectNavCommand::Up);
                app.request_redraw(window);
                true
            }
            KeyCode::ArrowDown => {
                if !modifiers.alt {
                    return false;
                }
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
                self.inspect_pending_nav
                    .insert(window, InspectNavCommand::Down);
                app.request_redraw(window);
                true
            }
            _ => false,
        }
    }

    /// Returns `true` if the event was consumed by diagnostics picking.
    ///
    /// When a pick is armed, the next pointer down is intercepted (not dispatched to the UI tree)
    /// to avoid triggering app behavior while selecting a target (GPUI/Zed inspect style).
    pub fn maybe_intercept_event_for_picking(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let Event::Pointer(PointerEvent::Down { position, .. }) = event else {
            return false;
        };

        if let Some(run_id) = self.pick_armed_run_id.take() {
            self.pending_pick = Some(PendingPick {
                run_id,
                window,
                position: *position,
            });
            app.request_redraw(window);
            return true;
        }

        if !self.inspect_enabled {
            return false;
        }

        let run_id = self.next_pick_run_id();

        self.pending_pick = Some(PendingPick {
            run_id,
            window,
            position: *position,
        });
        app.request_redraw(window);
        self.inspect_consume_clicks
    }

    pub fn drive_script_for_window(
        &mut self,
        app: &App,
        window: AppWindowId,
        window_bounds: Rect,
        scale_factor: f32,
        ui: Option<&UiTree<App>>,
        semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) -> UiScriptFrameOutput {
        if !self.is_enabled() {
            return UiScriptFrameOutput::default();
        }

        self.ensure_ready_file();
        self.poll_script_trigger();

        if !self.active_scripts.contains_key(&window)
            && let Some(script) = self.pending_script.clone()
        {
            let run_id = self.pending_script_run_id.take().unwrap_or(0);
            self.pending_script = None;
            self.active_scripts.insert(
                window,
                ActiveScript {
                    steps: script.steps,
                    run_id,
                    next_step: 0,
                    wait_frames_remaining: 0,
                    wait_until: None,
                    screenshot_wait: None,
                    v2_step_state: None,
                    last_reported_step: Some(0),
                    selector_resolution_trace: Vec::new(),
                    hit_test_trace: Vec::new(),
                },
            );
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Running,
                step_index: Some(0),
                reason_code: None,
                reason: None,
                evidence: None,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
            });
        }

        let Some(mut active) = self.active_scripts.remove(&window) else {
            return UiScriptFrameOutput::default();
        };

        if active.next_step >= active.steps.len() {
            return UiScriptFrameOutput::default();
        }

        if active.last_reported_step != Some(active.next_step) {
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Running,
                step_index: Some(active.next_step.min(u32::MAX as usize) as u32),
                reason_code: None,
                reason: None,
                evidence: None,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
            });
            active.last_reported_step = Some(active.next_step);
        }

        if active.wait_frames_remaining > 0 {
            active.wait_frames_remaining = active.wait_frames_remaining.saturating_sub(1);
            self.active_scripts.insert(window, active);
            return UiScriptFrameOutput {
                request_redraw: true,
                ..UiScriptFrameOutput::default()
            };
        }

        let step_index = active.next_step;
        let step = active.steps.get(step_index).cloned();
        let Some(step) = step else {
            return UiScriptFrameOutput::default();
        };

        // Keep selector resolution evidence scoped to the active step so failures remain focused.
        active
            .selector_resolution_trace
            .retain(|e| e.step_index == step_index.min(u32::MAX as usize) as u32);

        let mut output = UiScriptFrameOutput::default();
        let mut force_dump_label: Option<String> = None;
        let mut stop_script = false;
        let mut failure_reason: Option<String> = None;

        let is_v2_intent_step = matches!(
            &step,
            UiActionStepV2::ClickStable { .. }
                | UiActionStepV2::EnsureVisible { .. }
                | UiActionStepV2::ScrollIntoView { .. }
                | UiActionStepV2::TypeTextInto { .. }
                | UiActionStepV2::MenuSelect { .. }
                | UiActionStepV2::MenuSelectPath { .. }
                | UiActionStepV2::DragPointer { .. }
                | UiActionStepV2::DragTo { .. }
                | UiActionStepV2::SetSliderValue { .. }
                | UiActionStepV2::MovePointerSweep { .. }
        );
        if !is_v2_intent_step {
            active.v2_step_state = None;
        }

        match step {
            UiActionStepV2::SetWindowInnerSize {
                width_px,
                height_px,
            } => {
                let size = fret_core::Size::new(fret_core::Px(width_px), fret_core::Px(height_px));
                output
                    .effects
                    .push(Effect::Window(fret_app::WindowRequest::SetInnerSize {
                        window,
                        size,
                    }));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            }
            UiActionStepV2::WaitFrames { n } => {
                active.wait_frames_remaining = n;
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            }
            UiActionStepV2::ResetDiagnostics => {
                self.reset_diagnostics_ring_for_window(window);
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            }
            UiActionStepV2::CaptureBundle { label } => {
                force_dump_label =
                    Some(label.unwrap_or_else(|| format!("script-step-{step_index:04}-capture")));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            }
            UiActionStepV2::CaptureScreenshot {
                label,
                timeout_frames,
            } => {
                let window_ffi = window.data().as_ffi();
                active.wait_until = None;
                if !self.cfg.screenshots_enabled {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-capture_screenshot-disabled"
                    ));
                    stop_script = true;
                    failure_reason = Some("screenshots_disabled".to_string());
                    active.screenshot_wait = None;
                    output.request_redraw = true;
                } else {
                    let mut state = match active.screenshot_wait.take() {
                        Some(mut state) if state.step_index == step_index => {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            Some(state)
                        }
                        _ => None,
                    };

                    if state.is_none() {
                        if self.last_dump_dir.is_none() {
                            let dump_label =
                                label.as_deref().map(sanitize_label).unwrap_or_else(|| {
                                    format!("script-step-{step_index:04}-capture_screenshot")
                                });
                            self.dump_bundle(Some(&dump_label));
                        }

                        let bundle_dir_name = self
                            .last_dump_dir
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();

                        if bundle_dir_name.is_empty() {
                            force_dump_label = Some(format!(
                                "script-step-{step_index:04}-capture_screenshot-no-last-dump"
                            ));
                            stop_script = true;
                            failure_reason = Some("no_last_dump_dir".to_string());
                            active.screenshot_wait = None;
                            output.request_redraw = true;
                        } else {
                            let request_id = format!(
                                "script-run-{run_id}-window-{window_ffi}-step-{step_index:04}",
                                run_id = active.run_id
                            );

                            let req = serde_json::json!({
                                "schema_version": 1,
                                "out_dir": self.cfg.out_dir.to_string_lossy(),
                                "bundle_dir_name": bundle_dir_name,
                                "request_id": request_id,
                                "windows": [{
                                    "window": window_ffi,
                                    "tick_id": app.tick_id().0,
                                    "frame_id": app.frame_id().0,
                                    "scale_factor": scale_factor as f64,
                                }]
                            });

                            let bytes = serde_json::to_vec_pretty(&req).ok();
                            if let Some(bytes) = bytes {
                                if let Some(parent) = self.cfg.screenshot_request_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }
                                let write_ok =
                                    std::fs::write(&self.cfg.screenshot_request_path, bytes)
                                        .is_ok()
                                        && touch_file(&self.cfg.screenshot_trigger_path).is_ok();
                                if write_ok {
                                    state = Some(ScreenshotWaitState {
                                        step_index,
                                        remaining_frames: timeout_frames,
                                        request_id,
                                        window_ffi,
                                        last_result_trigger_stamp: None,
                                    });
                                } else {
                                    force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-capture_screenshot-write-failed"
                                    ));
                                    stop_script = true;
                                    failure_reason =
                                        Some("screenshot_request_write_failed".to_string());
                                    active.screenshot_wait = None;
                                    output.request_redraw = true;
                                }
                            } else {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-capture_screenshot-serialize-failed"
                                ));
                                stop_script = true;
                                failure_reason =
                                    Some("screenshot_request_serialize_failed".to_string());
                                active.screenshot_wait = None;
                                output.request_redraw = true;
                            }
                        }
                    }

                    if !stop_script {
                        if let Some(state) = state {
                            let trigger_stamp =
                                read_touch_stamp(&self.cfg.screenshot_result_trigger_path);
                            let completed = trigger_stamp.is_some()
                                && trigger_stamp != state.last_result_trigger_stamp
                                && screenshot_request_completed(
                                    &self.cfg.screenshot_result_path,
                                    &state.request_id,
                                    state.window_ffi,
                                );

                            if completed {
                                active.screenshot_wait = None;
                                active.next_step = active.next_step.saturating_add(1);
                                output.request_redraw = true;
                            } else if state.remaining_frames == 0 {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-capture_screenshot-timeout"
                                ));
                                stop_script = true;
                                failure_reason = Some("capture_screenshot_timeout".to_string());
                                active.screenshot_wait = None;
                                output.request_redraw = true;
                            } else {
                                active.screenshot_wait = Some(ScreenshotWaitState {
                                    step_index: state.step_index,
                                    remaining_frames: state.remaining_frames.saturating_sub(1),
                                    request_id: state.request_id,
                                    window_ffi: state.window_ffi,
                                    last_result_trigger_stamp: trigger_stamp,
                                });
                                output.request_redraw = true;
                            }
                        } else {
                            force_dump_label = Some(format!(
                                "script-step-{step_index:04}-capture_screenshot-no-state"
                            ));
                            stop_script = true;
                            failure_reason = Some("capture_screenshot_state_missing".to_string());
                            active.screenshot_wait = None;
                            output.request_redraw = true;
                        }
                    }
                }
            }
            UiActionStepV2::PressKey {
                key,
                modifiers,
                repeat,
            } => {
                if let Some(key) = parse_key_code(&key) {
                    output
                        .events
                        .extend(press_key_events(key, modifiers, repeat));
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if self.cfg.script_auto_dump {
                        force_dump_label = Some(format!("script-step-{step_index:04}-press_key"));
                    }
                } else {
                    force_dump_label =
                        Some(format!("script-step-{step_index:04}-press_key-unknown-key"));
                    stop_script = true;
                    failure_reason = Some(format!("unknown_key: {key}"));
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::PressShortcut { shortcut, repeat } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some((key, modifiers)) = parse_shortcut(&shortcut) {
                    output
                        .events
                        .extend(press_key_events(key, modifiers, repeat));
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if self.cfg.script_auto_dump {
                        force_dump_label =
                            Some(format!("script-step-{step_index:04}-press_shortcut"));
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-press_shortcut-parse-failed"
                    ));
                    stop_script = true;
                    failure_reason = Some(format!("invalid_shortcut: {shortcut}"));
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::TypeText { text } => {
                output.events.push(Event::TextInput(text));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if self.cfg.script_auto_dump {
                    force_dump_label = Some(format!("script-step-{step_index:04}-type_text"));
                }
            }
            UiActionStepV2::WaitUntil {
                predicate,
                timeout_frames,
            } => {
                if let Some(snapshot) = semantics_snapshot {
                    active.screenshot_wait = None;
                    let state = match active.wait_until.take() {
                        Some(mut state) if state.step_index == step_index => {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => WaitUntilState {
                            step_index,
                            remaining_frames: timeout_frames,
                        },
                    };

                    if eval_predicate(snapshot, window_bounds, window, element_runtime, &predicate)
                    {
                        active.wait_until = None;
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                    } else if state.remaining_frames == 0 {
                        force_dump_label =
                            Some(format!("script-step-{step_index:04}-wait_until-timeout"));
                        stop_script = true;
                        failure_reason = Some("wait_until_timeout".to_string());
                        active.wait_until = None;
                        output.request_redraw = true;
                    } else {
                        active.wait_until = Some(WaitUntilState {
                            step_index: state.step_index,
                            remaining_frames: state.remaining_frames.saturating_sub(1),
                        });
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-wait_until-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    output.request_redraw = true;
                    active.wait_until = None;
                    active.screenshot_wait = None;
                }
            }
            UiActionStepV2::Assert { predicate } => {
                active.wait_until = None;
                active.screenshot_wait = None;
                if let Some(snapshot) = semantics_snapshot {
                    if eval_predicate(snapshot, window_bounds, window, element_runtime, &predicate)
                    {
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                    } else {
                        force_dump_label =
                            Some(format!("script-step-{step_index:04}-assert-failed"));
                        stop_script = true;
                        failure_reason = Some("assert_failed".to_string());
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label =
                        Some(format!("script-step-{step_index:04}-assert-no-semantics"));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::Click {
                target,
                button,
                click_count,
                modifiers,
            } => {
                let Some(snapshot) = semantics_snapshot else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-click-no-semantics");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("semantics.missing".to_string()),
                        reason: Some("no_semantics_snapshot".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };
                let Some(node) = select_semantics_node_with_trace(
                    snapshot,
                    window,
                    element_runtime,
                    &target,
                    step_index as u32,
                    self.cfg.redact_text,
                    &mut active.selector_resolution_trace,
                ) else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-click-no-semantics-match");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("selector.not_found".to_string()),
                        reason: Some("click_no_semantics_match".to_string()),
                        evidence: script_evidence_for_active(&active),
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };

                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                if let Some(ui) = ui {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("click"),
                    );
                }
                let modifiers = core_modifiers_from_ui(modifiers);
                output.events.extend(click_events_with_modifiers(
                    pos,
                    button,
                    click_count,
                    modifiers,
                ));

                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if self.cfg.script_auto_dump {
                    force_dump_label = Some(format!("script-step-{step_index:04}-click"));
                }
            }
            UiActionStepV2::ClickStable {
                target,
                button,
                click_count,
                modifiers,
                stable_frames,
                max_move_px,
                timeout_frames,
            } => {
                let click_modifiers = core_modifiers_from_ui(modifiers);
                active.wait_until = None;
                active.screenshot_wait = None;

                // Stable-click: wait for semantics bounds to stop moving before clicking.
                //
                // This is intended for virtualization-heavy views where a target's measured bounds
                // can jump across frames (e.g. estimate -> measured), causing clicks to land at
                // stale coordinates when using a single-frame snapshot.
                if let Some(snapshot) = semantics_snapshot {
                    if let Some(node) = select_semantics_node_with_trace(
                        snapshot,
                        window,
                        element_runtime,
                        &target,
                        step_index as u32,
                        self.cfg.redact_text,
                        &mut active.selector_resolution_trace,
                    ) {
                        let stable_required = stable_frames.max(1);
                        let max_move_px = max_move_px.max(0.0);

                        let mut state = match active.v2_step_state.take() {
                            Some(V2StepState::ClickStable(mut state))
                                if state.step_index == step_index =>
                            {
                                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                                state
                            }
                            _ => V2ClickStableState {
                                step_index,
                                remaining_frames: timeout_frames,
                                stable_count: 0,
                                last_center: None,
                            },
                        };

                        let center = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                        if state.remaining_frames == 0 {
                            if let Some(ui) = ui {
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    Some(snapshot),
                                    &target,
                                    step_index as u32,
                                    center,
                                    Some(node),
                                    Some("click_stable.timeout"),
                                );
                            }
                            force_dump_label =
                                Some(format!("script-step-{step_index:04}-click_stable-timeout"));
                            stop_script = true;
                            failure_reason = Some("click_stable_timeout".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                        } else {
                            let moved = match state.last_center {
                                Some(last) => {
                                    let dx = (center.x.0 - last.x.0).abs();
                                    let dy = (center.y.0 - last.y.0).abs();
                                    dx.max(dy)
                                }
                                None => 0.0,
                            };

                            if moved <= max_move_px {
                                state.stable_count = state.stable_count.saturating_add(1);
                            } else {
                                state.stable_count = 1;
                            }
                            state.last_center = Some(center);

                            if state.stable_count >= stable_required {
                                if let Some(ui) = ui {
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &target,
                                        step_index as u32,
                                        center,
                                        Some(node),
                                        Some("click_stable.click"),
                                    );
                                }
                                output.events.extend(click_events_with_modifiers(
                                    center,
                                    button,
                                    click_count,
                                    click_modifiers,
                                ));
                                active.wait_until = None;
                                active.screenshot_wait = None;
                                active.next_step = active.next_step.saturating_add(1);
                                active.v2_step_state = None;
                                output.request_redraw = true;
                                if self.cfg.script_auto_dump {
                                    force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-click_stable-click"
                                    ));
                                }
                            } else {
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state = Some(V2StepState::ClickStable(state));
                                output.request_redraw = true;
                            }
                        }
                    } else {
                        force_dump_label = Some(format!(
                            "script-step-{step_index:04}-click_stable-no-semantics-match"
                        ));
                        stop_script = true;
                        failure_reason = Some("click_stable_no_semantics_match".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-click_stable-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }

                #[cfg(any())]
                if let Some(snapshot) = semantics_snapshot {
                    if let Some(node) =
                        select_semantics_node(snapshot, window, element_runtime, &target)
                    {
                        let stable_required = stable_frames.max(1);
                        let max_move_px = max_move_px.max(0.0);

                        let mut state = match active.v2_step_state.take() {
                            Some(V2StepState::ClickStable(mut state))
                                if state.step_index == step_index =>
                            {
                                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                                state
                            }
                            _ => V2ClickStableState {
                                step_index,
                                remaining_frames: timeout_frames,
                                stable_count: 0,
                                last_center: None,
                                resolved_center: None,
                                logged_no_hit: false,
                                logged_mismatch: false,
                            },
                        };

                        let bounds = node.bounds;
                        let mut center = state
                            .resolved_center
                            .unwrap_or_else(|| center_of_rect(bounds));

                        let target_test_id = node.test_id.as_deref();
                        let mut hit_snapshot_for_center = UiHitTestSnapshotV1::from_tree(
                            center,
                            ui,
                            element_runtime,
                            window,
                            Some(snapshot),
                        );
                        let mut seed_hittable = hit_snapshot_for_center
                            .hit_semantics_actionable_ancestor
                            .as_ref()
                            .is_some_and(|hit| {
                                hit.id == node.id.data().as_ffi()
                                    || (target_test_id.is_some()
                                        && hit.test_id.as_deref() == target_test_id)
                            });

                        let seed_hit = ui.debug_hit_test(center).hit;
                        let seed_hit_node = seed_hit.map(|n| n.data().as_ffi());
                        let seed_hit_children_transform =
                            seed_hit.and_then(|n| ui.debug_node_children_render_transform(n));
                        let target_debug_bounds = ui.debug_node_bounds(node.id);
                        let target_visual_bounds = ui.debug_node_visual_bounds(node.id);
                        if state.resolved_center.is_none() && !seed_hittable {
                            if let Some(found) = find_hittable_point_for_node(ui, node.id, bounds) {
                                state.resolved_center = Some(found);
                                center = found;
                                hit_snapshot_for_center = UiHitTestSnapshotV1::from_tree(
                                    center,
                                    ui,
                                    element_runtime,
                                    window,
                                    Some(snapshot),
                                );
                                seed_hittable = hit_snapshot_for_center
                                    .hit_semantics_actionable_ancestor
                                    .as_ref()
                                    .is_some_and(|hit| {
                                        hit.id == node.id.data().as_ffi()
                                            || (target_test_id.is_some()
                                                && hit.test_id.as_deref() == target_test_id)
                                    });
                            } else if env_flag_default_false("FRET_DIAG_DEBUG_CLICK_STABLE")
                                && !state.logged_no_hit
                            {
                                state.logged_no_hit = true;
                                let seed_hit_actionable_test_id = hit_snapshot_for_center
                                    .hit_semantics_actionable_ancestor
                                    .as_ref()
                                    .and_then(|n| n.test_id.as_deref())
                                    .unwrap_or("<none>");
                                let mut mapped = center;
                                let mut blocker_node_id: Option<u64> = None;
                                let mut blocker_bounds: Option<Rect> = None;
                                let mut blocker_pos: Option<Point> = None;
                                let mut blocker_clips_hit_test: Option<bool> = None;
                                let mut blocker_element: Option<u64> = None;
                                let mut blocker_element_path: Option<String> = None;
                                for id in ui.debug_node_path(node.id) {
                                    if let Some(t) = ui.debug_node_render_transform(id)
                                        && let Some(inv) = t.inverse()
                                    {
                                        mapped = inv.apply_point(mapped);
                                    }

                                    let bounds_here = ui.debug_node_bounds(id).unwrap_or_default();
                                    let clips_here =
                                        ui.debug_node_clips_hit_test(id).unwrap_or(true);
                                    if clips_here && !bounds_here.contains(mapped) {
                                        blocker_node_id = Some(id.data().as_ffi());
                                        blocker_bounds = Some(bounds_here);
                                        blocker_pos = Some(mapped);
                                        blocker_clips_hit_test = Some(clips_here);
                                        if let Some(el) = ui.debug_node_element(id) {
                                            blocker_element = Some(el.0);
                                            blocker_element_path = element_runtime.and_then(|rt| {
                                                rt.debug_path_for_element(window, el)
                                            });
                                        }
                                        break;
                                    }

                                    if let Some(t) = ui.debug_node_children_render_transform(id)
                                        && let Some(inv) = t.inverse()
                                    {
                                        mapped = inv.apply_point(mapped);
                                    }
                                }
                                tracing::info!(
                                    step_index,
                                    target = ?target,
                                    target_node_id = node.id.data().as_ffi(),
                                    bounds = ?bounds,
                                    seed_hit = ?hit_snapshot_for_center.hit,
                                    seed_hit_element = ?hit_snapshot_for_center.hit_element,
                                    seed_hit_element_path = ?hit_snapshot_for_center.hit_element_path,
                                    seed_hit_actionable_test_id,
                                    target_debug_bounds = ?target_debug_bounds,
                                    target_visual_bounds = ?target_visual_bounds,
                                    center_x = center.x.0,
                                    center_y = center.y.0,
                                    seed_hit_node,
                                    seed_hit_children_transform = ?seed_hit_children_transform,
                                    blocker_node_id,
                                    blocker_bounds = ?blocker_bounds,
                                    blocker_pos = ?blocker_pos,
                                    blocker_clips_hit_test,
                                    blocker_element,
                                    blocker_element_path = blocker_element_path.as_deref().unwrap_or("<none>"),
                                    "diag click_stable could not find hittable point"
                                );
                            }
                        }

                        if state.remaining_frames == 0 {
                            force_dump_label =
                                Some(format!("script-step-{step_index:04}-click_stable-timeout"));
                            stop_script = true;
                            failure_reason = Some("click_stable_timeout".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                        } else {
                            // While waiting for stability, continuously move the pointer to the
                            // computed center so scroll/hit-test-only transforms settle before we
                            // press.
                            let moved = match state.last_center {
                                Some(prev) => {
                                    let dx = center.x.0 - prev.x.0;
                                    let dy = center.y.0 - prev.y.0;
                                    (dx * dx + dy * dy).sqrt()
                                }
                                None => 0.0,
                            };

                            if state.last_center.is_none() || moved <= max_move_px {
                                state.stable_count = state.stable_count.saturating_add(1);
                            } else {
                                state.stable_count = 1;
                            }
                            state.last_center = Some(center);

                            output.events.push(move_pointer_event(center));
                            let hit_includes_target = hit_snapshot_for_center
                                .hit_semantics_actionable_ancestor
                                .as_ref()
                                .is_some_and(|hit| {
                                    hit.id == node.id.data().as_ffi()
                                        || (target_test_id.is_some()
                                            && hit.test_id.as_deref() == target_test_id)
                                });

                            if state.stable_count >= stable_required && hit_includes_target {
                                if env_flag_default_false("FRET_DIAG_DEBUG_CLICK_STABLE") {
                                    let hit = UiHitTestSnapshotV1::from_tree(
                                        center,
                                        ui,
                                        element_runtime,
                                        window,
                                        Some(snapshot),
                                    );
                                    let semantics_at_center =
                                        pick_semantics_node_at(snapshot, ui, center)
                                            .and_then(|n| n.test_id.as_deref())
                                            .unwrap_or("<none>");
                                    let target_node = select_semantics_node(
                                        snapshot,
                                        window,
                                        element_runtime,
                                        &target,
                                    );
                                    let target_node_id = target_node
                                        .as_ref()
                                        .map(|n| n.id.data().as_ffi())
                                        .unwrap_or(0);
                                    let target_bounds =
                                        target_node.map(|n| n.bounds).unwrap_or(Rect::default());
                                    let target_visual_bounds = target_node
                                        .and_then(|n| ui.debug_node_visual_bounds(n.id))
                                        .unwrap_or(Rect::default());
                                    let target_debug_bounds = target_node
                                        .and_then(|n| ui.debug_node_bounds(n.id))
                                        .unwrap_or(Rect::default());
                                    let viewport_node = snapshot.nodes.iter().find(|n| {
                                        n.test_id.as_deref() == Some("ui-gallery-content-viewport")
                                    });
                                    let viewport_node_id =
                                        viewport_node.map(|n| n.id.data().as_ffi()).unwrap_or(0);
                                    let viewport_children_transform = viewport_node.and_then(|n| {
                                        ui.debug_node_children_render_transform(n.id)
                                    });
                                    let target_path_contains_viewport = target_node
                                        .as_ref()
                                        .map(|n| {
                                            ui.debug_node_path(n.id)
                                                .iter()
                                                .any(|id| id.data().as_ffi() == viewport_node_id)
                                        })
                                        .unwrap_or(false);
                                    let hit_actionable_test_id = hit
                                        .hit_semantics_actionable_ancestor
                                        .as_ref()
                                        .and_then(|n| n.test_id.as_deref())
                                        .unwrap_or("<none>");
                                    let hit_path_contains_target_node =
                                        hit.hit_node_path.contains(&target_node_id);
                                    let hit_path_contains_sources =
                                        hit.hit_node_path_nodes.iter().any(|entry| {
                                            entry.element_path.as_ref().is_some_and(|p| {
                                                p.contains("sources_block.rs")
                                                    || p.contains("fret-ui-ai")
                                                    || p.contains("sources")
                                            })
                                        });

                                    // Probe a small grid inside the chosen click bounds. If none of these points
                                    // ever includes the target node in the hit-test path, the target bounds are
                                    // likely out-of-sync with the hit-test transform chain (e.g. scroll offsets
                                    // applied differently between semantics vs hit testing).
                                    let mut probe_hits_target = 0u8;
                                    let mut probe_logged_sample = false;
                                    let min_x = bounds.origin.x.0;
                                    let min_y = bounds.origin.y.0;
                                    let max_x = bounds.origin.x.0 + bounds.size.width.0;
                                    let max_y = bounds.origin.y.0 + bounds.size.height.0;
                                    let xs = [min_x + 0.5, center.x.0, max_x - 0.5];
                                    let ys = [min_y + 0.5, center.y.0, max_y - 0.5];
                                    for (xi, x) in xs.into_iter().enumerate() {
                                        for (yi, y) in ys.into_iter().enumerate() {
                                            let p = Point::new(fret_core::Px(x), fret_core::Px(y));
                                            let Some(hit_node) = ui.debug_hit_test(p).hit else {
                                                continue;
                                            };
                                            let path = ui.debug_node_path(hit_node);
                                            let includes = path
                                                .iter()
                                                .any(|id| id.data().as_ffi() == target_node_id);
                                            if includes {
                                                probe_hits_target =
                                                    probe_hits_target.saturating_add(1);
                                            }
                                            if !probe_logged_sample {
                                                probe_logged_sample = true;
                                                tracing::info!(
                                                    step_index,
                                                    target = ?target,
                                                    probe_x_index = xi,
                                                    probe_y_index = yi,
                                                    probe_x = x,
                                                    probe_y = y,
                                                    probe_hit = hit_node.data().as_ffi(),
                                                    probe_hit_path_contains_target = includes,
                                                    "diag click_stable probe"
                                                );
                                            }
                                        }
                                    }
                                    tracing::info!(
                                        step_index,
                                        target = ?target,
                                        target_node_id,
                                        target_bounds = ?target_bounds,
                                        target_visual_bounds = ?target_visual_bounds,
                                        target_debug_bounds = ?target_debug_bounds,
                                        viewport_node_id,
                                        viewport_children_transform = ?viewport_children_transform,
                                        target_path_contains_viewport,
                                        inspection_active = ui.inspection_active(),
                                        x = center.x.0,
                                        y = center.y.0,
                                        semantics_at_center,
                                        hit_actionable_test_id,
                                        hit_node = ?hit.hit,
                                        hit_element = ?hit.hit_element,
                                        hit_element_path = ?hit.hit_element_path,
                                        hit_ancestor_element = ?hit.hit_ancestor_element,
                                        hit_ancestor_element_path = ?hit.hit_ancestor_element_path,
                                        hit_path_contains_target_node,
                                        hit_path_contains_sources,
                                        probe_hits_target,
                                        "diag click_stable resolved hit-test"
                                    );
                                }
                                output.events.extend(click_events_with_modifiers(
                                    center,
                                    button,
                                    click_count,
                                    click_modifiers,
                                ));
                                active.v2_step_state = None;
                                active.next_step = active.next_step.saturating_add(1);
                                output.request_redraw = true;
                                if self.cfg.script_auto_dump {
                                    force_dump_label =
                                        Some(format!("script-step-{step_index:04}-click_stable"));
                                }
                            } else {
                                if state.stable_count >= stable_required && !hit_includes_target {
                                    if env_flag_default_false("FRET_DIAG_DEBUG_CLICK_STABLE")
                                        && !state.logged_mismatch
                                    {
                                        state.logged_mismatch = true;
                                        let hit_actionable = hit_snapshot_for_center
                                            .hit_semantics_actionable_ancestor
                                            .as_ref();
                                        tracing::info!(
                                            step_index,
                                            target = ?target,
                                            target_node_id = node.id.data().as_ffi(),
                                            target_test_id = target_test_id.unwrap_or("<none>"),
                                            x = center.x.0,
                                            y = center.y.0,
                                            hit_actionable_id = hit_actionable.map(|n| n.id).unwrap_or(0),
                                            hit_actionable_test_id = hit_actionable
                                                .and_then(|n| n.test_id.as_deref())
                                                .unwrap_or("<none>"),
                                            hit_node = ?ui.debug_hit_test(center).hit.map(|n| n.data().as_ffi()),
                                            "diag click_stable stable-but-mismatch"
                                        );
                                    }

                                    // Scroll and other transform-only updates can land after the
                                    // semantics snapshot used to choose `center`. If the target
                                    // is not actually hit-testable at this point, keep waiting
                                    // instead of clicking a stale coordinate.
                                    state.stable_count = 0;
                                    state.resolved_center = None;
                                }
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state = Some(V2StepState::ClickStable(state));
                                output.request_redraw = true;
                            }
                        }
                    } else {
                        force_dump_label = Some(format!(
                            "script-step-{step_index:04}-click_stable-no-semantics-match"
                        ));
                        stop_script = true;
                        failure_reason = Some("click_stable_no_semantics_match".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-click_stable-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::MovePointer { target } => {
                let Some(snapshot) = semantics_snapshot else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-move_pointer-no-semantics");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("semantics.missing".to_string()),
                        reason: Some("no_semantics_snapshot".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };
                let Some(node) = select_semantics_node_with_trace(
                    snapshot,
                    window,
                    element_runtime,
                    &target,
                    step_index as u32,
                    self.cfg.redact_text,
                    &mut active.selector_resolution_trace,
                ) else {
                    output.request_redraw = true;
                    let label =
                        format!("script-step-{step_index:04}-move_pointer-no-semantics-match");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("selector.not_found".to_string()),
                        reason: Some("move_pointer_no_semantics_match".to_string()),
                        evidence: script_evidence_for_active(&active),
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };

                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                if let Some(ui) = ui {
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some("move_pointer"),
                    );
                }
                output.events.push(move_pointer_event(pos));

                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if self.cfg.script_auto_dump {
                    force_dump_label = Some(format!("script-step-{step_index:04}-move_pointer"));
                }
            }
            UiActionStepV2::DragPointer {
                target,
                button,
                delta_x,
                delta_y,
                steps,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;
                output.request_redraw = true;

                let mut state = match active.v2_step_state.take() {
                    Some(V2StepState::DragPointer(state)) if state.step_index == step_index => {
                        state
                    }
                    _ => {
                        let Some(snapshot) = semantics_snapshot else {
                            output.request_redraw = true;
                            let label =
                                format!("script-step-{step_index:04}-drag_pointer-no-semantics");
                            if self.cfg.script_auto_dump {
                                self.dump_bundle(Some(&label));
                            }
                            self.write_script_result(UiScriptResultV1 {
                                schema_version: 1,
                                run_id: active.run_id,
                                updated_unix_ms: unix_ms_now(),
                                window: Some(window.data().as_ffi()),
                                stage: UiScriptStageV1::Failed,
                                step_index: Some(step_index as u32),
                                reason_code: Some("semantics.missing".to_string()),
                                reason: Some("no_semantics_snapshot".to_string()),
                                evidence: None,
                                last_bundle_dir: self
                                    .last_dump_dir
                                    .as_ref()
                                    .map(|p| display_path(&self.cfg.out_dir, p)),
                            });
                            return output;
                        };
                        let Some(node) = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &target,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        ) else {
                            output.request_redraw = true;
                            let label = format!(
                                "script-step-{step_index:04}-drag_pointer-no-semantics-match"
                            );
                            if self.cfg.script_auto_dump {
                                self.dump_bundle(Some(&label));
                            }
                            self.write_script_result(UiScriptResultV1 {
                                schema_version: 1,
                                run_id: active.run_id,
                                updated_unix_ms: unix_ms_now(),
                                window: Some(window.data().as_ffi()),
                                stage: UiScriptStageV1::Failed,
                                step_index: Some(step_index as u32),
                                reason_code: Some("selector.not_found".to_string()),
                                reason: Some("drag_pointer_no_semantics_match".to_string()),
                                evidence: script_evidence_for_active(&active),
                                last_bundle_dir: self
                                    .last_dump_dir
                                    .as_ref()
                                    .map(|p| display_path(&self.cfg.out_dir, p)),
                            });
                            return output;
                        };

                        let start = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                        if let Some(ui) = ui {
                            record_hit_test_trace_for_selector(
                                &mut active.hit_test_trace,
                                ui,
                                Some(snapshot),
                                &target,
                                step_index as u32,
                                start,
                                Some(node),
                                Some("drag_pointer.start"),
                            );
                        }
                        let end = Point::new(
                            fret_core::Px(start.x.0 + delta_x),
                            fret_core::Px(start.y.0 + delta_y),
                        );
                        V2DragPointerState {
                            step_index,
                            steps: steps.max(1),
                            button,
                            start,
                            end,
                            frame: 0,
                        }
                    }
                };

                let done = push_drag_playback_frame(&mut state, &mut output.events);
                if done {
                    if let Some(ui) = ui {
                        record_hit_test_trace_for_selector(
                            &mut active.hit_test_trace,
                            ui,
                            semantics_snapshot,
                            &target,
                            step_index as u32,
                            state.end,
                            None,
                            Some("drag_pointer.end"),
                        );
                    }
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    if self.cfg.script_auto_dump {
                        force_dump_label =
                            Some(format!("script-step-{step_index:04}-drag_pointer"));
                    }
                } else {
                    active.v2_step_state = Some(V2StepState::DragPointer(state));
                }
            }
            UiActionStepV2::MovePointerSweep {
                target,
                delta_x,
                delta_y,
                steps,
                frames_per_step,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                let Some(snapshot) = semantics_snapshot else {
                    output.request_redraw = true;
                    let label =
                        format!("script-step-{step_index:04}-move_pointer_sweep-no-semantics");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("semantics.missing".to_string()),
                        reason: Some("no_semantics_snapshot".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };

                let mut state = match active.v2_step_state.take() {
                    Some(V2StepState::MovePointerSweep(state))
                        if state.step_index == step_index =>
                    {
                        state
                    }
                    _ => {
                        let Some(node) = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &target,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        ) else {
                            output.request_redraw = true;
                            let label = format!(
                                "script-step-{step_index:04}-move_pointer_sweep-no-semantics-match"
                            );
                            if self.cfg.script_auto_dump {
                                self.dump_bundle(Some(&label));
                            }
                            self.write_script_result(UiScriptResultV1 {
                                schema_version: 1,
                                run_id: active.run_id,
                                updated_unix_ms: unix_ms_now(),
                                window: Some(window.data().as_ffi()),
                                stage: UiScriptStageV1::Failed,
                                step_index: Some(step_index as u32),
                                reason_code: Some("selector.not_found".to_string()),
                                reason: Some("move_pointer_sweep_no_semantics_match".to_string()),
                                evidence: script_evidence_for_active(&active),
                                last_bundle_dir: self
                                    .last_dump_dir
                                    .as_ref()
                                    .map(|p| display_path(&self.cfg.out_dir, p)),
                            });
                            return output;
                        };

                        let start = center_of_rect(node.bounds);
                        if let Some(ui) = ui {
                            record_hit_test_trace_for_selector(
                                &mut active.hit_test_trace,
                                ui,
                                Some(snapshot),
                                &target,
                                step_index as u32,
                                start,
                                Some(node),
                                Some("move_pointer_sweep.start"),
                            );
                        }
                        let end = Point::new(
                            fret_core::Px(start.x.0 + delta_x),
                            fret_core::Px(start.y.0 + delta_y),
                        );
                        V2MovePointerSweepState {
                            step_index,
                            start,
                            end,
                            steps: steps.max(1),
                            next_step: 0,
                            frames_per_step: frames_per_step.max(1),
                            wait_frames_remaining: 0,
                        }
                    }
                };

                if state.wait_frames_remaining > 0 {
                    state.wait_frames_remaining = state.wait_frames_remaining.saturating_sub(1);
                    active.v2_step_state = Some(V2StepState::MovePointerSweep(state));
                    output.request_redraw = true;
                } else if state.next_step > state.steps {
                    if let Some(ui) = ui {
                        record_hit_test_trace_for_selector(
                            &mut active.hit_test_trace,
                            ui,
                            semantics_snapshot,
                            &target,
                            step_index as u32,
                            state.end,
                            None,
                            Some("move_pointer_sweep.end"),
                        );
                    }
                    active.v2_step_state = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                    if self.cfg.script_auto_dump {
                        force_dump_label =
                            Some(format!("script-step-{step_index:04}-move_pointer_sweep"));
                    }
                } else {
                    let t = state.next_step as f32 / state.steps as f32;
                    let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
                    let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
                    let position = Point::new(fret_core::Px(x), fret_core::Px(y));
                    output.events.push(move_pointer_event(position));

                    state.next_step = state.next_step.saturating_add(1);
                    state.wait_frames_remaining = state.frames_per_step.saturating_sub(1);
                    active.v2_step_state = Some(V2StepState::MovePointerSweep(state));
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::Wheel {
                target,
                delta_x,
                delta_y,
            } => {
                let Some(snapshot) = semantics_snapshot else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-wheel-no-semantics");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("semantics.missing".to_string()),
                        reason: Some("no_semantics_snapshot".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };
                let Some(node) = select_semantics_node_with_trace(
                    snapshot,
                    window,
                    element_runtime,
                    &target,
                    step_index as u32,
                    self.cfg.redact_text,
                    &mut active.selector_resolution_trace,
                ) else {
                    output.request_redraw = true;
                    let label = format!("script-step-{step_index:04}-wheel-no-semantics-match");
                    if self.cfg.script_auto_dump {
                        self.dump_bundle(Some(&label));
                    }
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id: active.run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: Some(window.data().as_ffi()),
                        stage: UiScriptStageV1::Failed,
                        step_index: Some(step_index as u32),
                        reason_code: Some("selector.not_found".to_string()),
                        reason: Some("wheel_no_semantics_match".to_string()),
                        evidence: script_evidence_for_active(&active),
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                    });
                    return output;
                };

                let pos = center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                if let Some(ui) = ui {
                    let note = format!("wheel dx={delta_x} dy={delta_y}");
                    record_hit_test_trace_for_selector(
                        &mut active.hit_test_trace,
                        ui,
                        Some(snapshot),
                        &target,
                        step_index as u32,
                        pos,
                        Some(node),
                        Some(note.as_str()),
                    );
                }
                output.events.push(wheel_event(pos, delta_x, delta_y));

                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if self.cfg.script_auto_dump {
                    force_dump_label = Some(format!("script-step-{step_index:04}-wheel"));
                }
            }
            UiActionStepV2::EnsureVisible {
                target,
                within_window,
                padding_px,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::EnsureVisible(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2EnsureVisibleState {
                            step_index,
                            remaining_frames: timeout_frames,
                        },
                    };

                    let predicate = if within_window {
                        UiPredicateV1::BoundsWithinWindow {
                            target,
                            padding_px,
                            eps_px: 0.0,
                        }
                    } else {
                        UiPredicateV1::VisibleInWindow { target }
                    };

                    if eval_predicate(snapshot, window_bounds, window, element_runtime, &predicate)
                    {
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                        if self.cfg.script_auto_dump {
                            force_dump_label =
                                Some(format!("script-step-{step_index:04}-ensure_visible"));
                        }
                    } else if state.remaining_frames == 0 {
                        force_dump_label = Some(format!(
                            "script-step-{step_index:04}-ensure_visible-timeout"
                        ));
                        stop_script = true;
                        failure_reason = Some("ensure_visible_timeout".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    } else {
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state = Some(V2StepState::EnsureVisible(state));
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-ensure_visible-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::ScrollIntoView {
                container,
                target,
                delta_x,
                delta_y,
                require_fully_within_container,
                require_fully_within_window,
                padding_px,
                padding_insets_px,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::ScrollIntoView(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2ScrollIntoViewState {
                            step_index,
                            remaining_frames: timeout_frames,
                        },
                    };

                    let target_predicate = if require_fully_within_window {
                        UiPredicateV1::BoundsWithinWindow {
                            target: target.clone(),
                            padding_px,
                            eps_px: 0.0,
                        }
                    } else {
                        UiPredicateV1::VisibleInWindow {
                            target: target.clone(),
                        }
                    };
                    let visible_ok = eval_predicate(
                        snapshot,
                        window_bounds,
                        window,
                        element_runtime,
                        &target_predicate,
                    );
                    let container_ok = if require_fully_within_container {
                        let container_node = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &container,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        );
                        let target_node = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &target,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        );
                        if let (Some(container_node), Some(target_node)) =
                            (container_node, target_node)
                        {
                            let insets = padding_insets_px
                                .unwrap_or_else(|| UiPaddingInsetsV1::uniform(padding_px));
                            let inner = rect_inset(container_node.bounds, insets);
                            rect_fully_contains(inner, target_node.bounds)
                        } else {
                            false
                        }
                    } else {
                        true
                    };

                    if visible_ok && container_ok {
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                        if self.cfg.script_auto_dump {
                            force_dump_label =
                                Some(format!("script-step-{step_index:04}-scroll_into_view"));
                        }
                    } else if state.remaining_frames == 0 {
                        force_dump_label = Some(format!(
                            "script-step-{step_index:04}-scroll_into_view-timeout"
                        ));
                        stop_script = true;
                        failure_reason = Some("scroll_into_view_timeout".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    } else {
                        let container_node = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &container,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        );
                        if let Some(container_node) = container_node {
                            let pos = center_of_rect_clamped_to_rect(
                                container_node.bounds,
                                window_bounds,
                            );
                            if let Some(ui) = ui {
                                let note =
                                    format!("scroll_into_view.wheel dx={delta_x} dy={delta_y}");
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    semantics_snapshot,
                                    &container,
                                    step_index as u32,
                                    pos,
                                    Some(container_node),
                                    Some(note.as_str()),
                                );
                            }
                            output.events.push(wheel_event(pos, delta_x, delta_y));
                        }

                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state = Some(V2StepState::ScrollIntoView(state));
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-scroll_into_view-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::TypeTextInto {
                target,
                text,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::TypeTextInto(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2TypeTextIntoState {
                            step_index,
                            remaining_frames: timeout_frames,
                            phase: 0,
                        },
                    };

                    match state.phase {
                        0 => {
                            if select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &target,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            )
                            .is_some()
                            {
                                state.phase = 1;
                                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                                output.request_redraw = true;
                            } else if state.remaining_frames == 0 {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-type_text_into-timeout"
                                ));
                                stop_script = true;
                                failure_reason = Some("type_text_into_timeout".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            } else {
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                                output.request_redraw = true;
                            }
                        }
                        1 => {
                            if let Some(node) = select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &target,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            ) {
                                let pos =
                                    center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                                if let Some(ui) = ui {
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &target,
                                        step_index as u32,
                                        pos,
                                        Some(node),
                                        Some("type_text_into.click"),
                                    );
                                }
                                output
                                    .events
                                    .extend(click_events(pos, UiMouseButtonV1::Left, 1));
                                state.phase = 2;
                                active.v2_step_state = Some(V2StepState::TypeTextInto(state));
                                output.request_redraw = true;
                            } else {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-type_text_into-no-semantics-match"
                                ));
                                stop_script = true;
                                failure_reason =
                                    Some("type_text_into_no_semantics_match".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            }
                        }
                        _ => {
                            output.events.push(Event::TextInput(text));
                            active.v2_step_state = None;
                            active.next_step = active.next_step.saturating_add(1);
                            output.request_redraw = true;
                            if self.cfg.script_auto_dump {
                                force_dump_label =
                                    Some(format!("script-step-{step_index:04}-type_text_into"));
                            }
                        }
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-type_text_into-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::MenuSelect {
                menu,
                item,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;
                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::MenuSelect(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2MenuSelectState {
                            step_index,
                            remaining_frames: timeout_frames,
                            phase: 0,
                        },
                    };

                    match state.phase {
                        0 => {
                            if select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &menu,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            )
                            .is_some()
                            {
                                state.phase = 1;
                                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                                output.request_redraw = true;
                            } else if state.remaining_frames == 0 {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-menu_select-timeout"
                                ));
                                stop_script = true;
                                failure_reason = Some("menu_select_timeout".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            } else {
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                                output.request_redraw = true;
                            }
                        }
                        1 => {
                            if let Some(node) = select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &menu,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            ) {
                                let pos =
                                    center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                                if let Some(ui) = ui {
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &menu,
                                        step_index as u32,
                                        pos,
                                        Some(node),
                                        Some("menu_select.menu_click"),
                                    );
                                }
                                output
                                    .events
                                    .extend(click_events(pos, UiMouseButtonV1::Left, 1));
                                state.phase = 2;
                                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                                output.request_redraw = true;
                            } else {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-menu_select-menu-no-match"
                                ));
                                stop_script = true;
                                failure_reason = Some("menu_select_menu_no_match".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            }
                        }
                        2 => {
                            if select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &item,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            )
                            .is_some()
                            {
                                state.phase = 3;
                                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                                output.request_redraw = true;
                            } else if state.remaining_frames == 0 {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-menu_select-timeout"
                                ));
                                stop_script = true;
                                failure_reason = Some("menu_select_timeout".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            } else {
                                state.remaining_frames = state.remaining_frames.saturating_sub(1);
                                active.v2_step_state = Some(V2StepState::MenuSelect(state));
                                output.request_redraw = true;
                            }
                        }
                        _ => {
                            if let Some(node) = select_semantics_node_with_trace(
                                snapshot,
                                window,
                                element_runtime,
                                &item,
                                step_index as u32,
                                self.cfg.redact_text,
                                &mut active.selector_resolution_trace,
                            ) {
                                let pos = center_of_rect(node.bounds);
                                if let Some(ui) = ui {
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &item,
                                        step_index as u32,
                                        pos,
                                        Some(node),
                                        Some("menu_select.item_click"),
                                    );
                                }
                                output
                                    .events
                                    .extend(click_events(pos, UiMouseButtonV1::Left, 1));
                                active.v2_step_state = None;
                                active.next_step = active.next_step.saturating_add(1);
                                output.request_redraw = true;
                                if self.cfg.script_auto_dump {
                                    force_dump_label =
                                        Some(format!("script-step-{step_index:04}-menu_select"));
                                }
                            } else {
                                force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-menu_select-item-no-match"
                                ));
                                stop_script = true;
                                failure_reason = Some("menu_select_item_no_match".to_string());
                                active.v2_step_state = None;
                                output.request_redraw = true;
                            }
                        }
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-menu_select-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::MenuSelectPath {
                path,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if path.len() < 2 {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-menu_select_path-invalid"
                    ));
                    stop_script = true;
                    failure_reason = Some("menu_select_path_invalid".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                    // Let the common failure handler persist the result.
                } else if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::MenuSelectPath(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2MenuSelectPathState {
                            step_index,
                            remaining_frames: timeout_frames,
                            phase: 0,
                            next_index: 0,
                        },
                    };

                    if state.next_index >= path.len() {
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        output.request_redraw = true;
                        if self.cfg.script_auto_dump {
                            force_dump_label =
                                Some(format!("script-step-{step_index:04}-menu_select_path"));
                        }
                    } else {
                        match state.phase {
                            0 => {
                                if select_semantics_node_with_trace(
                                    snapshot,
                                    window,
                                    element_runtime,
                                    &path[state.next_index],
                                    step_index as u32,
                                    self.cfg.redact_text,
                                    &mut active.selector_resolution_trace,
                                )
                                .is_some()
                                {
                                    state.phase = 1;
                                    active.v2_step_state = Some(V2StepState::MenuSelectPath(state));
                                    output.request_redraw = true;
                                } else if state.remaining_frames == 0 {
                                    force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-menu_select_path-timeout"
                                    ));
                                    stop_script = true;
                                    failure_reason = Some("menu_select_path_timeout".to_string());
                                    active.v2_step_state = None;
                                    output.request_redraw = true;
                                } else {
                                    state.remaining_frames =
                                        state.remaining_frames.saturating_sub(1);
                                    active.v2_step_state = Some(V2StepState::MenuSelectPath(state));
                                    output.request_redraw = true;
                                }
                            }
                            _ => {
                                let selector = &path[state.next_index];
                                if let Some(node) = select_semantics_node_with_trace(
                                    snapshot,
                                    window,
                                    element_runtime,
                                    selector,
                                    step_index as u32,
                                    self.cfg.redact_text,
                                    &mut active.selector_resolution_trace,
                                ) {
                                    let pos =
                                        center_of_rect_clamped_to_rect(node.bounds, window_bounds);
                                    if let Some(ui) = ui {
                                        let note = format!(
                                            "menu_select_path.click index={}",
                                            state.next_index
                                        );
                                        record_hit_test_trace_for_selector(
                                            &mut active.hit_test_trace,
                                            ui,
                                            Some(snapshot),
                                            selector,
                                            step_index as u32,
                                            pos,
                                            Some(node),
                                            Some(note.as_str()),
                                        );
                                    }
                                    output.events.extend(click_events(
                                        pos,
                                        UiMouseButtonV1::Left,
                                        1,
                                    ));
                                    state.next_index = state.next_index.saturating_add(1);
                                    state.phase = 0;
                                    active.v2_step_state = Some(V2StepState::MenuSelectPath(state));
                                    output.request_redraw = true;
                                } else {
                                    force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-menu_select_path-no-match"
                                    ));
                                    stop_script = true;
                                    failure_reason =
                                        Some("menu_select_path_no_semantics_match".to_string());
                                    active.v2_step_state = None;
                                    output.request_redraw = true;
                                }
                            }
                        }
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-menu_select_path-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::DragTo {
                from,
                to,
                button,
                steps,
                timeout_frames,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::DragTo(mut state)) if state.step_index == step_index => {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2DragToState {
                            step_index,
                            remaining_frames: timeout_frames,
                            playback: None,
                        },
                    };

                    let mut playback = if let Some(playback) = state.playback.take() {
                        playback
                    } else {
                        let from_node = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &from,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        );
                        let to_node = select_semantics_node_with_trace(
                            snapshot,
                            window,
                            element_runtime,
                            &to,
                            step_index as u32,
                            self.cfg.redact_text,
                            &mut active.selector_resolution_trace,
                        );
                        if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
                            let start =
                                center_of_rect_clamped_to_rect(from_node.bounds, window_bounds);
                            let end = center_of_rect_clamped_to_rect(to_node.bounds, window_bounds);
                            if let Some(ui) = ui {
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    Some(snapshot),
                                    &from,
                                    step_index as u32,
                                    start,
                                    Some(from_node),
                                    Some("drag_to.start"),
                                );
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    Some(snapshot),
                                    &to,
                                    step_index as u32,
                                    end,
                                    Some(to_node),
                                    Some("drag_to.end"),
                                );
                            }
                            V2DragPointerState {
                                step_index,
                                steps: steps.max(1),
                                button,
                                start,
                                end,
                                frame: 0,
                            }
                        } else if state.remaining_frames == 0 {
                            output.request_redraw = true;
                            let label = format!("script-step-{step_index:04}-drag_to-timeout");
                            if self.cfg.script_auto_dump {
                                self.dump_bundle(Some(&label));
                            }
                            self.write_script_result(UiScriptResultV1 {
                                schema_version: 1,
                                run_id: active.run_id,
                                updated_unix_ms: unix_ms_now(),
                                window: Some(window.data().as_ffi()),
                                stage: UiScriptStageV1::Failed,
                                step_index: Some(step_index as u32),
                                reason_code: reason_code_for_script_failure("drag_to_timeout")
                                    .map(|s| s.to_string()),
                                reason: Some("drag_to_timeout".to_string()),
                                evidence: script_evidence_for_active(&active),
                                last_bundle_dir: self
                                    .last_dump_dir
                                    .as_ref()
                                    .map(|p| display_path(&self.cfg.out_dir, p)),
                            });
                            return output;
                        } else {
                            state.remaining_frames = state.remaining_frames.saturating_sub(1);
                            active.v2_step_state = Some(V2StepState::DragTo(state));
                            output.request_redraw = true;
                            return output;
                        }
                    };

                    let done = push_drag_playback_frame(&mut playback, &mut output.events);
                    output.request_redraw = true;
                    if done {
                        active.v2_step_state = None;
                        active.next_step = active.next_step.saturating_add(1);
                        if self.cfg.script_auto_dump {
                            force_dump_label = Some(format!("script-step-{step_index:04}-drag_to"));
                        }
                    } else {
                        state.playback = Some(playback);
                        active.v2_step_state = Some(V2StepState::DragTo(state));
                    }
                } else {
                    force_dump_label =
                        Some(format!("script-step-{step_index:04}-drag_to-no-semantics"));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
            UiActionStepV2::SetSliderValue {
                target,
                value,
                min,
                max,
                epsilon,
                timeout_frames,
                drag_steps,
            } => {
                active.wait_until = None;
                active.screenshot_wait = None;

                if let Some(snapshot) = semantics_snapshot {
                    let mut state = match active.v2_step_state.take() {
                        Some(V2StepState::SetSliderValue(mut state))
                            if state.step_index == step_index =>
                        {
                            state.remaining_frames = state.remaining_frames.min(timeout_frames);
                            state
                        }
                        _ => V2SetSliderValueState {
                            step_index,
                            remaining_frames: timeout_frames,
                            phase: 0,
                            last_drag_x: None,
                        },
                    };

                    let node = select_semantics_node_with_trace(
                        snapshot,
                        window,
                        element_runtime,
                        &target,
                        step_index as u32,
                        self.cfg.redact_text,
                        &mut active.selector_resolution_trace,
                    );
                    if let Some(node) = node {
                        if node.flags.disabled {
                            force_dump_label = Some(format!(
                                "script-step-{step_index:04}-set_slider_value-disabled"
                            ));
                            stop_script = true;
                            failure_reason = Some("set_slider_value_disabled".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                        } else {
                            let bounds = node.bounds;
                            let left = bounds.origin.x.0;
                            let width = bounds.size.width.0.max(0.0);
                            let right = left + width;
                            let span = (max - min).abs().max(0.0001);

                            let clamp_x = |x: f32| {
                                let pad = 2.0_f32;
                                x.clamp(left + pad, right - pad)
                            };
                            let target_t = ((value - min) / span).clamp(0.0, 1.0);

                            if state.phase == 0 {
                                let x = clamp_x(left + width * target_t);
                                let start = center_of_rect_clamped_to_rect(bounds, window_bounds);
                                let start_x = state.last_drag_x.unwrap_or(start.x.0);
                                let start = Point::new(fret_core::Px(start_x), start.y);
                                let end = Point::new(fret_core::Px(x), start.y);
                                if let Some(ui) = ui {
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &target,
                                        step_index as u32,
                                        start,
                                        Some(node),
                                        Some("set_slider_value.drag_start"),
                                    );
                                    record_hit_test_trace_for_selector(
                                        &mut active.hit_test_trace,
                                        ui,
                                        Some(snapshot),
                                        &target,
                                        step_index as u32,
                                        end,
                                        Some(node),
                                        Some("set_slider_value.drag_end"),
                                    );
                                }
                                output.events.extend(drag_events(
                                    start,
                                    end,
                                    UiMouseButtonV1::Left,
                                    drag_steps.max(1),
                                ));
                                state.phase = 1;
                                state.last_drag_x = Some(x);
                                active.v2_step_state = Some(V2StepState::SetSliderValue(state));
                                output.request_redraw = true;
                            } else {
                                let observed = node
                                    .value
                                    .as_deref()
                                    .and_then(parse_semantics_numeric_value);
                                if let Some(observed) = observed {
                                    if (observed - value).abs() <= epsilon.max(0.0) {
                                        active.v2_step_state = None;
                                        active.next_step = active.next_step.saturating_add(1);
                                        output.request_redraw = true;
                                        if self.cfg.script_auto_dump {
                                            force_dump_label = Some(format!(
                                                "script-step-{step_index:04}-set_slider_value"
                                            ));
                                        }
                                    } else if state.remaining_frames == 0 {
                                        force_dump_label = Some(format!(
                                            "script-step-{step_index:04}-set_slider_value-timeout"
                                        ));
                                        stop_script = true;
                                        failure_reason =
                                            Some("set_slider_value_timeout".to_string());
                                        active.v2_step_state = None;
                                        output.request_redraw = true;
                                    } else {
                                        let error = value - observed;
                                        let dx = (error / span) * width;
                                        let start =
                                            center_of_rect_clamped_to_rect(bounds, window_bounds);
                                        let start_x = state.last_drag_x.unwrap_or(start.x.0);
                                        let end_x = clamp_x(start_x + dx);
                                        let start = Point::new(fret_core::Px(start_x), start.y);
                                        let end = Point::new(fret_core::Px(end_x), start.y);
                                        if let Some(ui) = ui {
                                            record_hit_test_trace_for_selector(
                                                &mut active.hit_test_trace,
                                                ui,
                                                Some(snapshot),
                                                &target,
                                                step_index as u32,
                                                start,
                                                Some(node),
                                                Some("set_slider_value.adjust_drag_start"),
                                            );
                                            record_hit_test_trace_for_selector(
                                                &mut active.hit_test_trace,
                                                ui,
                                                Some(snapshot),
                                                &target,
                                                step_index as u32,
                                                end,
                                                Some(node),
                                                Some("set_slider_value.adjust_drag_end"),
                                            );
                                        }
                                        output.events.extend(drag_events(
                                            start,
                                            end,
                                            UiMouseButtonV1::Left,
                                            drag_steps.max(1),
                                        ));
                                        state.last_drag_x = Some(end_x);
                                        state.remaining_frames =
                                            state.remaining_frames.saturating_sub(1);
                                        active.v2_step_state =
                                            Some(V2StepState::SetSliderValue(state));
                                        output.request_redraw = true;
                                    }
                                } else {
                                    force_dump_label = Some(format!(
                                        "script-step-{step_index:04}-set_slider_value-unparseable"
                                    ));
                                    stop_script = true;
                                    failure_reason =
                                        Some("set_slider_value_unparseable".to_string());
                                    active.v2_step_state = None;
                                    output.request_redraw = true;
                                }
                            }
                        }
                    } else if state.remaining_frames == 0 {
                        force_dump_label = Some(format!(
                            "script-step-{step_index:04}-set_slider_value-timeout"
                        ));
                        stop_script = true;
                        failure_reason = Some("set_slider_value_timeout".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    } else {
                        state.remaining_frames = state.remaining_frames.saturating_sub(1);
                        active.v2_step_state = Some(V2StepState::SetSliderValue(state));
                        output.request_redraw = true;
                    }
                } else {
                    force_dump_label = Some(format!(
                        "script-step-{step_index:04}-set_slider_value-no-semantics"
                    ));
                    stop_script = true;
                    failure_reason = Some("no_semantics_snapshot".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                }
            }
        }

        if !output.events.is_empty() {
            for event in &output.events {
                self.record_script_event(app, window, event);
            }
        }

        if stop_script {
            if self.cfg.script_auto_dump {
                if let Some(label) = force_dump_label.as_deref() {
                    self.dump_bundle(Some(label));
                }
            } else if let Some(label) = force_dump_label {
                self.request_force_dump(label);
            }

            let reason_code = failure_reason
                .as_deref()
                .and_then(reason_code_for_script_failure)
                .map(|s| s.to_string());
            let evidence = script_evidence_for_active(&active);
            self.write_script_result(UiScriptResultV1 {
                schema_version: 1,
                run_id: active.run_id,
                updated_unix_ms: unix_ms_now(),
                window: Some(window.data().as_ffi()),
                stage: UiScriptStageV1::Failed,
                step_index: Some(step_index as u32),
                reason_code,
                reason: failure_reason,
                evidence,
                last_bundle_dir: self
                    .last_dump_dir
                    .as_ref()
                    .map(|p| display_path(&self.cfg.out_dir, p)),
            });
        } else {
            if let Some(label) = force_dump_label {
                self.request_force_dump(label);
            }

            if active.next_step >= active.steps.len() {
                self.write_script_result(UiScriptResultV1 {
                    schema_version: 1,
                    run_id: active.run_id,
                    updated_unix_ms: unix_ms_now(),
                    window: Some(window.data().as_ffi()),
                    stage: UiScriptStageV1::Passed,
                    step_index: Some(active.next_step.saturating_sub(1) as u32),
                    reason_code: None,
                    reason: None,
                    evidence: None,
                    last_bundle_dir: self
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&self.cfg.out_dir, p)),
                });
            } else if active.next_step < active.steps.len() {
                self.active_scripts.insert(window, active);
            }
        }
        output
    }

    fn record_script_event(&mut self, app: &App, window: AppWindowId, event: &Event) {
        let ring = self.per_window.entry(window).or_default();
        ring.update_pointer_position(event);

        let mut recorded = RecordedUiEventV1::from_event(app, window, event, self.cfg.redact_text);
        truncate_string_bytes(&mut recorded.debug, self.cfg.max_debug_string_bytes);
        ring.push_event(&self.cfg, recorded);
    }

    fn ensure_ready_file(&mut self) {
        if self.ready_written {
            return;
        }
        if !self.cfg.enabled {
            return;
        }

        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            // Web runners do not have a stable filesystem surface for the legacy `ready.touch` file.
            self.ready_written = true;
            return;
        }

        if let Some(parent) = self.cfg.ready_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        self.ensure_capabilities_file();

        let ts = unix_ms_now();
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.cfg.ready_path)
        {
            use std::io::Write as _;
            let _ = writeln!(f, "{ts}");
            let _ = f.flush();
        }

        self.ready_written = true;
    }

    fn ensure_capabilities_file(&mut self) {
        if self.capabilities_written {
            return;
        }
        if !self.cfg.enabled {
            return;
        }
        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            self.capabilities_written = true;
            return;
        }

        let mut caps = vec!["diag.script_v2".to_string()];
        if self.cfg.screenshots_enabled {
            caps.push("diag.screenshot_png".to_string());
        }

        let path = self.cfg.out_dir.join("capabilities.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let payload = FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: caps,
        };
        if let Ok(mut text) = serde_json::to_string_pretty(&payload) {
            text.push('\n');
            let _ = std::fs::write(&path, text);
        }

        self.capabilities_written = true;
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.per_window.remove(&window);
        self.active_scripts.remove(&window);
        self.last_picked_node_id.remove(&window);
        self.last_picked_selector_json.remove(&window);
        self.last_hovered_node_id.remove(&window);
        self.last_hovered_selector_json.remove(&window);
        self.inspect_focus_node_id.remove(&window);
        self.inspect_focus_selector_json.remove(&window);
        self.inspect_focus_down_stack.remove(&window);
        self.inspect_pending_nav.remove(&window);
        self.inspect_focus_summary_line.remove(&window);
        self.inspect_focus_path_line.remove(&window);
        self.inspect_locked_windows.remove(&window);
        self.inspect_toast.remove(&window);
        if self
            .pending_pick
            .as_ref()
            .is_some_and(|p| p.window == window)
        {
            self.pending_pick = None;
        }
    }

    fn reset_diagnostics_ring_for_window(&mut self, window: AppWindowId) {
        self.per_window.entry(window).or_default().clear();
    }

    pub fn update_inspect_hover(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        hovered_node_id: Option<u64>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }
        if !self.inspect_enabled {
            return;
        }
        let Some(snapshot) = snapshot else {
            return;
        };
        let Some(hovered_id) = hovered_node_id else {
            self.last_hovered_node_id.remove(&window);
            self.last_hovered_selector_json.remove(&window);
            return;
        };
        if self.inspect_is_locked(window) {
            return;
        }

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == hovered_id)
        else {
            return;
        };
        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);
        let Some(selector) = best_selector_for_node(snapshot, node, element, &self.cfg) else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.last_hovered_node_id.insert(window, hovered_id);
            self.last_hovered_selector_json.insert(window, json);
            self.inspect_focus_node_id.insert(window, hovered_id);
            if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                self.inspect_focus_selector_json.insert(window, sel);
            }
            self.inspect_focus_down_stack.insert(window, Vec::new());
        }
    }

    fn push_inspect_toast(&mut self, window: AppWindowId, message: String) {
        self.inspect_toast.insert(
            window,
            InspectToast {
                message,
                remaining_frames: 90,
            },
        );
    }

    pub fn apply_inspect_navigation(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }
        if !self.inspect_enabled {
            self.inspect_pending_nav.remove(&window);
            return;
        }
        let Some(cmd) = self.inspect_pending_nav.remove(&window) else {
            return;
        };
        let Some(snapshot) = snapshot else {
            self.push_inspect_toast(window, "inspect: no semantics snapshot".to_string());
            return;
        };

        match cmd {
            InspectNavCommand::Focus => {
                let Some(node) = snapshot.focus else {
                    self.push_inspect_toast(window, "inspect: no focused node".to_string());
                    return;
                };
                let id = node.data().as_ffi();
                self.inspect_focus_down_stack.insert(window, Vec::new());
                self.inspect_locked_windows.insert(window);
                self.set_inspect_focus(window, snapshot, id, element_runtime);
            }
            InspectNavCommand::Up => {
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }

                let current = self
                    .inspect_focus_node_id
                    .get(&window)
                    .copied()
                    .or_else(|| self.last_picked_node_id.get(&window).copied())
                    .or_else(|| self.last_hovered_node_id.get(&window).copied());
                let Some(current) = current else {
                    self.push_inspect_toast(window, "inspect: no focused node".to_string());
                    return;
                };

                let Some(parent) = parent_node_id(snapshot, current) else {
                    self.push_inspect_toast(window, "inspect: reached root".to_string());
                    return;
                };
                self.inspect_focus_down_stack
                    .entry(window)
                    .or_default()
                    .push(current);
                self.set_inspect_focus(window, snapshot, parent, element_runtime);
                self.push_inspect_toast(window, "inspect: parent".to_string());
            }
            InspectNavCommand::Down => {
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }
                let Some(prev) = self
                    .inspect_focus_down_stack
                    .get_mut(&window)
                    .and_then(|s| s.pop())
                else {
                    self.push_inspect_toast(window, "inspect: no child history".to_string());
                    return;
                };
                self.set_inspect_focus(window, snapshot, prev, element_runtime);
                self.push_inspect_toast(window, "inspect: child".to_string());
            }
        }
    }

    fn set_inspect_focus(
        &mut self,
        window: AppWindowId,
        snapshot: &fret_core::SemanticsSnapshot,
        node_id: u64,
        element_runtime: Option<&ElementRuntime>,
    ) {
        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            return;
        };
        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);
        let Some(selector) = best_selector_for_node(snapshot, node, element, &self.cfg) else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.inspect_focus_node_id.insert(window, node_id);
            self.inspect_focus_selector_json
                .insert(window, json.clone());
            self.last_picked_node_id.insert(window, node_id);
            self.last_picked_selector_json.insert(window, json);
        }
    }

    fn update_inspect_focus_lines(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }
        let Some(snapshot) = snapshot else {
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
            return;
        };

        let node_id = self
            .inspect_focus_node_id
            .get(&window)
            .copied()
            .or_else(|| self.last_picked_node_id.get(&window).copied())
            .or_else(|| self.last_hovered_node_id.get(&window).copied());
        let Some(node_id) = node_id else {
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
            return;
        };

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
            return;
        };

        let role = semantics_role_label(node.role);
        let mut summary = format!("focus: {role} node={node_id}");

        if let Some(runtime) = element_runtime
            && let Some(element) = runtime.element_for_node(window, node.id)
        {
            summary.push_str(&format!(" element={}", element.0));
            if let Some(path) = runtime.debug_path_for_element(window, element) {
                let path = truncate_debug_value(&path, 200);
                summary.push_str(&format!(" element_path={path}"));
            }
        }
        if let Some(test_id) = node.test_id.as_deref() {
            summary.push_str(&format!(" test_id={test_id}"));
        }
        if !self.cfg.redact_text
            && let Some(label) = node.label.as_deref()
        {
            let label = truncate_debug_value(label, 120);
            summary.push_str(&format!(" label={label}"));
        }

        let path = format_inspect_path(snapshot, node_id, self.cfg.redact_text, 10);

        self.inspect_focus_summary_line.insert(window, summary);
        if let Some(path) = path {
            self.inspect_focus_path_line.insert(window, path);
        } else {
            self.inspect_focus_path_line.remove(&window);
        }
    }

    fn inspect_copy_details_payload(&self, window: AppWindowId) -> String {
        let selector = self.inspect_best_selector_json(window);
        let summary = self.inspect_focus_summary_line(window);
        let path = self.inspect_focus_path_line(window);

        let mut lines: Vec<String> = Vec::new();
        if let Some(selector) = selector {
            lines.push(format!("selector: {selector}"));
        }
        if let Some(summary) = summary {
            lines.push(summary.to_string());
        }
        if let Some(path) = path {
            lines.push(path.to_string());
        }
        lines.join("\n")
    }

    pub fn record_model_changes(&mut self, window: AppWindowId, changed: &[ModelId]) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_models = changed.iter().map(|id| id.data().as_ffi()).collect();
    }

    pub fn record_global_changes(
        &mut self,
        app: &App,
        window: AppWindowId,
        changed: &[std::any::TypeId],
    ) {
        if !self.is_enabled() {
            return;
        }
        let ring = self.per_window.entry(window).or_default();
        ring.last_changed_globals = changed
            .iter()
            .map(|&t| {
                app.global_type_name(t)
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| format!("{t:?}"))
            })
            .collect();
    }

    pub fn record_event(&mut self, app: &App, window: AppWindowId, event: &Event) {
        if !self.is_enabled() {
            return;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let ring = self.per_window.entry(window).or_default();
        ring.update_pointer_position(event);

        let mut recorded = RecordedUiEventV1::from_event(app, window, event, self.cfg.redact_text);
        truncate_string_bytes(&mut recorded.debug, self.cfg.max_debug_string_bytes);
        ring.push_event(&self.cfg, recorded);
    }

    pub fn record_viewport_input(&mut self, event: fret_core::ViewportInputEvent) {
        if !self.is_enabled() {
            return;
        }

        let ring = self.per_window.entry(event.window).or_default();
        if ring.viewport_input_this_frame.len() >= self.cfg.max_events {
            return;
        }
        ring.viewport_input_this_frame
            .push(UiViewportInputEventV1::from_event(event));
    }

    pub fn record_snapshot(
        &mut self,
        app: &App,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        scene: &Scene,
    ) {
        if !self.is_enabled() {
            return;
        }

        let last_pointer_position = self
            .per_window
            .get(&window)
            .and_then(|ring| ring.last_pointer_position);
        let hit_test = last_pointer_position.map(|pos| UiHitTestSnapshotV1::from_tree(pos, ui));

        let element_diag = element_runtime.and_then(|runtime| {
            runtime.diagnostics_snapshot(window).map(|snapshot| {
                ElementDiagnosticsSnapshotV1::from_runtime(window, runtime, snapshot)
            })
        });

        let raw_semantics = ui.semantics_snapshot();
        let semantics_fingerprint = raw_semantics.map(|snapshot| {
            semantics_fingerprint_v1(
                snapshot,
                self.cfg.redact_text,
                self.cfg.max_debug_string_bytes,
            )
        });

        if self.inspect_enabled {
            let hovered = last_pointer_position.and_then(|pos| {
                raw_semantics.and_then(|snap| {
                    pick_semantics_node_by_bounds(snap, pos).map(|n| n.id.data().as_ffi())
                })
            });
            self.update_inspect_hover(window, raw_semantics, hovered, element_runtime);
        }
        self.apply_inspect_navigation(window, raw_semantics, element_runtime);
        self.update_inspect_focus_lines(window, raw_semantics, element_runtime);

        let semantics = self
            .cfg
            .capture_semantics
            .then_some(raw_semantics)
            .flatten()
            .map(|snap| {
                UiSemanticsSnapshotV1::from_snapshot(
                    snap,
                    self.cfg.redact_text,
                    self.cfg.max_debug_string_bytes,
                )
            });

        let ring = self.per_window.entry(window).or_default();
        let viewport_input = std::mem::take(&mut ring.viewport_input_this_frame);

        let changed_models = std::mem::take(&mut ring.last_changed_models);
        let changed_model_sources_top = if cfg!(debug_assertions) && !changed_models.is_empty() {
            let mut counts: HashMap<(String, String, u32, u32), u32> = HashMap::new();
            for &model in &changed_models {
                let id = ModelId::from(KeyData::from_ffi(model));
                let Some(info) = app.models().debug_last_changed_info_for_id(id) else {
                    continue;
                };
                let ty = info.type_name.to_string();
                *counts
                    .entry((ty, info.file.to_string(), info.line, info.column))
                    .or_insert(0) += 1;
            }
            let mut out: Vec<UiChangedModelSourceHotspotV1> = counts
                .into_iter()
                .map(
                    |((type_name, file, line, column), count)| UiChangedModelSourceHotspotV1 {
                        type_name,
                        changed_at: UiSourceLocationV1 { file, line, column },
                        count,
                    },
                )
                .collect();
            out.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.type_name.cmp(&b.type_name))
                    .then_with(|| a.changed_at.file.cmp(&b.changed_at.file))
                    .then_with(|| a.changed_at.line.cmp(&b.changed_at.line))
                    .then_with(|| a.changed_at.column.cmp(&b.changed_at.column))
            });
            out.truncate(8);
            out
        } else {
            Vec::new()
        };

        let resource_caches = {
            let icon_svg_cache = icon_svg_cache_stats(app);
            let canvas = canvas_cache_stats_for_window(app, window.data().as_ffi());
            let render_text = app
                .global::<fret_core::RendererTextPerfSnapshot>()
                .copied()
                .map(UiRendererTextPerfSnapshotV1::from_core);
            (icon_svg_cache.is_some() || !canvas.is_empty() || render_text.is_some()).then_some(
                UiResourceCachesV1 {
                    icon_svg_cache,
                    canvas,
                    render_text,
                },
            )
        };

        let renderer_perf = app
            .global::<fret_render::RendererPerfFrameStore>()
            .and_then(|store| store.latest_for_window(window));

        let mut debug = UiTreeDebugSnapshotV1::from_tree(
            app,
            window,
            ui,
            renderer_perf,
            element_runtime,
            hit_test,
            element_diag,
            semantics,
            self.cfg.max_gating_trace_entries,
        );
        debug.viewport_input = viewport_input;

        let app_snapshot = self
            .app_snapshot_provider
            .as_ref()
            .and_then(|provider| provider(app, window));

        let snapshot = UiDiagnosticsSnapshotV1 {
            schema_version: 1,
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window: window.data().as_ffi(),
            timestamp_unix_ms: unix_ms_now(),
            scale_factor,
            window_bounds: RectV1::from(bounds),
            scene_ops: scene.ops_len() as u64,
            scene_fingerprint: scene.fingerprint(),
            semantics_fingerprint,
            debug,
            changed_models,
            changed_globals: std::mem::take(&mut ring.last_changed_globals),
            changed_model_sources_top,
            resource_caches,
            app_snapshot,
        };

        ring.push_snapshot(&self.cfg, snapshot);

        if let Some(pending) = self.pending_pick.clone()
            && pending.window == window
        {
            self.resolve_pending_pick_for_window(
                window,
                pending.position,
                raw_semantics,
                ui,
                element_runtime,
            );
        }
    }

    pub fn maybe_dump_if_triggered(&mut self) -> Option<PathBuf> {
        if !self.is_enabled() {
            return None;
        }

        self.poll_ws_inbox();

        if let Some(label) = self.pending_force_dump_label.take() {
            return self.dump_bundle(Some(&label));
        }

        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            return None;
        }

        let Some(stamp) = read_touch_stamp(&self.cfg.trigger_path) else {
            if let Some(dir) = self.cfg.trigger_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            // Prime the trigger file with a baseline stamp so external drivers can reliably
            // advance it (Windows mtime resolution is not always sufficient for edge detection).
            let _ = std::fs::write(&self.cfg.trigger_path, b"0\n");
            self.last_trigger_stamp = Some(0);
            return None;
        };

        // Treat the first observed value as a baseline, not a trigger (avoids dumping stale runs
        // when the diagnostics directory is reused between launches).
        let Some(prev) = self.last_trigger_stamp else {
            self.last_trigger_stamp = Some(stamp);
            return None;
        };
        if prev == stamp {
            return None;
        }
        self.last_trigger_stamp = Some(stamp);

        self.dump_bundle(None)
    }

    fn request_force_dump(&mut self, label: String) {
        self.pending_force_dump_label = Some(sanitize_label(&label));
    }

    #[cfg(feature = "diagnostics-ws")]
    fn ws_is_configured(&self) -> bool {
        self.cfg.devtools_ws_url.is_some() && self.cfg.devtools_token.is_some()
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    fn ws_is_configured(&self) -> bool {
        false
    }

    #[cfg(feature = "diagnostics-ws")]
    fn poll_ws_inbox(&mut self) {
        if !self.ws_is_configured() {
            return;
        }

        let mut msgs: Vec<DiagTransportMessageV1> = Vec::new();
        self.ws_bridge.drain_inbox(
            self.cfg.devtools_ws_url.as_deref(),
            self.cfg.devtools_token.as_deref(),
            self.cfg.screenshots_enabled,
            &mut msgs,
        );

        for msg in msgs {
            self.apply_ws_message(msg);
        }
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    fn poll_ws_inbox(&mut self) {}

    #[cfg(feature = "diagnostics-ws")]
    fn ws_send(&mut self, ty: impl Into<String>, payload: serde_json::Value) {
        if !self.ws_is_configured() {
            return;
        }
        self.ws_bridge.send(
            self.cfg.devtools_ws_url.as_deref(),
            self.cfg.devtools_token.as_deref(),
            self.cfg.screenshots_enabled,
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: ty.into(),
                session_id: None,
                request_id: None,
                payload,
            },
        );
    }

    #[cfg(feature = "diagnostics-ws")]
    fn apply_ws_message(&mut self, msg: DiagTransportMessageV1) {
        match msg.r#type.as_str() {
            "inspect.set" => {
                let Ok(cfg) = serde_json::from_value::<UiInspectConfigV1>(msg.payload) else {
                    return;
                };
                self.inspect_enabled = cfg.enabled;
                self.inspect_consume_clicks = cfg.consume_clicks;
                if !self.inspect_enabled {
                    self.inspect_locked_windows.clear();
                }
            }
            "pick.arm" => {
                self.pending_pick = None;
                self.pick_armed_run_id = Some(self.next_pick_run_id());
            }
            "bundle.dump" => {
                let label = serde_json::from_value::<DevtoolsBundleDumpV1>(msg.payload)
                    .ok()
                    .and_then(|v| v.label)
                    .unwrap_or_else(|| "bundle".to_string());
                self.request_force_dump(label);
            }
            "script.push" | "script.run" => {
                let script_value = msg
                    .payload
                    .get("script")
                    .cloned()
                    .unwrap_or_else(|| msg.payload.clone());
                let Some(script) = PendingScript::from_json_value(script_value) else {
                    return;
                };
                let run_id = self.next_script_run_id();
                self.pending_script = Some(script);
                self.pending_script_run_id = Some(run_id);
                self.write_script_result(UiScriptResultV1 {
                    schema_version: 1,
                    run_id,
                    updated_unix_ms: unix_ms_now(),
                    window: None,
                    stage: UiScriptStageV1::Queued,
                    step_index: None,
                    reason_code: None,
                    reason: None,
                    evidence: None,
                    last_bundle_dir: self
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&self.cfg.out_dir, p)),
                });
            }
            _ => {}
        }
    }

    fn poll_script_trigger(&mut self) {
        self.poll_ws_inbox();
        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            return;
        }

        let Some(stamp) = read_touch_stamp(&self.cfg.script_trigger_path) else {
            if let Some(dir) = self.cfg.script_trigger_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            // Prime the trigger file with a baseline stamp so external drivers can reliably
            // advance it (Windows mtime resolution is not always sufficient for edge detection).
            let _ = std::fs::write(&self.cfg.script_trigger_path, b"0\n");
            self.last_script_trigger_stamp = Some(0);
            return;
        };

        // Treat the first observed value as a baseline, not a trigger (avoids re-running stale scripts
        // when the diagnostics directory is reused between runs).
        let Some(prev) = self.last_script_trigger_stamp else {
            self.last_script_trigger_stamp = Some(stamp);
            return;
        };
        if prev == stamp {
            return;
        }
        self.last_script_trigger_stamp = Some(stamp);

        let bytes = std::fs::read(&self.cfg.script_path).ok();
        let Some(bytes) = bytes else {
            return;
        };
        let schema_version: u32 = serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|v| v.get("schema_version").and_then(|v| v.as_u64()))
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32;

        let script = match schema_version {
            1 => {
                let Ok(script) = serde_json::from_slice::<UiActionScriptV1>(&bytes) else {
                    return;
                };
                let Some(script) = PendingScript::from_v1(script) else {
                    return;
                };
                script
            }
            2 => {
                let Ok(script) = serde_json::from_slice::<UiActionScriptV2>(&bytes) else {
                    return;
                };
                let Some(script) = PendingScript::from_v2(script) else {
                    return;
                };
                script
            }
            _ => return,
        };
        let run_id = self.next_script_run_id();
        self.pending_script = Some(script);
        self.pending_script_run_id = Some(run_id);
        self.write_script_result(UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: unix_ms_now(),
            window: None,
            stage: UiScriptStageV1::Queued,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: self
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&self.cfg.out_dir, p)),
        });
    }

    fn dump_bundle(&mut self, label: Option<&str>) -> Option<PathBuf> {
        let ts = unix_ms_now();
        let mut dir_name = ts.to_string();
        if let Some(label) = label {
            if !label.is_empty() {
                dir_name = format!("{dir_name}-{label}");
            }
        }

        let dir = self.cfg.out_dir.join(dir_name);

        let bundle = UiDiagnosticsBundleV1::from_service(ts, &dir, self);

        if !cfg!(target_arch = "wasm32") {
            if std::fs::create_dir_all(&dir).is_err() {
                return None;
            }
            if write_json(dir.join("bundle.json"), &bundle).is_err() {
                return None;
            }
            let _ = write_latest_pointer(&self.cfg.out_dir, &dir);
            if self.cfg.screenshot_on_dump {
                let _ = std::fs::write(dir.join("screenshot.request"), b"1\n");
            }
        }

        self.last_dump_dir = Some(dir.clone());

        #[cfg(feature = "diagnostics-ws")]
        {
            let embed = self.cfg.devtools_embed_bundle || cfg!(target_arch = "wasm32");
            let bundle_value = embed.then(|| serde_json::to_value(&bundle).ok()).flatten();
            let payload = serde_json::to_value(DevtoolsBundleDumpedV1 {
                schema_version: 1,
                exported_unix_ms: ts,
                out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                dir: display_path(&self.cfg.out_dir, &dir),
                bundle: bundle_value,
            })
            .unwrap_or(serde_json::Value::Null);
            self.ws_send("bundle.dumped", payload);
        }

        Some(dir)
    }

    fn next_script_run_id(&mut self) -> u64 {
        let mut id = unix_ms_now();
        if id <= self.last_script_run_id {
            id = self.last_script_run_id.saturating_add(1);
        }
        self.last_script_run_id = id;
        id
    }

    fn next_pick_run_id(&mut self) -> u64 {
        let mut id = unix_ms_now();
        if id <= self.last_pick_run_id {
            id = self.last_pick_run_id.saturating_add(1);
        }
        self.last_pick_run_id = id;
        id
    }

    fn write_script_result(&mut self, result: UiScriptResultV1) {
        if !self.is_enabled() {
            return;
        }

        if !cfg!(target_arch = "wasm32") {
            let _ = write_json(self.cfg.script_result_path.clone(), &result);
            let _ = touch_file(&self.cfg.script_result_trigger_path);
        }

        #[cfg(feature = "diagnostics-ws")]
        {
            let payload = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
            self.ws_send("script.result", payload);
        }
    }

    fn write_pick_result(&mut self, result: UiPickResultV1) {
        if !self.is_enabled() {
            return;
        }

        if !cfg!(target_arch = "wasm32") {
            let _ = write_json(self.cfg.pick_result_path.clone(), &result);
            let _ = touch_file(&self.cfg.pick_result_trigger_path);
        }

        #[cfg(feature = "diagnostics-ws")]
        {
            let payload = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
            self.ws_send("pick.result", payload);
        }
    }

    fn poll_pick_trigger(&mut self) {
        self.poll_ws_inbox();
        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            return;
        }

        let modified =
            match std::fs::metadata(&self.cfg.pick_trigger_path).and_then(|m| m.modified()) {
                Ok(modified) => modified,
                Err(_) => {
                    if let Some(dir) = self.cfg.pick_trigger_path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    if std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&self.cfg.pick_trigger_path)
                        .is_ok()
                        && let Ok(modified) = std::fs::metadata(&self.cfg.pick_trigger_path)
                            .and_then(|m| m.modified())
                    {
                        self.last_pick_trigger_mtime = Some(modified);
                    }
                    return;
                }
            };
        if self.last_pick_trigger_mtime.is_none() {
            self.last_pick_trigger_mtime = Some(modified);
            return;
        }
        if self
            .last_pick_trigger_mtime
            .is_some_and(|prev| prev >= modified)
        {
            return;
        }
        self.last_pick_trigger_mtime = Some(modified);

        self.pending_pick = None;
        self.pick_armed_run_id = Some(self.next_pick_run_id());
    }

    fn poll_inspect_trigger(&mut self) {
        self.poll_ws_inbox();
        if cfg!(target_arch = "wasm32") && self.ws_is_configured() {
            return;
        }

        let modified =
            match std::fs::metadata(&self.cfg.inspect_trigger_path).and_then(|m| m.modified()) {
                Ok(modified) => modified,
                Err(_) => {
                    if let Some(dir) = self.cfg.inspect_trigger_path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    if std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&self.cfg.inspect_trigger_path)
                        .is_ok()
                        && let Ok(modified) = std::fs::metadata(&self.cfg.inspect_trigger_path)
                            .and_then(|m| m.modified())
                    {
                        self.last_inspect_trigger_mtime = Some(modified);
                    }
                    return;
                }
            };
        if self.last_inspect_trigger_mtime.is_none() {
            self.last_inspect_trigger_mtime = Some(modified);
            return;
        }
        if self
            .last_inspect_trigger_mtime
            .is_some_and(|prev| prev >= modified)
        {
            return;
        }
        self.last_inspect_trigger_mtime = Some(modified);

        let bytes = std::fs::read(&self.cfg.inspect_path).ok();
        let Some(bytes) = bytes else {
            return;
        };
        let cfg: UiInspectConfigV1 = match serde_json::from_slice(&bytes) {
            Ok(cfg) => cfg,
            Err(_) => return,
        };
        if cfg.schema_version != 1 {
            return;
        }

        self.inspect_enabled = cfg.enabled;
        self.inspect_consume_clicks = cfg.consume_clicks;
    }

    fn resolve_pending_pick_for_window(
        &mut self,
        window: AppWindowId,
        position: Point,
        raw_semantics: Option<&fret_core::SemanticsSnapshot>,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        let Some(pending) = self.pending_pick.clone() else {
            return;
        };
        if pending.window != window {
            return;
        }

        let mut result = UiPickResultV1 {
            schema_version: 1,
            run_id: pending.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiPickStageV1::Failed,
            position: Some(PointV1::from(position)),
            selection: None,
            reason: None,
            last_bundle_dir: self
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&self.cfg.out_dir, p)),
        };

        let selection = match raw_semantics {
            Some(snapshot) => pick_semantics_node_at(snapshot, ui, position).map(|node| {
                let (element, element_path) = element_runtime
                    .and_then(|runtime| {
                        runtime.element_for_node(window, node.id).map(|id| {
                            let path = runtime.debug_path_for_element(window, id);
                            (Some(id.0), path)
                        })
                    })
                    .unwrap_or((None, None));
                UiPickSelectionV1::from_node(snapshot, node, element, element_path, &self.cfg)
            }),
            None => None,
        };

        match selection {
            Some(sel) => {
                result.stage = UiPickStageV1::Picked;
                self.last_picked_node_id.insert(window, sel.node.id);
                if let Some(best) = sel.selectors.first()
                    && let Ok(json) = serde_json::to_string(best)
                {
                    self.last_picked_selector_json.insert(window, json.clone());
                    self.inspect_focus_node_id.insert(window, sel.node.id);
                    self.inspect_focus_selector_json.insert(window, json);
                    self.inspect_focus_down_stack.insert(window, Vec::new());
                }
                self.pick_overlay_grace_frames.insert(window, 10);
                result.selection = Some(sel);
            }
            None => {
                result.reason = Some("no matching semantics node under pointer".to_string());
            }
        }

        if self.cfg.pick_auto_dump {
            if let Some(dir) = self.dump_bundle(Some("pick")) {
                result.last_bundle_dir = Some(display_path(&self.cfg.out_dir, &dir));
            }
        }

        self.write_pick_result(result);
        self.pending_pick = None;
    }
}

fn active_script_needs_semantics_snapshot(active: &ActiveScript) -> bool {
    if active.wait_until.is_some() {
        return true;
    }

    if active.v2_step_state.is_some() {
        return false;
    }

    let Some(step) = active.steps.get(active.next_step) else {
        return false;
    };

    match step {
        UiActionStepV2::Click { .. }
        | UiActionStepV2::ClickStable { .. }
        | UiActionStepV2::MovePointer { .. }
        | UiActionStepV2::DragPointer { .. }
        | UiActionStepV2::MovePointerSweep { .. }
        | UiActionStepV2::Wheel { .. }
        | UiActionStepV2::WaitUntil { .. }
        | UiActionStepV2::Assert { .. }
        | UiActionStepV2::EnsureVisible { .. }
        | UiActionStepV2::ScrollIntoView { .. }
        | UiActionStepV2::TypeTextInto { .. }
        | UiActionStepV2::MenuSelect { .. }
        | UiActionStepV2::MenuSelectPath { .. }
        | UiActionStepV2::DragTo { .. }
        | UiActionStepV2::SetSliderValue { .. } => true,
        UiActionStepV2::ResetDiagnostics
        | UiActionStepV2::PressKey { .. }
        | UiActionStepV2::PressShortcut { .. }
        | UiActionStepV2::TypeText { .. }
        | UiActionStepV2::WaitFrames { .. }
        | UiActionStepV2::CaptureBundle { .. }
        | UiActionStepV2::CaptureScreenshot { .. }
        | UiActionStepV2::SetWindowInnerSize { .. } => false,
    }
}

fn read_touch_stamp(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let text = std::str::from_utf8(&bytes).ok()?;
    text.lines()
        .rev()
        .find_map(|line| line.trim().parse::<u64>().ok())
}

#[derive(Debug, Clone)]
struct PendingPick {
    run_id: u64,
    window: AppWindowId,
    position: Point,
}

#[derive(Default)]
struct WindowRing {
    last_pointer_position: Option<Point>,
    events: VecDeque<RecordedUiEventV1>,
    snapshots: VecDeque<UiDiagnosticsSnapshotV1>,
    viewport_input_this_frame: Vec<UiViewportInputEventV1>,
    last_changed_models: Vec<u64>,
    last_changed_globals: Vec<String>,
}

impl WindowRing {
    fn update_pointer_position(&mut self, event: &Event) {
        let Some(pointer) = event.pointer_event() else {
            return;
        };
        self.last_pointer_position = Some(pointer.position());
    }

    fn clear(&mut self) {
        self.last_pointer_position = None;
        self.events.clear();
        self.snapshots.clear();
        self.viewport_input_this_frame.clear();
        self.last_changed_models.clear();
        self.last_changed_globals.clear();
    }

    fn push_event(&mut self, cfg: &UiDiagnosticsConfig, event: RecordedUiEventV1) {
        self.events.push_back(event);
        while self.events.len() > cfg.max_events {
            self.events.pop_front();
        }
    }

    fn push_snapshot(&mut self, cfg: &UiDiagnosticsConfig, snapshot: UiDiagnosticsSnapshotV1) {
        self.snapshots.push_back(snapshot);
        while self.snapshots.len() > cfg.max_snapshots {
            self.snapshots.pop_front();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub out_dir: String,
    pub config: UiDiagnosticsBundleConfigV1,
    pub windows: Vec<UiDiagnosticsWindowBundleV1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleConfigV1 {
    pub trigger_path: String,
    pub max_events: usize,
    pub max_snapshots: usize,
    pub capture_semantics: bool,
    pub script_path: String,
    pub script_trigger_path: String,
    pub script_result_path: String,
    pub script_result_trigger_path: String,
    pub script_auto_dump: bool,
    pub pick_trigger_path: String,
    pub pick_result_path: String,
    pub pick_result_trigger_path: String,
    pub pick_auto_dump: bool,
    #[serde(default)]
    pub inspect_path: String,
    #[serde(default)]
    pub inspect_trigger_path: String,
    pub redact_text: bool,
    pub max_debug_string_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsWindowBundleV1 {
    pub window: u64,
    pub events: Vec<RecordedUiEventV1>,
    pub snapshots: Vec<UiDiagnosticsSnapshotV1>,
}

impl UiDiagnosticsBundleV1 {
    fn from_service(exported_unix_ms: u64, out_dir: &Path, svc: &UiDiagnosticsService) -> Self {
        Self {
            schema_version: 1,
            exported_unix_ms,
            out_dir: sanitize_path_for_bundle(&svc.cfg.out_dir, out_dir),
            config: UiDiagnosticsBundleConfigV1 {
                trigger_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.trigger_path),
                max_events: svc.cfg.max_events,
                max_snapshots: svc.cfg.max_snapshots,
                capture_semantics: svc.cfg.capture_semantics,
                script_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.script_path),
                script_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_trigger_path,
                ),
                script_result_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_result_path,
                ),
                script_result_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_result_trigger_path,
                ),
                script_auto_dump: svc.cfg.script_auto_dump,
                pick_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_trigger_path,
                ),
                pick_result_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_result_path,
                ),
                pick_result_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_result_trigger_path,
                ),
                pick_auto_dump: svc.cfg.pick_auto_dump,
                inspect_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.inspect_path),
                inspect_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.inspect_trigger_path,
                ),
                redact_text: svc.cfg.redact_text,
                max_debug_string_bytes: svc.cfg.max_debug_string_bytes,
            },
            windows: svc
                .per_window
                .iter()
                .map(|(window, ring)| UiDiagnosticsWindowBundleV1 {
                    window: window.data().as_ffi(),
                    events: ring.events.iter().cloned().collect(),
                    snapshots: ring.snapshots.iter().cloned().collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsSnapshotV1 {
    pub schema_version: u32,
    pub tick_id: u64,
    pub frame_id: u64,
    pub window: u64,
    pub timestamp_unix_ms: u64,
    pub scale_factor: f32,
    pub window_bounds: RectV1,
    pub scene_ops: u64,
    #[serde(default)]
    pub scene_fingerprint: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_fingerprint: Option<u64>,

    pub changed_models: Vec<u64>,
    pub changed_globals: Vec<String>,

    /// Aggregated writers for `changed_models`, derived from `ModelStore` debug info.
    ///
    /// This is best-effort and only populated in debug builds.
    #[serde(default)]
    pub changed_model_sources_top: Vec<UiChangedModelSourceHotspotV1>,

    #[serde(default)]
    pub resource_caches: Option<UiResourceCachesV1>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_snapshot: Option<serde_json::Value>,

    pub debug: UiTreeDebugSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiChangedModelSourceHotspotV1 {
    pub type_name: String,
    pub changed_at: UiSourceLocationV1,
    pub count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiResourceCachesV1 {
    #[serde(default)]
    pub icon_svg_cache: Option<UiRetainedSvgCacheStatsV1>,
    #[serde(default)]
    pub canvas: Vec<UiCanvasCacheEntryV1>,
    #[serde(default)]
    pub render_text: Option<UiRendererTextPerfSnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRetainedSvgCacheStatsV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiCacheStatsV1 {
    pub get_calls: u64,
    pub get_hits: u64,
    pub get_misses: u64,
    pub prepare_calls: u64,
    pub prepare_hits: u64,
    pub prepare_misses: u64,
    pub prune_calls: u64,
    pub clear_calls: u64,
    pub evict_calls: u64,
    pub release_replaced: u64,
    pub release_prune_age: u64,
    pub release_prune_budget: u64,
    pub release_clear: u64,
    pub release_evict: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererTextPerfSnapshotV1 {
    pub frame_id: u64,

    pub font_stack_key: u64,
    pub font_db_revision: u64,

    pub blobs_live: u64,
    pub blob_cache_entries: u64,
    pub shape_cache_entries: u64,
    pub measure_cache_buckets: u64,

    #[serde(default)]
    pub unwrapped_layout_cache_entries: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_hits: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_misses: u64,
    #[serde(default)]
    pub frame_unwrapped_layouts_created: u64,

    pub frame_cache_resets: u64,
    pub frame_blob_cache_hits: u64,
    pub frame_blob_cache_misses: u64,
    pub frame_blobs_created: u64,
    pub frame_shape_cache_hits: u64,
    pub frame_shape_cache_misses: u64,
    pub frame_shapes_created: u64,

    pub mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub color_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
}

impl UiRendererTextPerfSnapshotV1 {
    fn from_core(snapshot: fret_core::RendererTextPerfSnapshot) -> Self {
        Self {
            frame_id: snapshot.frame_id.0,
            font_stack_key: snapshot.font_stack_key,
            font_db_revision: snapshot.font_db_revision,
            blobs_live: snapshot.blobs_live,
            blob_cache_entries: snapshot.blob_cache_entries,
            shape_cache_entries: snapshot.shape_cache_entries,
            measure_cache_buckets: snapshot.measure_cache_buckets,
            unwrapped_layout_cache_entries: snapshot.unwrapped_layout_cache_entries,
            frame_unwrapped_layout_cache_hits: snapshot.frame_unwrapped_layout_cache_hits,
            frame_unwrapped_layout_cache_misses: snapshot.frame_unwrapped_layout_cache_misses,
            frame_unwrapped_layouts_created: snapshot.frame_unwrapped_layouts_created,
            frame_cache_resets: snapshot.frame_cache_resets,
            frame_blob_cache_hits: snapshot.frame_blob_cache_hits,
            frame_blob_cache_misses: snapshot.frame_blob_cache_misses,
            frame_blobs_created: snapshot.frame_blobs_created,
            frame_shape_cache_hits: snapshot.frame_shape_cache_hits,
            frame_shape_cache_misses: snapshot.frame_shape_cache_misses,
            frame_shapes_created: snapshot.frame_shapes_created,
            mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.mask_atlas),
            color_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.color_atlas),
            subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.subpixel_atlas),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererGlyphAtlasPerfSnapshotV1 {
    pub width: u32,
    pub height: u32,
    pub pages: u32,
    pub entries: u64,

    pub used_px: u64,
    pub capacity_px: u64,

    pub frame_hits: u64,
    pub frame_misses: u64,
    pub frame_inserts: u64,
    pub frame_evict_glyphs: u64,
    pub frame_evict_pages: u64,
    pub frame_out_of_space: u64,
    pub frame_too_large: u64,

    pub frame_pending_uploads: u64,
    pub frame_pending_upload_bytes: u64,
    pub frame_upload_bytes: u64,
}

impl UiRendererGlyphAtlasPerfSnapshotV1 {
    fn from_core(snapshot: fret_core::RendererGlyphAtlasPerfSnapshot) -> Self {
        Self {
            width: snapshot.width,
            height: snapshot.height,
            pages: snapshot.pages,
            entries: snapshot.entries,
            used_px: snapshot.used_px,
            capacity_px: snapshot.capacity_px,
            frame_hits: snapshot.frame_hits,
            frame_misses: snapshot.frame_misses,
            frame_inserts: snapshot.frame_inserts,
            frame_evict_glyphs: snapshot.frame_evict_glyphs,
            frame_evict_pages: snapshot.frame_evict_pages,
            frame_out_of_space: snapshot.frame_out_of_space,
            frame_too_large: snapshot.frame_too_large,
            frame_pending_uploads: snapshot.frame_pending_uploads,
            frame_pending_upload_bytes: snapshot.frame_pending_upload_bytes,
            frame_upload_bytes: snapshot.frame_upload_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheStatsV1 {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub stored_tiles: u64,
    pub recorded_ops: u64,
    pub replayed_ops: u64,
    pub clear_calls: u64,
    pub prune_calls: u64,
    pub evict_calls: u64,
    pub evict_prune_age: u64,
    pub evict_prune_budget: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheSnapshotV1 {
    pub entries: usize,
    #[serde(default)]
    pub requested_tiles: usize,
    #[serde(default)]
    pub budget_limit: u32,
    #[serde(default)]
    pub budget_used: u32,
    #[serde(default)]
    pub skipped_tiles: u32,
    pub stats: UiSceneOpTileCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiWorkBudgetSnapshotV1 {
    pub requested_units: u32,
    pub limit: u32,
    pub used: u32,
    pub skipped_units: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCanvasCacheEntryV1 {
    pub node: u64,
    pub name: String,
    #[serde(default)]
    pub path: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub svg: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub text: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub scene_op_tiles: Option<UiSceneOpTileCacheSnapshotV1>,
    #[serde(default)]
    pub work_budget: Option<UiWorkBudgetSnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCacheKindSnapshotV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}

#[cfg(feature = "preload-icon-svgs")]
fn icon_svg_cache_stats(app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    let stats = app.global::<fret_ui_kit::declarative::icon::IconSvgPreloadDiagnostics>()?;
    let entries = stats.entries;
    let bytes_ready = stats.bytes_ready;
    let register_calls = stats.register_calls;
    Some(UiRetainedSvgCacheStatsV1 {
        entries,
        bytes_ready,
        stats: UiCacheStatsV1 {
            prepare_calls: register_calls,
            ..Default::default()
        },
    })
}

#[cfg(not(feature = "preload-icon-svgs"))]
fn icon_svg_cache_stats(_app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    None
}

fn canvas_cache_stats_for_window(app: &App, window: u64) -> Vec<UiCanvasCacheEntryV1> {
    let Some(registry) = app.global::<fret_canvas::diagnostics::CanvasCacheStatsRegistry>() else {
        return Vec::new();
    };

    registry
        .iter()
        .filter_map(|(key, snap)| {
            ((key.window == window) || (key.window == 0)).then_some((key, snap))
        })
        .map(|(key, snap)| UiCanvasCacheEntryV1 {
            node: key.node,
            name: key.name.to_string(),
            path: snap.path.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            svg: snap.svg.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            text: snap.text.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            scene_op_tiles: snap.scene_op_tiles.map(|s| UiSceneOpTileCacheSnapshotV1 {
                entries: s.entries,
                requested_tiles: s.requested_tiles,
                budget_limit: s.budget_limit,
                budget_used: s.budget_used,
                skipped_tiles: s.skipped_tiles,
                stats: UiSceneOpTileCacheStatsV1 {
                    calls: s.stats.calls,
                    hits: s.stats.hits,
                    misses: s.stats.misses,
                    stored_tiles: s.stats.stored_tiles,
                    recorded_ops: s.stats.recorded_ops,
                    replayed_ops: s.stats.replayed_ops,
                    clear_calls: s.stats.clear_calls,
                    prune_calls: s.stats.prune_calls,
                    evict_calls: s.stats.evict_calls,
                    evict_prune_age: s.stats.evict_prune_age,
                    evict_prune_budget: s.stats.evict_prune_budget,
                },
            }),
            work_budget: snap.work_budget.map(|b| UiWorkBudgetSnapshotV1 {
                requested_units: b.requested_units,
                limit: b.limit,
                used: b.used,
                skipped_units: b.skipped_units,
            }),
        })
        .collect()
}

#[cfg(any())]
mod legacy_forked_script_protocol {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiActionScriptV1 {
        pub schema_version: u32,
        pub steps: Vec<UiActionStepV1>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum UiActionStepV1 {
        Click {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
        },
        ResetDiagnostics,
        MovePointer {
            target: UiSelectorV1,
        },
        DragPointer {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
        },
        Wheel {
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default)]
            delta_y: f32,
        },
        PressKey {
            key: String,
            #[serde(default)]
            modifiers: UiKeyModifiersV1,
            #[serde(default)]
            repeat: bool,
        },
        TypeText {
            text: String,
        },
        WaitFrames {
            n: u32,
        },
        WaitUntil {
            predicate: UiPredicateV1,
            timeout_frames: u32,
        },
        Assert {
            predicate: UiPredicateV1,
        },
        CaptureBundle {
            label: Option<String>,
        },
        CaptureScreenshot {
            label: Option<String>,
            #[serde(default = "default_capture_screenshot_timeout_frames")]
            timeout_frames: u32,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiActionScriptV2 {
        pub schema_version: u32,
        pub steps: Vec<UiActionStepV2>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum UiActionStepV2 {
        // v1-compatible steps
        Click {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            modifiers: Option<UiKeyModifiersV1>,
        },
        ResetDiagnostics,
        MovePointer {
            target: UiSelectorV1,
        },
        DragPointer {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
        },
        /// Move the pointer along a straight line over multiple frames (one move event per frame).
        ///
        /// Prefer this over `drag_pointer` when measuring hit-test/dispatch time, because
        /// `drag_pointer` emits multiple pointer move events in a single frame.
        MovePointerSweep {
            target: UiSelectorV1,
            delta_x: f32,
            delta_y: f32,
            #[serde(default = "default_drag_steps")]
            steps: u32,
            #[serde(default = "default_move_frames_per_step")]
            frames_per_step: u32,
        },
        Wheel {
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default)]
            delta_y: f32,
        },
        PressKey {
            key: String,
            #[serde(default)]
            modifiers: UiKeyModifiersV1,
            #[serde(default)]
            repeat: bool,
        },
        TypeText {
            text: String,
        },
        WaitFrames {
            n: u32,
        },
        WaitUntil {
            predicate: UiPredicateV1,
            timeout_frames: u32,
        },
        Assert {
            predicate: UiPredicateV1,
        },
        CaptureBundle {
            label: Option<String>,
        },
        CaptureScreenshot {
            label: Option<String>,
            #[serde(default = "default_capture_screenshot_timeout_frames")]
            timeout_frames: u32,
        },

        // v2 intent-level steps
        /// Click a target only after its bounds have remained stable for `stable_frames`.
        ///
        /// This is useful for virtualized lists where a target's measured bounds can jump
        /// across frames (e.g. estimate -> measured), causing clicks to land at stale
        /// positions when using a single-frame snapshot.
        ClickStable {
            target: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_click_count")]
            click_count: u8,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            modifiers: Option<UiKeyModifiersV1>,
            #[serde(default = "default_click_stable_frames")]
            stable_frames: u32,
            #[serde(default = "default_click_stable_max_move_px")]
            max_move_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        EnsureVisible {
            target: UiSelectorV1,
            #[serde(default)]
            within_window: bool,
            #[serde(default)]
            padding_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        ScrollIntoView {
            container: UiSelectorV1,
            target: UiSelectorV1,
            #[serde(default)]
            delta_x: f32,
            #[serde(default = "default_scroll_delta_y")]
            delta_y: f32,
            #[serde(default)]
            require_fully_within_window: bool,
            #[serde(default)]
            padding_px: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        TypeTextInto {
            target: UiSelectorV1,
            text: String,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        MenuSelect {
            menu: UiSelectorV1,
            item: UiSelectorV1,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        DragTo {
            from: UiSelectorV1,
            to: UiSelectorV1,
            #[serde(default)]
            button: UiMouseButtonV1,
            #[serde(default = "default_drag_steps")]
            steps: u32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
        },
        SetSliderValue {
            target: UiSelectorV1,
            value: f32,
            #[serde(default = "default_slider_min")]
            min: f32,
            #[serde(default = "default_slider_max")]
            max: f32,
            #[serde(default = "default_slider_epsilon")]
            epsilon: f32,
            #[serde(default = "default_action_timeout_frames")]
            timeout_frames: u32,
            #[serde(default = "default_drag_steps")]
            drag_steps: u32,
        },
        /// Request a resize of the active window's inner size (logical px).
        ///
        /// This is intended for deterministic “resize stress” repro scripts and is best-effort:
        /// runners may ignore it on platforms where programmatic resizing is not supported.
        SetWindowInnerSize {
            width_px: f32,
            height_px: f32,
        },
    }

    impl From<UiActionStepV1> for UiActionStepV2 {
        fn from(value: UiActionStepV1) -> Self {
            match value {
                UiActionStepV1::Click {
                    target,
                    button,
                    click_count,
                } => Self::Click {
                    target,
                    button,
                    click_count,
                    modifiers: None,
                },
                UiActionStepV1::ResetDiagnostics => Self::ResetDiagnostics,
                UiActionStepV1::MovePointer { target } => Self::MovePointer { target },
                UiActionStepV1::DragPointer {
                    target,
                    button,
                    delta_x,
                    delta_y,
                    steps,
                } => Self::DragPointer {
                    target,
                    button,
                    delta_x,
                    delta_y,
                    steps,
                },
                UiActionStepV1::Wheel {
                    target,
                    delta_x,
                    delta_y,
                } => Self::Wheel {
                    target,
                    delta_x,
                    delta_y,
                },
                UiActionStepV1::PressKey {
                    key,
                    modifiers,
                    repeat,
                } => Self::PressKey {
                    key,
                    modifiers,
                    repeat,
                },
                UiActionStepV1::TypeText { text } => Self::TypeText { text },
                UiActionStepV1::WaitFrames { n } => Self::WaitFrames { n },
                UiActionStepV1::WaitUntil {
                    predicate,
                    timeout_frames,
                } => Self::WaitUntil {
                    predicate,
                    timeout_frames,
                },
                UiActionStepV1::Assert { predicate } => Self::Assert { predicate },
                UiActionStepV1::CaptureBundle { label } => Self::CaptureBundle { label },
                UiActionStepV1::CaptureScreenshot {
                    label,
                    timeout_frames,
                } => Self::CaptureScreenshot {
                    label,
                    timeout_frames,
                },
            }
        }
    }

    fn default_drag_steps() -> u32 {
        8
    }

    fn default_click_count() -> u8 {
        1
    }

    fn default_click_stable_frames() -> u32 {
        2
    }

    fn default_click_stable_max_move_px() -> f32 {
        1.0
    }

    fn default_move_frames_per_step() -> u32 {
        1
    }

    fn default_capture_screenshot_timeout_frames() -> u32 {
        300
    }

    fn default_action_timeout_frames() -> u32 {
        180
    }

    fn default_scroll_delta_y() -> f32 {
        -120.0
    }

    fn default_slider_min() -> f32 {
        0.0
    }

    fn default_slider_max() -> f32 {
        100.0
    }

    fn default_slider_epsilon() -> f32 {
        0.5
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UiMouseButtonV1 {
        Left,
        Right,
        Middle,
    }

    impl Default for UiMouseButtonV1 {
        fn default() -> Self {
            Self::Left
        }
    }

    impl UiMouseButtonV1 {
        fn from_button(button: fret_core::MouseButton) -> Self {
            match button {
                fret_core::MouseButton::Left => Self::Left,
                fret_core::MouseButton::Right => Self::Right,
                fret_core::MouseButton::Middle => Self::Middle,
                fret_core::MouseButton::Back
                | fret_core::MouseButton::Forward
                | fret_core::MouseButton::Other(_) => Self::Left,
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
    pub struct UiKeyModifiersV1 {
        #[serde(default)]
        pub shift: bool,
        #[serde(default)]
        pub ctrl: bool,
        #[serde(default)]
        pub alt: bool,
        #[serde(default)]
        pub meta: bool,
    }

    impl UiKeyModifiersV1 {
        fn from_modifiers(modifiers: fret_core::Modifiers) -> Self {
            Self {
                shift: modifiers.shift,
                ctrl: modifiers.ctrl,
                alt: modifiers.alt,
                meta: modifiers.meta,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum UiPredicateV1 {
        Exists {
            target: UiSelectorV1,
        },
        NotExists {
            target: UiSelectorV1,
        },
        FocusIs {
            target: UiSelectorV1,
        },
        /// Matches the current modal/pointer barrier root and focus barrier root (if any).
        ///
        /// This is intentionally coarse-grained: scripts should be able to assert that close
        /// transitions keep the pointer barrier active while releasing focus containment (or vice
        /// versa) without needing stable node ids.
        BarrierRoots {
            #[serde(default)]
            barrier_root: UiOptionalRootStateV1,
            #[serde(default)]
            focus_barrier_root: UiOptionalRootStateV1,
            /// When set, additionally enforces whether the two roots are equal.
            ///
            /// - `true`: requires `barrier_root == focus_barrier_root` (both `None`, or the same id).
            /// - `false`: requires `barrier_root != focus_barrier_root`.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            require_equal: Option<bool>,
        },
        /// True when the target exists and its semantics bounds intersect the active window bounds.
        ///
        /// This is useful for scroll-driven scenarios: it prevents scripts from “finding” an element
        /// that exists in the tree but is currently far off-screen due to an in-flight scroll/window
        /// update.
        VisibleInWindow {
            target: UiSelectorV1,
        },
        /// True when the target exists and its semantics bounds are fully contained within the active
        /// window bounds (optionally padded inward by `padding_px`).
        BoundsWithinWindow {
            target: UiSelectorV1,
            #[serde(default)]
            padding_px: f32,
            /// A small tolerance to account for subpixel rounding (e.g. 1 physical px at non-1.0 DPI).
            ///
            /// This does not replace `padding_px` (which shrinks the allowed region); it only relaxes
            /// strict edge containment checks by `eps_px`.
            #[serde(default)]
            eps_px: f32,
        },
        /// True when the target exists and its semantics bounds are at least the specified size.
        ///
        /// This is useful for demos where the content can legitimately be taller than the window
        /// (scrollable pages), but we still want to gate against "collapsed to ~0" layout regressions.
        BoundsMinSize {
            target: UiSelectorV1,
            #[serde(default)]
            min_w_px: f32,
            #[serde(default)]
            min_h_px: f32,
            /// A small tolerance to account for rounding / fractional layout units.
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds do not overlap.
        ///
        /// Use `eps_px` to tolerate tiny intersections caused by subpixel rounding (e.g. at 125% DPI).
        BoundsNonOverlapping {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap.
        ///
        /// Use `eps_px` to require at least `eps_px` overlap in both dimensions (helps tolerate
        /// subpixel rounding at fractional DPI).
        BoundsOverlapping {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap on the X axis.
        ///
        /// This is useful when two elements are intentionally vertically offset (e.g. a slider thumb
        /// and track), but we still want to assert horizontal alignment.
        BoundsOverlappingX {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
        /// True when both targets exist and their semantics bounds overlap on the Y axis.
        BoundsOverlappingY {
            a: UiSelectorV1,
            b: UiSelectorV1,
            #[serde(default)]
            eps_px: f32,
        },
    }

    #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UiOptionalRootStateV1 {
        /// Do not assert anything about the root (accept both `Some` and `None`).
        #[default]
        Any,
        None,
        Some,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum UiSelectorV1 {
        RoleAndName {
            role: String,
            name: String,
        },
        RoleAndPath {
            role: String,
            name: String,
            /// Ancestors ordered from outermost -> innermost.
            ancestors: Vec<UiRoleAndNameV1>,
        },
        TestId {
            id: String,
        },
        GlobalElementId {
            element: u64,
        },
        NodeId {
            node: u64,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UiRoleAndNameV1 {
        pub role: String,
        pub name: String,
    }
}

#[derive(Debug, Default)]
pub struct UiScriptFrameOutput {
    pub events: Vec<Event>,
    pub effects: Vec<Effect>,
    pub request_redraw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPickResultV1 {
    pub schema_version: u32,
    pub run_id: u64,
    pub updated_unix_ms: u64,
    pub window: Option<u64>,
    pub stage: UiPickStageV1,
    pub position: Option<PointV1>,
    pub selection: Option<UiPickSelectionV1>,
    pub reason: Option<String>,
    pub last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPickStageV1 {
    Picked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPickSelectionV1 {
    pub node: UiSemanticsNodeV1,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub selectors: Vec<UiSelectorV1>,
}

impl UiPickSelectionV1 {
    fn from_node(
        snapshot: &fret_core::SemanticsSnapshot,
        node: &fret_core::SemanticsNode,
        element: Option<u64>,
        element_path: Option<String>,
        cfg: &UiDiagnosticsConfig,
    ) -> Self {
        let exported =
            UiSemanticsNodeV1::from_node(node, cfg.redact_text, cfg.max_debug_string_bytes);
        let selectors = suggest_selectors(snapshot, node, &exported, element, cfg);
        Self {
            node: exported,
            element,
            element_path,
            selectors,
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveScript {
    steps: Vec<UiActionStepV2>,
    run_id: u64,
    next_step: usize,
    wait_frames_remaining: u32,
    wait_until: Option<WaitUntilState>,
    screenshot_wait: Option<ScreenshotWaitState>,
    v2_step_state: Option<V2StepState>,
    last_reported_step: Option<usize>,
    selector_resolution_trace: Vec<UiSelectorResolutionTraceEntryV1>,
    hit_test_trace: Vec<UiHitTestTraceEntryV1>,
}

#[derive(Debug, Clone)]
struct PendingScript {
    steps: Vec<UiActionStepV2>,
}

impl PendingScript {
    #[cfg(feature = "diagnostics-ws")]
    fn from_json_value(value: serde_json::Value) -> Option<Self> {
        let schema_version: u32 = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32;

        match schema_version {
            1 => serde_json::from_value::<UiActionScriptV1>(value)
                .ok()
                .and_then(Self::from_v1),
            2 => serde_json::from_value::<UiActionScriptV2>(value)
                .ok()
                .and_then(Self::from_v2),
            _ => None,
        }
    }

    fn from_v1(script: UiActionScriptV1) -> Option<Self> {
        if script.schema_version != 1 {
            return None;
        }
        Some(Self {
            steps: script.steps.into_iter().map(UiActionStepV2::from).collect(),
        })
    }

    fn from_v2(script: UiActionScriptV2) -> Option<Self> {
        if script.schema_version != 2 {
            return None;
        }
        Some(Self {
            steps: script.steps,
        })
    }
}

#[derive(Debug, Clone)]
enum V2StepState {
    ClickStable(V2ClickStableState),
    EnsureVisible(V2EnsureVisibleState),
    ScrollIntoView(V2ScrollIntoViewState),
    TypeTextInto(V2TypeTextIntoState),
    MenuSelect(V2MenuSelectState),
    MenuSelectPath(V2MenuSelectPathState),
    DragPointer(V2DragPointerState),
    DragTo(V2DragToState),
    SetSliderValue(V2SetSliderValueState),
    MovePointerSweep(V2MovePointerSweepState),
}

#[derive(Debug, Clone)]
struct V2ClickStableState {
    step_index: usize,
    remaining_frames: u32,
    stable_count: u32,
    last_center: Option<Point>,
}

#[derive(Debug, Clone)]
struct V2EnsureVisibleState {
    step_index: usize,
    remaining_frames: u32,
}

#[derive(Debug, Clone)]
struct V2ScrollIntoViewState {
    step_index: usize,
    remaining_frames: u32,
}

#[derive(Debug, Clone)]
struct V2TypeTextIntoState {
    step_index: usize,
    remaining_frames: u32,
    phase: u32,
}

#[derive(Debug, Clone)]
struct V2MenuSelectState {
    step_index: usize,
    remaining_frames: u32,
    phase: u32,
}

#[derive(Debug, Clone)]
struct V2MenuSelectPathState {
    step_index: usize,
    remaining_frames: u32,
    phase: u32,
    next_index: usize,
}

#[derive(Debug, Clone)]
struct V2DragPointerState {
    step_index: usize,
    /// Total move segments (not counting the initial `move+down` frame and the final `up` frame).
    steps: u32,
    button: UiMouseButtonV1,
    start: Point,
    end: Point,
    /// Playback cursor:
    /// - `0`: emit `move+down` at `start`
    /// - `1..=steps`: emit a pressed `move` (and `InternalDrag::Over`) at interpolated positions
    /// - `steps + 1`: emit `up` at `end` (and `InternalDrag::Drop`)
    frame: u32,
}

#[derive(Debug, Clone)]
struct V2DragToState {
    step_index: usize,
    remaining_frames: u32,
    playback: Option<V2DragPointerState>,
}

#[derive(Debug, Clone)]
struct V2SetSliderValueState {
    step_index: usize,
    remaining_frames: u32,
    phase: u32,
    last_drag_x: Option<f32>,
}

#[derive(Debug, Clone)]
struct V2MovePointerSweepState {
    step_index: usize,
    start: Point,
    end: Point,
    steps: u32,
    next_step: u32,
    frames_per_step: u32,
    wait_frames_remaining: u32,
}

#[derive(Debug, Clone)]
struct WaitUntilState {
    step_index: usize,
    remaining_frames: u32,
}

#[derive(Debug, Clone)]
struct ScreenshotWaitState {
    step_index: usize,
    remaining_frames: u32,
    request_id: String,
    window_ffi: u64,
    last_result_trigger_stamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInputArbitrationSnapshotV1 {
    #[serde(default)]
    pub modal_barrier_root: Option<u64>,
    #[serde(default)]
    pub focus_barrier_root: Option<u64>,
    #[serde(default)]
    pub pointer_occlusion: String,
    #[serde(default)]
    pub pointer_occlusion_layer_id: Option<u64>,
    #[serde(default)]
    pub pointer_capture_active: bool,
    #[serde(default)]
    pub pointer_capture_layer_id: Option<u64>,
    #[serde(default)]
    pub pointer_capture_multiple_layers: bool,
}

impl Default for UiInputArbitrationSnapshotV1 {
    fn default() -> Self {
        Self {
            modal_barrier_root: None,
            focus_barrier_root: None,
            pointer_occlusion: "none".to_string(),
            pointer_occlusion_layer_id: None,
            pointer_capture_active: false,
            pointer_capture_layer_id: None,
            pointer_capture_multiple_layers: false,
        }
    }
}

impl UiInputArbitrationSnapshotV1 {
    fn from_snapshot(snapshot: fret_ui::tree::UiInputArbitrationSnapshot) -> Self {
        Self {
            modal_barrier_root: snapshot.modal_barrier_root.map(key_to_u64),
            focus_barrier_root: snapshot.focus_barrier_root.map(key_to_u64),
            pointer_occlusion: pointer_occlusion_label(snapshot.pointer_occlusion),
            pointer_occlusion_layer_id: snapshot
                .pointer_occlusion_layer
                .map(|id| id.data().as_ffi()),
            pointer_capture_active: snapshot.pointer_capture_active,
            pointer_capture_layer_id: snapshot.pointer_capture_layer.map(|id| id.data().as_ffi()),
            pointer_capture_multiple_layers: snapshot.pointer_capture_multiple_layers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTreeDebugSnapshotV1 {
    pub stats: UiFrameStatsV1,
    #[serde(default)]
    pub invalidation_walks: Vec<UiInvalidationWalkV1>,
    #[serde(default)]
    pub hover_declarative_invalidation_hotspots: Vec<UiHoverDeclarativeInvalidationHotspotV1>,
    #[serde(default)]
    pub dirty_views: Vec<UiDirtyViewV1>,
    #[serde(default)]
    pub notify_requests: Vec<UiNotifyRequestV1>,
    #[serde(default)]
    pub virtual_list_windows: Vec<UiVirtualListWindowV1>,
    #[serde(default)]
    pub virtual_list_window_shift_samples: Vec<UiVirtualListWindowShiftSampleV1>,
    #[serde(default)]
    pub windowed_rows_surfaces: Vec<UiWindowedRowsSurfaceWindowV1>,
    #[serde(default)]
    pub retained_virtual_list_reconciles: Vec<UiRetainedVirtualListReconcileV1>,
    #[serde(default)]
    pub scroll_handle_changes: Vec<UiScrollHandleChangeV1>,
    #[serde(default)]
    pub prepaint_actions: Vec<UiPrepaintActionV1>,
    #[serde(default)]
    pub model_change_hotspots: Vec<UiModelChangeHotspotV1>,
    #[serde(default)]
    pub model_change_unobserved: Vec<UiModelChangeUnobservedV1>,
    #[serde(default)]
    pub global_change_hotspots: Vec<UiGlobalChangeHotspotV1>,
    #[serde(default)]
    pub global_change_unobserved: Vec<UiGlobalChangeUnobservedV1>,
    #[serde(default)]
    pub cache_roots: Vec<UiCacheRootStatsV1>,
    #[serde(default)]
    pub overlay_synthesis: Vec<UiOverlaySynthesisEventV1>,
    /// Viewport input forwarding events observed during the current frame.
    ///
    /// This records `Effect::ViewportInput` deliveries (ADR 0147) so scripted diagnostics can
    /// gate on “viewport tooling input was actually exercised” without scraping logs.
    #[serde(default)]
    pub viewport_input: Vec<UiViewportInputEventV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_ime_bridge: Option<UiWebImeBridgeDebugSnapshotV1>,
    /// Docking interaction ownership snapshot (best-effort).
    ///
    /// This is sourced from a frame-local diagnostics store populated by policy-heavy ecosystem
    /// crates (e.g. docking), and is intended for debugging arbitration regressions without logs.
    #[serde(default)]
    pub docking_interaction: Option<UiDockingInteractionSnapshotV1>,
    #[serde(default)]
    pub removed_subtrees: Vec<UiRemovedSubtreeV1>,
    #[serde(default)]
    pub layout_engine_solves: Vec<UiLayoutEngineSolveV1>,
    #[serde(default)]
    pub layout_hotspots: Vec<UiLayoutHotspotV1>,
    #[serde(default)]
    pub widget_measure_hotspots: Vec<UiWidgetMeasureHotspotV1>,
    #[serde(default)]
    pub paint_widget_hotspots: Vec<UiPaintWidgetHotspotV1>,
    #[serde(default)]
    pub paint_text_prepare_hotspots: Vec<UiPaintTextPrepareHotspotV1>,
    #[serde(default)]
    pub input_arbitration: UiInputArbitrationSnapshotV1,
    /// Best-effort command gating decisions for a small set of "interesting" commands.
    ///
    /// This is intended for debugging cross-surface inconsistencies (menus vs palette vs buttons)
    /// without relying on ad-hoc logs.
    #[serde(default)]
    pub command_gating_trace: Vec<UiCommandGatingTraceEntryV1>,
    pub layers_in_paint_order: Vec<UiLayerInfoV1>,
    #[serde(default)]
    pub all_layer_roots: Vec<u64>,
    #[serde(default)]
    pub layer_visible_writes: Vec<UiLayerVisibleWriteV1>,
    #[serde(default)]
    pub overlay_policy_decisions: Vec<UiOverlayPolicyDecisionV1>,
    pub hit_test: Option<UiHitTestSnapshotV1>,
    pub element_runtime: Option<ElementDiagnosticsSnapshotV1>,
    pub semantics: Option<UiSemanticsSnapshotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWebImeBridgeDebugSnapshotV1 {
    pub enabled: bool,
    pub composing: bool,
    pub suppress_next_input: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_has_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_element_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_pixel_ratio: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_value_chars: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_start_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_end_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_height_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_height_px: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_beforeinput_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_data: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_key_code: Option<KeyCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_area: Option<RectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_anchor_px: Option<(f32, f32)>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_cursor_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_text: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_events: Vec<String>,

    pub beforeinput_seen: u64,
    pub input_seen: u64,
    pub suppressed_input_seen: u64,
    pub composition_start_seen: u64,
    pub composition_update_seen: u64,
    pub composition_end_seen: u64,
    pub cursor_area_set_seen: u64,
}

impl UiWebImeBridgeDebugSnapshotV1 {
    fn from_snapshot(snapshot: &fret_core::input::WebImeBridgeDebugSnapshot) -> Self {
        Self {
            enabled: snapshot.enabled,
            composing: snapshot.composing,
            suppress_next_input: snapshot.suppress_next_input,
            textarea_has_focus: snapshot.textarea_has_focus,
            active_element_tag: snapshot.active_element_tag.clone(),
            position_mode: snapshot.position_mode.clone(),
            mount_kind: snapshot.mount_kind.clone(),
            device_pixel_ratio: snapshot.device_pixel_ratio,
            textarea_value_chars: snapshot.textarea_value_chars,
            textarea_selection_start_utf16: snapshot.textarea_selection_start_utf16,
            textarea_selection_end_utf16: snapshot.textarea_selection_end_utf16,
            textarea_client_width_px: snapshot.textarea_client_width_px,
            textarea_client_height_px: snapshot.textarea_client_height_px,
            textarea_scroll_width_px: snapshot.textarea_scroll_width_px,
            textarea_scroll_height_px: snapshot.textarea_scroll_height_px,
            last_input_type: snapshot.last_input_type.clone(),
            last_beforeinput_data: snapshot.last_beforeinput_data.clone(),
            last_input_data: snapshot.last_input_data.clone(),
            last_key_code: snapshot.last_key_code,
            last_cursor_area: snapshot.last_cursor_area.map(RectV1::from),
            last_cursor_anchor_px: snapshot.last_cursor_anchor_px,
            last_preedit_text: snapshot.last_preedit_text.clone(),
            last_preedit_cursor_utf16: snapshot.last_preedit_cursor_utf16,
            last_commit_text: snapshot.last_commit_text.clone(),
            recent_events: snapshot.recent_events.clone(),
            beforeinput_seen: snapshot.beforeinput_seen,
            input_seen: snapshot.input_seen,
            suppressed_input_seen: snapshot.suppressed_input_seen,
            composition_start_seen: snapshot.composition_start_seen,
            composition_update_seen: snapshot.composition_update_seen,
            composition_end_seen: snapshot.composition_end_seen,
            cursor_area_set_seen: snapshot.cursor_area_set_seen,
        }
    }
}

impl UiTreeDebugSnapshotV1 {
    fn from_tree(
        app: &App,
        window: AppWindowId,
        ui: &UiTree<App>,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
        element_runtime_state: Option<&ElementRuntime>,
        hit_test: Option<UiHitTestSnapshotV1>,
        element_runtime_snapshot: Option<ElementDiagnosticsSnapshotV1>,
        semantics: Option<UiSemanticsSnapshotV1>,
        max_gating_trace_entries: usize,
    ) -> Self {
        let contained_relayout_roots: HashSet<fret_core::NodeId> = ui
            .debug_view_cache_contained_relayout_roots()
            .iter()
            .copied()
            .collect();
        Self {
            stats: UiFrameStatsV1::from_stats(ui.debug_stats(), renderer_perf),
            invalidation_walks: ui
                .debug_invalidation_walks()
                .iter()
                .map(UiInvalidationWalkV1::from_walk)
                .collect(),
            hover_declarative_invalidation_hotspots: ui
                .debug_hover_declarative_invalidation_hotspots(20)
                .into_iter()
                .map(UiHoverDeclarativeInvalidationHotspotV1::from_hotspot)
                .collect(),
            dirty_views: ui
                .debug_dirty_views()
                .iter()
                .map(UiDirtyViewV1::from_dirty_view)
                .collect(),
            notify_requests: ui
                .debug_notify_requests()
                .iter()
                .map(UiNotifyRequestV1::from_notify_request)
                .collect(),
            virtual_list_windows: ui
                .debug_virtual_list_windows()
                .iter()
                .map(UiVirtualListWindowV1::from_window)
                .collect(),
            virtual_list_window_shift_samples: ui
                .debug_virtual_list_window_shift_samples()
                .iter()
                .map(UiVirtualListWindowShiftSampleV1::from_sample)
                .collect(),
            windowed_rows_surfaces: app
                .global::<fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceDiagnosticsStore>(
                )
                .and_then(|store| store.windows_for_window(window, app.frame_id()))
                .map(|windows| {
                    windows
                        .iter()
                        .map(UiWindowedRowsSurfaceWindowV1::from_telemetry)
                        .collect()
                })
                .unwrap_or_default(),
            retained_virtual_list_reconciles: ui
                .debug_retained_virtual_list_reconciles()
                .iter()
                .map(UiRetainedVirtualListReconcileV1::from_record)
                .collect(),
            scroll_handle_changes: ui
                .debug_scroll_handle_changes()
                .iter()
                .map(UiScrollHandleChangeV1::from_change)
                .collect(),
            prepaint_actions: ui
                .debug_prepaint_actions()
                .iter()
                .map(UiPrepaintActionV1::from_action)
                .collect(),
            model_change_hotspots: ui
                .debug_model_change_hotspots()
                .iter()
                .map(UiModelChangeHotspotV1::from_hotspot)
                .collect(),
            model_change_unobserved: ui
                .debug_model_change_unobserved()
                .iter()
                .map(UiModelChangeUnobservedV1::from_unobserved)
                .collect(),
            global_change_hotspots: ui
                .debug_global_change_hotspots()
                .iter()
                .map(|h| UiGlobalChangeHotspotV1::from_hotspot(app, h))
                .collect(),
            global_change_unobserved: ui
                .debug_global_change_unobserved()
                .iter()
                .map(|u| UiGlobalChangeUnobservedV1::from_unobserved(app, u))
                .collect(),
            cache_roots: ui
                .debug_cache_root_stats()
                .iter()
                .map(|stats| {
                    UiCacheRootStatsV1::from_stats(
                        window,
                        ui,
                        element_runtime_state,
                        semantics.as_ref(),
                        &contained_relayout_roots,
                        stats,
                    )
                })
                .collect(),
            overlay_synthesis: app
                .global::<fret_ui_kit::WindowOverlaySynthesisDiagnosticsStore>()
                .and_then(|diag| diag.events_for_window(window, app.frame_id()))
                .map(|events| {
                    events
                        .iter()
                        .copied()
                        .map(UiOverlaySynthesisEventV1::from_event)
                        .collect()
            })
                .unwrap_or_default(),
            viewport_input: Vec::new(),
            web_ime_bridge: app
                .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                .filter(|snapshot| **snapshot != fret_core::input::WebImeBridgeDebugSnapshot::default())
                .map(UiWebImeBridgeDebugSnapshotV1::from_snapshot),
            docking_interaction: app
                .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
                .and_then(|store| store.docking_for_window(window, app.frame_id()))
                .map(UiDockingInteractionSnapshotV1::from_snapshot),
            removed_subtrees: ui
                .debug_removed_subtrees()
                .iter()
                .map(|r| UiRemovedSubtreeV1::from_record(window, ui, element_runtime_state, r))
                .collect(),
            layout_engine_solves: ui
                .debug_layout_engine_solves()
                .iter()
                .map(UiLayoutEngineSolveV1::from_solve)
                .collect(),
            layout_hotspots: ui
                .debug_layout_hotspots()
                .iter()
                .map(UiLayoutHotspotV1::from_hotspot)
                .collect(),
            widget_measure_hotspots: ui
                .debug_widget_measure_hotspots()
                .iter()
                .map(UiWidgetMeasureHotspotV1::from_hotspot)
                .collect(),
            paint_widget_hotspots: ui
                .debug_paint_widget_hotspots()
                .iter()
                .map(UiPaintWidgetHotspotV1::from_hotspot)
                .collect(),
            paint_text_prepare_hotspots: ui
                .debug_paint_text_prepare_hotspots()
                .iter()
                .map(UiPaintTextPrepareHotspotV1::from_hotspot)
                .collect(),
            input_arbitration: UiInputArbitrationSnapshotV1::from_snapshot(
                ui.input_arbitration_snapshot(),
            ),
            command_gating_trace: command_gating_trace_for_window(
                app,
                window,
                max_gating_trace_entries,
            ),
            layers_in_paint_order: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(UiLayerInfoV1::from_layer)
                .collect(),
            all_layer_roots: ui
                .debug_layers_in_paint_order()
                .into_iter()
                .map(|l| l.root.data().as_ffi())
                .collect(),
            layer_visible_writes: ui
                .debug_layer_visible_writes()
                .iter()
                .map(UiLayerVisibleWriteV1::from_write)
                .collect(),
            overlay_policy_decisions: ui
                .debug_overlay_policy_decisions()
                .iter()
                .map(UiOverlayPolicyDecisionV1::from_decision)
                .collect(),
            hit_test,
            element_runtime: element_runtime_snapshot,
            semantics,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDockingInteractionSnapshotV1 {
    #[serde(default)]
    pub dock_drag: Option<UiDockDragDiagnosticsV1>,
    #[serde(default)]
    pub viewport_capture: Option<UiViewportCaptureDiagnosticsV1>,
}

impl UiDockingInteractionSnapshotV1 {
    fn from_snapshot(snapshot: &fret_runtime::DockingInteractionDiagnostics) -> Self {
        Self {
            dock_drag: snapshot
                .dock_drag
                .map(UiDockDragDiagnosticsV1::from_snapshot),
            viewport_capture: snapshot
                .viewport_capture
                .map(UiViewportCaptureDiagnosticsV1::from_snapshot),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiDockDragDiagnosticsV1 {
    pub pointer_id: u64,
    pub source_window: u64,
    pub current_window: u64,
    pub dragging: bool,
    pub cross_window_hover: bool,
}

impl UiDockDragDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::DockDragDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            source_window: snapshot.source_window.data().as_ffi(),
            current_window: snapshot.current_window.data().as_ffi(),
            dragging: snapshot.dragging,
            cross_window_hover: snapshot.cross_window_hover,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiViewportCaptureDiagnosticsV1 {
    pub pointer_id: u64,
    pub target: u64,
}

impl UiViewportCaptureDiagnosticsV1 {
    fn from_snapshot(snapshot: fret_runtime::ViewportCaptureDiagnostics) -> Self {
        Self {
            pointer_id: snapshot.pointer_id.0,
            target: snapshot.target.data().as_ffi(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiViewportInputEventV1 {
    pub target: u64,
    pub pointer_id: u64,
    pub pointer_type: String,
    pub cursor_px: PointV1,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: UiViewportInputKindV1,
}

impl UiViewportInputEventV1 {
    fn from_event(event: fret_core::ViewportInputEvent) -> Self {
        Self {
            target: event.target.data().as_ffi(),
            pointer_id: event.pointer_id.0 as u64,
            pointer_type: viewport_pointer_type_label(event.pointer_type).to_string(),
            cursor_px: PointV1::from(event.cursor_px),
            uv: event.uv,
            target_px: event.target_px,
            kind: UiViewportInputKindV1::from_kind(event.kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiViewportInputKindV1 {
    PointerMove {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
    },
    PointerDown {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        click_count: u8,
    },
    PointerUp {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        is_click: bool,
        click_count: u8,
    },
    PointerCancel {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
        reason: String,
    },
    Wheel {
        delta: PointV1,
        modifiers: UiKeyModifiersV1,
    },
}

impl UiViewportInputKindV1 {
    fn from_kind(kind: fret_core::ViewportInputKind) -> Self {
        match kind {
            fret_core::ViewportInputKind::PointerMove { buttons, modifiers } => Self::PointerMove {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
            fret_core::ViewportInputKind::PointerDown {
                button,
                modifiers,
                click_count,
            } => Self::PointerDown {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                click_count,
            },
            fret_core::ViewportInputKind::PointerUp {
                button,
                modifiers,
                is_click,
                click_count,
            } => Self::PointerUp {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                is_click,
                click_count,
            },
            fret_core::ViewportInputKind::PointerCancel {
                buttons,
                modifiers,
                reason,
            } => Self::PointerCancel {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                reason: viewport_cancel_reason_label(reason).to_string(),
            },
            fret_core::ViewportInputKind::Wheel { delta, modifiers } => Self::Wheel {
                delta: PointV1::from(delta),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiMouseButtonsV1 {
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub middle: bool,
}

impl UiMouseButtonsV1 {
    fn from_buttons(buttons: fret_core::MouseButtons) -> Self {
        Self {
            left: buttons.left,
            right: buttons.right,
            middle: buttons.middle,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAxisV1 {
    Horizontal,
    Vertical,
}

impl UiAxisV1 {
    fn from_axis(axis: fret_core::Axis) -> Self {
        match axis {
            fret_core::Axis::Horizontal => Self::Horizontal,
            fret_core::Axis::Vertical => Self::Vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisKindV1 {
    Modal,
    Popover,
    Hover,
    Tooltip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListMeasureModeV1 {
    Fixed,
    Measured,
    Known,
}

impl UiVirtualListMeasureModeV1 {
    fn from_mode(mode: fret_ui::element::VirtualListMeasureMode) -> Self {
        match mode {
            fret_ui::element::VirtualListMeasureMode::Fixed => Self::Fixed,
            fret_ui::element::VirtualListMeasureMode::Measured => Self::Measured,
            fret_ui::element::VirtualListMeasureMode::Known => Self::Known,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftKindV1 {
    None,
    Prefetch,
    Escape,
}

impl Default for UiVirtualListWindowShiftKindV1 {
    fn default() -> Self {
        Self::None
    }
}

impl UiVirtualListWindowShiftKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugVirtualListWindowShiftKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::None => Self::None,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Prefetch => Self::Prefetch,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Escape => Self::Escape,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftReasonV1 {
    ScrollOffset,
    ViewportResize,
    ItemsRevision,
    ScrollToItem,
    InputsChange,
    Unknown,
}

impl UiVirtualListWindowShiftReasonV1 {
    fn from_reason(reason: fret_ui::tree::UiDebugVirtualListWindowShiftReason) -> Self {
        match reason {
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset => Self::ScrollOffset,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => {
                Self::ViewportResize
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => {
                Self::ItemsRevision
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => Self::ScrollToItem,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::InputsChange => Self::InputsChange,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftApplyModeV1 {
    RetainedReconcile,
    NonRetainedRerender,
}

impl UiVirtualListWindowShiftApplyModeV1 {
    fn from_mode(mode: fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode) -> Self {
        match mode {
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile => {
                Self::RetainedReconcile
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender => {
                Self::NonRetainedRerender
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiVirtualRangeV1 {
    pub start_index: u64,
    pub end_index: u64,
    pub overscan: u64,
    pub count: u64,
}

impl UiVirtualRangeV1 {
    fn from_range(range: fret_ui::virtual_list::VirtualRange) -> Self {
        Self {
            start_index: range.start_index as u64,
            end_index: range.end_index as u64,
            overscan: range.overscan as u64,
            count: range.count as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowV1 {
    pub node: u64,
    pub element: u64,
    #[serde(default)]
    pub source: UiVirtualListWindowSourceV1,
    pub axis: UiAxisV1,
    #[serde(default)]
    pub is_probe_layout: bool,
    pub items_len: u64,
    pub items_revision: u64,
    pub prev_items_revision: u64,
    pub measure_mode: UiVirtualListMeasureModeV1,
    pub overscan: u64,
    #[serde(default)]
    pub policy_key: u64,
    #[serde(default)]
    pub inputs_key: u64,
    pub viewport: f32,
    pub prev_viewport: f32,
    pub offset: f32,
    pub prev_offset: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
    #[serde(default)]
    pub deferred_scroll_to_item: bool,
    #[serde(default)]
    pub deferred_scroll_consumed: bool,
    #[serde(default)]
    pub window_mismatch: bool,
    #[serde(default)]
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_reason: Option<UiVirtualListWindowShiftReasonV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_apply_mode: Option<UiVirtualListWindowShiftApplyModeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
}

impl UiVirtualListWindowV1 {
    fn from_window(window: &fret_ui::tree::UiDebugVirtualListWindow) -> Self {
        Self {
            node: key_to_u64(window.node),
            element: window.element.0,
            source: UiVirtualListWindowSourceV1::from_source(window.source),
            axis: UiAxisV1::from_axis(window.axis),
            is_probe_layout: window.is_probe_layout,
            items_len: window.items_len as u64,
            items_revision: window.items_revision,
            prev_items_revision: window.prev_items_revision,
            measure_mode: UiVirtualListMeasureModeV1::from_mode(window.measure_mode),
            overscan: window.overscan as u64,
            policy_key: window.policy_key,
            inputs_key: window.inputs_key,
            viewport: window.viewport.0,
            prev_viewport: window.prev_viewport.0,
            offset: window.offset.0,
            prev_offset: window.prev_offset.0,
            window_range: window.window_range.map(UiVirtualRangeV1::from_range),
            prev_window_range: window.prev_window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: window.render_window_range.map(UiVirtualRangeV1::from_range),
            deferred_scroll_to_item: window.deferred_scroll_to_item,
            deferred_scroll_consumed: window.deferred_scroll_consumed,
            window_mismatch: window.window_mismatch,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(window.window_shift_kind),
            window_shift_reason: window
                .window_shift_reason
                .map(UiVirtualListWindowShiftReasonV1::from_reason),
            window_shift_apply_mode: window
                .window_shift_apply_mode
                .map(UiVirtualListWindowShiftApplyModeV1::from_mode),
            window_shift_invalidation_detail: window
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowShiftSampleV1 {
    pub frame_id: u64,
    pub source: UiVirtualListWindowSourceV1,
    pub node: u64,
    pub element: u64,
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    pub window_shift_reason: UiVirtualListWindowShiftReasonV1,
    pub window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
}

impl UiVirtualListWindowShiftSampleV1 {
    fn from_sample(sample: &fret_ui::tree::UiDebugVirtualListWindowShiftSample) -> Self {
        Self {
            frame_id: sample.frame_id.0,
            source: UiVirtualListWindowSourceV1::from_source(sample.source),
            node: key_to_u64(sample.node),
            element: sample.element.0,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(sample.window_shift_kind),
            window_shift_reason: UiVirtualListWindowShiftReasonV1::from_reason(
                sample.window_shift_reason,
            ),
            window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1::from_mode(
                sample.window_shift_apply_mode,
            ),
            window_shift_invalidation_detail: sample
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
            prev_window_range: sample.prev_window_range.map(UiVirtualRangeV1::from_range),
            window_range: sample.window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: sample.render_window_range.map(UiVirtualRangeV1::from_range),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWindowedRowsSurfaceWindowV1 {
    pub callsite_id: u64,
    pub location: UiSourceLocationV1,

    pub len: u64,
    pub row_height: f32,
    pub overscan: u64,
    pub gap: f32,
    pub scroll_margin: f32,

    pub viewport_height: f32,
    pub offset_y: f32,
    pub content_height: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_end: Option<u64>,
    pub visible_count: u64,
}

impl UiWindowedRowsSurfaceWindowV1 {
    fn from_telemetry(
        telemetry: &fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceWindowTelemetry,
    ) -> Self {
        Self {
            callsite_id: telemetry.callsite_id,
            location: UiSourceLocationV1 {
                file: telemetry.file.to_string(),
                line: telemetry.line,
                column: telemetry.column,
            },
            len: telemetry.len,
            row_height: telemetry.row_height.0,
            overscan: telemetry.overscan,
            gap: telemetry.gap.0,
            scroll_margin: telemetry.scroll_margin.0,
            viewport_height: telemetry.viewport_height.0,
            offset_y: telemetry.offset_y.0,
            content_height: telemetry.content_height.0,
            visible_start: telemetry.visible_start,
            visible_end: telemetry.visible_end,
            visible_count: telemetry.visible_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRetainedVirtualListReconcileV1 {
    pub node: u64,
    pub element: u64,
    pub prev_items: u64,
    pub next_items: u64,
    pub preserved_items: u64,
    pub attached_items: u64,
    pub detached_items: u64,
    #[serde(default)]
    pub reused_from_keep_alive_items: u64,
    #[serde(default)]
    pub kept_alive_items: u64,
    #[serde(default)]
    pub evicted_keep_alive_items: u64,
    #[serde(default)]
    pub keep_alive_pool_len_before: u64,
    #[serde(default)]
    pub keep_alive_pool_len_after: u64,
}

impl UiRetainedVirtualListReconcileV1 {
    fn from_record(record: &fret_ui::tree::UiDebugRetainedVirtualListReconcile) -> Self {
        Self {
            node: key_to_u64(record.node),
            element: record.element.0,
            prev_items: record.prev_items as u64,
            next_items: record.next_items as u64,
            preserved_items: record.preserved_items as u64,
            attached_items: record.attached_items as u64,
            detached_items: record.detached_items as u64,
            reused_from_keep_alive_items: record.reused_from_keep_alive_items as u64,
            kept_alive_items: record.kept_alive_items as u64,
            evicted_keep_alive_items: record.evicted_keep_alive_items as u64,
            keep_alive_pool_len_before: record.keep_alive_pool_len_before as u64,
            keep_alive_pool_len_after: record.keep_alive_pool_len_after as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowSourceV1 {
    Prepaint,
    #[serde(other)]
    Layout,
}

impl Default for UiVirtualListWindowSourceV1 {
    fn default() -> Self {
        Self::Layout
    }
}

impl UiVirtualListWindowSourceV1 {
    fn from_source(source: fret_ui::tree::UiDebugVirtualListWindowSource) -> Self {
        match source {
            fret_ui::tree::UiDebugVirtualListWindowSource::Layout => Self::Layout,
            fret_ui::tree::UiDebugVirtualListWindowSource::Prepaint => Self::Prepaint,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisSourceV1 {
    CachedDeclaration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScrollHandleChangeKindV1 {
    Layout,
    HitTestOnly,
}

impl UiScrollHandleChangeKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugScrollHandleChangeKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugScrollHandleChangeKind::Layout => Self::Layout,
            fret_ui::tree::UiDebugScrollHandleChangeKind::HitTestOnly => Self::HitTestOnly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollHandleChangeV1 {
    pub handle_key: u64,
    pub kind: UiScrollHandleChangeKindV1,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<u64>,
    pub offset_x: f32,
    pub offset_y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_offset_x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_offset_y: Option<f32>,
    pub viewport_w: f32,
    pub viewport_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_viewport_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_viewport_h: Option<f32>,
    pub content_w: f32,
    pub content_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_content_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_content_h: Option<f32>,
    #[serde(default)]
    pub offset_changed: bool,
    #[serde(default)]
    pub viewport_changed: bool,
    #[serde(default)]
    pub content_changed: bool,
    #[serde(default)]
    pub bound_elements: u32,
    #[serde(default)]
    pub bound_nodes_sample: Vec<u64>,
    #[serde(default)]
    pub upgraded_to_layout_bindings: u32,
}

impl UiScrollHandleChangeV1 {
    fn from_change(change: &fret_ui::tree::UiDebugScrollHandleChange) -> Self {
        Self {
            handle_key: change.handle_key as u64,
            kind: UiScrollHandleChangeKindV1::from_kind(change.kind),
            revision: change.revision,
            prev_revision: change.prev_revision,
            offset_x: change.offset.x.0,
            offset_y: change.offset.y.0,
            prev_offset_x: change.prev_offset.map(|p| p.x.0),
            prev_offset_y: change.prev_offset.map(|p| p.y.0),
            viewport_w: change.viewport.width.0,
            viewport_h: change.viewport.height.0,
            prev_viewport_w: change.prev_viewport.map(|s| s.width.0),
            prev_viewport_h: change.prev_viewport.map(|s| s.height.0),
            content_w: change.content.width.0,
            content_h: change.content.height.0,
            prev_content_w: change.prev_content.map(|s| s.width.0),
            prev_content_h: change.prev_content.map(|s| s.height.0),
            offset_changed: change.offset_changed,
            viewport_changed: change.viewport_changed,
            content_changed: change.content_changed,
            bound_elements: change.bound_elements,
            bound_nodes_sample: change
                .bound_nodes_sample
                .iter()
                .copied()
                .map(key_to_u64)
                .collect(),
            upgraded_to_layout_bindings: change.upgraded_to_layout_bindings,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPrepaintActionKindV1 {
    Invalidate,
    RequestRedraw,
    RequestAnimationFrame,
    VirtualListWindowShift,
    ChartSamplingWindowShift,
    NodeGraphCullWindowShift,
}

impl UiPrepaintActionKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugPrepaintActionKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugPrepaintActionKind::Invalidate => Self::Invalidate,
            fret_ui::tree::UiDebugPrepaintActionKind::RequestRedraw => Self::RequestRedraw,
            fret_ui::tree::UiDebugPrepaintActionKind::RequestAnimationFrame => {
                Self::RequestAnimationFrame
            }
            fret_ui::tree::UiDebugPrepaintActionKind::VirtualListWindowShift => {
                Self::VirtualListWindowShift
            }
            fret_ui::tree::UiDebugPrepaintActionKind::ChartSamplingWindowShift => {
                Self::ChartSamplingWindowShift
            }
            fret_ui::tree::UiDebugPrepaintActionKind::NodeGraphCullWindowShift => {
                Self::NodeGraphCullWindowShift
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrepaintActionV1 {
    pub node: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node: Option<u64>,
    pub kind: UiPrepaintActionKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_list_window_shift_kind: Option<UiVirtualListWindowShiftKindV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_list_window_shift_reason: Option<UiVirtualListWindowShiftReasonV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart_sampling_window_key: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_graph_cull_window_key: Option<u64>,
    #[serde(default)]
    pub frame_id: u64,
}

impl UiPrepaintActionV1 {
    fn from_action(action: &fret_ui::tree::UiDebugPrepaintAction) -> Self {
        let invalidation = action.invalidation.map(|inv| match inv {
            fret_ui::Invalidation::Layout => "layout",
            fret_ui::Invalidation::Paint => "paint",
            fret_ui::Invalidation::HitTest => "hit_test",
            fret_ui::Invalidation::HitTestOnly => "hit_test_only",
        });

        Self {
            node: key_to_u64(action.node),
            target_node: action.target.map(key_to_u64),
            kind: UiPrepaintActionKindV1::from_kind(action.kind),
            invalidation: invalidation.map(|s| s.to_string()),
            element: action.element.map(|id| id.0),
            virtual_list_window_shift_kind: action
                .virtual_list_window_shift_kind
                .map(UiVirtualListWindowShiftKindV1::from_kind),
            virtual_list_window_shift_reason: action
                .virtual_list_window_shift_reason
                .map(UiVirtualListWindowShiftReasonV1::from_reason),
            chart_sampling_window_key: action.chart_sampling_window_key,
            node_graph_cull_window_key: action.node_graph_cull_window_key,
            frame_id: action.frame_id.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisOutcomeV1 {
    Synthesized,
    SuppressedMissingTrigger,
    SuppressedTriggerNotLiveInCurrentFrame,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlaySynthesisEventV1 {
    pub kind: UiOverlaySynthesisKindV1,
    pub id: u64,
    pub source: UiOverlaySynthesisSourceV1,
    pub outcome: UiOverlaySynthesisOutcomeV1,
}

impl UiOverlaySynthesisEventV1 {
    fn from_event(e: fret_ui_kit::OverlaySynthesisEvent) -> Self {
        use fret_ui_kit::OverlaySynthesisKind;
        use fret_ui_kit::OverlaySynthesisOutcome;
        use fret_ui_kit::OverlaySynthesisSource;

        let kind = match e.kind {
            OverlaySynthesisKind::Modal => UiOverlaySynthesisKindV1::Modal,
            OverlaySynthesisKind::Popover => UiOverlaySynthesisKindV1::Popover,
            OverlaySynthesisKind::Hover => UiOverlaySynthesisKindV1::Hover,
            OverlaySynthesisKind::Tooltip => UiOverlaySynthesisKindV1::Tooltip,
        };
        let source = match e.source {
            OverlaySynthesisSource::CachedDeclaration => {
                UiOverlaySynthesisSourceV1::CachedDeclaration
            }
        };
        let outcome = match e.outcome {
            OverlaySynthesisOutcome::Synthesized => UiOverlaySynthesisOutcomeV1::Synthesized,
            OverlaySynthesisOutcome::SuppressedMissingTrigger => {
                UiOverlaySynthesisOutcomeV1::SuppressedMissingTrigger
            }
            OverlaySynthesisOutcome::SuppressedTriggerNotLiveInCurrentFrame => {
                UiOverlaySynthesisOutcomeV1::SuppressedTriggerNotLiveInCurrentFrame
            }
        };

        Self {
            kind,
            id: e.id.0,
            source,
            outcome,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandGatingTraceEntryV1 {
    pub command: String,
    pub enabled: bool,
    pub reason: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_path: Option<String>,
    /// Structured explanation of why the command is disabled (multiple blockers may apply).
    #[serde(default)]
    pub blocked_by: Vec<String>,
    /// Best-effort detail fields to make debugging inconsistent gating easier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_override: Option<bool>,
    #[serde(default)]
    pub command_registered: bool,
}

#[derive(Debug, Clone)]
struct UiCommandGatingTraceCandidate {
    command: fret_runtime::CommandId,
    source: &'static str,
    menu_path: Option<String>,
    menu_when: Option<fret_runtime::WhenExpr>,
}

fn command_gating_trace_for_window(
    app: &App,
    window: AppWindowId,
    max_entries: usize,
) -> Vec<UiCommandGatingTraceEntryV1> {
    let gating = fret_runtime::best_effort_snapshot_for_window(app, window);

    let mut candidates: Vec<UiCommandGatingTraceCandidate> = Vec::new();

    // 1) Explicit gating inputs (useful for verifying that snapshots are being published).
    for (cmd, _) in gating.enabled_overrides() {
        candidates.push(UiCommandGatingTraceCandidate {
            command: cmd.clone(),
            source: "enabled_overrides",
            menu_path: None,
            menu_when: None,
        });
    }
    if let Some(map) = gating.action_availability() {
        for (cmd, _) in map {
            candidates.push(UiCommandGatingTraceCandidate {
                command: cmd.clone(),
                source: "action_availability",
                menu_path: None,
                menu_when: None,
            });
        }
    }

    // 2) Effective OS menubar model (data-only). This is the closest source of truth for
    // "visible menu commands" from the app's perspective.
    if let Some(menu_bar) = fret_app::effective_menu_bar(app) {
        collect_menu_bar_commands(&menu_bar, &mut candidates);
    }

    // 3) Command palette catalog (best-effort). This approximates the set of entries derived from
    // host commands; the actual palette filters further by query/group options.
    for (id, meta) in app.commands().iter() {
        if meta.hidden {
            continue;
        }
        candidates.push(UiCommandGatingTraceCandidate {
            command: id.clone(),
            source: "command_palette_catalog",
            menu_path: None,
            menu_when: None,
        });
    }

    // Always include a core, cross-surface set even if the host didn't publish any snapshot yet.
    for &cmd in &[
        "edit.undo",
        "edit.redo",
        "edit.copy",
        "edit.cut",
        "edit.paste",
        "edit.select_all",
        "focus.menu_bar",
    ] {
        candidates.push(UiCommandGatingTraceCandidate {
            command: fret_runtime::CommandId::from(cmd),
            source: "core",
            menu_path: None,
            menu_when: None,
        });
    }

    // Deduplicate by (command, source, menu_path) so repeated insertions don't explode snapshots.
    let mut seen: HashSet<(String, &'static str, Option<String>)> = HashSet::new();
    candidates.retain(|c| {
        let key = (
            c.command.as_str().to_string(),
            c.source,
            c.menu_path.clone(),
        );
        if seen.contains(&key) {
            return false;
        }
        seen.insert(key);
        true
    });

    candidates.sort_by(|a, b| {
        a.source
            .cmp(b.source)
            .then_with(|| a.menu_path.cmp(&b.menu_path))
            .then_with(|| a.command.as_str().cmp(b.command.as_str()))
    });

    let max_entries = max_entries.min(2000);
    candidates
        .into_iter()
        .take(max_entries)
        .map(|c| {
            let decision =
                command_gating_decision_trace(app, &gating, &c.command, c.menu_when.as_ref());

            UiCommandGatingTraceEntryV1 {
                command: c.command.as_str().to_string(),
                enabled: decision.enabled,
                reason: decision.reason,
                scope: decision.scope,
                source: c.source.to_string(),
                menu_path: c.menu_path,
                blocked_by: decision.blocked_by,
                action_available: decision.action_available,
                command_when: decision.command_when,
                menu_when: decision.menu_when,
                enabled_override: decision.enabled_override,
                command_registered: decision.command_registered,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct UiCommandGatingDecisionTrace {
    enabled: bool,
    reason: String,
    scope: String,
    blocked_by: Vec<String>,
    action_available: Option<bool>,
    command_when: Option<bool>,
    menu_when: Option<bool>,
    enabled_override: Option<bool>,
    command_registered: bool,
}

fn command_gating_decision_trace(
    app: &App,
    gating: &fret_runtime::WindowCommandGatingSnapshot,
    command: &fret_runtime::CommandId,
    menu_when: Option<&fret_runtime::WhenExpr>,
) -> UiCommandGatingDecisionTrace {
    let meta = app.commands().get(command.clone());
    let scope = meta
        .map(|m| format!("{:?}", m.scope))
        .unwrap_or_else(|| "Unknown".to_string());

    let mut blocked_by: Vec<String> = Vec::new();

    let action_available = if let Some(meta) = meta
        && meta.scope == fret_runtime::CommandScope::Widget
        && let Some(map) = gating.action_availability()
        && let Some(is_available) = map.get(command).copied()
    {
        Some(is_available)
    } else {
        None
    };
    if action_available == Some(false) {
        blocked_by.push("action_availability".to_string());
    }

    let command_when = meta.and_then(|m| m.when.as_ref().map(|w| w.eval(gating.input_ctx())));
    if command_when == Some(false) {
        blocked_by.push("when".to_string());
    }

    let enabled_override = gating.enabled_overrides().get(command).copied();
    if enabled_override == Some(false) {
        blocked_by.push("enabled_override".to_string());
    }

    let menu_when = menu_when.map(|w| w.eval(gating.input_ctx()));
    if menu_when == Some(false) {
        blocked_by.push("menu_when".to_string());
    }

    let command_registered = meta.is_some();
    let enabled = blocked_by.is_empty();

    // Keep a stable "primary reason" string for backwards compatibility / easy grepping.
    let reason = if blocked_by.iter().any(|b| b == "action_availability") {
        "action_unavailable"
    } else if blocked_by.iter().any(|b| b == "when") {
        "when_false"
    } else if blocked_by.iter().any(|b| b == "enabled_override") {
        "disabled_override"
    } else if blocked_by.iter().any(|b| b == "menu_when") {
        "menu_when_false"
    } else if !command_registered {
        "unknown_command"
    } else {
        "enabled"
    }
    .to_string();

    UiCommandGatingDecisionTrace {
        enabled,
        reason,
        scope,
        blocked_by,
        action_available,
        command_when,
        menu_when,
        enabled_override,
        command_registered,
    }
}

fn collect_menu_bar_commands(
    menu_bar: &fret_runtime::MenuBar,
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for menu in &menu_bar.menus {
        let menu_title = menu.title.as_ref().to_string();
        collect_menu_items(&menu_title, &menu.items, out);
    }
}

fn collect_menu_items(
    prefix: &str,
    items: &[fret_runtime::MenuItem],
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for item in items {
        match item {
            fret_runtime::MenuItem::Command { command, when, .. } => {
                out.push(UiCommandGatingTraceCandidate {
                    command: command.clone(),
                    source: "menu_bar",
                    menu_path: Some(prefix.to_string()),
                    menu_when: when.clone(),
                });
            }
            fret_runtime::MenuItem::Label { .. } => {}
            fret_runtime::MenuItem::Separator | fret_runtime::MenuItem::SystemMenu { .. } => {}
            fret_runtime::MenuItem::Submenu {
                title,
                when: _,
                items,
            } => {
                let next = format!("{prefix} > {}", title.as_ref());
                collect_menu_items(&next, items, out);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHoverDeclarativeInvalidationHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub hit_test: u32,
    #[serde(default)]
    pub layout: u32,
    #[serde(default)]
    pub paint: u32,
}

impl UiHoverDeclarativeInvalidationHotspotV1 {
    fn from_hotspot(h: fret_ui::tree::UiDebugHoverDeclarativeInvalidationHotspot) -> Self {
        Self {
            node: key_to_u64(h.node),
            element: h.element.map(|e| e.0),
            hit_test: h.hit_test,
            layout: h.layout,
            paint: h.paint,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRemovedSubtreeV1 {
    pub root: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_element_path: Option<String>,
    #[serde(default)]
    pub root_parent: Option<u64>,
    #[serde(default)]
    pub root_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_is_view_cache_reuse_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default)]
    pub root_layer: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_layer_visible: Option<bool>,
    #[serde(default)]
    pub reachable_from_layer_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable_from_view_cache_roots: Option<bool>,
    #[serde(default)]
    pub unreachable_from_liveness_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_layer_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_root_nodes_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_listed_under_reuse_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root_path: Option<String>,
    #[serde(default)]
    pub root_children_len: u32,
    #[serde(default)]
    pub root_parent_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_instance_present: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path: Vec<u64>,
    #[serde(default)]
    pub root_path_truncated: bool,
    /// For each `root_path` edge (`child -> parent`), whether `UiTree` currently has the
    /// corresponding `parent.children` edge:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing node entry)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_ui_contains_child: Vec<u8>,
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_frame_contains_child: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub removed_nodes: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_tail: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl UiRemovedSubtreeV1 {
    fn from_record(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime_state: Option<&ElementRuntime>,
        r: &fret_ui::tree::UiDebugRemoveSubtreeRecord,
    ) -> Self {
        let outcome = match r.outcome {
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::SkippedLayerRoot => "skipped_layer_root",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::RootMissing => "root_missing",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::Removed => "removed",
        };

        let root_element_path = r.root_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });

        let root_parent_element_path = r.root_parent_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });

        let trigger_element_path = r.trigger_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });

        let trigger_element_root_path = r.trigger_element_root.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });

        let root_path = r.root_path[..(r.root_path_len as usize).min(r.root_path.len())].to_vec();
        let root_path_edge_len = (r.root_path_edge_len as usize)
            .min(r.root_path_edge_ui_contains_child.len())
            .min(r.root_path_edge_frame_contains_child.len());
        let root_path_edge_ui_contains_child =
            r.root_path_edge_ui_contains_child[..root_path_edge_len].to_vec();
        let root_path_edge_frame_contains_child =
            r.root_path_edge_frame_contains_child[..root_path_edge_len].to_vec();

        let (
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
        ) = r
            .root_parent
            .and_then(|parent| ui.debug_set_children_write_for(parent))
            .map(|w| {
                (
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.old_len),
                    Some(w.new_len),
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((None, None, None, None));

        let (
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
        ) = r
            .root_root
            .and_then(|root| ui.debug_parent_sever_write_for(root))
            .map(|w| {
                let parent_element = element_runtime_state
                    .and_then(|runtime| runtime.element_for_node(window, w.parent));
                let parent_path = parent_element.and_then(|element| {
                    element_runtime_state
                        .and_then(|runtime| runtime.debug_path_for_element(window, element))
                });
                let parent_is_view_cache_reuse_root = parent_element.and_then(|element| {
                    element_runtime_state.and_then(|runtime| {
                        runtime
                            .diagnostics_snapshot(window)
                            .map(|s| s.view_cache_reuse_roots.contains(&element))
                    })
                });

                let mut old_elements_head: Vec<u64> = Vec::new();
                let mut old_elements_head_paths: Vec<String> = Vec::new();
                let mut new_elements_head: Vec<u64> = Vec::new();
                let mut new_elements_head_paths: Vec<String> = Vec::new();

                if let Some(write) = ui.debug_set_children_write_for(w.parent) {
                    for element in write.old_elements_head.into_iter().flatten() {
                        old_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            old_elements_head_paths.push(path);
                        }
                    }
                    for element in write.new_elements_head.into_iter().flatten() {
                        new_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            new_elements_head_paths.push(path);
                        }
                    }
                }

                (
                    Some(key_to_u64(w.parent)),
                    parent_element.map(|e| e.0),
                    parent_path,
                    parent_is_view_cache_reuse_root,
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.frame_id.0),
                    old_elements_head,
                    old_elements_head_paths,
                    new_elements_head,
                    new_elements_head_paths,
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));

        Self {
            root: key_to_u64(r.root),
            root_element: r.root_element.map(|e| e.0),
            root_parent_element: r.root_parent_element.map(|e| e.0),
            root_parent_element_path,
            root_element_path,
            root_parent: r.root_parent.map(key_to_u64),
            root_root: r.root_root.map(key_to_u64),
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
            root_layer: r.root_layer.map(|id| id.data().as_ffi()),
            root_layer_visible: r.root_layer_visible,
            reachable_from_layer_roots: r.reachable_from_layer_roots,
            reachable_from_view_cache_roots: r.reachable_from_view_cache_roots,
            unreachable_from_liveness_roots: r.unreachable_from_liveness_roots,
            liveness_layer_roots_len: r.liveness_layer_roots_len,
            view_cache_reuse_roots_len: r.view_cache_reuse_roots_len,
            view_cache_reuse_root_nodes_len: r.view_cache_reuse_root_nodes_len,
            trigger_element: r.trigger_element.map(|e| e.0),
            trigger_element_root: r.trigger_element_root.map(|e| e.0),
            trigger_element_in_view_cache_keep_alive: r.trigger_element_in_view_cache_keep_alive,
            trigger_element_listed_under_reuse_root: r
                .trigger_element_listed_under_reuse_root
                .map(|id| id.0),
            trigger_element_path,
            trigger_element_root_path,
            root_children_len: r.root_children_len,
            root_parent_children_len: r.root_parent_children_len,
            root_parent_children_contains_root: r.root_parent_children_contains_root,
            root_parent_frame_children_len: r.root_parent_frame_children_len,
            root_parent_frame_children_contains_root: r.root_parent_frame_children_contains_root,
            root_frame_instance_present: r.root_frame_instance_present,
            root_frame_children_len: r.root_frame_children_len,
            root_path,
            root_path_truncated: r.root_path_truncated,
            root_path_edge_ui_contains_child,
            root_path_edge_frame_contains_child,
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
            removed_nodes: r.removed_nodes,
            removed_head: r.removed_head[..(r.removed_head_len as usize).min(r.removed_head.len())]
                .to_vec(),
            removed_tail: r.removed_tail[..(r.removed_tail_len as usize).min(r.removed_tail.len())]
                .to_vec(),
            outcome: Some(outcome.to_string()),
            frame_id: Some(r.frame_id.0),
            location: Some(format!("{}:{}:{}", r.file, r.line, r.column)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDirtyViewV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

impl UiDirtyViewV1 {
    fn from_dirty_view(dirty: &fret_ui::tree::UiDebugDirtyView) -> Self {
        let source = match dirty.source {
            fret_ui::tree::UiDebugInvalidationSource::ModelChange => "model_change",
            fret_ui::tree::UiDebugInvalidationSource::GlobalChange => "global_change",
            fret_ui::tree::UiDebugInvalidationSource::Notify => "notify",
            fret_ui::tree::UiDebugInvalidationSource::Hover => "hover",
            fret_ui::tree::UiDebugInvalidationSource::Focus => "focus",
            fret_ui::tree::UiDebugInvalidationSource::Other => "other",
        };

        Self {
            root_node: key_to_u64(dirty.view.0),
            root_element: dirty.element.map(|e| e.0),
            source: Some(source.to_string()),
            detail: dirty.detail.as_str().map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNotifyRequestV1 {
    pub frame_id: u64,
    pub caller_node: u64,
    pub target_view: u64,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl UiNotifyRequestV1 {
    fn from_notify_request(req: &fret_ui::tree::UiDebugNotifyRequest) -> Self {
        Self {
            frame_id: req.frame_id.0,
            caller_node: key_to_u64(req.caller_node),
            target_view: key_to_u64(req.target_view.0),
            file: req.file.to_string(),
            line: req.line,
            column: req.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCacheRootStatsV1 {
    pub root: u64,
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub reused: bool,
    pub contained_layout: bool,
    #[serde(default)]
    pub contained_relayout_in_frame: bool,
    pub paint_replayed_ops: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_child_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes_truncated_at: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_in_semantics: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub reuse_reason: Option<String>,
}

impl UiCacheRootStatsV1 {
    fn from_stats(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        semantics: Option<&UiSemanticsSnapshotV1>,
        contained_relayout_roots: &HashSet<fret_core::NodeId>,
        stats: &fret_ui::tree::UiDebugCacheRootStats,
    ) -> Self {
        let element_path = stats.element.and_then(|id| {
            element_runtime.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });

        let direct_child_nodes = ui.children(stats.root).len().min(u32::MAX as usize) as u32;

        // Keep bundles bounded: cache roots can cover large subtrees in real apps.
        const MAX_SUBTREE_NODES: usize = 50_000;
        let mut subtree_nodes_truncated_at: Option<u32> = None;
        let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
        let mut stack: Vec<fret_core::NodeId> = vec![stats.root];
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_SUBTREE_NODES {
                subtree_nodes_truncated_at = Some(MAX_SUBTREE_NODES as u32);
                break;
            }
            for child in ui.children(node) {
                stack.push(child);
            }
        }

        let root_in_semantics = semantics.map(|snap| {
            let id = stats.root.data().as_ffi();
            snap.nodes.iter().any(|n| n.id == id)
        });
        let contained_relayout_in_frame = contained_relayout_roots.contains(&stats.root);

        let (
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
        ) = ui
            .debug_set_children_write_for(stats.root)
            .map(|w| {
                let old_elements_head: Vec<_> =
                    w.old_elements_head.iter().flatten().copied().collect();
                let new_elements_head: Vec<_> =
                    w.new_elements_head.iter().flatten().copied().collect();

                let old_paths: Vec<String> = old_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();
                let new_paths: Vec<String> = new_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();

                (
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.old_len),
                    Some(w.new_len),
                    old_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    new_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    old_paths,
                    new_paths,
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
            ));
        Self {
            root: stats.root.data().as_ffi(),
            element: stats.element.map(|id| id.0),
            element_path,
            reused: stats.reused,
            contained_layout: stats.contained_layout,
            contained_relayout_in_frame,
            paint_replayed_ops: stats.paint_replayed_ops,
            direct_child_nodes: Some(direct_child_nodes),
            subtree_nodes: Some(seen.len().min(u32::MAX as usize) as u32),
            subtree_nodes_truncated_at,
            root_in_semantics,
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
            reuse_reason: Some(stats.reuse_reason.as_str().to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineSolveV1 {
    pub root_node: u64,
    pub solve_time_us: u64,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    #[serde(default)]
    pub measure_time_us: u64,
    #[serde(default)]
    pub top_measures: Vec<UiLayoutEngineMeasureHotspotV1>,
}

impl UiLayoutEngineSolveV1 {
    fn from_solve(s: &fret_ui::tree::UiDebugLayoutEngineSolve) -> Self {
        Self {
            root_node: s.root.data().as_ffi(),
            solve_time_us: s.solve_time.as_micros().min(u64::MAX as u128) as u64,
            measure_calls: s.measure_calls,
            measure_cache_hits: s.measure_cache_hits,
            measure_time_us: s.measure_time.as_micros().min(u64::MAX as u128) as u64,
            top_measures: s
                .top_measures
                .iter()
                .map(|m| UiLayoutEngineMeasureHotspotV1 {
                    node: m.node.data().as_ffi(),
                    measure_time_us: m.measure_time.as_micros().min(u64::MAX as u128) as u64,
                    calls: m.calls,
                    cache_hits: m.cache_hits,
                    element: m.element.map(|id| id.0),
                    element_kind: m.element_kind.map(|s| s.to_string()),
                    top_children: m
                        .top_children
                        .iter()
                        .map(|c| UiLayoutEngineMeasureChildHotspotV1 {
                            child: c.child.data().as_ffi(),
                            measure_time_us: c.measure_time.as_micros().min(u64::MAX as u128)
                                as u64,
                            calls: c.calls,
                            element: c.element.map(|id| id.0),
                            element_kind: c.element_kind.map(|s| s.to_string()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    pub widget_type: String,
    pub layout_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiLayoutHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugLayoutHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            widget_type: h.widget_type.to_string(),
            layout_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWidgetMeasureHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    pub widget_type: String,
    pub measure_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiWidgetMeasureHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugWidgetMeasureHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            widget_type: h.widget_type.to_string(),
            measure_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintWidgetHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub widget_type: String,
    pub paint_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
    #[serde(default)]
    pub inclusive_scene_ops_delta: u32,
    #[serde(default)]
    pub exclusive_scene_ops_delta: u32,
}

impl UiPaintWidgetHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintWidgetHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            widget_type: h.widget_type.to_string(),
            paint_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_scene_ops_delta: h.inclusive_scene_ops_delta,
            exclusive_scene_ops_delta: h.exclusive_scene_ops_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintTextPrepareHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub prepare_time_us: u64,
    pub text_len: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f32>,
    #[serde(default)]
    pub reasons_mask: u16,
}

impl UiPaintTextPrepareHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintTextPrepareHotspot) -> Self {
        fn wrap_as_str(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_as_str(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: Some(h.element_kind.to_string()),
            prepare_time_us: h.prepare_time.as_micros().min(u64::MAX as u128) as u64,
            text_len: h.text_len,
            max_width: h.constraints.max_width.map(|v| v.0),
            wrap: Some(wrap_as_str(h.constraints.wrap).to_string()),
            overflow: Some(overflow_as_str(h.constraints.overflow).to_string()),
            scale_factor: Some(h.constraints.scale_factor),
            reasons_mask: h.reasons_mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureHotspotV1 {
    pub node: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    pub cache_hits: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    #[serde(default)]
    pub top_children: Vec<UiLayoutEngineMeasureChildHotspotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureChildHotspotV1 {
    pub child: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeHotspotV1 {
    pub model: u64,
    pub observation_edges: u32,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeHotspotV1 {
    fn from_hotspot(hotspot: &fret_ui::tree::UiDebugModelChangeHotspot) -> Self {
        let changed_type = hotspot.changed.map(|c| c.type_name.to_string());
        let changed_at = hotspot.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        Self {
            model: hotspot.model.data().as_ffi(),
            observation_edges: hotspot.observation_edges,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSourceLocationV1 {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeUnobservedV1 {
    pub model: u64,
    pub created_type: Option<String>,
    pub created_at: Option<UiSourceLocationV1>,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeUnobservedV1 {
    fn from_unobserved(unobserved: &fret_ui::tree::UiDebugModelChangeUnobserved) -> Self {
        let created_type = unobserved.created.map(|c| c.type_name.to_string());
        let created_at = unobserved.created.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        let changed_type = unobserved.changed.map(|c| c.type_name.to_string());
        let changed_at = unobserved.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });

        Self {
            model: unobserved.model.data().as_ffi(),
            created_type,
            created_at,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeHotspotV1 {
    pub type_name: String,
    pub observation_edges: u32,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeHotspotV1 {
    fn from_hotspot(app: &App, hotspot: &fret_ui::tree::UiDebugGlobalChangeHotspot) -> Self {
        let type_name = app
            .global_type_name(hotspot.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", hotspot.global));
        let changed_at = app
            .global_changed_at(hotspot.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            observation_edges: hotspot.observation_edges,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeUnobservedV1 {
    pub type_name: String,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeUnobservedV1 {
    fn from_unobserved(
        app: &App,
        unobserved: &fret_ui::tree::UiDebugGlobalChangeUnobserved,
    ) -> Self {
        let type_name = app
            .global_type_name(unobserved.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", unobserved.global));
        let changed_at = app
            .global_changed_at(unobserved.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationKindV1 {
    Paint,
    Layout,
    HitTest,
    HitTestOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationSourceV1 {
    ModelChange,
    GlobalChange,
    Hover,
    Focus,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInvalidationWalkV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    pub kind: UiInvalidationKindV1,
    pub source: UiInvalidationSourceV1,
    #[serde(default)]
    pub detail: Option<String>,
    pub walked_nodes: u32,
    #[serde(default)]
    pub truncated_at: Option<u64>,
}

impl UiInvalidationWalkV1 {
    fn from_walk(walk: &fret_ui::tree::UiDebugInvalidationWalk) -> Self {
        let kind = match walk.inv {
            Invalidation::Paint => UiInvalidationKindV1::Paint,
            Invalidation::Layout => UiInvalidationKindV1::Layout,
            Invalidation::HitTest => UiInvalidationKindV1::HitTest,
            Invalidation::HitTestOnly => UiInvalidationKindV1::HitTestOnly,
        };
        let source = match walk.source {
            fret_ui::tree::UiDebugInvalidationSource::ModelChange => {
                UiInvalidationSourceV1::ModelChange
            }
            fret_ui::tree::UiDebugInvalidationSource::GlobalChange => {
                UiInvalidationSourceV1::GlobalChange
            }
            fret_ui::tree::UiDebugInvalidationSource::Notify => UiInvalidationSourceV1::Other,
            fret_ui::tree::UiDebugInvalidationSource::Hover => UiInvalidationSourceV1::Hover,
            fret_ui::tree::UiDebugInvalidationSource::Focus => UiInvalidationSourceV1::Focus,
            fret_ui::tree::UiDebugInvalidationSource::Other => UiInvalidationSourceV1::Other,
        };
        Self {
            root_node: key_to_u64(walk.root),
            root_element: walk.root_element.map(|e| e.0),
            kind,
            source,
            detail: walk.detail.as_str().map(|s| s.to_string()),
            walked_nodes: walk.walked_nodes,
            truncated_at: walk.truncated_at.map(key_to_u64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsSnapshotV1 {
    pub window: u64,
    pub roots: Vec<UiSemanticsRootV1>,
    pub barrier_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_barrier_root: Option<u64>,
    pub focus: Option<u64>,
    pub captured: Option<u64>,
    pub nodes: Vec<UiSemanticsNodeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsRootV1 {
    pub root: u64,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub z_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsNodeV1 {
    pub id: u64,
    pub parent: Option<u64>,
    pub role: String,
    pub bounds: RectV1,
    pub flags: UiSemanticsFlagsV1,
    pub test_id: Option<String>,
    pub active_descendant: Option<u64>,
    pub pos_in_set: Option<u32>,
    pub set_size: Option<u32>,
    pub label: Option<String>,
    pub value: Option<String>,
    pub text_selection: Option<(u32, u32)>,
    pub text_composition: Option<(u32, u32)>,
    pub actions: UiSemanticsActionsV1,
    pub labelled_by: Vec<u64>,
    pub described_by: Vec<u64>,
    pub controls: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsFlagsV1 {
    pub focused: bool,
    pub captured: bool,
    pub disabled: bool,
    pub selected: bool,
    pub expanded: bool,
    pub checked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSemanticsActionsV1 {
    pub focus: bool,
    pub invoke: bool,
    pub set_value: bool,
    pub set_text_selection: bool,
}

impl UiSemanticsSnapshotV1 {
    fn from_snapshot(
        snapshot: &fret_core::SemanticsSnapshot,
        redact_text: bool,
        max_string_bytes: usize,
    ) -> Self {
        Self {
            window: snapshot.window.data().as_ffi(),
            roots: snapshot
                .roots
                .iter()
                .map(|r| UiSemanticsRootV1 {
                    root: key_to_u64(r.root),
                    visible: r.visible,
                    blocks_underlay_input: r.blocks_underlay_input,
                    hit_testable: r.hit_testable,
                    z_index: r.z_index,
                })
                .collect(),
            barrier_root: snapshot.barrier_root.map(key_to_u64),
            focus_barrier_root: snapshot.focus_barrier_root.map(key_to_u64),
            focus: snapshot.focus.map(key_to_u64),
            captured: snapshot.captured.map(key_to_u64),
            nodes: snapshot
                .nodes
                .iter()
                .map(|n| UiSemanticsNodeV1::from_node(n, redact_text, max_string_bytes))
                .collect(),
        }
    }
}

impl UiSemanticsNodeV1 {
    fn from_node(
        node: &fret_core::SemanticsNode,
        redact_text: bool,
        max_string_bytes: usize,
    ) -> Self {
        let mut label = node
            .label
            .as_deref()
            .map(|s| maybe_redact_string(s, redact_text));
        let mut value = node
            .value
            .as_deref()
            .map(|s| maybe_redact_string(s, redact_text));
        let mut test_id = node.test_id.clone();

        if let Some(s) = &mut label {
            truncate_string_bytes(s, max_string_bytes);
        }
        if let Some(s) = &mut value {
            truncate_string_bytes(s, max_string_bytes);
        }
        if let Some(s) = &mut test_id {
            truncate_string_bytes(s, max_string_bytes);
        }

        Self {
            id: key_to_u64(node.id),
            parent: node.parent.map(key_to_u64),
            role: semantics_role_label(node.role).to_string(),
            bounds: RectV1::from(node.bounds),
            flags: UiSemanticsFlagsV1 {
                focused: node.flags.focused,
                captured: node.flags.captured,
                disabled: node.flags.disabled,
                selected: node.flags.selected,
                expanded: node.flags.expanded,
                checked: node.flags.checked,
            },
            test_id,
            active_descendant: node.active_descendant.map(key_to_u64),
            pos_in_set: node.pos_in_set,
            set_size: node.set_size,
            label,
            value,
            text_selection: node.text_selection,
            text_composition: node.text_composition,
            actions: UiSemanticsActionsV1 {
                focus: node.actions.focus,
                invoke: node.actions.invoke,
                set_value: node.actions.set_value,
                set_text_selection: node.actions.set_text_selection,
            },
            labelled_by: node.labelled_by.iter().copied().map(key_to_u64).collect(),
            described_by: node.described_by.iter().copied().map(key_to_u64).collect(),
            controls: node.controls.iter().copied().map(key_to_u64).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFrameStatsV1 {
    #[serde(default)]
    pub frame_arena_capacity_estimate_bytes: u64,
    #[serde(default)]
    pub frame_arena_grow_events: u32,
    #[serde(default)]
    pub element_children_vec_pool_reuses: u32,
    #[serde(default)]
    pub element_children_vec_pool_misses: u32,
    pub layout_time_us: u64,
    #[serde(default)]
    pub layout_collect_roots_time_us: u64,
    #[serde(default)]
    pub layout_invalidate_scroll_handle_bindings_time_us: u64,
    #[serde(default)]
    pub layout_expand_view_cache_invalidations_time_us: u64,
    #[serde(default)]
    pub layout_request_build_roots_time_us: u64,
    #[serde(default)]
    pub layout_pending_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_repair_view_cache_bounds_time_us: u64,
    #[serde(default)]
    pub layout_contained_view_cache_roots_time_us: u64,
    #[serde(default)]
    pub layout_collapse_layout_observations_time_us: u64,
    #[serde(default)]
    pub layout_prepaint_after_layout_time_us: u64,
    #[serde(default)]
    pub layout_skipped_engine_frame: bool,
    #[serde(default)]
    pub layout_roots_time_us: u64,
    #[serde(default)]
    pub layout_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_view_cache_time_us: u64,
    #[serde(default)]
    pub layout_semantics_refresh_time_us: u64,
    #[serde(default)]
    pub layout_focus_repair_time_us: u64,
    #[serde(default)]
    pub layout_deferred_cleanup_time_us: u64,
    #[serde(default)]
    pub prepaint_time_us: u64,
    pub paint_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_calls: u32,
    #[serde(default)]
    pub paint_cache_key_time_us: u64,
    #[serde(default)]
    pub paint_cache_hit_check_time_us: u64,
    #[serde(default)]
    pub paint_widget_time_us: u64,
    #[serde(default)]
    pub paint_observation_record_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_items: u32,
    #[serde(default)]
    pub paint_host_widget_observed_globals_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_globals_items: u32,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_time_us: u64,
    #[serde(default)]
    pub paint_text_prepare_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_blob_missing: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_scale_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_text_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_rich_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_style_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_wrap_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_overflow_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_width_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_font_stack_changed: u32,
    #[serde(default)]
    pub paint_input_context_time_us: u64,
    #[serde(default)]
    pub paint_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub paint_collect_roots_time_us: u64,
    #[serde(default)]
    pub paint_publish_text_input_snapshot_time_us: u64,
    #[serde(default)]
    pub paint_collapse_observations_time_us: u64,
    #[serde(default)]
    pub dispatch_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_events: u32,
    #[serde(default)]
    pub dispatch_pointer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_events: u32,
    #[serde(default)]
    pub dispatch_timer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_targeted_events: u32,
    #[serde(default)]
    pub dispatch_timer_targeted_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_events: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_layers_visited: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_rebuild_visible_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_loop_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_token: Option<u64>,
    #[serde(default)]
    pub dispatch_timer_slowest_was_broadcast: bool,
    #[serde(default)]
    pub dispatch_other_events: u32,
    #[serde(default)]
    pub dispatch_other_event_time_us: u64,
    #[serde(default)]
    pub hit_test_time_us: u64,
    #[serde(default)]
    pub dispatch_events: u32,
    #[serde(default)]
    pub hit_test_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_disabled: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_misses: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_hits: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_candidate_rejected: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_visited: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_pushed: u32,
    #[serde(default)]
    pub hit_test_path_cache_hits: u32,
    #[serde(default)]
    pub hit_test_path_cache_misses: u32,
    #[serde(default)]
    pub hit_test_cached_path_time_us: u64,
    #[serde(default)]
    pub hit_test_bounds_tree_query_time_us: u64,
    #[serde(default)]
    pub hit_test_candidate_self_only_time_us: u64,
    #[serde(default)]
    pub hit_test_fallback_traversal_time_us: u64,
    #[serde(default)]
    pub dispatch_hover_update_time_us: u64,
    #[serde(default)]
    pub dispatch_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub dispatch_active_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_input_context_time_us: u64,
    #[serde(default)]
    pub dispatch_event_chain_build_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_capture_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_bubble_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_query_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_move_layer_observers_time_us: u64,
    #[serde(default)]
    pub dispatch_synth_hover_observer_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_effect_time_us: u64,
    #[serde(default)]
    pub dispatch_post_dispatch_snapshot_time_us: u64,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    #[serde(default)]
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    #[serde(default)]
    pub paint_cache_replay_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translate_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translated_nodes: u32,
    #[serde(default)]
    pub interaction_cache_hits: u32,
    #[serde(default)]
    pub interaction_cache_misses: u32,
    #[serde(default)]
    pub interaction_cache_replayed_records: u32,
    #[serde(default)]
    pub interaction_records: u32,
    pub layout_engine_solves: u64,
    pub layout_engine_solve_time_us: u64,
    pub layout_engine_widget_fallback_solves: u64,
    #[serde(default)]
    pub layout_fast_path_taken: bool,
    #[serde(default)]
    pub layout_invalidations_count: u32,
    #[serde(default)]
    pub model_change_invalidation_roots: u32,
    #[serde(default)]
    pub model_change_models: u32,
    #[serde(default)]
    pub model_change_observation_edges: u32,
    #[serde(default)]
    pub model_change_unobserved_models: u32,
    #[serde(default)]
    pub global_change_invalidation_roots: u32,
    #[serde(default)]
    pub global_change_globals: u32,
    #[serde(default)]
    pub global_change_observation_edges: u32,
    #[serde(default)]
    pub global_change_unobserved_globals: u32,
    #[serde(default)]
    pub invalidation_walk_nodes: u32,
    #[serde(default)]
    pub invalidation_walk_calls: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_hover: u32,
    #[serde(default)]
    pub invalidation_walk_calls_hover: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_focus: u32,
    #[serde(default)]
    pub invalidation_walk_calls_focus: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_other: u32,
    #[serde(default)]
    pub invalidation_walk_calls_other: u32,
    #[serde(default)]
    pub hover_pressable_target_changes: u32,
    #[serde(default)]
    pub hover_hover_region_target_changes: u32,
    #[serde(default)]
    pub hover_declarative_instance_changes: u32,
    #[serde(default)]
    pub hover_declarative_hit_test_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_layout_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_paint_invalidations: u32,
    #[serde(default)]
    pub view_cache_active: bool,
    #[serde(default)]
    pub view_cache_invalidation_truncations: u32,
    #[serde(default)]
    pub view_cache_contained_relayouts: u32,
    #[serde(default)]
    pub view_cache_roots_total: u32,
    #[serde(default)]
    pub view_cache_roots_reused: u32,
    #[serde(default)]
    pub view_cache_roots_first_mount: u32,
    #[serde(default)]
    pub view_cache_roots_node_recreated: u32,
    #[serde(default)]
    pub view_cache_roots_cache_key_mismatch: u32,
    #[serde(default)]
    pub view_cache_roots_not_marked_reuse_root: u32,
    #[serde(default)]
    pub view_cache_roots_needs_rerender: u32,
    #[serde(default)]
    pub view_cache_roots_layout_invalidated: u32,
    #[serde(default)]
    pub view_cache_roots_manual: u32,
    #[serde(default)]
    pub set_children_barrier_writes: u32,
    #[serde(default)]
    pub barrier_relayouts_scheduled: u32,
    #[serde(default)]
    pub barrier_relayouts_performed: u32,
    #[serde(default)]
    pub virtual_list_visible_range_checks: u32,
    #[serde(default)]
    pub virtual_list_visible_range_refreshes: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_total: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_non_retained: u32,
    #[serde(default)]
    pub retained_virtual_list_reconciles: u32,
    #[serde(default)]
    pub retained_virtual_list_attached_items: u32,
    #[serde(default)]
    pub retained_virtual_list_detached_items: u32,
    pub focused_node: Option<u64>,
    pub captured_node: Option<u64>,

    // Renderer (wgpu) perf sample (best-effort; may be absent or lag a frame).
    #[serde(default)]
    pub renderer_tick_id: u64,
    #[serde(default)]
    pub renderer_frame_id: u64,
    #[serde(default)]
    pub renderer_frames: u64,
    #[serde(default)]
    pub renderer_encode_scene_us: u64,
    #[serde(default)]
    pub renderer_prepare_svg_us: u64,
    #[serde(default)]
    pub renderer_prepare_text_us: u64,
    #[serde(default)]
    pub renderer_svg_uploads: u64,
    #[serde(default)]
    pub renderer_svg_upload_bytes: u64,
    #[serde(default)]
    pub renderer_image_uploads: u64,
    #[serde(default)]
    pub renderer_image_upload_bytes: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_bytes: u64,
    #[serde(default)]
    pub renderer_svg_rasters_live: u64,
    #[serde(default)]
    pub renderer_svg_standalone_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_pages_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_used_px: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_capacity_px: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_hits: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_misses: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_page_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_entries_evicted: u64,
    #[serde(default)]
    pub renderer_text_atlas_revision: u64,
    #[serde(default)]
    pub renderer_text_atlas_uploads: u64,
    #[serde(default)]
    pub renderer_text_atlas_upload_bytes: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_pages: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_page_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_resets: u64,
    #[serde(default)]
    pub renderer_intermediate_budget_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_peak_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_release_targets: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_allocations: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_reuses: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_releases: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_evictions: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_textures: u64,
    #[serde(default)]
    pub renderer_draw_calls: u64,
    #[serde(default)]
    pub renderer_text_draw_calls: u64,
    #[serde(default)]
    pub renderer_quad_draw_calls: u64,
    #[serde(default)]
    pub renderer_mask_draw_calls: u64,
    #[serde(default)]
    pub renderer_pipeline_switches: u64,
    #[serde(default)]
    pub renderer_bind_group_switches: u64,
    #[serde(default)]
    pub renderer_scissor_sets: u64,
    #[serde(default)]
    pub renderer_uniform_bytes: u64,
    #[serde(default)]
    pub renderer_instance_bytes: u64,
    #[serde(default)]
    pub renderer_vertex_bytes: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_hits: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_misses: u64,
}

impl UiFrameStatsV1 {
    fn from_stats(
        stats: UiDebugFrameStats,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
    ) -> Self {
        let mut out = Self {
            frame_arena_capacity_estimate_bytes: stats.frame_arena_capacity_estimate_bytes,
            frame_arena_grow_events: stats.frame_arena_grow_events,
            element_children_vec_pool_reuses: stats.element_children_vec_pool_reuses,
            element_children_vec_pool_misses: stats.element_children_vec_pool_misses,
            layout_time_us: stats.layout_time.as_micros() as u64,
            layout_collect_roots_time_us: stats.layout_collect_roots_time.as_micros() as u64,
            layout_invalidate_scroll_handle_bindings_time_us: stats
                .layout_invalidate_scroll_handle_bindings_time
                .as_micros() as u64,
            layout_expand_view_cache_invalidations_time_us: stats
                .layout_expand_view_cache_invalidations_time
                .as_micros() as u64,
            layout_request_build_roots_time_us: stats.layout_request_build_roots_time.as_micros()
                as u64,
            layout_pending_barrier_relayouts_time_us: stats
                .layout_pending_barrier_relayouts_time
                .as_micros() as u64,
            layout_repair_view_cache_bounds_time_us: stats
                .layout_repair_view_cache_bounds_time
                .as_micros() as u64,
            layout_contained_view_cache_roots_time_us: stats
                .layout_contained_view_cache_roots_time
                .as_micros() as u64,
            layout_collapse_layout_observations_time_us: stats
                .layout_collapse_layout_observations_time
                .as_micros() as u64,
            layout_prepaint_after_layout_time_us: stats
                .layout_prepaint_after_layout_time
                .as_micros() as u64,
            layout_skipped_engine_frame: stats.layout_skipped_engine_frame,
            layout_roots_time_us: stats.layout_roots_time.as_micros() as u64,
            layout_barrier_relayouts_time_us: stats.layout_barrier_relayouts_time.as_micros()
                as u64,
            layout_view_cache_time_us: stats.layout_view_cache_time.as_micros() as u64,
            layout_semantics_refresh_time_us: stats.layout_semantics_refresh_time.as_micros()
                as u64,
            layout_focus_repair_time_us: stats.layout_focus_repair_time.as_micros() as u64,
            layout_deferred_cleanup_time_us: stats.layout_deferred_cleanup_time.as_micros() as u64,
            prepaint_time_us: stats.prepaint_time.as_micros() as u64,
            paint_time_us: stats.paint_time.as_micros() as u64,
            paint_record_visual_bounds_time_us: stats.paint_record_visual_bounds_time.as_micros()
                as u64,
            paint_record_visual_bounds_calls: stats.paint_record_visual_bounds_calls,
            paint_cache_key_time_us: stats.paint_cache_key_time.as_micros() as u64,
            paint_cache_hit_check_time_us: stats.paint_cache_hit_check_time.as_micros() as u64,
            paint_widget_time_us: stats.paint_widget_time.as_micros() as u64,
            paint_observation_record_time_us: stats.paint_observation_record_time.as_micros()
                as u64,
            paint_host_widget_observed_models_time_us: stats
                .paint_host_widget_observed_models_time
                .as_micros() as u64,
            paint_host_widget_observed_models_items: stats.paint_host_widget_observed_models_items,
            paint_host_widget_observed_globals_time_us: stats
                .paint_host_widget_observed_globals_time
                .as_micros() as u64,
            paint_host_widget_observed_globals_items: stats
                .paint_host_widget_observed_globals_items,
            paint_host_widget_instance_lookup_time_us: stats
                .paint_host_widget_instance_lookup_time
                .as_micros() as u64,
            paint_host_widget_instance_lookup_calls: stats.paint_host_widget_instance_lookup_calls,
            paint_text_prepare_time_us: stats.paint_text_prepare_time.as_micros() as u64,
            paint_text_prepare_calls: stats.paint_text_prepare_calls,
            paint_text_prepare_reason_blob_missing: stats.paint_text_prepare_reason_blob_missing,
            paint_text_prepare_reason_scale_changed: stats.paint_text_prepare_reason_scale_changed,
            paint_text_prepare_reason_text_changed: stats.paint_text_prepare_reason_text_changed,
            paint_text_prepare_reason_rich_changed: stats.paint_text_prepare_reason_rich_changed,
            paint_text_prepare_reason_style_changed: stats.paint_text_prepare_reason_style_changed,
            paint_text_prepare_reason_wrap_changed: stats.paint_text_prepare_reason_wrap_changed,
            paint_text_prepare_reason_overflow_changed: stats
                .paint_text_prepare_reason_overflow_changed,
            paint_text_prepare_reason_width_changed: stats.paint_text_prepare_reason_width_changed,
            paint_text_prepare_reason_font_stack_changed: stats
                .paint_text_prepare_reason_font_stack_changed,
            paint_input_context_time_us: stats.paint_input_context_time.as_micros() as u64,
            paint_scroll_handle_invalidation_time_us: stats
                .paint_scroll_handle_invalidation_time
                .as_micros() as u64,
            paint_collect_roots_time_us: stats.paint_collect_roots_time.as_micros() as u64,
            paint_publish_text_input_snapshot_time_us: stats
                .paint_publish_text_input_snapshot_time
                .as_micros() as u64,
            paint_collapse_observations_time_us: stats.paint_collapse_observations_time.as_micros()
                as u64,
            dispatch_time_us: stats.dispatch_time.as_micros() as u64,
            dispatch_pointer_events: stats.dispatch_pointer_events,
            dispatch_pointer_event_time_us: stats.dispatch_pointer_event_time.as_micros() as u64,
            dispatch_timer_events: stats.dispatch_timer_events,
            dispatch_timer_event_time_us: stats.dispatch_timer_event_time.as_micros() as u64,
            dispatch_timer_targeted_events: stats.dispatch_timer_targeted_events,
            dispatch_timer_targeted_time_us: stats.dispatch_timer_targeted_time.as_micros() as u64,
            dispatch_timer_broadcast_events: stats.dispatch_timer_broadcast_events,
            dispatch_timer_broadcast_time_us: stats.dispatch_timer_broadcast_time.as_micros()
                as u64,
            dispatch_timer_broadcast_layers_visited: stats.dispatch_timer_broadcast_layers_visited,
            dispatch_timer_broadcast_rebuild_visible_layers_time_us: stats
                .dispatch_timer_broadcast_rebuild_visible_layers_time
                .as_micros()
                as u64,
            dispatch_timer_broadcast_loop_time_us: stats
                .dispatch_timer_broadcast_loop_time
                .as_micros() as u64,
            dispatch_timer_slowest_event_time_us: stats
                .dispatch_timer_slowest_event_time
                .as_micros() as u64,
            dispatch_timer_slowest_token: stats.dispatch_timer_slowest_token.map(|t| t.0),
            dispatch_timer_slowest_was_broadcast: stats.dispatch_timer_slowest_was_broadcast,
            dispatch_other_events: stats.dispatch_other_events,
            dispatch_other_event_time_us: stats.dispatch_other_event_time.as_micros() as u64,
            hit_test_time_us: stats.hit_test_time.as_micros() as u64,
            dispatch_events: stats.dispatch_events,
            hit_test_queries: stats.hit_test_queries,
            hit_test_bounds_tree_queries: stats.hit_test_bounds_tree_queries,
            hit_test_bounds_tree_disabled: stats.hit_test_bounds_tree_disabled,
            hit_test_bounds_tree_misses: stats.hit_test_bounds_tree_misses,
            hit_test_bounds_tree_hits: stats.hit_test_bounds_tree_hits,
            hit_test_bounds_tree_candidate_rejected: stats.hit_test_bounds_tree_candidate_rejected,
            hit_test_bounds_tree_nodes_visited: stats.hit_test_bounds_tree_nodes_visited,
            hit_test_bounds_tree_nodes_pushed: stats.hit_test_bounds_tree_nodes_pushed,
            hit_test_path_cache_hits: stats.hit_test_path_cache_hits,
            hit_test_path_cache_misses: stats.hit_test_path_cache_misses,
            hit_test_cached_path_time_us: stats.hit_test_cached_path_time.as_micros() as u64,
            hit_test_bounds_tree_query_time_us: stats.hit_test_bounds_tree_query_time.as_micros()
                as u64,
            hit_test_candidate_self_only_time_us: stats
                .hit_test_candidate_self_only_time
                .as_micros() as u64,
            hit_test_fallback_traversal_time_us: stats.hit_test_fallback_traversal_time.as_micros()
                as u64,
            dispatch_hover_update_time_us: stats.dispatch_hover_update_time.as_micros() as u64,
            dispatch_scroll_handle_invalidation_time_us: stats
                .dispatch_scroll_handle_invalidation_time
                .as_micros() as u64,
            dispatch_active_layers_time_us: stats.dispatch_active_layers_time.as_micros() as u64,
            dispatch_input_context_time_us: stats.dispatch_input_context_time.as_micros() as u64,
            dispatch_event_chain_build_time_us: stats.dispatch_event_chain_build_time.as_micros()
                as u64,
            dispatch_widget_capture_time_us: stats.dispatch_widget_capture_time.as_micros() as u64,
            dispatch_widget_bubble_time_us: stats.dispatch_widget_bubble_time.as_micros() as u64,
            dispatch_cursor_query_time_us: stats.dispatch_cursor_query_time.as_micros() as u64,
            dispatch_pointer_move_layer_observers_time_us: stats
                .dispatch_pointer_move_layer_observers_time
                .as_micros() as u64,
            dispatch_synth_hover_observer_time_us: stats
                .dispatch_synth_hover_observer_time
                .as_micros() as u64,
            dispatch_cursor_effect_time_us: stats.dispatch_cursor_effect_time.as_micros() as u64,
            dispatch_post_dispatch_snapshot_time_us: stats
                .dispatch_post_dispatch_snapshot_time
                .as_micros() as u64,
            layout_nodes_visited: stats.layout_nodes_visited,
            layout_nodes_performed: stats.layout_nodes_performed,
            prepaint_nodes_visited: stats.prepaint_nodes_visited,
            paint_nodes: stats.paint_nodes,
            paint_nodes_performed: stats.paint_nodes_performed,
            paint_cache_hits: stats.paint_cache_hits,
            paint_cache_misses: stats.paint_cache_misses,
            paint_cache_replayed_ops: stats.paint_cache_replayed_ops,
            paint_cache_replay_time_us: stats.paint_cache_replay_time.as_micros() as u64,
            paint_cache_bounds_translate_time_us: stats
                .paint_cache_bounds_translate_time
                .as_micros() as u64,
            paint_cache_bounds_translated_nodes: stats.paint_cache_bounds_translated_nodes,
            interaction_cache_hits: stats.interaction_cache_hits,
            interaction_cache_misses: stats.interaction_cache_misses,
            interaction_cache_replayed_records: stats.interaction_cache_replayed_records,
            interaction_records: stats.interaction_records,
            layout_engine_solves: stats.layout_engine_solves,
            layout_engine_solve_time_us: stats.layout_engine_solve_time.as_micros() as u64,
            layout_engine_widget_fallback_solves: stats.layout_engine_widget_fallback_solves,
            layout_fast_path_taken: stats.layout_fast_path_taken,
            layout_invalidations_count: stats.layout_invalidations_count,
            model_change_invalidation_roots: stats.model_change_invalidation_roots,
            model_change_models: stats.model_change_models,
            model_change_observation_edges: stats.model_change_observation_edges,
            model_change_unobserved_models: stats.model_change_unobserved_models,
            global_change_invalidation_roots: stats.global_change_invalidation_roots,
            global_change_globals: stats.global_change_globals,
            global_change_observation_edges: stats.global_change_observation_edges,
            global_change_unobserved_globals: stats.global_change_unobserved_globals,
            invalidation_walk_nodes: stats.invalidation_walk_nodes,
            invalidation_walk_calls: stats.invalidation_walk_calls,
            invalidation_walk_nodes_model_change: stats.invalidation_walk_nodes_model_change,
            invalidation_walk_calls_model_change: stats.invalidation_walk_calls_model_change,
            invalidation_walk_nodes_global_change: stats.invalidation_walk_nodes_global_change,
            invalidation_walk_calls_global_change: stats.invalidation_walk_calls_global_change,
            invalidation_walk_nodes_hover: stats.invalidation_walk_nodes_hover,
            invalidation_walk_calls_hover: stats.invalidation_walk_calls_hover,
            invalidation_walk_nodes_focus: stats.invalidation_walk_nodes_focus,
            invalidation_walk_calls_focus: stats.invalidation_walk_calls_focus,
            invalidation_walk_nodes_other: stats.invalidation_walk_nodes_other,
            invalidation_walk_calls_other: stats.invalidation_walk_calls_other,
            hover_pressable_target_changes: stats.hover_pressable_target_changes,
            hover_hover_region_target_changes: stats.hover_hover_region_target_changes,
            hover_declarative_instance_changes: stats.hover_declarative_instance_changes,
            hover_declarative_hit_test_invalidations: stats
                .hover_declarative_hit_test_invalidations,
            hover_declarative_layout_invalidations: stats.hover_declarative_layout_invalidations,
            hover_declarative_paint_invalidations: stats.hover_declarative_paint_invalidations,
            view_cache_active: stats.view_cache_active,
            view_cache_invalidation_truncations: stats.view_cache_invalidation_truncations,
            view_cache_contained_relayouts: stats.view_cache_contained_relayouts,
            view_cache_roots_total: stats.view_cache_roots_total,
            view_cache_roots_reused: stats.view_cache_roots_reused,
            view_cache_roots_first_mount: stats.view_cache_roots_first_mount,
            view_cache_roots_node_recreated: stats.view_cache_roots_node_recreated,
            view_cache_roots_cache_key_mismatch: stats.view_cache_roots_cache_key_mismatch,
            view_cache_roots_not_marked_reuse_root: stats.view_cache_roots_not_marked_reuse_root,
            view_cache_roots_needs_rerender: stats.view_cache_roots_needs_rerender,
            view_cache_roots_layout_invalidated: stats.view_cache_roots_layout_invalidated,
            view_cache_roots_manual: stats.view_cache_roots_manual,
            set_children_barrier_writes: stats.set_children_barrier_writes,
            barrier_relayouts_scheduled: stats.barrier_relayouts_scheduled,
            barrier_relayouts_performed: stats.barrier_relayouts_performed,
            virtual_list_visible_range_checks: stats.virtual_list_visible_range_checks,
            virtual_list_visible_range_refreshes: stats.virtual_list_visible_range_refreshes,
            virtual_list_window_shifts_total: stats.virtual_list_window_shifts_total,
            virtual_list_window_shifts_non_retained: stats.virtual_list_window_shifts_non_retained,
            retained_virtual_list_reconciles: stats.retained_virtual_list_reconciles,
            retained_virtual_list_attached_items: stats.retained_virtual_list_attached_items,
            retained_virtual_list_detached_items: stats.retained_virtual_list_detached_items,
            focused_node: stats.focus.map(key_to_u64),
            captured_node: stats.captured.map(key_to_u64),
            renderer_tick_id: 0,
            renderer_frame_id: 0,
            renderer_frames: 0,
            renderer_encode_scene_us: 0,
            renderer_prepare_svg_us: 0,
            renderer_prepare_text_us: 0,
            renderer_svg_uploads: 0,
            renderer_svg_upload_bytes: 0,
            renderer_image_uploads: 0,
            renderer_image_upload_bytes: 0,
            renderer_svg_raster_budget_bytes: 0,
            renderer_svg_rasters_live: 0,
            renderer_svg_standalone_bytes_live: 0,
            renderer_svg_mask_atlas_pages_live: 0,
            renderer_svg_mask_atlas_bytes_live: 0,
            renderer_svg_mask_atlas_used_px: 0,
            renderer_svg_mask_atlas_capacity_px: 0,
            renderer_svg_raster_cache_hits: 0,
            renderer_svg_raster_cache_misses: 0,
            renderer_svg_raster_budget_evictions: 0,
            renderer_svg_mask_atlas_page_evictions: 0,
            renderer_svg_mask_atlas_entries_evicted: 0,
            renderer_text_atlas_revision: 0,
            renderer_text_atlas_uploads: 0,
            renderer_text_atlas_upload_bytes: 0,
            renderer_text_atlas_evicted_glyphs: 0,
            renderer_text_atlas_evicted_pages: 0,
            renderer_text_atlas_evicted_page_glyphs: 0,
            renderer_text_atlas_resets: 0,
            renderer_intermediate_budget_bytes: 0,
            renderer_intermediate_in_use_bytes: 0,
            renderer_intermediate_peak_in_use_bytes: 0,
            renderer_intermediate_release_targets: 0,
            renderer_intermediate_pool_allocations: 0,
            renderer_intermediate_pool_reuses: 0,
            renderer_intermediate_pool_releases: 0,
            renderer_intermediate_pool_evictions: 0,
            renderer_intermediate_pool_free_bytes: 0,
            renderer_intermediate_pool_free_textures: 0,
            renderer_draw_calls: 0,
            renderer_text_draw_calls: 0,
            renderer_quad_draw_calls: 0,
            renderer_mask_draw_calls: 0,
            renderer_pipeline_switches: 0,
            renderer_bind_group_switches: 0,
            renderer_scissor_sets: 0,
            renderer_uniform_bytes: 0,
            renderer_instance_bytes: 0,
            renderer_vertex_bytes: 0,
            renderer_scene_encoding_cache_hits: 0,
            renderer_scene_encoding_cache_misses: 0,
        };

        if let Some(sample) = renderer_perf {
            out.renderer_tick_id = sample.tick_id;
            out.renderer_frame_id = sample.frame_id;
            out.renderer_frames = sample.perf.frames;
            out.renderer_encode_scene_us = sample.perf.encode_scene_us;
            out.renderer_prepare_svg_us = sample.perf.prepare_svg_us;
            out.renderer_prepare_text_us = sample.perf.prepare_text_us;
            out.renderer_svg_uploads = sample.perf.svg_uploads;
            out.renderer_svg_upload_bytes = sample.perf.svg_upload_bytes;
            out.renderer_image_uploads = sample.perf.image_uploads;
            out.renderer_image_upload_bytes = sample.perf.image_upload_bytes;
            out.renderer_svg_raster_budget_bytes = sample.perf.svg_raster_budget_bytes;
            out.renderer_svg_rasters_live = sample.perf.svg_rasters_live;
            out.renderer_svg_standalone_bytes_live = sample.perf.svg_standalone_bytes_live;
            out.renderer_svg_mask_atlas_pages_live = sample.perf.svg_mask_atlas_pages_live;
            out.renderer_svg_mask_atlas_bytes_live = sample.perf.svg_mask_atlas_bytes_live;
            out.renderer_svg_mask_atlas_used_px = sample.perf.svg_mask_atlas_used_px;
            out.renderer_svg_mask_atlas_capacity_px = sample.perf.svg_mask_atlas_capacity_px;
            out.renderer_svg_raster_cache_hits = sample.perf.svg_raster_cache_hits;
            out.renderer_svg_raster_cache_misses = sample.perf.svg_raster_cache_misses;
            out.renderer_svg_raster_budget_evictions = sample.perf.svg_raster_budget_evictions;
            out.renderer_svg_mask_atlas_page_evictions = sample.perf.svg_mask_atlas_page_evictions;
            out.renderer_svg_mask_atlas_entries_evicted =
                sample.perf.svg_mask_atlas_entries_evicted;
            out.renderer_text_atlas_revision = sample.perf.text_atlas_revision;
            out.renderer_text_atlas_uploads = sample.perf.text_atlas_uploads;
            out.renderer_text_atlas_upload_bytes = sample.perf.text_atlas_upload_bytes;
            out.renderer_text_atlas_evicted_glyphs = sample.perf.text_atlas_evicted_glyphs;
            out.renderer_text_atlas_evicted_pages = sample.perf.text_atlas_evicted_pages;
            out.renderer_text_atlas_evicted_page_glyphs =
                sample.perf.text_atlas_evicted_page_glyphs;
            out.renderer_text_atlas_resets = sample.perf.text_atlas_resets;
            out.renderer_intermediate_budget_bytes = sample.perf.intermediate_budget_bytes;
            out.renderer_intermediate_in_use_bytes = sample.perf.intermediate_in_use_bytes;
            out.renderer_intermediate_peak_in_use_bytes =
                sample.perf.intermediate_peak_in_use_bytes;
            out.renderer_intermediate_release_targets = sample.perf.intermediate_release_targets;
            out.renderer_intermediate_pool_allocations = sample.perf.intermediate_pool_allocations;
            out.renderer_intermediate_pool_reuses = sample.perf.intermediate_pool_reuses;
            out.renderer_intermediate_pool_releases = sample.perf.intermediate_pool_releases;
            out.renderer_intermediate_pool_evictions = sample.perf.intermediate_pool_evictions;
            out.renderer_intermediate_pool_free_bytes = sample.perf.intermediate_pool_free_bytes;
            out.renderer_intermediate_pool_free_textures =
                sample.perf.intermediate_pool_free_textures;
            out.renderer_draw_calls = sample.perf.draw_calls;
            out.renderer_text_draw_calls = sample.perf.text_draw_calls;
            out.renderer_quad_draw_calls = sample.perf.quad_draw_calls;
            out.renderer_mask_draw_calls = sample.perf.mask_draw_calls;
            out.renderer_pipeline_switches = sample.perf.pipeline_switches;
            out.renderer_bind_group_switches = sample.perf.bind_group_switches;
            out.renderer_scissor_sets = sample.perf.scissor_sets;
            out.renderer_uniform_bytes = sample.perf.uniform_bytes;
            out.renderer_instance_bytes = sample.perf.instance_bytes;
            out.renderer_vertex_bytes = sample.perf.vertex_bytes;
            out.renderer_scene_encoding_cache_hits = sample.perf.scene_encoding_cache_hits;
            out.renderer_scene_encoding_cache_misses = sample.perf.scene_encoding_cache_misses;
        }

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayerInfoV1 {
    pub id: String,
    /// Numeric layer id (stable across `Debug` formatting changes; not stable between runs).
    #[serde(default)]
    pub layer_id: u64,
    pub root: u64,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    /// Pointer occlusion mode for this layer root (when applicable).
    #[serde(default)]
    pub pointer_occlusion: String,
    pub wants_pointer_down_outside_events: bool,
    #[serde(default)]
    pub consume_pointer_down_outside_events: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pointer_down_outside_branches: Vec<u64>,
    pub wants_pointer_move_events: bool,
    pub wants_timer_events: bool,
}

impl UiLayerInfoV1 {
    fn from_layer(layer: UiDebugLayerInfo) -> Self {
        Self {
            id: format!("{:?}", layer.id),
            layer_id: layer.id.data().as_ffi(),
            root: key_to_u64(layer.root),
            visible: layer.visible,
            blocks_underlay_input: layer.blocks_underlay_input,
            hit_testable: layer.hit_testable,
            pointer_occlusion: pointer_occlusion_label(layer.pointer_occlusion),
            wants_pointer_down_outside_events: layer.wants_pointer_down_outside_events,
            consume_pointer_down_outside_events: layer.consume_pointer_down_outside_events,
            pointer_down_outside_branches: layer
                .pointer_down_outside_branches
                .into_iter()
                .take(32)
                .map(key_to_u64)
                .collect(),
            wants_pointer_move_events: layer.wants_pointer_move_events,
            wants_timer_events: layer.wants_timer_events,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayerVisibleWriteV1 {
    pub layer: String,
    pub prev_visible: Option<bool>,
    pub visible: bool,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl UiLayerVisibleWriteV1 {
    fn from_write(write: &fret_ui::tree::UiDebugSetLayerVisibleWrite) -> Self {
        Self {
            layer: format!("{:?}", write.layer),
            prev_visible: write.prev_visible,
            visible: write.visible,
            file: write.file.to_string(),
            line: write.line,
            column: write.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiOverlayPolicyDecisionV1 {
    pub layer: String,
    pub kind: String,
    pub present: bool,
    pub interactive: bool,
    pub wants_timer_events: bool,
    pub reason: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u32,
    #[serde(default)]
    pub column: u32,
}

impl UiOverlayPolicyDecisionV1 {
    fn from_decision(d: &fret_ui::tree::UiDebugOverlayPolicyDecisionWrite) -> Self {
        Self {
            layer: format!("{:?}", d.layer),
            kind: d.kind.to_string(),
            present: d.present,
            interactive: d.interactive,
            wants_timer_events: d.wants_timer_events,
            reason: d.reason.to_string(),
            file: d.file.to_string(),
            line: d.line,
            column: d.column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestSnapshotV1 {
    pub position: PointV1,
    pub hit: Option<u64>,
    pub active_layer_roots: Vec<u64>,
    pub barrier_root: Option<u64>,
    #[serde(default)]
    pub focus_barrier_root: Option<u64>,
    /// Stable, script-friendly labels for each scope root.
    ///
    /// Prefer this over `active_layer_roots` when validating behavior across refactors, since node
    /// ids are not stable between runs.
    #[serde(default)]
    pub scope_roots: Vec<UiHitTestScopeRootV1>,
}

impl UiHitTestSnapshotV1 {
    fn from_tree(position: Point, ui: &UiTree<App>) -> Self {
        let hit_test = ui.debug_hit_test(position);
        let arbitration = ui.input_arbitration_snapshot();
        let layers = ui.debug_layers_in_paint_order();
        Self::from_hit_test_with_layers(position, hit_test, arbitration.focus_barrier_root, &layers)
    }

    fn from_hit_test_with_layers(
        position: Point,
        hit_test: UiDebugHitTest,
        focus_barrier_root: Option<NodeId>,
        layers: &[UiDebugLayerInfo],
    ) -> Self {
        let mut scope_roots = Vec::new();
        if let Some(root) = hit_test.barrier_root {
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "modal_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: None,
                pointer_occlusion: None,
                blocks_underlay_input: None,
                hit_testable: None,
            });
        }

        let mut by_root: HashMap<NodeId, &UiDebugLayerInfo> = HashMap::new();
        for layer in layers {
            by_root.insert(layer.root, layer);
        }

        if let Some(root) = focus_barrier_root {
            let info = by_root.get(&root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "focus_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        for root in &hit_test.active_layer_roots {
            let info = by_root.get(root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "layer_root".to_string(),
                root: key_to_u64(*root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        Self {
            position: PointV1::from(position),
            hit: hit_test.hit.map(key_to_u64),
            active_layer_roots: hit_test
                .active_layer_roots
                .into_iter()
                .map(key_to_u64)
                .collect(),
            barrier_root: hit_test.barrier_root.map(key_to_u64),
            focus_barrier_root: focus_barrier_root.map(key_to_u64),
            scope_roots,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestScopeRootV1 {
    /// Stable scope root kind (e.g. `modal_barrier_root`, `layer_root`).
    pub kind: String,
    /// Node id of the root (not stable between runs; treat as an in-run reference only).
    pub root: u64,
    /// When `kind=layer_root`, the corresponding layer id (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<u64>,
    /// Pointer occlusion mode for the layer root (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_underlay_input: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_testable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiagnosticsSnapshotV1 {
    pub focused_element: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_path: Option<String>,
    pub focused_element_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_element_visual_bounds: Option<RectV1>,
    pub active_text_selection: Option<(u64, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_text_selection_path: Option<(String, String)>,
    pub hovered_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_path: Option<String>,
    pub hovered_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_pressable_visual_bounds: Option<RectV1>,
    pub pressed_pressable: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_path: Option<String>,
    pub pressed_pressable_node: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed_pressable_visual_bounds: Option<RectV1>,
    pub hovered_hover_region: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_hover_region_path: Option<String>,
    pub wants_continuous_frames: bool,
    pub observed_models: Vec<ElementObservedModelsV1>,
    pub observed_globals: Vec<ElementObservedGlobalsV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_layout_queries: Vec<ElementObservedLayoutQueriesV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_query_regions: Vec<ElementLayoutQueryRegionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ElementEnvironmentSnapshotV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_environment: Vec<ElementObservedEnvironmentV1>,
    #[serde(default)]
    pub view_cache_reuse_roots: Vec<u64>,
    #[serde(default)]
    pub view_cache_reuse_root_element_counts: Vec<(u64, u32)>,
    #[serde(default)]
    pub view_cache_reuse_root_element_samples: Vec<ElementViewCacheReuseRootElementsSampleV1>,
    #[serde(default)]
    pub retained_keep_alive_roots_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_keep_alive_roots_tail: Vec<u64>,
    #[serde(default)]
    pub node_entry_root_overwrites: Vec<ElementNodeEntryRootOverwriteV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementViewCacheReuseRootElementsSampleV1 {
    pub root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<u64>,
    pub elements_len: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements_tail: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNodeEntryRootOverwriteV1 {
    pub frame_id: u64,
    pub element: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub old_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_root_path: Option<String>,
    pub new_root: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_root_path: Option<String>,
    pub old_node: u64,
    pub new_node: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<UiSourceLocationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedModelsV1 {
    pub element: u64,
    pub models: Vec<(u64, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedGlobalsV1 {
    pub element: u64,
    pub globals: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueriesV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub regions: Vec<ElementObservedLayoutQueryRegionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedLayoutQueryRegionV1 {
    pub region: u64,
    pub invalidation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_name: Option<String>,
    pub region_revision: u64,
    pub region_changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_committed_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLayoutQueryRegionV1 {
    pub region: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub revision: u64,
    pub changed_this_frame: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_bounds: Option<RectV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_bounds: Option<RectV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementEnvironmentSnapshotV1 {
    pub viewport_bounds: RectV1,
    pub scale_factor: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentV1 {
    pub element: u64,
    pub deps_fingerprint: u64,
    pub keys: Vec<ElementObservedEnvironmentKeyV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementObservedEnvironmentKeyV1 {
    pub key: String,
    pub invalidation: String,
    pub key_revision: u64,
    pub key_changed_this_frame: bool,
}

impl ElementDiagnosticsSnapshotV1 {
    fn from_runtime(
        window: AppWindowId,
        runtime: &ElementRuntime,
        snapshot: fret_ui::elements::WindowElementDiagnosticsSnapshot,
    ) -> Self {
        let focused_element_path = snapshot
            .focused_element
            .and_then(|id| runtime.debug_path_for_element(window, id));
        let active_text_selection_path = snapshot.active_text_selection.and_then(|(a, b)| {
            let a = runtime.debug_path_for_element(window, a)?;
            let b = runtime.debug_path_for_element(window, b)?;
            Some((a, b))
        });
        let hovered_pressable_path = snapshot
            .hovered_pressable
            .and_then(|id| runtime.debug_path_for_element(window, id));
        let pressed_pressable_path = snapshot
            .pressed_pressable
            .and_then(|id| runtime.debug_path_for_element(window, id));
        let hovered_hover_region_path = snapshot
            .hovered_hover_region
            .and_then(|id| runtime.debug_path_for_element(window, id));

        Self {
            focused_element: snapshot.focused_element.map(|id| id.0),
            focused_element_path,
            focused_element_node: snapshot.focused_element_node.map(key_to_u64),
            focused_element_bounds: snapshot.focused_element_bounds.map(RectV1::from),
            focused_element_visual_bounds: snapshot.focused_element_visual_bounds.map(RectV1::from),
            active_text_selection: snapshot.active_text_selection.map(|(a, b)| (a.0, b.0)),
            active_text_selection_path,
            hovered_pressable: snapshot.hovered_pressable.map(|id| id.0),
            hovered_pressable_path,
            hovered_pressable_node: snapshot.hovered_pressable_node.map(key_to_u64),
            hovered_pressable_bounds: snapshot.hovered_pressable_bounds.map(RectV1::from),
            hovered_pressable_visual_bounds: snapshot
                .hovered_pressable_visual_bounds
                .map(RectV1::from),
            pressed_pressable: snapshot.pressed_pressable.map(|id| id.0),
            pressed_pressable_path,
            pressed_pressable_node: snapshot.pressed_pressable_node.map(key_to_u64),
            pressed_pressable_bounds: snapshot.pressed_pressable_bounds.map(RectV1::from),
            pressed_pressable_visual_bounds: snapshot
                .pressed_pressable_visual_bounds
                .map(RectV1::from),
            hovered_hover_region: snapshot.hovered_hover_region.map(|id| id.0),
            hovered_hover_region_path,
            wants_continuous_frames: snapshot.wants_continuous_frames,
            observed_models: snapshot
                .observed_models
                .into_iter()
                .map(|(element, list)| ElementObservedModelsV1 {
                    element: element.0,
                    models: list
                        .into_iter()
                        .map(|(id, inv)| (id, invalidation_label(inv).to_string()))
                        .collect(),
                })
                .collect(),
            observed_globals: snapshot
                .observed_globals
                .into_iter()
                .map(|(element, list)| ElementObservedGlobalsV1 {
                    element: element.0,
                    globals: list
                        .into_iter()
                        .map(|(id, inv)| (id, invalidation_label(inv).to_string()))
                        .collect(),
                })
                .collect(),
            observed_layout_queries: snapshot
                .observed_layout_queries
                .into_iter()
                .map(|entry| ElementObservedLayoutQueriesV1 {
                    element: entry.element.0,
                    deps_fingerprint: entry.deps_fingerprint,
                    regions: entry
                        .regions
                        .into_iter()
                        .map(|r| ElementObservedLayoutQueryRegionV1 {
                            region: r.region.0,
                            invalidation: invalidation_label(r.invalidation).to_string(),
                            region_name: r.region_name.map(|name| name.to_string()),
                            region_revision: r.region_revision,
                            region_changed_this_frame: r.region_changed_this_frame,
                            region_committed_bounds: r.region_committed_bounds.map(RectV1::from),
                        })
                        .collect(),
                })
                .collect(),
            layout_query_regions: snapshot
                .layout_query_regions
                .into_iter()
                .map(|r| ElementLayoutQueryRegionV1 {
                    region: r.region.0,
                    name: r.name.map(|name| name.to_string()),
                    revision: r.revision,
                    changed_this_frame: r.changed_this_frame,
                    committed_bounds: r.committed_bounds.map(RectV1::from),
                    current_bounds: r.current_bounds.map(RectV1::from),
                })
                .collect(),
            environment: Some(ElementEnvironmentSnapshotV1 {
                viewport_bounds: RectV1::from(snapshot.environment.viewport_bounds),
                scale_factor: snapshot.environment.scale_factor,
                prefers_reduced_motion: snapshot.environment.prefers_reduced_motion,
            }),
            observed_environment: snapshot
                .observed_environment
                .into_iter()
                .map(|entry| ElementObservedEnvironmentV1 {
                    element: entry.element.0,
                    deps_fingerprint: entry.deps_fingerprint,
                    keys: entry
                        .keys
                        .into_iter()
                        .map(|k| ElementObservedEnvironmentKeyV1 {
                            key: k.key.to_string(),
                            invalidation: invalidation_label(k.invalidation).to_string(),
                            key_revision: k.key_revision,
                            key_changed_this_frame: k.key_changed_this_frame,
                        })
                        .collect(),
                })
                .collect(),
            view_cache_reuse_roots: snapshot
                .view_cache_reuse_roots
                .into_iter()
                .map(|id| id.0)
                .collect(),
            view_cache_reuse_root_element_counts: snapshot
                .view_cache_reuse_root_element_counts
                .into_iter()
                .map(|(id, count)| (id.0, count))
                .collect(),
            view_cache_reuse_root_element_samples: snapshot
                .view_cache_reuse_root_element_samples
                .into_iter()
                .map(|s| ElementViewCacheReuseRootElementsSampleV1 {
                    root: s.root.0,
                    node: s.node.map(|n| n.data().as_ffi()),
                    elements_len: s.elements_len,
                    elements_head: s.elements_head.into_iter().map(|id| id.0).collect(),
                    elements_tail: s.elements_tail.into_iter().map(|id| id.0).collect(),
                })
                .collect(),
            retained_keep_alive_roots_len: snapshot.retained_keep_alive_roots_len,
            retained_keep_alive_roots_head: snapshot
                .retained_keep_alive_roots_head
                .into_iter()
                .map(|n| n.data().as_ffi())
                .collect(),
            retained_keep_alive_roots_tail: snapshot
                .retained_keep_alive_roots_tail
                .into_iter()
                .map(|n| n.data().as_ffi())
                .collect(),
            node_entry_root_overwrites: snapshot
                .node_entry_root_overwrites
                .into_iter()
                .map(|r| ElementNodeEntryRootOverwriteV1 {
                    frame_id: r.frame_id.0,
                    element: r.element.0,
                    element_path: runtime.debug_path_for_element(window, r.element),
                    old_root: r.old_root.0,
                    old_root_path: runtime.debug_path_for_element(window, r.old_root),
                    new_root: r.new_root.0,
                    new_root_path: runtime.debug_path_for_element(window, r.new_root),
                    old_node: r.old_node.data().as_ffi(),
                    new_node: r.new_node.data().as_ffi(),
                    location: Some(UiSourceLocationV1 {
                        file: r.file.to_string(),
                        line: r.line,
                        column: r.column,
                    }),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedUiEventV1 {
    pub tick_id: u64,
    pub frame_id: u64,
    pub window: u64,
    pub kind: String,
    pub position: Option<PointV1>,
    pub debug: String,
}

impl RecordedUiEventV1 {
    fn from_event(app: &App, window: AppWindowId, event: &Event, redact_text: bool) -> Self {
        let kind = event_kind(event);
        let position = event.pointer_event().map(|p| PointV1::from(p.position()));
        let debug = event_debug_string(event, redact_text);

        Self {
            tick_id: app.tick_id().0,
            frame_id: app.frame_id().0,
            window: window.data().as_ffi(),
            kind,
            position,
            debug,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointV1 {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for PointV1 {
    fn from(value: Point) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RectV1 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<Rect> for RectV1 {
    fn from(value: Rect) -> Self {
        Self {
            x: value.origin.x.0,
            y: value.origin.y.0,
            w: value.size.width.0,
            h: value.size.height.0,
        }
    }
}

fn invalidation_label(inv: Invalidation) -> &'static str {
    match inv {
        Invalidation::Paint => "paint",
        Invalidation::Layout => "layout",
        Invalidation::HitTest => "hit_test",
        Invalidation::HitTestOnly => "hit_test_only",
    }
}

fn pointer_occlusion_label(occlusion: fret_ui::tree::PointerOcclusion) -> String {
    match occlusion {
        fret_ui::tree::PointerOcclusion::None => "none",
        fret_ui::tree::PointerOcclusion::BlockMouse => "block_mouse",
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll => "block_mouse_except_scroll",
    }
    .to_string()
}

fn viewport_pointer_type_label(pointer_type: fret_core::PointerType) -> &'static str {
    match pointer_type {
        fret_core::PointerType::Mouse => "mouse",
        fret_core::PointerType::Touch => "touch",
        fret_core::PointerType::Pen => "pen",
        fret_core::PointerType::Unknown => "unknown",
    }
}

fn viewport_cancel_reason_label(reason: fret_core::PointerCancelReason) -> &'static str {
    match reason {
        fret_core::PointerCancelReason::LeftWindow => "left_window",
    }
}

fn event_kind(event: &Event) -> String {
    match event {
        Event::Pointer(p) => format!("pointer.{}", p.kind()),
        Event::KeyDown { .. } => "key.down".to_string(),
        Event::KeyUp { .. } => "key.up".to_string(),
        Event::TextInput(_) => "text.input".to_string(),
        Event::Ime(_) => "ime".to_string(),
        Event::Timer { .. } => "timer".to_string(),
        Event::WindowCloseRequested => "window.close_requested".to_string(),
        other => format!("{other:?}")
            .split_whitespace()
            .next()
            .unwrap_or("event")
            .to_string(),
    }
}

fn event_debug_string(event: &Event, redact_text: bool) -> String {
    if !redact_text {
        return format!("{event:?}");
    }

    match event {
        Event::TextInput(text) => format!("TextInput(len={})", text.len()),
        Event::Ime(_) => "Ime(<redacted>)".to_string(),
        _ => format!("{event:?}"),
    }
}

fn unix_ms_now() -> u64 {
    fret_core::time::SystemTime::now()
        .duration_since(fret_core::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn env_flag_default_false(name: &str) -> bool {
    let Ok(v) = std::env::var(name) else {
        return false;
    };
    let v = v.trim().to_ascii_lowercase();
    if v.is_empty() {
        return true;
    }
    !matches!(v.as_str(), "0" | "false" | "no" | "off")
}

fn env_flag_default_true(name: &str) -> bool {
    let Ok(v) = std::env::var(name) else {
        return true;
    };
    let v = v.trim().to_ascii_lowercase();
    if v.is_empty() {
        return true;
    }
    !matches!(v.as_str(), "0" | "false" | "no" | "off")
}

fn semantics_fingerprint_v1(
    snapshot: &fret_core::SemanticsSnapshot,
    redact_text: bool,
    max_string_bytes: usize,
) -> u64 {
    let mut hasher = Fnv1a64::new();
    hasher.write_u64(snapshot.window.data().as_ffi());

    for root in &snapshot.roots {
        hasher.write_u64(key_to_u64(root.root));
        hasher.write_bool(root.visible);
        hasher.write_bool(root.blocks_underlay_input);
        hasher.write_bool(root.hit_testable);
        hasher.write_u32(root.z_index);
    }

    hasher.write_opt_u64(snapshot.barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus_barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus.map(key_to_u64));
    hasher.write_opt_u64(snapshot.captured.map(key_to_u64));

    for node in &snapshot.nodes {
        hasher.write_u64(key_to_u64(node.id));
        hasher.write_opt_u64(node.parent.map(key_to_u64));
        hasher.write_str_bytes(semantics_role_label(node.role).as_bytes());

        hasher.write_f32(node.bounds.origin.x.0);
        hasher.write_f32(node.bounds.origin.y.0);
        hasher.write_f32(node.bounds.size.width.0);
        hasher.write_f32(node.bounds.size.height.0);

        hasher.write_bool(node.flags.focused);
        hasher.write_bool(node.flags.captured);
        hasher.write_bool(node.flags.disabled);
        hasher.write_bool(node.flags.selected);
        hasher.write_bool(node.flags.expanded);
        hasher.write_opt_bool(node.flags.checked);

        hasher.write_opt_str(node.test_id.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_u64(node.active_descendant.map(key_to_u64));
        hasher.write_opt_u32(node.pos_in_set);
        hasher.write_opt_u32(node.set_size);
        hasher.write_opt_str(node.label.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_str(node.value.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_pair_u32(node.text_selection);
        hasher.write_opt_pair_u32(node.text_composition);

        hasher.write_bool(node.actions.focus);
        hasher.write_bool(node.actions.invoke);
        hasher.write_bool(node.actions.set_value);
        hasher.write_bool(node.actions.set_text_selection);

        hasher.write_u32(node.labelled_by.len() as u32);
        for id in &node.labelled_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.described_by.len() as u32);
        for id in &node.described_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.controls.len() as u32);
        for id in &node.controls {
            hasher.write_u64(key_to_u64(*id));
        }
    }

    hasher.finish()
}

struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.state ^= v as u64;
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u8(b);
        }
    }

    fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    fn write_bool(&mut self, v: bool) {
        self.write_u8(if v { 1 } else { 0 });
    }

    fn write_opt_u64(&mut self, v: Option<u64>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u64(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u32(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_bool(&mut self, v: Option<bool>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_bool(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_pair_u32(&mut self, v: Option<(u32, u32)>) {
        match v {
            Some((a, b)) => {
                self.write_u8(1);
                self.write_u32(a);
                self.write_u32(b);
            }
            None => self.write_u8(0),
        }
    }

    fn write_str_bytes(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    fn write_opt_str(&mut self, s: Option<&str>, redact_text: bool, max_string_bytes: usize) {
        match s {
            Some(s) => {
                self.write_u8(1);
                if redact_text {
                    self.write_u32(s.len().min(u32::MAX as usize) as u32);
                } else {
                    let bytes = s.as_bytes();
                    self.write_u32(bytes.len().min(max_string_bytes) as u32);
                    self.write_bytes(&bytes[..bytes.len().min(max_string_bytes)]);
                }
            }
            None => self.write_u8(0),
        }
    }

    fn finish(self) -> u64 {
        self.state
    }
}

pub(crate) fn semantics_role_label(role: SemanticsRole) -> &'static str {
    match role {
        SemanticsRole::Generic => "generic",
        SemanticsRole::Window => "window",
        SemanticsRole::Panel => "panel",
        SemanticsRole::Toolbar => "toolbar",
        SemanticsRole::Dialog => "dialog",
        SemanticsRole::AlertDialog => "alert_dialog",
        SemanticsRole::Alert => "alert",
        SemanticsRole::Button => "button",
        SemanticsRole::Checkbox => "checkbox",
        SemanticsRole::Switch => "switch",
        SemanticsRole::Slider => "slider",
        SemanticsRole::ComboBox => "combo_box",
        SemanticsRole::RadioGroup => "radio_group",
        SemanticsRole::RadioButton => "radio_button",
        SemanticsRole::TabList => "tab_list",
        SemanticsRole::Tab => "tab",
        SemanticsRole::TabPanel => "tab_panel",
        SemanticsRole::MenuBar => "menu_bar",
        SemanticsRole::Menu => "menu",
        SemanticsRole::MenuItem => "menu_item",
        SemanticsRole::MenuItemCheckbox => "menu_item_checkbox",
        SemanticsRole::MenuItemRadio => "menu_item_radio",
        SemanticsRole::Tooltip => "tooltip",
        SemanticsRole::Text => "text",
        SemanticsRole::TextField => "text_field",
        SemanticsRole::List => "list",
        SemanticsRole::ListItem => "list_item",
        SemanticsRole::ListBox => "list_box",
        SemanticsRole::ListBoxOption => "list_box_option",
        SemanticsRole::TreeItem => "tree_item",
        SemanticsRole::Viewport => "viewport",
        _ => "unknown",
    }
}

fn parse_semantics_role(s: &str) -> Option<SemanticsRole> {
    let s = s.trim().to_ascii_lowercase();
    Some(match s.as_str() {
        "generic" => SemanticsRole::Generic,
        "window" => SemanticsRole::Window,
        "panel" => SemanticsRole::Panel,
        "toolbar" => SemanticsRole::Toolbar,
        "dialog" => SemanticsRole::Dialog,
        "alert_dialog" => SemanticsRole::AlertDialog,
        "alert" => SemanticsRole::Alert,
        "button" => SemanticsRole::Button,
        "checkbox" => SemanticsRole::Checkbox,
        "switch" => SemanticsRole::Switch,
        "slider" => SemanticsRole::Slider,
        "combo_box" => SemanticsRole::ComboBox,
        "radio_group" => SemanticsRole::RadioGroup,
        "radio_button" => SemanticsRole::RadioButton,
        "tab_list" => SemanticsRole::TabList,
        "tab" => SemanticsRole::Tab,
        "tab_panel" => SemanticsRole::TabPanel,
        "menu_bar" => SemanticsRole::MenuBar,
        "menu" => SemanticsRole::Menu,
        "menu_item" => SemanticsRole::MenuItem,
        "menu_item_checkbox" => SemanticsRole::MenuItemCheckbox,
        "menu_item_radio" => SemanticsRole::MenuItemRadio,
        "tooltip" => SemanticsRole::Tooltip,
        "text" => SemanticsRole::Text,
        "text_field" => SemanticsRole::TextField,
        "list" => SemanticsRole::List,
        "list_item" => SemanticsRole::ListItem,
        "list_box" => SemanticsRole::ListBox,
        "list_box_option" => SemanticsRole::ListBoxOption,
        "tree_item" => SemanticsRole::TreeItem,
        "viewport" => SemanticsRole::Viewport,
        _ => return None,
    })
}

fn select_semantics_node<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);

    match selector {
        UiSelectorV1::NodeId { node } => index
            .by_id
            .get(node)
            .copied()
            .filter(|n| index.is_selectable(n.id.data().as_ffi())),
        UiSelectorV1::RoleAndName { role, name } => {
            let role = parse_semantics_role(role)?;
            pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && n.role == role
                        && n.label.as_deref().is_some_and(|label| label == name)
                }),
                &index,
            )
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
        } => {
            let role = parse_semantics_role(role)?;

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                parsed_ancestors.push((parse_semantics_role(&a.role)?, a.name.as_str()));
            }

            pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && n.role == role
                        && n.label.as_deref().is_some_and(|label| label == name)
                        && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
                }),
                &index,
            )
        }
        UiSelectorV1::TestId { id } => pick_best_match(
            snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id) && n.test_id.as_deref().is_some_and(|v| v == id)
            }),
            &index,
        )
        .or_else(|| {
            // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
            pick_best_match(
                snapshot
                    .nodes
                    .iter()
                    .filter(|n| n.test_id.as_deref().is_some_and(|v| v == id)),
                &index,
            )
        }),
        UiSelectorV1::GlobalElementId { element } => {
            let node = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            })?;
            let node_id = node.data().as_ffi();
            index
                .by_id
                .get(&node_id)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
        }
    }
}

const MAX_SELECTOR_TRACE_ENTRIES: usize = 64;
const MAX_SELECTOR_TRACE_CANDIDATES: usize = 6;
const MAX_HIT_TEST_TRACE_ENTRIES: usize = 64;

fn selector_trace_eq(a: &UiSelectorV1, b: &UiSelectorV1) -> bool {
    match (a, b) {
        (
            UiSelectorV1::RoleAndName {
                role: a_role,
                name: a_name,
            },
            UiSelectorV1::RoleAndName {
                role: b_role,
                name: b_name,
            },
        ) => a_role == b_role && a_name == b_name,
        (
            UiSelectorV1::RoleAndPath {
                role: a_role,
                name: a_name,
                ancestors: a_ancestors,
            },
            UiSelectorV1::RoleAndPath {
                role: b_role,
                name: b_name,
                ancestors: b_ancestors,
            },
        ) => {
            a_role == b_role
                && a_name == b_name
                && a_ancestors.len() == b_ancestors.len()
                && a_ancestors
                    .iter()
                    .zip(b_ancestors.iter())
                    .all(|(a, b)| a.role == b.role && a.name == b.name)
        }
        (UiSelectorV1::TestId { id: a_id }, UiSelectorV1::TestId { id: b_id }) => a_id == b_id,
        (
            UiSelectorV1::GlobalElementId { element: a_el },
            UiSelectorV1::GlobalElementId { element: b_el },
        ) => a_el == b_el,
        (UiSelectorV1::NodeId { node: a_node }, UiSelectorV1::NodeId { node: b_node }) => {
            a_node == b_node
        }
        _ => false,
    }
}

fn hit_test_trace_entry_eq(a: &UiHitTestTraceEntryV1, b: &UiHitTestTraceEntryV1) -> bool {
    a.step_index == b.step_index
        && selector_trace_eq(&a.selector, &b.selector)
        && a.note == b.note
        && a.position.x_px == b.position.x_px
        && a.position.y_px == b.position.y_px
}

fn push_selector_resolution_trace(
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
    entry: UiSelectorResolutionTraceEntryV1,
) {
    if let Some(existing) = trace.iter_mut().rev().find(|e| {
        e.step_index == entry.step_index && selector_trace_eq(&e.selector, &entry.selector)
    }) {
        *existing = entry;
        return;
    }

    trace.push(entry);
    if trace.len() > MAX_SELECTOR_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_SELECTOR_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

fn push_hit_test_trace(trace: &mut Vec<UiHitTestTraceEntryV1>, entry: UiHitTestTraceEntryV1) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| hit_test_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_HIT_TEST_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_HIT_TEST_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

fn script_evidence_for_active(active: &ActiveScript) -> Option<UiScriptEvidenceV1> {
    if active.selector_resolution_trace.is_empty() && active.hit_test_trace.is_empty() {
        return None;
    }
    Some(UiScriptEvidenceV1 {
        selector_resolution_trace: active.selector_resolution_trace.clone(),
        hit_test_trace: active.hit_test_trace.clone(),
    })
}

fn hit_test_scope_roots_evidence(
    position: Point,
    ui: &UiTree<App>,
) -> (
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Vec<UiHitTestScopeRootEvidenceV1>,
) {
    let snap = UiHitTestSnapshotV1::from_tree(position, ui);
    let scope_roots = snap
        .scope_roots
        .into_iter()
        .map(|r| UiHitTestScopeRootEvidenceV1 {
            kind: r.kind,
            root: r.root,
            layer_id: r.layer_id,
            pointer_occlusion: r.pointer_occlusion,
            blocks_underlay_input: r.blocks_underlay_input,
            hit_testable: r.hit_testable,
        })
        .collect();
    (
        snap.hit,
        snap.barrier_root,
        snap.focus_barrier_root,
        scope_roots,
    )
}

fn record_hit_test_trace_for_selector(
    trace: &mut Vec<UiHitTestTraceEntryV1>,
    ui: &UiTree<App>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    selector: &UiSelectorV1,
    step_index: u32,
    position: Point,
    intended: Option<&fret_core::SemanticsNode>,
    note: Option<&str>,
) {
    let (hit_node_id, barrier_root, focus_barrier_root, scope_roots) =
        hit_test_scope_roots_evidence(position, ui);

    let hit_semantics =
        semantics_snapshot.and_then(|snapshot| pick_semantics_node_at(snapshot, ui, position));
    let hit_semantics_node_id = hit_semantics.map(|n| n.id.data().as_ffi());
    let hit_semantics_test_id = hit_semantics.and_then(|n| n.test_id.clone());

    let intended_node_id = intended.map(|n| n.id.data().as_ffi());
    let intended_test_id = intended.and_then(|n| n.test_id.clone());
    let intended_bounds = intended.map(|n| UiRectV1 {
        x_px: n.bounds.origin.x.0,
        y_px: n.bounds.origin.y.0,
        w_px: n.bounds.size.width.0,
        h_px: n.bounds.size.height.0,
    });

    let includes_intended = intended.map(|target| {
        if let Some(hit_id) = hit_semantics_node_id {
            if hit_id == target.id.data().as_ffi() {
                return true;
            }
        }
        if let (Some(want), Some(got)) =
            (target.test_id.as_deref(), hit_semantics_test_id.as_deref())
        {
            return want == got;
        }
        false
    });

    push_hit_test_trace(
        trace,
        UiHitTestTraceEntryV1 {
            step_index,
            selector: selector.clone(),
            position: UiPointV1 {
                x_px: position.x.0,
                y_px: position.y.0,
            },
            intended_node_id,
            intended_test_id,
            intended_bounds,
            hit_node_id,
            hit_semantics_node_id,
            hit_semantics_test_id,
            includes_intended,
            barrier_root,
            focus_barrier_root,
            scope_roots,
            note: note.map(|s| s.to_string()),
        },
    );
}

fn select_semantics_node_with_trace<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    step_index: u32,
    redact_text: bool,
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    let mut matches: Vec<&'a fret_core::SemanticsNode> = Vec::new();
    let mut note: Option<String> = None;

    match selector {
        UiSelectorV1::NodeId { node } => {
            if let Some(n) = index
                .by_id
                .get(node)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
        UiSelectorV1::RoleAndName { role, name } => {
            let Some(role) = parse_semantics_role(role) else {
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id) && n.role == role && n.label.as_deref() == Some(name)
            }));
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
        } => {
            let Some(role) = parse_semantics_role(role) else {
                note = Some("invalid_role".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                let Some(r) = parse_semantics_role(&a.role) else {
                    note = Some("invalid_ancestor_role".to_string());
                    push_selector_resolution_trace(
                        trace,
                        UiSelectorResolutionTraceEntryV1 {
                            step_index,
                            selector: selector.clone(),
                            match_count: 0,
                            chosen_node_id: None,
                            candidates: Vec::new(),
                            note,
                        },
                    );
                    return None;
                };
                parsed_ancestors.push((r, a.name.as_str()));
            }

            matches.extend(snapshot.nodes.iter().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id)
                    && n.role == role
                    && n.label.as_deref() == Some(name)
                    && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
            }));
        }
        UiSelectorV1::TestId { id } => {
            matches.extend(snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id) && n.test_id.as_deref() == Some(id)
            }));
            if matches.is_empty() {
                // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
                note = Some("fallback_hidden_nodes".to_string());
                matches.extend(
                    snapshot
                        .nodes
                        .iter()
                        .filter(|n| n.test_id.as_deref() == Some(id)),
                );
            }
        }
        UiSelectorV1::GlobalElementId { element } => {
            let Some(node) = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            }) else {
                note = Some("element_runtime_missing".to_string());
                push_selector_resolution_trace(
                    trace,
                    UiSelectorResolutionTraceEntryV1 {
                        step_index,
                        selector: selector.clone(),
                        match_count: 0,
                        chosen_node_id: None,
                        candidates: Vec::new(),
                        note,
                    },
                );
                return None;
            };
            let node_id = node.data().as_ffi();
            if let Some(n) = index
                .by_id
                .get(&node_id)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
            {
                matches.push(n);
            }
        }
    }

    let match_count = matches.len().min(u32::MAX as usize) as u32;
    let chosen = pick_best_match(matches.iter().copied(), &index);
    let chosen_node_id = chosen.map(|n| n.id.data().as_ffi());

    let mut ranked: Vec<((u32, u32, u64), &'a fret_core::SemanticsNode)> = matches
        .iter()
        .copied()
        .map(|n| {
            let id = n.id.data().as_ffi();
            ((index.root_z_for(id), index.depth_for(id), id), n)
        })
        .collect();
    ranked.sort_by(|(a, _), (b, _)| b.cmp(a));

    let candidates: Vec<UiSelectorResolutionCandidateV1> = ranked
        .into_iter()
        .take(MAX_SELECTOR_TRACE_CANDIDATES)
        .map(|(_rank, n)| UiSelectorResolutionCandidateV1 {
            node_id: n.id.data().as_ffi(),
            role: semantics_role_label(n.role).to_string(),
            name: if redact_text { None } else { n.label.clone() },
            test_id: n.test_id.clone(),
        })
        .collect();

    push_selector_resolution_trace(
        trace,
        UiSelectorResolutionTraceEntryV1 {
            step_index,
            selector: selector.clone(),
            match_count,
            chosen_node_id,
            candidates,
            note,
        },
    );

    chosen
}

fn reason_code_for_script_failure(reason: &str) -> Option<&'static str> {
    let reason = reason.trim();
    if reason.is_empty() {
        return None;
    }

    match reason {
        "no_semantics_snapshot" => Some("semantics.missing"),
        "assert_failed" => Some("assert.failed"),
        _ if reason.ends_with("_timeout") => Some("timeout"),
        _ if reason.contains("no_semantics_match") || reason.contains("no_match") => {
            Some("selector.not_found")
        }
        _ => None,
    }
}

struct SemanticsIndex<'a> {
    by_id: HashMap<u64, &'a fret_core::SemanticsNode>,
    visible_ids: HashSet<u64>,
    barrier_root: Option<u64>,
    root_z_index: HashMap<u64, u32>,
}

impl<'a> SemanticsIndex<'a> {
    fn new(snapshot: &'a fret_core::SemanticsSnapshot) -> Self {
        let mut by_id: HashMap<u64, &'a fret_core::SemanticsNode> = HashMap::new();
        let mut children: HashMap<u64, Vec<u64>> = HashMap::new();

        for n in &snapshot.nodes {
            let id = n.id.data().as_ffi();
            by_id.insert(id, n);
            if let Some(parent) = n.parent {
                children.entry(parent.data().as_ffi()).or_default().push(id);
            }
        }

        let mut root_z_index: HashMap<u64, u32> = HashMap::new();
        for r in &snapshot.roots {
            root_z_index.insert(r.root.data().as_ffi(), r.z_index);
        }

        let barrier_root = snapshot.barrier_root.map(|n| n.data().as_ffi());

        let mut visible_ids: HashSet<u64> = HashSet::new();
        for root in snapshot.roots.iter().filter(|r| r.visible) {
            collect_subtree_ids(root.root.data().as_ffi(), &children, &mut visible_ids);
        }

        Self {
            by_id,
            visible_ids,
            barrier_root,
            root_z_index,
        }
    }

    fn is_selectable(&self, id: u64) -> bool {
        if !self.visible_ids.contains(&id) {
            return false;
        }
        if let Some(barrier) = self.barrier_root {
            return self.is_descendant_of_or_self(id, barrier);
        }
        true
    }

    fn is_descendant_of_or_self(&self, mut id: u64, ancestor: u64) -> bool {
        if id == ancestor {
            return true;
        }
        while let Some(node) = self.by_id.get(&id).copied() {
            let Some(parent) = node.parent else {
                return false;
            };
            id = parent.data().as_ffi();
            if id == ancestor {
                return true;
            }
        }
        false
    }

    /// Match `ancestors` (outermost -> innermost) as an ordered subsequence along the parent chain.
    fn ancestors_match_subsequence(
        &self,
        start_parent: Option<NodeId>,
        ancestors: &[(SemanticsRole, &str)],
    ) -> bool {
        let mut cur = start_parent.and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());

        for (want_role, want_name) in ancestors.iter().rev() {
            let mut found = false;
            while let Some(node) = cur {
                if node.role == *want_role
                    && node
                        .label
                        .as_deref()
                        .is_some_and(|label| label == *want_name)
                {
                    found = true;
                    cur = node
                        .parent
                        .and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());
                    break;
                }
                cur = node
                    .parent
                    .and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());
            }
            if !found {
                return false;
            }
        }

        true
    }

    fn root_z_for(&self, id: u64) -> u32 {
        let mut cur = Some(id);
        while let Some(node_id) = cur {
            if let Some(z) = self.root_z_index.get(&node_id).copied() {
                return z;
            }
            cur = self
                .by_id
                .get(&node_id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
        0
    }

    fn depth_for(&self, id: u64) -> u32 {
        let mut depth = 0u32;
        let mut cur = Some(id);
        while let Some(node_id) = cur {
            let Some(node) = self.by_id.get(&node_id).copied() else {
                break;
            };
            let Some(parent) = node.parent else {
                break;
            };
            depth = depth.saturating_add(1);
            cur = Some(parent.data().as_ffi());
        }
        depth
    }
}

fn collect_subtree_ids(root: u64, children: &HashMap<u64, Vec<u64>>, out: &mut HashSet<u64>) {
    let mut stack: Vec<u64> = vec![root];
    while let Some(id) = stack.pop() {
        if !out.insert(id) {
            continue;
        }
        if let Some(kids) = children.get(&id) {
            stack.extend(kids.iter().copied());
        }
    }
}

fn pick_best_match<'a>(
    nodes: impl Iterator<Item = &'a fret_core::SemanticsNode>,
    index: &SemanticsIndex<'a>,
) -> Option<&'a fret_core::SemanticsNode> {
    let mut best: Option<(&'a fret_core::SemanticsNode, (u32, u32, u64))> = None;
    for n in nodes {
        let id = n.id.data().as_ffi();
        let rank = (index.root_z_for(id), index.depth_for(id), id);
        match best {
            None => best = Some((n, rank)),
            Some((_, best_rank)) if rank > best_rank => best = Some((n, rank)),
            _ => {}
        }
    }
    best.map(|(n, _)| n)
}

fn eval_predicate(
    snapshot: &fret_core::SemanticsSnapshot,
    window_bounds: Rect,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    pred: &UiPredicateV1,
) -> bool {
    match pred {
        UiPredicateV1::Exists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_some()
        }
        UiPredicateV1::NotExists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_none()
        }
        UiPredicateV1::FocusIs { target } => {
            let Some(focus) = snapshot.focus else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.id == focus
        }
        UiPredicateV1::RoleIs { target, role } => {
            let Some(want) = parse_semantics_role(role) else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.role == want
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked == Some(*checked)
        }
        UiPredicateV1::CheckedIsNone { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked.is_none()
        }
        UiPredicateV1::BarrierRoots {
            barrier_root,
            focus_barrier_root,
            require_equal,
        } => {
            let barrier = snapshot.barrier_root.map(|n| n.data().as_ffi());
            let focus_barrier = snapshot.focus_barrier_root.map(|n| n.data().as_ffi());

            let matches_root_state = |state: UiOptionalRootStateV1, value: Option<u64>| match state
            {
                UiOptionalRootStateV1::Any => true,
                UiOptionalRootStateV1::None => value.is_none(),
                UiOptionalRootStateV1::Some => value.is_some(),
            };

            if !matches_root_state(*barrier_root, barrier) {
                return false;
            }
            if !matches_root_state(*focus_barrier_root, focus_barrier) {
                return false;
            }

            match require_equal {
                None => true,
                Some(true) => barrier == focus_barrier,
                Some(false) => barrier != focus_barrier,
            }
        }
        UiPredicateV1::VisibleInWindow { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            rects_intersect(node.bounds, window_bounds)
        }
        UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            let bounds = node.bounds;
            let pad = padding_px.max(0.0);
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad;
            let window_top = window_bounds.origin.y.0 + pad;
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0 - pad;
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0 - pad;

            let node_left = bounds.origin.x.0;
            let node_top = bounds.origin.y.0;
            let node_right = bounds.origin.x.0 + bounds.size.width.0;
            let node_bottom = bounds.origin.y.0 + bounds.size.height.0;

            node_left >= window_left - eps
                && node_top >= window_top - eps
                && node_right <= window_right + eps
                && node_bottom <= window_bottom + eps
        }
        UiPredicateV1::BoundsMinSize {
            target,
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };

            let w = node.bounds.size.width.0.max(0.0);
            let h = node.bounds.size.height.0.max(0.0);

            let min_w = min_w_px.max(0.0);
            let min_h = min_h_px.max(0.0);
            let eps = eps_px.max(0.0);

            w + eps >= min_w && h + eps >= min_h
        }
        UiPredicateV1::BoundsNonOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            !(overlap_w > eps && overlap_h > eps)
        }
        UiPredicateV1::BoundsOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            overlap_w > eps && overlap_h > eps
        }
        UiPredicateV1::BoundsOverlappingX { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            overlap_w > eps
        }
        UiPredicateV1::BoundsOverlappingY { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ay0 = a.bounds.origin.y.0;
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let by0 = b.bounds.origin.y.0;
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);
            overlap_h > eps
        }
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0.max(0.0);
    let ay1 = ay0 + a.size.height.0.max(0.0);

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0.max(0.0);
    let by1 = by0 + b.size.height.0.max(0.0);

    ax1 > bx0 && bx1 > ax0 && ay1 > by0 && by1 > ay0
}

fn center_of_rect(rect: Rect) -> Point {
    let x = rect.origin.x + rect.size.width * 0.5;
    let y = rect.origin.y + rect.size.height * 0.5;
    Point::new(x, y)
}

fn center_of_rect_clamped_to_rect(rect: Rect, clamp: Rect) -> Point {
    if !rects_intersect(rect, clamp) {
        return center_of_rect(rect);
    }

    let rx0 = rect.origin.x.0;
    let ry0 = rect.origin.y.0;
    let rx1 = rx0 + rect.size.width.0.max(0.0);
    let ry1 = ry0 + rect.size.height.0.max(0.0);

    let cx0 = clamp.origin.x.0;
    let cy0 = clamp.origin.y.0;
    let cx1 = cx0 + clamp.size.width.0.max(0.0);
    let cy1 = cy0 + clamp.size.height.0.max(0.0);

    let ix0 = rx0.max(cx0);
    let iy0 = ry0.max(cy0);
    let ix1 = rx1.min(cx1);
    let iy1 = ry1.min(cy1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return center_of_rect(rect);
    }

    Point::new(
        fret_core::Px((ix0 + ix1) * 0.5),
        fret_core::Px((iy0 + iy1) * 0.5),
    )
}

fn pick_semantics_node_at<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    position: Point,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);

    let hit = ui.debug_hit_test(position).hit;
    if let Some(hit) = hit {
        let mut cur = Some(hit.data().as_ffi());
        while let Some(id) = cur {
            if index.is_selectable(id)
                && let Some(node) = index.by_id.get(&id).copied()
            {
                return Some(node);
            }
            cur = index
                .by_id
                .get(&id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
    }

    pick_semantics_node_by_bounds(snapshot, position)
}

pub(crate) fn pick_semantics_node_by_bounds<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    position: Point,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    pick_best_match(
        snapshot.nodes.iter().filter(|n| {
            let id = n.id.data().as_ffi();
            index.is_selectable(id) && n.bounds.contains(position)
        }),
        &index,
    )
}

fn suggest_selectors(
    snapshot: &fret_core::SemanticsSnapshot,
    raw_node: &fret_core::SemanticsNode,
    exported_node: &UiSemanticsNodeV1,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Vec<UiSelectorV1> {
    let mut out = Vec::new();

    if let Some(id) = raw_node.test_id.as_deref() {
        out.push(UiSelectorV1::TestId { id: id.to_string() });
    }

    let role = semantics_role_label(raw_node.role).to_string();
    if let Some(name) = exported_node.label.as_deref() {
        if !(cfg.redact_text && is_redacted_string(name)) {
            let ancestors = selector_ancestors_for(snapshot, raw_node);
            if !ancestors.is_empty() {
                out.push(UiSelectorV1::RoleAndPath {
                    role: role.clone(),
                    name: name.to_string(),
                    ancestors,
                });
            }
            out.push(UiSelectorV1::RoleAndName {
                role: role.clone(),
                name: name.to_string(),
            });
        }
    }

    if let Some(element) = element {
        out.push(UiSelectorV1::GlobalElementId { element });
    }

    out.push(UiSelectorV1::NodeId {
        node: raw_node.id.data().as_ffi(),
    });
    out
}

fn best_selector_for_node(
    snapshot: &fret_core::SemanticsSnapshot,
    raw_node: &fret_core::SemanticsNode,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Option<UiSelectorV1> {
    let exported =
        UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
    suggest_selectors(snapshot, raw_node, &exported, element, cfg)
        .into_iter()
        .next()
}

fn parent_node_id(snapshot: &fret_core::SemanticsSnapshot, node: u64) -> Option<u64> {
    let n = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == node)?;
    n.parent.map(|p| p.data().as_ffi())
}

fn truncate_debug_value(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut out = s[..max_bytes.min(s.len())].to_string();
    out.push('…');
    out
}

fn format_inspect_path(
    snapshot: &fret_core::SemanticsSnapshot,
    focus_node_id: u64,
    redact_text: bool,
    max_parts: usize,
) -> Option<String> {
    if max_parts == 0 {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    let mut cur: Option<u64> = Some(focus_node_id);
    while let Some(id) = cur {
        let Some(node) = snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id) else {
            break;
        };

        let role = semantics_role_label(node.role);
        let mut part = role.to_string();
        if let Some(test_id) = node.test_id.as_deref() {
            part.push('[');
            part.push_str(&truncate_debug_value(test_id, 32));
            part.push(']');
        } else if !redact_text && let Some(label) = node.label.as_deref() {
            part.push('(');
            part.push_str(&truncate_debug_value(label, 32));
            part.push(')');
        }
        parts.push(part);

        cur = node.parent.map(|p| p.data().as_ffi());
    }
    if parts.is_empty() {
        return None;
    }
    parts.reverse();

    if parts.len() > max_parts {
        parts = parts.split_off(parts.len() - max_parts);
        parts.insert(0, "…".to_string());
    }

    Some(format!("path: {}", parts.join(" > ")))
}

fn selector_ancestors_for(
    snapshot: &fret_core::SemanticsSnapshot,
    node: &fret_core::SemanticsNode,
) -> Vec<UiRoleAndNameV1> {
    let index = SemanticsIndex::new(snapshot);
    let mut rev: Vec<UiRoleAndNameV1> = Vec::new();

    let mut cur = node
        .parent
        .and_then(|p| index.by_id.get(&p.data().as_ffi()).copied());
    while let Some(n) = cur {
        if let Some(label) = n.label.as_deref() {
            rev.push(UiRoleAndNameV1 {
                role: semantics_role_label(n.role).to_string(),
                name: label.to_string(),
            });
        }
        cur = n
            .parent
            .and_then(|p| index.by_id.get(&p.data().as_ffi()).copied());
    }

    rev.reverse();
    rev
}

fn is_redacted_string(s: &str) -> bool {
    s.trim_start().starts_with("<redacted")
}

fn parse_semantics_numeric_value(value: &str) -> Option<f32> {
    let s = value.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(raw) = s.strip_suffix('%') {
        return raw.trim().parse::<f32>().ok();
    }
    if let Ok(v) = s.parse::<f32>() {
        return Some(v);
    }

    // Best-effort: extract the first float-ish token from the string.
    let mut token = String::new();
    let mut started = false;
    for ch in s.chars() {
        let keep = ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+');
        if keep {
            token.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if token.is_empty() {
        return None;
    }
    token.parse::<f32>().ok()
}

fn move_pointer_event(position: Point) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    })
}

fn wheel_event(position: Point, delta_x: f32, delta_y: f32) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    Event::Pointer(PointerEvent::Wheel {
        pointer_id,
        position,
        delta: Point::new(fret_core::Px(delta_x), fret_core::Px(delta_y)),
        modifiers,
        pointer_type,
    })
}

fn click_events(position: Point, button: UiMouseButtonV1, click_count: u8) -> [Event; 3] {
    click_events_with_modifiers(position, button, click_count, Modifiers::default())
}

fn click_events_with_modifiers(
    position: Point,
    button: UiMouseButtonV1,
    click_count: u8,
    modifiers: Modifiers,
) -> [Event; 3] {
    let pointer_id = PointerId(0);
    let pointer_type = PointerType::Mouse;
    let click_count = click_count.max(1);

    let move_event = Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    });
    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };
    let down = Event::Pointer(PointerEvent::Down {
        pointer_id,
        position,
        button,
        modifiers,
        click_count,
        pointer_type,
    });
    let up = Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button,
        modifiers,
        is_click: true,
        click_count,
        pointer_type,
    });

    [move_event, down, up]
}

fn drag_events(start: Point, end: Point, button: UiMouseButtonV1, steps: u32) -> Vec<Event> {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };

    let mut pressed_buttons = MouseButtons::default();
    match button {
        MouseButton::Left => pressed_buttons.left = true,
        MouseButton::Right => pressed_buttons.right = true,
        MouseButton::Middle => pressed_buttons.middle = true,
        _ => {}
    }

    let mut out = Vec::with_capacity(3 + steps as usize);
    out.push(Event::Pointer(PointerEvent::Move {
        pointer_id,
        position: start,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    }));
    out.push(Event::Pointer(PointerEvent::Down {
        pointer_id,
        position: start,
        button,
        modifiers,
        click_count: 1,
        pointer_type,
    }));

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x = start.x.0 + (end.x.0 - start.x.0) * t;
        let y = start.y.0 + (end.y.0 - start.y.0) * t;
        let position = Point::new(fret_core::Px(x), fret_core::Px(y));
        out.push(Event::Pointer(PointerEvent::Move {
            pointer_id,
            position,
            buttons: pressed_buttons,
            modifiers,
            pointer_type,
        }));

        // For scripted diagnostics, also emit `InternalDrag` events during pointer drags. The
        // runtime routes these to the active internal-drag anchor when a cross-window drag session
        // is active (e.g. docking tear-off / drop indicators).
        //
        // This is intentionally safe for generic scripts: `UiTree` ignores `InternalDrag` events
        // unless `app.drag(pointer_id)` exists and is marked `cross_window_hover`.
        out.push(Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id,
            position,
            kind: fret_core::InternalDragKind::Over,
            modifiers,
        }));
    }

    out.push(Event::Pointer(PointerEvent::Up {
        pointer_id,
        position: end,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type,
    }));

    // Mirror the runner's "mouse-up routes a drop then clears hover" behavior for internal drags.
    out.push(Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position: end,
        kind: fret_core::InternalDragKind::Drop,
        modifiers,
    }));
    out
}

fn push_drag_playback_frame(state: &mut V2DragPointerState, events: &mut Vec<Event>) -> bool {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();
    let pointer_type = PointerType::Mouse;

    let (button, pressed_buttons) = match state.button {
        UiMouseButtonV1::Left => (
            MouseButton::Left,
            MouseButtons {
                left: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Right => (
            MouseButton::Right,
            MouseButtons {
                right: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Middle => (
            MouseButton::Middle,
            MouseButtons {
                middle: true,
                ..Default::default()
            },
        ),
    };

    let steps = state.steps.max(1);
    let final_frame = steps.saturating_add(1);

    match state.frame {
        0 => {
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position: state.start,
                buttons: MouseButtons::default(),
                modifiers,
                pointer_type,
            }));
            events.push(Event::Pointer(PointerEvent::Down {
                pointer_id,
                position: state.start,
                button,
                modifiers,
                click_count: 1,
                pointer_type,
            }));
            state.frame = 1;
            false
        }
        f if (1..=steps).contains(&f) => {
            let t = f as f32 / steps as f32;
            let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
            let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
            let position = Point::new(fret_core::Px(x), fret_core::Px(y));
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position,
                buttons: pressed_buttons,
                modifiers,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position,
                kind: fret_core::InternalDragKind::Over,
                modifiers,
            }));
            state.frame = state.frame.saturating_add(1);
            false
        }
        f if f >= final_frame => {
            events.push(Event::Pointer(PointerEvent::Up {
                pointer_id,
                position: state.end,
                button,
                modifiers,
                is_click: false,
                click_count: 1,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position: state.end,
                kind: fret_core::InternalDragKind::Drop,
                modifiers,
            }));
            true
        }
        _ => true,
    }
}

fn press_key_events(key: KeyCode, modifiers: UiKeyModifiersV1, repeat: bool) -> [Event; 2] {
    let modifiers = core_modifiers_from_ui(Some(modifiers));
    let down = Event::KeyDown {
        key,
        modifiers,
        repeat,
    };
    let up = Event::KeyUp { key, modifiers };
    [down, up]
}

fn core_modifiers_from_ui(modifiers: Option<UiKeyModifiersV1>) -> Modifiers {
    let modifiers = modifiers.unwrap_or_default();
    Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.meta,
        ..Modifiers::default()
    }
}

fn parse_shortcut(shortcut: &str) -> Option<(KeyCode, UiKeyModifiersV1)> {
    let mut parts = shortcut
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    let key = parts.pop()?;

    let mut modifiers = UiKeyModifiersV1::default();
    for part in parts {
        match part.to_ascii_lowercase().as_str() {
            "shift" => modifiers.shift = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            "alt" => modifiers.alt = true,
            "meta" | "cmd" | "command" | "super" => modifiers.meta = true,
            "primary" => {
                if cfg!(target_os = "macos") {
                    modifiers.meta = true;
                } else {
                    modifiers.ctrl = true;
                }
            }
            _ => return None,
        }
    }

    Some((parse_key_code(key)?, modifiers))
}

fn rect_inset(rect: Rect, insets: UiPaddingInsetsV1) -> Rect {
    let left = Px(insets.left_px.max(0.0));
    let top = Px(insets.top_px.max(0.0));
    let right = Px(insets.right_px.max(0.0));
    let bottom = Px(insets.bottom_px.max(0.0));

    let origin = Point {
        x: rect.origin.x + left,
        y: rect.origin.y + top,
    };
    let w = (rect.size.width.0 - left.0 - right.0).max(0.0);
    let h = (rect.size.height.0 - top.0 - bottom.0).max(0.0);
    Rect {
        origin,
        size: fret_core::Size {
            width: Px(w),
            height: Px(h),
        },
    }
}

fn rect_fully_contains(outer: Rect, inner: Rect) -> bool {
    let ox0 = outer.origin.x.0;
    let oy0 = outer.origin.y.0;
    let ox1 = ox0 + outer.size.width.0;
    let oy1 = oy0 + outer.size.height.0;

    let ix0 = inner.origin.x.0;
    let iy0 = inner.origin.y.0;
    let ix1 = ix0 + inner.size.width.0;
    let iy1 = iy0 + inner.size.height.0;

    ix0 >= ox0 && iy0 >= oy0 && ix1 <= ox1 && iy1 <= oy1
}

fn parse_key_code(key: &str) -> Option<KeyCode> {
    let key = key.trim().to_ascii_lowercase();
    match key.as_str() {
        "escape" | "esc" => Some(KeyCode::Escape),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Space),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        "arrow_up" | "up" => Some(KeyCode::ArrowUp),
        "arrow_down" | "down" => Some(KeyCode::ArrowDown),
        "arrow_left" | "left" => Some(KeyCode::ArrowLeft),
        "arrow_right" | "right" => Some(KeyCode::ArrowRight),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "page_up" => Some(KeyCode::PageUp),
        "page_down" => Some(KeyCode::PageDown),
        _ => {
            if key.len() == 1 {
                return Some(match key.as_bytes()[0] {
                    b'a' => KeyCode::KeyA,
                    b'b' => KeyCode::KeyB,
                    b'c' => KeyCode::KeyC,
                    b'd' => KeyCode::KeyD,
                    b'e' => KeyCode::KeyE,
                    b'f' => KeyCode::KeyF,
                    b'g' => KeyCode::KeyG,
                    b'h' => KeyCode::KeyH,
                    b'i' => KeyCode::KeyI,
                    b'j' => KeyCode::KeyJ,
                    b'k' => KeyCode::KeyK,
                    b'l' => KeyCode::KeyL,
                    b'm' => KeyCode::KeyM,
                    b'n' => KeyCode::KeyN,
                    b'o' => KeyCode::KeyO,
                    b'p' => KeyCode::KeyP,
                    b'q' => KeyCode::KeyQ,
                    b'r' => KeyCode::KeyR,
                    b's' => KeyCode::KeyS,
                    b't' => KeyCode::KeyT,
                    b'u' => KeyCode::KeyU,
                    b'v' => KeyCode::KeyV,
                    b'w' => KeyCode::KeyW,
                    b'x' => KeyCode::KeyX,
                    b'y' => KeyCode::KeyY,
                    b'z' => KeyCode::KeyZ,
                    b'0' => KeyCode::Digit0,
                    b'1' => KeyCode::Digit1,
                    b'2' => KeyCode::Digit2,
                    b'3' => KeyCode::Digit3,
                    b'4' => KeyCode::Digit4,
                    b'5' => KeyCode::Digit5,
                    b'6' => KeyCode::Digit6,
                    b'7' => KeyCode::Digit7,
                    b'8' => KeyCode::Digit8,
                    b'9' => KeyCode::Digit9,
                    _ => return None,
                });
            }
            None
        }
    }
}

fn key_to_u64(key: NodeId) -> u64 {
    key.data().as_ffi()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(value).unwrap_or_default();
    std::fs::write(path, bytes)
}

fn truncate_string_bytes(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    if max_bytes == 0 {
        s.clear();
        return;
    }

    let suffix = "...";
    if max_bytes <= suffix.len() {
        let mut idx = max_bytes;
        while idx > 0 && !s.is_char_boundary(idx) {
            idx -= 1;
        }
        s.truncate(idx);
        return;
    }

    let mut idx = max_bytes - suffix.len();
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    s.truncate(idx);
    s.push_str(suffix);
}

fn write_latest_pointer(out_dir: &Path, export_dir: &Path) -> Result<(), std::io::Error> {
    let path = out_dir.join("latest.txt");
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let rel = export_dir.strip_prefix(out_dir).unwrap_or(export_dir);
    std::fs::write(path, rel.to_string_lossy().as_bytes())
}

fn touch_file(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(f, "{}", unix_ms_now())?;
    let _ = f.flush();
    Ok(())
}

fn screenshot_request_completed(path: &Path, request_id: &str, window_ffi: u64) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let Ok(root) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(completed) = root.get("completed").and_then(|v| v.as_array()) else {
        return false;
    };
    completed.iter().any(|entry| {
        entry.get("request_id").and_then(|v| v.as_str()) == Some(request_id)
            && entry.get("window").and_then(|v| v.as_u64()) == Some(window_ffi)
    })
}

fn display_path(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.to_string_lossy().to_string()
}

fn maybe_redact_string(s: &str, redact_text: bool) -> String {
    if !redact_text {
        return s.to_string();
    }
    format!("<redacted len={}>", s.len())
}

fn sanitize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            out.push(c);
        } else if matches!(c, ' ' | ':' | '/' | '\\') {
            out.push('_');
        }
    }
    if out.is_empty() {
        "bundle".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        AppWindowId, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRole,
        SemanticsRoot, SemanticsSnapshot, Size,
    };
    use fret_diag_protocol::UiActionStepV1;
    use slotmap::KeyData;

    #[test]
    fn parse_key_code_supports_function_keys() {
        assert_eq!(parse_key_code("f1"), Some(KeyCode::F1));
        assert_eq!(parse_key_code("f10"), Some(KeyCode::F10));
        assert_eq!(parse_key_code("F12"), Some(KeyCode::F12));
    }

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn window_id(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn semantics_node(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
    ) -> SemanticsNode {
        SemanticsNode {
            id: node_id(id),
            parent: parent.map(node_id),
            role,
            bounds,
            flags: SemanticsFlags::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: Some(label.to_string()),
            value: None,
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
        }
    }

    fn semantics_node_with_test_id(
        id: u64,
        parent: Option<u64>,
        role: SemanticsRole,
        bounds: Rect,
        label: &str,
        test_id: &str,
    ) -> SemanticsNode {
        let mut n = semantics_node(id, parent, role, bounds, label);
        n.test_id = Some(test_id.to_string());
        n
    }

    #[test]
    fn scripts_do_not_force_inspection_active() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = false;
        svc.pick_armed_run_id = None;
        svc.pending_pick = None;
        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        svc.cfg.pick_trigger_path = dir.join("pick.touch");
        svc.cfg.inspect_trigger_path = dir.join("inspect.touch");
        svc.cfg.inspect_path = dir.join("inspect.json");
        svc.pending_script = Some(PendingScript { steps: Vec::new() });

        assert!(
            !svc.wants_inspection_active(AppWindowId::default()),
            "scripts should not force inspection_active (allows view cache/paint cache during perf triage)"
        );
    }

    #[test]
    fn scripts_support_reset_diagnostics_step() {
        let parsed: UiActionScriptV1 =
            serde_json::from_str(r#"{"schema_version":1,"steps":[{"type":"reset_diagnostics"}]}"#)
                .expect("parse reset_diagnostics step");
        assert_eq!(parsed.schema_version, 1);
        assert!(
            matches!(parsed.steps.as_slice(), [UiActionStepV1::ResetDiagnostics]),
            "expected reset_diagnostics step"
        );
    }

    #[test]
    fn scripts_support_schema_v2_intent_steps() {
        let parsed: UiActionScriptV2 = serde_json::from_str(
            r#"{"schema_version":2,"steps":[{"type":"ensure_visible","target":{"kind":"test_id","id":"x"}}]}"#,
        )
        .expect("parse schema v2 script");
        assert_eq!(parsed.schema_version, 2);
        assert!(
            matches!(
                parsed.steps.as_slice(),
                [UiActionStepV2::EnsureVisible { .. }]
            ),
            "expected ensure_visible step"
        );
    }

    #[test]
    fn scripts_support_move_pointer_sweep_step() {
        let parsed: UiActionScriptV2 = serde_json::from_str(
            r#"{"schema_version":2,"steps":[{"type":"move_pointer_sweep","target":{"kind":"test_id","id":"x"},"delta_x":10.0,"delta_y":-5.0,"steps":3,"frames_per_step":2}]}"#,
        )
        .expect("parse move_pointer_sweep step");
        assert_eq!(parsed.schema_version, 2);
        assert!(
            matches!(
                parsed.steps.as_slice(),
                [UiActionStepV2::MovePointerSweep { .. }]
            ),
            "expected move_pointer_sweep step"
        );
    }

    #[test]
    fn pick_trigger_is_baselined_on_first_poll() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.pick_armed_run_id = None;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.pick_trigger_path = dir.join("pick.touch");
        std::fs::write(&svc.cfg.pick_trigger_path, []).expect("create pick.touch");

        svc.last_pick_trigger_mtime = None;
        svc.poll_pick_trigger();

        assert!(
            svc.pick_armed_run_id.is_none(),
            "the first observed pick.touch mtime should be baselined, not treated as a pick trigger"
        );
        assert!(svc.last_pick_trigger_mtime.is_some());
    }

    #[test]
    fn inspect_trigger_is_baselined_on_first_poll() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = false;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.inspect_trigger_path = dir.join("inspect.touch");
        svc.cfg.inspect_path = dir.join("inspect.json");
        std::fs::write(&svc.cfg.inspect_trigger_path, []).expect("create inspect.touch");

        svc.last_inspect_trigger_mtime = None;
        svc.poll_inspect_trigger();

        assert!(
            !svc.inspect_enabled,
            "the first observed inspect.touch mtime should be baselined, not treated as an inspect trigger"
        );
        assert!(svc.last_inspect_trigger_mtime.is_some());
    }

    #[test]
    fn pick_by_bounds_prefers_topmost_root_z() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-a",
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "a",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-b",
                ),
                semantics_node(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "b",
                ),
            ],
        };

        let picked = pick_semantics_node_by_bounds(&snapshot, Point::new(Px(10.0), Px(10.0)))
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));
    }

    #[test]
    fn select_by_test_id_prefers_topmost_root_z() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-a",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "a",
                    "open",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root-b",
                ),
                semantics_node_with_test_id(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "b",
                    "open",
                ),
            ],
        };

        let selector = UiSelectorV1::TestId {
            id: "open".to_string(),
        };
        let picked = select_semantics_node(&snapshot, window_id(1), None, &selector)
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));

        let cfg = UiDiagnosticsConfig::default();
        let best = best_selector_for_node(&snapshot, &snapshot.nodes[1], None, &cfg)
            .expect("expected a selector");
        match best {
            UiSelectorV1::TestId { id } => assert_eq!(id, "open"),
            other => panic!("expected TestId selector, got: {other:?}"),
        }
    }

    #[test]
    fn bounds_within_window_predicate_respects_padding() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "content",
                    "content",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsWithinWindow {
            target: UiSelectorV1::TestId {
                id: "content".to_string(),
            },
            padding_px: 0.0,
            eps_px: 0.0,
        };
        assert!(eval_predicate(
            &snapshot,
            window_bounds,
            window_id(1),
            None,
            &pred
        ));

        let pred = UiPredicateV1::BoundsWithinWindow {
            target: UiSelectorV1::TestId {
                id: "content".to_string(),
            },
            padding_px: 12.0,
            eps_px: 0.0,
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected padding to shrink the allowed window rect"
        );
    }

    #[test]
    fn bounds_min_size_predicate_accepts_large_enough_nodes() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node_with_test_id(
                1,
                None,
                SemanticsRole::Panel,
                rect(10.0, 10.0, 320.0, 240.0),
                "resizable",
                "ui-gallery-resizable-panels",
            )],
        };

        let pred = UiPredicateV1::BoundsMinSize {
            target: UiSelectorV1::TestId {
                id: "ui-gallery-resizable-panels".to_string(),
            },
            min_w_px: 200.0,
            min_h_px: 200.0,
            eps_px: 0.0,
        };

        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected node to satisfy the min-size gate"
        );
    }

    #[test]
    fn bounds_min_size_predicate_rejects_collapsed_nodes() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node_with_test_id(
                1,
                None,
                SemanticsRole::Panel,
                rect(10.0, 10.0, 320.0, 0.1),
                "resizable",
                "ui-gallery-resizable-panels",
            )],
        };

        let pred = UiPredicateV1::BoundsMinSize {
            target: UiSelectorV1::TestId {
                id: "ui-gallery-resizable-panels".to_string(),
            },
            min_w_px: 200.0,
            min_h_px: 200.0,
            eps_px: 0.0,
        };

        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "collapsed node should fail the min-size gate"
        );
    }

    #[test]
    fn bounds_non_overlapping_predicate_rejects_intersection() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 10.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsNonOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected overlap (a right edge > b left edge) to fail"
        );

        let pred = UiPredicateV1::BoundsNonOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 16.0,
        };
        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected eps_px to tolerate a small overlap"
        );
    }

    #[test]
    fn not_exists_predicate_matches_absence() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Panel,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let pred = UiPredicateV1::NotExists {
            target: UiSelectorV1::TestId {
                id: "missing".to_string(),
            },
        };
        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected missing test id to satisfy NotExists"
        );
    }

    #[test]
    fn bounds_overlapping_predicate_requires_intersection() {
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 10.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected overlap (a right edge > b left edge) to pass"
        );

        let pred = UiPredicateV1::BoundsOverlapping {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 16.0,
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected eps_px to require more overlap than available"
        );
    }

    #[test]
    fn bounds_overlapping_x_predicate_ignores_y() {
        let window_bounds = rect(0.0, 0.0, 100.0, 200.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 100.0, 200.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(25.0, 150.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlappingX {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected x overlap to pass even when y does not overlap"
        );

        let pred = UiPredicateV1::BoundsOverlappingX {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 8.0,
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected eps_px to require more x overlap than available"
        );
    }

    #[test]
    fn bounds_overlapping_y_predicate_ignores_x() {
        let window_bounds = rect(0.0, 0.0, 200.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 100.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(10.0, 10.0, 20.0, 20.0),
                    "a",
                    "a",
                ),
                semantics_node_with_test_id(
                    3,
                    Some(1),
                    SemanticsRole::Panel,
                    rect(150.0, 25.0, 20.0, 20.0),
                    "b",
                    "b",
                ),
            ],
        };

        let pred = UiPredicateV1::BoundsOverlappingY {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 0.0,
        };
        assert!(
            eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected y overlap to pass even when x does not overlap"
        );

        let pred = UiPredicateV1::BoundsOverlappingY {
            a: UiSelectorV1::TestId {
                id: "a".to_string(),
            },
            b: UiSelectorV1::TestId {
                id: "b".to_string(),
            },
            eps_px: 8.0,
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window_id(1), None, &pred),
            "expected eps_px to require more y overlap than available"
        );
    }

    #[test]
    fn inspect_focus_shortcut_locks_to_semantics_focus() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(node_id(2)),
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "root",
                ),
                semantics_node_with_test_id(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "focus",
                    "focused-btn",
                ),
            ],
        };

        let window = window_id(1);
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.inspect_enabled = true;

        svc.inspect_pending_nav
            .insert(window, InspectNavCommand::Focus);
        svc.apply_inspect_navigation(window, Some(&snapshot), None);

        assert!(svc.inspect_is_locked(window));
        let focus_id = snapshot.focus.expect("focus").data().as_ffi();
        assert_eq!(svc.inspect_focus_node_id(window), Some(focus_id));
        assert!(
            svc.inspect_best_selector_json(window)
                .is_some_and(|s| s.contains("test_id"))
        );
    }

    #[test]
    fn pick_by_bounds_respects_modal_barrier() {
        let snapshot = SemanticsSnapshot {
            window: window_id(1),
            roots: vec![
                SemanticsRoot {
                    root: node_id(1),
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: node_id(3),
                    visible: true,
                    blocks_underlay_input: true,
                    hit_testable: true,
                    z_index: 10,
                },
            ],
            barrier_root: Some(node_id(3)),
            focus_barrier_root: Some(node_id(3)),
            focus: None,
            captured: None,
            nodes: vec![
                semantics_node(
                    1,
                    None,
                    SemanticsRole::Panel,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "underlay",
                ),
                semantics_node(
                    2,
                    Some(1),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "underlay-button",
                ),
                semantics_node(
                    3,
                    None,
                    SemanticsRole::Dialog,
                    rect(0.0, 0.0, 200.0, 200.0),
                    "modal",
                ),
                semantics_node(
                    4,
                    Some(3),
                    SemanticsRole::Button,
                    rect(0.0, 0.0, 100.0, 100.0),
                    "modal-button",
                ),
            ],
        };

        let picked = pick_semantics_node_by_bounds(&snapshot, Point::new(Px(10.0), Px(10.0)))
            .expect("expected a pick");
        assert_eq!(picked.id, node_id(4));
    }

    #[test]
    fn scripts_can_assert_barrier_root_and_focus_barrier_root_independently() {
        let window = window_id(1);
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: Some(node_id(1)),
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Window,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let pred = UiPredicateV1::BarrierRoots {
            barrier_root: UiOptionalRootStateV1::Some,
            focus_barrier_root: UiOptionalRootStateV1::None,
            require_equal: Some(false),
        };

        assert!(
            eval_predicate(&snapshot, window_bounds, window, None, &pred),
            "expected scripts to assert that the pointer barrier can remain active while focus containment is released"
        );

        let pred = UiPredicateV1::BarrierRoots {
            barrier_root: UiOptionalRootStateV1::Some,
            focus_barrier_root: UiOptionalRootStateV1::None,
            require_equal: Some(true),
        };
        assert!(
            !eval_predicate(&snapshot, window_bounds, window, None, &pred),
            "expected require_equal=true to fail when the roots differ"
        );
    }

    #[test]
    fn scripts_can_assert_barrier_roots_via_drive_script() {
        let mut svc = UiDiagnosticsService::default();
        svc.cfg.enabled = true;
        svc.cfg.script_auto_dump = false;

        let unique = fret_core::time::SystemTime::now()
            .duration_since(fret_core::time::UNIX_EPOCH)
            .expect("system clock should be >= UNIX_EPOCH")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fret-diag-test-script-{}", unique));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        svc.cfg.out_dir = dir.clone();
        svc.cfg.ready_path = dir.join("ready.touch");
        svc.cfg.script_path = dir.join("script.json");
        svc.cfg.script_trigger_path = dir.join("script.touch");
        svc.cfg.script_result_path = dir.join("script.result.json");
        svc.cfg.script_result_trigger_path = dir.join("script.result.touch");

        let window = AppWindowId::default();
        let window_bounds = rect(0.0, 0.0, 100.0, 100.0);
        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: true,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: Some(node_id(1)),
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![semantics_node(
                1,
                None,
                SemanticsRole::Window,
                rect(0.0, 0.0, 100.0, 100.0),
                "root",
            )],
        };

        let script: UiActionScriptV1 = serde_json::from_str(
            r#"{
                "schema_version": 1,
                "steps": [
                    {
                        "type": "assert",
                        "predicate": {
                            "kind": "barrier_roots",
                            "barrier_root": "some",
                            "focus_barrier_root": "none",
                            "require_equal": false
                        }
                    }
                ]
            }"#,
        )
        .expect("parse barrier_roots predicate");
        svc.pending_script = PendingScript::from_v1(script);
        assert!(
            svc.pending_script.is_some(),
            "script schema_version should be valid"
        );
        svc.pending_script_run_id = Some(1);

        let app = App::new();
        let _ = svc.drive_script_for_window(
            &app,
            window,
            window_bounds,
            1.0,
            None,
            Some(&snapshot),
            None,
        );

        let bytes =
            std::fs::read(&svc.cfg.script_result_path).expect("read script result json file");
        let result: UiScriptResultV1 =
            serde_json::from_slice(&bytes).expect("parse UiScriptResultV1");
        assert!(
            matches!(result.stage, UiScriptStageV1::Passed),
            "expected drive_script to persist the passed result"
        );
    }

    #[test]
    fn hit_test_snapshot_exposes_focus_barrier_root() {
        let position = Point::new(Px(1.0), Px(2.0));
        let hit_test = UiDebugHitTest {
            hit: None,
            active_layer_roots: vec![node_id(10)],
            barrier_root: Some(node_id(10)),
        };

        let snap = UiHitTestSnapshotV1::from_hit_test_with_layers(
            position,
            hit_test,
            Some(node_id(11)),
            &[],
        );

        assert_eq!(snap.barrier_root, Some(key_to_u64(node_id(10))));
        assert_eq!(snap.focus_barrier_root, Some(key_to_u64(node_id(11))));
        assert!(
            snap.scope_roots
                .iter()
                .any(|r| r.kind == "modal_barrier_root" && r.root == key_to_u64(node_id(10)))
        );
        assert!(
            snap.scope_roots
                .iter()
                .any(|r| { r.kind == "focus_barrier_root" && r.root == key_to_u64(node_id(11)) })
        );
    }
}

fn sanitize_path_for_bundle(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

trait PointerEventExt {
    fn kind(&self) -> &'static str;
    fn position(&self) -> Point;
}

impl PointerEventExt for fret_core::PointerEvent {
    fn kind(&self) -> &'static str {
        match self {
            fret_core::PointerEvent::Down { .. } => "down",
            fret_core::PointerEvent::Up { .. } => "up",
            fret_core::PointerEvent::Move { .. } => "move",
            fret_core::PointerEvent::Wheel { .. } => "wheel",
            fret_core::PointerEvent::PinchGesture { .. } => "pinch_gesture",
        }
    }

    fn position(&self) -> Point {
        match self {
            fret_core::PointerEvent::Down { position, .. } => *position,
            fret_core::PointerEvent::Up { position, .. } => *position,
            fret_core::PointerEvent::Move { position, .. } => *position,
            fret_core::PointerEvent::Wheel { position, .. } => *position,
            fret_core::PointerEvent::PinchGesture { position, .. } => *position,
        }
    }
}

trait EventPointerExt {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent>;
}

impl EventPointerExt for Event {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent> {
        match self {
            Event::Pointer(p) => Some(p),
            _ => None,
        }
    }
}
