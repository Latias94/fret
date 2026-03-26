use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct LatestCommandArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,
}
