use std::path::PathBuf;

use clap::{ArgAction, Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
pub(crate) struct ConfigCommandArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigSubcommandArgs {
    Doctor(ConfigDoctorArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ConfigDoctorModeArg {
    Launch,
    Manual,
}

impl ConfigDoctorModeArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ConfigShowEnvArg {
    Set,
    All,
}

impl ConfigShowEnvArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct ConfigDoctorArgs {
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "env", value_name = "KEY=VALUE", action = ArgAction::Append)]
    pub env: Vec<String>,

    #[arg(long = "mode", value_name = "MODE")]
    pub mode: Option<ConfigDoctorModeArg>,

    #[arg(long = "config-path", value_name = "PATH")]
    pub config_path: Option<PathBuf>,

    #[arg(long = "show-env", value_name = "SHOW_ENV")]
    pub show_env: Option<ConfigShowEnvArg>,

    #[arg(long = "report-json")]
    pub report_json: bool,

    #[arg(long = "print-launch-policy")]
    pub print_launch_policy: bool,
}
