use std::path::PathBuf;

use clap::Args;

use super::super::shared::OutputArgs;

#[derive(Debug, Args)]
pub(crate) struct DashboardCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<PathBuf>,

    #[command(flatten)]
    pub output: OutputArgs,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,
}
