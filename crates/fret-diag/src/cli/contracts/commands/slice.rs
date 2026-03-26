use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct SliceCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "test-id", value_name = "TEST_ID")]
    pub test_id: String,

    #[arg(long = "frame-id", conflicts_with = "snapshot_seq")]
    pub frame_id: Option<u64>,

    #[arg(long = "snapshot-seq", conflicts_with = "frame_id")]
    pub snapshot_seq: Option<u64>,

    #[arg(long = "window")]
    pub window: Option<u64>,

    #[arg(
        long = "step-index",
        conflicts_with_all = ["frame_id", "snapshot_seq"]
    )]
    pub step_index: Option<u32>,

    #[arg(long = "max-matches", default_value_t = 20)]
    pub max_matches: usize,

    #[arg(long = "max-ancestors", default_value_t = 64)]
    pub max_ancestors: usize,
}
