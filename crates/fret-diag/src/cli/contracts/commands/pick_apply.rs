use std::path::PathBuf;

use clap::Args;

use super::super::shared::WaitArgs;

#[derive(Debug, Args)]
pub(crate) struct PickApplyCommandArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[command(flatten)]
    pub wait: WaitArgs,

    #[arg(long = "ptr", value_name = "JSON_POINTER")]
    pub ptr: String,

    #[arg(long = "out", value_name = "PATH")]
    pub out: Option<PathBuf>,
}
