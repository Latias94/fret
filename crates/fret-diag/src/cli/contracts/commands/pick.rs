use clap::Args;

use super::super::shared::WaitArgs;

#[derive(Debug, Args)]
pub(crate) struct PickCommandArgs {
    #[command(flatten)]
    pub wait: WaitArgs,
}
