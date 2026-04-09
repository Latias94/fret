use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::super::shared::{ReportOutputArgs, WarmupFramesArgs};

fn parse_query_mode(raw: &str) -> Result<String, String> {
    match raw {
        "contains" | "prefix" | "glob" => Ok(raw.to_string()),
        _ => Err("invalid value for --mode (expected contains|prefix|glob)".to_string()),
    }
}

fn parse_overlay_kind(raw: &str) -> Result<String, String> {
    match raw {
        "anchored_panel" | "placed_rect" => Ok(raw.to_string()),
        _ => Err("invalid value for --kind (expected anchored_panel|placed_rect)".to_string()),
    }
}

fn parse_overlay_side(raw: &str) -> Result<String, String> {
    match raw {
        "top" | "bottom" | "left" | "right" => Ok(raw.to_string()),
        _ => Err("invalid value (expected top|bottom|left|right)".to_string()),
    }
}

fn parse_overlay_align(raw: &str) -> Result<String, String> {
    match raw {
        "start" | "center" | "end" => Ok(raw.to_string()),
        _ => Err("invalid value for --align (expected start|center|end)".to_string()),
    }
}

fn parse_overlay_sticky(raw: &str) -> Result<String, String> {
    match raw {
        "partial" | "always" => Ok(raw.to_string()),
        _ => Err("invalid value for --sticky (expected partial|always)".to_string()),
    }
}

fn parse_overlay_flipped(raw: &str) -> Result<bool, String> {
    match raw {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err("invalid value for --flipped (expected true|false)".to_string()),
    }
}

fn looks_like_query_source(raw: &str) -> bool {
    raw.contains('/') || raw.contains('\\') || raw.ends_with(".json")
}

#[derive(Debug, Args)]
pub(crate) struct QueryCommandArgs {
    #[command(subcommand)]
    pub command: QuerySubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum QuerySubcommandArgs {
    TestId(QueryTestIdArgs),
    Snapshots(QuerySnapshotsArgs),
    OverlayPlacementTrace(QueryOverlayPlacementTraceArgs),
    ScrollExtentsObservation(QueryScrollExtentsObservationArgs),
}

#[derive(Debug, Args)]
#[command(
    override_usage = "fretboard-dev diag query test-id [SOURCE] PATTERN [OPTIONS]",
    after_help = "Examples:\n  fretboard-dev diag query test-id ui-gallery\n  fretboard-dev diag query test-id target/fret-diag/demo ui-gallery"
)]
pub(crate) struct QueryTestIdArgs {
    #[arg(value_name = "ARG", num_args = 1..=2, required = true)]
    pub inputs: Vec<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(
        long = "mode",
        value_name = "MODE",
        default_value = "contains",
        value_parser = parse_query_mode
    )]
    pub mode: String,

    #[arg(long = "top", default_value_t = 50)]
    pub top: usize,

    #[arg(long = "case-sensitive")]
    pub case_sensitive: bool,
}

#[derive(Debug, Args)]
pub(crate) struct QuerySnapshotsArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "top", default_value_t = 20)]
    pub top: usize,

    #[arg(long = "window")]
    pub window: Option<u64>,

    #[arg(long = "include-warmup")]
    pub include_warmup: bool,

    #[arg(long = "include-missing-semantics")]
    pub include_missing_semantics: bool,

    #[arg(
        long = "semantics-source",
        value_name = "SOURCE",
        value_parser = ["any", "inline", "table", "none"]
    )]
    pub semantics_source: Option<String>,

    #[arg(long = "test-id", value_name = "TEST_ID")]
    pub test_id: Option<String>,

    #[arg(long = "step-index")]
    pub step_index: Option<u32>,
}

#[derive(Debug, Args)]
pub(crate) struct QueryOverlayPlacementTraceArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "top", default_value_t = 50)]
    pub top: usize,

    #[arg(
        long = "kind",
        value_name = "KIND",
        value_parser = parse_overlay_kind
    )]
    pub kind: Option<String>,

    #[arg(long = "overlay-root-name", value_name = "NAME")]
    pub overlay_root_name: Option<String>,

    #[arg(long = "anchor-test-id", value_name = "TEST_ID")]
    pub anchor_test_id: Option<String>,

    #[arg(long = "content-test-id", value_name = "TEST_ID")]
    pub content_test_id: Option<String>,

    #[arg(
        long = "preferred-side",
        value_name = "SIDE",
        value_parser = parse_overlay_side
    )]
    pub preferred_side: Option<String>,

    #[arg(
        long = "chosen-side",
        value_name = "SIDE",
        value_parser = parse_overlay_side
    )]
    pub chosen_side: Option<String>,

    #[arg(
        long = "flipped",
        value_name = "BOOL",
        value_parser = parse_overlay_flipped
    )]
    pub flipped: Option<bool>,

    #[arg(
        long = "align",
        value_name = "ALIGN",
        value_parser = parse_overlay_align
    )]
    pub align: Option<String>,

    #[arg(
        long = "sticky",
        value_name = "MODE",
        value_parser = parse_overlay_sticky
    )]
    pub sticky: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct QueryScrollExtentsObservationArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "top", default_value_t = 200)]
    pub top: usize,

    #[arg(long = "window")]
    pub window: Option<u64>,

    #[arg(long = "all")]
    pub all: bool,

    #[arg(long = "deep-scan")]
    pub deep_scan: bool,

    #[arg(long = "timeline")]
    pub timeline: bool,
}

#[derive(Debug)]
pub(crate) struct QueryResolution {
    pub source: Option<String>,
    pub pattern: String,
}

pub(crate) fn resolve_query_test_id_inputs(
    inputs: &[String],
    workspace_root: &std::path::Path,
) -> Result<QueryResolution, String> {
    match inputs {
        [pattern] => {
            let maybe_path = crate::resolve_path(workspace_root, PathBuf::from(pattern));
            if looks_like_query_source(pattern) && (maybe_path.is_file() || maybe_path.is_dir()) {
                return Err(
                    "missing pattern (try: fretboard-dev diag query test-id <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> <pattern>)"
                        .to_string(),
                );
            }
            Ok(QueryResolution {
                source: None,
                pattern: pattern.clone(),
            })
        }
        [source, pattern] => Ok(QueryResolution {
            source: Some(source.clone()),
            pattern: pattern.clone(),
        }),
        _ => Err("missing pattern (try: fretboard-dev diag query test-id <pattern>)".to_string()),
    }
}
