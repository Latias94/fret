use std::path::PathBuf;

use clap::{ArgAction, Args};

use super::super::shared::OutputArgs;

#[derive(Debug, Args)]
pub(crate) struct SummarizeCommandArgs {
    #[arg(value_name = "INPUT", action = ArgAction::Append)]
    pub inputs: Vec<PathBuf>,

    #[command(flatten)]
    pub output: OutputArgs,
}
