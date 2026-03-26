use clap::{ArgAction, Args};

use super::super::shared::ReportOutputArgs;

#[derive(Debug, Args)]
pub(crate) struct MemorySummaryCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "within-session", value_name = "ID|latest|all")]
    pub within_session: Option<String>,

    #[arg(long = "top-sessions", value_name = "N")]
    pub top_sessions: Option<usize>,

    #[arg(
        long = "sort-key",
        value_name = "KEY",
        default_value = "macos_physical_footprint_peak_bytes"
    )]
    pub sort_key: String,

    #[arg(long = "fit-linear", value_name = "Y_KEY:X_KEY", action = ArgAction::Append)]
    pub fit_linear: Vec<String>,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(long = "vmmap-regions-sorted-top")]
    pub vmmap_regions_sorted_top: bool,

    #[arg(long = "vmmap-regions-sorted-agg")]
    pub vmmap_regions_sorted_agg: bool,

    #[arg(
        long = "vmmap-regions-sorted-agg-top",
        value_name = "N",
        default_value_t = 10
    )]
    pub vmmap_regions_sorted_agg_top: usize,

    #[arg(long = "vmmap-regions-sorted-detail-agg")]
    pub vmmap_regions_sorted_detail_agg: bool,

    #[arg(
        long = "vmmap-regions-sorted-detail-agg-top",
        value_name = "N",
        default_value_t = 12
    )]
    pub vmmap_regions_sorted_detail_agg_top: usize,

    #[arg(long = "footprint-categories-agg")]
    pub footprint_categories_agg: bool,

    #[arg(
        long = "footprint-categories-agg-top",
        value_name = "N",
        default_value_t = 12
    )]
    pub footprint_categories_agg_top: usize,

    #[arg(long = "no-recursive")]
    pub no_recursive: bool,

    #[arg(long = "max-depth", value_name = "N", default_value_t = 3)]
    pub max_depth: usize,

    #[arg(long = "max-samples", value_name = "N", default_value_t = 200)]
    pub max_samples: usize,
}
