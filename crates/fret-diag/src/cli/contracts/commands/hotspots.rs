use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

fn parse_hotspots_metric(raw: &str) -> Result<crate::frames_index::TriageLiteMetric, String> {
    match raw {
        "total" => Ok(crate::frames_index::TriageLiteMetric::TotalTimeUs),
        "layout" => Ok(crate::frames_index::TriageLiteMetric::LayoutTimeUs),
        "paint" => Ok(crate::frames_index::TriageLiteMetric::PaintTimeUs),
        _ => Err("invalid value for --metric (expected total|layout|paint)".to_string()),
    }
}

pub(crate) fn hotspots_metric_as_str(
    metric: crate::frames_index::TriageLiteMetric,
) -> &'static str {
    match metric {
        crate::frames_index::TriageLiteMetric::TotalTimeUs => "total",
        crate::frames_index::TriageLiteMetric::LayoutTimeUs => "layout",
        crate::frames_index::TriageLiteMetric::PaintTimeUs => "paint",
    }
}

#[derive(Debug, Args)]
pub(crate) struct HotspotsCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "hotspots-top", default_value_t = 20)]
    pub hotspots_top: usize,

    #[arg(long = "max-depth", default_value_t = 7)]
    pub max_depth: usize,

    #[arg(long = "min-bytes", default_value_t = 0)]
    pub min_bytes: u64,

    #[arg(long = "force")]
    pub force: bool,

    #[arg(long = "lite")]
    pub lite: bool,

    #[arg(
        long = "metric",
        value_name = "METRIC",
        value_parser = parse_hotspots_metric
    )]
    pub metric: Option<crate::frames_index::TriageLiteMetric>,
}
