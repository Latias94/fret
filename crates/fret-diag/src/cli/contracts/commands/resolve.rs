use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct ResolveCommandArgs {
    #[command(subcommand)]
    pub command: ResolveSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ResolveSubcommandArgs {
    Latest(ResolveLatestArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ResolveLatestArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "within-session", value_name = "ID")]
    pub within_session: Option<String>,

    #[arg(long = "json")]
    pub json: bool,
}
