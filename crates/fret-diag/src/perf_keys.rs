use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfKeyUnit {
    Microseconds,
    Cycles,
}

impl PerfKeyUnit {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Microseconds => "us",
            Self::Cycles => "cycles",
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
}

impl PerfKeyScope {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
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
pub(crate) struct PerfKey {
    pub(crate) key: &'static str,
    pub(crate) unit: PerfKeyUnit,
    pub(crate) kind: PerfKeyKind,
    pub(crate) scope: PerfKeyScope,
    pub(crate) suggested_aggregate: PerfKeyAggregate,
    pub(crate) trace_event: &'static str,
    pub(crate) trace_category: &'static str,
}

pub(crate) const TOTAL_TIME_US: PerfKey = timing_key("total_time_us", "fret.frame", "frame");
pub(crate) const LAYOUT_TIME_US: PerfKey = timing_key("layout_time_us", "layout", "layout");
pub(crate) const PREPAINT_TIME_US: PerfKey = timing_key("prepaint_time_us", "prepaint", "prepaint");
pub(crate) const PAINT_TIME_US: PerfKey = timing_key("paint_time_us", "paint", "paint");
pub(crate) const DISPATCH_TIME_US: PerfKey = timing_key("dispatch_time_us", "dispatch", "dispatch");
pub(crate) const HIT_TEST_TIME_US: PerfKey = timing_key("hit_test_time_us", "hit_test", "hit_test");
pub(crate) const UI_THREAD_CPU_TIME_US: PerfKey =
    timing_key("ui_thread_cpu_time_us", "ui_thread_cpu_time", "cpu");
pub(crate) const UI_THREAD_CPU_CYCLE_TIME_DELTA_CYCLES: PerfKey = counter_key(
    "ui_thread_cpu_cycle_time_delta_cycles",
    PerfKeyUnit::Cycles,
    "ui_thread_cpu_cycle_delta",
    "cpu",
);
pub(crate) const UI_THREAD_CPU_CYCLE_TIME_TOTAL_CYCLES: PerfKey = counter_key(
    "ui_thread_cpu_cycle_time_total_cycles",
    PerfKeyUnit::Cycles,
    "ui_thread_cpu_cycle_total",
    "cpu",
);

pub(crate) const LAYOUT_OBSERVATION_RECORD_TIME_US: PerfKey = timing_key(
    "layout_observation_record_time_us",
    "layout.obs_record",
    "layout",
);
pub(crate) const LAYOUT_COLLECT_ROOTS_TIME_US: PerfKey = timing_key(
    "layout_collect_roots_time_us",
    "layout.collect_roots",
    "layout",
);
pub(crate) const LAYOUT_INVALIDATE_SCROLL_HANDLE_BINDINGS_TIME_US: PerfKey = timing_key(
    "layout_invalidate_scroll_handle_bindings_time_us",
    "layout.invalidate_scroll_bindings",
    "layout",
);
pub(crate) const LAYOUT_EXPAND_VIEW_CACHE_INVALIDATIONS_TIME_US: PerfKey = timing_key(
    "layout_expand_view_cache_invalidations_time_us",
    "layout.expand_view_cache_invalidations",
    "layout",
);
pub(crate) const LAYOUT_REQUEST_BUILD_ROOTS_TIME_US: PerfKey = timing_key(
    "layout_request_build_roots_time_us",
    "layout.request_build_roots",
    "layout",
);
pub(crate) const LAYOUT_ROOTS_TIME_US: PerfKey =
    timing_key("layout_roots_time_us", "layout.roots", "layout");
pub(crate) const LAYOUT_VIEW_CACHE_TIME_US: PerfKey =
    timing_key("layout_view_cache_time_us", "layout.view_cache", "layout");
pub(crate) const LAYOUT_ENGINE_SOLVE_TIME_US: PerfKey = timing_key(
    "layout_engine_solve_time_us",
    "layout.engine_solve",
    "layout",
);

pub(crate) const PAINT_OBSERVATION_RECORD_TIME_US: PerfKey = timing_key(
    "paint_observation_record_time_us",
    "paint.obs_record",
    "paint",
);
pub(crate) const PAINT_TEXT_PREPARE_TIME_US: PerfKey =
    timing_key("paint_text_prepare_time_us", "paint.text_prepare", "paint");
pub(crate) const PAINT_RECORD_VISUAL_BOUNDS_TIME_US: PerfKey = timing_key(
    "paint_record_visual_bounds_time_us",
    "paint.record_visual_bounds",
    "paint",
);
pub(crate) const PAINT_CACHE_KEY_TIME_US: PerfKey =
    timing_key("paint_cache_key_time_us", "paint.cache_key", "paint");
pub(crate) const PAINT_CACHE_HIT_CHECK_TIME_US: PerfKey = timing_key(
    "paint_cache_hit_check_time_us",
    "paint.cache_hit_check",
    "paint",
);
pub(crate) const PAINT_CACHE_REPLAY_TIME_US: PerfKey =
    timing_key("paint_cache_replay_time_us", "paint.cache_replay", "paint");
pub(crate) const PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US: PerfKey = timing_key(
    "paint_cache_bounds_translate_time_us",
    "paint.cache_bounds_translate",
    "paint",
);
pub(crate) const PAINT_WIDGET_TIME_US: PerfKey =
    timing_key("paint_widget_time_us", "paint.widget", "paint");

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

pub(crate) fn read_u64(stats: Option<&Map<String, Value>>, key: PerfKey) -> u64 {
    stats
        .and_then(|m| m.get(key.key))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

pub(crate) fn trace_exported_frame_keys_json() -> Value {
    Value::Array(
        TRACE_EXPORTED_FRAME_KEYS
            .iter()
            .map(|key| {
                serde_json::json!({
                    "key": key.key,
                    "unit": key.unit.as_str(),
                    "kind": key.kind.as_str(),
                    "scope": key.scope.as_str(),
                    "suggested_aggregate": key.suggested_aggregate.as_str(),
                    "trace_event": key.trace_event,
                    "trace_category": key.trace_category,
                })
            })
            .collect(),
    )
}

const fn timing_key(
    key: &'static str,
    trace_event: &'static str,
    trace_category: &'static str,
) -> PerfKey {
    PerfKey {
        key,
        unit: PerfKeyUnit::Microseconds,
        kind: PerfKeyKind::Timing,
        scope: PerfKeyScope::Frame,
        suggested_aggregate: PerfKeyAggregate::Max,
        trace_event,
        trace_category,
    }
}

const fn counter_key(
    key: &'static str,
    unit: PerfKeyUnit,
    trace_event: &'static str,
    trace_category: &'static str,
) -> PerfKey {
    PerfKey {
        key,
        unit,
        kind: PerfKeyKind::Counter,
        scope: PerfKeyScope::Frame,
        suggested_aggregate: PerfKeyAggregate::P95,
        trace_event,
        trace_category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_exported_perf_keys_are_unique() {
        let mut keys: Vec<&str> = TRACE_EXPORTED_FRAME_KEYS
            .iter()
            .map(|key| key.key)
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), TRACE_EXPORTED_FRAME_KEYS.len());
    }

    #[test]
    fn trace_exported_perf_key_units_match_names() {
        for key in TRACE_EXPORTED_FRAME_KEYS {
            if key.key.ends_with("_time_us") || key.key.ends_with("_us") {
                assert_eq!(key.unit, PerfKeyUnit::Microseconds, "{}", key.key);
            }
            if key.key.ends_with("_cycles") {
                assert_eq!(key.unit, PerfKeyUnit::Cycles, "{}", key.key);
            }
            assert!(!key.unit.as_str().is_empty());
            assert_eq!(key.scope, PerfKeyScope::Frame);
            assert!(!key.trace_event.is_empty());
            assert!(!key.trace_category.is_empty());
        }
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
}
