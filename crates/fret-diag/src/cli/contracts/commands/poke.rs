use std::path::PathBuf;

use clap::Args;

use super::super::shared::WaitArgs;

#[derive(Debug, Args)]
pub(crate) struct PokeCommandArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "trigger-path", value_name = "PATH")]
    pub trigger_path: Option<PathBuf>,

    #[arg(long = "label", value_name = "LABEL")]
    pub label: Option<String>,

    #[arg(long = "max-snapshots", value_name = "N")]
    pub max_snapshots: Option<u32>,

    #[arg(long = "request-id", value_name = "ID")]
    pub request_id: Option<u64>,

    #[arg(long = "wait")]
    pub wait: bool,

    #[arg(long = "record-run")]
    pub record_run: bool,

    #[arg(long = "run-id", value_name = "ID", requires = "record_run")]
    pub run_id: Option<u64>,

    #[command(flatten)]
    pub wait_args: WaitArgs,
}
