use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct OutputArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "json")]
    pub json: bool,
}
