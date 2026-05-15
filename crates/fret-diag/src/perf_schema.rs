pub(crate) const PERF_STATS_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_STATS_KIND: &str = "perf_stats";
pub(crate) const PERF_STATS_DIFF_KIND: &str = "perf_stats_diff";

pub(crate) const PERF_TRIAGE_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_TRIAGE_KIND: &str = "perf_triage";

pub(crate) const PERF_GATE_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_THRESHOLDS_KIND: &str = "perf_thresholds";
pub(crate) const PERF_HINTS_KIND: &str = "perf_hints";

pub(crate) const PERF_TRACE_SCHEMA_VERSION: u32 = 1;
pub(crate) const PERF_TRACE_CHROME_KIND: &str = "perf_trace_chrome";

pub(crate) fn schema_policy_json() -> serde_json::Value {
    serde_json::json!({
        "compatibility": "additive_only",
        "breaking_change": "requires_schema_version_bump_or_migration_plan",
        "field_renames": "require_compatibility_window",
    })
}
