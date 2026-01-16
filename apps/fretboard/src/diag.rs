use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(crate) fn diag_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(sub) = it.next() else {
        return Err("missing diag subcommand (try: fretboard diag poke)".to_string());
    };

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
    let mut timeout_ms: u64 = 30_000;
    let mut poll_ms: u64 = 50;

    let mut rest: Vec<String> = it.collect();
    while let Some(arg) = rest.first().cloned() {
        match arg.as_str() {
            "--dir" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --dir".to_string());
                };
                rest.remove(0);
                out_dir = Some(PathBuf::from(v));
            }
            "--trigger-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --trigger-path".to_string());
                };
                rest.remove(0);
                trigger_path = Some(PathBuf::from(v));
            }
            "--script-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --script-path".to_string());
                };
                rest.remove(0);
                script_path = Some(PathBuf::from(v));
            }
            "--script-trigger-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --script-trigger-path".to_string());
                };
                rest.remove(0);
                script_trigger_path = Some(PathBuf::from(v));
            }
            "--script-result-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --script-result-path".to_string());
                };
                rest.remove(0);
                script_result_path = Some(PathBuf::from(v));
            }
            "--script-result-trigger-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --script-result-trigger-path".to_string());
                };
                rest.remove(0);
                script_result_trigger_path = Some(PathBuf::from(v));
            }
            "--pick-trigger-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --pick-trigger-path".to_string());
                };
                rest.remove(0);
                pick_trigger_path = Some(PathBuf::from(v));
            }
            "--pick-result-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --pick-result-path".to_string());
                };
                rest.remove(0);
                pick_result_path = Some(PathBuf::from(v));
            }
            "--pick-result-trigger-path" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --pick-result-trigger-path".to_string());
                };
                rest.remove(0);
                pick_result_trigger_path = Some(PathBuf::from(v));
            }
            "--pick-script-out" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --pick-script-out".to_string());
                };
                rest.remove(0);
                pick_script_out = Some(PathBuf::from(v));
            }
            "--ptr" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --ptr".to_string());
                };
                rest.remove(0);
                pick_apply_pointer = Some(v);
            }
            "--out" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --out".to_string());
                };
                rest.remove(0);
                pick_apply_out = Some(PathBuf::from(v));
            }
            "--timeout-ms" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --timeout-ms".to_string());
                };
                rest.remove(0);
                timeout_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --timeout-ms".to_string())?;
            }
            "--poll-ms" => {
                rest.remove(0);
                let Some(v) = rest.first().cloned() else {
                    return Err("missing value for --poll-ms".to_string());
                };
                rest.remove(0);
                poll_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --poll-ms".to_string())?;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag flag: {other}"));
            }
            _ => break,
        }
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
            let result = run_script_and_wait(
                &src,
                &resolved_script_path,
                &resolved_script_trigger_path,
                &resolved_script_result_path,
                &resolved_script_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;
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
                    "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
                    "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
                    "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
                    "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
                ]
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect()
            } else {
                rest.into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
            };

            for src in scripts {
                let result = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                )?;
                match result.stage.as_deref() {
                    Some("passed") => println!("PASS {} (run_id={})", src.display(), result.run_id),
                    Some("failed") => {
                        eprintln!(
                            "FAIL {} (run_id={}) reason={} last_bundle_dir={}",
                            src.display(),
                            result.run_id,
                            result.reason.as_deref().unwrap_or("unknown"),
                            result.last_bundle_dir.as_deref().unwrap_or("")
                        );
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!(
                            "unexpected script stage for {}: {:?}",
                            src.display(),
                            result
                        );
                        std::process::exit(1);
                    }
                }
            }

            std::process::exit(0);
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

fn resolve_path(workspace_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
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

#[derive(Debug, Clone)]
struct ScriptResultSummary {
    run_id: u64,
    stage: Option<String>,
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
            if run_id > prev_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
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
                    reason,
                    last_bundle_dir,
                });
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
            eprintln!("unexpected script stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn run_pick_and_wait(
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PickResultSummary, String> {
    let prev_run_id = read_pick_result_run_id(pick_result_path).unwrap_or(0);

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
            if run_id > prev_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
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
