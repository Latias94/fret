use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use fret::{FretApp, advanced::prelude::*};
use fret_core::{AppWindowId, Color, FontWeight, Px, TextAlign, TextOverflow, TextStyle, TextWrap};
use fret_render::{Renderer, RendererPerfFrameStore, WgpuContext};
use fret_runtime::{
    RunnerFrameDriveDiagnosticsStore, RunnerPresentDiagnosticsStore,
    RunnerSurfaceConfigDiagnosticsStore, WindowGlobalChangeDiagnosticsStore,
    WindowRedrawRequestDiagnosticsStore,
};
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, TextProps};
use fret_ui_kit::declarative::scheduling::set_continuous_frames;
use serde_json::json;

const TEST_ID_ROOT: &str = "hello_world_compare.root";
const TEST_ID_TITLE: &str = "hello_world_compare.title";
const TEST_ID_SWATCH_ROW: &str = "hello_world_compare.swatch_row";
const INTERNAL_GPU_REPORT_BASENAME: &str = "hello_world_compare.internal_gpu.json";

static PROCESS_LAUNCH_AT: OnceLock<Instant> = OnceLock::new();
static PROCESS_LAUNCH_UNIX_MS: OnceLock<u64> = OnceLock::new();
static COMPARE_WINDOW_ID: OnceLock<Mutex<Option<AppWindowId>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareActiveMode {
    Idle,
    PresentOnly,
    RerenderOnly,
    PaintModel,
    LayoutModel,
}

impl CompareActiveMode {
    fn from_env(legacy_continuous_redraw: bool) -> Self {
        let Some(raw) = env_string("FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE") else {
            return if legacy_continuous_redraw {
                Self::PresentOnly
            } else {
                Self::Idle
            };
        };
        match raw.trim().to_ascii_lowercase().as_str() {
            "idle" => Self::Idle,
            "present" | "present-only" | "present_only" => Self::PresentOnly,
            "rerender" | "rerender-only" | "rerender_only" | "render-only" | "render_only" => {
                Self::RerenderOnly
            }
            "paint" | "paint-model" | "paint_model" => Self::PaintModel,
            "layout" | "layout-model" | "layout_model" => Self::LayoutModel,
            other => {
                eprintln!(
                    "hello_world_compare active_mode_invalid value={other} default={} ",
                    if legacy_continuous_redraw {
                        Self::PresentOnly.as_str()
                    } else {
                        Self::Idle.as_str()
                    }
                );
                if legacy_continuous_redraw {
                    Self::PresentOnly
                } else {
                    Self::Idle
                }
            }
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::PresentOnly => "present-only",
            Self::RerenderOnly => "rerender-only",
            Self::PaintModel => "paint-model",
            Self::LayoutModel => "layout-model",
        }
    }

    fn is_active(self) -> bool {
        !matches!(self, Self::Idle)
    }

    fn uses_continuous_frames_lease(self) -> bool {
        matches!(self, Self::PresentOnly)
    }

    fn uses_animation_frame_loop(self) -> bool {
        matches!(
            self,
            Self::RerenderOnly | Self::PaintModel | Self::LayoutModel
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct CompareFlags {
    no_text: bool,
    no_swatches: bool,
    legacy_continuous_redraw: bool,
    active_mode: CompareActiveMode,
}

impl CompareFlags {
    fn from_env() -> Self {
        let legacy_continuous_redraw = env_flag("FRET_HELLO_WORLD_COMPARE_CONTINUOUS_REDRAW");
        Self {
            no_text: env_flag("FRET_HELLO_WORLD_COMPARE_NO_TEXT"),
            no_swatches: env_flag("FRET_HELLO_WORLD_COMPARE_NO_SWATCHES"),
            legacy_continuous_redraw,
            active_mode: CompareActiveMode::from_env(legacy_continuous_redraw),
        }
    }

    fn is_active(self) -> bool {
        self.active_mode.is_active()
    }

    fn uses_continuous_frames_lease(self) -> bool {
        self.active_mode.uses_continuous_frames_lease()
    }

    fn uses_animation_frame_loop(self) -> bool {
        self.active_mode.uses_animation_frame_loop()
    }
}

#[derive(Debug, Clone, Copy)]
struct CompareWindowConfig {
    width_px: f64,
    height_px: f64,
}

impl CompareWindowConfig {
    fn from_env() -> Self {
        Self {
            width_px: env_f64("FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH")
                .unwrap_or(500.0)
                .max(1.0),
            height_px: env_f64("FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT")
                .unwrap_or(500.0)
                .max(1.0),
        }
    }
}

#[derive(Debug, Clone)]
struct InternalGpuSamplingConfig {
    report_path: PathBuf,
    sample_at_secs: Vec<f64>,
    top_n: usize,
}

#[derive(Debug, Clone, Default)]
struct RuntimeFrameSampleState {
    render_count: u64,
    last_frame_id: u64,
    last_render_since_launch_ms: u64,
    element_runtime: Option<ElementRuntimeFrameSample>,
}

#[derive(Debug, Clone, Default)]
struct ElementRuntimeFrameSample {
    wants_continuous_frames: bool,
    continuous_frame_leases: Vec<ElementRuntimeContinuousFrameLeaseSample>,
    animation_frame_request_roots: Vec<ElementRuntimeAnimationFrameRequestRootSample>,
    nodes_count: u64,
    timer_targets_count: u64,
}

#[derive(Debug, Clone)]
struct ElementRuntimeContinuousFrameLeaseSample {
    element: u64,
    count: u32,
    debug_path: Option<String>,
}

#[derive(Debug, Clone)]
struct ElementRuntimeAnimationFrameRequestRootSample {
    element: u64,
    debug_path: Option<String>,
}

static RUNTIME_FRAME_SAMPLE_STATE: OnceLock<Mutex<RuntimeFrameSampleState>> = OnceLock::new();

fn runtime_frame_sample_state() -> &'static Mutex<RuntimeFrameSampleState> {
    RUNTIME_FRAME_SAMPLE_STATE.get_or_init(|| Mutex::new(RuntimeFrameSampleState::default()))
}

fn compare_window_id_state() -> &'static Mutex<Option<AppWindowId>> {
    COMPARE_WINDOW_ID.get_or_init(|| Mutex::new(None))
}

fn remember_compare_window(window: AppWindowId) {
    let mut slot = compare_window_id_state()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    *slot = Some(window);
}

fn compare_window_id() -> Option<AppWindowId> {
    compare_window_id_state()
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .as_ref()
        .copied()
}

fn internal_gpu_report_path() -> Option<PathBuf> {
    if let Some(path) = env_string("FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH") {
        return Some(PathBuf::from(path));
    }
    if env_flag("FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_TO_DIAG_DIR") {
        let diag_dir = env_string("FRET_DIAG_DIR")?;
        return Some(PathBuf::from(diag_dir).join(INTERNAL_GPU_REPORT_BASENAME));
    }
    None
}

fn runtime_sampling_enabled() -> bool {
    internal_gpu_report_path().is_some()
}

fn current_since_launch_ms() -> u64 {
    process_launch_at().elapsed().as_millis() as u64
}

fn capture_element_runtime_frame_sample(
    app: &mut KernelApp,
    window: AppWindowId,
) -> Option<ElementRuntimeFrameSample> {
    let snapshot = app
        .with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |runtime, _app| {
            runtime.diagnostics_snapshot(window)
        })?;
    Some(ElementRuntimeFrameSample {
        wants_continuous_frames: snapshot.wants_continuous_frames,
        continuous_frame_leases: snapshot
            .continuous_frame_leases
            .into_iter()
            .map(|entry| ElementRuntimeContinuousFrameLeaseSample {
                element: entry.element.0,
                count: entry.count,
                debug_path: entry.debug_path,
            })
            .collect(),
        animation_frame_request_roots: snapshot
            .animation_frame_request_roots
            .into_iter()
            .map(|entry| ElementRuntimeAnimationFrameRequestRootSample {
                element: entry.element.0,
                debug_path: entry.debug_path,
            })
            .collect(),
        nodes_count: snapshot.nodes_count,
        timer_targets_count: snapshot.timer_targets_count,
    })
}

fn update_runtime_frame_sample_state(cx: &mut ViewCx<'_, '_, KernelApp>) {
    if !runtime_sampling_enabled() {
        return;
    }
    let window = cx.window;
    let mut state = runtime_frame_sample_state()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    state.render_count = state.render_count.saturating_add(1);
    state.last_frame_id = cx.app.frame_id().0;
    state.last_render_since_launch_ms = current_since_launch_ms();
    state.element_runtime = capture_element_runtime_frame_sample(cx.app, window);
}

fn capture_runtime_frame_sample_json(
    runner_present_store: &RunnerPresentDiagnosticsStore,
    runner_frame_drive_store: &RunnerFrameDriveDiagnosticsStore,
    redraw_request_store: &WindowRedrawRequestDiagnosticsStore,
    global_change_store: &WindowGlobalChangeDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let state = runtime_frame_sample_state()
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    let mut obj = serde_json::Map::new();
    obj.insert("render_count".into(), json!(state.render_count));
    obj.insert("last_frame_id".into(), json!(state.last_frame_id));
    obj.insert(
        "last_render_since_launch_ms".into(),
        json!(state.last_render_since_launch_ms),
    );
    obj.insert(
        "element_runtime".into(),
        state
            .element_runtime
            .as_ref()
            .map(|snapshot| {
                json!({
                    "wants_continuous_frames": snapshot.wants_continuous_frames,
                    "continuous_frame_leases": snapshot.continuous_frame_leases.iter().map(|entry| {
                        json!({
                            "element": entry.element,
                            "count": entry.count,
                            "debug_path": entry.debug_path,
                        })
                    }).collect::<Vec<_>>(),
                    "animation_frame_request_roots": snapshot.animation_frame_request_roots.iter().map(|entry| {
                        json!({
                            "element": entry.element,
                            "debug_path": entry.debug_path,
                        })
                    }).collect::<Vec<_>>(),
                    "nodes_count": snapshot.nodes_count,
                    "timer_targets_count": snapshot.timer_targets_count,
                })
            })
            .unwrap_or(serde_json::Value::Null),
    );
    obj.insert(
        "runner_present".into(),
        capture_runner_present_sample_json(runner_present_store, launch_unix_ms),
    );
    obj.insert(
        "runner_frame_drive".into(),
        capture_runner_frame_drive_sample_json(runner_frame_drive_store, launch_unix_ms),
    );
    obj.insert(
        "redraw_requests".into(),
        capture_redraw_request_sample_json(redraw_request_store, launch_unix_ms),
    );
    obj.insert(
        "global_changes".into(),
        capture_global_change_sample_json(global_change_store, launch_unix_ms),
    );
    serde_json::Value::Object(obj)
}

fn capture_runner_present_sample_json(
    runner_present_store: &RunnerPresentDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let snapshot = runner_present_store.aggregate_snapshot();
    let Some(last_present_unix_ms) = snapshot.last_present_unix_ms else {
        return json!({
            "present": false,
            "window_count": snapshot.window_count,
            "total_present_count": snapshot.total_present_count,
            "max_present_count": snapshot.max_present_count,
            "last_present_frame_id": snapshot.last_present_frame_id,
        });
    };
    json!({
        "present": true,
        "window_count": snapshot.window_count,
        "total_present_count": snapshot.total_present_count,
        "max_present_count": snapshot.max_present_count,
        "last_present_frame_id": snapshot.last_present_frame_id,
        "last_present_unix_ms": last_present_unix_ms,
        "last_present_since_launch_ms": last_present_unix_ms.saturating_sub(launch_unix_ms),
    })
}

fn capture_runner_frame_drive_sample_json(
    runner_frame_drive_store: &RunnerFrameDriveDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let snapshot = runner_frame_drive_store.aggregate_snapshot();
    let mut reason_counts = serde_json::Map::new();
    for entry in &snapshot.reason_counts {
        reason_counts.insert(entry.reason.as_str().into(), json!(entry.count));
    }
    let Some(last_event_unix_ms) = snapshot.last_event_unix_ms else {
        return json!({
            "present": false,
            "window_count": snapshot.window_count,
            "total_event_count": snapshot.total_event_count,
            "max_event_count": snapshot.max_event_count,
            "last_event_frame_id": snapshot.last_event_frame_id,
            "reason_counts": reason_counts,
        });
    };
    json!({
        "present": true,
        "window_count": snapshot.window_count,
        "total_event_count": snapshot.total_event_count,
        "max_event_count": snapshot.max_event_count,
        "last_event_frame_id": snapshot.last_event_frame_id,
        "last_event_unix_ms": last_event_unix_ms,
        "last_event_since_launch_ms": last_event_unix_ms.saturating_sub(launch_unix_ms),
        "reason_counts": reason_counts,
    })
}

fn capture_redraw_request_sample_json(
    redraw_request_store: &WindowRedrawRequestDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let snapshot = redraw_request_store.aggregate_snapshot();
    let callsites = snapshot
        .callsites
        .iter()
        .map(|entry| {
            json!({
                "file": entry.file,
                "line": entry.line,
                "column": entry.column,
                "count": entry.count,
            })
        })
        .collect::<Vec<_>>();
    let Some(last_request_unix_ms) = snapshot.last_request_unix_ms else {
        return json!({
            "present": false,
            "window_count": snapshot.window_count,
            "total_request_count": snapshot.total_request_count,
            "max_request_count": snapshot.max_request_count,
            "last_request_frame_id": snapshot.last_request_frame_id,
            "callsites": callsites,
        });
    };
    json!({
        "present": true,
        "window_count": snapshot.window_count,
        "total_request_count": snapshot.total_request_count,
        "max_request_count": snapshot.max_request_count,
        "last_request_frame_id": snapshot.last_request_frame_id,
        "last_request_unix_ms": last_request_unix_ms,
        "last_request_since_launch_ms": last_request_unix_ms.saturating_sub(launch_unix_ms),
        "callsites": callsites,
    })
}

fn capture_global_change_sample_json(
    global_change_store: &WindowGlobalChangeDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let snapshot = global_change_store.aggregate_snapshot();
    let globals = snapshot
        .globals
        .iter()
        .map(|entry| {
            json!({
                "name": entry.name,
                "count": entry.count,
            })
        })
        .collect::<Vec<_>>();
    let Some(last_unix_ms) = snapshot.last_unix_ms else {
        return json!({
            "present": false,
            "window_count": snapshot.window_count,
            "batch_count": snapshot.batch_count,
            "total_global_count": snapshot.total_global_count,
            "max_batch_count": snapshot.max_batch_count,
            "last_frame_id": snapshot.last_frame_id,
            "globals": globals,
        });
    };
    json!({
        "present": true,
        "window_count": snapshot.window_count,
        "batch_count": snapshot.batch_count,
        "total_global_count": snapshot.total_global_count,
        "max_batch_count": snapshot.max_batch_count,
        "last_frame_id": snapshot.last_frame_id,
        "last_unix_ms": last_unix_ms,
        "last_since_launch_ms": last_unix_ms.saturating_sub(launch_unix_ms),
        "globals": globals,
    })
}

fn capture_surface_config_sample_json(
    surface_config_store: &RunnerSurfaceConfigDiagnosticsStore,
    launch_unix_ms: u64,
) -> serde_json::Value {
    let window = compare_window_id();
    let Some(window) = window else {
        return json!({
            "present": false,
            "window_known": false,
        });
    };
    let Some(snapshot) = surface_config_store.window_snapshot(window) else {
        return json!({
            "present": false,
            "window_known": true,
        });
    };
    json!({
        "present": true,
        "window_known": true,
        "width_px": snapshot.width_px,
        "height_px": snapshot.height_px,
        "format": snapshot.format,
        "present_mode": snapshot.present_mode,
        "desired_maximum_frame_latency": snapshot.desired_maximum_frame_latency,
        "alpha_mode": snapshot.alpha_mode,
        "configure_count": snapshot.configure_count,
        "last_configure_frame_id": snapshot.last_configure_frame_id,
        "last_configure_unix_ms": snapshot.last_configure_unix_ms,
        "last_configure_since_launch_ms": snapshot
            .last_configure_unix_ms
            .map(|unix_ms| unix_ms.saturating_sub(launch_unix_ms)),
    })
}

fn capture_renderer_perf_sample_json(
    renderer_perf_store: &RendererPerfFrameStore,
) -> serde_json::Value {
    let window = compare_window_id();
    let Some(window) = window else {
        return json!({
            "present": false,
            "window_known": false,
        });
    };
    let Some(snapshot) = renderer_perf_store.latest_for_window(window) else {
        return json!({
            "present": false,
            "window_known": true,
        });
    };
    let perf = snapshot.perf;
    let mut out = serde_json::Map::new();
    out.extend(
        json!({
            "present": true,
            "window_known": true,
            "tick_id": snapshot.tick_id,
            "frame_id": snapshot.frame_id,
            "frames": perf.frames,
            "gpu_images_live": perf.gpu_images_live,
            "gpu_images_bytes_estimate": perf.gpu_images_bytes_estimate,
            "gpu_images_max_bytes_estimate": perf.gpu_images_max_bytes_estimate,
            "gpu_render_targets_live": perf.gpu_render_targets_live,
            "gpu_render_targets_bytes_estimate": perf.gpu_render_targets_bytes_estimate,
            "gpu_render_targets_max_bytes_estimate": perf.gpu_render_targets_max_bytes_estimate,
        })
        .as_object()
        .expect("renderer perf json object")
        .clone(),
    );
    out.extend(
        json!({
            "intermediate_budget_bytes": perf.intermediate_budget_bytes,
            "intermediate_full_target_bytes": perf.intermediate_full_target_bytes,
            "intermediate_in_use_bytes": perf.intermediate_in_use_bytes,
            "intermediate_peak_in_use_bytes": perf.intermediate_peak_in_use_bytes,
            "intermediate_release_targets": perf.intermediate_release_targets,
            "intermediate_pool_allocations": perf.intermediate_pool_allocations,
            "intermediate_pool_reuses": perf.intermediate_pool_reuses,
            "intermediate_pool_releases": perf.intermediate_pool_releases,
            "intermediate_pool_evictions": perf.intermediate_pool_evictions,
            "intermediate_pool_free_bytes": perf.intermediate_pool_free_bytes,
            "intermediate_pool_free_textures": perf.intermediate_pool_free_textures,
            "path_intermediate_bytes_estimate": perf.path_intermediate_bytes_estimate,
            "path_intermediate_msaa_bytes_estimate": perf.path_intermediate_msaa_bytes_estimate,
            "path_intermediate_resolved_bytes_estimate": perf.path_intermediate_resolved_bytes_estimate,
            "custom_effect_v3_pyramid_scratch_bytes_estimate": perf.custom_effect_v3_pyramid_scratch_bytes_estimate,
        })
        .as_object()
        .expect("renderer perf json object")
        .clone(),
    );
    out.extend(
        json!({
            "render_plan_estimated_peak_intermediate_bytes": perf.render_plan_estimated_peak_intermediate_bytes,
            "render_plan_segments": perf.render_plan_segments,
            "render_plan_segments_changed": perf.render_plan_segments_changed,
            "render_plan_segments_passes_increased": perf.render_plan_segments_passes_increased,
            "render_plan_degradations": perf.render_plan_degradations,
            "render_plan_effect_chain_budget_samples": perf.render_plan_effect_chain_budget_samples,
            "render_plan_effect_chain_other_live_max_bytes": perf.render_plan_effect_chain_other_live_max_bytes,
            "render_plan_custom_effect_chain_budget_samples": perf.render_plan_custom_effect_chain_budget_samples,
            "render_plan_custom_effect_chain_other_live_max_bytes": perf.render_plan_custom_effect_chain_other_live_max_bytes,
            "render_plan_custom_effect_chain_base_required_full_targets_max": perf.render_plan_custom_effect_chain_base_required_full_targets_max,
            "render_plan_custom_effect_chain_optional_pyramid_max_bytes": perf.render_plan_custom_effect_chain_optional_pyramid_max_bytes,
            "clip_path_mask_cache_bytes_live": perf.clip_path_mask_cache_bytes_live,
            "clip_path_mask_cache_entries_live": perf.clip_path_mask_cache_entries_live,
            "custom_effect_v3_pyramid_cache_hits": perf.custom_effect_v3_pyramid_cache_hits,
            "custom_effect_v3_pyramid_cache_misses": perf.custom_effect_v3_pyramid_cache_misses,
            "path_msaa_samples_requested": perf.path_msaa_samples_requested,
            "path_msaa_samples_effective": perf.path_msaa_samples_effective,
        })
        .as_object()
        .expect("renderer perf json object")
        .clone(),
    );
    out.extend(
        json!({
            "draw_calls": perf.draw_calls,
            "quad_draw_calls": perf.quad_draw_calls,
            "text_draw_calls": perf.text_draw_calls,
            "path_draw_calls": perf.path_draw_calls,
            "fullscreen_draw_calls": perf.fullscreen_draw_calls,
        })
        .as_object()
        .expect("renderer perf json object")
        .clone(),
    );
    serde_json::Value::Object(out)
}

fn env_flag(name: &str) -> bool {
    std::env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "enabled"
            )
        })
        .unwrap_or(false)
}

fn env_string(name: &str) -> Option<String> {
    std::env::var_os(name)
        .and_then(|value| value.into_string().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_usize(name: &str) -> Option<usize> {
    env_string(name).and_then(|value| value.parse::<usize>().ok())
}

fn env_u32(name: &str) -> Option<u32> {
    env_string(name).and_then(|value| value.parse::<u32>().ok())
}

fn env_f64(name: &str) -> Option<f64> {
    env_string(name).and_then(|value| value.parse::<f64>().ok())
}

fn pre_init_sleep_duration() -> Option<Duration> {
    env_f64("FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS")
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(Duration::from_secs_f64)
}

fn spawn_process_exit_after_secs_from_env() {
    let Some(exit_after_secs) = env_f64("FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS") else {
        return;
    };
    if !(exit_after_secs.is_finite() && exit_after_secs > 0.0) {
        eprintln!("hello_world_compare exit_after_secs_invalid value={exit_after_secs}");
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs_f64(exit_after_secs));
        eprintln!("hello_world_compare auto_exit after_secs={exit_after_secs:.3}");
        std::process::exit(0);
    });
}

fn sleep_before_app_init_from_env() {
    let Some(pre_init_sleep) = pre_init_sleep_duration() else {
        return;
    };
    eprintln!(
        "hello_world_compare pre_init_sleep_secs={:.3}",
        pre_init_sleep.as_secs_f64()
    );
    std::thread::sleep(pre_init_sleep);
}

fn process_launch_at() -> Instant {
    *PROCESS_LAUNCH_AT.get_or_init(Instant::now)
}

fn process_launch_unix_ms() -> u64 {
    *PROCESS_LAUNCH_UNIX_MS.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    })
}

fn parse_sample_at_secs(raw: &str) -> anyhow::Result<Vec<f64>> {
    let mut out = Vec::new();
    for piece in raw.split(',') {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let value = piece
            .parse::<f64>()
            .map_err(|err| anyhow::anyhow!("invalid sample offset `{piece}`: {err}"))?;
        if value < 0.0 {
            anyhow::bail!("sample offset must be >= 0, got {value}");
        }
        out.push(value);
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    if out.is_empty() {
        anyhow::bail!("no sample offsets configured");
    }
    Ok(out)
}

fn internal_gpu_sampling_config() -> Option<InternalGpuSamplingConfig> {
    let report_path = internal_gpu_report_path()?;
    let sample_at_secs = env_string("FRET_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS")
        .map(|raw| parse_sample_at_secs(&raw))
        .transpose()
        .unwrap_or_else(|err| {
            eprintln!("hello_world_compare internal_gpu_sampling_config_error={err}");
            None
        })
        .unwrap_or_else(|| vec![2.0, 6.0, 12.0]);
    let top_n = env_usize("FRET_HELLO_WORLD_COMPARE_INTERNAL_TOP_N")
        .unwrap_or(8)
        .max(1);
    Some(InternalGpuSamplingConfig {
        report_path,
        sample_at_secs,
        top_n,
    })
}

fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

fn write_internal_report(path: &Path, payload: &serde_json::Value) {
    ensure_parent_dir(path);
    if let Err(err) = std::fs::write(
        path,
        serde_json::to_vec_pretty(payload).unwrap_or_else(|_| b"{}".to_vec()),
    ) {
        eprintln!(
            "hello_world_compare internal_gpu_report_write_error path={} err={err}",
            path.display()
        );
    }
}

fn on_gpu_ready(app: &mut KernelApp, context: &WgpuContext, _renderer: &mut Renderer) {
    let Some(config) = internal_gpu_sampling_config() else {
        return;
    };

    let launch_at = process_launch_at();
    let launch_unix_ms = process_launch_unix_ms();
    let runner_present_store = app
        .with_global_mut_untracked(RunnerPresentDiagnosticsStore::default, |store, _app| {
            store.clone()
        });
    let runner_frame_drive_store = app
        .with_global_mut_untracked(RunnerFrameDriveDiagnosticsStore::default, |store, _app| {
            store.clone()
        });
    let redraw_request_store = app.with_global_mut_untracked(
        WindowRedrawRequestDiagnosticsStore::default,
        |store, _app| store.clone(),
    );
    let global_change_store = app.with_global_mut_untracked(
        WindowGlobalChangeDiagnosticsStore::default,
        |store, _app| store.clone(),
    );
    let surface_config_store = app.with_global_mut_untracked(
        RunnerSurfaceConfigDiagnosticsStore::default,
        |store, _app| store.clone(),
    );
    let renderer_perf_store =
        app.with_global_mut_untracked(RendererPerfFrameStore::default, |store, _app| store.clone());
    let instance = context.instance.clone();
    let device = context.device.clone();
    let adapter_info = context.adapter.get_info();
    let init_diagnostics = context.init_diagnostics.clone();
    let compare_flags = CompareFlags::from_env();
    let window = CompareWindowConfig::from_env();
    let diag_env_enabled_guess = [
        env_string("FRET_DIAG"),
        env_string("FRET_DIAG_DIR"),
        env_string("FRET_DIAG_CONFIG_PATH"),
    ]
    .into_iter()
    .any(|value| value.is_some());
    let requested_runtime_json = json!({
        "scene": {
            "no_text": compare_flags.no_text,
            "no_swatches": compare_flags.no_swatches,
            "active_mode": compare_flags.active_mode.as_str(),
            "active": compare_flags.is_active(),
            "continuous_redraw": compare_flags.uses_continuous_frames_lease(),
            "animation_frame_loop": compare_flags.uses_animation_frame_loop(),
            "legacy_continuous_redraw_env": compare_flags.legacy_continuous_redraw,
        },
        "startup": {
            "pre_init_sleep_secs": pre_init_sleep_duration()
                .map(|duration| duration.as_secs_f64()),
        },
        "window": {
            "width_px": window.width_px,
            "height_px": window.height_px,
        },
        "surface": {
            "present_mode_raw": env_string("FRET_RENDER_WGPU_SURFACE_PRESENT_MODE"),
            "desired_maximum_frame_latency": env_u32("FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY"),
        },
        "renderer": {
            "path_msaa_samples_override": env_u32("FRET_RENDER_WGPU_PATH_MSAA_SAMPLES"),
            "path_msaa_samples_effective": _renderer.path_msaa_samples(),
        },
        "diagnostics": {
            "fret_diag": env_string("FRET_DIAG"),
            "fret_diag_dir": env_string("FRET_DIAG_DIR"),
            "fret_diag_config_path": env_string("FRET_DIAG_CONFIG_PATH"),
            "diag_env_enabled_guess": diag_env_enabled_guess,
            "internal_report_path": config.report_path,
        },
    });

    std::thread::spawn(move || {
        let adapter_json = json!({
            "name": adapter_info.name,
            "backend": format!("{:?}", adapter_info.backend),
            "device_type": format!("{:?}", adapter_info.device_type),
            "driver": adapter_info.driver,
            "driver_info": adapter_info.driver_info,
        });
        let init_json = json!({
            "allow_fallback": init_diagnostics.allow_fallback,
            "requested_backend": init_diagnostics.requested_backend,
            "requested_backend_is_override": init_diagnostics.requested_backend_is_override,
            "attempts": init_diagnostics.attempts,
        });

        let mut samples = Vec::new();
        for &offset_secs in &config.sample_at_secs {
            let deadline = launch_at + Duration::from_secs_f64(offset_secs);
            let now = Instant::now();
            if deadline > now {
                std::thread::sleep(deadline.duration_since(now));
            }

            let sample = capture_internal_gpu_sample(
                &instance,
                &device,
                &surface_config_store,
                &renderer_perf_store,
                &runner_present_store,
                &runner_frame_drive_store,
                &redraw_request_store,
                &global_change_store,
                launch_unix_ms,
                offset_secs,
                config.top_n,
            );
            eprintln!(
                "hello_world_compare internal_gpu_sample offset_secs={offset_secs:.3} metal_current_allocated_size_bytes={} allocator_total_allocated_bytes={} hub_textures={} surface_configures={} renderer_gpu_images_bytes={} renderer_gpu_render_targets_bytes={} runner_present_total={} runner_frame_drive_total={} redraw_request_total={} global_change_batches={}",
                sample["allocator"]["metal_current_allocated_size_bytes"]
                    .as_u64()
                    .unwrap_or(0),
                sample["allocator"]["total_allocated_bytes"]
                    .as_u64()
                    .unwrap_or(0),
                sample["hub"]["textures"].as_u64().unwrap_or(0),
                sample["surface"]["configure_count"].as_u64().unwrap_or(0),
                sample["renderer_perf"]["gpu_images_bytes_estimate"]
                    .as_u64()
                    .unwrap_or(0),
                sample["renderer_perf"]["gpu_render_targets_bytes_estimate"]
                    .as_u64()
                    .unwrap_or(0),
                sample["runtime"]["runner_present"]["total_present_count"]
                    .as_u64()
                    .unwrap_or(0),
                sample["runtime"]["runner_frame_drive"]["total_event_count"]
                    .as_u64()
                    .unwrap_or(0),
                sample["runtime"]["redraw_requests"]["total_request_count"]
                    .as_u64()
                    .unwrap_or(0),
                sample["runtime"]["global_changes"]["batch_count"]
                    .as_u64()
                    .unwrap_or(0),
            );
            samples.push(sample);

            let payload = json!({
                "schema_version": 1,
                "kind": "hello_world_compare_internal_gpu_timeline",
                "process_launch_unix_ms": launch_unix_ms,
                "sample_at_secs": config.sample_at_secs,
                "adapter": adapter_json,
                "init_diagnostics": init_json,
                "requested_runtime": requested_runtime_json,
                "samples": samples,
            });
            write_internal_report(&config.report_path, &payload);
        }
    });
}

fn capture_internal_gpu_sample(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
    surface_config_store: &RunnerSurfaceConfigDiagnosticsStore,
    renderer_perf_store: &RendererPerfFrameStore,
    runner_present_store: &RunnerPresentDiagnosticsStore,
    runner_frame_drive_store: &RunnerFrameDriveDiagnosticsStore,
    redraw_request_store: &WindowRedrawRequestDiagnosticsStore,
    global_change_store: &WindowGlobalChangeDiagnosticsStore,
    launch_unix_ms: u64,
    offset_secs: f64,
    top_n: usize,
) -> serde_json::Value {
    let captured_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let hub_json = if let Some(report) = instance.generate_report() {
        let hub = report.hub_report();
        json!({
            "present": true,
            "adapters": (hub.adapters.num_allocated + hub.adapters.num_kept_from_user) as u64,
            "devices": (hub.devices.num_allocated + hub.devices.num_kept_from_user) as u64,
            "queues": (hub.queues.num_allocated + hub.queues.num_kept_from_user) as u64,
            "command_encoders": (hub.command_encoders.num_allocated + hub.command_encoders.num_kept_from_user) as u64,
            "buffers": (hub.buffers.num_allocated + hub.buffers.num_kept_from_user) as u64,
            "textures": (hub.textures.num_allocated + hub.textures.num_kept_from_user) as u64,
            "texture_views": (hub.texture_views.num_allocated + hub.texture_views.num_kept_from_user) as u64,
            "samplers": (hub.samplers.num_allocated + hub.samplers.num_kept_from_user) as u64,
            "shader_modules": (hub.shader_modules.num_allocated + hub.shader_modules.num_kept_from_user) as u64,
            "render_pipelines": (hub.render_pipelines.num_allocated + hub.render_pipelines.num_kept_from_user) as u64,
            "compute_pipelines": (hub.compute_pipelines.num_allocated + hub.compute_pipelines.num_kept_from_user) as u64,
        })
    } else {
        json!({ "present": false })
    };

    let allocator_report = device.generate_allocator_report();
    let metal_current_allocated_size_bytes = current_metal_allocated_size_bytes(device);
    let allocator_json = if let Some(report) = allocator_report {
        let allocation_count = report.allocations.len() as u64;
        let mut allocations = report.allocations;
        allocations.sort_unstable_by_key(|allocation| std::cmp::Reverse(allocation.size));
        allocations.truncate(top_n);
        let top_allocations = allocations
            .into_iter()
            .map(|allocation| {
                json!({
                    "name": allocation.name,
                    "size": allocation.size,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "present": true,
            "total_allocated_bytes": report.total_allocated_bytes,
            "total_reserved_bytes": report.total_reserved_bytes,
            "blocks": report.blocks.len() as u64,
            "allocations": allocation_count,
            "metal_current_allocated_size_bytes": metal_current_allocated_size_bytes,
            "top_allocations": top_allocations,
        })
    } else {
        json!({
            "present": false,
            "metal_current_allocated_size_bytes": metal_current_allocated_size_bytes,
            "top_allocations": [],
        })
    };

    json!({
        "offset_secs": offset_secs,
        "captured_unix_ms": captured_unix_ms,
        "captured_since_launch_ms": captured_unix_ms.saturating_sub(launch_unix_ms),
        "hub": hub_json,
        "allocator": allocator_json,
        "surface": capture_surface_config_sample_json(surface_config_store, launch_unix_ms),
        "renderer_perf": capture_renderer_perf_sample_json(renderer_perf_store),
        "runtime": capture_runtime_frame_sample_json(
            runner_present_store,
            runner_frame_drive_store,
            redraw_request_store,
            global_change_store,
            launch_unix_ms,
        ),
    })
}

#[cfg(target_os = "macos")]
fn current_metal_allocated_size_bytes(device: &wgpu::Device) -> Option<u64> {
    unsafe {
        device
            .as_hal::<wgpu::hal::api::Metal>()
            .map(|device| device.raw_device().current_allocated_size() as u64)
    }
}

#[cfg(not(target_os = "macos"))]
fn current_metal_allocated_size_bytes(_device: &wgpu::Device) -> Option<u64> {
    None
}

struct HelloWorldCompareView {
    flags: CompareFlags,
    frame_tick: u64,
}

impl View for HelloWorldCompareView {
    fn init(_app: &mut KernelApp, window: AppWindowId) -> Self {
        remember_compare_window(window);
        Self {
            flags: CompareFlags::from_env(),
            frame_tick: 0,
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements {
        set_continuous_frames(cx, self.flags.uses_continuous_frames_lease());
        if self.flags.uses_animation_frame_loop() {
            cx.request_animation_frame();
            self.frame_tick = self.frame_tick.wrapping_add(1);
        }
        update_runtime_frame_sample_state(cx);

        let title_color = Color::from_srgb_hex_rgb(0xffffff);
        let bg_wave = if matches!(self.flags.active_mode, CompareActiveMode::PaintModel) {
            ((self.frame_tick / 2) % 24) as u32
        } else {
            0
        };
        let panel_bg = Color::from_srgb_hex_rgb(
            ((0x50 + bg_wave) << 16) | ((0x50 + (bg_wave / 2)) << 8) | (0x50 + bg_wave),
        );
        let layout_probe_height_px =
            if matches!(self.flags.active_mode, CompareActiveMode::LayoutModel) {
                8.0 + (((self.frame_tick / 2) % 5) as f32 * 12.0)
            } else {
                8.0
            };

        let swatch = |cx: &mut ElementContext<'_, KernelApp>,
                      fill_rgb: u32,
                      border_rgb: u32|
         -> AnyElement {
            ui::container(|_cx| Vec::<AnyElement>::new())
                .w_px(Px(32.0))
                .h_px(Px(32.0))
                .bg(ColorRef::Color(Color::from_srgb_hex_rgb(fill_rgb)))
                .rounded(Radius::Md)
                .border_1()
                .border_color(ColorRef::Color(Color::from_srgb_hex_rgb(border_rgb)))
                .into_element(cx)
        };
        let layout_probe = ui::container(|_cx| Vec::<AnyElement>::new())
            .w_px(Px(160.0))
            .h_px(Px(layout_probe_height_px))
            .bg(ColorRef::Color(panel_bg))
            .rounded(Radius::Md)
            .into_element(cx);

        let title = (!self.flags.no_text).then(|| {
            cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::from("Hello, World!"),
                style: Some(TextStyle {
                    size: Px(24.0),
                    weight: FontWeight::SEMIBOLD,
                    ..Default::default()
                }),
                color: Some(title_color),
                align: TextAlign::Center,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                ink_overflow: Default::default(),
            })
            .test_id(TEST_ID_TITLE)
        });

        let swatch_row = (!self.flags.no_swatches).then(|| {
            ui::h_flex(|cx| {
                [
                    swatch(cx, 0xff0000, 0xffffff),
                    swatch(cx, 0x00ff00, 0xffffff),
                    swatch(cx, 0x0000ff, 0xffffff),
                    swatch(cx, 0xffff00, 0xffffff),
                    swatch(cx, 0x000000, 0xffffff),
                    swatch(cx, 0xffffff, 0x000000),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
            .test_id(TEST_ID_SWATCH_ROW)
        });

        let mut children = Vec::<AnyElement>::new();
        children.push(layout_probe);
        if let Some(title) = title {
            children.push(title);
        }
        if let Some(swatch_row) = swatch_row {
            children.push(swatch_row);
        }

        ui::v_flex(move |_cx| children)
            .w_full()
            .h_full()
            .gap(Space::N3)
            .items_center()
            .justify_center()
            .bg(ColorRef::Color(panel_bg))
            .into_element(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = process_launch_at();
    let _ = process_launch_unix_ms();
    spawn_process_exit_after_secs_from_env();
    sleep_before_app_init_from_env();
    let window = CompareWindowConfig::from_env();

    FretApp::new("hello-world-compare-demo")
        .minimal_defaults()
        .config_files(false)
        .window(
            "hello_world_compare_demo",
            (window.width_px, window.height_px),
        )
        .view::<HelloWorldCompareView>()?
        .on_gpu_ready(on_gpu_ready)
        .configure(|config| {
            config.accessibility_enabled = false;
        })
        .run()
        .map_err(anyhow::Error::from)
}
