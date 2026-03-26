use std::path::PathBuf;

use clap::Args;

use super::super::shared::{ChecksArgs, LaunchArgs, OutputArgs, TimingArgs};

#[derive(Debug, Args)]
pub(crate) struct ReproCommandArgs {
    #[arg(value_name = "TARGET", num_args = 1.., required = true)]
    pub targets: Vec<String>,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub pack: ReproPackArgs,

    #[command(flatten)]
    pub checks: ChecksArgs,

    #[arg(long = "script-path", value_name = "PATH")]
    pub script_path: Option<PathBuf>,

    #[arg(long = "script-trigger-path", value_name = "PATH")]
    pub script_trigger_path: Option<PathBuf>,

    #[arg(long = "script-result-path", value_name = "PATH")]
    pub script_result_path: Option<PathBuf>,

    #[arg(long = "script-result-trigger-path", value_name = "PATH")]
    pub script_result_trigger_path: Option<PathBuf>,

    #[arg(long = "trace-chrome")]
    pub trace_chrome: bool,

    #[command(flatten)]
    pub launch: LaunchArgs,
}

#[derive(Debug, Args)]
pub(crate) struct ReproPackArgs {
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
