use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct AgentCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,
}
