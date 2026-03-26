use std::path::PathBuf;

use clap::Args;

use super::super::shared::{
    ChecksArgs, DevtoolsArgs, LaunchArgs, OutputArgs, PackArgs, SessionArgs, TimingArgs,
};

#[derive(Debug, Args)]
pub(crate) struct RunCommandArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub session: SessionArgs,

    #[command(flatten)]
    pub devtools: DevtoolsArgs,

    #[command(flatten)]
    pub pack: PackArgs,

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

    #[arg(long = "exit-after-run")]
    pub exit_after_run: bool,

    #[arg(long = "reuse-launch")]
    pub reuse_launch: bool,

    #[arg(long = "trace-chrome")]
    pub trace_chrome: bool,

    #[command(flatten)]
    pub launch: LaunchArgs,
}
