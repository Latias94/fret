use clap::Args;

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct ExtensionsCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "key", value_name = "KEY")]
    pub key: Option<String>,

    #[arg(long = "print")]
    pub print: bool,
}
