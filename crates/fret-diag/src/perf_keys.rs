use serde_json::{Map, Value};

pub(crate) const PERF_KEY_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_KEY_REGISTRY_KIND: &str = "perf_key_registry";
pub(crate) const PERF_THRESHOLD_KEY_REGISTRY_KIND: &str = "perf_threshold_key_registry";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyUnit {
    Microseconds,
    Cycles,
    Count,
    Bytes,
    Pixels,
    Boolean,
    Id,
    Label,
}

impl PerfKeyUnit {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Microseconds => "us",
            Self::Cycles => "cycles",
            Self::Count => "count",
            Self::Bytes => "bytes",
            Self::Pixels => "px",
            Self::Boolean => "bool",
            Self::Id => "id",
            Self::Label => "label",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyKind {
    Timing,
    Counter,
    Flag,
    Identifier,
    Label,
}

impl PerfKeyKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Timing => "timing",
            Self::Counter => "counter",
            Self::Flag => "flag",
            Self::Identifier => "identifier",
            Self::Label => "label",
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
    Any,
    None,
}

impl PerfKeyAggregate {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::P95 => "p95",
            Self::Any => "any",
            Self::None => "none",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfThresholdDirection {
    Max,
    Min,
}

impl PerfThresholdDirection {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::Min => "min",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PerfThresholdKey {
    pub(crate) key: &'static str,
    pub(crate) metric: &'static str,
    pub(crate) source_metric: Option<&'static str>,
    pub(crate) unit: PerfKeyUnit,
    pub(crate) direction: PerfThresholdDirection,
    pub(crate) scope: &'static str,
    pub(crate) observed_aggregate: &'static str,
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
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_TAKE_ENGINE_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_take_engine_time_us",
    "layout.request_build_roots.take_engine",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_PHASE1_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_phase1_time_us",
    "layout.request_build_roots.phase1",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_phase2_time_us",
    "layout.request_build_roots.phase2",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_CLEAN_GEOMETRY_PROOF_TIME_US: PerfKey =
    trace_timing_key(
        "layout_request_build_roots_phase2_clean_geometry_proof_time_us",
        "layout.request_build_roots.phase2.clean_geometry_proof",
        "layout",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_NODES: PerfKey =
    count_key("layout_clean_geometry_proof_nodes", PerfKeyAggregate::Max);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_BOUNDARIES: PerfKey = count_key(
    "layout_clean_geometry_proof_boundaries",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_LEAF_SHORTCUT_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_leaf_shortcut_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_NODE_STATE_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_node_state_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_contract_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_RECORD_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_record_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_EVAL_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_contract_eval_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_child_bounds_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_ORIGIN_ONLY_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_child_bounds_origin_only_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_STYLE_LOOKUP_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_PREV_BOUNDS_LOOKUP_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_ABSOLUTE_CHILD_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_RELATIVE_CHILD_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_VERTICAL_NO_WRAP_FLEX_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_HORIZONTAL_FIXED_FLEX_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_CONTAINER_PX_INSETS_TIME_US: PerfKey =
    timing_key(
        "layout_clean_geometry_proof_child_bounds_container_px_insets_time_us",
        PerfKeyAggregate::P95,
    );
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_SINGLE_COLUMN_AUTO_ROWS_GRID_TIME_US:
    PerfKey = timing_key(
    "layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_TEXT_METRICS_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_text_metrics_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_PREV_BOUNDS_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_child_prev_bounds_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_PROOF_EMIT_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_proof_emit_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_NODES: PerfKey =
    count_key("layout_clean_geometry_apply_nodes", PerfKeyAggregate::Max);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS: PerfKey = count_key(
    "layout_clean_geometry_apply_fallback_layouts",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_apply_fallback_layouts_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_apply_fallback_layouts_top_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_KIND: PerfKey =
    label_key("layout_clean_geometry_apply_fallback_layouts_top_kind");
pub(crate) const LAYOUT_CLEAN_GEOMETRY_SCROLL_SIDE_EFFECT_FAST_PATHS: PerfKey = count_key(
    "layout_clean_geometry_scroll_side_effect_fast_paths",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_APPLY_PAINT_FINGERPRINT_TIME_US: PerfKey = timing_key(
    "layout_clean_geometry_apply_paint_fingerprint_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_COMPUTE_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_phase2_compute_time_us",
    "layout.request_build_roots.phase2.compute",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_PUT_ENGINE_TIME_US: PerfKey = trace_timing_key(
    "layout_request_build_roots_put_engine_time_us",
    "layout.request_build_roots.put_engine",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "layout_roots_time_us",
    "layout.roots",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_ROOTS_APPLY_TIME_US: PerfKey = trace_timing_key(
    "layout_roots_apply_time_us",
    "layout.roots.apply",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_ROOTS_FLUSH_VIEWPORT_TIME_US: PerfKey = trace_timing_key(
    "layout_roots_flush_viewport_time_us",
    "layout.roots.flush_viewport",
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
pub(crate) const LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US: PerfKey = trace_timing_key(
    "layout_pending_barrier_relayouts_time_us",
    "layout.pending_barriers",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US: PerfKey = trace_timing_key(
    "layout_repair_view_cache_bounds_time_us",
    "layout.view_cache.repair_bounds",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "layout_contained_view_cache_roots_time_us",
    "layout.view_cache.layout_contained_roots",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_COLLAPSE_LAYOUT_OBSERVATIONS_TIME_US: PerfKey = trace_timing_key(
    "layout_collapse_layout_observations_time_us",
    "layout.view_cache.collapse_observations",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_PREPAINT_AFTER_LAYOUT_TIME_US: PerfKey = timing_key(
    "layout_prepaint_after_layout_time_us",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_BARRIER_RELAYOUTS_TIME_US: PerfKey =
    timing_key("layout_barrier_relayouts_time_us", PerfKeyAggregate::P95);
pub(crate) const LAYOUT_SEMANTICS_REFRESH_TIME_US: PerfKey = trace_timing_key(
    "layout_semantics_refresh_time_us",
    "layout.refresh_semantics",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_FOCUS_REPAIR_TIME_US: PerfKey = trace_timing_key(
    "layout_focus_repair_time_us",
    "layout.focus_repair",
    "layout",
    PerfKeyAggregate::P95,
);
pub(crate) const LAYOUT_DEFERRED_CLEANUP_TIME_US: PerfKey = trace_timing_key(
    "layout_deferred_cleanup_time_us",
    "layout.flush_deferred_cleanup",
    "layout",
    PerfKeyAggregate::P95,
);
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
pub(crate) const LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_REJECTIONS: PerfKey = count_key(
    "layout_clean_geometry_solve_skip_rejections",
    PerfKeyAggregate::Max,
);
pub(crate) const LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_REJECTION: PerfKey =
    label_key("layout_clean_geometry_solve_skip_first_rejection");
pub(crate) const LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_DETAIL: PerfKey =
    label_key("layout_clean_geometry_solve_skip_first_detail");
pub(crate) const LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_ELEMENT_KIND: PerfKey =
    label_key("layout_clean_geometry_solve_skip_first_element_kind");
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
pub(crate) const PAINT_CANVAS_ON_PAINT_TIME_US: PerfKey = trace_timing_key(
    "paint_canvas_on_paint_time_us",
    "paint.canvas_on_paint",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_INPUT_CONTEXT_TIME_US: PerfKey = trace_timing_key(
    "paint_input_context_time_us",
    "paint.input_context",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_SCROLL_HANDLE_INVALIDATION_TIME_US: PerfKey = trace_timing_key(
    "paint_scroll_handle_invalidation_time_us",
    "paint.scroll_handle_invalidation",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_COLLECT_ROOTS_TIME_US: PerfKey = trace_timing_key(
    "paint_collect_roots_time_us",
    "paint.collect_roots",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_PUBLISH_TEXT_INPUT_SNAPSHOT_TIME_US: PerfKey = trace_timing_key(
    "paint_publish_text_input_snapshot_time_us",
    "paint.publish_text_input_snapshot",
    "paint",
    PerfKeyAggregate::P95,
);
pub(crate) const PAINT_COLLAPSE_OBSERVATIONS_TIME_US: PerfKey = trace_timing_key(
    "paint_collapse_observations_time_us",
    "paint.collapse_observations",
    "paint",
    PerfKeyAggregate::P95,
);
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
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_DEPS_CALLS: PerfKey = count_key(
    "paint_host_widget_observed_deps_calls",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_DEPS_EMPTY_CALLS: PerfKey = count_key(
    "paint_host_widget_observed_deps_empty_calls",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_MODELS_NON_EMPTY_CALLS: PerfKey = count_key(
    "paint_host_widget_observed_models_non_empty_calls",
    PerfKeyAggregate::Max,
);
pub(crate) const PAINT_HOST_WIDGET_OBSERVED_GLOBALS_NON_EMPTY_CALLS: PerfKey = count_key(
    "paint_host_widget_observed_globals_non_empty_calls",
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
pub(crate) const DISPATCH_SNAPSHOT_CACHE_HITS: PerfKey =
    count_key("dispatch_snapshot_cache_hits", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_SNAPSHOT_CACHE_MISSES: PerfKey =
    count_key("dispatch_snapshot_cache_misses", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_SNAPSHOT_BUILDS: PerfKey =
    count_key("dispatch_snapshot_builds", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_SNAPSHOT_BUILT_NODES: PerfKey =
    count_key("dispatch_snapshot_built_nodes", PerfKeyAggregate::Max);
pub(crate) const DISPATCH_SNAPSHOT_CACHE_INVALIDATIONS: PerfKey = count_key(
    "dispatch_snapshot_cache_invalidations",
    PerfKeyAggregate::Max,
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
pub(crate) const RENDERER_PREPARE_TEXT_COLLECT_PIN_KEYS_US: PerfKey = timing_key(
    "renderer_prepare_text_collect_pin_keys_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_PREPARE_TEXT_BUCKET_DELTA_US: PerfKey = timing_key(
    "renderer_prepare_text_bucket_delta_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_PREPARE_TEXT_PREWARM_US: PerfKey =
    timing_key("renderer_prepare_text_prewarm_us", PerfKeyAggregate::P95);
pub(crate) const RENDERER_PREPARE_TEXT_PIN_BUCKET_UPDATE_US: PerfKey = timing_key(
    "renderer_prepare_text_pin_bucket_update_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_PREPARE_TEXT_FLUSH_UPLOADS_US: PerfKey = timing_key(
    "renderer_prepare_text_flush_uploads_us",
    PerfKeyAggregate::P95,
);
pub(crate) const RENDERER_PREPARE_TEXT_SCENE_TEXT_BLOBS: PerfKey = count_key(
    "renderer_prepare_text_scene_text_blobs",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_FAST_SCENE_BUCKET_REUSES: PerfKey = count_key(
    "renderer_prepare_text_fast_scene_bucket_reuses",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_PINNED_GLYPH_KEYS: PerfKey = count_key(
    "renderer_prepare_text_pinned_glyph_keys",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_RETAINED_GLYPH_KEYS: PerfKey = count_key(
    "renderer_prepare_text_retained_glyph_keys",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_ADDED_GLYPH_KEYS: PerfKey = count_key(
    "renderer_prepare_text_added_glyph_keys",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_REMOVED_GLYPH_KEYS: PerfKey = count_key(
    "renderer_prepare_text_removed_glyph_keys",
    PerfKeyAggregate::Max,
);
pub(crate) const RENDERER_PREPARE_TEXT_PREWARM_GLYPH_KEYS: PerfKey = count_key(
    "renderer_prepare_text_prewarm_glyph_keys",
    PerfKeyAggregate::Max,
);
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

pub(crate) const PERF_THRESHOLD_KEYS: &[PerfThresholdKey] = &[
    threshold_max_us_key("max_top_total_us", "top_total_time_us", "top_frame"),
    threshold_max_us_key("max_top_layout_us", "top_layout_time_us", "top_frame"),
    threshold_max_us_key(
        "max_top_solve_us",
        "top_layout_engine_solve_time_us",
        "top_frame",
    ),
    threshold_max_us_key(
        "max_frame_p95_total_us",
        "frame_p95_total_time_us",
        "frame_distribution",
    ),
    threshold_max_us_key(
        "max_frame_p95_layout_us",
        "frame_p95_layout_time_us",
        "frame_distribution",
    ),
    threshold_max_us_key(
        "max_frame_p95_solve_us",
        "frame_p95_layout_engine_solve_time_us",
        "frame_distribution",
    ),
    threshold_max_us_key(
        "max_pointer_move_dispatch_us",
        "pointer_move_max_dispatch_time_us",
        "pointer_move",
    ),
    threshold_max_us_key(
        "max_pointer_move_hit_test_us",
        "pointer_move_max_hit_test_time_us",
        "pointer_move",
    ),
    threshold_max_count_key(
        "max_pointer_move_global_changes",
        "pointer_move_snapshots_with_global_changes",
        "pointer_move",
    ),
    PerfThresholdKey {
        key: "min_run_paint_cache_hit_test_only_replay_allowed_max",
        metric: "run_paint_cache_hit_test_only_replay_allowed_max",
        source_metric: Some("paint_cache_hit_test_only_replay_allowed"),
        unit: PerfKeyUnit::Count,
        direction: PerfThresholdDirection::Min,
        scope: "run",
        observed_aggregate: "max",
    },
    threshold_max_count_key(
        "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max",
        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max",
        "run",
    ),
    threshold_max_us_key(
        "max_renderer_encode_scene_us",
        "renderer_encode_scene_us",
        "renderer",
    ),
    threshold_max_us_key("max_renderer_upload_us", "renderer_upload_us", "renderer"),
    threshold_max_us_key(
        "max_renderer_record_passes_us",
        "renderer_record_passes_us",
        "renderer",
    ),
    threshold_max_us_key(
        "max_renderer_encoder_finish_us",
        "renderer_encoder_finish_us",
        "renderer",
    ),
    threshold_max_us_key(
        "max_renderer_prepare_text_us",
        "renderer_prepare_text_us",
        "renderer",
    ),
    threshold_max_us_key(
        "max_renderer_prepare_svg_us",
        "renderer_prepare_svg_us",
        "renderer",
    ),
    threshold_max_bytes_key(
        "max_renderer_instance_bytes",
        "renderer_instance_bytes",
        "renderer",
    ),
    threshold_max_count_key(
        "max_renderer_encode_scene_text_ops",
        "renderer_encode_scene_text_ops",
        "renderer",
    ),
];

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
    LAYOUT_REQUEST_BUILD_ROOTS_TAKE_ENGINE_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE1_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_CLEAN_GEOMETRY_PROOF_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_NODES,
    LAYOUT_CLEAN_GEOMETRY_PROOF_BOUNDARIES,
    LAYOUT_CLEAN_GEOMETRY_PROOF_LEAF_SHORTCUT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_NODE_STATE_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_RECORD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_EVAL_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_ORIGIN_ONLY_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_STYLE_LOOKUP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_PREV_BOUNDS_LOOKUP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_ABSOLUTE_CHILD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_RELATIVE_CHILD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_VERTICAL_NO_WRAP_FLEX_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_HORIZONTAL_FIXED_FLEX_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_CONTAINER_PX_INSETS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_SINGLE_COLUMN_AUTO_ROWS_GRID_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_TEXT_METRICS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_PREV_BOUNDS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_EMIT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_NODES,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_KIND,
    LAYOUT_CLEAN_GEOMETRY_SCROLL_SIDE_EFFECT_FAST_PATHS,
    LAYOUT_CLEAN_GEOMETRY_APPLY_PAINT_FINGERPRINT_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_COMPUTE_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PUT_ENGINE_TIME_US,
    LAYOUT_ROOTS_TIME_US,
    LAYOUT_ROOTS_APPLY_TIME_US,
    LAYOUT_ROOTS_FLUSH_VIEWPORT_TIME_US,
    LAYOUT_VIEW_CACHE_TIME_US,
    LAYOUT_ENGINE_SOLVE_TIME_US,
    LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US,
    LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US,
    LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US,
    LAYOUT_COLLAPSE_LAYOUT_OBSERVATIONS_TIME_US,
    LAYOUT_FOCUS_REPAIR_TIME_US,
    LAYOUT_SEMANTICS_REFRESH_TIME_US,
    LAYOUT_DEFERRED_CLEANUP_TIME_US,
    PAINT_OBSERVATION_RECORD_TIME_US,
    PAINT_TEXT_PREPARE_TIME_US,
    PAINT_RECORD_VISUAL_BOUNDS_TIME_US,
    PAINT_CACHE_KEY_TIME_US,
    PAINT_CACHE_HIT_CHECK_TIME_US,
    PAINT_CACHE_REPLAY_TIME_US,
    PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US,
    PAINT_WIDGET_TIME_US,
    PAINT_INPUT_CONTEXT_TIME_US,
    PAINT_SCROLL_HANDLE_INVALIDATION_TIME_US,
    PAINT_COLLECT_ROOTS_TIME_US,
    PAINT_PUBLISH_TEXT_INPUT_SNAPSHOT_TIME_US,
    PAINT_COLLAPSE_OBSERVATIONS_TIME_US,
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
    LAYOUT_REQUEST_BUILD_ROOTS_TAKE_ENGINE_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE1_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_CLEAN_GEOMETRY_PROOF_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_NODES,
    LAYOUT_CLEAN_GEOMETRY_PROOF_BOUNDARIES,
    LAYOUT_CLEAN_GEOMETRY_PROOF_LEAF_SHORTCUT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_NODE_STATE_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_RECORD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CONTRACT_EVAL_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_ORIGIN_ONLY_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_STYLE_LOOKUP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_PREV_BOUNDS_LOOKUP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_ABSOLUTE_CHILD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_PRESERVE_LOCAL_ORIGINS_RELATIVE_CHILD_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_VERTICAL_NO_WRAP_FLEX_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_HORIZONTAL_FIXED_FLEX_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_CONTAINER_PX_INSETS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_BOUNDS_SINGLE_COLUMN_AUTO_ROWS_GRID_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_TEXT_METRICS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_CHILD_PREV_BOUNDS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_PROOF_EMIT_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_NODES,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_TIME_US,
    LAYOUT_CLEAN_GEOMETRY_APPLY_FALLBACK_LAYOUTS_TOP_KIND,
    LAYOUT_CLEAN_GEOMETRY_SCROLL_SIDE_EFFECT_FAST_PATHS,
    LAYOUT_CLEAN_GEOMETRY_APPLY_PAINT_FINGERPRINT_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_COMPUTE_TIME_US,
    LAYOUT_REQUEST_BUILD_ROOTS_PUT_ENGINE_TIME_US,
    LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US,
    LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US,
    LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US,
    LAYOUT_COLLAPSE_LAYOUT_OBSERVATIONS_TIME_US,
    LAYOUT_OBSERVATION_RECORD_TIME_US,
    LAYOUT_OBSERVATION_RECORD_MODELS_ITEMS,
    LAYOUT_OBSERVATION_RECORD_GLOBALS_ITEMS,
    LAYOUT_PREPAINT_AFTER_LAYOUT_TIME_US,
    LAYOUT_ROOTS_TIME_US,
    LAYOUT_ROOTS_APPLY_TIME_US,
    LAYOUT_ROOTS_FLUSH_VIEWPORT_TIME_US,
    LAYOUT_BARRIER_RELAYOUTS_TIME_US,
    LAYOUT_VIEW_CACHE_TIME_US,
    LAYOUT_SEMANTICS_REFRESH_TIME_US,
    LAYOUT_FOCUS_REPAIR_TIME_US,
    LAYOUT_DEFERRED_CLEANUP_TIME_US,
    LAYOUT_NODES_PERFORMED,
    LAYOUT_ENGINE_SOLVES,
    LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_REJECTIONS,
    LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_REJECTION,
    LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_DETAIL,
    LAYOUT_CLEAN_GEOMETRY_SOLVE_SKIP_FIRST_ELEMENT_KIND,
    LAYOUT_ENGINE_SOLVE_TIME_US,
    LAYOUT_ENGINE_CHILD_RECT_QUERIES,
    LAYOUT_ENGINE_CHILD_RECT_TIME_US,
    LAYOUT_ENGINE_WIDGET_FALLBACK_SOLVES,
    PAINT_RECORD_VISUAL_BOUNDS_TIME_US,
    PAINT_RECORD_VISUAL_BOUNDS_CALLS,
    PAINT_CACHE_KEY_TIME_US,
    PAINT_CACHE_HIT_CHECK_TIME_US,
    PAINT_WIDGET_TIME_US,
    PAINT_CANVAS_ON_PAINT_TIME_US,
    PAINT_OBSERVATION_RECORD_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_MODELS_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_MODELS_ITEMS,
    PAINT_HOST_WIDGET_OBSERVED_GLOBALS_TIME_US,
    PAINT_HOST_WIDGET_OBSERVED_GLOBALS_ITEMS,
    PAINT_HOST_WIDGET_OBSERVED_DEPS_CALLS,
    PAINT_HOST_WIDGET_OBSERVED_DEPS_EMPTY_CALLS,
    PAINT_HOST_WIDGET_OBSERVED_MODELS_NON_EMPTY_CALLS,
    PAINT_HOST_WIDGET_OBSERVED_GLOBALS_NON_EMPTY_CALLS,
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
    DISPATCH_SNAPSHOT_CACHE_HITS,
    DISPATCH_SNAPSHOT_CACHE_MISSES,
    DISPATCH_SNAPSHOT_BUILDS,
    DISPATCH_SNAPSHOT_BUILT_NODES,
    DISPATCH_SNAPSHOT_CACHE_INVALIDATIONS,
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
    RENDERER_PREPARE_TEXT_COLLECT_PIN_KEYS_US,
    RENDERER_PREPARE_TEXT_BUCKET_DELTA_US,
    RENDERER_PREPARE_TEXT_PREWARM_US,
    RENDERER_PREPARE_TEXT_PIN_BUCKET_UPDATE_US,
    RENDERER_PREPARE_TEXT_FLUSH_UPLOADS_US,
    RENDERER_PREPARE_TEXT_SCENE_TEXT_BLOBS,
    RENDERER_PREPARE_TEXT_FAST_SCENE_BUCKET_REUSES,
    RENDERER_PREPARE_TEXT_PINNED_GLYPH_KEYS,
    RENDERER_PREPARE_TEXT_RETAINED_GLYPH_KEYS,
    RENDERER_PREPARE_TEXT_ADDED_GLYPH_KEYS,
    RENDERER_PREPARE_TEXT_REMOVED_GLYPH_KEYS,
    RENDERER_PREPARE_TEXT_PREWARM_GLYPH_KEYS,
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
    count_key("barrier_relayouts_performed", PerfKeyAggregate::Max),
    count_key("barrier_relayouts_scheduled", PerfKeyAggregate::Max),
    id_key("dispatch_timer_slowest_token"),
    flag_key("dispatch_timer_slowest_was_broadcast"),
    count_key(
        "element_children_vec_pool_grow_events",
        PerfKeyAggregate::Max,
    ),
    count_key("element_children_vec_pool_misses", PerfKeyAggregate::Max),
    count_key("element_children_vec_pool_reuses", PerfKeyAggregate::Max),
    byte_key("frame_arena_capacity_estimate_bytes", PerfKeyAggregate::Max),
    count_key("frame_arena_grow_events", PerfKeyAggregate::Max),
    count_key(
        "dirty_frontier_boundaries_at_layout_start",
        PerfKeyAggregate::Max,
    ),
    count_key("dirty_frontier_boundaries_max", PerfKeyAggregate::Max),
    count_key("dirty_frontier_contained_candidates", PerfKeyAggregate::Max),
    count_key("dirty_frontier_hit_test_nodes_max", PerfKeyAggregate::Max),
    count_key("dirty_frontier_layout_nodes_max", PerfKeyAggregate::Max),
    count_key("dirty_frontier_paint_nodes_max", PerfKeyAggregate::Max),
    count_key("gc_reachability_layer_nodes", PerfKeyAggregate::Max),
    count_key("gc_reachability_view_cache_nodes", PerfKeyAggregate::Max),
    count_key("gc_stale_candidates", PerfKeyAggregate::Max),
    count_key("gc_stale_removed", PerfKeyAggregate::Max),
    count_key("global_change_globals", PerfKeyAggregate::Max),
    count_key("global_change_invalidation_roots", PerfKeyAggregate::Max),
    count_key("global_change_observation_edges", PerfKeyAggregate::Max),
    count_key("global_change_unobserved_globals", PerfKeyAggregate::Max),
    count_key(
        "global_observation_index_edges_added",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "global_observation_index_edges_mask_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "global_observation_index_edges_removed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "hover_declarative_hit_test_invalidations",
        PerfKeyAggregate::Max,
    ),
    count_key("hover_declarative_instance_changes", PerfKeyAggregate::Max),
    count_key(
        "hover_declarative_layout_invalidations",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "hover_declarative_paint_invalidations",
        PerfKeyAggregate::Max,
    ),
    count_key("hover_hover_region_target_changes", PerfKeyAggregate::Max),
    count_key("hover_pressable_target_changes", PerfKeyAggregate::Max),
    count_key("identity_resolve_seeded_hits", PerfKeyAggregate::Max),
    count_key("identity_resolve_seeded_stale", PerfKeyAggregate::Max),
    count_key("identity_resolve_fallback_scans", PerfKeyAggregate::Max),
    count_key(
        "identity_resolve_fallback_scan_nodes",
        PerfKeyAggregate::Max,
    ),
    count_key("identity_resolve_fallback_hits", PerfKeyAggregate::Max),
    count_key("identity_resolve_fallback_misses", PerfKeyAggregate::Max),
    count_key("invalidation_walk_calls", PerfKeyAggregate::Max),
    count_key("invalidation_walk_calls_focus", PerfKeyAggregate::Max),
    count_key(
        "invalidation_walk_calls_global_change",
        PerfKeyAggregate::Max,
    ),
    count_key("invalidation_walk_calls_hover", PerfKeyAggregate::Max),
    count_key(
        "invalidation_walk_calls_model_change",
        PerfKeyAggregate::Max,
    ),
    count_key("invalidation_walk_calls_other", PerfKeyAggregate::Max),
    count_key("invalidation_walk_nodes", PerfKeyAggregate::Max),
    count_key("invalidation_walk_nodes_focus", PerfKeyAggregate::Max),
    count_key(
        "invalidation_walk_nodes_global_change",
        PerfKeyAggregate::Max,
    ),
    count_key("invalidation_walk_nodes_hover", PerfKeyAggregate::Max),
    count_key(
        "invalidation_walk_nodes_model_change",
        PerfKeyAggregate::Max,
    ),
    count_key("invalidation_walk_nodes_other", PerfKeyAggregate::Max),
    flag_key("layout_fast_path_taken"),
    flag_key("layout_skipped_engine_frame"),
    count_key("model_change_invalidation_roots", PerfKeyAggregate::Max),
    count_key("model_change_models", PerfKeyAggregate::Max),
    count_key("model_change_observation_edges", PerfKeyAggregate::Max),
    count_key("model_change_unobserved_models", PerfKeyAggregate::Max),
    count_key("model_observation_index_edges_added", PerfKeyAggregate::Max),
    count_key(
        "model_observation_index_edges_mask_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "model_observation_index_edges_removed",
        PerfKeyAggregate::Max,
    ),
    count_key("parent_pointer_repair_passes", PerfKeyAggregate::Max),
    count_key("parent_pointer_repairs", PerfKeyAggregate::Max),
    count_key(
        "paint_text_prepare_reason_blob_missing",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_font_stack_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_overflow_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_rich_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_scale_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_style_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_text_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_width_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "paint_text_prepare_reason_wrap_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_applied_raw",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_pyramid_applied_levels_ge2",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_pyramid_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_raw_degraded_budget_insufficient",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_raw_degraded_budget_zero",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_raw_degraded_target_exhausted",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_backdrop_source_groups_requested",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_bind_group_switches", PerfKeyAggregate::Max),
    count_key(
        "renderer_custom_effect_v1_passes_emitted",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v1_steps_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v2_passes_emitted",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v2_steps_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v2_user_image_incompatible_fallbacks",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_passes_emitted",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_pyramid_cache_hits",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_pyramid_cache_misses",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_pyramid_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_raw_aliased_to_src",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_raw_distinct",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_sources_raw_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_steps_requested",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_user0_image_incompatible_fallbacks",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_custom_effect_v3_user1_image_incompatible_fallbacks",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_draw_calls", PerfKeyAggregate::Max),
    id_key("renderer_frame_id"),
    byte_key(
        "renderer_geometry_upload_path_paint_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_path_paint_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_path_vertex_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_path_vertex_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_resident_dirty_range_bytes_estimate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_stream_coverage_gaps",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_resident_partial_write_dry_run_bytes_estimate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_partial_write_dry_run_streams",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_partial_write_dry_run_write_count_estimate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_buffer_resized",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_missing_payload",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_no_candidate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_reassembly_blocked",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_stream_content_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_stream_layout_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_full_upload_fallbacks_uninitialized",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_stream_candidates",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_stream_content_mismatches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_stream_hits",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_resident_stream_misses",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_quad_instance_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_quad_instance_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_text_glyph_instance_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_text_glyph_instance_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_text_paint_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_text_paint_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_text_vertex_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_text_vertex_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_geometry_upload_viewport_vertex_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_geometry_upload_viewport_vertex_write_count",
        PerfKeyAggregate::Max,
    ),
    byte_key("renderer_image_upload_bytes", PerfKeyAggregate::Max),
    byte_key("renderer_intermediate_budget_bytes", PerfKeyAggregate::Max),
    byte_key(
        "renderer_intermediate_full_target_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key("renderer_intermediate_in_use_bytes", PerfKeyAggregate::Max),
    byte_key(
        "renderer_intermediate_peak_in_use_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_intermediate_pool_allocations",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_intermediate_pool_evictions",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_intermediate_pool_free_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_intermediate_pool_free_textures",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_intermediate_pool_releases", PerfKeyAggregate::Max),
    count_key("renderer_intermediate_pool_reuses", PerfKeyAggregate::Max),
    count_key(
        "renderer_intermediate_release_targets",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_material_degraded_due_to_budget",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_material_distinct", PerfKeyAggregate::Max),
    count_key("renderer_material_quad_ops", PerfKeyAggregate::Max),
    count_key("renderer_material_sampled_quad_ops", PerfKeyAggregate::Max),
    count_key("renderer_material_unknown_ids", PerfKeyAggregate::Max),
    count_key("renderer_pipeline_switches", PerfKeyAggregate::Max),
    count_key(
        "renderer_render_plan_custom_effect_chain_base_required_full_targets_max",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_base_required_max_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_custom_effect_chain_budget_samples",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_effective_budget_max_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_effective_budget_min_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_optional_mask_max_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_optional_required_max_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_custom_effect_chain_other_live_max_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_effect_chain_budget_samples",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_effect_chain_effective_budget_max_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_effect_chain_effective_budget_min_bytes",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_effect_chain_other_live_max_bytes",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_cpu_upload",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_external_zero_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_fallbacks",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_gpu_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_owned",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_ingest_unknown",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_requested_ingest_cpu_upload",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_requested_ingest_external_zero_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_requested_ingest_gpu_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_requested_ingest_owned",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_target_updates_requested_ingest_unknown",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_scene_chunk_input_chunks", PerfKeyAggregate::Max),
    count_key("renderer_scene_chunk_input_ops", PerfKeyAggregate::Max),
    count_key(
        "renderer_scene_chunk_input_fingerprint",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_key_cache_entries",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_key_cache_hits",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_key_cache_misses",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_key_cache_stale_entries",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_key_cache_context_fingerprint",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_cache_hits",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_cache_misses",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_chunks_encoded",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_scene_chunk_encoding_payload_bytes_estimate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_entries_live",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_candidate_segments",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_shape_matches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_shape_mismatches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_stream_fingerprint_matches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_dry_run_candidates",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_append_only_matches",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_shape_mismatch",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_non_quad_draws",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_side_tables",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_material_state",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_entries_without_plan_candidate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_chunk_encoding_payload_plan_candidates_without_payload",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_scene_chunk_candidates",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_scene_chunk_candidate_draws",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_scene_chunk_candidates_stable",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_scene_chunk_candidates_changed",
        PerfKeyAggregate::Max,
    ),
    byte_key(
        "renderer_render_plan_scene_chunk_candidate_upload_bytes_estimate",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_render_plan_scene_chunk_candidate_stream_ranges_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_misses",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_cold_start",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_custom_effects_generation_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_format_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_images_generation_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_material_distinct_budget_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_material_paint_budget_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_materials_generation_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_render_targets_generation_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_scale_factor_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_scene_fingerprint_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_scene_ops_len_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_text_atlas_revision_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_text_quality_key_changed",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_scene_encoding_cache_miss_viewport_size_changed",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_scissor_sets", PerfKeyAggregate::Max),
    byte_key("renderer_svg_mask_atlas_bytes_live", PerfKeyAggregate::Max),
    pixel_key("renderer_svg_mask_atlas_capacity_px", PerfKeyAggregate::Max),
    count_key(
        "renderer_svg_mask_atlas_entries_evicted",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_svg_mask_atlas_page_evictions",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_svg_mask_atlas_pages_live", PerfKeyAggregate::Max),
    pixel_key("renderer_svg_mask_atlas_used_px", PerfKeyAggregate::Max),
    byte_key("renderer_svg_raster_budget_bytes", PerfKeyAggregate::Max),
    count_key(
        "renderer_svg_raster_budget_evictions",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_svg_raster_cache_hits", PerfKeyAggregate::Max),
    count_key("renderer_svg_raster_cache_misses", PerfKeyAggregate::Max),
    count_key("renderer_svg_rasters_live", PerfKeyAggregate::Max),
    byte_key("renderer_svg_standalone_bytes_live", PerfKeyAggregate::Max),
    byte_key("renderer_svg_upload_bytes", PerfKeyAggregate::Max),
    count_key("renderer_text_atlas_evicted_pages", PerfKeyAggregate::Max),
    count_key(
        "renderer_text_atlas_revision_changed_scene_text_resources_stable",
        PerfKeyAggregate::Max,
    ),
    byte_key("renderer_text_atlas_upload_bytes", PerfKeyAggregate::Max),
    count_key("renderer_text_scene_resource_blobs", PerfKeyAggregate::Max),
    id_key("renderer_text_scene_resource_fingerprint"),
    count_key(
        "renderer_text_scene_resource_fingerprint_changed",
        PerfKeyAggregate::Max,
    ),
    count_key("renderer_text_scene_resource_glyphs", PerfKeyAggregate::Max),
    count_key(
        "renderer_text_scene_resource_missing_glyph_resources",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_text_scene_resource_reset_generation",
        PerfKeyAggregate::Max,
    ),
    id_key("renderer_tick_id"),
    count_key("renderer_viewport_draw_calls", PerfKeyAggregate::Max),
    count_key(
        "renderer_viewport_draw_calls_ingest_cpu_upload",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_viewport_draw_calls_ingest_external_zero_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_viewport_draw_calls_ingest_gpu_copy",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_viewport_draw_calls_ingest_owned",
        PerfKeyAggregate::Max,
    ),
    count_key(
        "renderer_viewport_draw_calls_ingest_unknown",
        PerfKeyAggregate::Max,
    ),
    count_key("set_children_barrier_writes", PerfKeyAggregate::Max),
    count_key("view_cache_contained_relayouts", PerfKeyAggregate::Max),
    count_key("view_cache_roots_cache_key_mismatch", PerfKeyAggregate::Max),
    count_key("view_cache_roots_first_mount", PerfKeyAggregate::Max),
    count_key("view_cache_roots_layout_invalidated", PerfKeyAggregate::Max),
    count_key("view_cache_roots_manual", PerfKeyAggregate::Max),
    count_key("view_cache_roots_needs_rerender", PerfKeyAggregate::Max),
    count_key("view_cache_roots_node_recreated", PerfKeyAggregate::Max),
    count_key(
        "view_cache_roots_not_marked_reuse_root",
        PerfKeyAggregate::Max,
    ),
    count_key("view_cache_roots_reused", PerfKeyAggregate::Max),
    count_key("view_cache_roots_total", PerfKeyAggregate::Max),
    count_key("virtual_list_visible_range_checks", PerfKeyAggregate::Max),
    count_key(
        "virtual_list_visible_range_refreshes",
        PerfKeyAggregate::Max,
    ),
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
        "coverage": "debug_stats_consumed_by_diag_stats",
        "complete": true,
        "note": "Complete registry for debug.stats frame fields consumed by diag stats and perf gates. Derived report-only fields and app_snapshot-specific payloads remain outside this frame-stats inventory.",
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "keys": registered_frame_stats_keys_json(),
    })
}

pub(crate) fn perf_threshold_keys_json() -> Value {
    Value::Array(
        PERF_THRESHOLD_KEYS
            .iter()
            .map(|key| {
                let mut obj = serde_json::json!({
                    "key": key.key,
                    "metric": key.metric,
                    "unit": key.unit.as_str(),
                    "direction": key.direction.as_str(),
                    "scope": key.scope,
                    "observed_aggregate": key.observed_aggregate,
                });
                if let Some(source_metric) = key.source_metric {
                    obj["source_metric"] = Value::from(source_metric);
                }
                obj
            })
            .collect(),
    )
}

pub(crate) fn perf_threshold_inventory_json() -> Value {
    serde_json::json!({
        "schema_version": PERF_KEY_REGISTRY_SCHEMA_VERSION,
        "kind": PERF_THRESHOLD_KEY_REGISTRY_KIND,
        "scope": "diag_perf_thresholds",
        "coverage": "diag_perf_threshold_config_keys",
        "complete": true,
        "note": "Registry for diag perf threshold configuration keys. Each entry maps a CLI/baseline threshold key to the observed metric used in check.perf_thresholds.json failure rows.",
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "keys": perf_threshold_keys_json(),
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

const fn threshold_max_us_key(
    key: &'static str,
    metric: &'static str,
    scope: &'static str,
) -> PerfThresholdKey {
    PerfThresholdKey {
        key,
        metric,
        source_metric: None,
        unit: PerfKeyUnit::Microseconds,
        direction: PerfThresholdDirection::Max,
        scope,
        observed_aggregate: "max",
    }
}

const fn threshold_max_count_key(
    key: &'static str,
    metric: &'static str,
    scope: &'static str,
) -> PerfThresholdKey {
    PerfThresholdKey {
        key,
        metric,
        source_metric: None,
        unit: PerfKeyUnit::Count,
        direction: PerfThresholdDirection::Max,
        scope,
        observed_aggregate: "max",
    }
}

const fn threshold_max_bytes_key(
    key: &'static str,
    metric: &'static str,
    scope: &'static str,
) -> PerfThresholdKey {
    PerfThresholdKey {
        key,
        metric,
        source_metric: None,
        unit: PerfKeyUnit::Bytes,
        direction: PerfThresholdDirection::Max,
        scope,
        observed_aggregate: "max",
    }
}

const fn pixel_key(key: &'static str, suggested_aggregate: PerfKeyAggregate) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Pixels,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::Frame,
        suggested_aggregate,
        trace: None,
    }
}

const fn flag_key(key: &'static str) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Boolean,
        kind: PerfKeyKind::Flag,
        scope: PerfKeyScope::Frame,
        suggested_aggregate: PerfKeyAggregate::Any,
        trace: None,
    }
}

const fn id_key(key: &'static str) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Id,
        kind: PerfKeyKind::Identifier,
        scope: PerfKeyScope::Frame,
        suggested_aggregate: PerfKeyAggregate::None,
        trace: None,
    }
}

const fn label_key(key: &'static str) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Label,
        kind: PerfKeyKind::Label,
        scope: PerfKeyScope::Frame,
        suggested_aggregate: PerfKeyAggregate::None,
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

    fn key_set(keys: &[PerfKey]) -> std::collections::BTreeSet<&'static str> {
        keys.iter().map(|key| key.key).collect()
    }

    fn threshold_key_set() -> std::collections::BTreeSet<&'static str> {
        PERF_THRESHOLD_KEYS.iter().map(|key| key.key).collect()
    }

    fn scan_struct_option_fields<'a>(
        source: &'a str,
        struct_name: &str,
    ) -> std::collections::BTreeSet<&'a str> {
        let mut fields = std::collections::BTreeSet::new();
        let Some(start) = source.find(&format!("struct {struct_name}")) else {
            return fields;
        };
        let source = &source[start..];
        let Some(open) = source.find('{') else {
            return fields;
        };
        let source = &source[open + 1..];
        let Some(close) = source.find("}\n") else {
            return fields;
        };
        for line in source[..close].lines() {
            let line = line.trim();
            if !line.contains("Option<u64>") {
                continue;
            }
            let Some(name_start) = line.rfind(' ') else {
                continue;
            };
            let Some(name_end) = line[name_start + 1..].find(':') else {
                continue;
            };
            fields.insert(&line[name_start + 1..name_start + 1 + name_end]);
        }
        fields
    }

    fn scan_cli_perf_threshold_fields() -> std::collections::BTreeSet<&'static str> {
        scan_struct_option_fields(
            include_str!("cli/contracts/commands/perf.rs"),
            "PerfCommandArgs",
        )
        .into_iter()
        .filter(|key: &&str| {
            key.starts_with("max_") || key.starts_with("min_run_paint_cache_hit_test_only_replay")
        })
        .collect()
    }

    fn consumed_debug_stats_keys_from_bundle_stats_compute_source()
    -> std::collections::BTreeSet<&'static str> {
        let mut keys = std::collections::BTreeSet::new();
        let mut source = include_str!("stats/bundle_stats_compute.inc.rs");
        while let Some(start) = source.find("m.get(\"") {
            if start > 0
                && source
                    .as_bytes()
                    .get(start - 1)
                    .is_some_and(u8::is_ascii_alphanumeric)
            {
                source = &source[start + "m.get(\"".len()..];
                continue;
            }
            source = &source[start + "m.get(\"".len()..];
            let Some(end) = source.find('"') else {
                break;
            };
            keys.insert(&source[..end]);
            source = &source[end + 1..];
        }
        keys
    }

    fn assert_units_match_names(keys: &[PerfKey]) {
        for key in keys {
            if key.key.ends_with("_time_us") || key.key.ends_with("_us") {
                assert_eq!(key.unit, PerfKeyUnit::Microseconds, "{}", key.key);
            }
            if key.key.ends_with("_cycles") {
                assert_eq!(key.unit, PerfKeyUnit::Cycles, "{}", key.key);
            }
            if key.key.ends_with("_bytes") || key.key.contains("_bytes_") {
                assert_eq!(key.unit, PerfKeyUnit::Bytes, "{}", key.key);
            }
            if key.key.ends_with("_px") {
                assert_eq!(key.unit, PerfKeyUnit::Pixels, "{}", key.key);
            }
            if key.key.ends_with("_id") || key.key.ends_with("_token") {
                assert_eq!(key.unit, PerfKeyUnit::Id, "{}", key.key);
            }
            if matches!(key.kind, PerfKeyKind::Flag) {
                assert_eq!(key.unit, PerfKeyUnit::Boolean, "{}", key.key);
                assert_eq!(
                    key.suggested_aggregate,
                    PerfKeyAggregate::Any,
                    "{}",
                    key.key
                );
            }
            if matches!(key.kind, PerfKeyKind::Identifier) {
                assert_eq!(
                    key.suggested_aggregate,
                    PerfKeyAggregate::None,
                    "{}",
                    key.key
                );
            }
            if matches!(key.kind, PerfKeyKind::Label) {
                assert_eq!(key.unit, PerfKeyUnit::Label, "{}", key.key);
                assert_eq!(
                    key.suggested_aggregate,
                    PerfKeyAggregate::None,
                    "{}",
                    key.key
                );
            }
            assert!(!key.unit.as_str().is_empty());
        }
    }

    fn assert_threshold_units_match_names(keys: &[PerfThresholdKey]) {
        for key in keys {
            if key.key.ends_with("_us")
                || key.metric.ends_with("_time_us")
                || key.metric.ends_with("_us")
            {
                assert_eq!(key.unit, PerfKeyUnit::Microseconds, "{}", key.key);
            }
            if key.key.ends_with("_bytes") || key.metric.ends_with("_bytes") {
                assert_eq!(key.unit, PerfKeyUnit::Bytes, "{}", key.key);
            }
            if key.key.ends_with("_ops")
                || key.key.ends_with("_changes")
                || key.key.ends_with("_max")
                || key.metric.ends_with("_ops")
                || key.metric.ends_with("_changes")
                || key.metric.ends_with("_max")
            {
                assert_eq!(key.unit, PerfKeyUnit::Count, "{}", key.key);
            }
            if key.key.starts_with("min_") {
                assert_eq!(key.direction, PerfThresholdDirection::Min, "{}", key.key);
            }
            if key.key.starts_with("max_") {
                assert_eq!(key.direction, PerfThresholdDirection::Max, "{}", key.key);
            }
            assert!(!key.metric.is_empty(), "{}", key.key);
            assert!(!key.scope.is_empty(), "{}", key.key);
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
    fn perf_threshold_keys_are_unique() {
        let mut keys: Vec<&str> = PERF_THRESHOLD_KEYS.iter().map(|key| key.key).collect();
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
    fn perf_threshold_key_units_match_names() {
        assert_threshold_units_match_names(PERF_THRESHOLD_KEYS);
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
        let keys = key_set(REGISTERED_FRAME_STATS_KEYS);
        for expected in [
            "total_time_us",
            "layout_time_us",
            "frame_arena_capacity_estimate_bytes",
            "element_children_vec_pool_grow_events",
            "identity_resolve_fallback_scans",
            "identity_resolve_fallback_scan_nodes",
            "parent_pointer_repairs",
            "gc_reachability_layer_nodes",
            "gc_stale_removed",
            "dispatch_snapshot_cache_misses",
            "dispatch_snapshot_built_nodes",
            "dirty_frontier_boundaries_max",
            "dirty_frontier_boundaries_at_layout_start",
            "dirty_frontier_contained_candidates",
            "model_observation_index_edges_added",
            "model_observation_index_edges_removed",
            "global_observation_index_edges_added",
            "global_observation_index_edges_removed",
            "layout_pending_barrier_relayouts_time_us",
            "layout_repair_view_cache_bounds_time_us",
            "layout_contained_view_cache_roots_time_us",
            "layout_engine_child_rect_time_us",
            "layout_fast_path_taken",
            "view_cache_roots_cache_key_mismatch",
            "invalidation_walk_nodes_hover",
            "paint_input_context_time_us",
            "paint_publish_text_input_snapshot_time_us",
            "paint_text_prepare_reason_font_stack_changed",
            "dispatch_accounted_time_us",
            "dispatch_pointer_move_layer_observers_time_us",
            "dispatch_timer_slowest_token",
            "window_runtime_snapshot_command_availability_eval_time_us",
            "hit_test_bounds_tree_query_time_us",
            "renderer_encode_scene_us",
            "renderer_upload_us",
            "renderer_record_passes_us",
            "renderer_encoder_finish_us",
            "renderer_prepare_text_us",
            "renderer_prepare_svg_us",
            "renderer_instance_bytes",
            "renderer_geometry_upload_quad_instance_bytes",
            "renderer_geometry_upload_resident_dirty_range_bytes_estimate",
            "renderer_geometry_upload_resident_stream_coverage_gaps",
            "renderer_geometry_upload_resident_partial_write_dry_run_bytes_estimate",
            "renderer_geometry_upload_resident_stream_content_mismatches",
            "renderer_geometry_upload_resident_stream_hits",
            "renderer_geometry_upload_resident_full_upload_fallbacks_reassembly_blocked",
            "renderer_geometry_upload_text_vertex_write_count",
            "renderer_scene_encoding_cache_miss_scene_fingerprint_changed",
            "renderer_scene_chunk_input_chunks",
            "renderer_scene_chunk_encoding_key_cache_entries",
            "renderer_scene_chunk_encoding_key_cache_hits",
            "renderer_scene_chunk_encoding_payload_cache_hits",
            "renderer_scene_chunk_encoding_payload_bytes_estimate",
            "renderer_scene_chunk_encoding_payload_plan_shape_matches",
            "renderer_scene_chunk_encoding_payload_plan_shape_mismatches",
            "renderer_scene_chunk_encoding_payload_plan_stream_fingerprint_matches",
            "renderer_scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches",
            "renderer_scene_chunk_encoding_payload_reassembly_dry_run_candidates",
            "renderer_scene_chunk_encoding_payload_reassembly_append_only_matches",
            "renderer_scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch",
            "renderer_render_plan_scene_chunk_candidates_changed",
            "renderer_render_plan_scene_chunk_candidate_upload_bytes_estimate",
            "renderer_text_scene_resource_fingerprint_changed",
            "renderer_text_atlas_revision_changed_scene_text_resources_stable",
            "renderer_encode_scene_text_ops",
            "renderer_render_target_updates_ingest_gpu_copy",
            "renderer_svg_mask_atlas_capacity_px",
            "renderer_custom_effect_v3_pyramid_cache_hits",
            "renderer_intermediate_peak_in_use_bytes",
            "pointer_move.max_dispatch_time_us",
            "pointer_move.max_hit_test_time_us",
        ] {
            assert!(keys.contains(expected), "missing perf key: {expected}");
        }
    }

    #[test]
    fn full_registered_perf_key_registry_covers_consumed_debug_stats_fields() {
        let registered = key_set(REGISTERED_FRAME_STATS_KEYS);
        let consumed = consumed_debug_stats_keys_from_bundle_stats_compute_source();
        let missing: Vec<&str> = consumed.difference(&registered).copied().collect();
        assert!(
            missing.is_empty(),
            "debug.stats fields consumed by diag stats are missing from perf key registry: {missing:?}"
        );
    }

    #[test]
    fn perf_threshold_registry_covers_threshold_struct_fields() {
        let registered = threshold_key_set();
        let fields = scan_struct_option_fields(include_str!("compare.rs"), "PerfThresholds");
        let missing: Vec<&str> = fields.difference(&registered).copied().collect();
        assert!(
            missing.is_empty(),
            "PerfThresholds fields missing from threshold registry: {missing:?}"
        );
    }

    #[test]
    fn perf_threshold_registry_covers_diag_perf_cli_flags() {
        let registered = threshold_key_set();
        let fields = scan_cli_perf_threshold_fields();
        let missing: Vec<&str> = fields.difference(&registered).copied().collect();
        assert!(
            missing.is_empty(),
            "diag perf CLI threshold fields missing from threshold registry: {missing:?}"
        );
    }

    #[test]
    fn perf_threshold_registry_metric_names_are_frame_stats_or_known_derived_metrics() {
        let frame_stats = key_set(REGISTERED_FRAME_STATS_KEYS);
        let known_derived = [
            "top_total_time_us",
            "top_layout_time_us",
            "top_layout_engine_solve_time_us",
            "frame_p95_total_time_us",
            "frame_p95_layout_time_us",
            "frame_p95_layout_engine_solve_time_us",
            "pointer_move_max_dispatch_time_us",
            "pointer_move_max_hit_test_time_us",
            "pointer_move_snapshots_with_global_changes",
            "run_paint_cache_hit_test_only_replay_allowed_max",
            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
        let missing: Vec<&str> = PERF_THRESHOLD_KEYS
            .iter()
            .map(|key| key.metric)
            .filter(|metric| !frame_stats.contains(metric) && !known_derived.contains(metric))
            .collect();
        assert!(
            missing.is_empty(),
            "threshold registry metrics are not registered frame stats or known derived metrics: {missing:?}"
        );
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

    #[test]
    fn perf_threshold_key_inventory_doc_is_in_sync() {
        let expected = perf_threshold_inventory_json();
        let doc: Value = serde_json::from_str(include_str!(
            "../../../docs/workstreams/diag-perf-profiling-infra-v1/perf-threshold-key-registry.diag-perf.json"
        ))
        .expect("parse perf threshold key registry inventory doc");
        assert_eq!(doc, expected);
    }
}
