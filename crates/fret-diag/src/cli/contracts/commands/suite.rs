use std::path::PathBuf;

use clap::{ArgAction, Args};

use super::super::shared::{
    ChecksArgs, DevtoolsArgs, LaunchArgs, OutputArgs, PackArgs, SessionArgs, TimingArgs,
};

#[derive(Debug, Args)]
pub(crate) struct SuiteCommandArgs {
    #[arg(value_name = "SUITE")]
    pub suite: Option<String>,

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

    #[arg(long = "glob", value_name = "GLOB", action = ArgAction::Append)]
    pub globs: Vec<String>,

    #[arg(long = "script-dir", value_name = "DIR", action = ArgAction::Append)]
    pub script_dirs: Vec<PathBuf>,

    #[arg(long = "prewarm-script", value_name = "PATH", action = ArgAction::Append)]
    pub prewarm_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-script", value_name = "PATH", action = ArgAction::Append)]
    pub prelude_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-each-run")]
    pub prelude_each_run: bool,

    #[arg(long = "max-test-ids", default_value_t = 200)]
    pub max_test_ids: usize,

    #[arg(long = "all-test-ids")]
    pub lint_all_test_ids_bounds: bool,

    #[arg(long = "lint-eps-px", default_value_t = 0.5)]
    pub lint_eps_px: f32,

    #[arg(long = "no-suite-lint")]
    pub no_suite_lint: bool,

    #[arg(long = "reuse-launch")]
    pub reuse_launch: bool,

    #[command(flatten)]
    pub launch: LaunchArgs,
}
