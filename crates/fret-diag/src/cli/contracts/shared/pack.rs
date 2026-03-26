use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct PackArgs {
    #[arg(long = "pack")]
    pub pack: bool,

    #[arg(long = "pack-out", value_name = "PATH")]
    pub pack_out: Option<PathBuf>,

    #[arg(long = "ai-packet")]
    pub ai_packet: bool,

    #[arg(long = "ai-only")]
    pub ai_only: bool,

    #[arg(long = "include-all")]
    pub include_all: bool,

    #[arg(long = "include-root-artifacts")]
    pub include_root_artifacts: bool,

    #[arg(long = "include-triage")]
    pub include_triage: bool,

    #[arg(long = "include-screenshots")]
    pub include_screenshots: bool,

    #[arg(long = "pack-schema2-only")]
    pub pack_schema2_only: bool,
}
