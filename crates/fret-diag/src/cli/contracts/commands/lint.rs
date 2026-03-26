use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct LintCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "all-test-ids")]
    pub all_test_ids: bool,

    #[arg(long = "lint-eps-px", default_value_t = 0.5)]
    pub lint_eps_px: f32,
}
