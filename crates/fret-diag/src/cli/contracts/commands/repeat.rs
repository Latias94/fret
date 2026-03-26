use clap::{ArgAction, Args};

use super::super::shared::{CompareArgs, LaunchArgs, OutputArgs, TimingArgs};

#[derive(Debug, Args)]
pub(crate) struct RepeatCommandArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub compare: CompareArgs,

    #[arg(long = "repeat", default_value_t = 1)]
    pub repeat: u64,

    #[arg(long = "no-compare")]
    pub no_compare: bool,

    #[arg(
        long = "check-memory-p90-max",
        value_name = "KEY:BYTES",
        action = ArgAction::Append
    )]
    pub check_memory_p90_max: Vec<String>,

    #[command(flatten)]
    pub launch: LaunchArgs,
}
