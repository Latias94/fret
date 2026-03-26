use std::path::PathBuf;

use clap::Args;

use super::super::shared::WaitArgs;

#[derive(Debug, Args)]
pub(crate) struct PickScriptCommandArgs {
    #[command(flatten)]
    pub wait: WaitArgs,

    #[arg(long = "pick-script-out", value_name = "PATH")]
    pub pick_script_out: Option<PathBuf>,
}
