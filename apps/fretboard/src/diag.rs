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

            let prev_run_id = read_script_result_run_id(&resolved_script_result_path).unwrap_or(0);

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            write_script(&src, &resolved_script_path)?;
            touch(&resolved_script_trigger_path)?;

            let deadline = Instant::now() + Duration::from_millis(timeout_ms);
            loop {
                if Instant::now() >= deadline {
                    eprintln!(
                        "timeout waiting for script result (result: {}, trigger: {})",
                        resolved_script_result_path.display(),
                        resolved_script_result_trigger_path.display()
                    );
                    std::process::exit(1);
                }

                if let Some(result) = read_script_result(&resolved_script_result_path) {
                    let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    if run_id > prev_run_id {
                        let stage = result
                            .get("stage")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        match stage {
                            "passed" => {
                                println!("PASS (run_id={run_id})");
                                std::process::exit(0);
                            }
                            "failed" => {
                                let reason = result
                                    .get("reason")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                let last_bundle_dir = result
                                    .get("last_bundle_dir")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if last_bundle_dir.is_empty() {
                                    eprintln!("FAIL (run_id={run_id}) reason={reason}");
                                } else {
                                    eprintln!(
                                        "FAIL (run_id={run_id}) reason={reason} last_bundle_dir={last_bundle_dir}"
                                    );
                                }
                                std::process::exit(1);
                            }
                            _ => {}
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
            }
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
