use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct ThemeCommandArgs {
    #[command(subcommand)]
    pub target: ThemeTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum ThemeTargetContract {
    /// Convert a VS Code theme JSON into a Fret syntax theme config.
    ImportVscode(ThemeImportVscodeCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct ThemeImportVscodeCommandArgs {
    /// VS Code theme JSON input file.
    pub input: PathBuf,
    /// Output Fret theme config path. When omitted, prints JSON to stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Base Fret theme config to patch into.
    #[arg(long)]
    pub base: Option<PathBuf>,
    /// Override the generated output theme name.
    #[arg(long)]
    pub name: Option<String>,
    /// Optional JSON report output path.
    #[arg(long)]
    pub report: Option<PathBuf>,
    /// Optional mapping JSON path.
    #[arg(long)]
    pub map: Option<PathBuf>,
    /// Additional `key=value` overrides to merge into the import mapping.
    #[arg(long = "set")]
    pub sets: Vec<String>,
    /// Generate all Fret syntax tags from the VS Code theme.
    #[arg(long = "all-tags")]
    pub all_tags: bool,
    /// Overwrite any output files that already exist.
    #[arg(long)]
    pub force: bool,
}
