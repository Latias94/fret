use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct NewCommandArgs {
    #[command(subcommand)]
    pub template: Option<NewTemplateContract>,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum NewTemplateContract {
    /// Create a minimal Cargo-like starter app.
    Empty(ScaffoldEmptyCommandArgs),
    /// Create the smallest runnable UI app.
    Hello(ScaffoldHelloCommandArgs),
    /// Create the recommended starter app without selector/query extras.
    SimpleTodo(ScaffoldTodoCommandArgs),
    /// Create the golden-path todo starter with selector/query follow-up wiring.
    Todo(ScaffoldTodoCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ScaffoldOutputArgs {
    /// Output directory for the generated app.
    #[arg(long)]
    pub path: Option<PathBuf>,
    /// Package name for the generated app.
    #[arg(long)]
    pub name: Option<String>,
    /// Skip the final `cargo check` smoke.
    #[arg(long = "no-check")]
    pub no_check: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ScaffoldIconArgs {
    /// Icon pack to wire into the generated app.
    #[arg(long, value_enum, conflicts_with = "no_icons")]
    pub icons: Option<ScaffoldIconPackValue>,
    /// Disable icons in the generated app.
    #[arg(long, conflicts_with = "icons")]
    pub no_icons: bool,
    /// Enable the starter command palette wiring.
    #[arg(long = "command-palette")]
    pub command_palette: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ScaffoldEmptyCommandArgs {
    #[command(flatten)]
    pub output: ScaffoldOutputArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ScaffoldHelloCommandArgs {
    #[command(flatten)]
    pub output: ScaffoldOutputArgs,
    #[command(flatten)]
    pub icons: ScaffoldIconArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ScaffoldTodoCommandArgs {
    #[command(flatten)]
    pub output: ScaffoldOutputArgs,
    #[command(flatten)]
    pub icons: ScaffoldIconArgs,
    /// Generate the default UI assets cache/bundle stub.
    #[arg(long = "ui-assets")]
    pub ui_assets: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ScaffoldIconPackValue {
    Lucide,
    Radix,
    None,
}
