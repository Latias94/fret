use clap::{Args, Subcommand};

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

#[derive(Debug, Args)]
pub(crate) struct ArtifactCommandArgs {
    #[command(subcommand)]
    pub command: ArtifactSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ArtifactSubcommandArgs {
    Lint(ArtifactLintArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ArtifactLintArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,
}
