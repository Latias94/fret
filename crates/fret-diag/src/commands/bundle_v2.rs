use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn resolve_bundle_json_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_json_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

fn parse_semantics_mode(mode: &str) -> Result<&'static str, String> {
    let m = mode.trim().to_ascii_lowercase();
    match m.as_str() {
        "all" => Ok("all"),
        "changed" => Ok("changed"),
        "last" => Ok("last"),
        "off" | "none" => Ok("off"),
        _ => Err("invalid value for --mode (expected all|changed|last|off)".to_string()),
    }
}

fn apply_semantics_mode_inline(windows: &mut [serde_json::Value], mode: &str) {
    match mode {
        "all" => {}
        "off" => {
            for w in windows {
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                for s in snaps {
                    if let Some(debug) = s.get_mut("debug")
                        && let Some(obj) = debug.as_object_mut()
                    {
                        obj.insert("semantics".to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
        "last" => {
            for w in windows {
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                let mut keep_idx: Option<usize> = None;
                for (idx, s) in snaps.iter().enumerate() {
                    if s.get("debug")
                        .and_then(|v| v.get("semantics"))
                        .is_some_and(|v| !v.is_null())
                    {
                        keep_idx = Some(idx);
                    }
                }
                for (idx, s) in snaps.iter_mut().enumerate() {
                    if Some(idx) == keep_idx {
                        continue;
                    }
                    if let Some(debug) = s.get_mut("debug")
                        && let Some(obj) = debug.as_object_mut()
                    {
                        obj.insert("semantics".to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
        "changed" => {
            for w in windows {
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                let snapshots_len = snaps.len();
                let mut last_kept_fingerprint: Option<u64> = None;
                for (idx, s) in snaps.iter_mut().enumerate() {
                    let has_semantics = s
                        .get("debug")
                        .and_then(|v| v.get("semantics"))
                        .is_some_and(|v| !v.is_null());
                    if !has_semantics {
                        continue;
                    }
                    let is_last = idx + 1 == snapshots_len;
                    if is_last {
                        last_kept_fingerprint =
                            s.get("semantics_fingerprint").and_then(|v| v.as_u64());
                        continue;
                    }
                    let fp = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
                    let keep = match (last_kept_fingerprint, fp) {
                        (None, _) => true,
                        (_, None) => true,
                        (Some(a), Some(b)) => a != b,
                    };
                    if keep {
                        last_kept_fingerprint = fp;
                    } else if let Some(debug) = s.get_mut("debug")
                        && let Some(obj) = debug.as_object_mut()
                    {
                        obj.insert("semantics".to_string(), serde_json::Value::Null);
                    }
                }
            }
        }
        _ => {}
    }
}

fn schema_version(v: &serde_json::Value) -> Option<u64> {
    v.get("schema_version").and_then(|v| v.as_u64())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_bundle_v2(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    bundle_v2_out: Option<PathBuf>,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut mode: &'static str = "last";
    let mut pretty: bool = false;
    let mut force: bool = false;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--mode" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --mode".to_string());
                };
                mode = parse_semantics_mode(v.as_str())?;
                i += 1;
            }
            "--pretty" => {
                pretty = true;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for bundle-v2: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let bundle_path =
        resolve_bundle_json_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;
    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;

    let out = bundle_v2_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| bundle_dir.join("bundle.schema2.json"));

    let file_bytes = std::fs::metadata(&bundle_path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;

    const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
    if !force && file_bytes > DEFAULT_MAX_FILE_BYTES {
        return Err(format!(
            "bundle.json is too large to analyze safely by default (size={} > {}); re-run with --force",
            file_bytes, DEFAULT_MAX_FILE_BYTES
        ));
    }

    let t_parse = Instant::now();
    let file = std::fs::File::open(&bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut bundle: serde_json::Value =
        serde_json::from_reader(reader).map_err(|e| e.to_string())?;
    let parse_ms = t_parse.elapsed().as_millis() as u64;

    let input_schema_version = schema_version(&bundle).unwrap_or(0);
    if input_schema_version != 1 && input_schema_version != 2 {
        return Err(format!(
            "unsupported bundle schema_version={input_schema_version} (expected 1 or 2)"
        ));
    }

    let t_convert = Instant::now();

    if input_schema_version == 1
        && let Some(obj) = bundle.as_object_mut()
    {
        obj.insert("schema_version".to_string(), serde_json::Value::from(2u64));
    }

    if mode == "off" {
        if let Some(obj) = bundle.as_object_mut() {
            obj.insert("tables".to_string(), serde_json::json!({}));
        }
    } else {
        let has_semantics_table = bundle
            .get("tables")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.get("entries"))
            .and_then(|v| v.as_array())
            .is_some_and(|v| !v.is_empty());

        if !has_semantics_table {
            let windows = bundle
                .get("windows")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

            let mut table: BTreeMap<(u64, u64), serde_json::Value> = BTreeMap::new();
            for w in windows {
                let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
                let snaps = w
                    .get("snapshots")
                    .and_then(|v| v.as_array())
                    .map_or(&[][..], |v| v.as_slice());
                for s in snaps {
                    let Some(sem) = s.get("debug").and_then(|v| v.get("semantics")) else {
                        continue;
                    };
                    if sem.is_null() {
                        continue;
                    }
                    let Some(fp) = s.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let snap_window = s
                        .get("window")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(window_id);
                    table
                        .entry((snap_window, fp))
                        .or_insert_with(|| sem.clone());
                }
            }

            let entries: Vec<serde_json::Value> = table
                .into_iter()
                .map(|((window, fp), semantics)| {
                    serde_json::json!({
                        "window": window,
                        "semantics_fingerprint": fp,
                        "semantics": semantics,
                    })
                })
                .collect();
            let tables = serde_json::json!({
                "semantics": {
                    "schema_version": 1,
                    "entries": entries,
                }
            });
            if let Some(obj) = bundle.as_object_mut() {
                obj.insert("tables".to_string(), tables);
            }
        }
    }

    let windows_mut = bundle
        .get_mut("windows")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    apply_semantics_mode_inline(windows_mut.as_mut_slice(), mode);

    let convert_ms = t_convert.elapsed().as_millis() as u64;

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let bytes = if pretty {
        serde_json::to_vec_pretty(&bundle).map_err(|e| e.to_string())?
    } else {
        serde_json::to_vec(&bundle).map_err(|e| e.to_string())?
    };
    std::fs::write(&out, &bytes).map_err(|e| e.to_string())?;

    let payload = serde_json::json!({
        "kind": "bundle.convert_v2",
        "schema_version": 1,
        "input": bundle_path.display().to_string(),
        "input_schema_version": input_schema_version,
        "output": out.display().to_string(),
        "output_schema_version": 2,
        "mode": mode,
        "pretty": pretty,
        "timing_ms": {
            "parse": parse_ms,
            "convert": convert_ms,
        },
        "output_bytes": bytes.len(),
    });

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
    } else {
        println!("{}", out.display());
    }

    Ok(())
}
