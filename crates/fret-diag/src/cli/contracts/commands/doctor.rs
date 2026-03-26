use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub(crate) struct DoctorCommandArgs {
    #[command(subcommand)]
    pub command: Option<DoctorSubcommandArgs>,

    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "warmup-frames", default_value_t = 0)]
    pub warmup_frames: u64,

    #[arg(long = "json")]
    pub json: bool,

    #[arg(long = "fix")]
    pub fix: bool,

    #[arg(long = "fix-dry-run", alias = "fix-plan")]
    pub fix_dry_run: bool,

    #[arg(long = "fix-schema2")]
    pub fix_schema2: bool,

    #[arg(long = "fix-bundle-json")]
    pub fix_bundle_json: bool,

    #[arg(long = "fix-sidecars")]
    pub fix_sidecars: bool,

    #[arg(long = "check", alias = "check-required")]
    pub check_required: bool,

    #[arg(long = "check-all", alias = "strict")]
    pub check_all: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum DoctorSubcommandArgs {
    Scripts(DoctorScriptsArgs),
    Campaigns(DoctorCampaignsArgs),
}

#[derive(Debug, Args)]
pub(crate) struct DoctorScriptsArgs {
    #[arg(
        long = "max-examples",
        value_name = "N",
        default_value_t = 20,
        alias = "examples",
        alias = "top"
    )]
    pub max_examples: usize,

    #[arg(long = "strict")]
    pub strict: bool,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct DoctorCampaignsArgs {
    #[arg(long = "strict")]
    pub strict: bool,

    #[arg(long = "json")]
    pub json: bool,
}
