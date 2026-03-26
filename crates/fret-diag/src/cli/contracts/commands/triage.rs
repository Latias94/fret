use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

fn parse_bundle_stats_sort(raw: &str) -> Result<crate::BundleStatsSort, String> {
    crate::BundleStatsSort::parse(raw)
}

fn parse_triage_metric(raw: &str) -> Result<crate::frames_index::TriageLiteMetric, String> {
    match raw {
        "total" => Ok(crate::frames_index::TriageLiteMetric::TotalTimeUs),
        "layout" => Ok(crate::frames_index::TriageLiteMetric::LayoutTimeUs),
        "paint" => Ok(crate::frames_index::TriageLiteMetric::PaintTimeUs),
        _ => Err("invalid value for --metric (expected total|layout|paint)".to_string()),
    }
}

pub(crate) fn triage_metric_as_str(metric: crate::frames_index::TriageLiteMetric) -> &'static str {
    match metric {
        crate::frames_index::TriageLiteMetric::TotalTimeUs => "total",
        crate::frames_index::TriageLiteMetric::LayoutTimeUs => "layout",
        crate::frames_index::TriageLiteMetric::PaintTimeUs => "paint",
    }
}

#[derive(Debug, Args)]
pub(crate) struct TriageCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(
        long = "sort",
        value_name = "SORT",
        value_parser = parse_bundle_stats_sort
    )]
    pub sort: Option<crate::BundleStatsSort>,

    #[arg(long = "lite")]
    pub lite: bool,

    #[arg(long = "metric", value_name = "METRIC", value_parser = parse_triage_metric)]
    pub metric: Option<crate::frames_index::TriageLiteMetric>,
}
