use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct TraceCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[arg(long = "trace-out", value_name = "PATH")]
    pub trace_out: Option<PathBuf>,
}
