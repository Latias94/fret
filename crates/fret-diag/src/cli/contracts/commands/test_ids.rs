use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct TestIdsCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "max-test-ids", value_name = "N", default_value_t = 200)]
    pub max_test_ids: usize,
}
