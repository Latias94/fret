use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct LayoutPerfSummaryCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(
        long = "top",
        default_value_t = crate::layout_perf_summary::DEFAULT_LAYOUT_PERF_SUMMARY_TOP
    )]
    pub top: usize,
}
