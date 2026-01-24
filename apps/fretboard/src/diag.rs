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

pub(crate) fn diag_cmd(args: Vec<String>) -> Result<(), String> {
    let mut out_dir: Option<PathBuf> = None;
    let mut trigger_path: Option<PathBuf> = None;
    let mut pack_out: Option<PathBuf> = None;
    let mut pack_include_root_artifacts: bool = false;
    let mut pack_include_triage: bool = false;
    let mut pack_include_screenshots: bool = false;
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
    let mut check_stale_paint_test_id: Option<String> = None;
    let mut check_stale_paint_eps: f32 = 0.5;
    let mut check_wheel_scroll_test_id: Option<String> = None;
    let mut check_hover_layout_max: Option<u32> = None;
    let mut launch: Option<Vec<String>> = None;
    let mut launch_env: Vec<(String, String)> = Vec::new();

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
            "--check-wheel-scroll" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-wheel-scroll".to_string());
                };
                check_wheel_scroll_test_id = Some(v);
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
            "--json" => {
                stats_json = true;
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
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "poke" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            touch(&resolved_trigger_path)?;
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "latest" => {
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
                &artifacts_root,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
                warmup_frames,
            )?;
            println!("{}", out.display());
            Ok(())
        }
        "triage" => {
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

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let mut child = maybe_launch_demo(
                &launch,
                &launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
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
                    || check_wheel_scroll_test_id.is_some()
                    || check_hover_layout_max.is_some()
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
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_wheel_scroll_test_id.as_deref(),
                        check_hover_layout_max,
                        warmup_frames,
                    )?;
                }
            }
            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            report_result_and_exit(&result);
        }
        "suite" => {
            if rest.is_empty() {
                return Err(
                    "missing suite name or script paths (try: fretboard diag suite ui-gallery)"
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
                    "tools/diag-scripts/ui-gallery-hover-layout-torture.json",
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

            let reuse_process = launch.is_none();
            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
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

                if result.stage.as_deref() == Some("passed")
                    && (check_stale_paint_test_id.is_some()
                        || check_wheel_scroll_test_id.is_some()
                        || check_hover_layout_max.is_some())
                {
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
                    apply_post_run_checks(
                        &bundle_path,
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_wheel_scroll_test_id.as_deref(),
                        check_hover_layout_max,
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
            let reuse_process = launch.is_none();
            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };

            let mut perf_json_rows: Vec<serde_json::Value> = Vec::new();
            let mut overall_worst: Option<(u64, PathBuf, PathBuf)> = None;
            let stats_opts = BundleStatsOptions { warmup_frames };

            for src in scripts {
                if !reuse_process {
                    child = maybe_launch_demo(
                        &launch,
                        &launch_env,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
                        &resolved_exit_path,
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

                if let Some(bundle_dir) = bundle_dir {
                    let bundle_path = resolve_bundle_json_path(&resolved_out_dir.join(bundle_dir));
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
                    let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                    let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                    let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                    let top_tick = top.map(|r| r.tick_id).unwrap_or(0);

                    if stats_json {
                        perf_json_rows.push(serde_json::json!({
                            "script": src.display().to_string(),
                            "sort": sort.as_str(),
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_prepaint_time_us": top_prepaint,
                            "top_paint_time_us": top_paint,
                            "top_tick_id": top_tick,
                            "top_frame_id": top_frame,
                            "bundle": bundle_path.display().to_string(),
                        }));
                    } else {
                        println!(
                            "PERF {} sort={} top.us(total/layout/prepaint/paint)={}/{}/{}/{} top.tick={} top.frame={} bundle={}",
                            src.display(),
                            sort.as_str(),
                            top_total,
                            top_layout,
                            top_prepaint,
                            top_paint,
                            top_tick,
                            top_frame,
                            bundle_path.display(),
                        );
                    }

                    match &overall_worst {
                        Some((prev_us, _, _)) if *prev_us >= top_total => {}
                        _ => overall_worst = Some((top_total, src.clone(), bundle_path)),
                    }
                } else {
                    if stats_json {
                        perf_json_rows.push(serde_json::json!({
                            "script": src.display().to_string(),
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
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

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
            if let Some(test_id) = check_wheel_scroll_test_id.as_deref() {
                check_bundle_for_wheel_scroll(bundle_path.as_path(), test_id, warmup_frames)?;
            }
            if let Some(max_allowed) = check_hover_layout_max {
                check_report_for_hover_layout_invalidations(&report, max_allowed)?;
            }
            Ok(())
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

    if include_root_artifacts {
        let root_prefix = format!("{bundle_name}/_root");
        zip_add_root_artifacts(&mut zip, artifacts_root, &root_prefix, options)?;
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
        "script.json",
        "script.result.json",
        "pick.result.json",
        "screenshots.result.json",
        "triage.json",
        "picked.script.json",
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
    if path.is_dir() {
        path.join("bundle.json")
    } else {
        path.to_path_buf()
    }
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

fn apply_post_run_checks(
    bundle_path: &Path,
    check_stale_paint_test_id: Option<&str>,
    check_stale_paint_eps: f32,
    check_wheel_scroll_test_id: Option<&str>,
    check_hover_layout_max: Option<u32>,
    warmup_frames: u64,
) -> Result<(), String> {
    if let Some(test_id) = check_stale_paint_test_id {
        check_bundle_for_stale_paint(bundle_path, test_id, check_stale_paint_eps)?;
    }
    if let Some(test_id) = check_wheel_scroll_test_id {
        check_bundle_for_wheel_scroll(bundle_path, test_id, warmup_frames)?;
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
    Ok(())
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
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Option<Child>, String> {
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

    let child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn `{}`: {e}", launch.join(" ")))?;

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
            return Ok(Some(child));
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    Ok(Some(child))
}

fn kill_launched_demo(child: &mut Option<Child>) {
    let Some(mut child_proc) = child.take() else {
        return;
    };

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

fn stop_launched_demo(child: &mut Option<Child>, exit_path: &Path, poll_ms: u64) {
    if child.is_none() {
        return;
    }

    let _ = touch(exit_path);
    let deadline = Instant::now() + Duration::from_millis(20_000);
    while Instant::now() < deadline {
        let exited = child
            .as_mut()
            .and_then(|c| c.try_wait().ok().flatten())
            .is_some();
        if exited {
            if let Some(mut c) = child.take() {
                let _ = c.wait();
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    kill_launched_demo(child);
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
    cache_replayed_ops: u64,
    top_cache_roots: Vec<BundleStatsCacheRoot>,
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
    reused: bool,
    contained_layout: bool,
    paint_replayed_ops: u32,
    reuse_reason: Option<String>,
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
                "  window={} tick={} frame={} ts={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} cache_roots={} cache.reused={} cache.replayed_ops={} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
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
                        s
                    })
                    .collect();
                println!("    top_cache_roots: {}", items.join(" | "));
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
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
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
            let (cache_roots, cache_roots_reused, cache_replayed_ops, top_cache_roots) =
                snapshot_cache_root_stats(s, 3);
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
                cache_replayed_ops,
                top_cache_roots,
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
) -> (u32, u32, u64, Vec<BundleStatsCacheRoot>) {
    let roots = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if roots.is_empty() {
        return (0, 0, 0, Vec::new());
    }

    let mut reused: u32 = 0;
    let mut replayed_ops_sum: u64 = 0;

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
            replayed_ops_sum = replayed_ops_sum.saturating_add(paint_replayed_ops as u64);

            let (role, test_id) = snapshot_lookup_semantics(snapshot, root_node);
            BundleStatsCacheRoot {
                root_node,
                element: r.get("element").and_then(|v| v.as_u64()),
                reused: reused_flag,
                contained_layout: r
                    .get("contained_layout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                paint_replayed_ops,
                reuse_reason: r
                    .get("reuse_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_role: role,
                root_test_id: test_id,
            }
        })
        .collect();

    out.sort_by(|a, b| b.paint_replayed_ops.cmp(&a.paint_replayed_ops));
    out.truncate(max);

    (
        roots.len().min(u32::MAX as usize) as u32,
        reused,
        replayed_ops_sum,
        out,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
