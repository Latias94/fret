use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

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
    let mut launch: Option<Vec<String>> = None;

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
                pick_apply_out = Some(PathBuf::from(v));
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
            "--json" => {
                stats_json = true;
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
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
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
            kill_launched_demo(&mut child);
            let result = result?;
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
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
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
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
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
                        kill_launched_demo(&mut child);
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
                        kill_launched_demo(&mut child);
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!(
                            "unexpected script stage for {}: {:?}",
                            src.display(),
                            result
                        );
                        kill_launched_demo(&mut child);
                        std::process::exit(1);
                    }
                }

                if !reuse_process {
                    kill_launched_demo(&mut child);
                }
            }

            kill_launched_demo(&mut child);
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
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };

            let mut overall_worst: Option<(u64, PathBuf, PathBuf)> = None;

            for src in scripts {
                if !reuse_process {
                    child = maybe_launch_demo(
                        &launch,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
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
                        kill_launched_demo(&mut child);
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
                        kill_launched_demo(&mut child);
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!(
                            "unexpected script stage for {}: {:?}",
                            src.display(),
                            result
                        );
                        kill_launched_demo(&mut child);
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
                    let report = bundle_stats_from_path(&bundle_path, stats_top.max(1), sort)?;
                    let top = report.top.first();
                    let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                    let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                    let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                    let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                    let top_tick = top.map(|r| r.tick_id).unwrap_or(0);

                    println!(
                        "PERF {} sort={} top.us(total/layout/paint)={}/{}/{} top.tick={} top.frame={} bundle={}",
                        src.display(),
                        sort.as_str(),
                        top_total,
                        top_layout,
                        top_paint,
                        top_tick,
                        top_frame,
                        bundle_path.display(),
                    );

                    match &overall_worst {
                        Some((prev_us, _, _)) if *prev_us >= top_total => {}
                        _ => overall_worst = Some((top_total, src.clone(), bundle_path)),
                    }
                } else {
                    println!(
                        "PERF {} sort={} (no last_bundle_dir recorded)",
                        src.display(),
                        sort.as_str()
                    );
                }

                if !reuse_process {
                    kill_launched_demo(&mut child);
                }
            }

            kill_launched_demo(&mut child);

            if let Some((us, src, bundle)) = overall_worst {
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
            let report = bundle_stats_from_path(
                &bundle_path,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
            )?;

            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report.to_json())
                        .unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                report.print_human(&bundle_path);
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
    workspace_root: &Path,
    out_dir: &Path,
    ready_path: &Path,
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
    let Some(c) = child.as_mut() else {
        return;
    };

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &c.id().to_string(), "/T", "/F"])
            .status();
    }

    #[cfg(not(windows))]
    {
        let _ = c.kill();
    }

    let _ = c.wait();
    *child = None;
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
    windows: u32,
    snapshots: u32,
    snapshots_with_model_changes: u32,
    snapshots_with_global_changes: u32,
    snapshots_with_propagated_model_changes: u32,
    snapshots_with_propagated_global_changes: u32,
    sum_layout_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_invalidation_walk_calls: u64,
    sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    max_layout_time_us: u64,
    max_paint_time_us: u64,
    max_total_time_us: u64,
    max_invalidation_walk_calls: u32,
    max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    top: Vec<BundleStatsSnapshotRow>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsSnapshotRow {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    timestamp_unix_ms: Option<u64>,
    layout_time_us: u64,
    paint_time_us: u64,
    total_time_us: u64,
    layout_nodes_performed: u32,
    paint_nodes_performed: u32,
    paint_cache_misses: u32,
    layout_engine_solves: u64,
    layout_engine_solve_time_us: u64,
    changed_models: u32,
    changed_globals: u32,
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
    model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsInvalidationWalk {
    root_node: u64,
    root_element: Option<u64>,
    kind: Option<String>,
    source: Option<String>,
    walked_nodes: u32,
    truncated_at: Option<u64>,
    root_role: Option<String>,
    root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelChangeHotspot {
    model: u64,
    observation_edges: u32,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelChangeUnobserved {
    model: u64,
    created_type: Option<String>,
    created_at: Option<String>,
}

impl BundleStatsReport {
    fn print_human(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        println!(
            "windows={} snapshots={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} paint={}",
            self.sum_total_time_us, self.sum_layout_time_us, self.sum_paint_time_us
        );
        println!(
            "time max (us): total={} layout={} paint={}",
            self.max_total_time_us, self.max_layout_time_us, self.max_paint_time_us
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
                "  window={} tick={} frame={} ts={} time.us(total/layout/paint)={}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.total_time_us,
                row.layout_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
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
            if !row.model_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .model_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| format!("{}={}", h.model, h.observation_edges))
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
                        s
                    })
                    .collect();
                println!("    unobs_models: {}", items.join(" | "));
            }
        }
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value};

        let mut root = Map::new();
        root.insert("schema_version".to_string(), Value::from(1));
        root.insert("sort".to_string(), Value::from(self.sort.as_str()));
        root.insert("windows".to_string(), Value::from(self.windows));
        root.insert("snapshots".to_string(), Value::from(self.snapshots));
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

        let mut sum = Map::new();
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "paint_time_us".to_string(),
            Value::from(self.sum_paint_time_us),
        );
        sum.insert(
            "total_time_us".to_string(),
            Value::from(self.sum_total_time_us),
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
        root.insert("sum".to_string(), Value::Object(sum));

        let mut max = Map::new();
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
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
        root.insert("max".to_string(), Value::Object(max));

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
                obj.insert(
                    "changed_models".to_string(),
                    Value::from(row.changed_models),
                );
                obj.insert(
                    "changed_globals".to_string(),
                    Value::from(row.changed_globals),
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

                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        root.insert("top".to_string(), Value::Array(top));
        Value::Object(root)
    }
}

fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
) -> Result<BundleStatsReport, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json(&bundle, top, sort)
}

fn bundle_stats_from_json(
    bundle: &serde_json::Value,
    top: usize,
    sort: BundleStatsSort,
) -> Result<BundleStatsReport, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out = BundleStatsReport::default();
    out.sort = sort;
    out.windows = windows.len().min(u32::MAX as usize) as u32;

    let mut rows: Vec<BundleStatsSnapshotRow> = Vec::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            out.snapshots = out.snapshots.saturating_add(1);

            let changed_models = s
                .get("changed_models")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
                .min(u32::MAX as usize) as u32;
            let changed_globals = s
                .get("changed_globals")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
                .min(u32::MAX as usize) as u32;

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
            let paint_time_us = stats
                .and_then(|m| m.get("paint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_time_us = layout_time_us.saturating_add(paint_time_us);
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
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);

            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout_time_us);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint_time_us);
            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total_time_us);
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
            out.max_layout_time_us = out.max_layout_time_us.max(layout_time_us);
            out.max_paint_time_us = out.max_paint_time_us.max(paint_time_us);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                layout_time_us,
                paint_time_us,
                total_time_us,
                layout_nodes_performed,
                paint_nodes_performed,
                paint_cache_misses,
                layout_engine_solves,
                layout_engine_solve_time_us,
                changed_models,
                changed_globals,
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
                model_change_hotspots,
                model_change_unobserved,
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

        let report = bundle_stats_from_json(&bundle, 1, BundleStatsSort::Invalidation).unwrap();
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

        let report = bundle_stats_from_json(&bundle, 1, BundleStatsSort::Invalidation).unwrap();
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
