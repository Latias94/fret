use std::collections::HashSet;
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

struct BundleSemanticsPresence {
    table_keys: HashSet<(u64, u64)>,
}

impl BundleSemanticsPresence {
    fn new(bundle: &serde_json::Value) -> Self {
        let mut table_keys: HashSet<(u64, u64)> = HashSet::new();
        if let Some(entries) = bundle
            .get("tables")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.get("entries"))
            .and_then(|v| v.as_array())
        {
            for e in entries {
                let Some(window) = e.get("window").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(fp) = e.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                    continue;
                };
                table_keys.insert((window, fp));
            }
        }
        Self { table_keys }
    }

    fn snapshot_has_semantics(&self, snapshot: &serde_json::Value, default_window: u64) -> bool {
        if let Some(sem) = crate::json_bundle::snapshot_semantics(snapshot) {
            return !sem.is_null();
        }
        let window = crate::json_bundle::snapshot_window_id(snapshot).unwrap_or(default_window);
        let Some(fp) = crate::json_bundle::snapshot_semantics_fingerprint(snapshot) else {
            return false;
        };
        self.table_keys.contains(&(window, fp))
    }
}

fn apply_semantics_mode_inline(
    windows: &mut [serde_json::Value],
    mode: &str,
    semantics: &BundleSemanticsPresence,
) {

    fn clear_snapshot_semantics(s: &mut serde_json::Value) {
        // Clear any known inline semantics locations so tooling won't accidentally treat legacy
        // keys as semantics when producing a v2 bundle.
        if let Some(debug) = s.get_mut("debug").and_then(|v| v.as_object_mut()) {
            debug.insert("semantics".to_string(), serde_json::Value::Null);
        }
        if let Some(obj) = s.as_object_mut() {
            for k in [
                "semantics",
                "semantic_tree",
                "semanticTree",
                "semantic_tree_v1",
                "tree",
            ] {
                if obj.contains_key(k) {
                    obj.insert(k.to_string(), serde_json::Value::Null);
                }
            }
        }
    }

    match mode {
        "all" => {}
        "off" => {
            for w in windows {
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                for s in snaps {
                    clear_snapshot_semantics(s);
                }
            }
        }
        "last" => {
            for w in windows {
                let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                let mut keep_idx: Option<usize> = None;
                for (idx, s) in snaps.iter().enumerate() {
                    if semantics.snapshot_has_semantics(s, window_id) {
                        keep_idx = Some(idx);
                    }
                }
                for (idx, s) in snaps.iter_mut().enumerate() {
                    if Some(idx) == keep_idx {
                        continue;
                    }
                    clear_snapshot_semantics(s);
                }
            }
        }
        "changed" => {
            for w in windows {
                let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
                let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                let snapshots_len = snaps.len();
                let mut last_kept_fingerprint: Option<u64> = None;
                for (idx, s) in snaps.iter_mut().enumerate() {
                    if !semantics.snapshot_has_semantics(s, window_id) {
                        continue;
                    }
                    let is_last = idx + 1 == snapshots_len;
                    if is_last {
                        last_kept_fingerprint =
                            crate::json_bundle::snapshot_semantics_fingerprint(s);
                        continue;
                    }
                    let fp = crate::json_bundle::snapshot_semantics_fingerprint(s);
                    let keep = match (last_kept_fingerprint, fp) {
                        (None, _) => true,
                        (_, None) => true,
                        (Some(a), Some(b)) => a != b,
                    };
                    if keep {
                        last_kept_fingerprint = fp;
                    } else {
                        clear_snapshot_semantics(s);
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

fn prune_semantics_table(bundle: &mut serde_json::Value, semantics: &BundleSemanticsPresence) {
    let Some(windows) = bundle.get("windows").and_then(|v| v.as_array()) else {
        return;
    };

    let mut referenced: HashSet<(u64, u64)> = HashSet::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());
        for s in snaps {
            let has_semantics = semantics.snapshot_has_semantics(s, window_id);
            if !has_semantics {
                continue;
            }
            let Some(fp) = crate::json_bundle::snapshot_semantics_fingerprint(s) else {
                continue;
            };
            let snap_window = crate::json_bundle::snapshot_window_id(s).unwrap_or(window_id);
            referenced.insert((snap_window, fp));
        }
    }

    let Some(entries) = bundle
        .get_mut("tables")
        .and_then(|v| v.get_mut("semantics"))
        .and_then(|v| v.get_mut("entries"))
        .and_then(|v| v.as_array_mut())
    else {
        return;
    };

    entries.retain(|e| {
        let Some(window) = e.get("window").and_then(|v| v.as_u64()) else {
            return false;
        };
        let Some(fp) = e.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
            return false;
        };
        referenced.contains(&(window, fp))
    });

    if entries.is_empty() {
        if let Some(tables) = bundle.get_mut("tables").and_then(|v| v.as_object_mut()) {
            if let Some(sem) = tables.get_mut("semantics") {
                *sem = serde_json::json!({});
            }
        }
    }
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

            let entries = crate::json_bundle::build_semantics_table_entries_from_windows(windows);
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

    let semantics = BundleSemanticsPresence::new(&bundle);

    let windows_mut = bundle
        .get_mut("windows")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    apply_semantics_mode_inline(windows_mut.as_mut_slice(), mode, &semantics);
    if mode == "off" {
        if let Some(obj) = bundle.as_object_mut() {
            obj.insert("tables".to_string(), serde_json::json!({}));
        }
    } else {
        prune_semantics_table(&mut bundle, &semantics);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn semantics_mode_last_keeps_table_only_semantics_for_last_snapshot() {
        let mut bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 1, "window": 1, "semantics_fingerprint": 10, "debug": {} },
                    { "frame_id": 2, "window": 1, "semantics_fingerprint": 20, "debug": {} }
                ]
            }],
            "tables": {
                "semantics": {
                    "schema_version": 1,
                    "entries": [
                        { "window": 1, "semantics_fingerprint": 10, "semantics": { "nodes": [{ "id": 1 }] } },
                        { "window": 1, "semantics_fingerprint": 20, "semantics": { "nodes": [{ "id": 2 }] } }
                    ]
                }
            }
        });

        let semantics = BundleSemanticsPresence::new(&bundle);
        let windows_mut = bundle
            .get_mut("windows")
            .and_then(|v| v.as_array_mut())
            .expect("windows must be an array");

        apply_semantics_mode_inline(windows_mut.as_mut_slice(), "last", &semantics);
        prune_semantics_table(&mut bundle, &semantics);

        let snaps = bundle["windows"][0]["snapshots"]
            .as_array()
            .expect("snapshots must be an array");
        assert_eq!(
            snaps[0]["debug"]["semantics"].as_null(),
            Some(()),
            "expected non-last snapshot semantics cleared"
        );
        assert!(
            snaps[1]["debug"].get("semantics").is_none(),
            "expected last snapshot to remain table-only"
        );

        let entries = bundle["tables"]["semantics"]["entries"]
            .as_array()
            .expect("entries must be an array");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["semantics_fingerprint"].as_u64(), Some(20));
    }
}
