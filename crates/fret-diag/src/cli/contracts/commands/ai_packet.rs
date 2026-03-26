use std::path::PathBuf;

use clap::Args;

use super::super::shared::WarmupFramesArgs;

#[derive(Debug, Args)]
pub(crate) struct AiPacketCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[arg(long = "packet-out", value_name = "DIR")]
    pub packet_out: Option<PathBuf>,

    #[arg(long = "test-id", value_name = "TEST_ID")]
    pub test_id: Option<String>,

    #[arg(long = "sidecars-only")]
    pub sidecars_only: bool,

    #[arg(long = "include-triage")]
    pub include_triage: bool,
}
