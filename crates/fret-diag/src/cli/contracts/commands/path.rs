use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct PathCommandArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "trigger-path", value_name = "PATH")]
    pub trigger_path: Option<PathBuf>,
}
