use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct ConfigCommandArgs {
    #[command(subcommand)]
    pub target: ConfigTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum ConfigTargetContract {
    /// Generate a starter `.fret/menubar.json`.
    Menubar(ConfigMenubarCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ConfigMenubarCommandArgs {
    /// Project root that should receive `.fret/menubar.json`.
    #[arg(long)]
    pub path: Option<PathBuf>,
    /// Overwrite an existing config file.
    #[arg(long)]
    pub force: bool,
}
