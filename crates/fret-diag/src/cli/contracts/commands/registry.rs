use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct RegistryCommandArgs {
    #[command(subcommand)]
    pub command: RegistrySubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RegistrySubcommandArgs {
    Check(RegistryActionArgs),
    Write(RegistryActionArgs),
    Print(RegistryActionArgs),
}

#[derive(Debug, Args, Default)]
pub(crate) struct RegistryActionArgs {
    #[arg(long = "path", value_name = "PATH")]
    pub path: Option<PathBuf>,

    #[arg(long = "json")]
    pub json: bool,
}
