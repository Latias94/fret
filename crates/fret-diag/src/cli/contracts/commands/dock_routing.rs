use clap::Args;

use super::super::shared::WarmupFramesArgs;

#[derive(Debug, Args)]
pub(crate) struct DockRoutingCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[arg(long = "json")]
    pub json: bool,
}
