use serde_json::{Map, Value};

pub(crate) const PERF_KEY_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_KEY_REGISTRY_KIND: &str = "perf_key_registry";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyUnit {
    Microseconds,
    Cycles,
    Count,
    Bytes,
}

impl PerfKeyUnit {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Microseconds => "us",
            Self::Cycles => "cycles",
            Self::Count => "count",
            Self::Bytes => "bytes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyKind {
    Timing,
    Counter,
}

impl PerfKeyKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Timing => "timing",
            Self::Counter => "counter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyScope {
    Frame,
    DerivedStats,
    PointerMove,
}

impl PerfKeyScope {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::DerivedStats => "derived_stats",
            Self::PointerMove => "pointer_move",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyAggregate {
    Max,
    P95,
}

impl PerfKeyAggregate {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::P95 => "p95",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PerfKeyTrace {
    pub(crate) event: &'static str,
    pub(crate) category: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PerfKey {
    pub(crate) key: &'static str,
    pub(crate) unit: PerfKeyUnit,
    pub(crate) kind: PerfKeyKind,
    pub(crate) scope: PerfKeyScope,
    pub(crate) suggested_aggregate: PerfKeyAggregate,
    pub(crate) trace: Option<PerfKeyTrace>,
}

impl PerfKey {
    pub(crate) const fn trace_event_name(self) -> &'static str {
        match self.trace {
            Some(trace) => trace.event,
            None => self.key,
        }
    }

    pub(crate) const fn trace_category_name(self) -> &'static str {
        match self.trace {
            Some(trace) => trace.category,
            None => "fret",
        }
    }
}

pub(crate) const TOTAL_TIME_US: PerfKey = trace_timing_key(
    "total_time_us",
    "fret.frame",
    "frame",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_TIME_US: PerfKey =
    trace_timing_key("layout_time_us", "layout", "layout", PerfKeyAggregate::P95);
pub(crate) const PREPAINT_TIME_US: PerfKey = trace_timing_key(
    "prepaint_time_us",
    "prepaint",
    "prepaint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_TIME_US: PerfKey =
    trace_timing_key("paint_time_us", "paint", "paint", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_TIME_US: PerfKey = trace_timing_key(
    "dispatch_time_us",
    "dispatch",
    "dispatch",
    PerfKeyAggregate::P95,
);
pub(crate) const HIT_TEST_TIME_US: PerfKey = trace_timing_key(
    "hit_test_time_us",
    "hit_test",
    "hit_test",
    PerfKeyAggregate::P95,
);
pub(crate) const UI_THREAD_CPU_TIME_US: PerfKey = trace_timing_key(
    "ui_thread_cpu_time_us",
    "ui_thread_cpu_time",
    "cpu",
    PerfKeyAggregate::P95,
);
pub(crate) const UI_THREAD_CPU_TOTAL_TIME_US: PerfKey =
    timing_key("ui_thread_cpu_total_time_us", PerfKeyAggregate::Max);
pub(crate) const UI_THREAD_CPU_CYCLE_TIME_DELTA_CYCLES: PerfKey = trace_counter_key(
    "ui_thread_cpu_cycle_time_delta_cycles",
    PerfKeyUnit::Cycles,
    "ui_thread_cpu_cycle_delta",
    "cpu",
    PerfKeyAggregate::P95,
);
pub(crate) const UI_THREAD_CPU_CYCLE_TIME_TOTAL_CYCLES: PerfKey = trace_counter_key(
    "ui_thread_cpu_cycle_time_total_cycles",
    PerfKeyUnit::Cycles,
    "ui_thread_cpu_cycle_total",
    "cpu",
    PerfKeyAggregate::Max,
);

pub(crate) const LAYOUT_OBSERVATION_RECORD_TIME_US: PerfKey = trace_timing_key(
    "layout_observation_record_time_us",
    "layout.obs_record",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_COLLECT_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "layout_collect_roots_time_us",
    "layout.collect_roots",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_INVALIDATE_SCROLL_HANDLE_BINDINGS_TIME_US: PerfKey = trace_timing_key(
    "layout_invalidate_scroll_handle_bindings_time_us",
    "layout.invalidate_scroll_bindings",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_EXPAND_VIEW_CACHE_INVALIDATIONS_TIME_US: PerfKey = trace_timing_key(
    "layout_expand_view_cache_invalidations_time_us",
    "layout.expand_view_cache_invalidations",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_time_us",
    "layout.request_build_roots",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "layout_roots_time_us",
    "layout.roots",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_VIEW_CACHE_TIME_US: PerfKey = trace_timing_key(
    "layout_view_cache_time_us",
    "layout.view_cache",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_ENGINE_SOLVE_TIME_US: PerfKey = trace_timing_key(
    "layout_engine_solve_time_us",
    "layout.engine_solve",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US: PerfKey = timing_key(
    "layout_pending_barrier_relayouts_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US: PerfKey = timing_key(
    "layout_repair_view_cache_bounds_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US: PerfKey = timing_key(
    "layout_contained_view_cache_roots_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_COLLAPSE_LAYOUT_OBSERVATIONS_TIME_US: PerfKey = timing_key(
    "layout_collapse_layout_observations_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_PREPAINT_AFTER_LAYOUT_TIME_US: PerfKey = timing_key(
    "layout_prepaint_after_layout_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_BARRIER_RELAYOUTS_TIME_US: PerfKey =
    timing_key("layout_barrier_relayouts_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_SEMANTICS_REFRESH_TIME_US: PerfKey =
    timing_key("layout_semantics_refresh_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_FOCUS_REPAIR_TIME_US: PerfKey =
    timing_key("layout_focus_repair_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_DEFERRED_CLEANUP_TIME_US: PerfKey =
    timing_key("layout_deferred_cleanup_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_ENGINE_CHILD_RECT_TIME_US: PerfKey =
    timing_key("layout_engine_child_rect_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_OBSERVATION_RECORD_MODELS_ITEMS: PerfKey = count_key(
    "layout_observation_record_models_items",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_OBSERVATION_RECORD_GLOBALS_ITEMS: PerfKey = count_key(
    "layout_observation_record_globals_items",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_NODES_PERFORMED: PerfKey =
    count_key("layout_nodes_performed", PerfKeyAggregate::Max);
pub(crate) const LAYOUT_ENGINE_SOLVES: PerfKey =
    count_key("layout_engine_solves", PerfKeyAggregate::Max);
pub(crate) const LAYOUT_ENGINE_CHILD_RECT_QUERIES: PerfKey =
    count_key("layout_engine_child_rect_queries", PerfKeyAggregate::Max);
pub(crate) const LAYOUT_ENGINE_WIDGET_FALLBACK_SOLVES: PerfKey = count_key(
    "layout_engine_widget_fallback_solves",
    PerfKeyAggregate::Max,
);

pub(crate) const PAINT_OBSERVATION_RECORD_TIME_US: PerfKey = trace_timing_key(
    "paint_observation_record_time_us",
    "paint.obs_record",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_TEXT_PREPARE_TIME_US: PerfKey = trace_timing_key(
    "paint_text_prepare_time_us",
    "paint.text_prepare",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_RECORD_VISUAL_BOUNDS_TIME_US: PerfKey = trace_timing_key(
    "paint_record_visual_bounds_time_us",
    "paint.record_visual_bounds",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_CACHE_KEY_TIME_US: PerfKey = trace_timing_key(
    "paint_cache_key_time_us",
    "paint.cache_key",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_CACHE_HIT_CHECK_TIME_US: PerfKey = trace_timing_key(
    "paint_cache_hit_check_time_us",
    "paint.cache_hit_check",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_CACHE_REPLAY_TIME_US: PerfKey = trace_timing_key(
    "paint_cache_replay_time_us",
    "paint.cache_replay",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US: PerfKey = trace_timing_key(
    "paint_cache_bounds_translate_time_us",
    "paint.cache_bounds_translate",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_WIDGET_TIME_US: PerfKey = trace_timing_key(
    "paint_widget_time_us",
    "paint.widget",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_INPUT_CONTEXT_TIME_US: PerfKey =
    timing_key("paint_input_context_time_us", PerfKeyAggregate::P95);
pub(crate) const PAINT_SCROLL_HANDLE_INVALIDATION_TIME_US: PerfKey = timing_key(
    "paint_scroll_handle_invalidation_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_COLLECT_ROOTS_TIME_US: PerfKey =
    timing_key("paint_collect_roots_time_us", PerfKeyAggregate::P95);
pub(crate) const PAINT_PUBLISH_TEXT_INPUT_SNAPSHOT_TIME_US: PerfKey = timing_key(
    "paint_publish_text_input_snapshot_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_COLLAPSE_OBSERVATIONS_TIME_US: PerfKey =
    timing_key("paint_collapse_observations_time_us", PerfKeyAggregate::P95);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_MODELS_TIME_US: PerfKey = timing_key(
    "paint_host_widget_observed_models_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_GLOBALS_TIME_US: PerfKey = timing_key(
    "paint_host_widget_observed_globals_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_HOST_WIDGET_INSTANCE_LOOKUP_TIME_US: PerfKey = timing_key(
    "paint_host_widget_instance_lookup_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_RECORD_VISUAL_BOUNDS_CALLS: PerfKey =
    count_key("paint_record_visual_bounds_calls", PerfKeyAggregate::Max);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_MODELS_ITEMS: PerfKey = count_key(
    "paint_host_widget_observed_models_items",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_GLOBALS_ITEMS: PerfKey = count_key(
    "paint_host_widget_observed_globals_items",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_HOST_WIDGET_INSTANCE_LOOKUP_CALLS: PerfKey = count_key(
    "paint_host_widget_instance_lookup_calls",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_TEXT_PREPARE_CALLS: PerfKey =
    count_key("paint_text_prepare_calls", PerfKeyAggregate::Max);
pub(crate) const PAINT_NODES_PERFORMED: PerfKey =
    count_key("paint_nodes_performed", PerfKeyAggregate::Max);
pub(crate) const PAINT_CACHE_MISSES: PerfKey =
    count_key("paint_cache_misses", PerfKeyAggregate::Max);
pub(crate) const PAINT_CACHE_BOUNDS_TRANSLATED_NODES: PerfKey =
    count_key("paint_cache_bounds_translated_nodes", PerfKeyAggregate::Max);

pub(crate) const DISPATCH_INNER_BODY_TIME_US: PerfKey =
    timing_key("dispatch_inner_body_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_ACCOUNTED_TIME_US: PerfKey =
    derived_timing_key("dispatch_accounted_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_UNATTRIBUTED_TIME_US: PerfKey =
    derived_timing_key("dispatch_unattributed_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_INNER_BODY_UNATTRIBUTED_TIME_US: PerfKey = derived_timing_key(
    "dispatch_inner_body_unattributed_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_RUNTIME_WRAPPER_TIME_US: PerfKey =
    derived_timing_key("dispatch_runtime_wrapper_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_POINTER_EVENT_TIME_US: PerfKey =
    timing_key("dispatch_pointer_event_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_TIMER_EVENT_TIME_US: PerfKey =
    timing_key("dispatch_timer_event_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_TIMER_TARGETED_TIME_US: PerfKey =
    timing_key("dispatch_timer_targeted_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_TIMER_BROADCAST_TIME_US: PerfKey =
    timing_key("dispatch_timer_broadcast_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_TIMER_BROADCAST_REBUILD_VISIBLE_LAYERS_TIME_US: PerfKey = timing_key(
    "dispatch_timer_broadcast_rebuild_visible_layers_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_TIMER_BROADCAST_LOOP_TIME_US: PerfKey = timing_key(
    "dispatch_timer_broadcast_loop_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_TIMER_SLOWEST_EVENT_TIME_US: PerfKey = timing_key(
    "dispatch_timer_slowest_event_time_us",
    PerfKeyAggregate::Max,
);
pub(crate) const DISPATCH_OTHER_EVENT_TIME_US: PerfKey =
    timing_key("dispatch_other_event_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_HOVER_UPDATE_TIME_US: PerfKey =
    timing_key("dispatch_hover_update_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_INPUT_STATE_UPDATE_TIME_US: PerfKey =
    timing_key("dispatch_input_state_update_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_CONTEXT_BUILD_TIME_US: PerfKey =
    timing_key("dispatch_context_build_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_PRELUDE_TIME_US: PerfKey =
    timing_key("dispatch_prelude_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_POINTER_ARBITRATION_TIME_US: PerfKey = timing_key(
    "dispatch_pointer_arbitration_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_POINTER_TARGET_ROUTING_TIME_US: PerfKey = timing_key(
    "dispatch_pointer_target_routing_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_POST_WIDGET_CONTROL_FLOW_TIME_US: PerfKey = timing_key(
    "dispatch_post_widget_control_flow_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_SCROLL_HANDLE_INVALIDATION_TIME_US: PerfKey = timing_key(
    "dispatch_scroll_handle_invalidation_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_ACTIVE_LAYERS_TIME_US: PerfKey =
    timing_key("dispatch_active_layers_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_INPUT_CONTEXT_TIME_US: PerfKey =
    timing_key("dispatch_input_context_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_EVENT_CHAIN_BUILD_TIME_US: PerfKey =
    timing_key("dispatch_event_chain_build_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_WIDGET_CAPTURE_TIME_US: PerfKey =
    timing_key("dispatch_widget_capture_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_WIDGET_BUBBLE_TIME_US: PerfKey =
    timing_key("dispatch_widget_bubble_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_CURSOR_QUERY_TIME_US: PerfKey =
    timing_key("dispatch_cursor_query_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_POINTER_MOVE_LAYER_OBSERVERS_TIME_US: PerfKey = timing_key(
    "dispatch_pointer_move_layer_observers_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_SYNTH_HOVER_OBSERVER_TIME_US: PerfKey = timing_key(
    "dispatch_synth_hover_observer_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_CURSOR_EFFECT_TIME_US: PerfKey =
    timing_key("dispatch_cursor_effect_time_us", PerfKeyAggregate::P95);
pub(crate) const DISPATCH_POST_DISPATCH_SNAPSHOT_TIME_US: PerfKey = timing_key(
    "dispatch_post_dispatch_snapshot_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const DISPATCH_EVENTS: PerfKey = count_key("dispatch_events", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_POINTER_EVENTS: PerfKey =
    count_key("dispatch_pointer_events", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_TIMER_EVENTS: PerfKey =
    count_key("dispatch_timer_events", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_TIMER_TARGETED_EVENTS: PerfKey =
    count_key("dispatch_timer_targeted_events", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_TIMER_BROADCAST_EVENTS: PerfKey =
    count_key("dispatch_timer_broadcast_events", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_TIMER_BROADCAST_LAYERS_VISITED: PerfKey = count_key(
    "dispatch_timer_broadcast_layers_visited",
    PerfKeyAggregate::Max,
);
pub(crate) const DISPATCH_OTHER_EVENTS: PerfKey =
    count_key("dispatch_other_events", PerfKeyAggregate::Max);

pub(crate) const HIT_TEST_CACHED_PATH_TIME_US: PerfKey =
    timing_key("hit_test_cached_path_time_us", PerfKeyAggregate::P95);
pub(crate) const HIT_TEST_BOUNDS_TREE_QUERY_TIME_US: PerfKey =
    timing_key("hit_test_bounds_tree_query_time_us", PerfKeyAggregate::P95);
pub(crate) const HIT_TEST_CANDIDATE_SELF_ONLY_TIME_US: PerfKey = timing_key(
    "hit_test_candidate_self_only_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const HIT_TEST_FALLBACK_TRAVERSAL_TIME_US: PerfKey =
    timing_key("hit_test_fallback_traversal_time_us", PerfKeyAggregate::P95);
pub(crate) const HIT_TEST_QUERIES: PerfKey = count_key("hit_test_queries", PerfKeyAggregate::Max);
pub(crate) const HIT_TEST_BOUNDS_TREE_QUERIES: PerfKey =
    count_key("hit_test_bounds_tree_queries", PerfKeyAggregate::Max);
pub(crate) const HIT_TEST_BOUNDS_TREE_DISABLED: PerfKey =
    count_key("hit_test_bounds_tree_disabled", PerfKeyAggregate::Max);
pub(crate) const HIT_TEST_BOUNDS_TREE_MISSES: PerfKey =
    count_key("hit_test_bounds_tree_misses", PerfKeyAggregate::Max);
pub(crate) const HIT_TEST_BOUNDS_TREE_HITS: PerfKey =
    count_key("hit_test_bounds_tree_hits", PerfKeyAggregate::Max);
pub(crate) const HIT_TEST_BOUNDS_TREE_CANDIDATE_REJECTED: PerfKey = count_key(
    "hit_test_bounds_tree_candidate_rejected",
    PerfKeyAggregate::Max,
);

pub(crate) const WINDOW_RUNTIME_SNAPSHOT_FOCUS_REPAIR_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_focus_repair_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_INPUT_CONTEXT_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_input_context_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_COMMAND_AVAILABILITY_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_command_availability_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_COMMAND_REGISTRY_COLLECT_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_command_registry_collect_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_COMMAND_AVAILABILITY_EVAL_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_command_availability_eval_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_SHORTCUT_OVERLAY_TIME_US: PerfKey = timing_key(
    "window_runtime_snapshot_shortcut_overlay_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const WINDOW_RUNTIME_SNAPSHOT_WIDGET_COMMAND_COUNT: PerfKey = count_key(
    "window_runtime_snapshot_widget_command_count",
    PerfKeyAggregate::Max,
);

pub(crate) const RENDERER_ENCODE_SCENE_US: PerfKey =
    timing_key("renderer_encode_scene_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENSURE_PIPELINES_US: PerfKey =
    timing_key("renderer_ensure_pipelines_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_PLAN_COMPILE_US: PerfKey =
    timing_key("renderer_plan_compile_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_UPLOAD_US: PerfKey =
    timing_key("renderer_upload_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_RECORD_PASSES_US: PerfKey =
    timing_key("renderer_record_passes_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODER_FINISH_US: PerfKey =
    timing_key("renderer_encoder_finish_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_PREPARE_SVG_US: PerfKey =
    timing_key("renderer_prepare_svg_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_PREPARE_TEXT_US: PerfKey =
    timing_key("renderer_prepare_text_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_STACK_US: PerfKey =
    timing_key("renderer_encode_scene_stack_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_CLIP_US: PerfKey =
    timing_key("renderer_encode_scene_clip_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_MASK_US: PerfKey =
    timing_key("renderer_encode_scene_mask_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_EFFECT_US: PerfKey =
    timing_key("renderer_encode_scene_effect_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_QUAD_US: PerfKey =
    timing_key("renderer_encode_scene_quad_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_IMAGE_US: PerfKey =
    timing_key("renderer_encode_scene_image_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_US: PerfKey =
    timing_key("renderer_encode_scene_text_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_PATH_US: PerfKey =
    timing_key("renderer_encode_scene_path_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_VIEWPORT_US: PerfKey =
    timing_key("renderer_encode_scene_viewport_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_FLUSH_US: PerfKey =
    timing_key("renderer_encode_scene_flush_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_SHADOW_US: PerfKey = timing_key(
    "renderer_encode_scene_text_shadow_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_SETUP_US: PerfKey =
    timing_key("renderer_encode_scene_text_setup_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_GLYPHS_US: PerfKey = timing_key(
    "renderer_encode_scene_text_glyphs_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_GLYPH_TRANSFORM_US: PerfKey = timing_key(
    "renderer_encode_scene_text_glyph_transform_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_GLYPH_EMIT_US: PerfKey = timing_key(
    "renderer_encode_scene_text_glyph_emit_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_GROUP_FLUSH_US: PerfKey = timing_key(
    "renderer_encode_scene_text_group_flush_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_UNIFORM_BYTES: PerfKey =
    byte_key("renderer_uniform_bytes", PerfKeyAggregate::Max);
pub(crate) const RENDERER_INSTANCE_BYTES: PerfKey =
    byte_key("renderer_instance_bytes", PerfKeyAggregate::Max);
pub(crate) const RENDERER_VERTEX_BYTES: PerfKey =
    byte_key("renderer_vertex_bytes", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_VERTEX_GROW_EVENTS: PerfKey = count_key(
    "renderer_encode_scene_text_vertex_grow_events",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_TRANSFORM_FAST_PATH_GLYPHS: PerfKey = count_key(
    "renderer_encode_scene_text_transform_fast_path_glyphs",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_TRANSFORM_GENERIC_GLYPHS: PerfKey = count_key(
    "renderer_encode_scene_text_transform_generic_glyphs",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_ENCODE_SCENE_STACK_OPS: PerfKey =
    count_key("renderer_encode_scene_stack_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_CLIP_OPS: PerfKey =
    count_key("renderer_encode_scene_clip_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_MASK_OPS: PerfKey =
    count_key("renderer_encode_scene_mask_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_EFFECT_OPS: PerfKey =
    count_key("renderer_encode_scene_effect_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_QUAD_OPS: PerfKey =
    count_key("renderer_encode_scene_quad_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_IMAGE_OPS: PerfKey =
    count_key("renderer_encode_scene_image_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_TEXT_OPS: PerfKey =
    count_key("renderer_encode_scene_text_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_PATH_OPS: PerfKey =
    count_key("renderer_encode_scene_path_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_VIEWPORT_OPS: PerfKey =
    count_key("renderer_encode_scene_viewport_ops", PerfKeyAggregate::Max);
pub(crate) const RENDERER_ENCODE_SCENE_FLUSHES: PerfKey =
    count_key("renderer_encode_scene_flushes", PerfKeyAggregate::Max);

pub(crate) const POINTER_MOVE_MAX_DISPATCH_TIME_US: PerfKey =
    pointer_move_timing_key("pointer_move.max_dispatch_time_us", PerfKeyAggregate::Max);
pub(crate) const POINTER_MOVE_MAX_HIT_TEST_TIME_US: PerfKey =
    pointer_move_timing_key("pointer_move.max_hit_test_time_us", PerfKeyAggregate::Max);
pub(crate) const POINTER_MOVE_SNAPSHOTS_WITH_GLOBAL_CHANGES: PerfKey = pointer_move_count_key(
    "pointer_move.snapshots_with_global_changes",
    PerfKeyAggregate::Max,
);

pub(crate) const TRACE_EXPORTED_FRAME_KEYS: &[PerfKey] = &[
    TOTAL_TIME_US,
    LAYOUT_TIME_US,
    PREPAINT_TIME_US,
    PAINT_TIME_US,
    DISPATCH_TIME_US,
    HIT_TEST_TIME_US,
    UI_THREAD_CPU_TIME_US,
    UI_THREAD_CPU_CYCLE_TIME_DELTA_CYCLES,
    UI_THREAD_CPU_CYCLE_TIME_TOTAL_CYCLES,
    LAYOUT_OBSERVATION_RECORD_TIME_US,
    LAYOUT_COLLECT_ROOTS_TIME_US,
    LAYOUT_INVALIDATE_SCROLL_HANDLE_BINDINGS_TIME_US,
    LAYOUT_EXPAND_VIEW_CACHE_INVALIDATIONS_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_TIME_US,
    LAYOUT_ROOTS_TIME_US,
    LAYOUT_VIEW_CACHE_TIME_US,
    LAYOUT_ENGINE_SOLVE_TIME_US,
    PAINT_OBSERVATION_RECORD_TIME_US,
    PAINT_TEXT_PREPARE_TIME_US,
    PAINT_RECORD_VISUAL_BOUNDS_TIME_US,
    PAINT_CACHE_KEY_TIME_US,
    PAINT_CACHE_HIT_CHECK_TIME_US,
    PAINT_CACHE_REPLAY_TIME_US,
    PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US,
    PAINT_WIDGET_TIME_US,
];

pub(crate) const REGISTERED_FRAME_STATS_KEYS: &[PerfKey] = &[
    TOTAL_TIME_US,
    LAYOUT_TIME_US,
    PREPAINT_TIME_US,
    PAINT_TIME_US,
    DISPATCH_TIME_US,
    HIT_TEST_TIME_US,
    UI_THREAD_CPU_TIME_US,
    UI_THREAD_CPU_TOTAL_TIME_US,
    UI_THREAD_CPU_CYCLE_TIME_DELTA_CYCLES,
    UI_THREAD_CPU_CYCLE_TIME_TOTAL_CYCLES,
    LAYOUT_COLLECT_ROOTS_TIME_US,
    LAYOUT_INVALIDATE_SCROLL_HANDLE_BINDINGS_TIME_US,
    LAYOUT_EXPAND_VIEW_CACHE_INVALIDATIONS_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_TIME_US,
    LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US,
    LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US,
    LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US,
    LAYOUT_COLLAPSE_LAYOUT_OBSERVATIONS_TIME_US,
    LAYOUT_OBSERVATION_RECORD_TIME_US,
    LAYOUT_OBSERVATION_RECORD_MODELS_ITEMS,
    LAYOUT_OBSERVATION_RECORD_GLOBALS_ITEMS,
    LAYOUT_PREPAINT_AFTER_LAYOUT_TIME_US,
    LAYOUT_ROOTS_TIME_US,
    LAYOUT_BARRIER_RELAYOUTS_TIME_US,
    LAYOUT_VIEW_CACHE_TIME_US,
    LAYOUT_SEMANTICS_REFRESH_TIME_US,
    LAYOUT_FOCUS_REPAIR_TIME_US,
    LAYOUT_DEFERRED_CLEANUP_TIME_US,
    LAYOUT_NODES_PERFORMED,
    LAYOUT_ENGINE_SOLVES,
    LAYOUT_ENGINE_SOLVE_TIME_US,
    LAYOUT_ENGINE_CHILD_RECT_QUERIES,
    LAYOUT_ENGINE_CHILD_RECT_TIME_US,
    LAYOUT_ENGINE_WIDGET_FALLBACK_SOLVES,
    PAINT_RECORD_VISUAL_BOUNDS_TIME_US,
    PAINT_RECORD_VISUAL_BOUNDS_CALLS,
    PAINT_CACHE_KEY_TIME_US,
    PAINT_CACHE_HIT_CHECK_TIME_US,
    PAINT_WIDGET_TIME_US,
    PAINT_OBSERVATION_RECORD_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_MODELS_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_MODELS_ITEMS,
    PAINT_HOST_WIDGET_OBSERVED_GLOBALS_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_GLOBALS_ITEMS,
    PAINT_HOST_WIDGET_INSTANCE_LOOKUP_TIME_US,
    PAINT_HOST_WIDGET_INSTANCE_LOOKUP_CALLS,
    PAINT_TEXT_PREPARE_TIME_US,
    PAINT_TEXT_PREPARE_CALLS,
    PAINT_INPUT_CONTEXT_TIME_US,
    PAINT_SCROLL_HANDLE_INVALIDATION_TIME_US,
    PAINT_COLLECT_ROOTS_TIME_US,
    PAINT_PUBLISH_TEXT_INPUT_SNAPSHOT_TIME_US,
    PAINT_COLLAPSE_OBSERVATIONS_TIME_US,
    PAINT_NODES_PERFORMED,
    PAINT_CACHE_MISSES,
    PAINT_CACHE_REPLAY_TIME_US,
    PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US,
    PAINT_CACHE_BOUNDS_TRANSLATED_NODES,
    DISPATCH_INNER_BODY_TIME_US,
    DISPATCH_ACCOUNTED_TIME_US,
    DISPATCH_UNATTRIBUTED_TIME_US,
    DISPATCH_INNER_BODY_UNATTRIBUTED_TIME_US,
    DISPATCH_RUNTIME_WRAPPER_TIME_US,
    DISPATCH_POINTER_EVENT_TIME_US,
    DISPATCH_TIMER_EVENT_TIME_US,
    DISPATCH_TIMER_TARGETED_TIME_US,
    DISPATCH_TIMER_BROADCAST_TIME_US,
    DISPATCH_TIMER_BROADCAST_REBUILD_VISIBLE_LAYERS_TIME_US,
    DISPATCH_TIMER_BROADCAST_LOOP_TIME_US,
    DISPATCH_TIMER_SLOWEST_EVENT_TIME_US,
    DISPATCH_OTHER_EVENT_TIME_US,
    DISPATCH_HOVER_UPDATE_TIME_US,
    DISPATCH_INPUT_STATE_UPDATE_TIME_US,
    DISPATCH_CONTEXT_BUILD_TIME_US,
    DISPATCH_PRELUDE_TIME_US,
    DISPATCH_POINTER_ARBITRATION_TIME_US,
    DISPATCH_POINTER_TARGET_ROUTING_TIME_US,
    DISPATCH_POST_WIDGET_CONTROL_FLOW_TIME_US,
    DISPATCH_SCROLL_HANDLE_INVALIDATION_TIME_US,
    DISPATCH_ACTIVE_LAYERS_TIME_US,
    DISPATCH_INPUT_CONTEXT_TIME_US,
    DISPATCH_EVENT_CHAIN_BUILD_TIME_US,
    DISPATCH_WIDGET_CAPTURE_TIME_US,
    DISPATCH_WIDGET_BUBBLE_TIME_US,
    DISPATCH_CURSOR_QUERY_TIME_US,
    DISPATCH_POINTER_MOVE_LAYER_OBSERVERS_TIME_US,
    DISPATCH_SYNTH_HOVER_OBSERVER_TIME_US,
    DISPATCH_CURSOR_EFFECT_TIME_US,
    DISPATCH_POST_DISPATCH_SNAPSHOT_TIME_US,
    DISPATCH_EVENTS,
    DISPATCH_POINTER_EVENTS,
    DISPATCH_TIMER_EVENTS,
    DISPATCH_TIMER_TARGETED_EVENTS,
    DISPATCH_TIMER_BROADCAST_EVENTS,
    DISPATCH_TIMER_BROADCAST_LAYERS_VISITED,
    DISPATCH_OTHER_EVENTS,
    HIT_TEST_CACHED_PATH_TIME_US,
    HIT_TEST_BOUNDS_TREE_QUERY_TIME_US,
    HIT_TEST_CANDIDATE_SELF_ONLY_TIME_US,
    HIT_TEST_FALLBACK_TRAVERSAL_TIME_US,
    HIT_TEST_QUERIES,
    HIT_TEST_BOUNDS_TREE_QUERIES,
    HIT_TEST_BOUNDS_TREE_DISABLED,
    HIT_TEST_BOUNDS_TREE_MISSES,
    HIT_TEST_BOUNDS_TREE_HITS,
    HIT_TEST_BOUNDS_TREE_CANDIDATE_REJECTED,
    WINDOW_RUNTIME_SNAPSHOT_FOCUS_REPAIR_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_INPUT_CONTEXT_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_COMMAND_AVAILABILITY_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_COMMAND_REGISTRY_COLLECT_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_COMMAND_AVAILABILITY_EVAL_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_SHORTCUT_OVERLAY_TIME_US,
    WINDOW_RUNTIME_SNAPSHOT_WIDGET_COMMAND_COUNT,
    RENDERER_ENCODE_SCENE_US,
    RENDERER_ENSURE_PIPELINES_US,
    RENDERER_PLAN_COMPILE_US,
    RENDERER_UPLOAD_US,
    RENDERER_RECORD_PASSES_US,
    RENDERER_ENCODER_FINISH_US,
    RENDERER_PREPARE_SVG_US,
    RENDERER_PREPARE_TEXT_US,
    RENDERER_ENCODE_SCENE_STACK_US,
    RENDERER_ENCODE_SCENE_CLIP_US,
    RENDERER_ENCODE_SCENE_MASK_US,
    RENDERER_ENCODE_SCENE_EFFECT_US,
    RENDERER_ENCODE_SCENE_QUAD_US,
    RENDERER_ENCODE_SCENE_IMAGE_US,
    RENDERER_ENCODE_SCENE_TEXT_US,
    RENDERER_ENCODE_SCENE_PATH_US,
    RENDERER_ENCODE_SCENE_VIEWPORT_US,
    RENDERER_ENCODE_SCENE_FLUSH_US,
    RENDERER_ENCODE_SCENE_TEXT_SHADOW_US,
    RENDERER_ENCODE_SCENE_TEXT_SETUP_US,
    RENDERER_ENCODE_SCENE_TEXT_GLYPHS_US,
    RENDERER_ENCODE_SCENE_TEXT_GLYPH_TRANSFORM_US,
    RENDERER_ENCODE_SCENE_TEXT_GLYPH_EMIT_US,
    RENDERER_ENCODE_SCENE_TEXT_GROUP_FLUSH_US,
    RENDERER_UNIFORM_BYTES,
    RENDERER_INSTANCE_BYTES,
    RENDERER_VERTEX_BYTES,
    RENDERER_ENCODE_SCENE_TEXT_VERTEX_GROW_EVENTS,
    RENDERER_ENCODE_SCENE_TEXT_TRANSFORM_FAST_PATH_GLYPHS,
    RENDERER_ENCODE_SCENE_TEXT_TRANSFORM_GENERIC_GLYPHS,
    RENDERER_ENCODE_SCENE_STACK_OPS,
    RENDERER_ENCODE_SCENE_CLIP_OPS,
    RENDERER_ENCODE_SCENE_MASK_OPS,
    RENDERER_ENCODE_SCENE_EFFECT_OPS,
    RENDERER_ENCODE_SCENE_QUAD_OPS,
    RENDERER_ENCODE_SCENE_IMAGE_OPS,
    RENDERER_ENCODE_SCENE_TEXT_OPS,
    RENDERER_ENCODE_SCENE_PATH_OPS,
    RENDERER_ENCODE_SCENE_VIEWPORT_OPS,
    RENDERER_ENCODE_SCENE_FLUSHES,
    POINTER_MOVE_MAX_DISPATCH_TIME_US,
    POINTER_MOVE_MAX_HIT_TEST_TIME_US,
    POINTER_MOVE_SNAPSHOTS_WITH_GLOBAL_CHANGES,
];

pub(crate) fn read_u64(stats: Option<&Map<String, Value>>, key: PerfKey) -> u64 {
    stats
        .and_then(|m| m.get(key.key))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

pub(crate) fn trace_exported_frame_keys_json() -> Value {
    perf_keys_json(TRACE_EXPORTED_FRAME_KEYS)
}

pub(crate) fn registered_frame_stats_keys_json() -> Value {
    perf_keys_json(REGISTERED_FRAME_STATS_KEYS)
}

pub(crate) fn registered_frame_stats_inventory_json() -> Value {
    serde_json::json!({
        "schema_version": PERF_KEY_REGISTRY_SCHEMA_VERSION,
        "kind": PERF_KEY_REGISTRY_KIND,
        "scope": "frame_stats",
        "coverage": "registered_subset",
        "complete": false,
        "note": "Registered frame/stats/gate perf key subset. Full bundle/stats/gate coverage is still tracked by diag-perf-profiling-infra-v1.",
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "keys": registered_frame_stats_keys_json(),
    })
}

fn perf_keys_json(keys: &[PerfKey]) -> Value {
    Value::Array(
        keys.iter()
            .map(|key| {
                let mut obj = serde_json::json!({
                    "key": key.key,
                    "unit": key.unit.as_str(),
                    "kind": key.kind.as_str(),
                    "scope": key.scope.as_str(),
                    "suggested_aggregate": key.suggested_aggregate.as_str(),
                });
                if let Some(trace) = key.trace {
                    obj["trace_event"] = Value::from(trace.event);
                    obj["trace_category"] = Value::from(trace.category);
                }
                obj
            })
            .collect(),
    )
}

const fn trace_timing_key(
    key: &'static str,
    trace_event: &'static str,
    trace_category: &'static str,
    suggested_aggregate: PerfKeyAggregate,
) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Microseconds,
        kind: PerfKeyKind::Timing,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: Some(PerfKeyTrace {
            event: trace_event,
            category: trace_category,
        }),
    }
}

const fn timing_key(key: &'static str, suggested_aggregate: PerfKeyAggregate) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Microseconds,
        kind: PerfKeyKind::Timing,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: None,
    }
}

const fn derived_timing_key(key: &'static str, suggested_aggregate: PerfKeyAggregate) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Microseconds,
        kind: PerfKeyKind::Timing,
        scope: PerfKeyScope::DerivedStats,
        suggested_aggregate,
        trace: None,
    }
}

const fn trace_counter_key(
    key: &'static str,
    unit: PerfKeyUnit,
    trace_event: &'static str,
    trace_category: &'static str,
    suggested_aggregate: PerfKeyAggregate,
) -> PerfKey {
    PerfKey {
        key,
        unit,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: Some(PerfKeyTrace {
            event: trace_event,
            category: trace_category,
        }),
    }
}

const fn count_key(key: &'static str, suggested_aggregate: PerfKeyAggregate) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Count,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: None,
    }
}

const fn byte_key(key: &'static str, suggested_aggregate: PerfKeyAggregate) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Bytes,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: None,
    }
}

const fn pointer_move_timing_key(
    key: &'static str,
    suggested_aggregate: PerfKeyAggregate,
) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Microseconds,
        kind: PerfKeyKind::Timing,
        scope: PerfKeyScope::PointerMove,
        suggested_aggregate,
        trace: None,
    }
}

const fn pointer_move_count_key(
    key: &'static str,
    suggested_aggregate: PerfKeyAggregate,
) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Count,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::PointerMove,
        suggested_aggregate,
        trace: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unique(keys: &[PerfKey]) {
        let mut keys: Vec<&str> = keys.iter().map(|key| key.key).collect();
        let original_len = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), original_len);
    }

    fn assert_units_match_names(keys: &[PerfKey]) {
        for key in keys {
            if key.key.ends_with("_time_us") || key.key.ends_with("_us") {
                assert_eq!(key.unit, PerfKeyUnit::Microseconds, "{}", key.key);
            }
            if key.key.ends_with("_cycles") {
                assert_eq!(key.unit, PerfKeyUnit::Cycles, "{}", key.key);
            }
            if key.key.ends_with("_bytes") {
                assert_eq!(key.unit, PerfKeyUnit::Bytes, "{}", key.key);
            }
            assert!(!key.unit.as_str().is_empty());
        }
    }

    #[test]
    fn trace_exported_perf_keys_are_unique() {
        assert_unique(TRACE_EXPORTED_FRAME_KEYS);
    }

    #[test]
    fn registered_perf_keys_are_unique() {
        let mut keys: Vec<&str> = REGISTERED_FRAME_STATS_KEYS
            .iter()
            .map(|key| key.key)
            .collect();
        let len = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), len);
    }

    #[test]
    fn trace_exported_perf_key_units_match_names() {
        assert_units_match_names(TRACE_EXPORTED_FRAME_KEYS);
        for key in TRACE_EXPORTED_FRAME_KEYS {
            assert_eq!(key.scope, PerfKeyScope::Frame);
            assert!(key.trace.is_some(), "missing trace metadata: {}", key.key);
        }
    }

    #[test]
    fn registered_perf_key_units_match_names() {
        assert_units_match_names(REGISTERED_FRAME_STATS_KEYS);
    }

    #[test]
    fn trace_exported_perf_key_registry_contains_core_timeline_keys() {
        let keys: std::collections::BTreeSet<&str> = TRACE_EXPORTED_FRAME_KEYS
            .iter()
            .map(|key| key.key)
            .collect();
        for expected in [
            "total_time_us",
            "dispatch_time_us",
            "hit_test_time_us",
            "layout_time_us",
            "prepaint_time_us",
            "paint_time_us",
            "layout_request_build_roots_time_us",
            "layout_engine_solve_time_us",
            "paint_cache_replay_time_us",
            "paint_widget_time_us",
            "ui_thread_cpu_time_us",
            "ui_thread_cpu_cycle_time_delta_cycles",
        ] {
            assert!(keys.contains(expected), "missing perf key: {expected}");
        }
    }

    #[test]
    fn registered_perf_key_contract_keeps_stats_and_gate_keys_additive() {
        let keys: std::collections::BTreeSet<&str> = REGISTERED_FRAME_STATS_KEYS
            .iter()
            .map(|key| key.key)
            .collect();
        for expected in [
            "total_time_us",
            "layout_time_us",
            "layout_pending_barrier_relayouts_time_us",
            "layout_repair_view_cache_bounds_time_us",
            "layout_contained_view_cache_roots_time_us",
            "layout_engine_child_rect_time_us",
            "paint_input_context_time_us",
            "paint_publish_text_input_snapshot_time_us",
            "dispatch_accounted_time_us",
            "dispatch_pointer_move_layer_observers_time_us",
            "window_runtime_snapshot_command_availability_eval_time_us",
            "hit_test_bounds_tree_query_time_us",
            "renderer_encode_scene_us",
            "renderer_upload_us",
            "renderer_record_passes_us",
            "renderer_encoder_finish_us",
            "renderer_prepare_text_us",
            "renderer_prepare_svg_us",
            "renderer_instance_bytes",
            "renderer_encode_scene_text_ops",
            "pointer_move.max_dispatch_time_us",
            "pointer_move.max_hit_test_time_us",
        ] {
            assert!(keys.contains(expected), "missing perf key: {expected}");
        }
    }

    #[test]
    fn registered_perf_key_inventory_doc_is_in_sync() {
        let expected = registered_frame_stats_inventory_json();
        let doc: Value = serde_json::from_str(include_str!(
            "../../../docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json"
        ))
        .expect("parse perf key registry inventory doc");
        assert_eq!(doc, expected);
    }
}
