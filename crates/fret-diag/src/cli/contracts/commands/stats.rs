use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, Args};

use super::super::shared::{ChecksArgs, WarmupFramesArgs};

fn parse_bundle_stats_sort(raw: &str) -> Result<crate::BundleStatsSort, String> {
    crate::BundleStatsSort::parse(raw)
}

#[derive(Debug, Args)]
pub(crate) struct StatsChecksArgs {
    #[command(flatten)]
    pub common: ChecksArgs,

    #[arg(long = "check-asset-load-missing-bundle-assets-max", value_name = "N")]
    pub check_asset_load_missing_bundle_assets_max: Option<u64>,

    #[arg(long = "check-asset-load-stale-manifest-max", value_name = "N")]
    pub check_asset_load_stale_manifest_max: Option<u64>,

    #[arg(long = "check-asset-load-unsupported-file-max", value_name = "N")]
    pub check_asset_load_unsupported_file_max: Option<u64>,

    #[arg(long = "check-asset-load-unsupported-url-max", value_name = "N")]
    pub check_asset_load_unsupported_url_max: Option<u64>,

    #[arg(
        long = "check-asset-load-external-reference-unavailable-max",
        value_name = "N"
    )]
    pub check_asset_load_external_reference_unavailable_max: Option<u64>,

    #[arg(long = "check-asset-load-revision-changes-max", value_name = "N")]
    pub check_asset_load_revision_changes_max: Option<u64>,

    #[arg(long = "check-bundled-font-baseline-source", value_name = "SOURCE")]
    pub check_bundled_font_baseline_source: Option<String>,

    #[arg(long = "check-asset-reload-epoch-min", value_name = "N")]
    pub check_asset_reload_epoch_min: Option<u64>,

    #[arg(long = "check-asset-reload-configured-backend", value_name = "NAME")]
    pub check_asset_reload_configured_backend: Option<String>,

    #[arg(long = "check-asset-reload-active-backend", value_name = "NAME")]
    pub check_asset_reload_active_backend: Option<String>,

    #[arg(long = "check-asset-reload-fallback-reason", value_name = "REASON")]
    pub check_asset_reload_fallback_reason: Option<String>,

    #[arg(long = "check-semantics-changed-repainted")]
    pub check_semantics_changed_repainted: bool,

    #[arg(long = "dump-semantics-changed-repainted-json")]
    pub dump_semantics_changed_repainted_json: bool,

    #[arg(long = "check-wheel-scroll", value_name = "TEST_ID")]
    pub check_wheel_scroll: Option<String>,

    #[arg(long = "check-wheel-scroll-hit-changes", value_name = "TEST_ID")]
    pub check_wheel_scroll_hit_changes: Option<String>,

    #[arg(long = "check-drag-cache-root-paint-only", value_name = "TEST_ID")]
    pub check_drag_cache_root_paint_only: Option<String>,

    #[arg(long = "check-hover-layout", conflicts_with = "check_hover_layout_max")]
    pub check_hover_layout: bool,

    #[arg(long = "check-hover-layout-max", value_name = "N")]
    pub check_hover_layout_max: Option<u32>,

    #[arg(long = "check-gc-sweep-liveness")]
    pub check_gc_sweep_liveness: bool,

    #[arg(
        long = "check-notify-hotspot-file-max",
        value_names = ["FILE", "MAX"],
        num_args = 2,
        action = ArgAction::Append
    )]
    pub check_notify_hotspot_file_max: Vec<String>,

    #[arg(long = "check-view-cache-reuse-stable-min", value_name = "N")]
    pub check_view_cache_reuse_stable_min: Option<u64>,

    #[arg(long = "check-view-cache-reuse-min", value_name = "N")]
    pub check_view_cache_reuse_min: Option<u64>,

    #[arg(long = "check-overlay-synthesis-min", value_name = "N")]
    pub check_overlay_synthesis_min: Option<u64>,

    #[arg(long = "check-viewport-input-min", value_name = "N")]
    pub check_viewport_input_min: Option<u64>,

    #[arg(long = "check-dock-drag-min", value_name = "N")]
    pub check_dock_drag_min: Option<u64>,

    #[arg(long = "check-viewport-capture-min", value_name = "N")]
    pub check_viewport_capture_min: Option<u64>,

    #[arg(
        long = "check-retained-vlist-reconcile-no-notify-min",
        value_name = "N"
    )]
    pub check_retained_vlist_reconcile_no_notify_min: Option<u64>,

    #[arg(long = "check-retained-vlist-attach-detach-max", value_name = "N")]
    pub check_retained_vlist_attach_detach_max: Option<u64>,

    #[arg(long = "check-retained-vlist-keep-alive-reuse-min", value_name = "N")]
    pub check_retained_vlist_keep_alive_reuse_min: Option<u64>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("stats_target")
        .args(["source", "diff", "stats_lite_checks_json"])
        .required(true)
        .multiple(false)
))]
pub(crate) struct StatsCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[arg(long = "diff", value_names = ["SOURCE_A", "SOURCE_B"], num_args = 2)]
    pub diff: Option<Vec<PathBuf>>,

    #[command(flatten)]
    pub warmup: WarmupFramesArgs,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(
        long = "sort",
        value_name = "SORT",
        value_parser = parse_bundle_stats_sort
    )]
    pub sort: Option<crate::BundleStatsSort>,

    #[arg(long = "verbose")]
    pub verbose: bool,

    #[arg(long = "json")]
    pub json: bool,

    #[arg(long = "stats-lite-checks-json")]
    pub stats_lite_checks_json: bool,

    #[command(flatten)]
    pub checks: StatsChecksArgs,
}
