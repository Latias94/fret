use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use zip::write::FileOptions;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum BundleStatsSort {
    #[default]
    Invalidation,
    Time,
}

impl BundleStatsSort {
    fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "invalidation" => Ok(Self::Invalidation),
            "time" => Ok(Self::Time),
            other => Err(format!(
                "invalid --sort value: {other} (expected: invalidation|time)"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Invalidation => "invalidation",
            Self::Time => "time",
        }
    }
}

#[derive(Debug, Clone)]
struct ReproPackItem {
    script_path: PathBuf,
    bundle_json: PathBuf,
}

#[derive(Debug)]
struct LaunchedDemo {
    child: Child,
    launched_unix_ms: u64,
    launched_instant: Instant,
    launch_cmd: Vec<String>,
}

pub(crate) fn diag_cmd(args: Vec<String>) -> Result<(), String> {
    let mut out_dir: Option<PathBuf> = None;
    let mut trigger_path: Option<PathBuf> = None;
    let mut pack_out: Option<PathBuf> = None;
    let mut pack_include_root_artifacts: bool = false;
    let mut pack_include_triage: bool = false;
    let mut pack_include_screenshots: bool = false;
    let mut pack_after_run: bool = false;
    let mut triage_out: Option<PathBuf> = None;
    let mut script_path: Option<PathBuf> = None;
    let mut script_trigger_path: Option<PathBuf> = None;
    let mut script_result_path: Option<PathBuf> = None;
    let mut script_result_trigger_path: Option<PathBuf> = None;
    let mut pick_trigger_path: Option<PathBuf> = None;
    let mut pick_result_path: Option<PathBuf> = None;
    let mut pick_result_trigger_path: Option<PathBuf> = None;
    let mut pick_script_out: Option<PathBuf> = None;
    let mut pick_apply_pointer: Option<String> = None;
    let mut pick_apply_out: Option<PathBuf> = None;
    let mut inspect_path: Option<PathBuf> = None;
    let mut inspect_trigger_path: Option<PathBuf> = None;
    let mut inspect_consume_clicks: Option<bool> = None;
    let mut timeout_ms: u64 = 30_000;
    let mut poll_ms: u64 = 50;
    let mut stats_top: usize = 5;
    let mut sort_override: Option<BundleStatsSort> = None;
    let mut stats_json: bool = false;
    let mut warmup_frames: u64 = 0;
    let mut perf_repeat: u64 = 1;
    let mut max_top_total_us: Option<u64> = None;
    let mut max_top_layout_us: Option<u64> = None;
    let mut max_top_solve_us: Option<u64> = None;
    let mut perf_baseline_path: Option<PathBuf> = None;
    let mut perf_baseline_out: Option<PathBuf> = None;
    let mut perf_baseline_headroom_pct: u32 = 20;
    let mut check_idle_no_paint_min: Option<u64> = None;
    let mut check_stale_paint_test_id: Option<String> = None;
    let mut check_stale_paint_eps: f32 = 0.5;
    let mut check_stale_scene_test_id: Option<String> = None;
    let mut check_stale_scene_eps: f32 = 0.5;
    let mut check_pixels_changed_test_id: Option<String> = None;
    let mut check_semantics_changed_repainted: bool = false;
    let mut dump_semantics_changed_repainted_json: bool = false;
    let mut check_wheel_scroll_test_id: Option<String> = None;
    let mut check_drag_cache_root_paint_only_test_id: Option<String> = None;
    let mut check_hover_layout_max: Option<u32> = None;
    let mut check_gc_sweep_liveness: bool = false;
    let mut check_view_cache_reuse_min: Option<u64> = None;
    let mut check_view_cache_reuse_stable_min: Option<u64> = None;
    let mut check_overlay_synthesis_min: Option<u64> = None;
    let mut check_viewport_input_min: Option<u64> = None;
    let mut check_dock_drag_min: Option<u64> = None;
    let mut check_viewport_capture_min: Option<u64> = None;
    let mut check_retained_vlist_reconcile_no_notify_min: Option<u64> = None;
    let mut check_retained_vlist_attach_detach_max: Option<u64> = None;
    let mut compare_eps_px: f32 = 0.5;
    let mut compare_ignore_bounds: bool = false;
    let mut compare_ignore_scene_fingerprint: bool = false;
    let mut launch: Option<Vec<String>> = None;
    let mut launch_env: Vec<(String, String)> = Vec::new();
    let mut with_tracy: bool = false;
    let mut with_renderdoc: bool = false;
    let mut renderdoc_after_frames: Option<u32> = None;
    let mut renderdoc_markers: Vec<String> = Vec::new();
    let mut renderdoc_no_outputs_png: bool = false;

    // Parse global `diag` flags regardless of their position, leaving positional args intact.
    // This keeps the behavior aligned with the help text in `apps/fretboard/src/cli.rs`.
    let mut positionals: Vec<String> = Vec::new();
    let mut i: usize = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--dir" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --dir".to_string());
                };
                out_dir = Some(PathBuf::from(v));
                i += 1;
            }
            "--trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --trigger-path".to_string());
                };
                trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pack-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pack-out".to_string());
                };
                pack_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--include-root-artifacts" => {
                pack_include_root_artifacts = true;
                i += 1;
            }
            "--include-all" => {
                pack_include_root_artifacts = true;
                pack_include_triage = true;
                pack_include_screenshots = true;
                i += 1;
            }
            "--include-triage" => {
                pack_include_triage = true;
                i += 1;
            }
            "--include-screenshots" => {
                pack_include_screenshots = true;
                i += 1;
            }
            "--pack" => {
                pack_after_run = true;
                i += 1;
            }
            "--script-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-path".to_string());
                };
                script_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-trigger-path".to_string());
                };
                script_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-result-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-result-path".to_string());
                };
                script_result_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-result-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-result-trigger-path".to_string());
                };
                script_result_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-trigger-path".to_string());
                };
                pick_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-result-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-result-path".to_string());
                };
                pick_result_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-result-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-result-trigger-path".to_string());
                };
                pick_result_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-script-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-script-out".to_string());
                };
                pick_script_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--ptr" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --ptr".to_string());
                };
                pick_apply_pointer = Some(v);
                i += 1;
            }
            "--out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --out".to_string());
                };
                let p = PathBuf::from(v);
                pick_apply_out = Some(p.clone());
                triage_out = Some(p);
                i += 1;
            }
            "--inspect-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --inspect-path".to_string());
                };
                inspect_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--inspect-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --inspect-trigger-path".to_string());
                };
                inspect_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--consume-clicks" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --consume-clicks".to_string());
                };
                inspect_consume_clicks = Some(
                    parse_bool(&v).map_err(|_| "invalid value for --consume-clicks".to_string())?,
                );
                i += 1;
            }
            "--timeout-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --timeout-ms".to_string());
                };
                timeout_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --timeout-ms".to_string())?;
                i += 1;
            }
            "--poll-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --poll-ms".to_string());
                };
                poll_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --poll-ms".to_string())?;
                i += 1;
            }
            "--sort" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --sort".to_string());
                };
                sort_override = Some(BundleStatsSort::parse(&v)?);
                i += 1;
            }
            "--top" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                stats_top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --top".to_string())?;
                i += 1;
            }
            "--warmup-frames" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --warmup-frames".to_string());
                };
                warmup_frames = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --warmup-frames".to_string())?;
                i += 1;
            }
            "--repeat" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --repeat".to_string());
                };
                perf_repeat = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --repeat".to_string())?
                    .max(1);
                i += 1;
            }
            "--max-top-total-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-total-us".to_string());
                };
                max_top_total_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-total-us".to_string())?,
                );
                i += 1;
            }
            "--max-top-layout-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-layout-us".to_string());
                };
                max_top_layout_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-layout-us".to_string())?,
                );
                i += 1;
            }
            "--max-top-solve-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-solve-us".to_string());
                };
                max_top_solve_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-solve-us".to_string())?,
                );
                i += 1;
            }
            "--perf-baseline" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline".to_string());
                };
                perf_baseline_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--perf-baseline-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-out".to_string());
                };
                perf_baseline_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--perf-baseline-headroom-pct" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-headroom-pct".to_string());
                };
                perf_baseline_headroom_pct = v
                    .parse::<u32>()
                    .map_err(|_| "invalid value for --perf-baseline-headroom-pct".to_string())?;
                i += 1;
            }
            "--check-idle-no-paint-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-idle-no-paint-min".to_string());
                };
                check_idle_no_paint_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-idle-no-paint-min".to_string())?,
                );
                i += 1;
            }
            "--check-stale-paint" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-paint".to_string());
                };
                check_stale_paint_test_id = Some(v);
                i += 1;
            }
            "--check-stale-paint-eps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-paint-eps".to_string());
                };
                check_stale_paint_eps = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --check-stale-paint-eps".to_string())?;
                i += 1;
            }
            "--check-stale-scene" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-scene".to_string());
                };
                check_stale_scene_test_id = Some(v);
                i += 1;
            }
            "--check-stale-scene-eps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-scene-eps".to_string());
                };
                check_stale_scene_eps = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --check-stale-scene-eps".to_string())?;
                i += 1;
            }
            "--check-pixels-changed" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-pixels-changed".to_string());
                };
                check_pixels_changed_test_id = Some(v);
                i += 1;
            }
            "--check-semantics-changed-repainted" => {
                check_semantics_changed_repainted = true;
                i += 1;
            }
            "--dump-semantics-changed-repainted-json" => {
                check_semantics_changed_repainted = true;
                dump_semantics_changed_repainted_json = true;
                i += 1;
            }
            "--check-wheel-scroll" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-wheel-scroll".to_string());
                };
                check_wheel_scroll_test_id = Some(v);
                i += 1;
            }
            "--check-drag-cache-root-paint-only" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-drag-cache-root-paint-only".to_string());
                };
                check_drag_cache_root_paint_only_test_id = Some(v);
                i += 1;
            }
            "--check-hover-layout" => {
                check_hover_layout_max = Some(0);
                i += 1;
            }
            "--check-hover-layout-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-hover-layout-max".to_string());
                };
                check_hover_layout_max = Some(
                    v.parse::<u32>()
                        .map_err(|_| "invalid value for --check-hover-layout-max".to_string())?,
                );
                i += 1;
            }
            "--check-gc-sweep-liveness" => {
                check_gc_sweep_liveness = true;
                i += 1;
            }
            "--check-view-cache-reuse-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-view-cache-reuse-min".to_string());
                };
                check_view_cache_reuse_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-view-cache-reuse-min".to_string()
                    })?);
                i += 1;
            }
            "--check-view-cache-reuse-stable-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-view-cache-reuse-stable-min".to_string());
                };
                check_view_cache_reuse_stable_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-view-cache-reuse-stable-min".to_string()
                })?);
                i += 1;
            }
            "--check-overlay-synthesis-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-overlay-synthesis-min".to_string());
                };
                check_overlay_synthesis_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-overlay-synthesis-min".to_string()
                    })?);
                i += 1;
            }
            "--check-viewport-input-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-viewport-input-min".to_string());
                };
                check_viewport_input_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-viewport-input-min".to_string())?,
                );
                i += 1;
            }
            "--check-dock-drag-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-dock-drag-min".to_string());
                };
                check_dock_drag_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-dock-drag-min".to_string())?,
                );
                i += 1;
            }
            "--check-viewport-capture-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-viewport-capture-min".to_string());
                };
                check_viewport_capture_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-viewport-capture-min".to_string()
                    })?);
                i += 1;
            }
            "--check-retained-vlist-reconcile-no-notify" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-retained-vlist-reconcile-no-notify".to_string(),
                    );
                };
                check_retained_vlist_reconcile_no_notify_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-retained-vlist-reconcile-no-notify".to_string()
                    })?);
                i += 1;
            }
            "--check-retained-vlist-attach-detach-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-retained-vlist-attach-detach-max".to_string()
                    );
                };
                check_retained_vlist_attach_detach_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-retained-vlist-attach-detach-max".to_string()
                })?);
                i += 1;
            }
            "--compare-eps-px" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --compare-eps-px".to_string());
                };
                compare_eps_px = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --compare-eps-px".to_string())?;
                i += 1;
            }
            "--compare-ignore-bounds" => {
                compare_ignore_bounds = true;
                i += 1;
            }
            "--compare-ignore-scene-fingerprint" => {
                compare_ignore_scene_fingerprint = true;
                i += 1;
            }
            "--json" => {
                stats_json = true;
                i += 1;
            }
            "--with" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --with (expected tracy|renderdoc)".to_string());
                };
                match v.as_str() {
                    "tracy" => with_tracy = true,
                    "renderdoc" => with_renderdoc = true,
                    other => {
                        return Err(format!(
                            "invalid value for --with: {other} (expected tracy|renderdoc)"
                        ));
                    }
                }
                i += 1;
            }
            "--with-tracy" => {
                with_tracy = true;
                i += 1;
            }
            "--with-renderdoc" => {
                with_renderdoc = true;
                i += 1;
            }
            "--renderdoc-after-frames" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --renderdoc-after-frames".to_string());
                };
                let parsed = v
                    .parse::<u32>()
                    .map_err(|_| "invalid value for --renderdoc-after-frames".to_string())?;
                if parsed == 0 {
                    return Err(
                        "invalid value for --renderdoc-after-frames (must be > 0)".to_string()
                    );
                }
                renderdoc_after_frames = Some(parsed);
                i += 1;
            }
            "--renderdoc-marker" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --renderdoc-marker".to_string());
                };
                renderdoc_markers.push(v);
                i += 1;
            }
            "--renderdoc-no-outputs-png" => {
                renderdoc_no_outputs_png = true;
                i += 1;
            }
            "--env" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --env (expected KEY=VALUE)".to_string());
                };
                let (key, value) = v
                    .split_once('=')
                    .ok_or_else(|| "invalid value for --env (expected KEY=VALUE)".to_string())?;
                let key = key.trim();
                if key.is_empty() {
                    return Err("invalid value for --env (empty KEY)".to_string());
                }
                launch_env.push((key.to_string(), value.to_string()));
                i += 1;
            }
            "--launch" => {
                i += 1;
                let launch_args = args.get(i..).unwrap_or_default();
                if launch_args.is_empty() {
                    return Err("missing command after --launch (try: --launch -- cargo run -p fret-demo --bin todo_demo)".to_string());
                }
                let launch_args: Vec<String> = if launch_args.first().is_some_and(|v| v == "--") {
                    launch_args.iter().skip(1).cloned().collect()
                } else {
                    launch_args.to_vec()
                };
                if launch_args.is_empty() {
                    return Err("missing command after --launch --".to_string());
                }
                launch = Some(launch_args);
                break;
            }
            other if other.starts_with('-') => return Err(format!("unknown diag flag: {other}")),
            _ => {
                positionals.push(arg.clone());
                i += 1;
            }
        }
    }

    let Some(sub) = positionals.first().cloned() else {
        return Err("missing diag subcommand (try: fretboard diag poke)".to_string());
    };
    let rest: Vec<String> = positionals.into_iter().skip(1).collect();

    if sub != "repro" && (with_tracy || with_renderdoc || renderdoc_after_frames.is_some()) {
        return Err(
            "--with tracy/renderdoc and --renderdoc-after-frames are only supported with `diag repro` for now"
                .to_string(),
        );
    }
    if sub != "repro" && (!renderdoc_markers.is_empty() || renderdoc_no_outputs_png) {
        return Err(
            "--renderdoc-marker and --renderdoc-no-outputs-png are only supported with `diag repro` for now"
                .to_string(),
        );
    }

    let workspace_root = crate::cli::workspace_root()?;

    let resolved_out_dir = {
        let raw = out_dir
            .or_else(|| {
                std::env::var_os("FRET_DIAG_DIR")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_trigger_path = {
        let raw = trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("trigger.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_ready_path = {
        let raw = std::env::var_os("FRET_DIAG_READY_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("ready.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_exit_path = {
        let raw = std::env::var_os("FRET_DIAG_EXIT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("exit.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_path = {
        let raw = script_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_trigger_path = {
        let raw = script_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_result_path = {
        let raw = script_result_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_RESULT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.result.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_result_trigger_path = {
        let raw = script_result_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.result.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_trigger_path = {
        let raw = pick_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_result_path = {
        let raw = pick_result_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_RESULT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.result.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_result_trigger_path = {
        let raw = pick_result_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_RESULT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.result.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_script_out = {
        let raw = pick_script_out.unwrap_or_else(|| resolved_out_dir.join("picked.script.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_inspect_path = {
        let raw = inspect_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_INSPECT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("inspect.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_inspect_trigger_path = {
        let raw = inspect_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_INSPECT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("inspect.touch"));
        resolve_path(&workspace_root, raw)
    };

    match sub.as_str() {
        "path" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "poke" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            touch(&resolved_trigger_path)?;
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "latest" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            if let Some(path) = read_latest_pointer(&resolved_out_dir)
                .or_else(|| find_latest_export_dir(&resolved_out_dir))
            {
                println!("{}", path.display());
                return Ok(());
            }
            Err(format!(
                "no diagnostics bundle found under {}",
                resolved_out_dir.display()
            ))
        }
        "pack" => {
            if rest.len() > 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let bundle_dir = match rest.first() {
                Some(src) => {
                    let src = resolve_path(&workspace_root, PathBuf::from(src));
                    resolve_bundle_root_dir(&src)?
                }
                None => read_latest_pointer(&resolved_out_dir)
                    .or_else(|| find_latest_export_dir(&resolved_out_dir))
                    .ok_or_else(|| {
                        format!(
                            "no diagnostics bundle found under {} (try: fretboard diag pack ./target/fret-diag/<timestamp>)",
                            resolved_out_dir.display()
                        )
                    })?,
            };

            let bundle_dir = resolve_bundle_root_dir(&bundle_dir)?;
            let out = pack_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

            let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                resolved_out_dir.clone()
            } else {
                bundle_dir
                    .parent()
                    .unwrap_or(&resolved_out_dir)
                    .to_path_buf()
            };

            pack_bundle_dir_to_zip(
                &bundle_dir,
                &out,
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
                false,
                false,
                &artifacts_root,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
                warmup_frames,
            )?;
            println!("{}", out.display());
            Ok(())
        }
        "triage" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing bundle path (try: fretboard diag triage ./target/fret-diag/1234/bundle.json)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let bundle_path = resolve_bundle_json_path(&src);
            let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);

            let report = bundle_stats_from_path(
                &bundle_path,
                stats_top,
                sort,
                BundleStatsOptions { warmup_frames },
            )?;
            let payload = triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);

            let out = triage_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| default_triage_out_path(&bundle_path));

            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out.display());
            }
            Ok(())
        }
        "script" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag script ./script.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            write_script(&src, &resolved_script_path)?;
            touch(&resolved_script_trigger_path)?;
            println!("{}", resolved_script_trigger_path.display());
            Ok(())
        }
        "run" => {
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag run ./script.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let wants_pack = pack_after_run
                || pack_out.is_some()
                || pack_include_root_artifacts
                || pack_include_triage
                || pack_include_screenshots;

            let mut pack_defaults = (
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
            );
            if pack_after_run && !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
                pack_defaults = (true, true, true);
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let mut child = maybe_launch_demo(
                &launch,
                &launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
                pack_defaults.2,
                timeout_ms,
                poll_ms,
            )?;
            let mut result = run_script_and_wait(
                &src,
                &resolved_script_path,
                &resolved_script_trigger_path,
                &resolved_script_result_path,
                &resolved_script_result_trigger_path,
                timeout_ms,
                poll_ms,
            );
            if let Ok(summary) = &result
                && summary.stage.as_deref() == Some("failed")
            {
                if let Some(dir) =
                    wait_for_failure_dump_bundle(&resolved_out_dir, summary, timeout_ms, poll_ms)
                {
                    if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                        if let Ok(summary) = result.as_mut() {
                            summary.last_bundle_dir = Some(name.to_string());
                        }
                    }
                }
            }
            let result = result?;
            if result.stage.as_deref() == Some("passed") {
                if check_stale_paint_test_id.is_some()
                    || check_stale_scene_test_id.is_some()
                    || check_idle_no_paint_min.is_some()
                    || check_pixels_changed_test_id.is_some()
                    || check_semantics_changed_repainted
                    || check_wheel_scroll_test_id.is_some()
                    || check_drag_cache_root_paint_only_test_id.is_some()
                    || check_hover_layout_max.is_some()
                    || check_gc_sweep_liveness
                    || check_view_cache_reuse_min.is_some()
                    || check_view_cache_reuse_stable_min.is_some()
                    || check_overlay_synthesis_min.is_some()
                    || check_viewport_input_min.is_some()
                    || check_dock_drag_min.is_some()
                    || check_viewport_capture_min.is_some()
                    || check_retained_vlist_reconcile_no_notify_min.is_some()
                    || check_retained_vlist_attach_detach_max.is_some()
                {
                    let bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    )
                    .ok_or_else(|| {
                        "script passed but no bundle.json was found (required for post-run checks)"
                            .to_string()
                    })?;

                    apply_post_run_checks(
                        &bundle_path,
                        &resolved_out_dir,
                        check_idle_no_paint_min,
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_stale_scene_test_id.as_deref(),
                        check_stale_scene_eps,
                        check_pixels_changed_test_id.as_deref(),
                        check_semantics_changed_repainted,
                        dump_semantics_changed_repainted_json,
                        check_wheel_scroll_test_id.as_deref(),
                        check_drag_cache_root_paint_only_test_id.as_deref(),
                        check_hover_layout_max,
                        check_gc_sweep_liveness,
                        check_view_cache_reuse_stable_min,
                        check_view_cache_reuse_min,
                        check_overlay_synthesis_min,
                        check_viewport_input_min,
                        check_dock_drag_min,
                        check_viewport_capture_min,
                        check_retained_vlist_reconcile_no_notify_min,
                        check_retained_vlist_attach_detach_max,
                        warmup_frames,
                    )?;
                }
            }

            if wants_pack {
                let mut bundle_path = wait_for_bundle_json_from_script_result(
                    &resolved_out_dir,
                    &result,
                    timeout_ms,
                    poll_ms,
                );
                if bundle_path.is_none() {
                    let _ = touch(&resolved_trigger_path);
                    bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    );
                }

                if let Some(bundle_path) = bundle_path {
                    let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                    let out = pack_out
                        .clone()
                        .map(|p| resolve_path(&workspace_root, p))
                        .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

                    let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                        resolved_out_dir.clone()
                    } else {
                        bundle_dir
                            .parent()
                            .unwrap_or(&resolved_out_dir)
                            .to_path_buf()
                    };

                    if let Err(err) = pack_bundle_dir_to_zip(
                        &bundle_dir,
                        &out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        false,
                        false,
                        &artifacts_root,
                        stats_top,
                        sort_override.unwrap_or(BundleStatsSort::Invalidation),
                        warmup_frames,
                    ) {
                        eprintln!("PACK-ERROR {err}");
                    } else {
                        println!("PACK {}", out.display());
                    }
                } else {
                    eprintln!(
                        "PACK-ERROR no bundle.json found (add `capture_bundle` or enable script auto-dumps)"
                    );
                }
            }

            let _ = stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            report_result_and_exit(&result);
        }
        "repro" => {
            if rest.is_empty() {
                return Err(
                    "missing script path or suite name (try: fretboard diag repro ui-gallery | fretboard diag repro ./script.json)"
                        .to_string(),
                );
            }

            let mut pack_defaults = (
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
            );
            if !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
                pack_defaults = (true, true, true);
            }

            let (scripts, suite_name): (Vec<PathBuf>, Option<String>) =
                if rest.len() == 1 && rest[0] == "ui-gallery" {
                    (
                        ui_gallery_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some("ui-gallery".to_string()),
                    )
                } else if rest.len() == 1 && rest[0] == "docking-arbitration" {
                    (
                        docking_arbitration_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some("docking-arbitration".to_string()),
                    )
                } else {
                    (
                        rest.into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        None,
                    )
                };

            let summary_path = resolved_out_dir.join("repro.summary.json");

            let mut repro_launch = launch.clone();
            let mut repro_launch_env = launch_env.clone();

            let mut tracy_feature_injected: bool = false;
            let mut tracy_env_enabled: bool = false;
            if with_tracy {
                tracy_env_enabled = ensure_env_var(&mut repro_launch_env, "FRET_TRACY", "1");
                if let Some(cmd) = repro_launch.as_mut() {
                    tracy_feature_injected = cargo_run_inject_feature(cmd, "fret-bootstrap/tracy");
                }

                let note = "\
# Tracy capture (best-effort)\n\
\n\
This repro was run with `FRET_TRACY=1` (and may have auto-injected `--features fret-bootstrap/tracy` when the launch command was `cargo run`).\n\
\n\
Notes:\n\
- Tracy requires running the target with the `fret-bootstrap/tracy` feature enabled.\n\
- The capture file is not recorded automatically by `fretboard` yet. Use the Tracy UI to connect and save a capture.\n\
\n\
See: `docs/tracy.md`.\n";
                let _ = std::fs::write(resolved_out_dir.join("tracy.note.md"), note);
            }

            let mut renderdoc_capture_dir: Option<PathBuf> = None;
            let mut renderdoc_autocapture_after_frames: Option<u32> = None;
            if with_renderdoc {
                let after = renderdoc_after_frames.unwrap_or(60);
                let capture_dir = resolved_out_dir.join("renderdoc");
                let _ = std::fs::create_dir_all(&capture_dir);

                let _ = ensure_env_var(&mut repro_launch_env, "FRET_RENDERDOC", "1");
                let _ = ensure_env_var(
                    &mut repro_launch_env,
                    "FRET_RENDERDOC_CAPTURE_DIR",
                    capture_dir.to_string_lossy().as_ref(),
                );
                let _ = ensure_env_var(
                    &mut repro_launch_env,
                    "FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES",
                    &after.to_string(),
                );

                renderdoc_capture_dir = Some(capture_dir);
                renderdoc_autocapture_after_frames = Some(after);
            }

            let mut child = maybe_launch_demo(
                &repro_launch,
                &repro_launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
                pack_defaults.2,
                timeout_ms,
                poll_ms,
            )?;
            let mut repro_process_footprint: Option<serde_json::Value> = None;

            let mut run_rows: Vec<serde_json::Value> = Vec::new();
            let mut selected_bundle_path: Option<PathBuf> = None;
            let mut last_script_result: Option<ScriptResultSummary> = None;
            let mut overall_error: Option<String> = None;
            let mut pack_items: Vec<ReproPackItem> = Vec::new();

            for src in scripts {
                let mut result = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                );

                if let Ok(summary) = &result
                    && summary.stage.as_deref() == Some("failed")
                {
                    if let Some(dir) = wait_for_failure_dump_bundle(
                        &resolved_out_dir,
                        summary,
                        timeout_ms,
                        poll_ms,
                    ) {
                        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                            if let Ok(summary) = result.as_mut() {
                                summary.last_bundle_dir = Some(name.to_string());
                            }
                        }
                    }
                }

                let result = match result {
                    Ok(r) => r,
                    Err(err) => {
                        overall_error = Some(err);
                        break;
                    }
                };
                last_script_result = Some(result.clone());

                let mut bundle_path = wait_for_bundle_json_from_script_result(
                    &resolved_out_dir,
                    &result,
                    timeout_ms,
                    poll_ms,
                );
                if bundle_path.is_none() {
                    let _ = touch(&resolved_trigger_path);
                    bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    );
                }

                if let Some(bundle_path) = bundle_path.as_ref() {
                    pack_items.push(ReproPackItem {
                        script_path: src.clone(),
                        bundle_json: bundle_path.clone(),
                    });
                }

                if result.stage.as_deref() == Some("failed") && bundle_path.is_some() {
                    selected_bundle_path = bundle_path.clone();
                }
                if selected_bundle_path.is_none() {
                    selected_bundle_path = bundle_path.clone();
                }

                run_rows.push(serde_json::json!({
                    "script_path": src.display().to_string(),
                    "run_id": result.run_id,
                    "stage": result.stage,
                    "step_index": result.step_index,
                    "reason": result.reason,
                    "last_bundle_dir": result.last_bundle_dir,
                    "bundle_json": bundle_path.as_ref().map(|p| p.display().to_string()),
                }));

                if result.stage.as_deref() == Some("passed") {
                    let wants_post_run_checks_for_script = check_stale_paint_test_id.is_some()
                        || check_stale_scene_test_id.is_some()
                        || check_idle_no_paint_min.is_some()
                        || check_pixels_changed_test_id.is_some()
                        || check_semantics_changed_repainted
                        || check_wheel_scroll_test_id.is_some()
                        || check_drag_cache_root_paint_only_test_id.is_some()
                        || check_hover_layout_max.is_some()
                        || check_gc_sweep_liveness
                        || check_view_cache_reuse_min.is_some()
                        || check_view_cache_reuse_stable_min.is_some()
                        || check_overlay_synthesis_min.is_some()
                        || check_viewport_input_min.is_some()
                        || check_dock_drag_min.is_some()
                        || check_viewport_capture_min.is_some()
                        || check_retained_vlist_reconcile_no_notify_min.is_some()
                        || check_retained_vlist_attach_detach_max.is_some();

                    if wants_post_run_checks_for_script {
                        let Some(bundle_path) = bundle_path.as_ref() else {
                            overall_error = Some(
                                "script passed but no bundle.json was found (required for post-run checks)"
                                    .to_string(),
                            );
                            break;
                        };

                        if let Err(err) = apply_post_run_checks(
                            bundle_path,
                            &resolved_out_dir,
                            check_idle_no_paint_min,
                            check_stale_paint_test_id.as_deref(),
                            check_stale_paint_eps,
                            check_stale_scene_test_id.as_deref(),
                            check_stale_scene_eps,
                            check_pixels_changed_test_id.as_deref(),
                            check_semantics_changed_repainted,
                            dump_semantics_changed_repainted_json,
                            check_wheel_scroll_test_id.as_deref(),
                            check_drag_cache_root_paint_only_test_id.as_deref(),
                            check_hover_layout_max,
                            check_gc_sweep_liveness,
                            check_view_cache_reuse_stable_min,
                            check_view_cache_reuse_min,
                            check_overlay_synthesis_min,
                            check_viewport_input_min,
                            check_dock_drag_min,
                            check_viewport_capture_min,
                            check_retained_vlist_reconcile_no_notify_min,
                            check_retained_vlist_attach_detach_max,
                            warmup_frames,
                        ) {
                            overall_error = Some(err);
                            break;
                        }
                    }
                } else {
                    overall_error = Some(format!(
                        "script failed: {} (run_id={}, step={:?}, reason={:?})",
                        src.display(),
                        result.run_id,
                        result.step_index,
                        result.reason
                    ));
                    break;
                }
            }

            let zip_out = pack_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| resolved_out_dir.join("repro.zip"));

            let mut packed_zip: Option<PathBuf> = None;
            let mut packed_bundle_json: Option<PathBuf> = None;
            if let Some(bundle_path) = selected_bundle_path.as_ref() {
                let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                packed_bundle_json = Some(bundle_dir.join("bundle.json"));
            }

            let multi_pack = pack_items.len() > 1;
            let packed_bundles = if multi_pack {
                serde_json::Value::Array(
                    pack_items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            serde_json::json!({
                                "zip_prefix": repro_zip_prefix_for_script(item, idx),
                                "script_path": item.script_path.display().to_string(),
                                "bundle_json": item.bundle_json.display().to_string(),
                            })
                        })
                        .collect(),
                )
            } else {
                serde_json::Value::Null
            };

            let mut renderdoc_capture_payload: Option<serde_json::Value> = None;
            if with_renderdoc {
                let markers: Vec<String> = if renderdoc_markers.is_empty() {
                    vec![
                        "fret clip mask pass".to_string(),
                        "fret downsample-nearest pass".to_string(),
                        "fret upscale-nearest pass".to_string(),
                    ]
                } else {
                    renderdoc_markers.clone()
                };

                if let Some(dir) = renderdoc_capture_dir.as_ref() {
                    let captures = wait_for_files_with_extensions(dir, &["rdc"], 10_000, poll_ms);
                    repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                        &mut child,
                        &resolved_exit_path,
                        poll_ms,
                    ));

                    let mut capture_rows: Vec<serde_json::Value> = Vec::new();
                    for (cap_idx, capture) in captures.iter().enumerate() {
                        let stem = capture
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("capture");
                        let safe_stem = format!(
                            "{:02}-{}",
                            cap_idx.saturating_add(1),
                            zip_safe_component(stem)
                        );
                        let inspect_root = dir.join("inspect").join(&safe_stem);

                        let summary_dir = inspect_root.join("summary");
                        let summary_attempt = run_fret_renderdoc_dump(
                            &workspace_root,
                            capture,
                            &summary_dir,
                            "summary",
                            "",
                            Some(200_000),
                            true,
                            true,
                            Some(30),
                        );

                        let mut attempts: Vec<RenderdocDumpAttempt> = Vec::new();
                        attempts.push(summary_attempt);

                        for (idx, marker) in markers.iter().enumerate() {
                            let safe_marker = zip_safe_component(marker);
                            let out_dir = inspect_root
                                .join(format!("marker_{:02}_{safe_marker}", idx.saturating_add(1)));
                            let attempt = run_fret_renderdoc_dump(
                                &workspace_root,
                                capture,
                                &out_dir,
                                "dump",
                                marker,
                                Some(2_000),
                                true,
                                renderdoc_no_outputs_png,
                                None,
                            );
                            attempts.push(attempt);
                        }

                        let attempt_rows = attempts
                            .into_iter()
                            .map(|a| {
                                let out_dir = a
                                    .out_dir
                                    .strip_prefix(&resolved_out_dir)
                                    .unwrap_or(&a.out_dir)
                                    .display()
                                    .to_string();
                                let stdout_file = a.stdout_file.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });
                                let stderr_file = a.stderr_file.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });
                                let response_json = a.response_json.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });

                                serde_json::json!({
                                    "marker": a.marker,
                                    "out_dir": out_dir,
                                    "exit_code": a.exit_code,
                                    "response_json": response_json,
                                    "stdout_file": stdout_file,
                                    "stderr_file": stderr_file,
                                    "error": a.error,
                                })
                            })
                            .collect::<Vec<_>>();

                        let capture_rel = capture
                            .strip_prefix(&resolved_out_dir)
                            .unwrap_or(capture)
                            .display()
                            .to_string();
                        let inspect_rel = inspect_root
                            .strip_prefix(&resolved_out_dir)
                            .unwrap_or(&inspect_root)
                            .display()
                            .to_string();

                        capture_rows.push(serde_json::json!({
                            "capture": capture_rel,
                            "inspect_dir": inspect_rel,
                            "dumps": attempt_rows,
                        }));
                    }

                    let payload = serde_json::json!({
                        "schema_version": 2,
                        "generated_unix_ms": now_unix_ms(),
                        "capture_dir": "renderdoc",
                        "autocapture_after_frames": renderdoc_autocapture_after_frames,
                        "captures": capture_rows,
                    });
                    let _ = write_json_value(
                        &resolved_out_dir.join("renderdoc.captures.json"),
                        &payload,
                    );
                    renderdoc_capture_payload = Some(payload);
                } else {
                    repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                        &mut child,
                        &resolved_exit_path,
                        poll_ms,
                    ));
                }
            } else {
                repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                    &mut child,
                    &resolved_exit_path,
                    poll_ms,
                ));
            }

            let repro_process_footprint_file = resolved_out_dir.join("resource.footprint.json");
            if let Some(payload) = repro_process_footprint.as_ref() {
                let _ = write_json_value(&repro_process_footprint_file, payload);
            }

            let captures_json = serde_json::json!({
                "tracy": if with_tracy {
                    serde_json::json!({
                        "requested": true,
                        "env_enabled": tracy_env_enabled,
                        "feature_injected": tracy_feature_injected,
                        "note": "Capture is not recorded automatically yet; use the Tracy UI to save a capture."
                    })
                } else {
                    serde_json::Value::Null
                },
                "renderdoc": if with_renderdoc {
                    renderdoc_capture_payload.clone().unwrap_or_else(|| serde_json::json!({
                        "schema_version": 2,
                        "generated_unix_ms": now_unix_ms(),
                        "capture_dir": "renderdoc",
                        "autocapture_after_frames": renderdoc_autocapture_after_frames,
                        "captures": [],
                    }))
                } else {
                    serde_json::Value::Null
                }
            });

            let summary_json = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": now_unix_ms(),
                "out_dir": resolved_out_dir.display().to_string(),
                "suite": suite_name,
                "scripts": run_rows,
                "selected_bundle_json": selected_bundle_path.as_ref().map(|p| p.display().to_string()),
                "packed_bundle_json": packed_bundle_json.as_ref().map(|p| p.display().to_string()),
                "packed_bundles": packed_bundles,
                "repro_zip": Some(zip_out.display().to_string()),
                "resources": serde_json::json!({
                    "process_footprint_file": if repro_process_footprint_file.is_file() {
                        Some("resource.footprint.json")
                    } else {
                        None
                    },
                    "process_footprint": repro_process_footprint,
                }),
                "captures": captures_json,
                "last_result": last_script_result.as_ref().map(|r| serde_json::json!({
                    "run_id": r.run_id,
                    "stage": r.stage,
                    "step_index": r.step_index,
                    "reason": r.reason,
                    "last_bundle_dir": r.last_bundle_dir,
                })),
                "error": overall_error,
            });

            if let Some(parent) = summary_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = write_json_value(&summary_path, &summary_json);
            let _ = write_evidence_index(&resolved_out_dir, &summary_path, Some(&summary_json));

            if overall_error.is_none() {
                let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
                if multi_pack {
                    let bundles: Vec<ReproZipBundle> = pack_items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| ReproZipBundle {
                            prefix: repro_zip_prefix_for_script(item, idx),
                            bundle_json: item.bundle_json.clone(),
                            source_script: item.script_path.clone(),
                        })
                        .collect();

                    if let Err(err) = pack_repro_zip_multi(
                        &zip_out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        with_renderdoc,
                        with_tracy,
                        &resolved_out_dir,
                        &summary_path,
                        &bundles,
                        stats_top,
                        sort,
                        warmup_frames,
                    ) {
                        overall_error = Some(format!("failed to pack repro zip: {err}"));
                    } else {
                        packed_zip = Some(zip_out.clone());
                    }
                } else if let Some(bundle_path) = selected_bundle_path.as_ref() {
                    let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                    let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                        resolved_out_dir.clone()
                    } else {
                        bundle_dir
                            .parent()
                            .unwrap_or(&resolved_out_dir)
                            .to_path_buf()
                    };

                    if let Err(err) = pack_bundle_dir_to_zip(
                        &bundle_dir,
                        &zip_out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        with_renderdoc,
                        with_tracy,
                        &artifacts_root,
                        stats_top,
                        sort,
                        warmup_frames,
                    ) {
                        overall_error = Some(format!("failed to pack repro zip: {err}"));
                    } else {
                        packed_zip = Some(zip_out.clone());
                    }
                } else {
                    overall_error = Some(
                        "no bundle.json found (add `capture_bundle` or enable script auto-dumps)"
                            .to_string(),
                    );
                }

                if overall_error.is_some() {
                    // Keep the summary coherent even when packing fails.
                    let _ = write_json_value(
                        &summary_path,
                        &summary_json
                            .as_object()
                            .cloned()
                            .map(|mut obj| {
                                obj.insert(
                                    "error".to_string(),
                                    serde_json::Value::String(
                                        overall_error.clone().unwrap_or_default(),
                                    ),
                                );
                                serde_json::Value::Object(obj)
                            })
                            .unwrap_or(summary_json.clone()),
                    );
                }
            }

            let final_summary_json = summary_json
                .as_object()
                .cloned()
                .map(|mut obj| {
                    if let Some(err) = overall_error.as_ref() {
                        obj.insert("error".to_string(), serde_json::Value::String(err.clone()));
                    }
                    serde_json::Value::Object(obj)
                })
                .unwrap_or_else(|| summary_json.clone());
            let _ = write_json_value(&summary_path, &final_summary_json);
            if let Err(err) =
                write_evidence_index(&resolved_out_dir, &summary_path, Some(&final_summary_json))
            {
                eprintln!("WARN failed to write evidence index: {err}");
            }

            if let Some(path) = packed_bundle_json.as_ref() {
                println!("BUNDLE {}", path.display());
            }
            if let Some(path) = packed_zip.as_ref() {
                println!("PACK {}", path.display());
            }
            println!("SUMMARY {}", summary_path.display());

            if let Some(err) = overall_error {
                eprintln!("REPRO-FAIL {err}");
                std::process::exit(1);
            }

            println!("REPRO-OK");
            std::process::exit(0);
        }
        "suite" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if rest.is_empty() {
                return Err(
                    "missing suite name or script paths (try: fretboard diag suite ui-gallery | fretboard diag suite docking-arbitration)"
                        .to_string(),
                );
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            enum BuiltinSuite {
                UiGallery,
                DockingArbitration,
            }

            let (scripts, builtin_suite): (Vec<PathBuf>, Option<BuiltinSuite>) = if rest.len() == 1
                && rest[0] == "ui-gallery"
            {
                (
                    ui_gallery_suite_scripts()
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect(),
                    Some(BuiltinSuite::UiGallery),
                )
            } else if rest.len() == 1 && rest[0] == "ui-gallery-virt-retained" {
                (
                    vec![resolve_path(
                        &workspace_root,
                        PathBuf::from(
                            "tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll-retained.json",
                        ),
                    )],
                    Some(BuiltinSuite::UiGallery),
                )
            } else if rest.len() == 1 && rest[0] == "docking-arbitration" {
                (
                    docking_arbitration_suite_scripts()
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect(),
                    Some(BuiltinSuite::DockingArbitration),
                )
            } else {
                (
                    rest.into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect(),
                    None,
                )
            };

            let reuse_process = launch.is_none();
            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    false,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };
            for src in scripts {
                if !reuse_process {
                    child = maybe_launch_demo(
                        &launch,
                        &launch_env,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
                        &resolved_exit_path,
                        false,
                        timeout_ms,
                        poll_ms,
                    )?;
                }
                let mut result = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                );
                if let Ok(summary) = &result
                    && summary.stage.as_deref() == Some("failed")
                {
                    if let Some(dir) = wait_for_failure_dump_bundle(
                        &resolved_out_dir,
                        summary,
                        timeout_ms,
                        poll_ms,
                    ) {
                        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                            if let Ok(summary) = result.as_mut() {
                                summary.last_bundle_dir = Some(name.to_string());
                            }
                        }
                    }
                }

                let result = match result {
                    Ok(v) => v,
                    Err(e) => {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        return Err(e);
                    }
                };
                match result.stage.as_deref() {
                    Some("passed") => {
                        println!("PASS {} (run_id={})", src.display(), result.run_id)
                    }
                    Some("failed") => {
                        eprintln!(
                            "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                            src.display(),
                            result.run_id,
                            result.step_index.unwrap_or(0),
                            result.reason.as_deref().unwrap_or("unknown"),
                            result.last_bundle_dir.as_deref().unwrap_or("")
                        );
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!(
                            "unexpected script stage for {}: {:?}",
                            src.display(),
                            result
                        );
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        std::process::exit(1);
                    }
                }

                let retained_vlist_gate_for_script = check_retained_vlist_reconcile_no_notify_min
                    .filter(|_| ui_gallery_script_requires_retained_vlist_reconcile_gate(&src));
                let retained_vlist_attach_detach_max_for_script =
                    check_retained_vlist_attach_detach_max
                        .filter(|_| ui_gallery_script_requires_retained_vlist_reconcile_gate(&src));
                let wants_post_run_checks_for_script = check_stale_paint_test_id.is_some()
                    || check_stale_scene_test_id.is_some()
                    || check_idle_no_paint_min.is_some()
                    || check_pixels_changed_test_id.is_some()
                    || check_semantics_changed_repainted
                    || check_wheel_scroll_test_id.is_some()
                    || check_drag_cache_root_paint_only_test_id.is_some()
                    || check_hover_layout_max.is_some()
                    || check_gc_sweep_liveness
                    || check_view_cache_reuse_min.is_some()
                    || check_view_cache_reuse_stable_min.is_some()
                    || check_overlay_synthesis_min.is_some()
                    || check_viewport_input_min.is_some()
                    || check_dock_drag_min.is_some()
                    || check_viewport_capture_min.is_some()
                    || retained_vlist_gate_for_script.is_some()
                    || retained_vlist_attach_detach_max_for_script.is_some();

                let wants_post_run_checks_for_script = wants_post_run_checks_for_script
                    || builtin_suite == Some(BuiltinSuite::DockingArbitration);

                if result.stage.as_deref() == Some("passed") && wants_post_run_checks_for_script {
                    let bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    )
                    .ok_or_else(|| {
                        format!(
                            "script passed but no bundle.json was found (required for post-run checks): {}",
                            src.display()
                        )
                    })?;

                    let (suite_viewport_input_min, suite_dock_drag_min, suite_viewport_capture_min) =
                        if builtin_suite == Some(BuiltinSuite::DockingArbitration) {
                            docking_arbitration_script_default_gates(&src)
                        } else {
                            (None, None, None)
                        };
                    apply_post_run_checks(
                        &bundle_path,
                        &resolved_out_dir,
                        check_idle_no_paint_min,
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_stale_scene_test_id.as_deref(),
                        check_stale_scene_eps,
                        check_pixels_changed_test_id.as_deref(),
                        check_semantics_changed_repainted,
                        dump_semantics_changed_repainted_json,
                        check_wheel_scroll_test_id.as_deref(),
                        check_drag_cache_root_paint_only_test_id.as_deref(),
                        check_hover_layout_max,
                        check_gc_sweep_liveness,
                        check_view_cache_reuse_stable_min,
                        check_view_cache_reuse_min,
                        check_overlay_synthesis_min,
                        check_viewport_input_min.or(suite_viewport_input_min),
                        check_dock_drag_min.or(suite_dock_drag_min),
                        check_viewport_capture_min.or(suite_viewport_capture_min),
                        retained_vlist_gate_for_script,
                        retained_vlist_attach_detach_max_for_script,
                        warmup_frames,
                    )?;
                }

                if !reuse_process {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            std::process::exit(0);
        }
        "perf" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if rest.is_empty() {
                return Err(
                    "missing suite name or script paths (try: fretboard diag perf ui-gallery)"
                        .to_string(),
                );
            }

            let scripts: Vec<PathBuf> = if rest.len() == 1 && rest[0] == "ui-gallery" {
                [
                    "tools/diag-scripts/ui-gallery-overlay-torture.json",
                    "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
                    "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
                    "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
                    "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
                    "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
                ]
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect()
            } else {
                rest.into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
            };

            let sort = sort_override.unwrap_or(BundleStatsSort::Time);
            let repeat = perf_repeat.max(1) as usize;
            let reuse_process = launch.is_none();
            let cli_thresholds = PerfThresholds {
                max_top_total_us,
                max_top_layout_us,
                max_top_solve_us,
            };
            let perf_baseline = perf_baseline_path
                .clone()
                .map(|p| resolve_path(&workspace_root, p))
                .map(|p| read_perf_baseline_file(&workspace_root, &p))
                .transpose()?;
            let perf_baseline_out = perf_baseline_out
                .clone()
                .map(|p| resolve_path(&workspace_root, p));
            let wants_perf_thresholds = cli_thresholds.any() || perf_baseline.is_some();
            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    false,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };

            let mut perf_json_rows: Vec<serde_json::Value> = Vec::new();
            let mut perf_threshold_rows: Vec<serde_json::Value> = Vec::new();
            let mut perf_threshold_failures: Vec<serde_json::Value> = Vec::new();
            let mut perf_baseline_rows: Vec<serde_json::Value> = Vec::new();
            let mut overall_worst: Option<(u64, PathBuf, PathBuf)> = None;
            let stats_opts = BundleStatsOptions { warmup_frames };

            if let Some(baseline) = perf_baseline.as_ref() {
                for src in &scripts {
                    let key = normalize_repo_relative_path(&workspace_root, src);
                    if !baseline.thresholds_by_script.contains_key(&key) {
                        return Err(format!(
                            "perf baseline missing entry for script: {key} (baseline={})",
                            baseline.path.display()
                        ));
                    }
                }
            }

            for src in scripts {
                if repeat == 1 {
                    if !reuse_process {
                        child = maybe_launch_demo(
                            &launch,
                            &launch_env,
                            &workspace_root,
                            &resolved_out_dir,
                            &resolved_ready_path,
                            &resolved_exit_path,
                            false,
                            timeout_ms,
                            poll_ms,
                        )?;
                    }

                    if !reuse_process {
                        clear_script_result_files(
                            &resolved_script_result_path,
                            &resolved_script_result_trigger_path,
                        );
                    }

                    let mut result = run_script_and_wait(
                        &src,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    );
                    if let Ok(summary) = &result
                        && summary.stage.as_deref() == Some("failed")
                    {
                        if let Some(dir) = wait_for_failure_dump_bundle(
                            &resolved_out_dir,
                            summary,
                            timeout_ms,
                            poll_ms,
                        ) {
                            if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                                if let Ok(summary) = result.as_mut() {
                                    summary.last_bundle_dir = Some(name.to_string());
                                }
                            }
                        }
                    }
                    let result = match result {
                        Ok(v) => v,
                        Err(e) => {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            return Err(e);
                        }
                    };

                    match result.stage.as_deref() {
                        Some("passed") => {}
                        Some("failed") => {
                            eprintln!(
                                "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                                src.display(),
                                result.run_id,
                                result.step_index.unwrap_or(0),
                                result.reason.as_deref().unwrap_or("unknown"),
                                result.last_bundle_dir.as_deref().unwrap_or("")
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!(
                                "unexpected script stage for {}: {:?}",
                                src.display(),
                                result
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                    }

                    let bundle_dir = result
                        .last_bundle_dir
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .map(PathBuf::from);

                    let script_key = normalize_repo_relative_path(&workspace_root, &src);

                    if let Some(bundle_dir) = bundle_dir {
                        let bundle_path =
                            resolve_bundle_json_path(&resolved_out_dir.join(bundle_dir));
                        let mut report = bundle_stats_from_path(
                            &bundle_path,
                            stats_top.max(1),
                            sort,
                            stats_opts,
                        )?;
                        if warmup_frames > 0 && report.top.is_empty() {
                            report = bundle_stats_from_path(
                                &bundle_path,
                                stats_top.max(1),
                                sort,
                                BundleStatsOptions::default(),
                            )?;
                        }
                        let top = report.top.first();
                        let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                        let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                        let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
                        let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
                        let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                        let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                        let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                        let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                        let top_view_cache_contained_relayouts =
                            top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
                        let top_cache_roots_contained_relayout =
                            top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
                        let top_set_children_barrier_writes =
                            top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
                        let top_barrier_relayouts_scheduled =
                            top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
                        let top_barrier_relayouts_performed =
                            top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
                        let top_virtual_list_visible_range_checks = top
                            .map(|r| r.virtual_list_visible_range_checks)
                            .unwrap_or(0);
                        let top_virtual_list_visible_range_refreshes = top
                            .map(|r| r.virtual_list_visible_range_refreshes)
                            .unwrap_or(0);

                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "top_total_time_us": top_total,
                                "top_layout_time_us": top_layout,
                                "top_layout_engine_solve_time_us": top_solve,
                                "top_layout_engine_solves": top_solves,
                                "top_prepaint_time_us": top_prepaint,
                                "top_paint_time_us": top_paint,
                                "top_tick_id": top_tick,
                                "top_frame_id": top_frame,
                                "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                                "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                                "top_set_children_barrier_writes": top_set_children_barrier_writes,
                                "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                                "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                                "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                                "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                                "bundle": bundle_path.display().to_string(),
                            }));
                        } else {
                            println!(
                                "PERF {} sort={} top.us(total/layout/solve/prepaint/paint)={}/{}/{}/{}/{} top.tick={} top.frame={} bundle={}",
                                src.display(),
                                sort.as_str(),
                                top_total,
                                top_layout,
                                top_solve,
                                top_prepaint,
                                top_paint,
                                top_tick,
                                top_frame,
                                bundle_path.display(),
                            );
                        }

                        if perf_baseline_out.is_some() {
                            perf_baseline_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "max": {
                                    "top_total_time_us": top_total,
                                    "top_layout_time_us": top_layout,
                                    "top_layout_engine_solve_time_us": top_solve,
                                },
                            }));
                        }
                        if wants_perf_thresholds {
                            let baseline_thresholds = perf_baseline
                                .as_ref()
                                .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                                .unwrap_or_default();
                            let (thr_total, src_total) = resolve_threshold(
                                cli_thresholds.max_top_total_us,
                                baseline_thresholds.max_top_total_us,
                            );
                            let (thr_layout, src_layout) = resolve_threshold(
                                cli_thresholds.max_top_layout_us,
                                baseline_thresholds.max_top_layout_us,
                            );
                            let (thr_solve, src_solve) = resolve_threshold(
                                cli_thresholds.max_top_solve_us,
                                baseline_thresholds.max_top_solve_us,
                            );
                            let run = serde_json::json!({
                                "run_index": 0,
                                "top_total_time_us": top_total,
                                "top_layout_time_us": top_layout,
                                "top_layout_engine_solve_time_us": top_solve,
                                "top_layout_engine_solves": top_solves,
                                "top_tick_id": top_tick,
                                "top_frame_id": top_frame,
                                "bundle": bundle_path.display().to_string(),
                            });
                            let row = serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "repeat": 1,
                                "runs": [run],
                                "max": {
                                    "top_total_time_us": top_total,
                                    "top_layout_time_us": top_layout,
                                    "top_layout_engine_solve_time_us": top_solve,
                                },
                                "thresholds": {
                                    "max_top_total_us": thr_total,
                                    "max_top_layout_us": thr_layout,
                                    "max_top_solve_us": thr_solve,
                                },
                                "threshold_sources": {
                                    "max_top_total_us": src_total,
                                    "max_top_layout_us": src_layout,
                                    "max_top_solve_us": src_solve,
                                },
                            });
                            perf_threshold_rows.push(row);
                            perf_threshold_failures.extend(scan_perf_threshold_failures(
                                script_key.as_str(),
                                sort,
                                cli_thresholds,
                                baseline_thresholds,
                                top_total,
                                top_layout,
                                top_solve,
                            ));
                        }

                        match &overall_worst {
                            Some((prev_us, _, _)) if *prev_us >= top_total => {}
                            _ => overall_worst = Some((top_total, src.clone(), bundle_path)),
                        }
                    } else {
                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "error": "no_last_bundle_dir",
                            }));
                        } else {
                            println!(
                                "PERF {} sort={} (no last_bundle_dir recorded)",
                                src.display(),
                                sort.as_str()
                            );
                        }
                    }

                    if !reuse_process {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    }
                    continue;
                }

                let mut runs_total: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_layout: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_solve: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_prepaint: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_paint: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_json: Vec<serde_json::Value> = Vec::with_capacity(repeat);
                let mut script_worst: Option<(u64, PathBuf)> = None;

                for run_index in 0..repeat {
                    if !reuse_process {
                        child = maybe_launch_demo(
                            &launch,
                            &launch_env,
                            &workspace_root,
                            &resolved_out_dir,
                            &resolved_ready_path,
                            &resolved_exit_path,
                            false,
                            timeout_ms,
                            poll_ms,
                        )?;
                    }

                    if !reuse_process {
                        clear_script_result_files(
                            &resolved_script_result_path,
                            &resolved_script_result_trigger_path,
                        );
                    }

                    let mut result = run_script_and_wait(
                        &src,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    );
                    if let Ok(summary) = &result
                        && summary.stage.as_deref() == Some("failed")
                    {
                        if let Some(dir) = wait_for_failure_dump_bundle(
                            &resolved_out_dir,
                            summary,
                            timeout_ms,
                            poll_ms,
                        ) {
                            if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                                if let Ok(summary) = result.as_mut() {
                                    summary.last_bundle_dir = Some(name.to_string());
                                }
                            }
                        }
                    }
                    let result = match result {
                        Ok(v) => v,
                        Err(e) => {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            return Err(e);
                        }
                    };

                    match result.stage.as_deref() {
                        Some("passed") => {}
                        Some("failed") => {
                            eprintln!(
                                "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                                src.display(),
                                result.run_id,
                                result.step_index.unwrap_or(0),
                                result.reason.as_deref().unwrap_or("unknown"),
                                result.last_bundle_dir.as_deref().unwrap_or("")
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!(
                                "unexpected script stage for {}: {:?}",
                                src.display(),
                                result
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                    }

                    let bundle_dir = result
                        .last_bundle_dir
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .map(PathBuf::from);

                    let Some(bundle_dir) = bundle_dir else {
                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": src.display().to_string(),
                                "sort": sort.as_str(),
                                "repeat": repeat,
                                "error": "no_last_bundle_dir",
                            }));
                        } else {
                            println!(
                                "PERF {} sort={} repeat={} (no last_bundle_dir recorded)",
                                src.display(),
                                sort.as_str(),
                                repeat
                            );
                        }
                        if !reuse_process {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        }
                        break;
                    };

                    let bundle_path =
                        resolve_bundle_json_path(&resolved_out_dir.join(bundle_dir.clone()));
                    let mut report =
                        bundle_stats_from_path(&bundle_path, stats_top.max(1), sort, stats_opts)?;
                    if warmup_frames > 0 && report.top.is_empty() {
                        report = bundle_stats_from_path(
                            &bundle_path,
                            stats_top.max(1),
                            sort,
                            BundleStatsOptions::default(),
                        )?;
                    }
                    let top = report.top.first();
                    let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                    let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                    let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
                    let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
                    let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                    let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                    let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                    let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                    let top_view_cache_contained_relayouts =
                        top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
                    let top_cache_roots_contained_relayout =
                        top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
                    let top_set_children_barrier_writes =
                        top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
                    let top_barrier_relayouts_scheduled =
                        top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
                    let top_barrier_relayouts_performed =
                        top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
                    let top_virtual_list_visible_range_checks = top
                        .map(|r| r.virtual_list_visible_range_checks)
                        .unwrap_or(0);
                    let top_virtual_list_visible_range_refreshes = top
                        .map(|r| r.virtual_list_visible_range_refreshes)
                        .unwrap_or(0);

                    runs_total.push(top_total);
                    runs_layout.push(top_layout);
                    runs_solve.push(top_solve);
                    runs_prepaint.push(top_prepaint);
                    runs_paint.push(top_paint);
                    runs_json.push(serde_json::json!({
                        "run_index": run_index,
                        "top_total_time_us": top_total,
                        "top_layout_time_us": top_layout,
                        "top_layout_engine_solve_time_us": top_solve,
                        "top_layout_engine_solves": top_solves,
                        "top_prepaint_time_us": top_prepaint,
                        "top_paint_time_us": top_paint,
                        "top_tick_id": top_tick,
                        "top_frame_id": top_frame,
                        "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                        "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                        "top_set_children_barrier_writes": top_set_children_barrier_writes,
                        "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                        "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                        "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                        "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                        "bundle": bundle_path.display().to_string(),
                    }));

                    match &script_worst {
                        Some((prev_us, _)) if *prev_us >= top_total => {}
                        _ => script_worst = Some((top_total, bundle_path.clone())),
                    }

                    match &overall_worst {
                        Some((prev_us, _, _)) if *prev_us >= top_total => {}
                        _ => overall_worst = Some((top_total, src.clone(), bundle_path.clone())),
                    }

                    if !reuse_process {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    }
                }

                if runs_total.len() == repeat {
                    if stats_json {
                        let mut top_view_cache_contained_relayouts: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_cache_roots_contained_relayout: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_set_children_barrier_writes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_barrier_relayouts_scheduled: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_barrier_relayouts_performed: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_virtual_list_visible_range_checks: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_virtual_list_visible_range_refreshes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        for run in &runs_json {
                            top_view_cache_contained_relayouts.push(
                                run.get("top_view_cache_contained_relayouts")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_cache_roots_contained_relayout.push(
                                run.get("top_cache_roots_contained_relayout")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_set_children_barrier_writes.push(
                                run.get("top_set_children_barrier_writes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_barrier_relayouts_scheduled.push(
                                run.get("top_barrier_relayouts_scheduled")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_barrier_relayouts_performed.push(
                                run.get("top_barrier_relayouts_performed")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_virtual_list_visible_range_checks.push(
                                run.get("top_virtual_list_visible_range_checks")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_virtual_list_visible_range_refreshes.push(
                                run.get("top_virtual_list_visible_range_refreshes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                        }
                        perf_json_rows.push(serde_json::json!({
                            "script": src.display().to_string(),
                            "sort": sort.as_str(),
                            "repeat": repeat,
                            "runs": runs_json,
                            "stats": {
                                "total_time_us": summarize_times_us(&runs_total),
                                "layout_time_us": summarize_times_us(&runs_layout),
                                "layout_engine_solve_time_us": summarize_times_us(&runs_solve),
                                "prepaint_time_us": summarize_times_us(&runs_prepaint),
                                "paint_time_us": summarize_times_us(&runs_paint),
                                "top_view_cache_contained_relayouts": summarize_times_us(&top_view_cache_contained_relayouts),
                                "top_cache_roots_contained_relayout": summarize_times_us(&top_cache_roots_contained_relayout),
                                "top_set_children_barrier_writes": summarize_times_us(&top_set_children_barrier_writes),
                                "top_barrier_relayouts_scheduled": summarize_times_us(&top_barrier_relayouts_scheduled),
                                "top_barrier_relayouts_performed": summarize_times_us(&top_barrier_relayouts_performed),
                                "top_virtual_list_visible_range_checks": summarize_times_us(&top_virtual_list_visible_range_checks),
                                "top_virtual_list_visible_range_refreshes": summarize_times_us(&top_virtual_list_visible_range_refreshes),
                            },
                            "worst_run": script_worst.as_ref().map(|(us, bundle)| serde_json::json!({
                                "top_total_time_us": us,
                                "bundle": bundle.display().to_string(),
                            })),
                        }));
                    } else {
                        let total = summarize_times_us(&runs_total);
                        let layout = summarize_times_us(&runs_layout);
                        let solve = summarize_times_us(&runs_solve);
                        let prepaint = summarize_times_us(&runs_prepaint);
                        let paint = summarize_times_us(&runs_paint);
                        println!(
                            "PERF {} sort={} repeat={} p50.us(total/layout/solve/prepaint/paint)={}/{}/{}/{}/{} p95.us(total/layout/solve/prepaint/paint)={}/{}/{}/{}/{} max.us(total/layout/solve/prepaint/paint)={}/{}/{}/{}/{}",
                            src.display(),
                            sort.as_str(),
                            repeat,
                            total.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            total.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            total.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                        );
                    }

                    let max_total = *runs_total.iter().max().unwrap_or(&0);
                    let max_layout = *runs_layout.iter().max().unwrap_or(&0);
                    let max_solve = *runs_solve.iter().max().unwrap_or(&0);
                    let script_key = normalize_repo_relative_path(&workspace_root, &src);

                    if perf_baseline_out.is_some() {
                        perf_baseline_rows.push(serde_json::json!({
                            "script": script_key.clone(),
                            "max": {
                                "top_total_time_us": max_total,
                                "top_layout_time_us": max_layout,
                                "top_layout_engine_solve_time_us": max_solve,
                            },
                        }));
                    }

                    if wants_perf_thresholds {
                        let baseline_thresholds = perf_baseline
                            .as_ref()
                            .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                            .unwrap_or_default();
                        let (thr_total, src_total) = resolve_threshold(
                            cli_thresholds.max_top_total_us,
                            baseline_thresholds.max_top_total_us,
                        );
                        let (thr_layout, src_layout) = resolve_threshold(
                            cli_thresholds.max_top_layout_us,
                            baseline_thresholds.max_top_layout_us,
                        );
                        let (thr_solve, src_solve) = resolve_threshold(
                            cli_thresholds.max_top_solve_us,
                            baseline_thresholds.max_top_solve_us,
                        );
                        let row = serde_json::json!({
                            "script": script_key.clone(),
                            "sort": sort.as_str(),
                            "repeat": repeat,
                            "runs": runs_json,
                            "max": {
                                "top_total_time_us": max_total,
                                "top_layout_time_us": max_layout,
                                "top_layout_engine_solve_time_us": max_solve,
                            },
                            "thresholds": {
                                "max_top_total_us": thr_total,
                                "max_top_layout_us": thr_layout,
                                "max_top_solve_us": thr_solve,
                            },
                            "threshold_sources": {
                                "max_top_total_us": src_total,
                                "max_top_layout_us": src_layout,
                                "max_top_solve_us": src_solve,
                            },
                        });
                        perf_threshold_rows.push(row);
                        perf_threshold_failures.extend(scan_perf_threshold_failures(
                            script_key.as_str(),
                            sort,
                            cli_thresholds,
                            baseline_thresholds,
                            max_total,
                            max_layout,
                            max_solve,
                        ));
                    }
                }
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

            if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
                check_out_dir_for_pixels_changed(&resolved_out_dir, test_id, warmup_frames)?;
            }

            if let Some(path) = perf_baseline_out.as_ref() {
                let out_path = path;
                let rows = perf_baseline_rows
                    .iter()
                    .filter_map(|row| {
                        let script = row.get("script")?.as_str()?.to_string();
                        let max = row.get("max")?;
                        let max_total = max.get("top_total_time_us")?.as_u64()?;
                        let max_layout = max.get("top_layout_time_us")?.as_u64()?;
                        let max_solve = max.get("top_layout_engine_solve_time_us")?.as_u64()?;
                        let thr_total =
                            apply_perf_baseline_headroom(max_total, perf_baseline_headroom_pct);
                        let thr_layout =
                            apply_perf_baseline_headroom(max_layout, perf_baseline_headroom_pct);
                        let thr_solve =
                            apply_perf_baseline_headroom(max_solve, perf_baseline_headroom_pct);
                        Some(serde_json::json!({
                            "script": script,
                            "thresholds": {
                                "max_top_total_us": thr_total,
                                "max_top_layout_us": thr_layout,
                                "max_top_solve_us": thr_solve,
                            },
                            "measured_max": {
                                "top_total_time_us": max_total,
                                "top_layout_time_us": max_layout,
                                "top_layout_engine_solve_time_us": max_solve,
                            },
                        }))
                    })
                    .collect::<Vec<_>>();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": now_unix_ms(),
                    "kind": "perf_baseline",
                    "out_path": out_path.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "headroom_pct": perf_baseline_headroom_pct,
                    "rows": rows,
                });
                write_json_value(out_path, &payload)?;
                if !stats_json {
                    println!("wrote perf baseline: {}", out_path.display());
                }
            }

            if wants_perf_thresholds {
                let out_path = resolved_out_dir.join("check.perf_thresholds.json");
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": now_unix_ms(),
                    "kind": "perf_thresholds",
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "thresholds": {
                        "max_top_total_us": cli_thresholds.max_top_total_us,
                        "max_top_layout_us": cli_thresholds.max_top_layout_us,
                        "max_top_solve_us": cli_thresholds.max_top_solve_us,
                    },
                    "baseline": perf_baseline.as_ref().map(|b| serde_json::json!({
                        "path": b.path.display().to_string(),
                        "scripts": b.thresholds_by_script.len(),
                    })),
                    "rows": perf_threshold_rows,
                    "failures": perf_threshold_failures,
                });
                let _ = write_json_value(&out_path, &payload);
                if !perf_threshold_failures.is_empty() {
                    eprintln!(
                        "PERF threshold gate failed (failures={}, evidence={})",
                        perf_threshold_failures.len(),
                        out_path.display()
                    );
                    std::process::exit(1);
                }
            }

            if stats_json {
                let worst = overall_worst.as_ref().map(|(us, src, bundle)| {
                    serde_json::json!({
                        "script": src.display().to_string(),
                        "top_total_time_us": us,
                        "bundle": bundle.display().to_string(),
                    })
                });
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "rows": perf_json_rows,
                    "worst_overall": worst,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
            } else if let Some((us, src, bundle)) = overall_worst {
                println!(
                    "PERF worst overall: {} us={} bundle={}",
                    src.display(),
                    us,
                    bundle.display()
                );
            }

            std::process::exit(0);
        }
        "stats" => {
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing bundle path (try: fretboard diag stats ./target/fret-diag/1234/bundle.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let bundle_path = resolve_bundle_json_path(&src);
            let mut report = bundle_stats_from_path(
                &bundle_path,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
                BundleStatsOptions { warmup_frames },
            )?;
            if warmup_frames > 0 && report.top.is_empty() {
                report = bundle_stats_from_path(
                    &bundle_path,
                    stats_top,
                    sort_override.unwrap_or(BundleStatsSort::Invalidation),
                    BundleStatsOptions::default(),
                )?;
            }

            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report.to_json())
                        .unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                report.print_human(&bundle_path);
            }
            if let Some(test_id) = check_stale_paint_test_id.as_deref() {
                check_bundle_for_stale_paint(&bundle_path, test_id, check_stale_paint_eps)?;
            }
            if let Some(test_id) = check_stale_scene_test_id.as_deref() {
                check_bundle_for_stale_scene(&bundle_path, test_id, check_stale_scene_eps)?;
            }
            if let Some(min) = check_idle_no_paint_min {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_bundle_for_idle_no_paint_min(&bundle_path, out_dir, min, warmup_frames)?;
            }
            if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_out_dir_for_pixels_changed(out_dir, test_id, warmup_frames)?;
            }
            if check_semantics_changed_repainted {
                check_bundle_for_semantics_changed_repainted(
                    &bundle_path,
                    warmup_frames,
                    dump_semantics_changed_repainted_json,
                )?;
            }
            if let Some(test_id) = check_wheel_scroll_test_id.as_deref() {
                check_bundle_for_wheel_scroll(bundle_path.as_path(), test_id, warmup_frames)?;
            }
            if let Some(test_id) = check_drag_cache_root_paint_only_test_id.as_deref() {
                check_bundle_for_drag_cache_root_paint_only(&bundle_path, test_id, warmup_frames)?;
            }
            if let Some(max_allowed) = check_hover_layout_max {
                check_report_for_hover_layout_invalidations(&report, max_allowed)?;
            }
            if check_gc_sweep_liveness {
                check_bundle_for_gc_sweep_liveness(bundle_path.as_path(), warmup_frames)?;
            }
            if let Some(min) = check_view_cache_reuse_stable_min
                && min > 0
            {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_bundle_for_view_cache_reuse_stable_min(
                    bundle_path.as_path(),
                    out_dir,
                    min,
                    warmup_frames,
                )?;
            }
            if let Some(min) = check_view_cache_reuse_min
                && min > 0
            {
                check_bundle_for_view_cache_reuse_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_overlay_synthesis_min
                && min > 0
            {
                check_bundle_for_overlay_synthesis_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_viewport_input_min
                && min > 0
            {
                check_bundle_for_viewport_input_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_dock_drag_min
                && min > 0
            {
                check_bundle_for_dock_drag_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_viewport_capture_min
                && min > 0
            {
                check_bundle_for_viewport_capture_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_retained_vlist_reconcile_no_notify_min
                && min > 0
            {
                check_bundle_for_retained_vlist_reconcile_no_notify_min(
                    bundle_path.as_path(),
                    min,
                    warmup_frames,
                )?;
            }
            if let Some(max_delta) = check_retained_vlist_attach_detach_max {
                check_bundle_for_retained_vlist_attach_detach_max(
                    bundle_path.as_path(),
                    max_delta,
                    warmup_frames,
                )?;
            }
            Ok(())
        }
        "matrix" => {
            let Some(target) = rest.first().cloned() else {
                return Err(
                    "missing matrix target (try: fretboard diag matrix ui-gallery)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }
            if target != "ui-gallery" {
                return Err(format!("unknown matrix target: {target}"));
            }

            let Some(launch) = &launch else {
                return Err(
                    "diag matrix requires --launch to run uncached/cached variants (for env control)"
                        .to_string(),
                );
            };

            let scripts: Vec<PathBuf> = ui_gallery_suite_scripts()
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect();

            let compare_opts = CompareOptions {
                warmup_frames,
                eps_px: compare_eps_px,
                ignore_bounds: compare_ignore_bounds,
                ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
            };

            // In matrix mode, treat `--check-view-cache-reuse-min 0` as “disabled”.
            let reuse_gate = match check_view_cache_reuse_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => Some(1),
            };

            // In matrix mode, treat `--check-view-cache-reuse-stable-min 0` as “disabled”.
            let reuse_stable_gate = match check_view_cache_reuse_stable_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => None,
            };

            // In matrix mode, treat `--check-overlay-synthesis-min 0` as “disabled”.
            //
            // Default behavior:
            //
            // - If the caller enables shell reuse (`FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`), also
            //   enable a minimal overlay synthesis gate by default. This helps ensure the
            //   cached-synthesis seam is actually exercised (rather than “view cache enabled but
            //   overlay producers always rerendered”).
            // - Otherwise, leave the gate off by default to avoid forcing overlay-specific
            //   assumptions onto non-overlay scripts (e.g. virtual-list torture).
            let shell_reuse_enabled = launch_env.iter().any(|(k, v)| {
                (k.as_str() == "FRET_UI_GALLERY_VIEW_CACHE_SHELL")
                    && !v.trim().is_empty()
                    && (v.as_str() != "0")
            });
            let overlay_synthesis_gate = match check_overlay_synthesis_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => shell_reuse_enabled.then_some(1),
            };

            // In matrix mode, treat `--check-viewport-input-min 0` as “disabled”.
            let viewport_input_gate = match check_viewport_input_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => None,
            };

            let uncached_out_dir = resolved_out_dir.join("uncached");
            let cached_out_dir = resolved_out_dir.join("cached");

            let uncached_paths =
                ResolvedScriptPaths::for_out_dir(&workspace_root, &uncached_out_dir);
            let cached_paths = ResolvedScriptPaths::for_out_dir(&workspace_root, &cached_out_dir);

            let uncached_env = matrix_launch_env(&launch_env, false)?;
            let cached_env = matrix_launch_env(&launch_env, true)?;

            let uncached_bundles = run_script_suite_collect_bundles(
                &scripts,
                &uncached_paths,
                launch,
                &uncached_env,
                &workspace_root,
                timeout_ms,
                poll_ms,
                warmup_frames,
                None,
                None,
                None,
                None,
                viewport_input_gate,
                viewport_input_gate
                    .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
                None,
                None,
            )?;
            let cached_bundles = run_script_suite_collect_bundles(
                &scripts,
                &cached_paths,
                launch,
                &cached_env,
                &workspace_root,
                timeout_ms,
                poll_ms,
                warmup_frames,
                reuse_stable_gate,
                reuse_gate,
                overlay_synthesis_gate,
                overlay_synthesis_gate.map(|_| {
                    ui_gallery_script_requires_overlay_synthesis_gate as fn(&Path) -> bool
                }),
                viewport_input_gate,
                viewport_input_gate
                    .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
                None,
                None,
            )?;

            let mut ok = true;
            let mut comparisons: Vec<(PathBuf, CompareReport)> = Vec::new();
            for (idx, script) in scripts.iter().enumerate() {
                let a = uncached_bundles.get(idx).cloned().ok_or_else(|| {
                    format!("missing uncached bundle for script: {}", script.display())
                })?;
                let b = cached_bundles.get(idx).cloned().ok_or_else(|| {
                    format!("missing cached bundle for script: {}", script.display())
                })?;
                let report = compare_bundles(&a, &b, compare_opts)?;
                ok &= report.ok;
                comparisons.push((script.clone(), report));
            }

            if stats_json {
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "ok": ok,
                    "out_dir_uncached": uncached_paths.out_dir.display().to_string(),
                    "out_dir_cached": cached_paths.out_dir.display().to_string(),
                    "options": {
                        "warmup_frames": compare_opts.warmup_frames,
                        "eps_px": compare_opts.eps_px,
                        "ignore_bounds": compare_opts.ignore_bounds,
                        "ignore_scene_fingerprint": compare_opts.ignore_scene_fingerprint,
                        "check_view_cache_reuse_min": reuse_gate,
                        "check_view_cache_reuse_stable_min": reuse_stable_gate,
                        "check_overlay_synthesis_min": overlay_synthesis_gate,
                        "check_viewport_input_min": viewport_input_gate,
                    },
                    "comparisons": comparisons.iter().map(|(script, report)| serde_json::json!({
                        "script": script.display().to_string(),
                        "report": report.to_json(),
                    })).collect::<Vec<_>>(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                if !ok {
                    std::process::exit(1);
                }
                Ok(())
            } else if ok {
                println!(
                    "matrix: ok (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
                    scripts.len(),
                    warmup_frames,
                    reuse_gate,
                    reuse_stable_gate,
                    overlay_synthesis_gate,
                    viewport_input_gate
                );
                Ok(())
            } else {
                println!(
                    "matrix: failed (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
                    scripts.len(),
                    warmup_frames,
                    reuse_gate,
                    reuse_stable_gate,
                    overlay_synthesis_gate,
                    viewport_input_gate
                );
                for (script, report) in comparisons {
                    if report.ok {
                        continue;
                    }
                    println!("\nscript: {}", script.display());
                    report.print_human();
                }
                Err("matrix compare failed".to_string())
            }
        }
        "compare" => {
            let Some(a_src) = rest.first().cloned() else {
                return Err(
                    "missing bundle A path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)".to_string(),
                );
            };
            let Some(b_src) = rest.get(1).cloned() else {
                return Err(
                    "missing bundle B path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)".to_string(),
                );
            };
            if rest.len() != 2 {
                return Err(format!("unexpected arguments: {}", rest[2..].join(" ")));
            }

            let a_src = resolve_path(&workspace_root, PathBuf::from(a_src));
            let b_src = resolve_path(&workspace_root, PathBuf::from(b_src));
            let a_bundle_path = resolve_bundle_json_path(&a_src);
            let b_bundle_path = resolve_bundle_json_path(&b_src);

            let report = compare_bundles(
                &a_bundle_path,
                &b_bundle_path,
                CompareOptions {
                    warmup_frames,
                    eps_px: compare_eps_px,
                    ignore_bounds: compare_ignore_bounds,
                    ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
                },
            )?;

            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report.to_json())
                        .unwrap_or_else(|_| "{}".to_string())
                );
                if !report.ok {
                    std::process::exit(1);
                }
                Ok(())
            } else if report.ok {
                report.print_human();
                Ok(())
            } else {
                Err(report.to_human_error())
            }
        }
        "inspect" => {
            let Some(action) = rest.first().cloned() else {
                return Err(
                    "missing inspect action (try: fretboard diag inspect on|off|toggle|status)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            match action.as_str() {
                "status" => {
                    let cfg = read_inspect_config(&resolved_inspect_path);
                    let (enabled, consume_clicks) = match cfg {
                        Some(c) => (c.enabled, c.consume_clicks),
                        None => (false, true),
                    };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "enabled": enabled,
                        "consume_clicks": consume_clicks,
                        "inspect_path": resolved_inspect_path.display().to_string(),
                        "inspect_trigger_path": resolved_inspect_trigger_path.display().to_string(),
                    });
                    println!(
                        "{}",
                        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
                    );
                    Ok(())
                }
                "on" | "off" | "toggle" => {
                    let prev = read_inspect_config(&resolved_inspect_path);
                    let prev_enabled = prev.as_ref().map(|c| c.enabled).unwrap_or(false);
                    let prev_consume_clicks =
                        prev.as_ref().map(|c| c.consume_clicks).unwrap_or(true);

                    let next_enabled = match action.as_str() {
                        "on" => true,
                        "off" => false,
                        "toggle" => !prev_enabled,
                        _ => unreachable!(),
                    };
                    let next_consume_clicks = inspect_consume_clicks.unwrap_or(prev_consume_clicks);

                    write_inspect_config(
                        &resolved_inspect_path,
                        InspectConfigV1 {
                            schema_version: 1,
                            enabled: next_enabled,
                            consume_clicks: next_consume_clicks,
                        },
                    )?;
                    touch(&resolved_inspect_trigger_path)?;
                    println!("{}", resolved_inspect_trigger_path.display());
                    Ok(())
                }
                other => Err(format!("unknown inspect action: {other}")),
            }
        }
        "pick-arm" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            touch(&resolved_pick_trigger_path)?;
            println!("{}", resolved_pick_trigger_path.display());
            Ok(())
        }
        "pick" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;
            report_pick_result_and_exit(&result);
        }
        "pick-script" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            let Some(selector) = result.selector.clone() else {
                return Err("pick succeeded but no selector was returned".to_string());
            };

            write_pick_script(&selector, &resolved_pick_script_out)?;
            println!("{}", resolved_pick_script_out.display());
            Ok(())
        }
        "pick-apply" => {
            let Some(script) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag pick-apply ./script.json --ptr /steps/0/target)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }
            let Some(ptr) = pick_apply_pointer.as_deref() else {
                return Err("missing --ptr (example: --ptr /steps/0/target)".to_string());
            };

            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            let Some(selector) = result.selector.clone() else {
                return Err("pick succeeded but no selector was returned".to_string());
            };

            let script_path = resolve_path(&workspace_root, PathBuf::from(script));
            let out_path = pick_apply_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| script_path.clone());

            apply_pick_to_script(&script_path, &out_path, ptr, selector)?;
            println!("{}", out_path.display());
            Ok(())
        }
        other => Err(format!("unknown diag subcommand: {other}")),
    }
}

fn resolve_bundle_root_dir(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        return Ok(path.to_path_buf());
    }
    let Some(parent) = path.parent() else {
        return Err(format!("invalid bundle path: {}", path.display()));
    };
    Ok(parent.to_path_buf())
}

fn default_pack_out_path(out_dir: &Path, bundle_dir: &Path) -> PathBuf {
    let name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    if bundle_dir.starts_with(out_dir) {
        out_dir.join("share").join(format!("{name}.zip"))
    } else {
        bundle_dir.with_extension("zip")
    }
}

fn default_triage_out_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("triage.json")
}

fn pack_bundle_dir_to_zip(
    bundle_dir: &Path,
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    if !bundle_dir.is_dir() {
        return Err(format!(
            "bundle_dir is not a directory: {}",
            bundle_dir.display()
        ));
    }

    let bundle_json = bundle_dir.join("bundle.json");
    if !bundle_json.is_file() {
        return Err(format!(
            "bundle_dir does not contain bundle.json: {}",
            bundle_dir.display()
        ));
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let bundle_name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip_add_dir(
        &mut zip,
        bundle_dir,
        bundle_dir,
        bundle_name,
        out_path,
        options,
    )?;

    // Repro workflow helper: if a repro summary exists next to the bundle output root, include it.
    let repro_summary = artifacts_root.join("repro.summary.json");
    if repro_summary.is_file() {
        let dst = format!("{bundle_name}/_root/repro.summary.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&repro_summary).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        let root_prefix = format!("{bundle_name}/_root");
        zip_add_root_artifacts(&mut zip, artifacts_root, &root_prefix, options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            let renderdoc_prefix = format!("{bundle_name}/_root/renderdoc");
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                &renderdoc_prefix,
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            let tracy_prefix = format!("{bundle_name}/_root/tracy");
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                &tracy_prefix,
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    if include_screenshots {
        let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
        if screenshots_dir.is_dir() {
            let screenshots_prefix = format!("{bundle_name}/_root/screenshots");
            zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
        }
    }

    if include_triage {
        use std::io::Write;

        let report = bundle_stats_from_path(
            &bundle_json,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        let payload = triage_json_from_stats(&bundle_json, &report, sort, warmup_frames);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        let dst = format!("{bundle_name}/_root/triage.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn triage_json_from_stats(
    bundle_path: &Path,
    report: &BundleStatsReport,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> serde_json::Value {
    use serde_json::json;

    let generated_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64);

    let file_size_bytes = std::fs::metadata(bundle_path).ok().map(|m| m.len());

    let worst = report.top.first().map(|row| {
        json!({
            "window": row.window,
            "tick_id": row.tick_id,
            "frame_id": row.frame_id,
            "timestamp_unix_ms": row.timestamp_unix_ms,
            "total_time_us": row.total_time_us,
            "layout_time_us": row.layout_time_us,
            "prepaint_time_us": row.prepaint_time_us,
            "paint_time_us": row.paint_time_us,
            "invalidation_walk_calls": row.invalidation_walk_calls,
            "invalidation_walk_nodes": row.invalidation_walk_nodes,
            "cache_roots": row.cache_roots,
            "cache_roots_reused": row.cache_roots_reused,
            "cache_replayed_ops": row.cache_replayed_ops,
            "top_invalidation_walks": row.top_invalidation_walks.iter().take(10).map(|w| {
                json!({
                    "root_node": w.root_node,
                    "root_element": w.root_element,
                    "walked_nodes": w.walked_nodes,
                    "kind": w.kind,
                    "source": w.source,
                    "detail": w.detail,
                    "truncated_at": w.truncated_at,
                    "root_role": w.root_role,
                    "root_test_id": w.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_cache_roots": row.top_cache_roots.iter().take(10).map(|r| {
                json!({
                    "root_node": r.root_node,
                    "element": r.element,
                    "reused": r.reused,
                    "contained_layout": r.contained_layout,
                    "paint_replayed_ops": r.paint_replayed_ops,
                    "reuse_reason": r.reuse_reason,
                    "root_role": r.root_role,
                    "root_test_id": r.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_layout_engine_solves": row.top_layout_engine_solves.iter().take(4).map(|s| {
                json!({
                    "root_node": s.root_node,
                    "solve_time_us": s.solve_time_us,
                    "measure_calls": s.measure_calls,
                    "measure_cache_hits": s.measure_cache_hits,
                    "measure_time_us": s.measure_time_us,
                    "root_role": s.root_role,
                    "root_test_id": s.root_test_id,
                    "top_measures": s.top_measures.iter().take(10).map(|m| {
                        json!({
                            "node": m.node,
                            "measure_time_us": m.measure_time_us,
                            "calls": m.calls,
                            "cache_hits": m.cache_hits,
                            "element": m.element,
                            "element_kind": m.element_kind,
                            "role": m.role,
                            "test_id": m.test_id,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        })
    });

    json!({
        "schema_version": 1,
        "generated_unix_ms": generated_unix_ms,
        "bundle": {
            "bundle_path": bundle_path.display().to_string(),
            "bundle_dir": bundle_path.parent().map(|p| p.display().to_string()),
            "bundle_file_size_bytes": file_size_bytes,
        },
        "params": {
            "sort": sort.as_str(),
            "top": report.top.len(),
            "warmup_frames": warmup_frames,
        },
        "stats": report.to_json(),
        "worst": worst,
    })
}

fn zip_add_root_artifacts(
    zip: &mut zip::ZipWriter<std::fs::File>,
    artifacts_root: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let candidates = [
        "evidence.index.json",
        "script.json",
        "script.result.json",
        "pick.result.json",
        "screenshots.result.json",
        "triage.json",
        "picked.script.json",
        "check.semantics_changed_repainted.json",
        "check.pixels_changed.json",
        "check.idle_no_paint.json",
        "check.perf_thresholds.json",
        "check.view_cache_reuse_stable.json",
        "resource.footprint.json",
        "renderdoc.captures.json",
        "tracy.note.md",
    ];

    for name in candidates {
        let src = artifacts_root.join(name);
        if !src.is_file() {
            continue;
        }
        let dst = format!("{zip_prefix}/{name}");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn metadata_mtime_unix_ms(meta: &std::fs::Metadata) -> Option<u64> {
    let modified = meta.modified().ok()?;
    let dur = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()?;
    Some(dur.as_millis().min(u64::MAX as u128) as u64)
}

fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn json_file_summary(path: &Path) -> Option<serde_json::Value> {
    let v = read_json_value(path)?;
    let schema_version = v.get("schema_version").and_then(|v| v.as_u64());
    let kind = v
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let failures_len = v
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64);
    let ok = failures_len.map(|n| n == 0);

    Some(serde_json::json!({
        "schema_version": schema_version,
        "kind": kind,
        "ok": ok,
        "failures_len": failures_len,
    }))
}

fn resource_footprint_summary(path: &Path) -> Option<serde_json::Value> {
    let v = read_json_value(path)?;
    let pid = v.get("pid").and_then(|v| v.as_u64());
    let wall_time_ms = v.get("wall_time_ms").and_then(|v| v.as_u64());
    let killed = v.get("killed").and_then(|v| v.as_bool());
    let note = v
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cpu_avg_pct_total_cores = v
        .get("cpu")
        .and_then(|v| v.get("avg_cpu_percent_total_cores"))
        .and_then(|v| v.as_f64());
    let cpu_usage_pct_avg = v
        .get("cpu")
        .and_then(|v| v.get("usage_percent_avg"))
        .and_then(|v| v.as_f64());

    let working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("working_set_bytes"))
        .and_then(|v| v.as_u64());
    let peak_working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("peak_working_set_bytes"))
        .and_then(|v| v.as_u64());

    Some(serde_json::json!({
        "pid": pid,
        "wall_time_ms": wall_time_ms,
        "killed": killed,
        "note": note,
        "cpu_avg_percent_total_cores": cpu_avg_pct_total_cores,
        "cpu_usage_percent_avg": cpu_usage_pct_avg,
        "working_set_bytes": working_set_bytes,
        "peak_working_set_bytes": peak_working_set_bytes,
    }))
}

fn write_evidence_index(
    artifacts_root: &Path,
    summary_path: &Path,
    summary_json: Option<&serde_json::Value>,
) -> Result<PathBuf, String> {
    let out_path = artifacts_root.join("evidence.index.json");

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut checks: Vec<serde_json::Value> = Vec::new();

    let mut add_file = |name: &str, rel: &str| {
        let path = artifacts_root.join(rel);
        let meta = std::fs::metadata(&path).ok();
        let exists = meta.is_some();
        let size_bytes = meta.as_ref().map(|m| m.len());
        let mtime_unix_ms = meta.as_ref().and_then(metadata_mtime_unix_ms);
        let json = if exists && rel.ends_with(".json") {
            json_file_summary(&path)
        } else {
            None
        };

        entries.push(serde_json::json!({
            "name": name,
            "rel_path": rel,
            "kind": "file",
            "exists": exists,
            "size_bytes": size_bytes,
            "mtime_unix_ms": mtime_unix_ms,
            "json": json,
        }));
    };

    add_file("repro.summary", "repro.summary.json");
    add_file("repro.zip", "repro.zip");
    add_file("resource.footprint", "resource.footprint.json");
    add_file("renderdoc.captures", "renderdoc.captures.json");
    add_file("tracy.note", "tracy.note.md");
    add_file("script", "script.json");
    add_file("script.result", "script.result.json");
    add_file("pick.result", "pick.result.json");
    add_file("screenshots.result", "screenshots.result.json");
    add_file(
        "check.semantics_changed_repainted",
        "check.semantics_changed_repainted.json",
    );
    add_file("check.idle_no_paint", "check.idle_no_paint.json");
    add_file("check.pixels_changed", "check.pixels_changed.json");
    add_file("check.perf_thresholds", "check.perf_thresholds.json");
    add_file(
        "check.view_cache_reuse_stable",
        "check.view_cache_reuse_stable.json",
    );

    for e in entries.iter() {
        let Some(rel) = e.get("rel_path").and_then(|v| v.as_str()) else {
            continue;
        };
        if !rel.starts_with("check.") || !rel.ends_with(".json") {
            continue;
        }
        if e.get("exists").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }
        let name = e.get("name").cloned().unwrap_or(serde_json::Value::Null);
        let json = e.get("json").cloned().unwrap_or(serde_json::Value::Null);
        let ok = json.get("ok").cloned().unwrap_or(serde_json::Value::Null);
        checks.push(serde_json::json!({
            "name": name,
            "file": rel,
            "ok": ok,
            "summary": json,
        }));
    }

    let footprint = artifacts_root.join("resource.footprint.json");
    let resources = serde_json::json!({
        "process_footprint": if footprint.is_file() {
            resource_footprint_summary(&footprint)
        } else {
            None
        },
    });

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "out_dir": artifacts_root.display().to_string(),
        "summary_file": summary_path.file_name().and_then(|s| s.to_str()).unwrap_or("repro.summary.json"),
        "summary": summary_json.cloned(),
        "entries": entries,
        "checks": checks,
        "resources": resources,
    });

    let _ = write_json_value(&out_path, &payload);
    Ok(out_path)
}

#[derive(Debug, Clone)]
struct ReproZipBundle {
    prefix: String,
    bundle_json: PathBuf,
    source_script: PathBuf,
}

fn repro_zip_prefix_for_script(item: &ReproPackItem, idx: usize) -> String {
    let stem = item
        .script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("script");
    let safe = zip_safe_component(stem);
    format!("{:02}-{safe}", idx.saturating_add(1))
}

fn zip_safe_component(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.');
        if keep {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "bundle".to_string()
    } else {
        trimmed.to_string()
    }
}

fn pack_repro_zip_multi(
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    summary_path: &Path,
    bundles: &[ReproZipBundle],
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    use std::io::Write;

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Always include a machine-readable repro summary.
    if summary_path.is_file() {
        let bytes = std::fs::read(summary_path).map_err(|e| e.to_string())?;
        zip.start_file("_root/repro.summary.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Include script sources for offline triage.
    for (idx, item) in bundles.iter().enumerate() {
        let bytes = std::fs::read(&item.source_script).map_err(|e| e.to_string())?;
        let name = item
            .source_script
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("script.json");
        let safe = zip_safe_component(name);
        let dst = format!("_root/scripts/{:02}-{safe}", idx.saturating_add(1));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        zip_add_root_artifacts(&mut zip, artifacts_root, "_root", options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                "_root/renderdoc",
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                "_root/tracy",
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    for item in bundles {
        let bundle_dir = resolve_bundle_root_dir(&item.bundle_json)?;
        zip_add_dir(
            &mut zip,
            &bundle_dir,
            &bundle_dir,
            &item.prefix,
            out_path,
            options,
        )?;

        if include_screenshots {
            let bundle_name = bundle_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
            if screenshots_dir.is_dir() {
                let screenshots_prefix = format!("{}/_root/screenshots", item.prefix);
                zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
            }
        }

        if include_triage {
            let report = bundle_stats_from_path(
                &item.bundle_json,
                stats_top,
                sort,
                BundleStatsOptions { warmup_frames },
            )?;
            let payload = triage_json_from_stats(&item.bundle_json, &report, sort, warmup_frames);
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            let dst = format!("{}/_root/triage.json", item.prefix);
            zip.start_file(dst, options).map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn zip_add_screenshots(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    zip_add_screenshot_dir(zip, dir, dir, zip_prefix, options)
}

fn zip_add_screenshot_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_screenshot_dir(zip, &path, base_dir, zip_prefix, options)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        // Keep this conservative to avoid exploding zip sizes accidentally.
        let should_include = matches!(ext.as_str(), "png") || name == "manifest.json";
        if !should_include {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir_filtered(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
    allowed_exts: &[&str],
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir_filtered(zip, &path, base_dir, zip_prefix, options, allowed_exts)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ext.is_empty() {
            continue;
        }
        if !allowed_exts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(ext.as_str()))
        {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    prefix: &str,
    out_path: &Path,
    options: FileOptions,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path == out_path {
            continue;
        }

        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir(zip, &path, base_dir, prefix, out_path, options)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let name = format!("{}/{}", prefix, zip_name(rel));
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_name(path: &Path) -> String {
    let mut out = String::new();
    for (i, c) in path.components().enumerate() {
        if i > 0 {
            out.push('/');
        }
        out.push_str(&c.as_os_str().to_string_lossy());
    }
    out
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    match s {
        "1" | "true" | "True" | "TRUE" => Ok(true),
        "0" | "false" | "False" | "FALSE" => Ok(false),
        _ => Err(()),
    }
}

#[derive(Debug, Clone)]
struct InspectConfigV1 {
    schema_version: u32,
    enabled: bool,
    consume_clicks: bool,
}

fn read_inspect_config(path: &Path) -> Option<InspectConfigV1> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("schema_version")?.as_u64()? != 1 {
        return None;
    }
    let enabled = v.get("enabled")?.as_bool()?;
    let consume_clicks = v
        .get("consume_clicks")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Some(InspectConfigV1 {
        schema_version: 1,
        enabled,
        consume_clicks,
    })
}

fn write_inspect_config(path: &Path, cfg: InspectConfigV1) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let v = serde_json::json!({
        "schema_version": cfg.schema_version,
        "enabled": cfg.enabled,
        "consume_clicks": cfg.consume_clicks,
    });
    let bytes = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

fn resolve_path(workspace_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn resolve_bundle_json_path(path: &Path) -> PathBuf {
    if !path.is_dir() {
        return path.to_path_buf();
    }

    let direct = path.join("bundle.json");
    if direct.is_file() {
        return direct;
    }

    if let Some(dir) = read_latest_pointer(path).or_else(|| find_latest_export_dir(path)) {
        let nested = dir.join("bundle.json");
        if nested.is_file() {
            return nested;
        }
    }

    direct
}

fn wait_for_bundle_json_from_script_result(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    while Instant::now() < deadline {
        let dir = result
            .last_bundle_dir
            .as_deref()
            .and_then(|s| (!s.trim().is_empty()).then_some(s.trim()))
            .map(PathBuf::from)
            .map(|p| if p.is_absolute() { p } else { out_dir.join(p) })
            .or_else(|| read_latest_pointer(out_dir))
            .or_else(|| find_latest_export_dir(out_dir));
        if let Some(dir) = dir {
            let bundle_path = resolve_bundle_json_path(&dir);
            if bundle_path.is_file() {
                return Some(bundle_path);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn ui_gallery_suite_scripts() -> [&'static str; 13] {
    [
        "tools/diag-scripts/ui-gallery-overlay-torture.json",
        "tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json",
        "tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json",
        "tools/diag-scripts/ui-gallery-portal-geometry-scroll-clamp.json",
        "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
        "tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json",
        "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
        "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
        "tools/diag-scripts/ui-gallery-hover-layout-torture.json",
        "tools/diag-scripts/ui-gallery-table-smoke.json",
        "tools/diag-scripts/ui-gallery-data-table-smoke.json",
        "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
    ]
}

fn docking_arbitration_suite_scripts() -> [&'static str; 2] {
    [
        "tools/diag-scripts/docking-arbitration-demo-split-viewports.json",
        "tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json",
    ]
}

fn docking_arbitration_script_default_gates(
    script: &Path,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return (None, None, None);
    };

    match name {
        "docking-arbitration-demo-split-viewports.json" => (Some(1), None, None),
        "docking-arbitration-demo-modal-dock-drag-viewport-capture.json" => {
            (Some(1), Some(1), Some(1))
        }
        _ => (None, None, None),
    }
}

fn ui_gallery_script_requires_retained_vlist_reconcile_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-virtual-list-window-boundary-scroll-retained.json"
    )
}

fn ui_gallery_script_requires_overlay_synthesis_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // These scripts are expected to exercise the cached overlay synthesis seam when view-cache
    // shell reuse is enabled.
    matches!(
        name,
        "ui-gallery-overlay-torture.json"
            | "ui-gallery-modal-barrier-underlay-block.json"
            | "ui-gallery-popover-dialog-escape-underlay.json"
            | "ui-gallery-portal-geometry-scroll-clamp.json"
            | "ui-gallery-dropdown-open-select.json"
            | "ui-gallery-dropdown-submenu-underlay-dismiss.json"
            | "ui-gallery-context-menu-right-click.json"
            | "ui-gallery-dialog-escape-focus-restore.json"
            | "ui-gallery-menubar-keyboard-nav.json"
    )
}

fn ui_gallery_script_requires_viewport_input_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // Viewport input forwarding is only expected in scripts that explicitly exercise viewport
    // panels / docking viewport tooling scenarios.
    name.contains("viewport") || name.contains("dock")
}

#[derive(Debug, Clone)]
struct ResolvedScriptPaths {
    out_dir: PathBuf,
    ready_path: PathBuf,
    exit_path: PathBuf,
    script_path: PathBuf,
    script_trigger_path: PathBuf,
    script_result_path: PathBuf,
    script_result_trigger_path: PathBuf,
}

impl ResolvedScriptPaths {
    fn for_out_dir(workspace_root: &Path, out_dir: &Path) -> Self {
        let out_dir = resolve_path(workspace_root, out_dir.to_path_buf());
        Self {
            ready_path: resolve_path(workspace_root, out_dir.join("ready.touch")),
            exit_path: resolve_path(workspace_root, out_dir.join("exit.touch")),
            script_path: resolve_path(workspace_root, out_dir.join("script.json")),
            script_trigger_path: resolve_path(workspace_root, out_dir.join("script.touch")),
            script_result_path: resolve_path(workspace_root, out_dir.join("script.result.json")),
            script_result_trigger_path: resolve_path(
                workspace_root,
                out_dir.join("script.result.touch"),
            ),
            out_dir,
        }
    }
}

fn matrix_launch_env(
    base: &[(String, String)],
    view_cache_enabled: bool,
) -> Result<Vec<(String, String)>, String> {
    if base
        .iter()
        .any(|(k, _)| k.as_str() == "FRET_UI_GALLERY_VIEW_CACHE")
    {
        return Err(
            "--env cannot override reserved var for diag matrix: FRET_UI_GALLERY_VIEW_CACHE"
                .to_string(),
        );
    }
    let mut env = base.to_vec();
    env.push((
        "FRET_UI_GALLERY_VIEW_CACHE".to_string(),
        if view_cache_enabled { "1" } else { "0" }.to_string(),
    ));
    Ok(env)
}

fn run_script_suite_collect_bundles(
    scripts: &[PathBuf],
    paths: &ResolvedScriptPaths,
    launch: &[String],
    launch_env: &[(String, String)],
    workspace_root: &Path,
    timeout_ms: u64,
    poll_ms: u64,
    warmup_frames: u64,
    check_view_cache_reuse_stable_min: Option<u64>,
    check_view_cache_reuse_min: Option<u64>,
    check_overlay_synthesis_min: Option<u64>,
    overlay_synthesis_gate_predicate: Option<fn(&Path) -> bool>,
    check_viewport_input_min: Option<u64>,
    viewport_input_gate_predicate: Option<fn(&Path) -> bool>,
    check_dock_drag_min: Option<u64>,
    check_viewport_capture_min: Option<u64>,
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(&paths.out_dir).map_err(|e| e.to_string())?;

    let launch = Some(launch.to_vec());
    let mut child = maybe_launch_demo(
        &launch,
        launch_env,
        workspace_root,
        &paths.out_dir,
        &paths.ready_path,
        &paths.exit_path,
        false,
        timeout_ms,
        poll_ms,
    )?;

    let mut bundle_paths: Vec<PathBuf> = Vec::new();
    for src in scripts {
        let mut result = run_script_and_wait(
            src,
            &paths.script_path,
            &paths.script_trigger_path,
            &paths.script_result_path,
            &paths.script_result_trigger_path,
            timeout_ms,
            poll_ms,
        );
        if let Ok(summary) = &result
            && summary.stage.as_deref() == Some("failed")
        {
            if let Some(dir) =
                wait_for_failure_dump_bundle(&paths.out_dir, summary, timeout_ms, poll_ms)
            {
                if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                    if let Ok(summary) = result.as_mut() {
                        summary.last_bundle_dir = Some(name.to_string());
                    }
                }
            }
        }
        let result = result?;
        if result.stage.as_deref() != Some("passed") {
            let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
            return Err(format!(
                "unexpected script stage for {}: {:?}",
                src.display(),
                result.stage
            ));
        }

        let bundle_path =
            wait_for_bundle_json_from_script_result(&paths.out_dir, &result, timeout_ms, poll_ms)
                .ok_or_else(|| {
                format!(
                    "script passed but no bundle.json was found (required for matrix): {}",
                    src.display()
                )
            })?;

        if let Some(min) = check_view_cache_reuse_stable_min
            && min > 0
        {
            check_bundle_for_view_cache_reuse_stable_min(
                &bundle_path,
                &paths.out_dir,
                min,
                warmup_frames,
            )?;
        }
        if let Some(min) = check_view_cache_reuse_min
            && min > 0
        {
            check_bundle_for_view_cache_reuse_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_overlay_synthesis_min
            && min > 0
        {
            let should_gate = overlay_synthesis_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                check_bundle_for_overlay_synthesis_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_viewport_input_min
            && min > 0
        {
            let should_gate = viewport_input_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                check_bundle_for_viewport_input_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_dock_drag_min
            && min > 0
        {
            check_bundle_for_dock_drag_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_viewport_capture_min
            && min > 0
        {
            check_bundle_for_viewport_capture_min(&bundle_path, min, warmup_frames)?;
        }

        bundle_paths.push(bundle_path);
    }

    let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
    Ok(bundle_paths)
}

fn apply_post_run_checks(
    bundle_path: &Path,
    out_dir: &Path,
    check_idle_no_paint_min: Option<u64>,
    check_stale_paint_test_id: Option<&str>,
    check_stale_paint_eps: f32,
    check_stale_scene_test_id: Option<&str>,
    check_stale_scene_eps: f32,
    check_pixels_changed_test_id: Option<&str>,
    check_semantics_changed_repainted: bool,
    dump_semantics_changed_repainted_json: bool,
    check_wheel_scroll_test_id: Option<&str>,
    check_drag_cache_root_paint_only_test_id: Option<&str>,
    check_hover_layout_max: Option<u32>,
    check_gc_sweep_liveness: bool,
    check_view_cache_reuse_stable_min: Option<u64>,
    check_view_cache_reuse_min: Option<u64>,
    check_overlay_synthesis_min: Option<u64>,
    check_viewport_input_min: Option<u64>,
    check_dock_drag_min: Option<u64>,
    check_viewport_capture_min: Option<u64>,
    check_retained_vlist_reconcile_no_notify_min: Option<u64>,
    check_retained_vlist_attach_detach_max: Option<u64>,
    warmup_frames: u64,
) -> Result<(), String> {
    if let Some(test_id) = check_stale_paint_test_id {
        check_bundle_for_stale_paint(bundle_path, test_id, check_stale_paint_eps)?;
    }
    if let Some(test_id) = check_stale_scene_test_id {
        check_bundle_for_stale_scene(bundle_path, test_id, check_stale_scene_eps)?;
    }
    if let Some(min) = check_idle_no_paint_min {
        check_bundle_for_idle_no_paint_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(test_id) = check_pixels_changed_test_id {
        check_out_dir_for_pixels_changed(out_dir, test_id, warmup_frames)?;
    }
    if check_semantics_changed_repainted {
        check_bundle_for_semantics_changed_repainted(
            bundle_path,
            warmup_frames,
            dump_semantics_changed_repainted_json,
        )?;
    }
    if let Some(test_id) = check_wheel_scroll_test_id {
        check_bundle_for_wheel_scroll(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_drag_cache_root_paint_only_test_id {
        check_bundle_for_drag_cache_root_paint_only(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(max_allowed) = check_hover_layout_max {
        let report = bundle_stats_from_path(
            bundle_path,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions { warmup_frames },
        )?;
        check_report_for_hover_layout_invalidations(&report, max_allowed)?;
    }
    if let Some(min) = check_view_cache_reuse_stable_min
        && min > 0
    {
        check_bundle_for_view_cache_reuse_stable_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(min) = check_view_cache_reuse_min
        && min > 0
    {
        check_bundle_for_view_cache_reuse_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_overlay_synthesis_min
        && min > 0
    {
        check_bundle_for_overlay_synthesis_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_input_min
        && min > 0
    {
        check_bundle_for_viewport_input_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_dock_drag_min
        && min > 0
    {
        check_bundle_for_dock_drag_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_capture_min
        && min > 0
    {
        check_bundle_for_viewport_capture_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_retained_vlist_reconcile_no_notify_min
        && min > 0
    {
        check_bundle_for_retained_vlist_reconcile_no_notify_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(max_delta) = check_retained_vlist_attach_detach_max {
        check_bundle_for_retained_vlist_attach_detach_max(bundle_path, max_delta, warmup_frames)?;
    }
    if check_gc_sweep_liveness {
        check_bundle_for_gc_sweep_liveness(bundle_path, warmup_frames)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct IdleNoPaintWindowReport {
    window: u64,
    examined_snapshots: u64,
    idle_frames_total: u64,
    paint_frames_total: u64,
    idle_streak_max: u64,
    idle_streak_tail: u64,
    last_paint: Option<serde_json::Value>,
}

fn snapshot_is_idle_no_paint(snapshot: &serde_json::Value) -> bool {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let prepaint_time_us = stats
        .and_then(|v| v.get("prepaint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_time_us = stats
        .and_then(|v| v.get("paint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_nodes_performed = stats
        .and_then(|v| v.get("paint_nodes_performed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    prepaint_time_us == 0 && paint_time_us == 0 && paint_nodes_performed == 0
}

fn check_bundle_for_idle_no_paint_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_idle_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut reports: Vec<IdleNoPaintWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut idle_frames_total: u64 = 0;
        let mut paint_frames_total: u64 = 0;
        let mut idle_streak: u64 = 0;
        let mut idle_streak_max: u64 = 0;
        let mut last_paint: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let is_idle = snapshot_is_idle_no_paint(s);
            if is_idle {
                idle_frames_total = idle_frames_total.saturating_add(1);
                idle_streak = idle_streak.saturating_add(1);
                idle_streak_max = idle_streak_max.max(idle_streak);
            } else {
                paint_frames_total = paint_frames_total.saturating_add(1);
                idle_streak = 0;

                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let stats = s.get("debug").and_then(|v| v.get("stats"));
                let prepaint_time_us = stats
                    .and_then(|v| v.get("prepaint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_time_us = stats
                    .and_then(|v| v.get("paint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_nodes_performed = stats
                    .and_then(|v| v.get("paint_nodes_performed"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                last_paint = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "prepaint_time_us": prepaint_time_us,
                    "paint_time_us": paint_time_us,
                    "paint_nodes_performed": paint_nodes_performed,
                }));
            }
        }

        reports.push(IdleNoPaintWindowReport {
            window,
            examined_snapshots,
            idle_frames_total,
            paint_frames_total,
            idle_streak_max,
            idle_streak_tail: idle_streak,
            last_paint: last_paint.clone(),
        });

        let mut fail_reason: Option<&'static str> = None;
        if min_idle_frames > 0 && examined_snapshots < min_idle_frames {
            fail_reason = Some("insufficient_snapshots");
        } else if min_idle_frames > 0 && idle_streak < min_idle_frames {
            fail_reason = Some("idle_tail_streak_too_small");
        }

        if let Some(reason) = fail_reason {
            failures.push(serde_json::json!({
                "window": window,
                "reason": reason,
                "examined_snapshots": examined_snapshots,
                "idle_streak_tail": idle_streak,
                "idle_streak_max": idle_streak_max,
                "idle_frames_total": idle_frames_total,
                "paint_frames_total": paint_frames_total,
                "last_paint": last_paint,
            }));
        }
    }

    let out_path = out_dir.join("check.idle_no_paint.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "idle_no_paint",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_idle_frames": min_idle_frames,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "idle_frames_total": r.idle_frames_total,
            "paint_frames_total": r.paint_frames_total,
            "idle_streak_max": r.idle_streak_max,
            "idle_streak_tail": r.idle_streak_tail,
            "last_paint": r.last_paint,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        return Ok(());
    }

    Err(format!(
        "idle no-paint gate failed (min_idle_frames={min_idle_frames}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

#[derive(Debug, Clone, Copy)]
struct RectF {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Clone, Copy)]
struct RectPx {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
}

impl RectPx {
    fn width_px(self) -> u32 {
        self.x1.saturating_sub(self.x0)
    }
    fn height_px(self) -> u32 {
        self.y1.saturating_sub(self.y0)
    }
}

#[derive(Debug, Clone)]
struct PixelCheckResolvedShot {
    bundle_dir_name: String,
    file: String,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f64,
    rect_px: RectPx,
    hash: u64,
}

fn check_out_dir_for_pixels_changed(
    out_dir: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    use std::collections::HashMap;

    let screenshots_result_path = out_dir.join("screenshots.result.json");
    if !screenshots_result_path.is_file() {
        return Err(format!(
            "pixels changed check requires screenshots results under {} (set FRET_DIAG_SCREENSHOTS=1 and add capture_screenshot steps): {}",
            out_dir.display(),
            screenshots_result_path.display()
        ));
    }

    let bytes = std::fs::read(&screenshots_result_path).map_err(|e| e.to_string())?;
    let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let completed = root
        .get("completed")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid screenshots.result.json: missing completed array".to_string())?;

    let mut bundles_cache: HashMap<String, serde_json::Value> = HashMap::new();
    let mut resolved: Vec<PixelCheckResolvedShot> = Vec::new();

    for entry in completed {
        let bundle_dir_name = entry
            .get("bundle_dir_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if bundle_dir_name.trim().is_empty() {
            continue;
        }

        let window = entry.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let tick_id = entry.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let frame_id = entry.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        if frame_id < warmup_frames {
            continue;
        }

        let scale_factor = entry
            .get("scale_factor")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        if !(scale_factor.is_finite() && scale_factor > 0.0) {
            continue;
        }

        let file = entry
            .get("file")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if file.trim().is_empty() {
            continue;
        }

        let screenshot_path = out_dir
            .join("screenshots")
            .join(&bundle_dir_name)
            .join(&file);
        if !screenshot_path.is_file() {
            continue;
        }

        let bundle_json_path = out_dir.join(&bundle_dir_name).join("bundle.json");
        if !bundle_json_path.is_file() {
            continue;
        }

        let bundle = if let Some(b) = bundles_cache.get(&bundle_dir_name) {
            b.clone()
        } else {
            let bytes = std::fs::read(&bundle_json_path).map_err(|e| e.to_string())?;
            let bundle: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            bundles_cache.insert(bundle_dir_name.clone(), bundle.clone());
            bundle
        };

        let bounds =
            match find_semantics_bounds_for_test_id(&bundle, window, tick_id, frame_id, test_id) {
                Some(r) => r,
                None => continue,
            };

        let img = image::ImageReader::open(&screenshot_path)
            .map_err(|e| {
                format!(
                    "failed to open screenshot png: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .with_guessed_format()
            .map_err(|e| {
                format!(
                    "failed to read screenshot format: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .decode()
            .map_err(|e| {
                format!(
                    "failed to decode screenshot png: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .to_rgba8();

        let (img_w, img_h) = img.dimensions();
        let rect_px = rect_f_to_px(bounds, scale_factor, img_w, img_h);
        if rect_px.width_px() == 0 || rect_px.height_px() == 0 {
            continue;
        }

        let hash = hash_rgba_region(&img, rect_px);
        resolved.push(PixelCheckResolvedShot {
            bundle_dir_name,
            file,
            window,
            tick_id,
            frame_id,
            scale_factor,
            rect_px,
            hash,
        });
    }

    resolved.sort_by(|a, b| {
        a.tick_id
            .cmp(&b.tick_id)
            .then_with(|| a.frame_id.cmp(&b.frame_id))
            .then_with(|| a.file.cmp(&b.file))
    });

    let out_path = out_dir.join("check.pixels_changed.json");

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "pixels_changed",
        "out_dir": out_dir.display().to_string(),
        "test_id": test_id,
        "warmup_frames": warmup_frames,
        "resolved": resolved.iter().map(|s| serde_json::json!({
            "bundle_dir_name": s.bundle_dir_name,
            "file": s.file,
            "window": s.window,
            "tick_id": s.tick_id,
            "frame_id": s.frame_id,
            "scale_factor": s.scale_factor,
            "rect_px": { "x0": s.rect_px.x0, "y0": s.rect_px.y0, "x1": s.rect_px.x1, "y1": s.rect_px.y1 },
            "hash": format!("0x{:016x}", s.hash),
        })).collect::<Vec<_>>(),
    });
    let _ = write_json_value(&out_path, &payload);

    if resolved.len() < 2 {
        return Err(format!(
            "pixels changed check requires at least 2 resolved screenshots for test_id={test_id} (resolved={}, out_dir={})",
            resolved.len(),
            out_dir.display()
        ));
    }

    let first = &resolved[0];
    let last = &resolved[resolved.len() - 1];
    if first.hash != last.hash {
        return Ok(());
    }

    Err(format!(
        "pixels unchanged suspected for test_id={test_id} (hash=0x{hash:016x})\n  first: bundle={b0} file={f0} tick={t0} frame={fr0} rect_px=({x0},{y0})-({x1},{y1})\n  last:  bundle={b1} file={f1} tick={t1} frame={fr1} rect_px=({x2},{y2})-({x3},{y3})\n  evidence: {}",
        out_path.display(),
        hash = first.hash,
        b0 = first.bundle_dir_name,
        f0 = first.file,
        t0 = first.tick_id,
        fr0 = first.frame_id,
        x0 = first.rect_px.x0,
        y0 = first.rect_px.y0,
        x1 = first.rect_px.x1,
        y1 = first.rect_px.y1,
        b1 = last.bundle_dir_name,
        f1 = last.file,
        t1 = last.tick_id,
        fr1 = last.frame_id,
        x2 = last.rect_px.x0,
        y2 = last.rect_px.y0,
        x3 = last.rect_px.x1,
        y3 = last.rect_px.y1,
    ))
}

fn find_semantics_bounds_for_test_id(
    bundle: &serde_json::Value,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    test_id: &str,
) -> Option<RectF> {
    let windows = bundle.get("windows").and_then(|v| v.as_array())?;
    let w = windows
        .iter()
        .find(|w| w.get("window").and_then(|v| v.as_u64()) == Some(window))?;
    let snaps = w.get("snapshots").and_then(|v| v.as_array())?;

    let snap = snaps.iter().find(|s| {
        s.get("tick_id").and_then(|v| v.as_u64()) == Some(tick_id)
            && s.get("frame_id").and_then(|v| v.as_u64()) == Some(frame_id)
    })?;

    let nodes = snap
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;

    let node = nodes
        .iter()
        .find(|n| n.get("test_id").and_then(|v| v.as_str()) == Some(test_id))?;

    let bounds = node.get("bounds")?;
    Some(RectF {
        x: bounds.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
        y: bounds.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
        w: bounds.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0),
        h: bounds.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}

fn rect_f_to_px(bounds: RectF, scale_factor: f64, img_w: u32, img_h: u32) -> RectPx {
    let sx0 = (bounds.x * scale_factor).floor();
    let sy0 = (bounds.y * scale_factor).floor();
    let sx1 = ((bounds.x + bounds.w) * scale_factor).ceil();
    let sy1 = ((bounds.y + bounds.h) * scale_factor).ceil();

    let clamp = |v: f64, max: u32| -> u32 {
        if !v.is_finite() {
            return 0;
        }
        let v = v.max(0.0).min(max as f64);
        v as u32
    };

    let x0 = clamp(sx0, img_w);
    let y0 = clamp(sy0, img_h);
    let x1 = clamp(sx1, img_w);
    let y1 = clamp(sy1, img_h);

    RectPx { x0, y0, x1, y1 }
}

fn hash_rgba_region(img: &image::RgbaImage, rect: RectPx) -> u64 {
    // Stable, tiny hash for CI gates (not cryptographic).
    let mut h: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;

    let (w, _h_px) = img.dimensions();
    let bytes = img.as_raw();

    // Mix dimensions so two different rects are unlikely to collide.
    for b in rect.x0.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.y0.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.x1.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.y1.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }

    let row_bytes = (rect.width_px() as usize).saturating_mul(4);
    for y in rect.y0..rect.y1 {
        let start = (y as usize)
            .saturating_mul(w as usize)
            .saturating_add(rect.x0 as usize)
            .saturating_mul(4);
        let end = start.saturating_add(row_bytes).min(bytes.len());
        for &b in &bytes[start..end] {
            h ^= b as u64;
            h = h.wrapping_mul(prime);
        }
    }

    h
}

#[derive(Debug, Clone, Copy)]
struct CompareOptions {
    warmup_frames: u64,
    eps_px: f32,
    ignore_bounds: bool,
    ignore_scene_fingerprint: bool,
}

#[derive(Debug, Clone)]
struct CompareReport {
    ok: bool,
    a_path: PathBuf,
    b_path: PathBuf,
    a_frame_id: Option<u64>,
    b_frame_id: Option<u64>,
    a_scene_fingerprint: Option<u64>,
    b_scene_fingerprint: Option<u64>,
    opts: CompareOptions,
    diffs: Vec<CompareDiff>,
}

#[derive(Debug, Clone)]
struct CompareDiff {
    kind: &'static str,
    key: Option<String>,
    field: Option<&'static str>,
    a: Option<serde_json::Value>,
    b: Option<serde_json::Value>,
}

impl CompareReport {
    fn print_human(&self) {
        println!("bundle_a: {}", self.a_path.display());
        println!("bundle_b: {}", self.b_path.display());
        if let (Some(a), Some(b)) = (self.a_frame_id, self.b_frame_id) {
            println!("frame_id: a={a} b={b}");
        }
        if let (Some(a), Some(b)) = (self.a_scene_fingerprint, self.b_scene_fingerprint) {
            println!("scene_fingerprint: a=0x{a:016x} b=0x{b:016x}");
        }
        if self.ok {
            println!(
                "compare: ok (diffs=0, warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})",
                self.opts.warmup_frames,
                self.opts.eps_px,
                self.opts.ignore_bounds,
                self.opts.ignore_scene_fingerprint
            );
            return;
        }
        println!(
            "compare: failed (diffs={}, warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})",
            self.diffs.len(),
            self.opts.warmup_frames,
            self.opts.eps_px,
            self.opts.ignore_bounds,
            self.opts.ignore_scene_fingerprint
        );
        for d in self.diffs.iter().take(20) {
            let key = d.key.as_deref().unwrap_or("<none>");
            let field = d.field.unwrap_or("<none>");
            println!("  - {} key={} field={}", d.kind, key, field);
        }
        if self.diffs.len() > 20 {
            println!("  ... ({} more)", self.diffs.len() - 20);
        }
    }

    fn to_human_error(&self) -> String {
        let mut msg = String::new();
        msg.push_str("bundle compare failed\n");
        msg.push_str(&format!("bundle_a: {}\n", self.a_path.display()));
        msg.push_str(&format!("bundle_b: {}\n", self.b_path.display()));
        if let (Some(a), Some(b)) = (self.a_frame_id, self.b_frame_id) {
            msg.push_str(&format!("frame_id: a={a} b={b}\n"));
        }
        if let (Some(a), Some(b)) = (self.a_scene_fingerprint, self.b_scene_fingerprint) {
            msg.push_str(&format!("scene_fingerprint: a=0x{a:016x} b=0x{b:016x}\n"));
        }
        msg.push_str(&format!(
            "diffs: {} (warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})\n",
            self.diffs.len(),
            self.opts.warmup_frames,
            self.opts.eps_px,
            self.opts.ignore_bounds,
            self.opts.ignore_scene_fingerprint
        ));
        for d in self.diffs.iter().take(20) {
            let key = d.key.as_deref().unwrap_or("<none>");
            let field = d.field.unwrap_or("<none>");
            msg.push_str(&format!("  - {} key={} field={}\n", d.kind, key, field));
        }
        if self.diffs.len() > 20 {
            msg.push_str(&format!("  ... ({} more)\n", self.diffs.len() - 20));
        }
        msg
    }

    fn to_json(&self) -> serde_json::Value {
        let diffs = self
            .diffs
            .iter()
            .map(|d| {
                serde_json::json!({
                    "kind": d.kind,
                    "key": d.key,
                    "field": d.field,
                    "a": d.a,
                    "b": d.b,
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version": 1,
            "ok": self.ok,
            "bundle_a": self.a_path.display().to_string(),
            "bundle_b": self.b_path.display().to_string(),
            "a_frame_id": self.a_frame_id,
            "b_frame_id": self.b_frame_id,
            "a_scene_fingerprint": self.a_scene_fingerprint,
            "b_scene_fingerprint": self.b_scene_fingerprint,
            "options": {
                "warmup_frames": self.opts.warmup_frames,
                "eps_px": self.opts.eps_px,
                "ignore_bounds": self.opts.ignore_bounds,
                "ignore_scene_fingerprint": self.opts.ignore_scene_fingerprint,
            },
            "diffs": diffs,
        })
    }
}

fn compare_bundles(
    a_bundle_path: &Path,
    b_bundle_path: &Path,
    opts: CompareOptions,
) -> Result<CompareReport, String> {
    let a_bytes = std::fs::read(a_bundle_path).map_err(|e| e.to_string())?;
    let b_bytes = std::fs::read(b_bundle_path).map_err(|e| e.to_string())?;
    let a_bundle: serde_json::Value =
        serde_json::from_slice(&a_bytes).map_err(|e| e.to_string())?;
    let b_bundle: serde_json::Value =
        serde_json::from_slice(&b_bytes).map_err(|e| e.to_string())?;
    compare_bundles_json(&a_bundle, a_bundle_path, &b_bundle, b_bundle_path, opts)
}

fn compare_bundles_json(
    a_bundle: &serde_json::Value,
    a_bundle_path: &Path,
    b_bundle: &serde_json::Value,
    b_bundle_path: &Path,
    opts: CompareOptions,
) -> Result<CompareReport, String> {
    let a_window = first_window_from_bundle(a_bundle)?;
    let b_window = first_window_from_bundle(b_bundle)?;

    let (a_snapshot, a_selected) = select_snapshot_for_compare(a_window, opts.warmup_frames);
    let (b_snapshot, b_selected) = select_snapshot_for_compare(b_window, opts.warmup_frames);

    let mut diffs: Vec<CompareDiff> = Vec::new();

    let a_fp = a_snapshot
        .and_then(|s| s.get("scene_fingerprint"))
        .and_then(|v| v.as_u64());
    let b_fp = b_snapshot
        .and_then(|s| s.get("scene_fingerprint"))
        .and_then(|v| v.as_u64());
    if !opts.ignore_scene_fingerprint {
        if let (Some(a_fp), Some(b_fp)) = (a_fp, b_fp) {
            if a_fp != b_fp {
                diffs.push(CompareDiff {
                    kind: "scene_fingerprint_mismatch",
                    key: None,
                    field: Some("scene_fingerprint"),
                    a: Some(serde_json::Value::from(a_fp)),
                    b: Some(serde_json::Value::from(b_fp)),
                });
            }
        }
    }

    if let (Some(a_snapshot), Some(b_snapshot)) = (a_snapshot, b_snapshot) {
        compare_semantics_by_test_id(&mut diffs, a_snapshot, b_snapshot, opts)?;
    }

    Ok(CompareReport {
        ok: diffs.is_empty(),
        a_path: a_bundle_path.to_path_buf(),
        b_path: b_bundle_path.to_path_buf(),
        a_frame_id: a_selected.frame_id,
        b_frame_id: b_selected.frame_id,
        a_scene_fingerprint: a_fp,
        b_scene_fingerprint: b_fp,
        opts,
        diffs,
    })
}

fn first_window_from_bundle(bundle: &serde_json::Value) -> Result<&serde_json::Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    windows
        .first()
        .ok_or_else(|| "bundle.json contains no windows".to_string())
}

#[derive(Debug, Clone, Copy, Default)]
struct SelectedSnapshotInfo {
    frame_id: Option<u64>,
}

fn select_snapshot_for_compare<'a>(
    window: &'a serde_json::Value,
    warmup_frames: u64,
) -> (Option<&'a serde_json::Value>, SelectedSnapshotInfo) {
    let snaps = window
        .get("snapshots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if snaps.is_empty() {
        return (None, SelectedSnapshotInfo::default());
    }

    let mut selected: Option<&serde_json::Value> = None;
    for s in snaps {
        let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        if frame_id >= warmup_frames {
            selected = Some(s);
        }
    }
    let selected = selected.or_else(|| snaps.last());
    let info = SelectedSnapshotInfo {
        frame_id: selected.and_then(|s| s.get("frame_id").and_then(|v| v.as_u64())),
    };
    (selected, info)
}

#[derive(Debug, Clone)]
struct SemanticsNodeSummary {
    role: String,
    flags: serde_json::Value,
    actions: serde_json::Value,
    bounds: Option<(f64, f64, f64, f64)>,
}

fn compare_semantics_by_test_id(
    diffs: &mut Vec<CompareDiff>,
    a_snapshot: &serde_json::Value,
    b_snapshot: &serde_json::Value,
    opts: CompareOptions,
) -> Result<(), String> {
    let a_sem = a_snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .ok_or_else(|| {
            "bundle snapshot missing debug.semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
        })?;
    let b_sem = b_snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .ok_or_else(|| {
            "bundle snapshot missing debug.semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
        })?;

    let (a_by_test_id, a_id_to_test_id) = semantics_nodes_by_test_id(a_sem);
    let (b_by_test_id, b_id_to_test_id) = semantics_nodes_by_test_id(b_sem);

    for test_id in a_by_test_id.keys() {
        if !b_by_test_id.contains_key(test_id) {
            diffs.push(CompareDiff {
                kind: "missing_test_id",
                key: Some(test_id.clone()),
                field: None,
                a: Some(serde_json::Value::from("present")),
                b: Some(serde_json::Value::Null),
            });
        }
    }
    for test_id in b_by_test_id.keys() {
        if !a_by_test_id.contains_key(test_id) {
            diffs.push(CompareDiff {
                kind: "extra_test_id",
                key: Some(test_id.clone()),
                field: None,
                a: Some(serde_json::Value::Null),
                b: Some(serde_json::Value::from("present")),
            });
        }
    }

    for (test_id, a_node) in a_by_test_id.iter() {
        let Some(b_node) = b_by_test_id.get(test_id) else {
            continue;
        };
        if a_node.role != b_node.role {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("role"),
                a: Some(serde_json::Value::from(a_node.role.clone())),
                b: Some(serde_json::Value::from(b_node.role.clone())),
            });
        }
        if a_node.flags != b_node.flags {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("flags"),
                a: Some(a_node.flags.clone()),
                b: Some(b_node.flags.clone()),
            });
        }
        if a_node.actions != b_node.actions {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("actions"),
                a: Some(a_node.actions.clone()),
                b: Some(b_node.actions.clone()),
            });
        }
        if !opts.ignore_bounds {
            if let (Some(a), Some(b)) = (a_node.bounds, b_node.bounds) {
                if !rect_eq_eps(a, b, opts.eps_px) {
                    diffs.push(CompareDiff {
                        kind: "node_field_mismatch",
                        key: Some(test_id.clone()),
                        field: Some("bounds"),
                        a: Some(serde_json::json!({ "x": a.0, "y": a.1, "w": a.2, "h": a.3 })),
                        b: Some(serde_json::json!({ "x": b.0, "y": b.1, "w": b.2, "h": b.3 })),
                    });
                }
            }
        }
    }

    compare_semantics_root_distribution(diffs, a_sem, b_sem);
    compare_focus_and_capture_by_test_id(diffs, a_sem, b_sem, &a_id_to_test_id, &b_id_to_test_id);

    Ok(())
}

fn semantics_nodes_by_test_id(
    semantics: &serde_json::Value,
) -> (
    std::collections::BTreeMap<String, SemanticsNodeSummary>,
    std::collections::HashMap<u64, String>,
) {
    let mut by_test_id: std::collections::BTreeMap<String, SemanticsNodeSummary> =
        std::collections::BTreeMap::new();
    let mut id_to_test_id: std::collections::HashMap<u64, String> =
        std::collections::HashMap::new();

    let nodes = semantics
        .get("nodes")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for node in nodes {
        let Some(test_id) = node.get("test_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let test_id = test_id.to_string();
        let role = node
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let flags = node
            .get("flags")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let actions = node
            .get("actions")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let bounds = node.get("bounds").and_then(rect_xywh_from_json);
        by_test_id.insert(
            test_id.clone(),
            SemanticsNodeSummary {
                role,
                flags,
                actions,
                bounds,
            },
        );
        if let Some(id) = node.get("id").and_then(|v| v.as_u64()) {
            id_to_test_id.insert(id, test_id);
        }
    }

    (by_test_id, id_to_test_id)
}

fn rect_xywh_from_json(bounds: &serde_json::Value) -> Option<(f64, f64, f64, f64)> {
    let x = bounds.get("x").and_then(|v| v.as_f64())?;
    let y = bounds.get("y").and_then(|v| v.as_f64())?;
    let w = bounds.get("w").and_then(|v| v.as_f64())?;
    let h = bounds.get("h").and_then(|v| v.as_f64())?;
    Some((x, y, w, h))
}

fn rect_eq_eps(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64), eps_px: f32) -> bool {
    let eps = eps_px as f64;
    (a.0 - b.0).abs() <= eps
        && (a.1 - b.1).abs() <= eps
        && (a.2 - b.2).abs() <= eps
        && (a.3 - b.3).abs() <= eps
}

fn compare_semantics_root_distribution(
    diffs: &mut Vec<CompareDiff>,
    a_sem: &serde_json::Value,
    b_sem: &serde_json::Value,
) {
    let a = semantics_root_distribution(a_sem);
    let b = semantics_root_distribution(b_sem);
    if a != b {
        diffs.push(CompareDiff {
            kind: "semantics_roots_mismatch",
            key: None,
            field: Some("roots"),
            a: Some(semantics_root_distribution_to_json(&a)),
            b: Some(semantics_root_distribution_to_json(&b)),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticsRootSummary {
    visible: bool,
    blocks_underlay_input: bool,
    hit_testable: bool,
    z_index: u32,
}

fn semantics_root_distribution(sem: &serde_json::Value) -> Vec<SemanticsRootSummary> {
    let roots = sem
        .get("roots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    let mut out: Vec<SemanticsRootSummary> = roots
        .iter()
        .map(|r| SemanticsRootSummary {
            visible: r.get("visible").and_then(|v| v.as_bool()).unwrap_or(false),
            blocks_underlay_input: r
                .get("blocks_underlay_input")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            hit_testable: r
                .get("hit_testable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            z_index: r.get("z_index").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        })
        .collect();
    out.sort();
    out
}

fn semantics_root_distribution_to_json(dist: &[SemanticsRootSummary]) -> serde_json::Value {
    serde_json::Value::Array(
        dist.iter()
            .map(|r| {
                serde_json::json!({
                    "visible": r.visible,
                    "blocks_underlay_input": r.blocks_underlay_input,
                    "hit_testable": r.hit_testable,
                    "z_index": r.z_index,
                })
            })
            .collect(),
    )
}

fn compare_focus_and_capture_by_test_id(
    diffs: &mut Vec<CompareDiff>,
    a_sem: &serde_json::Value,
    b_sem: &serde_json::Value,
    a_id_to_test_id: &std::collections::HashMap<u64, String>,
    b_id_to_test_id: &std::collections::HashMap<u64, String>,
) {
    let a_focus = a_sem.get("focus").and_then(|v| v.as_u64());
    let b_focus = b_sem.get("focus").and_then(|v| v.as_u64());
    let a_focus_tid = a_focus.and_then(|id| a_id_to_test_id.get(&id).cloned());
    let b_focus_tid = b_focus.and_then(|id| b_id_to_test_id.get(&id).cloned());
    if a_focus_tid.is_some() || b_focus_tid.is_some() {
        if a_focus_tid != b_focus_tid {
            diffs.push(CompareDiff {
                kind: "focus_mismatch",
                key: None,
                field: Some("focus.test_id"),
                a: a_focus_tid.map(serde_json::Value::from),
                b: b_focus_tid.map(serde_json::Value::from),
            });
        }
    }

    let a_captured = a_sem.get("captured").and_then(|v| v.as_u64());
    let b_captured = b_sem.get("captured").and_then(|v| v.as_u64());
    let a_captured_tid = a_captured.and_then(|id| a_id_to_test_id.get(&id).cloned());
    let b_captured_tid = b_captured.and_then(|id| b_id_to_test_id.get(&id).cloned());
    if a_captured_tid.is_some() || b_captured_tid.is_some() {
        if a_captured_tid != b_captured_tid {
            diffs.push(CompareDiff {
                kind: "captured_mismatch",
                key: None,
                field: Some("captured.test_id"),
                a: a_captured_tid.map(serde_json::Value::from),
                b: b_captured_tid.map(serde_json::Value::from),
            });
        }
    }
}

fn read_latest_pointer(out_dir: &Path) -> Option<PathBuf> {
    let s = std::fs::read_to_string(out_dir.join("latest.txt")).ok()?;
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let path = PathBuf::from(s);
    Some(if path.is_absolute() {
        path
    } else {
        out_dir.join(path)
    })
}

fn find_latest_export_dir(out_dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(ts) = name.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn maybe_launch_demo(
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    workspace_root: &Path,
    out_dir: &Path,
    ready_path: &Path,
    exit_path: &Path,
    wants_screenshots: bool,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Option<LaunchedDemo>, String> {
    let Some(launch) = launch else {
        return Ok(None);
    };

    let prev_ready_mtime = std::fs::metadata(ready_path)
        .and_then(|m| m.modified())
        .ok();

    let exe = launch
        .first()
        .ok_or_else(|| "missing launch command".to_string())?;

    let mut cmd = Command::new(exe);
    cmd.args(launch.iter().skip(1));
    cmd.current_dir(workspace_root);
    cmd.env("FRET_DIAG", "1");
    cmd.env("FRET_DIAG_DIR", out_dir);
    cmd.env("FRET_DIAG_READY_PATH", ready_path);
    cmd.env("FRET_DIAG_EXIT_PATH", exit_path);
    if wants_screenshots {
        cmd.env("FRET_DIAG_SCREENSHOTS", "1");
    }
    for (key, value) in launch_env {
        match key.as_str() {
            "FRET_DIAG" | "FRET_DIAG_DIR" | "FRET_DIAG_READY_PATH" | "FRET_DIAG_EXIT_PATH" => {
                return Err(format!("--env cannot override reserved var: {key}"));
            }
            _ => cmd.env(key, value),
        };
    }
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").filter(|v| !v.is_empty()) {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    let launched_unix_ms = now_unix_ms();
    let launched_instant = Instant::now();
    let child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn `{}`: {e}", launch.join(" ")))?;
    let demo = LaunchedDemo {
        child,
        launched_unix_ms,
        launched_instant,
        launch_cmd: launch.clone(),
    };

    // Avoid racing cold-start compilation by waiting for the app to signal readiness.
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(180_000));
    while Instant::now() < deadline {
        let ready_mtime = std::fs::metadata(ready_path)
            .and_then(|m| m.modified())
            .ok();
        let ready = match (prev_ready_mtime, ready_mtime) {
            (Some(prev), Some(now)) => now > prev,
            (None, Some(_)) => true,
            _ => false,
        };
        if ready {
            return Ok(Some(demo));
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    Ok(Some(demo))
}

#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone)]
struct ObservedProcessFootprint {
    collector: String,
    sample_interval_ms: u64,
    samples: u64,
    cpu_usage_percent_avg: f64,
    cpu_usage_percent_max: f64,
    cpu_usage_percent_last: f64,
    working_set_bytes_last: u64,
    working_set_bytes_peak: u64,
    virtual_memory_bytes_last: u64,
    virtual_memory_bytes_peak: u64,
}

#[cfg(not(windows))]
struct SysinfoProcessFootprintSampler {
    sys: sysinfo::System,
    pid: sysinfo::Pid,
    refresh_kind: sysinfo::ProcessRefreshKind,
    refresh_count: u64,
    last_refresh: std::time::Instant,
    samples: u64,
    cpu_usage_percent_sum: f64,
    cpu_usage_percent_max: f64,
    cpu_usage_percent_last: f64,
    working_set_bytes_last: u64,
    working_set_bytes_peak: u64,
    virtual_memory_bytes_last: u64,
    virtual_memory_bytes_peak: u64,
}

#[cfg(not(windows))]
impl SysinfoProcessFootprintSampler {
    fn new(pid_u32: u32) -> Self {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

        let mut sys = System::new();
        let pid = sysinfo::Pid::from_u32(pid_u32);
        let refresh_kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

        let _ =
            sys.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, refresh_kind);

        let mut out = Self {
            sys,
            pid,
            refresh_kind,
            refresh_count: 1,
            last_refresh: std::time::Instant::now(),
            samples: 0,
            cpu_usage_percent_sum: 0.0,
            cpu_usage_percent_max: 0.0,
            cpu_usage_percent_last: 0.0,
            working_set_bytes_last: 0,
            working_set_bytes_peak: 0,
            virtual_memory_bytes_last: 0,
            virtual_memory_bytes_peak: 0,
        };

        out.capture_latest();
        out
    }

    fn refresh_force(&mut self) {
        use sysinfo::ProcessesToUpdate;

        let _ = self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            false,
            self.refresh_kind,
        );
        self.refresh_count = self.refresh_count.saturating_add(1);
        self.last_refresh = std::time::Instant::now();
        self.capture_latest();
    }

    fn refresh_if_due(&mut self) {
        let since = self.last_refresh.elapsed();
        if since < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            return;
        }
        self.refresh_force();
    }

    fn capture_latest(&mut self) {
        let Some(p) = self.sys.process(self.pid) else {
            return;
        };

        let mem = p.memory();
        let vmem = p.virtual_memory();
        self.working_set_bytes_last = mem;
        self.virtual_memory_bytes_last = vmem;
        self.working_set_bytes_peak = self.working_set_bytes_peak.max(mem);
        self.virtual_memory_bytes_peak = self.virtual_memory_bytes_peak.max(vmem);

        let cpu = p.cpu_usage().max(0.0) as f64;
        self.cpu_usage_percent_last = cpu;
        self.cpu_usage_percent_max = self.cpu_usage_percent_max.max(cpu);

        // sysinfo computes CPU usage based on a diff between two refreshes.
        // The first refresh establishes a baseline and produces a (likely) zero usage.
        if self.refresh_count >= 2 {
            self.samples = self.samples.saturating_add(1);
            self.cpu_usage_percent_sum += cpu;
        }
    }

    fn finish(self) -> ObservedProcessFootprint {
        let sample_interval_ms = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
            .as_millis()
            .min(u64::MAX as u128) as u64;
        let avg = if self.samples > 0 {
            self.cpu_usage_percent_sum / (self.samples as f64)
        } else {
            0.0
        };

        ObservedProcessFootprint {
            collector: "sysinfo".to_string(),
            sample_interval_ms,
            samples: self.samples,
            cpu_usage_percent_avg: avg,
            cpu_usage_percent_max: self.cpu_usage_percent_max,
            cpu_usage_percent_last: self.cpu_usage_percent_last,
            working_set_bytes_last: self.working_set_bytes_last,
            working_set_bytes_peak: self.working_set_bytes_peak,
            virtual_memory_bytes_last: self.virtual_memory_bytes_last,
            virtual_memory_bytes_peak: self.virtual_memory_bytes_peak,
        }
    }
}

fn collect_demo_footprint_json(
    demo: &LaunchedDemo,
    killed: bool,
    observed: Option<ObservedProcessFootprint>,
) -> serde_json::Value {
    #[cfg(windows)]
    let _ = observed;

    let ended_unix_ms = now_unix_ms();
    let ended_instant = Instant::now();
    let wall_time_ms = ended_instant
        .duration_since(demo.launched_instant)
        .as_millis()
        .min(u64::MAX as u128) as u64;
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(1)
        .max(1);

    let mut out = serde_json::json!({
        "schema_version": 1,
        "kind": "process_footprint",
        "pid": demo.child.id(),
        "launch_cmd": demo.launch_cmd,
        "launched_unix_ms": demo.launched_unix_ms,
        "ended_unix_ms": ended_unix_ms,
        "wall_time_ms": wall_time_ms,
        "killed": killed,
        "logical_cores": logical_cores,
    });

    #[cfg(windows)]
    {
        use std::mem::size_of;
        use std::os::windows::io::AsRawHandle as _;
        use windows_sys::Win32::Foundation::FILETIME;
        use windows_sys::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetProcessTimes;

        let handle = demo.child.as_raw_handle();
        unsafe fn filetime_to_100ns(ft: FILETIME) -> u64 {
            ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
        }

        let mut creation: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };

        let times_ok =
            unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) }
                != 0;
        if times_ok {
            let kernel_100ns = unsafe { filetime_to_100ns(kernel) };
            let user_100ns = unsafe { filetime_to_100ns(user) };
            let total_100ns = kernel_100ns.saturating_add(user_100ns);
            let total_time_ms = total_100ns / 10_000;
            let kernel_time_ms = kernel_100ns / 10_000;
            let user_time_ms = user_100ns / 10_000;

            let avg_cpu_cores = if wall_time_ms > 0 {
                (total_time_ms as f64) / (wall_time_ms as f64)
            } else {
                0.0
            };
            let avg_cpu_percent_total = if logical_cores > 0 {
                (avg_cpu_cores / (logical_cores as f64)) * 100.0
            } else {
                0.0
            };

            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "cpu".to_string(),
                    serde_json::json!({
                        "total_time_ms": total_time_ms,
                        "kernel_time_ms": kernel_time_ms,
                        "user_time_ms": user_time_ms,
                        "avg_cpu_cores": avg_cpu_cores,
                        "avg_cpu_percent_total_cores": avg_cpu_percent_total,
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "cpu_error".to_string(),
                serde_json::Value::String("GetProcessTimes failed".to_string()),
            );
        }

        let mut mem: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
        mem.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let mem_ok = unsafe { GetProcessMemoryInfo(handle, &mut mem, mem.cb) } != 0;
        if mem_ok {
            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "memory".to_string(),
                    serde_json::json!({
                        "working_set_bytes": mem.WorkingSetSize as u64,
                        "peak_working_set_bytes": mem.PeakWorkingSetSize as u64,
                        "pagefile_bytes": mem.PagefileUsage as u64,
                        "peak_pagefile_bytes": mem.PeakPagefileUsage as u64,
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "memory_error".to_string(),
                serde_json::Value::String("GetProcessMemoryInfo failed".to_string()),
            );
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(observed) = observed {
            let avg_cpu_cores = observed.cpu_usage_percent_avg / 100.0;
            let avg_cpu_percent_total_cores = if logical_cores > 0 {
                observed.cpu_usage_percent_avg / (logical_cores as f64)
            } else {
                0.0
            };

            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "cpu".to_string(),
                    serde_json::json!({
                        "collector": observed.collector,
                        "sample_interval_ms": observed.sample_interval_ms,
                        "samples": observed.samples,
                        "usage_percent_avg": observed.cpu_usage_percent_avg,
                        "usage_percent_max": observed.cpu_usage_percent_max,
                        "usage_percent_last": observed.cpu_usage_percent_last,
                        "avg_cpu_cores": avg_cpu_cores,
                        "avg_cpu_percent_total_cores": avg_cpu_percent_total_cores,
                    }),
                );
                obj.insert(
                    "memory".to_string(),
                    serde_json::json!({
                        // Align naming with Windows for easier tooling consumption.
                        "working_set_bytes": observed.working_set_bytes_last,
                        "peak_working_set_bytes": observed.working_set_bytes_peak,
                        "virtual_memory_bytes": observed.virtual_memory_bytes_last,
                        "peak_virtual_memory_bytes": observed.virtual_memory_bytes_peak,
                        "collector": "sysinfo",
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "note".to_string(),
                serde_json::Value::String("process footprint sampling unavailable".to_string()),
            );
        }
    }

    out
}

fn kill_launched_demo(child: &mut Option<LaunchedDemo>) {
    let Some(demo) = child.take() else {
        return;
    };
    let mut child_proc = demo.child;

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child_proc.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = child_proc.kill();
    }

    #[cfg(not(windows))]
    {
        let _ = child_proc.kill();
    }

    let deadline = Instant::now() + Duration::from_millis(3_000);
    while Instant::now() < deadline {
        if child_proc.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn stop_launched_demo(
    child: &mut Option<LaunchedDemo>,
    exit_path: &Path,
    poll_ms: u64,
) -> Option<serde_json::Value> {
    let demo = child.as_mut()?;

    let _ = touch(exit_path);
    #[cfg(not(windows))]
    let mut sampler = SysinfoProcessFootprintSampler::new(demo.child.id());

    let deadline = Instant::now() + Duration::from_millis(20_000);
    while Instant::now() < deadline {
        #[cfg(not(windows))]
        sampler.refresh_if_due();

        let exited = demo.child.try_wait().ok().flatten().is_some();
        if exited {
            #[cfg(not(windows))]
            sampler.refresh_force();

            let footprint = Some(collect_demo_footprint_json(demo, false, {
                #[cfg(not(windows))]
                {
                    Some(sampler.finish())
                }
                #[cfg(windows)]
                {
                    None
                }
            }));
            if let Some(mut c) = child.take().map(|d| d.child) {
                let _ = c.wait();
            }
            return footprint;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    #[cfg(not(windows))]
    sampler.refresh_force();

    let footprint = Some(collect_demo_footprint_json(demo, true, {
        #[cfg(not(windows))]
        {
            Some(sampler.finish())
        }
        #[cfg(windows)]
        {
            None
        }
    }));
    kill_launched_demo(child);
    footprint
}

fn touch(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    use std::io::Write as _;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    writeln!(f, "{ts}").map_err(|e| e.to_string())?;
    let _ = f.flush();
    Ok(())
}

fn write_script(src: &Path, dst: &Path) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn ensure_env_var(env: &mut Vec<(String, String)>, key: &str, value: &str) -> bool {
    if env.iter().any(|(k, _)| k == key) {
        return false;
    }
    env.push((key.to_string(), value.to_string()));
    true
}

fn cargo_run_inject_feature(cmd: &mut Vec<String>, feature: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }

    let exe = cmd[0].to_ascii_lowercase();
    let is_cargo = exe == "cargo" || exe.ends_with("\\cargo.exe") || exe.ends_with("/cargo");
    if !is_cargo {
        return false;
    }

    let Some(run_idx) = cmd.iter().position(|a| a == "run") else {
        return false;
    };

    // If `--features` already exists, try to extend it.
    for i in 0..cmd.len() {
        if cmd[i] == "--features" || cmd[i] == "-F" {
            if let Some(value) = cmd.get_mut(i + 1) {
                let mut features: Vec<&str> = value
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if features.iter().any(|f| *f == feature) {
                    return false;
                }
                features.push(feature);
                *value = features.join(",");
                return true;
            }
        }
        if let Some(rest) = cmd[i].strip_prefix("--features=") {
            let mut features: Vec<&str> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.iter().any(|f| *f == feature) {
                return false;
            }
            features.push(feature);
            cmd[i] = format!("--features={}", features.join(","));
            return true;
        }
        if let Some(rest) = cmd[i].strip_prefix("-F=") {
            let mut features: Vec<&str> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.iter().any(|f| *f == feature) {
                return false;
            }
            features.push(feature);
            cmd[i] = format!("-F={}", features.join(","));
            return true;
        }
    }

    // Insert `--features` before the `--` sentinel (if any), otherwise append.
    let sentinel = cmd.iter().position(|a| a == "--").unwrap_or(cmd.len());
    let insert_at = sentinel.max(run_idx + 1);
    cmd.insert(insert_at, "--features".to_string());
    cmd.insert(insert_at + 1, feature.to_string());
    true
}

fn list_files_with_extensions(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    fn visit(out: &mut Vec<PathBuf>, dir: &Path, exts: &[&str]) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                visit(out, &path, exts);
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if ext.is_empty() {
                continue;
            }
            if exts
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(ext.as_str()))
            {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    visit(&mut out, dir, exts);
    out.sort();
    out
}

fn wait_for_files_with_extensions(
    dir: &Path,
    exts: &[&str],
    timeout_ms: u64,
    poll_ms: u64,
) -> Vec<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let files = list_files_with_extensions(dir, exts);
        if !files.is_empty() {
            return files;
        }
        if Instant::now() >= deadline {
            return files;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
}

#[derive(Debug, Clone)]
struct RenderdocDumpAttempt {
    marker: String,
    out_dir: PathBuf,
    exit_code: Option<i32>,
    response_json: Option<PathBuf>,
    stdout_file: Option<PathBuf>,
    stderr_file: Option<PathBuf>,
    error: Option<String>,
}

fn run_fret_renderdoc_dump(
    workspace_root: &Path,
    capture: &Path,
    out_dir: &Path,
    basename: &str,
    marker: &str,
    max_results: Option<u32>,
    no_uniform_bytes: bool,
    no_outputs_png: bool,
    print_summary_md_top: Option<u32>,
) -> RenderdocDumpAttempt {
    let _ = std::fs::create_dir_all(out_dir);

    let stdout_path = out_dir.join("stdout.txt");
    let stderr_path = out_dir.join("stderr.txt");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root);
    cmd.args(["run", "-p", "fret-renderdoc", "--", "dump"]);
    cmd.args(["--capture", &capture.display().to_string()]);
    cmd.args(["--marker", marker]);
    cmd.args(["--out", &out_dir.display().to_string()]);
    cmd.args(["--basename", basename]);
    if let Some(n) = max_results {
        cmd.args(["--max-results", &n.to_string()]);
    }
    if no_uniform_bytes {
        cmd.arg("--no-uniform-bytes");
    }
    if no_outputs_png {
        cmd.arg("--no-outputs-png");
    }
    if let Some(top) = print_summary_md_top {
        cmd.args(["--print-summary", "md", &top.to_string()]);
    }
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").filter(|v| !v.is_empty()) {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return RenderdocDumpAttempt {
                marker: marker.to_string(),
                out_dir: out_dir.to_path_buf(),
                exit_code: None,
                response_json: None,
                stdout_file: None,
                stderr_file: None,
                error: Some(format!("failed to run fret-renderdoc: {e}")),
            };
        }
    };

    let _ = std::fs::write(&stdout_path, &output.stdout);
    let _ = std::fs::write(&stderr_path, &output.stderr);

    let exit_code = output.status.code();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let response_json = stdout_str
        .lines()
        .rev()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && l.ends_with(".response.json"))
        .map(|l| {
            let p = PathBuf::from(l);
            if p.is_absolute() {
                p
            } else {
                workspace_root.join(p)
            }
        });

    RenderdocDumpAttempt {
        marker: marker.to_string(),
        out_dir: out_dir.to_path_buf(),
        exit_code,
        response_json,
        stdout_file: Some(stdout_path),
        stderr_file: Some(stderr_path),
        error: None,
    }
}

fn write_json_value(path: &Path, v: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, Default)]
struct PerfThresholds {
    max_top_total_us: Option<u64>,
    max_top_layout_us: Option<u64>,
    max_top_solve_us: Option<u64>,
}

impl PerfThresholds {
    fn any(self) -> bool {
        self.max_top_total_us.is_some()
            || self.max_top_layout_us.is_some()
            || self.max_top_solve_us.is_some()
    }
}

#[derive(Debug, Clone)]
struct PerfBaselineFile {
    path: PathBuf,
    thresholds_by_script: std::collections::HashMap<String, PerfThresholds>,
}

fn normalize_repo_relative_path(workspace_root: &Path, p: &Path) -> String {
    let rel = p.strip_prefix(workspace_root).unwrap_or(p);
    let mut out = String::new();
    for (idx, part) in rel.components().enumerate() {
        let s = part.as_os_str().to_string_lossy();
        if idx > 0 {
            out.push('/');
        }
        out.push_str(&s);
    }
    out
}

fn read_perf_baseline_file(workspace_root: &Path, path: &Path) -> Result<PerfBaselineFile, String> {
    use std::collections::HashMap;

    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };

    let bytes = std::fs::read(&resolved).map_err(|e| {
        format!(
            "failed to read perf baseline file {}: {e}",
            resolved.display()
        )
    })?;
    let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "failed to parse perf baseline JSON {}: {e}",
            resolved.display()
        )
    })?;

    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if schema_version != 1 {
        return Err(format!(
            "unsupported perf baseline schema_version={schema_version} (expected 1): {}",
            resolved.display()
        ));
    }
    let kind = root.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    if kind != "perf_baseline" {
        return Err(format!(
            "invalid perf baseline kind={kind:?} (expected \"perf_baseline\"): {}",
            resolved.display()
        ));
    }

    let rows = root.get("rows").and_then(|v| v.as_array()).ok_or_else(|| {
        format!(
            "invalid perf baseline: missing rows array: {}",
            resolved.display()
        )
    })?;

    let mut thresholds_by_script: HashMap<String, PerfThresholds> = HashMap::new();
    for row in rows {
        let Some(script) = row.get("script").and_then(|v| v.as_str()) else {
            continue;
        };
        let t = row.get("thresholds").and_then(|v| v.as_object());
        let thresholds = PerfThresholds {
            max_top_total_us: t
                .and_then(|m| m.get("max_top_total_us"))
                .and_then(|v| v.as_u64()),
            max_top_layout_us: t
                .and_then(|m| m.get("max_top_layout_us"))
                .and_then(|v| v.as_u64()),
            max_top_solve_us: t
                .and_then(|m| m.get("max_top_solve_us"))
                .and_then(|v| v.as_u64()),
        };

        thresholds_by_script.insert(script.to_string(), thresholds);
    }

    Ok(PerfBaselineFile {
        path: resolved,
        thresholds_by_script,
    })
}

fn apply_perf_baseline_headroom(value_us: u64, headroom_pct: u32) -> u64 {
    let pct = (headroom_pct as u64).min(10_000);
    value_us.saturating_mul(100 + pct).saturating_add(99) / 100
}

fn resolve_threshold(
    cli: Option<u64>,
    baseline: Option<u64>,
) -> (Option<u64>, Option<&'static str>) {
    if let Some(v) = cli {
        return (Some(v), Some("cli"));
    }
    if let Some(v) = baseline {
        return (Some(v), Some("baseline"));
    }
    (None, None)
}

fn scan_perf_threshold_failures(
    script: &str,
    sort: BundleStatsSort,
    cli: PerfThresholds,
    baseline: PerfThresholds,
    max_total_time_us: u64,
    max_layout_time_us: u64,
    max_layout_engine_solve_time_us: u64,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let (threshold_total, source_total) =
        resolve_threshold(cli.max_top_total_us, baseline.max_top_total_us);
    let (threshold_layout, source_layout) =
        resolve_threshold(cli.max_top_layout_us, baseline.max_top_layout_us);
    let (threshold_solve, source_solve) =
        resolve_threshold(cli.max_top_solve_us, baseline.max_top_solve_us);

    if let Some(threshold_us) = threshold_total
        && max_total_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_total_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_total,
            "actual_us": max_total_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_layout
        && max_layout_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_layout,
            "actual_us": max_layout_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_solve
        && max_layout_engine_solve_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_engine_solve_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_solve,
            "actual_us": max_layout_engine_solve_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    out
}

fn read_script_result(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn read_script_result_run_id(path: &Path) -> Option<u64> {
    read_script_result(path)?.get("run_id")?.as_u64()
}

fn read_pick_result(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn read_pick_result_run_id(path: &Path) -> Option<u64> {
    read_pick_result(path)?.get("run_id")?.as_u64()
}

#[derive(Debug, Default, Clone)]
struct BundleStatsReport {
    sort: BundleStatsSort,
    warmup_frames: u64,
    windows: u32,
    snapshots: u32,
    snapshots_considered: u32,
    snapshots_skipped_warmup: u32,
    snapshots_with_model_changes: u32,
    snapshots_with_global_changes: u32,
    snapshots_with_propagated_model_changes: u32,
    snapshots_with_propagated_global_changes: u32,
    snapshots_with_hover_layout_invalidations: u32,
    sum_layout_time_us: u64,
    sum_prepaint_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_cache_roots: u64,
    sum_cache_roots_reused: u64,
    sum_cache_replayed_ops: u64,
    sum_invalidation_walk_calls: u64,
    sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    sum_hover_layout_invalidations: u64,
    max_layout_time_us: u64,
    max_prepaint_time_us: u64,
    max_paint_time_us: u64,
    max_total_time_us: u64,
    max_invalidation_walk_calls: u32,
    max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    max_hover_layout_invalidations: u32,
    worst_hover_layout: Option<BundleStatsWorstHoverLayout>,
    global_type_hotspots: Vec<BundleStatsGlobalTypeHotspot>,
    model_source_hotspots: Vec<BundleStatsModelSourceHotspot>,
    top: Vec<BundleStatsSnapshotRow>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsSnapshotRow {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    timestamp_unix_ms: Option<u64>,
    layout_time_us: u64,
    prepaint_time_us: u64,
    paint_time_us: u64,
    total_time_us: u64,
    layout_nodes_performed: u32,
    paint_nodes_performed: u32,
    paint_cache_misses: u32,
    layout_engine_solves: u64,
    layout_engine_solve_time_us: u64,
    changed_models: u32,
    changed_globals: u32,
    changed_global_types_sample: Vec<String>,
    propagated_model_change_models: u32,
    propagated_model_change_observation_edges: u32,
    propagated_model_change_unobserved_models: u32,
    propagated_global_change_globals: u32,
    propagated_global_change_observation_edges: u32,
    propagated_global_change_unobserved_globals: u32,
    invalidation_walk_calls: u32,
    invalidation_walk_nodes: u32,
    model_change_invalidation_roots: u32,
    global_change_invalidation_roots: u32,
    invalidation_walk_calls_model_change: u32,
    invalidation_walk_nodes_model_change: u32,
    invalidation_walk_calls_global_change: u32,
    invalidation_walk_nodes_global_change: u32,
    invalidation_walk_calls_hover: u32,
    invalidation_walk_nodes_hover: u32,
    invalidation_walk_calls_focus: u32,
    invalidation_walk_nodes_focus: u32,
    invalidation_walk_calls_other: u32,
    invalidation_walk_nodes_other: u32,
    top_invalidation_walks: Vec<BundleStatsInvalidationWalk>,
    hover_pressable_target_changes: u32,
    hover_hover_region_target_changes: u32,
    hover_declarative_instance_changes: u32,
    hover_declarative_hit_test_invalidations: u32,
    hover_declarative_layout_invalidations: u32,
    hover_declarative_paint_invalidations: u32,
    top_hover_declarative_invalidations: Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
    cache_roots: u32,
    cache_roots_reused: u32,
    cache_roots_contained_relayout: u32,
    cache_replayed_ops: u64,
    view_cache_contained_relayouts: u32,
    set_children_barrier_writes: u32,
    barrier_relayouts_scheduled: u32,
    barrier_relayouts_performed: u32,
    virtual_list_visible_range_checks: u32,
    virtual_list_visible_range_refreshes: u32,
    top_cache_roots: Vec<BundleStatsCacheRoot>,
    top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot>,
    top_layout_engine_solves: Vec<BundleStatsLayoutEngineSolve>,
    model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsHoverDeclarativeInvalidationHotspot {
    node: u64,
    element: Option<u64>,
    hit_test: u32,
    layout: u32,
    paint: u32,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsWorstHoverLayout {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    hover_declarative_layout_invalidations: u32,
    hotspots: Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsInvalidationWalk {
    root_node: u64,
    root_element: Option<u64>,
    kind: Option<String>,
    source: Option<String>,
    detail: Option<String>,
    walked_nodes: u32,
    truncated_at: Option<u64>,
    root_role: Option<String>,
    root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsCacheRoot {
    root_node: u64,
    element: Option<u64>,
    element_path: Option<String>,
    reused: bool,
    contained_layout: bool,
    contained_relayout_in_frame: bool,
    paint_replayed_ops: u32,
    reuse_reason: Option<String>,
    root_in_semantics: Option<bool>,
    root_role: Option<String>,
    root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsLayoutEngineSolve {
    root_node: u64,
    solve_time_us: u64,
    measure_calls: u64,
    measure_cache_hits: u64,
    measure_time_us: u64,
    top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot>,
    root_role: Option<String>,
    root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsLayoutEngineMeasureHotspot {
    node: u64,
    measure_time_us: u64,
    calls: u64,
    cache_hits: u64,
    element: Option<u64>,
    element_kind: Option<String>,
    top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsLayoutEngineMeasureChildHotspot {
    child: u64,
    measure_time_us: u64,
    calls: u64,
    element: Option<u64>,
    element_kind: Option<String>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelChangeHotspot {
    model: u64,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelChangeUnobserved {
    model: u64,
    created_type: Option<String>,
    created_at: Option<String>,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalChangeHotspot {
    type_name: String,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalChangeUnobserved {
    type_name: String,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalTypeHotspot {
    type_name: String,
    count: u64,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelSourceHotspot {
    source: String,
    count: u64,
}

impl BundleStatsReport {
    fn print_human(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        println!(
            "windows={} snapshots={} considered={} warmup_skipped={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_considered,
            self.snapshots_skipped_warmup,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        if self.warmup_frames > 0 {
            println!("warmup_frames={}", self.warmup_frames);
        }
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} prepaint={} paint={}",
            self.sum_total_time_us,
            self.sum_layout_time_us,
            self.sum_prepaint_time_us,
            self.sum_paint_time_us
        );
        println!(
            "time max (us): total={} layout={} prepaint={} paint={}",
            self.max_total_time_us,
            self.max_layout_time_us,
            self.max_prepaint_time_us,
            self.max_paint_time_us
        );
        println!(
            "cache roots sum: roots={} reused={} replayed_ops={}",
            self.sum_cache_roots, self.sum_cache_roots_reused, self.sum_cache_replayed_ops
        );
        println!(
            "invalidation sum: calls={} nodes={}",
            self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
        );
        println!(
            "invalidation max: calls={} nodes={}",
            self.max_invalidation_walk_calls, self.max_invalidation_walk_nodes
        );
        println!(
            "roots sum: model={} global={}",
            self.sum_model_change_invalidation_roots, self.sum_global_change_invalidation_roots
        );
        println!(
            "roots max: model={} global={}",
            self.max_model_change_invalidation_roots, self.max_global_change_invalidation_roots
        );
        if self.sum_hover_layout_invalidations > 0 || self.max_hover_layout_invalidations > 0 {
            println!(
                "hover decl layout invalidations: sum={} max_per_frame={} frames_with_hover_layout={}",
                self.sum_hover_layout_invalidations,
                self.max_hover_layout_invalidations,
                self.snapshots_with_hover_layout_invalidations
            );
        }

        if !self.global_type_hotspots.is_empty() {
            let items: Vec<String> = self
                .global_type_hotspots
                .iter()
                .map(|h| format!("{}={}", h.type_name, h.count))
                .collect();
            println!("changed_globals_top: {}", items.join(" | "));
        }
        if !self.model_source_hotspots.is_empty() {
            let items: Vec<String> = self
                .model_source_hotspots
                .iter()
                .map(|h| format!("{}={}", h.source, h.count))
                .collect();
            println!("changed_models_top: {}", items.join(" | "));
        }

        if self.top.is_empty() {
            return;
        }

        println!("top (sort={}):", self.sort.as_str());
        for row in &self.top {
            let ts = row
                .timestamp_unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            println!(
                "  window={} tick={} frame={} ts={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} cache_roots={} cache.reused={} cache.replayed_ops={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.total_time_us,
                row.layout_time_us,
                row.prepaint_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.cache_roots,
                row.cache_roots_reused,
                row.cache_replayed_ops,
                row.view_cache_contained_relayouts,
                row.cache_roots_contained_relayout,
                row.set_children_barrier_writes,
                row.barrier_relayouts_scheduled,
                row.barrier_relayouts_performed,
                row.virtual_list_visible_range_checks,
                row.virtual_list_visible_range_refreshes,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
                row.hover_declarative_layout_invalidations,
                row.hover_declarative_hit_test_invalidations,
                row.hover_declarative_paint_invalidations,
                row.model_change_invalidation_roots,
                row.global_change_invalidation_roots,
                row.changed_models,
                row.changed_globals,
                row.propagated_model_change_models,
                row.propagated_model_change_observation_edges,
                row.propagated_model_change_unobserved_models,
                row.propagated_global_change_globals,
                row.propagated_global_change_observation_edges,
                row.propagated_global_change_unobserved_globals
            );
            if !row.top_invalidation_walks.is_empty() {
                let items: Vec<String> = row
                    .top_invalidation_walks
                    .iter()
                    .take(3)
                    .map(|w| {
                        let mut s = format!(
                            "nodes={} src={} kind={} root={}",
                            w.walked_nodes,
                            w.source.as_deref().unwrap_or("?"),
                            w.kind.as_deref().unwrap_or("?"),
                            w.root_node
                        );
                        if let Some(detail) = w.detail.as_deref()
                            && !detail.is_empty()
                        {
                            s.push_str(&format!(" detail={detail}"));
                        }
                        if let Some(test_id) = w.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={}", test_id));
                        }
                        if let Some(role) = w.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={}", role));
                        }
                        if let Some(el) = w.root_element {
                            s.push_str(&format!(" element={}", el));
                        }
                        if let Some(trunc) = w.truncated_at {
                            s.push_str(&format!(" trunc_at={}", trunc));
                        }
                        s
                    })
                    .collect();
                println!("    top_walks: {}", items.join(" | "));
            }
            if !row.top_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!("    top_cache_roots: {}", items.join(" | "));
            }
            if !row.top_contained_relayout_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!(
                    "    top_contained_relayout_cache_roots: {}",
                    items.join(" | ")
                );
            }
            if row.hover_declarative_layout_invalidations > 0
                && !row.top_hover_declarative_invalidations.is_empty()
            {
                let items: Vec<String> = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "layout={} hit={} paint={} node={}",
                            h.layout, h.hit_test, h.paint, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    hover_layout_hotspots: {}", items.join(" | "));
            }
            if !row.top_layout_engine_solves.is_empty() {
                let items: Vec<String> = row
                    .top_layout_engine_solves
                    .iter()
                    .take(3)
                    .map(|s| {
                        let mut out = format!(
                            "us={} measure.us={} measure.calls={} hits={} root={}",
                            s.solve_time_us,
                            s.measure_time_us,
                            s.measure_calls,
                            s.measure_cache_hits,
                            s.root_node
                        );
                        if let Some(test_id) = s.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = s.root_role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(m) = s.top_measures.first() {
                            if m.measure_time_us > 0 && m.node != 0 {
                                out.push_str(&format!(
                                    " top_measure.us={} node={}",
                                    m.measure_time_us, m.node
                                ));
                                if let Some(kind) = m.element_kind.as_deref()
                                    && !kind.is_empty()
                                {
                                    out.push_str(&format!(" kind={kind}"));
                                }
                                if let Some(el) = m.element {
                                    out.push_str(&format!(" element={el}"));
                                }
                                if let Some(test_id) = m.test_id.as_deref()
                                    && !test_id.is_empty()
                                {
                                    out.push_str(&format!(" test_id={test_id}"));
                                }
                                if let Some(role) = m.role.as_deref()
                                    && !role.is_empty()
                                {
                                    out.push_str(&format!(" role={role}"));
                                }
                                if let Some(c) = m.top_children.first() {
                                    if c.measure_time_us > 0 && c.child != 0 {
                                        out.push_str(&format!(
                                            " child.us={} child={}",
                                            c.measure_time_us, c.child
                                        ));
                                        if let Some(kind) = c.element_kind.as_deref()
                                            && !kind.is_empty()
                                        {
                                            out.push_str(&format!(" child.kind={kind}"));
                                        }
                                        if let Some(el) = c.element {
                                            out.push_str(&format!(" child.element={el}"));
                                        }
                                        if let Some(test_id) = c.test_id.as_deref()
                                            && !test_id.is_empty()
                                        {
                                            out.push_str(&format!(" child.test_id={test_id}"));
                                        }
                                        if let Some(role) = c.role.as_deref()
                                            && !role.is_empty()
                                        {
                                            out.push_str(&format!(" child.role={role}"));
                                        }
                                    }
                                }
                            }
                        }
                        out
                    })
                    .collect();
                println!("    top_layout_engine_solves: {}", items.join(" | "));
            }
            if !row.model_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .model_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.model, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_models: {}", items.join(" | "));
            }
            if !row.model_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .model_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = format!("{}", u.model);
                        if let Some(ty) = u.created_type.as_deref() {
                            s.push_str(&format!("={}", ty));
                        }
                        if let Some(at) = u.created_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!(" changed@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_models: {}", items.join(" | "));
            }
            if !row.global_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .global_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.type_name, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_globals: {}", items.join(" | "));
            }
            if !row.global_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .global_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = u.type_name.clone();
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_globals: {}", items.join(" | "));
            }
            if !row.changed_global_types_sample.is_empty() {
                println!(
                    "    changed_globals: {}",
                    row.changed_global_types_sample.join(" | ")
                );
            }
        }
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value};

        let mut root = Map::new();
        root.insert("schema_version".to_string(), Value::from(1));
        root.insert("sort".to_string(), Value::from(self.sort.as_str()));
        root.insert("warmup_frames".to_string(), Value::from(self.warmup_frames));
        root.insert("windows".to_string(), Value::from(self.windows));
        root.insert("snapshots".to_string(), Value::from(self.snapshots));
        root.insert(
            "snapshots_considered".to_string(),
            Value::from(self.snapshots_considered),
        );
        root.insert(
            "snapshots_skipped_warmup".to_string(),
            Value::from(self.snapshots_skipped_warmup),
        );
        root.insert(
            "snapshots_with_model_changes".to_string(),
            Value::from(self.snapshots_with_model_changes),
        );
        root.insert(
            "snapshots_with_global_changes".to_string(),
            Value::from(self.snapshots_with_global_changes),
        );
        root.insert(
            "snapshots_with_propagated_model_changes".to_string(),
            Value::from(self.snapshots_with_propagated_model_changes),
        );
        root.insert(
            "snapshots_with_propagated_global_changes".to_string(),
            Value::from(self.snapshots_with_propagated_global_changes),
        );
        root.insert(
            "snapshots_with_hover_layout_invalidations".to_string(),
            Value::from(self.snapshots_with_hover_layout_invalidations),
        );

        let mut sum = Map::new();
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.sum_prepaint_time_us),
        );
        sum.insert(
            "paint_time_us".to_string(),
            Value::from(self.sum_paint_time_us),
        );
        sum.insert(
            "total_time_us".to_string(),
            Value::from(self.sum_total_time_us),
        );
        sum.insert("cache_roots".to_string(), Value::from(self.sum_cache_roots));
        sum.insert(
            "cache_roots_reused".to_string(),
            Value::from(self.sum_cache_roots_reused),
        );
        sum.insert(
            "cache_replayed_ops".to_string(),
            Value::from(self.sum_cache_replayed_ops),
        );
        sum.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.sum_invalidation_walk_calls),
        );
        sum.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.sum_invalidation_walk_nodes),
        );
        sum.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.sum_model_change_invalidation_roots),
        );
        sum.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.sum_global_change_invalidation_roots),
        );
        sum.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.sum_hover_layout_invalidations),
        );
        root.insert("sum".to_string(), Value::Object(sum));

        let mut max = Map::new();
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
        );
        max.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.max_prepaint_time_us),
        );
        max.insert(
            "paint_time_us".to_string(),
            Value::from(self.max_paint_time_us),
        );
        max.insert(
            "total_time_us".to_string(),
            Value::from(self.max_total_time_us),
        );
        max.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.max_invalidation_walk_calls),
        );
        max.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.max_invalidation_walk_nodes),
        );
        max.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.max_model_change_invalidation_roots),
        );
        max.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.max_global_change_invalidation_roots),
        );
        max.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.max_hover_layout_invalidations),
        );
        root.insert("max".to_string(), Value::Object(max));

        let global_type_hotspots = self
            .global_type_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "global_type_hotspots".to_string(),
            Value::Array(global_type_hotspots),
        );
        let model_source_hotspots = self
            .model_source_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("source".to_string(), Value::from(h.source.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "model_source_hotspots".to_string(),
            Value::Array(model_source_hotspots),
        );

        let top = self
            .top
            .iter()
            .map(|row| {
                let mut obj = Map::new();
                obj.insert("window".to_string(), Value::from(row.window));
                obj.insert("tick_id".to_string(), Value::from(row.tick_id));
                obj.insert("frame_id".to_string(), Value::from(row.frame_id));
                obj.insert(
                    "timestamp_unix_ms".to_string(),
                    row.timestamp_unix_ms
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "layout_time_us".to_string(),
                    Value::from(row.layout_time_us),
                );
                obj.insert(
                    "prepaint_time_us".to_string(),
                    Value::from(row.prepaint_time_us),
                );
                obj.insert("paint_time_us".to_string(), Value::from(row.paint_time_us));
                obj.insert("total_time_us".to_string(), Value::from(row.total_time_us));
                obj.insert(
                    "layout_nodes_performed".to_string(),
                    Value::from(row.layout_nodes_performed),
                );
                obj.insert(
                    "paint_nodes_performed".to_string(),
                    Value::from(row.paint_nodes_performed),
                );
                obj.insert(
                    "paint_cache_misses".to_string(),
                    Value::from(row.paint_cache_misses),
                );
                obj.insert(
                    "layout_engine_solves".to_string(),
                    Value::from(row.layout_engine_solves),
                );
                obj.insert(
                    "layout_engine_solve_time_us".to_string(),
                    Value::from(row.layout_engine_solve_time_us),
                );
                obj.insert("cache_roots".to_string(), Value::from(row.cache_roots));
                obj.insert(
                    "cache_roots_reused".to_string(),
                    Value::from(row.cache_roots_reused),
                );
                obj.insert(
                    "cache_roots_contained_relayout".to_string(),
                    Value::from(row.cache_roots_contained_relayout),
                );
                obj.insert(
                    "cache_replayed_ops".to_string(),
                    Value::from(row.cache_replayed_ops),
                );
                obj.insert(
                    "changed_models".to_string(),
                    Value::from(row.changed_models),
                );
                obj.insert(
                    "changed_globals".to_string(),
                    Value::from(row.changed_globals),
                );
                obj.insert(
                    "changed_global_types_sample".to_string(),
                    Value::Array(
                        row.changed_global_types_sample
                            .iter()
                            .cloned()
                            .map(Value::from)
                            .collect(),
                    ),
                );
                obj.insert(
                    "propagated_model_change_models".to_string(),
                    Value::from(row.propagated_model_change_models),
                );
                obj.insert(
                    "propagated_model_change_observation_edges".to_string(),
                    Value::from(row.propagated_model_change_observation_edges),
                );
                obj.insert(
                    "propagated_model_change_unobserved_models".to_string(),
                    Value::from(row.propagated_model_change_unobserved_models),
                );
                obj.insert(
                    "propagated_global_change_globals".to_string(),
                    Value::from(row.propagated_global_change_globals),
                );
                obj.insert(
                    "propagated_global_change_observation_edges".to_string(),
                    Value::from(row.propagated_global_change_observation_edges),
                );
                obj.insert(
                    "propagated_global_change_unobserved_globals".to_string(),
                    Value::from(row.propagated_global_change_unobserved_globals),
                );
                obj.insert(
                    "invalidation_walk_calls".to_string(),
                    Value::from(row.invalidation_walk_calls),
                );
                obj.insert(
                    "invalidation_walk_nodes".to_string(),
                    Value::from(row.invalidation_walk_nodes),
                );
                obj.insert(
                    "model_change_invalidation_roots".to_string(),
                    Value::from(row.model_change_invalidation_roots),
                );
                obj.insert(
                    "global_change_invalidation_roots".to_string(),
                    Value::from(row.global_change_invalidation_roots),
                );
                obj.insert(
                    "invalidation_walk_calls_model_change".to_string(),
                    Value::from(row.invalidation_walk_calls_model_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_model_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_model_change),
                );
                obj.insert(
                    "invalidation_walk_calls_global_change".to_string(),
                    Value::from(row.invalidation_walk_calls_global_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_global_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_global_change),
                );
                obj.insert(
                    "invalidation_walk_calls_hover".to_string(),
                    Value::from(row.invalidation_walk_calls_hover),
                );
                obj.insert(
                    "invalidation_walk_nodes_hover".to_string(),
                    Value::from(row.invalidation_walk_nodes_hover),
                );
                obj.insert(
                    "invalidation_walk_calls_focus".to_string(),
                    Value::from(row.invalidation_walk_calls_focus),
                );
                obj.insert(
                    "invalidation_walk_nodes_focus".to_string(),
                    Value::from(row.invalidation_walk_nodes_focus),
                );
                obj.insert(
                    "invalidation_walk_calls_other".to_string(),
                    Value::from(row.invalidation_walk_calls_other),
                );
                obj.insert(
                    "invalidation_walk_nodes_other".to_string(),
                    Value::from(row.invalidation_walk_nodes_other),
                );
                obj.insert(
                    "hover_pressable_target_changes".to_string(),
                    Value::from(row.hover_pressable_target_changes),
                );
                obj.insert(
                    "hover_hover_region_target_changes".to_string(),
                    Value::from(row.hover_hover_region_target_changes),
                );
                obj.insert(
                    "hover_declarative_instance_changes".to_string(),
                    Value::from(row.hover_declarative_instance_changes),
                );
                obj.insert(
                    "hover_declarative_hit_test_invalidations".to_string(),
                    Value::from(row.hover_declarative_hit_test_invalidations),
                );
                obj.insert(
                    "hover_declarative_layout_invalidations".to_string(),
                    Value::from(row.hover_declarative_layout_invalidations),
                );
                obj.insert(
                    "hover_declarative_paint_invalidations".to_string(),
                    Value::from(row.hover_declarative_paint_invalidations),
                );

                let top_invalidation_walks = row
                    .top_invalidation_walks
                    .iter()
                    .map(|w| {
                        let mut w_obj = Map::new();
                        w_obj.insert("root_node".to_string(), Value::from(w.root_node));
                        w_obj.insert(
                            "root_element".to_string(),
                            w.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "kind".to_string(),
                            w.kind.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "source".to_string(),
                            w.source.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "detail".to_string(),
                            w.detail.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert("walked_nodes".to_string(), Value::from(w.walked_nodes));
                        w_obj.insert(
                            "truncated_at".to_string(),
                            w.truncated_at.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_role".to_string(),
                            w.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_test_id".to_string(),
                            w.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(w_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_invalidation_walks".to_string(),
                    Value::Array(top_invalidation_walks),
                );

                let top_hover_declarative_invalidations = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert("hit_test".to_string(), Value::from(h.hit_test));
                        h_obj.insert("layout".to_string(), Value::from(h.layout));
                        h_obj.insert("paint".to_string(), Value::from(h.paint));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_hover_declarative_invalidations".to_string(),
                    Value::Array(top_hover_declarative_invalidations),
                );

                let top_cache_roots = row
                    .top_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert("top_cache_roots".to_string(), Value::Array(top_cache_roots));

                let top_contained_relayout_cache_roots = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_contained_relayout_cache_roots".to_string(),
                    Value::Array(top_contained_relayout_cache_roots),
                );

                let top_layout_engine_solves = row
                    .top_layout_engine_solves
                    .iter()
                    .map(|s| {
                        let mut s_obj = Map::new();
                        s_obj.insert("root_node".to_string(), Value::from(s.root_node));
                        s_obj.insert("solve_time_us".to_string(), Value::from(s.solve_time_us));
                        s_obj.insert("measure_calls".to_string(), Value::from(s.measure_calls));
                        s_obj.insert(
                            "measure_cache_hits".to_string(),
                            Value::from(s.measure_cache_hits),
                        );
                        s_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(s.measure_time_us),
                        );
                        let top_measures = s
                            .top_measures
                            .iter()
                            .map(|m| {
                                let mut m_obj = Map::new();
                                m_obj.insert("node".to_string(), Value::from(m.node));
                                m_obj.insert(
                                    "measure_time_us".to_string(),
                                    Value::from(m.measure_time_us),
                                );
                                m_obj.insert("calls".to_string(), Value::from(m.calls));
                                m_obj.insert("cache_hits".to_string(), Value::from(m.cache_hits));
                                m_obj.insert(
                                    "element".to_string(),
                                    m.element.map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "element_kind".to_string(),
                                    m.element_kind
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "role".to_string(),
                                    m.role.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "test_id".to_string(),
                                    m.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                let top_children = m
                                    .top_children
                                    .iter()
                                    .map(|c| {
                                        let mut c_obj = Map::new();
                                        c_obj.insert("child".to_string(), Value::from(c.child));
                                        c_obj.insert(
                                            "measure_time_us".to_string(),
                                            Value::from(c.measure_time_us),
                                        );
                                        c_obj.insert("calls".to_string(), Value::from(c.calls));
                                        c_obj.insert(
                                            "element".to_string(),
                                            c.element.map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "element_kind".to_string(),
                                            c.element_kind
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "role".to_string(),
                                            c.role.clone().map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "test_id".to_string(),
                                            c.test_id
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        Value::Object(c_obj)
                                    })
                                    .collect::<Vec<_>>();
                                m_obj
                                    .insert("top_children".to_string(), Value::Array(top_children));
                                Value::Object(m_obj)
                            })
                            .collect::<Vec<_>>();
                        s_obj.insert("top_measures".to_string(), Value::Array(top_measures));
                        s_obj.insert(
                            "root_role".to_string(),
                            s.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_test_id".to_string(),
                            s.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(s_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_layout_engine_solves".to_string(),
                    Value::Array(top_layout_engine_solves),
                );

                let model_change_hotspots = row
                    .model_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("model".to_string(), Value::from(h.model));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_hotspots".to_string(),
                    Value::Array(model_change_hotspots),
                );

                let model_change_unobserved = row
                    .model_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("model".to_string(), Value::from(u.model));
                        u_obj.insert(
                            "created_type".to_string(),
                            u.created_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        u_obj.insert(
                            "created_at".to_string(),
                            u.created_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_unobserved".to_string(),
                    Value::Array(model_change_unobserved),
                );

                let global_change_hotspots = row
                    .global_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        h_obj.insert(
                            "changed_at".to_string(),
                            h.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_hotspots".to_string(),
                    Value::Array(global_change_hotspots),
                );

                let global_change_unobserved = row
                    .global_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("type_name".to_string(), Value::from(u.type_name.clone()));
                        u_obj.insert(
                            "changed_at".to_string(),
                            u.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_unobserved".to_string(),
                    Value::Array(global_change_unobserved),
                );

                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        root.insert("top".to_string(), Value::Array(top));
        Value::Object(root)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BundleStatsOptions {
    warmup_frames: u64,
}

fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json_with_options(&bundle, top, sort, opts)
}

fn check_bundle_for_stale_paint(bundle_path: &Path, test_id: &str, eps: f32) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_paint_json(&bundle, bundle_path, test_id, eps)
}

fn check_bundle_for_stale_paint_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_fp: Option<u64> = None;
        for s in snaps {
            let y = semantics_node_y_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }
            let (Some(y), Some(fp)) = (y, fp) else {
                prev_y = y;
                prev_fp = fp;
                continue;
            };

            if let (Some(prev_y), Some(prev_fp)) = (prev_y, prev_fp) {
                if (y - prev_y).abs() >= eps as f64 && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} delta_y={:.2} scene_fingerprint=0x{:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_replayed_ops}",
                        y - prev_y,
                        fp
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = Some(y);
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale paint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale paint suspected (semantics bounds moved but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn check_bundle_for_stale_scene(bundle_path: &Path, test_id: &str, eps: f32) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_scene_json(&bundle, bundle_path, test_id, eps)
}

#[derive(Debug, Clone, Default)]
struct SemanticsChangedRepaintedScan {
    missing_scene_fingerprint: bool,
    missing_semantics_fingerprint: bool,
    suspicious_lines: Vec<String>,
    findings: Vec<serde_json::Value>,
}

fn check_bundle_for_semantics_changed_repainted(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let scan = scan_semantics_changed_repainted_json(&bundle, warmup_frames);
    if dump_json && !scan.findings.is_empty() {
        let out_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = out_dir.join("check.semantics_changed_repainted.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "semantics_changed_repainted",
            "bundle_json": bundle_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "findings": scan.findings,
        });
        let _ = write_json_value(&out_path, &payload);
    }

    check_bundle_for_semantics_changed_repainted_json(&bundle, bundle_path, warmup_frames)
}

fn check_bundle_for_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let scan = scan_semantics_changed_repainted_json(bundle, warmup_frames);

    if scan.missing_scene_fingerprint {
        return Err(format!(
            "semantics repaint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.missing_semantics_fingerprint {
        return Err(format!(
            "semantics repaint check requires `semantics_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.suspicious_lines.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "missing repaint suspected (semantics fingerprint changed but scene fingerprint did not)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in scan.suspicious_lines {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn scan_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    warmup_frames: u64,
) -> SemanticsChangedRepaintedScan {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if windows.is_empty() {
        return SemanticsChangedRepaintedScan::default();
    }

    let mut scan = SemanticsChangedRepaintedScan::default();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_scene_fingerprint: Option<u64> = None;
        let mut prev_semantics_fingerprint: Option<u64> = None;
        let mut prev_tick_id: u64 = 0;
        let mut prev_frame_id: u64 = 0;
        let mut prev_snapshot: Option<&serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scene_fingerprint = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if scene_fingerprint.is_none() {
                scan.missing_scene_fingerprint = true;
            }

            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
            if semantics_fingerprint.is_none() {
                scan.missing_semantics_fingerprint = true;
            }

            let (Some(scene_fingerprint), Some(semantics_fingerprint)) =
                (scene_fingerprint, semantics_fingerprint)
            else {
                prev_scene_fingerprint = None;
                prev_semantics_fingerprint = None;
                prev_tick_id = tick_id;
                prev_frame_id = frame_id;
                prev_snapshot = Some(s);
                continue;
            };

            if let (Some(prev_scene), Some(prev_sem)) =
                (prev_scene_fingerprint, prev_semantics_fingerprint)
            {
                let semantics_changed = semantics_fingerprint != prev_sem;
                let scene_unchanged = scene_fingerprint == prev_scene;
                if semantics_changed && scene_unchanged {
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_cache_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let diff_detail = prev_snapshot
                        .map(|prev| semantics_diff_detail(prev, s))
                        .unwrap_or(serde_json::Value::Null);

                    scan.findings.push(serde_json::json!({
                        "window": window_id,
                        "prev": {
                            "tick_id": prev_tick_id,
                            "frame_id": prev_frame_id,
                            "scene_fingerprint": prev_scene,
                            "semantics_fingerprint": prev_sem,
                        },
                        "now": {
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "scene_fingerprint": scene_fingerprint,
                            "semantics_fingerprint": semantics_fingerprint,
                        },
                        "paint_nodes_performed": paint_nodes_performed,
                        "paint_cache_replayed_ops": paint_cache_replayed_ops,
                        "semantics_diff": diff_detail,
                    }));

                    let mut detail = String::new();
                    if let Some(prev) = prev_snapshot {
                        let diff = semantics_diff_summary(prev, s);
                        if !diff.is_empty() {
                            detail.push(' ');
                            detail.push_str(&diff);
                        }
                    }

                    scan.suspicious_lines.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} prev_tick={prev_tick_id} prev_frame={prev_frame_id} semantics_fingerprint=0x{semantics_fingerprint:016x} prev_semantics_fingerprint=0x{prev_sem:016x} scene_fingerprint=0x{scene_fingerprint:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_cache_replayed_ops}{detail}"
                    ));
                    if scan.suspicious_lines.len() >= 8 {
                        break;
                    }
                }
            }

            prev_scene_fingerprint = Some(scene_fingerprint);
            prev_semantics_fingerprint = Some(semantics_fingerprint);
            prev_tick_id = tick_id;
            prev_frame_id = frame_id;
            prev_snapshot = Some(s);
        }
    }

    scan
}

fn semantics_diff_detail(
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::json;
    use std::collections::{HashMap, HashSet};

    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return serde_json::Value::Null;
    };

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64)> = Vec::new(); // (score, id)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            let score = semantics_node_score_json(a);
            changed.push((score, id));
        }
    }
    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let sample_len = 6usize;

    let added_nodes = added
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, after_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let removed_nodes = removed
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, before_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let changed_nodes = changed
        .iter()
        .take(sample_len)
        .map(|(_score, id)| {
            let before = semantics_node_summary_json(*id, before_by_id.get(id).copied());
            let after = semantics_node_summary_json(*id, after_by_id.get(id).copied());
            json!({ "id": id, "before": before, "after": after })
        })
        .collect::<Vec<_>>();

    json!({
        "counts": {
            "added": added.len(),
            "removed": removed.len(),
            "changed": changed.len(),
        },
        "samples": {
            "added_nodes": added_nodes,
            "removed_nodes": removed_nodes,
            "changed_nodes": changed_nodes,
        }
    })
}

fn semantics_node_summary_json(id: u64, node: Option<&serde_json::Value>) -> serde_json::Value {
    use serde_json::json;
    let Some(node) = node else {
        return json!({ "id": id });
    };

    let role = node.get("role").and_then(|v| v.as_str());
    let parent = node.get("parent").and_then(|v| v.as_u64());
    let test_id = node.get("test_id").and_then(|v| v.as_str());
    let label = node.get("label").and_then(|v| v.as_str());
    let value = node.get("value").and_then(|v| v.as_str());

    let bounds = node.get("bounds").and_then(|b| {
        Some(json!({
            "x": b.get("x").and_then(|v| v.as_f64()),
            "y": b.get("y").and_then(|v| v.as_f64()),
            "w": b.get("w").and_then(|v| v.as_f64()),
            "h": b.get("h").and_then(|v| v.as_f64()),
        }))
    });

    json!({
        "id": id,
        "parent": parent,
        "role": role,
        "test_id": test_id,
        "label": label,
        "value": value,
        "bounds": bounds,
    })
}

fn semantics_diff_summary(before: &serde_json::Value, after: &serde_json::Value) -> String {
    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return String::new();
    };

    use std::collections::{HashMap, HashSet};

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64, u64)> = Vec::new(); // (score, id, fp_after)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            // Score heuristic: test_id changes are the most useful to report.
            let score = semantics_node_score_json(a);
            changed.push((score, id, fp_a));
        }
    }

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return String::new();
    }

    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::new();
    out.push_str("semantics_diff={");
    out.push_str(&format!(
        "added={} removed={} changed={}",
        added.len(),
        removed.len(),
        changed.len()
    ));

    let sample_len = 6usize;
    if !changed.is_empty() {
        out.push_str(" changed_nodes=[");
        for (i, (_score, id, _fp)) in changed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if changed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !added.is_empty() {
        out.push_str(" added_nodes=[");
        for (i, id) in added.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if added.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !removed.is_empty() {
        out.push_str(" removed_nodes=[");
        for (i, id) in removed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = before_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if removed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    out.push('}');
    out
}

fn semantics_node_score_json(node: &serde_json::Value) -> u64 {
    // Higher is “more useful for debugging”.
    let mut score: u64 = 0;
    if node.get("test_id").and_then(|v| v.as_str()).is_some() {
        score += 10_000;
    }
    if node.get("label").and_then(|v| v.as_str()).is_some() {
        score += 1_000;
    }
    if node.get("value").and_then(|v| v.as_str()).is_some() {
        score += 500;
    }
    score
}

fn semantics_node_label_json(id: u64, node: Option<&serde_json::Value>) -> String {
    let Some(node) = node else {
        return format!("id={id}");
    };
    let role = node
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let test_id = node
        .get("test_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());

    let mut out = format!("id={id} role={role}");
    if let Some(v) = test_id {
        out.push_str(" test_id=");
        out.push_str(v);
    }
    if let Some(v) = label {
        out.push_str(" label=");
        out.push_str(v);
    }
    if let Some(v) = value {
        out.push_str(" value=");
        out.push_str(v);
    }
    out
}

fn semantics_node_fingerprint_json(node: &serde_json::Value) -> u64 {
    use std::hash::{Hash, Hasher};

    // Use a stable hash for a curated subset of fields.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    node.get("id").and_then(|v| v.as_u64()).hash(&mut hasher);
    node.get("parent")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("role").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(bounds) = node.get("bounds") {
        if let Some(v) = bounds.get("x").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("y").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("w").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("h").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
    }

    if let Some(flags) = node.get("flags") {
        flags
            .get("focused")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("captured")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("disabled")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("selected")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("expanded")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("checked")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    node.get("test_id")
        .and_then(|v| v.as_str())
        .hash(&mut hasher);
    node.get("active_descendant")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("pos_in_set")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("set_size")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("label").and_then(|v| v.as_str()).hash(&mut hasher);
    node.get("value").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(actions) = node.get("actions") {
        actions
            .get("focus")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("invoke")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_value")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_text_selection")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    hasher.finish()
}

fn check_bundle_for_stale_scene_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_label: Option<String> = None;
        let mut prev_value: Option<String> = None;
        let mut prev_fp: Option<u64> = None;

        for s in snaps {
            let (y, label, value) = semantics_node_fields_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let Some(fp) = fp else {
                prev_y = y;
                prev_label = label;
                prev_value = value;
                prev_fp = None;
                continue;
            };

            if let (Some(prev_fp), Some(prev_y)) = (prev_fp, prev_y) {
                let moved = y
                    .zip(Some(prev_y))
                    .is_some_and(|(y, prev_y)| (y - prev_y).abs() >= eps as f64);
                let label_changed = label.as_deref() != prev_label.as_deref();
                let value_changed = value.as_deref() != prev_value.as_deref();
                let changed = moved || label_changed || value_changed;

                if changed && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let label_len_prev = prev_label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let label_len_now = label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_prev = prev_value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_now = value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let delta_y = y
                        .zip(Some(prev_y))
                        .map(|(y, prev_y)| y - prev_y)
                        .unwrap_or(0.0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} changed={{moved={moved} label={label_changed} value={value_changed}}} delta_y={delta_y:.2} label_len={label_len_prev}->{label_len_now} value_len={value_len_prev}->{value_len_now} scene_fingerprint=0x{fp:016x}",
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = y;
            prev_label = label;
            prev_value = value;
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale scene check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale scene suspected (semantics changed but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn semantics_node_y_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<f64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    })?;
    node.get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64())
}

fn semantics_node_fields_for_test_id(
    snapshot: &serde_json::Value,
    test_id: &str,
) -> (Option<f64>, Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return (None, None, None);
    };
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    });
    let Some(node) = node else {
        return (None, None, None);
    };
    let y = node
        .get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (y, label, value)
}

fn first_wheel_frame_id_for_window(window: &serde_json::Value) -> Option<u64> {
    window
        .get("events")
        .and_then(|v| v.as_array())?
        .iter()
        .filter(|e| e.get("kind").and_then(|v| v.as_str()) == Some("pointer.wheel"))
        .filter_map(|e| e.get("frame_id").and_then(|v| v.as_u64()))
        .min()
}

fn semantics_node_id_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<u64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    nodes
        .iter()
        .find(|n| {
            n.get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id == test_id)
        })?
        .get("id")
        .and_then(|v| v.as_u64())
}

fn hit_test_node_id(snapshot: &serde_json::Value) -> Option<u64> {
    snapshot
        .get("debug")
        .and_then(|v| v.get("hit_test"))
        .and_then(|v| v.get("hit"))
        .and_then(|v| v.as_u64())
}

fn is_descendant(
    mut node: u64,
    ancestor: u64,
    parents: &std::collections::HashMap<u64, u64>,
) -> bool {
    if node == ancestor {
        return true;
    }
    while let Some(parent) = parents.get(&node).copied() {
        if parent == ancestor {
            return true;
        }
        node = parent;
    }
    false
}

fn semantics_parent_map(snapshot: &serde_json::Value) -> std::collections::HashMap<u64, u64> {
    let mut parents = std::collections::HashMap::new();
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return parents;
    };
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) else {
            continue;
        };
        parents.insert(id, parent);
    }
    parents
}

fn check_bundle_for_wheel_scroll(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_wheel_scroll_json(&bundle, bundle_path, test_id, warmup_frames)
}

fn check_bundle_for_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(hit_before) = hit_test_node_id(before) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = hit_test_node_id(after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        let Some(target_before) = semantics_node_id_for_test_id(before, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = semantics_node_id_for_test_id(after, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let before_parents = semantics_parent_map(before);
        let after_parents = semantics_parent_map(after);

        if !is_descendant(hit_before, target_before, &before_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }

        if is_descendant(hit_after, target_after, &after_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_still_within_target_after hit={hit_after} target={target_after}"
            ));
        }
    }

    if !any_wheel {
        return Err(format!(
            "wheel scroll check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll check failed (expected hit-test result to move after wheel)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut good_frames: u64 = 0;
    let mut bad_frames: Vec<String> = Vec::new();
    let mut missing_target_count: u64 = 0;
    let mut any_view_cache_active = false;
    let mut seen_good = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let Some(target_node_id) = semantics_node_id_for_test_id(s, test_id) else {
                missing_target_count = missing_target_count.saturating_add(1);
                continue;
            };

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.semantics.nodes".to_string())?;
            let mut parents: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
            for n in nodes {
                let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                if let Some(parent) = n.get("parent").and_then(|v| v.as_u64()) {
                    parents.insert(id, parent);
                }
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.cache_roots".to_string())?;
            let mut cache_roots: std::collections::HashMap<u64, &serde_json::Value> =
                std::collections::HashMap::new();
            for r in roots {
                if let Some(root) = r.get("root").and_then(|v| v.as_u64()) {
                    cache_roots.insert(root, r);
                }
            }

            let mut current = target_node_id;
            let mut cache_root_node: Option<u64> = None;
            loop {
                if cache_roots.contains_key(&current) {
                    cache_root_node = Some(current);
                    break;
                }
                let Some(parent) = parents.get(&current).copied() else {
                    break;
                };
                current = parent;
            }
            let Some(cache_root_node) = cache_root_node else {
                return Err(format!(
                    "could not resolve a cache root ancestor for test_id={test_id} (node_id={target_node_id}) in bundle: {}",
                    bundle_path.display()
                ));
            };

            let root = cache_roots
                .get(&cache_root_node)
                .ok_or_else(|| "internal error: cache root missing".to_string())?;

            let reused = root
                .get("reused")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let contained_relayout_in_frame = root
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let dirty = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(false, |dirty| {
                    dirty.iter().any(|d| {
                        d.get("root_node")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|n| n == cache_root_node)
                    })
                });

            let ok = reused && !contained_relayout_in_frame && !dirty;
            if ok {
                good_frames = good_frames.saturating_add(1);
                seen_good = true;
                continue;
            }

            if seen_good {
                bad_frames.push(format!(
                    "window={window_id} frame_id={frame_id} cache_root={cache_root_node} reused={reused} contained_relayout_in_frame={contained_relayout_in_frame} dirty={dirty}"
                ));
            }
        }
    }

    if !bad_frames.is_empty() {
        let mut msg = String::new();
        msg.push_str("expected paint-only drag indicator updates (cache-root reuse, no contained relayout, no dirty view), but found violations after reuse began\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!("test_id: {test_id}\n"));
        for line in bad_frames.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if good_frames == 0 {
        return Err(format!(
            "did not observe any cache-root-reuse paint-only frames for test_id={test_id} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, missing_target_count={missing_target_count}) \
in bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

fn check_bundle_for_gc_sweep_liveness(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(removed) = s
                .get("debug")
                .and_then(|v| v.get("removed_subtrees"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for r in removed {
                let unreachable = r
                    .get("unreachable_from_liveness_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let reachable_from_layer_roots = r
                    .get("reachable_from_layer_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reachable_from_view_cache_roots = r
                    .get("reachable_from_view_cache_roots")
                    .and_then(|v| v.as_bool());
                let root_layer_visible = r.get("root_layer_visible").and_then(|v| v.as_bool());

                if !unreachable
                    || reachable_from_layer_roots
                    || reachable_from_view_cache_roots == Some(true)
                    || root_layer_visible == Some(true)
                {
                    let root = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
                    let root_element_path = r
                        .get("root_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let trigger_path = r
                        .get("trigger_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    offenders.push(format!(
                        "window={window_id} frame_id={frame_id} root={root} unreachable_from_liveness_roots={unreachable} reachable_from_layer_roots={reachable_from_layer_roots} reachable_from_view_cache_roots={reachable_from_view_cache_roots:?} root_layer_visible={root_layer_visible:?} root_element_path={root_element_path} trigger_element_path={trigger_path}"
                    ));
                }
            }
        }
    }

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("GC sweep liveness violation: removed_subtrees contains entries that appear live (reachable/visible)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn check_bundle_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_view_cache_reuse_min_json(
        &bundle,
        bundle_path,
        min_reuse_events,
        warmup_frames,
    )
}

fn check_bundle_for_view_cache_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reuse_events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array());
            let Some(roots) = roots else {
                continue;
            };

            for r in roots {
                if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                    reuse_events = reuse_events.saturating_add(1);
                    if reuse_events >= min_reuse_events {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_reuse_events} view-cache reuse events, got {reuse_events} \
 (any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
 in bundle: {}",
        bundle_path.display()
    ))
}

#[derive(Debug, Clone)]
struct ViewCacheReuseStableWindowReport {
    window: u64,
    examined_snapshots: u64,
    view_cache_active_snapshots: u64,
    non_reuse_cache_inactive_snapshots: u64,
    non_reuse_active_no_signal_snapshots: u64,
    reuse_snapshots: u64,
    reuse_streak_max: u64,
    reuse_streak_tail: u64,
    last_non_reuse: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
struct ViewCacheReuseSignal {
    view_cache_active: bool,
    has_reuse_signal: bool,
    reused_roots: u64,
    paint_cache_replayed_ops: u64,
    cache_roots_present: bool,
}

impl ViewCacheReuseSignal {
    fn no_signal_reason(self) -> &'static str {
        if !self.view_cache_active {
            return "view_cache_inactive";
        }
        "active_no_signal"
    }
}

fn snapshot_view_cache_reuse_signal(snapshot: &serde_json::Value) -> ViewCacheReuseSignal {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let view_cache_active = stats
        .and_then(|v| v.get("view_cache_active"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let replayed_ops = stats
        .and_then(|v| v.get("paint_cache_replayed_ops"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut reused_roots: u64 = 0;
    let mut cache_roots_present = false;
    if let Some(roots) = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
    {
        cache_roots_present = true;
        for r in roots {
            if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                reused_roots = reused_roots.saturating_add(1);
            }
        }
    }

    let has_signal = view_cache_active && (reused_roots > 0 || replayed_ops > 0);
    ViewCacheReuseSignal {
        view_cache_active,
        has_reuse_signal: has_signal,
        reused_roots,
        paint_cache_replayed_ops: replayed_ops,
        cache_roots_present,
    }
}

fn check_bundle_for_view_cache_reuse_stable_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_tail_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reports: Vec<ViewCacheReuseStableWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    let mut any_view_cache_active = false;
    let mut best_tail: u64 = 0;

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut view_cache_active_snapshots: u64 = 0;
        let mut non_reuse_cache_inactive_snapshots: u64 = 0;
        let mut non_reuse_active_no_signal_snapshots: u64 = 0;
        let mut reuse_snapshots: u64 = 0;
        let mut reuse_streak: u64 = 0;
        let mut reuse_streak_max: u64 = 0;
        let mut last_non_reuse: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let stats = s.get("debug").and_then(|v| v.get("stats"));
            let view_cache_active = stats
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if view_cache_active {
                view_cache_active_snapshots = view_cache_active_snapshots.saturating_add(1);
            }

            let signal = snapshot_view_cache_reuse_signal(s);
            if signal.has_reuse_signal {
                reuse_snapshots = reuse_snapshots.saturating_add(1);
                reuse_streak = reuse_streak.saturating_add(1);
                reuse_streak_max = reuse_streak_max.max(reuse_streak);
            } else {
                reuse_streak = 0;
                match signal.no_signal_reason() {
                    "view_cache_inactive" => {
                        non_reuse_cache_inactive_snapshots =
                            non_reuse_cache_inactive_snapshots.saturating_add(1);
                    }
                    _ => {
                        non_reuse_active_no_signal_snapshots =
                            non_reuse_active_no_signal_snapshots.saturating_add(1);
                    }
                }
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_non_reuse = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "reason": signal.no_signal_reason(),
                    "view_cache_active": signal.view_cache_active,
                    "cache_roots_present": signal.cache_roots_present,
                    "reused_roots": signal.reused_roots,
                    "paint_cache_replayed_ops": signal.paint_cache_replayed_ops,
                }));
            }
        }

        best_tail = best_tail.max(reuse_streak);

        reports.push(ViewCacheReuseStableWindowReport {
            window,
            examined_snapshots,
            view_cache_active_snapshots,
            non_reuse_cache_inactive_snapshots,
            non_reuse_active_no_signal_snapshots,
            reuse_snapshots,
            reuse_streak_max,
            reuse_streak_tail: reuse_streak,
            last_non_reuse: last_non_reuse.clone(),
        });

        if min_tail_frames > 0 && examined_snapshots < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "insufficient_snapshots",
                "examined_snapshots": examined_snapshots,
            }));
        } else if min_tail_frames > 0 && reuse_streak < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "reuse_tail_streak_too_small",
                "examined_snapshots": examined_snapshots,
                "view_cache_active_snapshots": view_cache_active_snapshots,
                "non_reuse_cache_inactive_snapshots": non_reuse_cache_inactive_snapshots,
                "non_reuse_active_no_signal_snapshots": non_reuse_active_no_signal_snapshots,
                "reuse_streak_tail": reuse_streak,
                "reuse_streak_max": reuse_streak_max,
                "reuse_snapshots": reuse_snapshots,
                "last_non_reuse": last_non_reuse,
            }));
        }
    }

    let out_path = out_dir.join("check.view_cache_reuse_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "view_cache_reuse_stable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_tail_frames": min_tail_frames,
        "any_view_cache_active": any_view_cache_active,
        "best_reuse_streak_tail": best_tail,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "view_cache_active_snapshots": r.view_cache_active_snapshots,
            "non_reuse_cache_inactive_snapshots": r.non_reuse_cache_inactive_snapshots,
            "non_reuse_active_no_signal_snapshots": r.non_reuse_active_no_signal_snapshots,
            "reuse_snapshots": r.reuse_snapshots,
            "reuse_streak_max": r.reuse_streak_max,
            "reuse_streak_tail": r.reuse_streak_tail,
            "last_non_reuse": r.last_non_reuse,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if min_tail_frames == 0 {
        return Ok(());
    }
    if !any_view_cache_active {
        return Err(format!(
            "view-cache reuse stable gate requires view_cache_active snapshots, but none were observed (warmup_frames={warmup_frames})\n  hint: enable view-cache for the target demo if applicable (e.g. UI gallery: FRET_UI_GALLERY_VIEW_CACHE=1)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }
    if best_tail >= min_tail_frames {
        return Ok(());
    }

    Err(format!(
        "view-cache reuse stable gate failed (min_tail_frames={min_tail_frames}, best_tail={best_tail}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

fn check_bundle_for_overlay_synthesis_min(
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_overlay_synthesis_min_json(
        &bundle,
        bundle_path,
        min_synthesized_events,
        warmup_frames,
    )
}

fn check_bundle_for_overlay_synthesis_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut synthesized_events: u64 = 0;
    let mut suppression_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;

            let Some(events) = s
                .get("debug")
                .and_then(|v| v.get("overlay_synthesis"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for e in events {
                let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
                let outcome = e
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                if outcome == "synthesized" {
                    synthesized_events = synthesized_events.saturating_add(1);
                    if synthesized_events >= min_synthesized_events {
                        return Ok(());
                    }
                } else {
                    let key = format!("{kind}/{outcome}");
                    *suppression_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    let mut suppressions: Vec<(String, u64)> = suppression_counts.into_iter().collect();
    suppressions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    suppressions.truncate(12);
    let suppressions = if suppressions.is_empty() {
        String::new()
    } else {
        let mut msg = String::new();
        msg.push_str(" suppressions=[");
        for (idx, (k, c)) in suppressions.into_iter().enumerate() {
            if idx > 0 {
                msg.push_str(", ");
            }
            msg.push_str(&format!("{k}:{c}"));
        }
        msg.push(']');
        msg
    };

    Err(format!(
        "expected at least {min_synthesized_events} overlay synthesis events, got {synthesized_events} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}).{suppressions} \
bundle: {}",
        bundle_path.display()
    ))
}

fn check_bundle_for_retained_vlist_reconcile_no_notify_min(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        bundle_path,
        min_reconcile_events,
        warmup_frames,
    )
}

fn check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut notify_offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let dirty_views = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for dv in dirty_views {
                let source = dv
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let detail = dv
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if source == "notify" || detail.contains("notify") {
                    let root_node = dv.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0);
                    notify_offenders.push(format!(
                        "frame_id={frame_id} dirty_view_root_node={root_node} source={source} detail={detail}"
                    ));
                    break;
                }
            }
        }
    }

    if !notify_offenders.is_empty() {
        let mut msg = String::new();
        msg.push_str(
            "retained virtual-list reconcile should not require notify-based dirty views\n",
        );
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_reconcile_events={min_reconcile_events} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in notify_offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if reconcile_events < min_reconcile_events {
        return Err(format!(
            "expected at least {min_reconcile_events} retained virtual-list reconcile events, got {reconcile_events} \
(reconcile_frames={reconcile_frames}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

fn check_bundle_for_retained_vlist_attach_detach_max(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        bundle_path,
        max_delta,
        warmup_frames,
    )
}

fn check_bundle_for_retained_vlist_attach_detach_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let records = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let (attached, detached) = if records.is_empty() {
                let stats = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.as_object());
                let attached = stats
                    .and_then(|v| v.get("retained_virtual_list_attached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let detached = stats
                    .and_then(|v| v.get("retained_virtual_list_detached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (attached, detached)
            } else {
                let attached = records
                    .iter()
                    .map(|r| {
                        r.get("attached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                let detached = records
                    .iter()
                    .map(|r| {
                        r.get("detached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                (attached, detached)
            };

            let delta = attached.saturating_add(detached);
            if delta > max_delta {
                offenders.push(format!(
                    "frame_id={frame_id} attached={attached} detached={detached} delta={delta} max={max_delta}"
                ));
            }
        }
    }

    if reconcile_events == 0 {
        return Err(format!(
            "expected at least 1 retained virtual-list reconcile event (required for attach/detach max check), got 0 \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
            bundle_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained virtual-list attach/detach delta exceeded the configured maximum\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "max_delta={max_delta} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn check_bundle_for_viewport_input_min(
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_input_min_json(&bundle, bundle_path, min_events, warmup_frames)
}

fn check_bundle_for_viewport_input_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut events: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(arr) = s
                .get("debug")
                .and_then(|v| v.get("viewport_input"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            events = events.saturating_add(arr.len() as u64);
            if events >= min_events {
                return Ok(());
            }
        }
    }

    Err(format!(
        "expected at least {min_events} viewport input events, got {events} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

fn check_bundle_for_dock_drag_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_dock_drag_min_json(&bundle, bundle_path, min_active_frames, warmup_frames)
}

fn check_bundle_for_dock_drag_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(dock_drag) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("dock_drag"))
            else {
                continue;
            };
            if dock_drag.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active dock drag, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

fn check_bundle_for_viewport_capture_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_capture_min_json(
        &bundle,
        bundle_path,
        min_active_frames,
        warmup_frames,
    )
}

fn check_bundle_for_viewport_capture_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(viewport_capture) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("viewport_capture"))
            else {
                continue;
            };
            if viewport_capture.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active viewport capture, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

fn bundle_stats_from_json_with_options(
    bundle: &serde_json::Value,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out = BundleStatsReport::default();
    out.sort = sort;
    out.warmup_frames = opts.warmup_frames;
    out.windows = windows.len().min(u32::MAX as usize) as u32;

    let mut rows: Vec<BundleStatsSnapshotRow> = Vec::new();
    let mut global_type_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut model_source_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            out.snapshots = out.snapshots.saturating_add(1);
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < opts.warmup_frames {
                out.snapshots_skipped_warmup = out.snapshots_skipped_warmup.saturating_add(1);
                continue;
            }
            out.snapshots_considered = out.snapshots_considered.saturating_add(1);

            let changed_models = s
                .get("changed_models")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
                .min(u32::MAX as usize) as u32;
            let changed_globals_arr = s
                .get("changed_globals")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let changed_globals = changed_globals_arr.len().min(u32::MAX as usize) as u32;
            let mut changed_global_types_sample: Vec<String> = Vec::new();
            for (idx, g) in changed_globals_arr.iter().enumerate() {
                let Some(ty) = g.as_str() else {
                    continue;
                };
                *global_type_counts.entry(ty.to_string()).or_insert(0) += 1;
                if idx < 6 {
                    changed_global_types_sample.push(ty.to_string());
                }
            }

            if let Some(arr) = s
                .get("changed_model_sources_top")
                .and_then(|v| v.as_array())
            {
                for item in arr {
                    let Some(type_name) = item.get("type_name").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(at) = item.get("changed_at").and_then(|v| v.as_object()) else {
                        continue;
                    };
                    let Some(file) = at.get("file").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(line) = at.get("line").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let Some(column) = at.get("column").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                    let key = format!("{}@{}:{}:{}", type_name, file, line, column);
                    *model_source_counts.entry(key).or_insert(0) += count;
                }
            }

            if changed_models > 0 {
                out.snapshots_with_model_changes =
                    out.snapshots_with_model_changes.saturating_add(1);
            }
            if changed_globals > 0 {
                out.snapshots_with_global_changes =
                    out.snapshots_with_global_changes.saturating_add(1);
            }

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());

            let layout_time_us = stats
                .and_then(|m| m.get("layout_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let prepaint_time_us = stats
                .and_then(|m| m.get("prepaint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_time_us = stats
                .and_then(|m| m.get("paint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_time_us = layout_time_us
                .saturating_add(prepaint_time_us)
                .saturating_add(paint_time_us);
            let layout_nodes_performed = stats
                .and_then(|m| m.get("layout_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_nodes_performed = stats
                .and_then(|m| m.get("paint_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_cache_misses = stats
                .and_then(|m| m.get("paint_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let layout_engine_solves = stats
                .and_then(|m| m.get("layout_engine_solves"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_solve_time_us = stats
                .and_then(|m| m.get("layout_engine_solve_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let view_cache_contained_relayouts = stats
                .and_then(|m| m.get("view_cache_contained_relayouts"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let set_children_barrier_writes = stats
                .and_then(|m| m.get("set_children_barrier_writes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_scheduled = stats
                .and_then(|m| m.get("barrier_relayouts_scheduled"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_performed = stats
                .and_then(|m| m.get("barrier_relayouts_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_checks = stats
                .and_then(|m| m.get("virtual_list_visible_range_checks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_refreshes = stats
                .and_then(|m| m.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let propagated_model_change_models = stats
                .and_then(|m| m.get("model_change_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_model_change_observation_edges = stats
                .and_then(|m| m.get("model_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_model_change_unobserved_models = stats
                .and_then(|m| m.get("model_change_unobserved_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_globals = stats
                .and_then(|m| m.get("global_change_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_global_change_observation_edges = stats
                .and_then(|m| m.get("global_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_unobserved_globals = stats
                .and_then(|m| m.get("global_change_unobserved_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;

            if propagated_model_change_models > 0 {
                out.snapshots_with_propagated_model_changes = out
                    .snapshots_with_propagated_model_changes
                    .saturating_add(1);
            }
            if propagated_global_change_globals > 0 {
                out.snapshots_with_propagated_global_changes = out
                    .snapshots_with_propagated_global_changes
                    .saturating_add(1);
            }

            let invalidation_walk_calls = stats
                .and_then(|m| m.get("invalidation_walk_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes = stats
                .and_then(|m| m.get("invalidation_walk_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let model_change_invalidation_roots = stats
                .and_then(|m| m.get("model_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let global_change_invalidation_roots = stats
                .and_then(|m| m.get("global_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_model_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_model_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_global_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_nodes_global_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_calls_hover = stats
                .and_then(|m| m.get("invalidation_walk_calls_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_hover = stats
                .and_then(|m| m.get("invalidation_walk_nodes_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_focus = stats
                .and_then(|m| m.get("invalidation_walk_calls_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_focus = stats
                .and_then(|m| m.get("invalidation_walk_nodes_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_other = stats
                .and_then(|m| m.get("invalidation_walk_calls_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_other = stats
                .and_then(|m| m.get("invalidation_walk_nodes_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let top_invalidation_walks = snapshot_top_invalidation_walks(s, 3);
            let hover_pressable_target_changes = stats
                .and_then(|m| m.get("hover_pressable_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_hover_region_target_changes = stats
                .and_then(|m| m.get("hover_hover_region_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_instance_changes = stats
                .and_then(|m| m.get("hover_declarative_instance_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_hit_test_invalidations = stats
                .and_then(|m| m.get("hover_declarative_hit_test_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_layout_invalidations = stats
                .and_then(|m| m.get("hover_declarative_layout_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_paint_invalidations = stats
                .and_then(|m| m.get("hover_declarative_paint_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let top_hover_declarative_invalidations =
                snapshot_top_hover_declarative_invalidations(s, 3);
            let (
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                top_cache_roots,
                top_contained_relayout_cache_roots,
            ) = snapshot_cache_root_stats(s, 3);
            let top_layout_engine_solves = snapshot_layout_engine_solves(s, 3);
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);
            let global_change_hotspots = snapshot_global_change_hotspots(s, 3);
            let global_change_unobserved = snapshot_global_change_unobserved(s, 3);

            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout_time_us);
            out.sum_prepaint_time_us = out.sum_prepaint_time_us.saturating_add(prepaint_time_us);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint_time_us);
            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total_time_us);
            out.sum_cache_roots = out.sum_cache_roots.saturating_add(cache_roots as u64);
            out.sum_cache_roots_reused = out
                .sum_cache_roots_reused
                .saturating_add(cache_roots_reused as u64);
            out.sum_cache_replayed_ops = out
                .sum_cache_replayed_ops
                .saturating_add(cache_replayed_ops);
            out.sum_invalidation_walk_calls = out
                .sum_invalidation_walk_calls
                .saturating_add(invalidation_walk_calls as u64);
            out.sum_invalidation_walk_nodes = out
                .sum_invalidation_walk_nodes
                .saturating_add(invalidation_walk_nodes as u64);
            out.sum_model_change_invalidation_roots = out
                .sum_model_change_invalidation_roots
                .saturating_add(model_change_invalidation_roots as u64);
            out.sum_global_change_invalidation_roots = out
                .sum_global_change_invalidation_roots
                .saturating_add(global_change_invalidation_roots as u64);
            if hover_declarative_layout_invalidations > 0 {
                out.snapshots_with_hover_layout_invalidations = out
                    .snapshots_with_hover_layout_invalidations
                    .saturating_add(1);
            }
            out.sum_hover_layout_invalidations = out
                .sum_hover_layout_invalidations
                .saturating_add(hover_declarative_layout_invalidations as u64);

            out.max_invalidation_walk_calls =
                out.max_invalidation_walk_calls.max(invalidation_walk_calls);
            out.max_invalidation_walk_nodes =
                out.max_invalidation_walk_nodes.max(invalidation_walk_nodes);
            out.max_model_change_invalidation_roots = out
                .max_model_change_invalidation_roots
                .max(model_change_invalidation_roots);
            out.max_global_change_invalidation_roots = out
                .max_global_change_invalidation_roots
                .max(global_change_invalidation_roots);
            if hover_declarative_layout_invalidations > out.max_hover_layout_invalidations {
                out.worst_hover_layout = Some(BundleStatsWorstHoverLayout {
                    window: window_id,
                    tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    hover_declarative_layout_invalidations,
                    hotspots: snapshot_top_hover_declarative_invalidations(s, 8),
                });
            }
            out.max_hover_layout_invalidations = out
                .max_hover_layout_invalidations
                .max(hover_declarative_layout_invalidations);
            out.max_layout_time_us = out.max_layout_time_us.max(layout_time_us);
            out.max_prepaint_time_us = out.max_prepaint_time_us.max(prepaint_time_us);
            out.max_paint_time_us = out.max_paint_time_us.max(paint_time_us);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                layout_time_us,
                prepaint_time_us,
                paint_time_us,
                total_time_us,
                layout_nodes_performed,
                paint_nodes_performed,
                paint_cache_misses,
                layout_engine_solves,
                layout_engine_solve_time_us,
                changed_models,
                changed_globals,
                changed_global_types_sample,
                propagated_model_change_models,
                propagated_model_change_observation_edges,
                propagated_model_change_unobserved_models,
                propagated_global_change_globals,
                propagated_global_change_observation_edges,
                propagated_global_change_unobserved_globals,
                invalidation_walk_calls,
                invalidation_walk_nodes,
                model_change_invalidation_roots,
                global_change_invalidation_roots,
                invalidation_walk_calls_model_change,
                invalidation_walk_nodes_model_change,
                invalidation_walk_calls_global_change,
                invalidation_walk_nodes_global_change,
                invalidation_walk_calls_hover,
                invalidation_walk_nodes_hover,
                invalidation_walk_calls_focus,
                invalidation_walk_nodes_focus,
                invalidation_walk_calls_other,
                invalidation_walk_nodes_other,
                top_invalidation_walks,
                hover_pressable_target_changes,
                hover_hover_region_target_changes,
                hover_declarative_instance_changes,
                hover_declarative_hit_test_invalidations,
                hover_declarative_layout_invalidations,
                hover_declarative_paint_invalidations,
                top_hover_declarative_invalidations,
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                view_cache_contained_relayouts,
                set_children_barrier_writes,
                barrier_relayouts_scheduled,
                barrier_relayouts_performed,
                virtual_list_visible_range_checks,
                virtual_list_visible_range_refreshes,
                top_cache_roots,
                top_contained_relayout_cache_roots,
                top_layout_engine_solves,
                model_change_hotspots,
                model_change_unobserved,
                global_change_hotspots,
                global_change_unobserved,
            });
        }
    }

    match sort {
        BundleStatsSort::Invalidation => {
            rows.sort_by(|a, b| {
                b.invalidation_walk_nodes
                    .cmp(&a.invalidation_walk_nodes)
                    .then_with(|| b.invalidation_walk_calls.cmp(&a.invalidation_walk_calls))
                    .then_with(|| {
                        b.model_change_invalidation_roots
                            .cmp(&a.model_change_invalidation_roots)
                    })
                    .then_with(|| {
                        b.global_change_invalidation_roots
                            .cmp(&a.global_change_invalidation_roots)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::Time => {
            rows.sort_by(|a, b| {
                b.total_time_us
                    .cmp(&a.total_time_us)
                    .then_with(|| b.layout_time_us.cmp(&a.layout_time_us))
                    .then_with(|| b.paint_time_us.cmp(&a.paint_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
    }
    let mut hotspots: Vec<BundleStatsGlobalTypeHotspot> = global_type_counts
        .into_iter()
        .map(|(type_name, count)| BundleStatsGlobalTypeHotspot { type_name, count })
        .collect();
    hotspots.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.type_name.cmp(&b.type_name))
    });
    hotspots.truncate(top);
    out.global_type_hotspots = hotspots;

    let mut model_hotspots: Vec<BundleStatsModelSourceHotspot> = model_source_counts
        .into_iter()
        .map(|(source, count)| BundleStatsModelSourceHotspot { source, count })
        .collect();
    model_hotspots.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.source.cmp(&b.source)));
    model_hotspots.truncate(top);
    out.model_source_hotspots = model_hotspots;

    out.top = rows.into_iter().take(top).collect();
    Ok(out)
}

fn snapshot_top_invalidation_walks(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsInvalidationWalk> {
    let walks = snapshot
        .get("debug")
        .and_then(|v| v.get("invalidation_walks"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if walks.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsInvalidationWalk> = walks
        .iter()
        .map(|w| BundleStatsInvalidationWalk {
            root_node: w.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
            root_element: w.get("root_element").and_then(|v| v.as_u64()),
            kind: w
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source: w
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            detail: w
                .get("detail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            walked_nodes: w
                .get("walked_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            truncated_at: w.get("truncated_at").and_then(|v| v.as_u64()),
            root_role: None,
            root_test_id: None,
        })
        .collect();

    out.sort_by(|a, b| b.walked_nodes.cmp(&a.walked_nodes));
    out.truncate(max);

    for walk in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, walk.root_node);
        walk.root_role = role;
        walk.root_test_id = test_id;
    }

    out
}

fn snapshot_cache_root_stats(
    snapshot: &serde_json::Value,
    max: usize,
) -> (
    u32,
    u32,
    u32,
    u64,
    Vec<BundleStatsCacheRoot>,
    Vec<BundleStatsCacheRoot>,
) {
    let roots = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if roots.is_empty() {
        return (0, 0, 0, 0, Vec::new(), Vec::new());
    }

    let mut reused: u32 = 0;
    let mut contained_relayout: u32 = 0;
    let mut replayed_ops_sum: u64 = 0;

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsCacheRoot> = roots
        .iter()
        .map(|r| {
            let root_node = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
            let paint_replayed_ops = r
                .get("paint_replayed_ops")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let reused_flag = r.get("reused").and_then(|v| v.as_bool()).unwrap_or(false);
            if reused_flag {
                reused = reused.saturating_add(1);
            }
            let contained_relayout_in_frame = r
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if contained_relayout_in_frame {
                contained_relayout = contained_relayout.saturating_add(1);
            }
            replayed_ops_sum = replayed_ops_sum.saturating_add(paint_replayed_ops as u64);

            let (role, test_id) = semantics_index.lookup_for_cache_root(root_node);
            BundleStatsCacheRoot {
                root_node,
                element: r.get("element").and_then(|v| v.as_u64()),
                element_path: r
                    .get("element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                reused: reused_flag,
                contained_layout: r
                    .get("contained_layout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                contained_relayout_in_frame,
                paint_replayed_ops,
                reuse_reason: r
                    .get("reuse_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_in_semantics: r.get("root_in_semantics").and_then(|v| v.as_bool()),
                root_role: role,
                root_test_id: test_id,
            }
        })
        .collect();

    out.sort_by(|a, b| b.paint_replayed_ops.cmp(&a.paint_replayed_ops));
    let top_cache_roots: Vec<BundleStatsCacheRoot> = out.iter().take(max).cloned().collect();
    let top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot> = out
        .iter()
        .filter(|r| r.contained_relayout_in_frame)
        .take(max)
        .cloned()
        .collect();

    (
        roots.len().min(u32::MAX as usize) as u32,
        reused,
        contained_relayout,
        replayed_ops_sum,
        top_cache_roots,
        top_contained_relayout_cache_roots,
    )
}

fn snapshot_top_hover_declarative_invalidations(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsHoverDeclarativeInvalidationHotspot> {
    let items = snapshot
        .get("debug")
        .and_then(|v| v.get("hover_declarative_invalidation_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if items.is_empty() || max == 0 {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsHoverDeclarativeInvalidationHotspot> = items
        .iter()
        .map(|h| BundleStatsHoverDeclarativeInvalidationHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            hit_test: h
                .get("hit_test")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            layout: h
                .get("layout")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            paint: h
                .get("paint")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    out.sort_by(|a, b| {
        b.layout
            .cmp(&a.layout)
            .then_with(|| b.hit_test.cmp(&a.hit_test))
            .then_with(|| b.paint.cmp(&a.paint))
    });
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

fn check_report_for_hover_layout_invalidations(
    report: &BundleStatsReport,
    max_allowed: u32,
) -> Result<(), String> {
    if report.max_hover_layout_invalidations <= max_allowed {
        return Ok(());
    }

    let mut extra = String::new();
    if let Some(worst) = report.worst_hover_layout.as_ref() {
        extra.push_str(&format!(
            " worst(window={} tick={} frame={} hover_layout={})",
            worst.window,
            worst.tick_id,
            worst.frame_id,
            worst.hover_declarative_layout_invalidations
        ));
        if !worst.hotspots.is_empty() {
            let items: Vec<String> = worst
                .hotspots
                .iter()
                .take(3)
                .map(|h| {
                    let mut s = format!(
                        "layout={} hit={} paint={} node={}",
                        h.layout, h.hit_test, h.paint, h.node
                    );
                    if let Some(test_id) = h.test_id.as_deref()
                        && !test_id.is_empty()
                    {
                        s.push_str(&format!(" test_id={test_id}"));
                    }
                    if let Some(role) = h.role.as_deref()
                        && !role.is_empty()
                    {
                        s.push_str(&format!(" role={role}"));
                    }
                    s
                })
                .collect();
            extra.push_str(&format!(" hotspots=[{}]", items.join(" | ")));
        }
    }

    Err(format!(
        "hover-attributed declarative layout invalidations detected (max_per_frame={} allowed={max_allowed}).{}",
        report.max_hover_layout_invalidations, extra
    ))
}

fn snapshot_layout_engine_solves(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutEngineSolve> {
    let solves = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_engine_solves"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if solves.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsLayoutEngineSolve> = solves
        .iter()
        .map(|s| {
            let top_measures = s
                .get("top_measures")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let mut top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot> = top_measures
                .iter()
                .take(3)
                .map(|m| {
                    let children = m
                        .get("top_children")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let mut top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot> =
                        children
                            .iter()
                            .take(3)
                            .map(|c| BundleStatsLayoutEngineMeasureChildHotspot {
                                child: c.get("child").and_then(|v| v.as_u64()).unwrap_or(0),
                                measure_time_us: c
                                    .get("measure_time_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                calls: c.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                                element: c.get("element").and_then(|v| v.as_u64()),
                                element_kind: c
                                    .get("element_kind")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                role: None,
                                test_id: None,
                            })
                            .collect();

                    for item in &mut top_children {
                        let (role, test_id) = snapshot_lookup_semantics(snapshot, item.child);
                        item.role = role;
                        item.test_id = test_id;
                    }

                    BundleStatsLayoutEngineMeasureHotspot {
                        node: m.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                        measure_time_us: m
                            .get("measure_time_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        calls: m.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                        cache_hits: m.get("cache_hits").and_then(|v| v.as_u64()).unwrap_or(0),
                        element: m.get("element").and_then(|v| v.as_u64()),
                        element_kind: m
                            .get("element_kind")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        top_children,
                        role: None,
                        test_id: None,
                    }
                })
                .collect();

            for item in &mut top_measures {
                let (role, test_id) = snapshot_lookup_semantics(snapshot, item.node);
                item.role = role;
                item.test_id = test_id;
            }

            BundleStatsLayoutEngineSolve {
                root_node: s.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
                solve_time_us: s.get("solve_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_calls: s.get("measure_calls").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_cache_hits: s
                    .get("measure_cache_hits")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                measure_time_us: s
                    .get("measure_time_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                top_measures,
                root_role: None,
                root_test_id: None,
            }
        })
        .collect();

    out.sort_by(|a, b| b.solve_time_us.cmp(&a.solve_time_us));
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, item.root_node);
        item.root_role = role;
        item.root_test_id = test_id;
    }

    out
}

fn snapshot_model_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsModelChangeHotspot {
            model: h.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_model_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsModelChangeUnobserved {
            model: u.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            created_type: u
                .get("created_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: u
                .get("created_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsGlobalChangeHotspot {
            type_name: h
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsGlobalChangeUnobserved {
            type_name: u
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_lookup_semantics(
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for n in nodes {
        if n.get("id").and_then(|v| v.as_u64()) == Some(node_id) {
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return (role, test_id);
        }
    }
    (None, None)
}

#[derive(Debug, Clone)]
struct SemanticsNodeLite {
    id: u64,
    parent: Option<u64>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default)]
struct SemanticsIndex {
    by_id: std::collections::HashMap<u64, SemanticsNodeLite>,
    best_descendant_with_test_id: std::collections::HashMap<u64, (Option<String>, Option<String>)>,
}

impl SemanticsIndex {
    fn from_snapshot(snapshot: &serde_json::Value) -> Self {
        let nodes = snapshot
            .get("debug")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.get("nodes"))
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut by_id: std::collections::HashMap<u64, SemanticsNodeLite> =
            std::collections::HashMap::new();
        by_id.reserve(nodes.len());

        for n in nodes {
            let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };

            let parent = n.get("parent").and_then(|v| v.as_u64());
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            by_id.insert(
                id,
                SemanticsNodeLite {
                    id,
                    parent,
                    role,
                    test_id,
                },
            );
        }

        let mut best_descendant_with_test_id: std::collections::HashMap<
            u64,
            (Option<String>, Option<String>),
        > = std::collections::HashMap::new();

        for node in by_id.values() {
            let Some(test_id) = node.test_id.as_deref() else {
                continue;
            };
            if test_id.is_empty() {
                continue;
            }

            let mut cursor: Option<u64> = Some(node.id);
            let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    break;
                }

                best_descendant_with_test_id
                    .entry(id)
                    .or_insert_with(|| (node.role.clone(), node.test_id.clone()));

                cursor = by_id.get(&id).and_then(|n| n.parent);
            }
        }

        Self {
            by_id,
            best_descendant_with_test_id,
        }
    }

    fn lookup_for_cache_root(&self, root_node: u64) -> (Option<String>, Option<String>) {
        if let Some(node) = self.by_id.get(&root_node) {
            return (node.role.clone(), node.test_id.clone());
        }

        if let Some((role, test_id)) = self.best_descendant_with_test_id.get(&root_node) {
            return (role.clone(), test_id.clone());
        }

        (None, None)
    }
}

#[derive(Debug, Clone)]
struct ScriptResultSummary {
    run_id: u64,
    stage: Option<String>,
    step_index: Option<u64>,
    reason: Option<String>,
    last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone)]
struct PickResultSummary {
    run_id: u64,
    stage: Option<String>,
    reason: Option<String>,
    last_bundle_dir: Option<String>,
    selector: Option<serde_json::Value>,
}

fn run_script_and_wait(
    src: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ScriptResultSummary, String> {
    let prev_run_id = read_script_result_run_id(script_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    write_script(src, script_path)?;
    touch(script_trigger_path)?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for script result (result: {}, trigger: {})",
                script_result_path.display(),
                script_result_trigger_path.display()
            ));
        }

        if let Some(result) = read_script_result(script_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("passed") | Some("failed")) {
                    let step_index = result.get("step_index").and_then(|v| v.as_u64());
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(ScriptResultSummary {
                        run_id,
                        stage,
                        step_index,
                        reason,
                        last_bundle_dir,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

fn clear_script_result_files(script_result_path: &Path, script_result_trigger_path: &Path) {
    let _ = std::fs::remove_file(script_result_path);
    let _ = std::fs::remove_file(script_result_trigger_path);
}

fn report_result_and_exit(result: &ScriptResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("passed") => {
            println!("PASS (run_id={})", result.run_id);
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason}",
                        result.run_id, step
                    );
                } else {
                    eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
                }
            } else {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id, step
                    );
                } else {
                    eprintln!(
                        "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id
                    );
                }
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected script stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn expected_failure_dump_suffixes(result: &ScriptResultSummary) -> Vec<String> {
    let Some(step_index) = result.step_index else {
        return Vec::new();
    };
    let Some(reason) = result.reason.as_deref() else {
        return Vec::new();
    };

    match reason {
        "wait_until_timeout" => vec![format!("script-step-{step_index:04}-wait_until-timeout")],
        "assert_failed" => vec![format!("script-step-{step_index:04}-assert-failed")],
        "no_semantics_snapshot" => vec![
            format!("script-step-{step_index:04}-wait_until-no-semantics"),
            format!("script-step-{step_index:04}-assert-no-semantics"),
        ],
        _ => Vec::new(),
    }
}

fn wait_for_failure_dump_bundle(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let suffixes = expected_failure_dump_suffixes(result);
    if suffixes.is_empty() {
        return None;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    while Instant::now() < deadline {
        for suffix in &suffixes {
            if let Some(dir) = find_latest_export_dir_with_suffix(out_dir, suffix)
                && dir.join("bundle.json").is_file()
            {
                return Some(dir);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn find_latest_export_dir_with_suffix(out_dir: &Path, suffix: &str) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(suffix) {
            continue;
        }
        let Some((ts_str, _)) = name.split_once('-') else {
            continue;
        };
        let Ok(ts) = ts_str.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn run_pick_and_wait(
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PickResultSummary, String> {
    let prev_run_id = read_pick_result_run_id(pick_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    touch(pick_trigger_path)?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for pick result (result: {}, trigger: {})",
                pick_result_path.display(),
                pick_result_trigger_path.display()
            ));
        }

        if let Some(result) = read_pick_result(pick_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("picked")) {
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let selector = result
                        .get("selection")
                        .and_then(|v| v.get("selectors"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .cloned();

                    return Ok(PickResultSummary {
                        run_id,
                        stage,
                        reason,
                        last_bundle_dir,
                        selector,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

fn report_pick_result_and_exit(result: &PickResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("picked") => {
            if let Some(sel) = result.selector.as_ref() {
                println!("{}", serde_json::to_string(sel).unwrap_or_default());
            } else {
                println!("PICKED (run_id={})", result.run_id);
            }
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
            } else {
                eprintln!(
                    "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id
                );
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected pick stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn write_pick_script(selector: &serde_json::Value, dst: &Path) -> Result<(), String> {
    let script = serde_json::json!({
        "schema_version": 1,
        "steps": [
            { "type": "click", "target": selector },
            { "type": "wait_frames", "frames": 2 },
            { "type": "capture_bundle", "label": "after-picked-click" }
        ]
    });

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

fn apply_pick_to_script(
    src: &Path,
    dst: &Path,
    json_pointer: &str,
    selector: serde_json::Value,
) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let mut script: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    json_pointer_set(&mut script, json_pointer, selector)?;

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

fn json_pointer_set(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    if pointer.is_empty() {
        *root = value;
        return Ok(());
    }
    if !pointer.starts_with('/') {
        return Err(format!(
            "invalid JSON pointer (must start with '/'): {pointer}"
        ));
    }

    let mut tokens: Vec<String> = pointer[1..]
        .split('/')
        .map(unescape_json_pointer_token)
        .collect();
    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }

    let last = tokens
        .pop()
        .ok_or_else(|| "invalid JSON pointer".to_string())?;

    let mut cur: &mut serde_json::Value = root;
    for t in tokens {
        match cur {
            serde_json::Value::Object(map) => {
                let Some(next) = map.get_mut(&t) else {
                    return Err(format!("JSON pointer path does not exist: {pointer}"));
                };
                cur = next;
            }
            serde_json::Value::Array(arr) => {
                let idx = t
                    .parse::<usize>()
                    .map_err(|_| format!("JSON pointer expected array index, got: {t}"))?;
                let Some(next) = arr.get_mut(idx) else {
                    return Err(format!("JSON pointer array index out of bounds: {pointer}"));
                };
                cur = next;
            }
            _ => {
                return Err(format!(
                    "JSON pointer path does not resolve to a container: {pointer}"
                ));
            }
        }
    }

    match cur {
        serde_json::Value::Object(map) => {
            map.insert(last, value);
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }
            let idx = last
                .parse::<usize>()
                .map_err(|_| format!("JSON pointer expected array index, got: {last}"))?;
            if idx < arr.len() {
                arr[idx] = value;
                return Ok(());
            }
            if idx == arr.len() {
                arr.push(value);
                return Ok(());
            }
            Err(format!("JSON pointer array index out of bounds: {pointer}"))
        }
        _ => Err(format!(
            "JSON pointer final target is not a container: {pointer}"
        )),
    }
}

fn unescape_json_pointer_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '~' {
            match it.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                Some(other) => {
                    out.push('~');
                    out.push(other);
                }
                None => out.push('~'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn summarize_times_us(values: &[u64]) -> serde_json::Value {
    if values.is_empty() {
        return serde_json::json!({
            "min": 0,
            "p50": 0,
            "p95": 0,
            "max": 0,
        });
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let min = *sorted.first().unwrap_or(&0);
    let max = *sorted.last().unwrap_or(&0);
    let p50 = percentile_nearest_rank_sorted(&sorted, 0.50);
    let p95 = percentile_nearest_rank_sorted(&sorted, 0.95);

    serde_json::json!({
        "min": min,
        "p50": p50,
        "p95": p95,
        "max": max,
    })
}

fn percentile_nearest_rank_sorted(sorted: &[u64], percentile: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let percentile = percentile.clamp(0.0, 1.0);
    let n = sorted.len();
    let rank_1_based = (percentile * n as f64).ceil().max(1.0) as usize;
    let idx = rank_1_based.saturating_sub(1).min(n - 1);
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stale_scene_check_fails_when_label_changes_without_scene_change() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "hello" }
                            ]}}
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "world" }
                            ]}}
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_stale_scene_json(&bundle, Path::new("bundle.json"), "search", 0.5)
                .unwrap_err();
        assert!(err.contains("stale scene suspected"));
    }

    #[test]
    fn semantics_repaint_check_fails_when_semantics_fingerprint_changes_without_scene_change() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100,
                            "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 101,
                            "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
                .unwrap_err();
        assert!(err.contains("missing repaint suspected"));
    }

    #[test]
    fn semantics_repaint_scan_includes_semantics_diff_detail_when_available() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "role": "textbox", "label": "hello", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                            ]}}
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 101,
                            "debug": {
                                "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 },
                                "semantics": { "nodes": [
                                    { "id": 1, "test_id": "search", "role": "textbox", "label": "world", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                                ]}
                            }
                        }
                    ]
                }
            ]
        });

        let scan = scan_semantics_changed_repainted_json(&bundle, 0);
        assert_eq!(scan.findings.len(), 1);
        assert!(scan.findings[0].get("semantics_diff").is_some());
        assert_eq!(
            scan.findings[0]
                .get("semantics_diff")
                .and_then(|v| v.get("counts"))
                .and_then(|v| v.get("changed"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn semantics_repaint_check_passes_when_scene_fingerprint_changes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 8,
                            "semantics_fingerprint": 101
                        }
                    ]
                }
            ]
        });

        check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
            .unwrap();
    }

    #[test]
    fn bundle_stats_sums_and_sorts_top_by_invalidation_nodes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": { "stats": {
                                "invalidation_walk_calls": 2,
                                "invalidation_walk_nodes": 10,
                                "model_change_invalidation_roots": 1,
                                "global_change_invalidation_roots": 0
                            } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "changed_models": [123],
                            "changed_globals": ["TypeId(0x0)"],
                            "debug": { "stats": {
                                "invalidation_walk_calls": 5,
                                "invalidation_walk_nodes": 7,
                                "model_change_invalidation_roots": 2,
                                "global_change_invalidation_roots": 1
                            } }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();
        assert_eq!(report.windows, 1);
        assert_eq!(report.snapshots, 2);
        assert_eq!(report.snapshots_with_model_changes, 1);
        assert_eq!(report.snapshots_with_global_changes, 1);
        assert_eq!(report.sum_invalidation_walk_calls, 7);
        assert_eq!(report.sum_invalidation_walk_nodes, 17);
        assert_eq!(report.max_invalidation_walk_calls, 5);
        assert_eq!(report.max_invalidation_walk_nodes, 10);
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].invalidation_walk_nodes, 10);
        assert_eq!(report.top[0].tick_id, 1);
    }

    #[test]
    fn bundle_stats_extracts_top_invalidation_walks_with_semantics() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 1,
                                    "invalidation_walk_nodes": 42,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0
                                },
                                "invalidation_walks": [
                                    { "root_node": 42, "kind": "paint", "source": "other", "walked_nodes": 10 },
                                    { "root_node": 43, "kind": "layout", "source": "other", "walked_nodes": 20, "root_element": 9 }
                                ],
                                "semantics": {
                                    "nodes": [
                                        { "id": 43, "role": "button", "test_id": "todo-add" }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].top_invalidation_walks.len(), 2);
        assert_eq!(report.top[0].top_invalidation_walks[0].root_node, 43);
        assert_eq!(
            report.top[0].top_invalidation_walks[0]
                .root_test_id
                .as_deref(),
            Some("todo-add")
        );
        assert_eq!(
            report.top[0].top_invalidation_walks[0].root_role.as_deref(),
            Some("button")
        );
        assert_eq!(
            report.top[0].top_invalidation_walks[0].root_element,
            Some(9)
        );
    }

    #[test]
    fn perf_percentile_nearest_rank_is_stable() {
        let values = vec![10u64, 20, 30, 40, 50, 60, 70];
        let mut sorted = values.clone();
        sorted.sort_unstable();
        assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.50), 40);
        assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.95), 70);
        assert_eq!(
            summarize_times_us(&values),
            json!({"min":10,"p50":40,"p95":70,"max":70})
        );
    }

    #[test]
    fn bundle_stats_tracks_hover_declarative_layout_invalidations() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 1,
                                    "invalidation_walk_nodes": 1,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0,
                                    "hover_declarative_layout_invalidations": 0
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 2,
                                    "invalidation_walk_nodes": 10,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0,
                                    "hover_declarative_layout_invalidations": 2
                                },
                                "hover_declarative_invalidation_hotspots": [
                                    { "node": 43, "layout": 2, "hit_test": 0, "paint": 0 }
                                ],
                                "semantics": {
                                    "nodes": [
                                        { "id": 43, "role": "button", "test_id": "hover-offender" }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();

        assert_eq!(report.sum_hover_layout_invalidations, 2);
        assert_eq!(report.max_hover_layout_invalidations, 2);
        assert_eq!(report.snapshots_with_hover_layout_invalidations, 1);
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].tick_id, 2);
        assert_eq!(report.top[0].hover_declarative_layout_invalidations, 2);
        assert_eq!(report.top[0].top_hover_declarative_invalidations.len(), 1);
        assert_eq!(
            report.top[0].top_hover_declarative_invalidations[0].node,
            43
        );
        assert_eq!(
            report.top[0].top_hover_declarative_invalidations[0]
                .test_id
                .as_deref(),
            Some("hover-offender")
        );
    }

    #[test]
    fn json_pointer_set_updates_object_field() {
        let mut v = json!({
            "steps": [
                { "type": "click", "target": { "kind": "node_id", "node": 1 } }
            ]
        });
        json_pointer_set(
            &mut v,
            "/steps/0/target",
            json!({"kind":"test_id","id":"x"}),
        )
        .unwrap();
        assert_eq!(v["steps"][0]["target"]["kind"], "test_id");
    }

    #[test]
    fn json_pointer_set_updates_predicate_target() {
        let mut v = json!({
            "steps": [
                { "type": "wait_until", "predicate": { "kind": "exists", "target": { "kind": "node_id", "node": 1 } }, "timeout_frames": 10 }
            ]
        });
        json_pointer_set(
            &mut v,
            "/steps/0/predicate/target",
            json!({"kind":"test_id","id":"open"}),
        )
        .unwrap();
        assert_eq!(v["steps"][0]["predicate"]["target"]["id"], "open");
    }

    #[test]
    fn check_bundle_for_view_cache_reuse_min_counts_reused_cache_roots() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 1, "reused": true },
                                    { "root": 2, "reused": false }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 3, "reused": true }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected reuse>=2");
    }

    #[test]
    fn check_bundle_for_view_cache_reuse_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 1, "reused": true }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 2, "reused": true }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 1)
                .expect_err("expected reuse<2 due to warmup");
        assert!(err.contains("expected at least 2 view-cache reuse events"));
        assert!(err.contains("got 1"));
    }

    #[test]
    fn view_cache_reuse_stable_check_passes_when_tail_streak_meets_min() {
        let out_dir = tmp_out_dir("view_cache_reuse_stable_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": false }] } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": true }] } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 5 }, "cache_roots": [] } },
                    { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 3, "reused": true }] } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 3, 0).unwrap();
        assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
    }

    #[test]
    fn view_cache_reuse_stable_check_fails_when_tail_streak_is_too_small() {
        let out_dir = tmp_out_dir("view_cache_reuse_stable_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": true }] } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": false }] } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
        assert!(err.contains("view-cache reuse stable gate failed"));
        assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
    }

    #[test]
    fn check_bundle_for_overlay_synthesis_min_counts_synthesized_events() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "popover", "id": 101, "source": "cached_declaration", "outcome": "synthesized" },
                                    { "kind": "tooltip", "id": 202, "source": "cached_declaration", "outcome": "suppressed_missing_trigger" }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "tooltip", "id": 303, "source": "cached_declaration", "outcome": "synthesized" }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected synthesized>=2");
    }

    #[test]
    fn check_bundle_for_overlay_synthesis_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "tooltip", "id": 1, "source": "cached_declaration", "outcome": "synthesized" }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "hover", "id": 2, "source": "cached_declaration", "outcome": "suppressed_trigger_not_live_in_current_frame" }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 1, 1)
                .expect_err("expected synthesized<1 due to warmup");
        assert!(err.contains("expected at least 1 overlay synthesis events"));
        assert!(err.contains("got 0"));
        assert!(err.contains("suppressions=["));
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_passes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "retained_virtual_list_reconciles": 1 },
                            "dirty_views": [{ "root_node": 1, "source": "notify" }]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "retained_virtual_list_reconciles": 2 },
                            "retained_virtual_list_reconciles": [
                                { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 1, "detached_items": 0 },
                                { "node": 11, "element": 21, "prev_items": 2, "next_items": 3, "preserved_items": 2, "attached_items": 1, "detached_items": 0 }
                            ],
                            "dirty_views": []
                        }
                    }
                ]
            }]
        });

        check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            1,
        )
        .expect("expected reconcile>=1 without notify dirtiness");
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_on_notify_dirty_view() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "debug": {
                        "stats": { "retained_virtual_list_reconciles": 1 },
                        "dirty_views": [
                            { "root_node": 123, "source": "notify", "detail": "notify_call" }
                        ]
                    }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            0,
        )
        .expect_err("expected notify offenders");
        assert!(err.contains(
            "retained virtual-list reconcile should not require notify-based dirty views"
        ));
        assert!(err.contains("source=notify"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_when_missing_reconciles() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            0,
        )
        .expect_err("expected missing reconcile events");
        assert!(err.contains("expected at least 1 retained virtual-list reconcile events"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_passes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "stats": {
                            "retained_virtual_list_reconciles": 1,
                            "retained_virtual_list_attached_items": 12,
                            "retained_virtual_list_detached_items": 13
                        },
                        "retained_virtual_list_reconciles": [
                            { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 12, "detached_items": 13 }
                        ]
                    }
                }]
            }]
        });

        check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            25,
            0,
        )
        .expect("expected delta<=25");
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_exceeded() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "stats": {
                            "retained_virtual_list_reconciles": 1,
                            "retained_virtual_list_attached_items": 20,
                            "retained_virtual_list_detached_items": 21
                        }
                    }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            40,
            0,
        )
        .expect_err("expected delta>40 to fail");
        assert!(err.contains("attach/detach delta exceeded"));
        assert!(err.contains("delta=41"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_missing_reconciles() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            10,
            0,
        )
        .expect_err("expected missing reconcile events");
        assert!(err.contains("expected at least 1 retained virtual-list reconcile event"));
    }

    #[test]
    fn check_bundle_for_viewport_input_min_counts_events() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 2.0, "y": 3.0}, "uv": [0.1, 0.1], "target_px": [10, 10], "kind": { "type": "pointer_move", "buttons": {"left": true, "right": false, "middle": false}, "modifiers": {} } }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected viewport_input>=2");
    }

    #[test]
    fn check_bundle_for_viewport_input_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "viewport_input": []
                            }
                        }
                    ]
                }
            ]
        });

        let err = check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 1, 1)
            .expect_err("expected viewport input < 1 due to warmup");
        assert!(err.contains("expected at least 1 viewport input events"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn check_bundle_for_dock_drag_min_counts_active_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "docking_interaction": {
                                    "dock_drag": { "pointer_id": 0, "source_window": 1, "current_window": 1, "dragging": true, "cross_window_hover": false },
                                    "viewport_capture": null
                                }
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_dock_drag_min_json(&bundle, Path::new("bundle.json"), 1, 0)
            .expect("expected dock_drag>=1");
    }

    #[test]
    fn check_bundle_for_viewport_capture_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "docking_interaction": {
                                    "dock_drag": null,
                                    "viewport_capture": { "pointer_id": 0, "target": 2 }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_viewport_capture_min_json(&bundle, Path::new("bundle.json"), 1, 1)
                .expect_err("expected viewport_capture<1 due to warmup");
        assert!(err.contains("expected at least 1 snapshots with an active viewport capture"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn compare_bundles_passes_when_test_id_semantics_match() {
        let a = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "scene_fingerprint": 42,
                    "debug": {
                        "semantics": {
                            "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                            "nodes": [{
                                "id": 1,
                                "role": "button",
                                "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                                "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                "test_id": "ok"
                            }]
                        }
                    }
                }]
            }]
        });
        let b = a.clone();
        let report = compare_bundles_json(
            &a,
            Path::new("a/bundle.json"),
            &b,
            Path::new("b/bundle.json"),
            CompareOptions {
                warmup_frames: 0,
                eps_px: 0.5,
                ignore_bounds: false,
                ignore_scene_fingerprint: false,
            },
        )
        .unwrap();
        assert!(report.ok);
        assert!(report.diffs.is_empty());
    }

    #[test]
    fn compare_bundles_reports_role_mismatch_for_test_id() {
        let a = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "scene_fingerprint": 42,
                    "debug": {
                        "semantics": {
                            "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                            "nodes": [{
                                "id": 1,
                                "role": "button",
                                "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                                "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                "test_id": "t"
                            }]
                        }
                    }
                }]
            }]
        });
        let mut b = a.clone();
        b["windows"][0]["snapshots"][0]["debug"]["semantics"]["nodes"][0]["role"] =
            serde_json::Value::from("menuitem");

        let report = compare_bundles_json(
            &a,
            Path::new("a/bundle.json"),
            &b,
            Path::new("b/bundle.json"),
            CompareOptions {
                warmup_frames: 0,
                eps_px: 0.5,
                ignore_bounds: false,
                ignore_scene_fingerprint: false,
            },
        )
        .unwrap();
        assert!(!report.ok);
        assert!(report.diffs.iter().any(|d| d.kind == "node_field_mismatch"
            && d.key.as_deref() == Some("t")
            && d.field == Some("role")));
    }

    fn tmp_out_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fretboard_test_{label}_pid{}_{}",
            std::process::id(),
            nanos
        ))
    }

    fn write_png_solid(path: &std::path::Path, w: u32, h: u32, rgba: [u8; 4]) {
        let _ = std::fs::create_dir_all(
            path.parent()
                .expect("png output must have a parent directory"),
        );
        let mut img = image::RgbaImage::new(w, h);
        for p in img.pixels_mut() {
            *p = image::Rgba(rgba);
        }
        img.save(path).expect("png save should succeed");
    }

    fn write_bundle_with_bounds(
        out_dir: &std::path::Path,
        bundle_dir_name: &str,
        window: u64,
        tick_id: u64,
        frame_id: u64,
        test_id: &str,
        bounds: RectF,
    ) {
        let path = out_dir.join(bundle_dir_name).join("bundle.json");
        let _ = std::fs::create_dir_all(
            path.parent()
                .expect("bundle output must have a parent directory"),
        );

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": window,
                "snapshots": [{
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "debug": {
                        "semantics": { "nodes": [{
                            "id": 1,
                            "test_id": test_id,
                            "bounds": { "x": bounds.x, "y": bounds.y, "w": bounds.w, "h": bounds.h }
                        }]}
                    }
                }]
            }]
        });

        std::fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");
    }

    #[test]
    fn idle_no_paint_check_passes_when_tail_streak_meets_min() {
        let out_dir = tmp_out_dir("idle_no_paint_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 3, 0).unwrap();
        assert!(out_dir.join("check.idle_no_paint.json").is_file());
    }

    #[test]
    fn idle_no_paint_check_fails_when_tail_streak_is_too_small() {
        let out_dir = tmp_out_dir("idle_no_paint_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
        assert!(err.contains("idle no-paint gate failed"));
        assert!(out_dir.join("check.idle_no_paint.json").is_file());
    }

    #[test]
    fn pixels_changed_check_passes_when_region_hash_changes() {
        let out_dir = tmp_out_dir("pixels_changed_pass");
        let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

        let window = 1u64;
        let test_id = "root";
        let bounds = RectF {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };

        write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
        write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

        write_png_solid(
            &out_dir.join("screenshots").join("b0").join("shot0.png"),
            10,
            10,
            [0, 0, 0, 255],
        );
        write_png_solid(
            &out_dir.join("screenshots").join("b1").join("shot1.png"),
            10,
            10,
            [255, 0, 0, 255],
        );

        let result = json!({
            "schema_version": 1,
            "completed": [
                { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
                { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
            ]
        });
        std::fs::write(
            out_dir.join("screenshots.result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("screenshots.result.json write should succeed");

        check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap();
        assert!(out_dir.join("check.pixels_changed.json").is_file());
    }

    #[test]
    fn pixels_changed_check_fails_when_region_hash_is_unchanged() {
        let out_dir = tmp_out_dir("pixels_changed_fail");
        let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

        let window = 1u64;
        let test_id = "root";
        let bounds = RectF {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };

        write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
        write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

        write_png_solid(
            &out_dir.join("screenshots").join("b0").join("shot0.png"),
            10,
            10,
            [0, 0, 0, 255],
        );
        write_png_solid(
            &out_dir.join("screenshots").join("b1").join("shot1.png"),
            10,
            10,
            [0, 0, 0, 255],
        );

        let result = json!({
            "schema_version": 1,
            "completed": [
                { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
                { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
            ]
        });
        std::fs::write(
            out_dir.join("screenshots.result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("screenshots.result.json write should succeed");

        let err = check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap_err();
        assert!(err.contains("pixels unchanged suspected"));
        assert!(out_dir.join("check.pixels_changed.json").is_file());
    }

    #[test]
    fn perf_threshold_scan_passes_when_under_limits() {
        let failures = scan_perf_threshold_failures(
            "script.json",
            BundleStatsSort::Time,
            PerfThresholds {
                max_top_total_us: Some(100),
                max_top_layout_us: Some(80),
                max_top_solve_us: Some(50),
            },
            PerfThresholds::default(),
            99,
            79,
            49,
        );
        assert!(failures.is_empty());
    }

    #[test]
    fn perf_threshold_scan_reports_each_exceeded_metric() {
        let failures = scan_perf_threshold_failures(
            "script.json",
            BundleStatsSort::Time,
            PerfThresholds {
                max_top_total_us: Some(100),
                max_top_layout_us: Some(80),
                max_top_solve_us: Some(50),
            },
            PerfThresholds::default(),
            101,
            81,
            51,
        );
        assert_eq!(failures.len(), 3);
        let metrics: Vec<String> = failures
            .iter()
            .filter_map(|v| {
                v.get("metric")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(metrics.contains(&"top_total_time_us".to_string()));
        assert!(metrics.contains(&"top_layout_time_us".to_string()));
        assert!(metrics.contains(&"top_layout_engine_solve_time_us".to_string()));
    }

    #[test]
    fn perf_baseline_headroom_rounds_up() {
        assert_eq!(apply_perf_baseline_headroom(100, 20), 120);
        assert_eq!(apply_perf_baseline_headroom(101, 20), 122);
        assert_eq!(apply_perf_baseline_headroom(0, 20), 0);
    }

    #[test]
    fn perf_baseline_parse_reads_script_thresholds() {
        let out_dir = tmp_out_dir("perf_baseline_parse");
        let _ = std::fs::create_dir_all(&out_dir);
        let path = out_dir.join("perf.baseline.json");

        let v = json!({
            "schema_version": 1,
            "kind": "perf_baseline",
            "rows": [{
                "script": "tools/diag-scripts/ui-gallery-overlay-torture.json",
                "thresholds": {
                    "max_top_total_us": 25000,
                    "max_top_layout_us": 15000,
                    "max_top_solve_us": 8000
                }
            }]
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&v).unwrap())
            .expect("baseline write should succeed");

        let baseline = read_perf_baseline_file(Path::new("."), &path).unwrap();
        let t = baseline
            .thresholds_by_script
            .get("tools/diag-scripts/ui-gallery-overlay-torture.json")
            .copied()
            .unwrap();
        assert_eq!(t.max_top_total_us, Some(25_000));
        assert_eq!(t.max_top_layout_us, Some(15_000));
        assert_eq!(t.max_top_solve_us, Some(8_000));
    }
}
