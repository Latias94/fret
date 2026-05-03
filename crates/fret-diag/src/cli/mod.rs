#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagCliMode {
    RepoMaintainer,
    PublicAppAuthor,
}

impl DiagCliMode {
    pub(crate) const fn root_command_name(self) -> &'static str {
        match self {
            Self::RepoMaintainer => "fretboard-dev diag",
            Self::PublicAppAuthor => "fretboard diag",
        }
    }

    pub(crate) const fn about(self) -> &'static str {
        match self {
            Self::RepoMaintainer => "Diagnostics tooling for the Fret workspace.",
            Self::PublicAppAuthor => {
                "Capture, inspect, compare, and share diagnostics for the current app project."
            }
        }
    }

    pub(crate) const fn root_after_help(self) -> &'static str {
        match self {
            Self::RepoMaintainer => {
                "Examples:\n  fretboard-dev diag poke\n  fretboard-dev diag latest\n  fretboard-dev diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --launch -- cargo run -p fret-ui-gallery --release\n  fretboard-dev diag suite ui-gallery --launch -- cargo run -p fret-ui-gallery --release\n  fretboard-dev diag repro ui-gallery --launch -- cargo run -p fret-ui-gallery --release\n  fretboard-dev diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release\n  fretboard-dev diag campaign list --lane smoke --tag ui-gallery"
            }
            Self::PublicAppAuthor => {
                "Examples:\n  fretboard diag config doctor --mode launch --print-launch-policy\n  fretboard diag run ./diag/dialog-escape.json --launch -- cargo run --manifest-path ./Cargo.toml\n  fretboard diag perf ./diag/dialog-escape.json --repeat 5 --launch -- cargo run --manifest-path ./Cargo.toml\n  fretboard diag latest\n  fretboard diag compare ./target/fret-diag/baseline ./target/fret-diag/candidate --json"
            }
        }
    }

    pub(crate) const fn query_test_id_usage(self) -> &'static str {
        match self {
            Self::RepoMaintainer => "fretboard-dev diag query test-id [SOURCE] PATTERN [OPTIONS]",
            Self::PublicAppAuthor => "fretboard diag query test-id [SOURCE] PATTERN [OPTIONS]",
        }
    }

    pub(crate) const fn query_test_id_after_help(self) -> &'static str {
        match self {
            Self::RepoMaintainer => {
                "Examples:\n  fretboard-dev diag query test-id ui-gallery\n  fretboard-dev diag query test-id target/fret-diag/demo ui-gallery"
            }
            Self::PublicAppAuthor => {
                "Examples:\n  fretboard diag query test-id dialog\n  fretboard diag query test-id ./target/fret-diag/latest dialog"
            }
        }
    }

    pub(crate) const fn script_after_help(self) -> &'static str {
        match self {
            Self::RepoMaintainer => {
                "Direct execution:\n  fretboard-dev diag script <script.json> [--dir <dir>] [--script-path <path>] [--script-trigger-path <path>]"
            }
            Self::PublicAppAuthor => {
                "Direct execution:\n  fretboard diag script <script.json> [--dir <dir>] [--script-path <path>] [--script-trigger-path <path>]"
            }
        }
    }
}

mod contracts;
mod cutover;
mod runtime;
mod workspace;

pub(crate) use cutover::dispatch_diag_command;
pub(crate) use cutover::dispatch_diag_command_with_mode;
pub(crate) use runtime::{
    DiagPathOverrides, ResolveDiagCliPathsRequest, ResolvedDiagCliPaths, resolve_diag_cli_paths,
};
pub(crate) use workspace::workspace_root;
