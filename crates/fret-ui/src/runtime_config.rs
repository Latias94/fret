use std::ffi::OsStr;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeNodeProfileConfig {
    pub(crate) top_n: usize,
    pub(crate) min_elapsed: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeScrollLayoutProfileConfig {
    pub(crate) min_elapsed: Duration,
    pub(crate) min_self_measure: Duration,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTaffyDumpConfig {
    pub(crate) max: Option<u32>,
    pub(crate) root_filter: Option<String>,
    pub(crate) root_label_filter: Option<String>,
    pub(crate) out_dir: String,
}

#[derive(Debug)]
pub(crate) struct UiRuntimeEnvConfig {
    pub(crate) interactive_resize_text_width_cache_entries: usize,
    pub(crate) keep_alive_view_cache_scratch_disabled: bool,

    pub(crate) scroll_layout_profile: Option<RuntimeScrollLayoutProfileConfig>,
    pub(crate) scroll_defer_unbounded_probe_on_resize: bool,
    pub(crate) scroll_defer_unbounded_probe_on_invalidation: bool,
    pub(crate) scroll_defer_unbounded_probe_stable_frames: u8,

    pub(crate) debug_pointer_region_move_hook: bool,
    pub(crate) debug_pointer_region_move_backtrace: bool,
    pub(crate) debug_scroll_wheel_vlist: bool,
    pub(crate) debug_scroll_wheel: bool,
    pub(crate) debug_scroll_handle_set_offset: bool,
    pub(crate) resizable_split_log: bool,

    pub(crate) hit_test_bounds_tree_disabled: bool,
    pub(crate) hit_test_bounds_tree_min_records: usize,

    pub(crate) paint_cache_relax_view_cache_gating: bool,
    pub(crate) paint_cache_allow_hit_test_only: bool,

    pub(crate) validate_semantics: bool,
    pub(crate) validate_semantics_panic: bool,

    pub(crate) layout_all_profile: bool,
    pub(crate) layout_profile: bool,
    pub(crate) layout_engine_sweep_policy: LayoutEngineSweepPolicy,
    pub(crate) layout_skip_request_build_translation_only: bool,
    pub(crate) layout_flow_skip_barrier_clean_children: bool,
    pub(crate) debug_focus_repair: bool,
    pub(crate) taffy_dump: Option<RuntimeTaffyDumpConfig>,
    pub(crate) taffy_dump_once: bool,
    pub(crate) layout_forbid_widget_fallback_solves: bool,
    pub(crate) layout_trace_widget_fallback_solves: bool,

    pub(crate) layout_node_profile: Option<RuntimeNodeProfileConfig>,
    pub(crate) measure_node_profile: Option<RuntimeNodeProfileConfig>,

    pub(crate) debug_interactivity_gate_sync: bool,
    pub(crate) debug_hit_test_gate_sync: bool,
    pub(crate) debug_focus_traversal_gate_sync: bool,
    pub(crate) debug_pointer_down_outside: bool,

    pub(crate) semantics_profile: bool,

    pub(crate) interactive_resize_stable_frames_required: u8,
    pub(crate) text_wrap_width_bucket_px: u8,
    pub(crate) text_wrap_width_small_step_bucket_px: u8,
    pub(crate) text_wrap_width_small_step_max_dw_px: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutEngineSweepPolicy {
    Always,
    OnDemand,
}

pub(crate) fn ui_runtime_config() -> &'static UiRuntimeEnvConfig {
    static CONFIG: OnceLock<UiRuntimeEnvConfig> = OnceLock::new();
    CONFIG.get_or_init(UiRuntimeEnvConfig::from_env)
}

impl UiRuntimeEnvConfig {
    fn from_env() -> Self {
        let interactive_resize_text_width_cache_entries =
            std::env::var("FRET_UI_INTERACTIVE_RESIZE_TEXT_WIDTH_CACHE_ENTRIES")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(2)
                .min(8);

        let keep_alive_view_cache_scratch_disabled =
            env_enabled("FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE");

        let scroll_layout_profile = env_is_one("FRET_SCROLL_LAYOUT_PROFILE").then(|| {
            let min_us = std::env::var("FRET_SCROLL_LAYOUT_PROFILE_MIN_US")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(2_000);
            let min_measure_us = std::env::var("FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1_000);
            RuntimeScrollLayoutProfileConfig {
                min_elapsed: Duration::from_micros(min_us),
                min_self_measure: Duration::from_micros(min_measure_us),
            }
        });

        // Default-on for interactive resize/viewport churn. Set to "0" to disable.
        let scroll_defer_unbounded_probe_on_resize =
            std::env::var("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_RESIZE")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(true);

        // Default-on: unbounded scroll probing can trigger deep measure walks when descendant
        // layout invalidations are present (common under view-cache reconciliation). Deferring the
        // probe by a few frames (when we have prior size data) helps avoid tail spikes on those
        // invalidation frames. Set to "0" to disable.
        let scroll_defer_unbounded_probe_on_invalidation =
            std::env::var("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(true);

        let scroll_defer_unbounded_probe_stable_frames =
            std::env::var("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES")
                .ok()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(2)
                .min(60);

        let debug_pointer_region_move_hook = env_present("FRET_DEBUG_POINTER_REGION_MOVE_HOOK");
        let debug_pointer_region_move_backtrace =
            env_present("FRET_DEBUG_POINTER_REGION_MOVE_BACKTRACE");

        let debug_scroll_wheel_vlist = env_is_one("FRET_DEBUG_SCROLL_WHEEL_VLIST");
        let debug_scroll_wheel = env_is_one("FRET_DEBUG_SCROLL_WHEEL");
        let debug_scroll_handle_set_offset = env_enabled("FRET_DEBUG_SCROLL_HANDLE_SET_OFFSET");

        let resizable_split_log = env_present("FRET_RESIZABLE_SPLIT_LOG");

        let hit_test_bounds_tree_disabled = env_non_empty("FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE");
        let hit_test_bounds_tree_min_records =
            std::env::var("FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(256)
                .max(1);

        let paint_cache_relax_view_cache_gating =
            env_non_empty("FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING");
        let paint_cache_allow_hit_test_only =
            env_non_empty("FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY");

        let validate_semantics = env_present("FRET_VALIDATE_SEMANTICS");
        let validate_semantics_panic = env_present("FRET_VALIDATE_SEMANTICS_PANIC");

        let layout_all_profile = env_enabled("FRET_LAYOUT_ALL_PROFILE");
        let layout_profile = env_enabled("FRET_LAYOUT_PROFILE");
        let layout_engine_sweep_policy = match std::env::var("FRET_UI_LAYOUT_ENGINE_SWEEP")
            .ok()
            .as_deref()
        {
            Some("always") => LayoutEngineSweepPolicy::Always,
            Some("on_demand") => LayoutEngineSweepPolicy::OnDemand,
            Some(v) => {
                tracing::warn!(
                    "invalid FRET_UI_LAYOUT_ENGINE_SWEEP={v:?} (expected always|on_demand); defaulting to on_demand"
                );
                LayoutEngineSweepPolicy::OnDemand
            }
            None => LayoutEngineSweepPolicy::OnDemand,
        };
        // Default-on: these are perf knobs intended to keep the request/build phase cheap on
        // steady-state frames. Set to "0" to disable.
        let layout_skip_request_build_translation_only =
            std::env::var("FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(true);
        let layout_flow_skip_barrier_clean_children =
            std::env::var("FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(true);
        let debug_focus_repair = env_present("FRET_DEBUG_FOCUS_REPAIR");

        let taffy_dump = env_present("FRET_TAFFY_DUMP").then(|| RuntimeTaffyDumpConfig {
            max: std::env::var("FRET_TAFFY_DUMP_MAX")
                .ok()
                .and_then(|s| s.parse().ok()),
            root_filter: std::env::var("FRET_TAFFY_DUMP_ROOT").ok(),
            root_label_filter: std::env::var("FRET_TAFFY_DUMP_ROOT_LABEL").ok(),
            out_dir: std::env::var("FRET_TAFFY_DUMP_DIR")
                .ok()
                .unwrap_or_else(|| ".fret/taffy-dumps".to_string()),
        });
        let taffy_dump_once = env_is_one("FRET_TAFFY_DUMP_ONCE");

        let layout_forbid_widget_fallback_solves =
            env_present("FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES");
        let layout_trace_widget_fallback_solves =
            env_present("FRET_LAYOUT_TRACE_WIDGET_FALLBACK_SOLVES");

        let layout_node_profile = env_is_one("FRET_LAYOUT_NODE_PROFILE").then(|| {
            let top_n = std::env::var("FRET_LAYOUT_NODE_PROFILE_TOP")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(16)
                .clamp(1, 128);
            let min_us = std::env::var("FRET_LAYOUT_NODE_PROFILE_MIN_US")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(500);
            RuntimeNodeProfileConfig {
                top_n,
                min_elapsed: Duration::from_micros(min_us),
            }
        });
        let measure_node_profile = env_is_one("FRET_MEASURE_NODE_PROFILE").then(|| {
            let top_n = std::env::var("FRET_MEASURE_NODE_PROFILE_TOP")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(16)
                .clamp(1, 128);
            let min_us = std::env::var("FRET_MEASURE_NODE_PROFILE_MIN_US")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(500);
            RuntimeNodeProfileConfig {
                top_n,
                min_elapsed: Duration::from_micros(min_us),
            }
        });

        let debug_interactivity_gate_sync = env_present("FRET_DEBUG_INTERACTIVITY_GATE_SYNC");
        let debug_hit_test_gate_sync = env_present("FRET_DEBUG_HIT_TEST_GATE_SYNC");
        let debug_focus_traversal_gate_sync = env_present("FRET_DEBUG_FOCUS_TRAVERSAL_GATE_SYNC");
        let debug_pointer_down_outside = env_present("FRET_DEBUG_POINTER_DOWN_OUTSIDE");

        let semantics_profile = env_enabled("FRET_SEMANTICS_PROFILE");

        let interactive_resize_stable_frames_required =
            std::env::var("FRET_UI_INTERACTIVE_RESIZE_STABLE_FRAMES")
                .ok()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(2)
                .min(60);

        let text_wrap_width_bucket_px = std::env::var("FRET_UI_TEXT_WRAP_WIDTH_BUCKET_PX")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0)
            .min(64);

        let text_wrap_width_small_step_bucket_px =
            std::env::var("FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_BUCKET_PX")
                .ok()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(32)
                .min(64);

        let text_wrap_width_small_step_max_dw_px =
            std::env::var("FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_MAX_DW_PX")
                .ok()
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(64)
                .clamp(1, 255);

        Self {
            interactive_resize_text_width_cache_entries,
            keep_alive_view_cache_scratch_disabled,
            scroll_layout_profile,
            scroll_defer_unbounded_probe_on_resize,
            scroll_defer_unbounded_probe_on_invalidation,
            scroll_defer_unbounded_probe_stable_frames,
            debug_pointer_region_move_hook,
            debug_pointer_region_move_backtrace,
            debug_scroll_wheel_vlist,
            debug_scroll_wheel,
            debug_scroll_handle_set_offset,
            resizable_split_log,
            hit_test_bounds_tree_disabled,
            hit_test_bounds_tree_min_records,
            paint_cache_relax_view_cache_gating,
            paint_cache_allow_hit_test_only,
            validate_semantics,
            validate_semantics_panic,
            layout_all_profile,
            layout_profile,
            layout_engine_sweep_policy,
            layout_skip_request_build_translation_only,
            layout_flow_skip_barrier_clean_children,
            debug_focus_repair,
            taffy_dump,
            taffy_dump_once,
            layout_forbid_widget_fallback_solves,
            layout_trace_widget_fallback_solves,
            layout_node_profile,
            measure_node_profile,
            debug_interactivity_gate_sync,
            debug_hit_test_gate_sync,
            debug_focus_traversal_gate_sync,
            debug_pointer_down_outside,
            semantics_profile,
            interactive_resize_stable_frames_required,
            text_wrap_width_bucket_px,
            text_wrap_width_small_step_bucket_px,
            text_wrap_width_small_step_max_dw_px,
        }
    }
}

fn env_present(name: &str) -> bool {
    std::env::var_os(name).is_some()
}

fn env_non_empty(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty())
}

fn env_enabled(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty() && v != OsStr::new("0"))
}

fn env_is_one(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}
