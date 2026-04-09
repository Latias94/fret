use std::path::PathBuf;

use clap::{ArgAction, Args, Subcommand};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct DevCommandArgs {
    #[command(subcommand)]
    pub target: DevTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum DevTargetContract {
    /// Run a native Cargo binary or example.
    Native(DevNativeCommandArgs),
    /// Run a web target via Trunk from the selected package root.
    Web(DevWebCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct DevNativeCommandArgs {
    /// Path to the Cargo.toml that defines the current app project or workspace.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    /// Select a workspace package explicitly.
    #[arg(long)]
    pub package: Option<String>,
    /// Run a specific binary target.
    #[arg(long, conflicts_with = "example")]
    pub bin: Option<String>,
    /// Run a specific example target.
    #[arg(long, conflicts_with = "bin")]
    pub example: Option<String>,
    /// Cargo profile to use.
    #[arg(long = "profile", visible_alias = "cargo-profile")]
    pub profile: Option<String>,
    /// Force watch mode on.
    #[arg(
        long = "watch",
        action = ArgAction::SetTrue,
        overrides_with = "no_watch"
    )]
    pub watch: bool,
    /// Force watch mode off.
    #[arg(
        long = "no-watch",
        action = ArgAction::SetTrue,
        overrides_with = "watch"
    )]
    pub no_watch: bool,
    /// Force the restart supervisor on.
    #[arg(
        long = "supervise",
        action = ArgAction::SetTrue,
        overrides_with = "no_supervise"
    )]
    pub supervise: bool,
    /// Force the restart supervisor off.
    #[arg(
        long = "no-supervise",
        action = ArgAction::SetTrue,
        overrides_with = "supervise"
    )]
    pub no_supervise: bool,
    /// Reset dev state before launch.
    #[arg(long = "dev-state-reset")]
    pub dev_state_reset: bool,
    /// Override watch polling interval in milliseconds.
    #[arg(long = "watch-poll-ms")]
    pub watch_poll_ms: Option<u64>,
    /// Remaining arguments passed through after `--`.
    #[arg(
        value_name = "ARG",
        num_args = 0..,
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub passthrough: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct DevWebCommandArgs {
    /// Path to the Cargo.toml that defines the current app project or workspace.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    /// Select a workspace package explicitly.
    #[arg(long)]
    pub package: Option<String>,
    /// Select a specific web binary target when the package exposes more than one.
    #[arg(long)]
    pub bin: Option<String>,
    /// Override the Trunk dev server port.
    #[arg(long)]
    pub port: Option<u16>,
    /// Auto-open the browser when the dev server becomes reachable.
    #[arg(
        long = "open",
        action = ArgAction::SetTrue,
        overrides_with = "no_open"
    )]
    pub open: bool,
    /// Disable browser auto-open.
    #[arg(
        long = "no-open",
        action = ArgAction::SetTrue,
        overrides_with = "open"
    )]
    pub no_open: bool,
    /// Append a devtools websocket override to the launched URL.
    #[arg(long = "devtools-ws-url")]
    pub devtools_ws_url: Option<String>,
    /// Append a devtools token override to the launched URL.
    #[arg(long = "devtools-token")]
    pub devtools_token: Option<String>,
}
