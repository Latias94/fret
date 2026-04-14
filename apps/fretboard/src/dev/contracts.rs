use clap::{ArgAction, Args};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct DevNativeCommandArgs {
    /// Run a specific native binary.
    #[arg(long)]
    pub bin: Option<String>,
    /// Run a specific demo id in the native demo shell.
    #[arg(long)]
    pub demo: Option<String>,
    /// Run a cookbook example.
    #[arg(long)]
    pub example: Option<String>,
    /// Cargo profile to use.
    #[arg(long = "profile", visible_alias = "cargo-profile")]
    pub profile: Option<String>,
    /// Force strict runtime diagnostics on for the launched app.
    #[arg(
        long = "strict-runtime",
        action = ArgAction::SetTrue,
        overrides_with = "no_strict_runtime"
    )]
    pub strict_runtime: bool,
    /// Disable the default strict runtime diagnostics for this dev launch.
    #[arg(
        long = "no-strict-runtime",
        action = ArgAction::SetTrue,
        overrides_with = "strict_runtime"
    )]
    pub no_strict_runtime: bool,
    /// Prompt to choose a demo interactively.
    #[arg(long)]
    pub choose: bool,
    /// Include maintainer-only demos in the chooser.
    #[arg(long)]
    pub all: bool,
    /// Enable hotpatch mode.
    #[arg(long)]
    pub hotpatch: bool,
    /// Force reload-boundary hotpatch mode.
    #[arg(long = "hotpatch-reload")]
    pub hotpatch_reload: bool,
    /// Override the hotpatch trigger file path.
    #[arg(long = "hotpatch-trigger-path")]
    pub hotpatch_trigger_path: Option<String>,
    /// Override the hotpatch polling interval in milliseconds.
    #[arg(long = "hotpatch-poll-ms")]
    pub hotpatch_poll_ms: Option<u64>,
    /// Connect to an external hotpatch devserver.
    #[arg(long = "hotpatch-devserver")]
    pub hotpatch_devserver: Option<String>,
    /// Run through `dx serve --hotpatch`.
    #[arg(long = "hotpatch-dx")]
    pub hotpatch_dx: bool,
    /// Override the dx hotpatch websocket endpoint.
    #[arg(long = "hotpatch-dx-ws")]
    pub hotpatch_dx_ws: Option<String>,
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
    /// Reset dev state before launch.
    #[arg(long = "dev-state-reset")]
    pub dev_state_reset: bool,
    /// Override watch polling interval in milliseconds.
    #[arg(long = "watch-poll-ms")]
    pub watch_poll_ms: Option<u64>,
    /// Override the hotpatch build id.
    #[arg(long = "hotpatch-build-id")]
    pub hotpatch_build_id: Option<String>,
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
    /// Override the Trunk dev server port.
    #[arg(long)]
    pub port: Option<u16>,
    /// Force strict runtime diagnostics on for the launched app.
    #[arg(
        long = "strict-runtime",
        action = ArgAction::SetTrue,
        overrides_with = "no_strict_runtime"
    )]
    pub strict_runtime: bool,
    /// Disable the default strict runtime diagnostics for this dev launch.
    #[arg(
        long = "no-strict-runtime",
        action = ArgAction::SetTrue,
        overrides_with = "strict_runtime"
    )]
    pub no_strict_runtime: bool,
    /// Select a web demo id.
    #[arg(long)]
    pub demo: Option<String>,
    /// Prompt to choose a web demo interactively.
    #[arg(long)]
    pub choose: bool,
    /// Append a devtools websocket override to the launched URL.
    #[arg(long = "devtools-ws-url")]
    pub devtools_ws_url: Option<String>,
    /// Append a devtools token override to the launched URL.
    #[arg(long = "devtools-token")]
    pub devtools_token: Option<String>,
}
