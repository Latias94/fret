use std::path::PathBuf;

use clap::Args;

use super::super::shared::WarmupFramesArgs;

fn parse_bundle_stats_sort(raw: &str) -> Result<crate::BundleStatsSort, String> {
    crate::BundleStatsSort::parse(raw)
}

#[derive(Debug, Args)]
pub(crate) struct PerfBaselineFromBundlesCommandArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[arg(value_name = "BUNDLE", num_args = 1.., required = true)]
    pub bundle_artifacts: Vec<String>,

    #[arg(
        long = "sort",
        value_name = "SORT",
        value_parser = parse_bundle_stats_sort
    )]
    pub sort: Option<crate::BundleStatsSort>,

    #[arg(long = "perf-baseline-out", value_name = "PATH", required = true)]
    pub perf_baseline_out: PathBuf,

    #[arg(long = "perf-baseline-headroom-pct", default_value_t = 20)]
    pub perf_baseline_headroom_pct: u32,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[arg(long = "json")]
    pub json: bool,
}
