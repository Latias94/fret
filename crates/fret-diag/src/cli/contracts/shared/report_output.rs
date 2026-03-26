use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ReportOutputArgs {
    #[arg(long = "json")]
    pub json: bool,

    #[arg(long = "out", value_name = "PATH")]
    pub out: Option<PathBuf>,
}
