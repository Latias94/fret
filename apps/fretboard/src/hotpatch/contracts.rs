use clap::{Args, Subcommand};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(
    arg_required_else_help = true,
    after_help = "Notes:\n  - Requires running the app with `--hotpatch` (sets `FRET_HOTPATCH=1`).\n  - The runner watches `FRET_HOTPATCH_TRIGGER_PATH` (default: `.fret/hotpatch.touch`).\n  - `watch` is polling-based and ignores `target/`, `.git/`, `.fret/`, and `repo-ref/`."
)]
pub(crate) struct HotpatchCommandArgs {
    #[command(subcommand)]
    pub action: HotpatchActionContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum HotpatchActionContract {
    /// Update the hotpatch trigger file.
    Poke(HotpatchPathCommandArgs),
    /// Print the hotpatch trigger file path.
    Path(HotpatchPathCommandArgs),
    /// Show hotpatch-related log tails.
    Status(HotpatchStatusCommandArgs),
    /// Poll workspace sources and poke the trigger file on change.
    Watch(HotpatchWatchCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct HotpatchPathCommandArgs {
    /// Override the hotpatch trigger file path.
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct HotpatchStatusCommandArgs {
    /// Number of log lines to print per log.
    #[arg(long, default_value_t = 40)]
    pub tail: usize,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct HotpatchWatchCommandArgs {
    /// Additional workspace-relative roots to watch.
    #[arg(long = "path")]
    pub paths: Vec<String>,
    /// Override the hotpatch trigger file path.
    #[arg(long = "trigger-path")]
    pub trigger_path: Option<String>,
    /// Poll interval in milliseconds.
    #[arg(long = "poll-ms", default_value_t = 500)]
    pub poll_ms: u64,
    /// Minimum delay between trigger pokes in milliseconds.
    #[arg(long = "debounce-ms", default_value_t = 200)]
    pub debounce_ms: u64,
}
