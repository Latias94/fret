use std::path::PathBuf;

use clap::{ArgAction, Args};

use super::super::shared::{DevtoolsArgs, LaunchArgs, OutputArgs, TimingArgs};

fn parse_bundle_stats_sort(raw: &str) -> Result<crate::BundleStatsSort, String> {
    crate::BundleStatsSort::parse(raw)
}

#[derive(Debug, Args)]
pub(crate) struct PerfCommandArgs {
    #[arg(value_name = "TARGET", num_args = 1.., required = true)]
    pub targets: Vec<String>,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub devtools: DevtoolsArgs,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(
        long = "sort",
        value_name = "SORT",
        value_parser = parse_bundle_stats_sort
    )]
    pub sort: Option<crate::BundleStatsSort>,

    #[arg(long = "repeat", default_value_t = 1)]
    pub repeat: u64,

    #[arg(long = "prewarm-script", value_name = "PATH", action = ArgAction::Append)]
    pub prewarm_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-script", value_name = "PATH", action = ArgAction::Append)]
    pub prelude_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-each-run")]
    pub prelude_each_run: bool,

    #[arg(long = "reuse-launch")]
    pub reuse_launch: bool,

    #[arg(long = "reuse-launch-per-script")]
    pub reuse_launch_per_script: bool,

    #[arg(long = "trace")]
    pub trace_chrome: bool,

    #[arg(long = "check-perf-hints")]
    pub check_perf_hints: bool,

    #[arg(
        long = "check-perf-hints-deny",
        value_name = "CODE[,CODE...]",
        action = ArgAction::Append
    )]
    pub check_perf_hints_deny: Vec<String>,

    #[arg(
        long = "check-perf-hints-min-severity",
        value_name = "LEVEL",
        value_parser = ["info", "warn", "error"]
    )]
    pub check_perf_hints_min_severity: Option<String>,

    #[arg(long = "perf-threshold-agg", value_name = "AGG")]
    pub perf_threshold_agg: Option<crate::PerfThresholdAggregate>,

    #[arg(long = "max-top-total-us")]
    pub max_top_total_us: Option<u64>,

    #[arg(long = "max-top-layout-us")]
    pub max_top_layout_us: Option<u64>,

    #[arg(long = "max-top-solve-us")]
    pub max_top_solve_us: Option<u64>,

    #[arg(long = "max-frame-p95-total-us")]
    pub max_frame_p95_total_us: Option<u64>,

    #[arg(long = "max-frame-p95-layout-us")]
    pub max_frame_p95_layout_us: Option<u64>,

    #[arg(long = "max-frame-p95-solve-us")]
    pub max_frame_p95_solve_us: Option<u64>,

    #[arg(long = "max-pointer-move-dispatch-us")]
    pub max_pointer_move_dispatch_us: Option<u64>,

    #[arg(long = "max-pointer-move-hit-test-us")]
    pub max_pointer_move_hit_test_us: Option<u64>,

    #[arg(long = "max-pointer-move-global-changes")]
    pub max_pointer_move_global_changes: Option<u64>,

    #[arg(long = "min-run-paint-cache-hit-test-only-replay-allowed-max")]
    pub min_run_paint_cache_hit_test_only_replay_allowed_max: Option<u64>,

    #[arg(long = "max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max")]
    pub max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64>,

    #[arg(long = "perf-baseline", value_name = "PATH")]
    pub perf_baseline_path: Option<PathBuf>,

    #[arg(long = "perf-baseline-out", value_name = "PATH")]
    pub perf_baseline_out: Option<PathBuf>,

    #[arg(long = "perf-baseline-headroom-pct", default_value_t = 20)]
    pub perf_baseline_headroom_pct: u32,

    #[arg(
        long = "perf-baseline-seed-preset",
        value_name = "PATH",
        action = ArgAction::Append
    )]
    pub perf_baseline_seed_preset_paths: Vec<PathBuf>,

    #[arg(
        long = "perf-baseline-seed",
        value_name = "SCOPE@METRIC=AGG",
        action = ArgAction::Append
    )]
    pub perf_baseline_seed_specs: Vec<String>,

    #[arg(long = "check-pixels-changed", value_name = "TEST_ID")]
    pub check_pixels_changed: Option<String>,

    #[arg(long = "check-pixels-unchanged", value_name = "TEST_ID")]
    pub check_pixels_unchanged: Option<String>,

    #[command(flatten)]
    pub launch: LaunchArgs,
}
