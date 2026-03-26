use clap::Args;

use super::super::shared::{CompareArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct CompareCommandArgs {
    #[arg(value_name = "SOURCE_A")]
    pub source_a: String,

    #[arg(value_name = "SOURCE_B")]
    pub source_b: String,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub compare: CompareArgs,

    #[arg(long = "footprint")]
    pub footprint: bool,

    #[arg(long = "json")]
    pub json: bool,
}
