use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct SessionsCommandArgs {
    #[command(subcommand)]
    pub command: SessionsSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SessionsSubcommandArgs {
    Clean(SessionsCleanArgs),
}

#[derive(Debug, Args)]
pub(crate) struct SessionsCleanArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(
        long = "keep",
        value_name = "N",
        required_unless_present = "older_than_days"
    )]
    pub keep: Option<usize>,

    #[arg(
        long = "older-than-days",
        value_name = "N",
        required_unless_present = "keep"
    )]
    pub older_than_days: Option<u64>,

    #[arg(long = "top", value_name = "N")]
    pub top: Option<usize>,

    #[arg(long = "apply")]
    pub apply: bool,

    #[arg(long = "json")]
    pub json: bool,
}
