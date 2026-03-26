use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

use super::super::shared::{OutputArgs, SessionArgs, TimingArgs};

#[derive(Debug, Args)]
#[command(
    after_help = "Direct execution:\n  fretboard diag script <script.json> [--dir <dir>] [--script-path <path>] [--script-trigger-path <path>]"
)]
pub(crate) struct ScriptCommandArgs {
    #[command(subcommand)]
    pub command: ScriptSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ScriptSubcommandArgs {
    Normalize(ScriptNormalizeArgs),
    Upgrade(ScriptUpgradeArgs),
    Validate(ScriptValidateArgs),
    Lint(ScriptLintArgs),
    Shrink(ScriptShrinkArgs),
    #[command(external_subcommand)]
    Direct(Vec<String>),
}

#[derive(Debug, Parser)]
pub(crate) struct ScriptDirectArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "script-path", value_name = "PATH")]
    pub script_path: Option<PathBuf>,

    #[arg(long = "script-trigger-path", value_name = "PATH")]
    pub script_trigger_path: Option<PathBuf>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn try_parse_direct_script_args<I, T>(args: I) -> Result<ScriptDirectArgs, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    ScriptDirectArgs::try_parse_from(args)
}

#[derive(Debug, Args)]
pub(crate) struct ScriptNormalizeArgs {
    #[arg(value_name = "INPUT", num_args = 1.., required = true)]
    pub inputs: Vec<String>,

    #[arg(long = "write", conflicts_with = "check")]
    pub write: bool,

    #[arg(long = "check", conflicts_with = "write")]
    pub check: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ScriptUpgradeArgs {
    #[arg(value_name = "INPUT", num_args = 1.., required = true)]
    pub inputs: Vec<String>,

    #[arg(long = "write", conflicts_with = "check")]
    pub write: bool,

    #[arg(long = "check", conflicts_with = "write")]
    pub check: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ScriptValidateArgs {
    #[arg(value_name = "INPUT", num_args = 1.., required = true)]
    pub inputs: Vec<String>,

    #[command(flatten)]
    pub output: OutputArgs,

    #[arg(long = "check-out", value_name = "PATH")]
    pub check_out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct ScriptLintArgs {
    #[arg(value_name = "INPUT", num_args = 1.., required = true)]
    pub inputs: Vec<String>,

    #[command(flatten)]
    pub output: OutputArgs,

    #[arg(long = "check-out", value_name = "PATH")]
    pub check_out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct ScriptRuntimePathArgs {
    #[arg(long = "script-path", value_name = "PATH")]
    pub script_path: Option<PathBuf>,

    #[arg(long = "script-trigger-path", value_name = "PATH")]
    pub script_trigger_path: Option<PathBuf>,

    #[arg(long = "script-result-path", value_name = "PATH")]
    pub script_result_path: Option<PathBuf>,

    #[arg(long = "script-result-trigger-path", value_name = "PATH")]
    pub script_result_trigger_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct ScriptShrinkLaunchArgs {
    #[arg(long = "env", value_name = "KEY=VALUE", action = ArgAction::Append)]
    pub env: Vec<String>,

    #[arg(
        long = "launch",
        value_name = "CMD",
        num_args = 1..,
        allow_hyphen_values = true
    )]
    pub launch: Option<Vec<String>>,
}

impl ScriptShrinkLaunchArgs {
    pub(crate) fn normalized_launch_argv(&self) -> Option<Vec<String>> {
        self.launch.as_ref().map(|argv| {
            let mut values = argv.clone();
            if values.first().is_some_and(|value| value == "--") {
                values.remove(0);
            }
            values
        })
    }
}

#[derive(Debug, Args)]
pub(crate) struct ScriptShrinkArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: String,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub session: SessionArgs,

    #[command(flatten)]
    pub paths: ScriptRuntimePathArgs,

    #[arg(long = "shrink-out", value_name = "PATH")]
    pub shrink_out: Option<PathBuf>,

    #[arg(long = "shrink-any-fail")]
    pub shrink_any_fail: bool,

    #[arg(long = "shrink-match-reason-code", value_name = "CODE")]
    pub shrink_match_reason_code: Option<String>,

    #[arg(long = "shrink-match-reason", value_name = "TEXT")]
    pub shrink_match_reason: Option<String>,

    #[arg(long = "shrink-min-steps", default_value_t = 1)]
    pub shrink_min_steps: u64,

    #[arg(long = "shrink-max-iters", default_value_t = 200)]
    pub shrink_max_iters: u64,

    #[command(flatten)]
    pub launch: ScriptShrinkLaunchArgs,
}
