use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Value, json};

use super::args;

#[derive(Debug, Clone)]
struct ExtensionOccurrence {
    window: u64,
    frame_id: u64,
    window_snapshot_seq: Option<u64>,
    bytes: usize,
    clipped: bool,
    value: Value,
}

pub(crate) fn cmd_extensions(
    rest: &[String],
    resolved_out_dir: &Path,
    workspace_root: &Path,
    warmup_frames: u64,
    stats_json: bool,
    out: Option<&Path>,
) -> Result<(), String> {
    let mut print: bool = false;
    let mut key: Option<String> = None;
    let mut target: Option<String> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--print" => {
                print = true;
                i += 1;
            }
            "--key" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --key".to_string());
                };
                key = Some(v);
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag extensions flag: {other}"));
            }
            _ => {
                if target.is_some() {
                    return Err(format!("unexpected arguments: {}", rest[i..].join(" ")));
                }
                target = Some(rest[i].clone());
                i += 1;
            }
        }
    }

    let bundle_path = args::resolve_bundle_artifact_path_or_latest(
        target.as_deref(),
        workspace_root,
        resolved_out_dir,
    )?;
    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;

    let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let (keys, occurrences) = collect_debug_extensions_from_bundle_value(&bundle, warmup_frames);

    let output_bytes: Vec<u8> = if let Some(key) = key.as_deref() {
        let hits = occurrences.get(key).cloned().unwrap_or_default();
        if hits.is_empty() {
            let mut available: Vec<String> = keys.into_iter().collect();
            available.sort();
            return Err(format!(
                "debug extension not found: {key}\n  bundle_artifact: {}\n  warmup_frames: {warmup_frames}\n  available_keys: {}\n  hint: run without `--key` to list keys, or capture a bundle after the relevant state is exercised",
                bundle_path.display(),
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available.join(", ")
                }
            ));
        }

        if stats_json {
            serde_json::to_vec_pretty(&json!({
                "found": true,
                "key": key,
                "warmup_frames": warmup_frames,
                "bundle_artifact": bundle_path.display().to_string(),
                "bundle_dir": bundle_dir.display().to_string(),
                "occurrences": hits.iter().map(|h| json!({
                    "window": h.window,
                    "frame_id": h.frame_id,
                    "window_snapshot_seq": h.window_snapshot_seq,
                    "bytes": h.bytes,
                    "clipped": h.clipped,
                    "value": h.value,
                })).collect::<Vec<_>>(),
            }))
            .map_err(|e| e.to_string())?
        } else if print {
            let v = if hits.len() == 1 {
                hits[0].value.clone()
            } else {
                json!(
                    hits.iter()
                        .map(|h| json!({
                            "window": h.window,
                            "frame_id": h.frame_id,
                            "window_snapshot_seq": h.window_snapshot_seq,
                            "bytes": h.bytes,
                            "clipped": h.clipped,
                            "value": h.value,
                        }))
                        .collect::<Vec<_>>()
                )
            };
            serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?
        } else {
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "key": key,
                    "occurrences": hits.len(),
                }))
                .map_err(|e| e.to_string())?
            )
            .into_bytes()
        }
    } else if stats_json {
        let mut summary: Vec<Value> = Vec::new();
        let mut sorted_keys: Vec<String> = keys.iter().cloned().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            let hits = occurrences.get(&k).map(|v| v.len()).unwrap_or(0);
            summary.push(json!({
                "key": k,
                "occurrences": hits,
            }));
        }
        serde_json::to_vec_pretty(&json!({
            "bundle_artifact": bundle_path.display().to_string(),
            "bundle_dir": bundle_dir.display().to_string(),
            "warmup_frames": warmup_frames,
            "keys_total": keys.len(),
            "keys": summary,
        }))
        .map_err(|e| e.to_string())?
    } else if print {
        // Printing the full map is acceptable: runtime budgets keep this bounded.
        let mut obj: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut sorted_keys: Vec<String> = keys.iter().cloned().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            let hits = occurrences.get(&k).cloned().unwrap_or_default();
            if hits.len() == 1 {
                obj.insert(k, hits[0].value.clone());
            } else {
                obj.insert(
                    k,
                    json!(
                        hits.iter()
                            .map(|h| json!({
                                "window": h.window,
                                "frame_id": h.frame_id,
                                "window_snapshot_seq": h.window_snapshot_seq,
                                "bytes": h.bytes,
                                "clipped": h.clipped,
                                "value": h.value,
                            }))
                            .collect::<Vec<_>>()
                    ),
                );
            }
        }
        serde_json::to_vec_pretty(&Value::Object(obj)).map_err(|e| e.to_string())?
    } else {
        print_extensions_report(&bundle_path, warmup_frames, &keys, &occurrences).into_bytes()
    };

    if let Some(out) = out {
        let out = crate::resolve_path(workspace_root, out.to_path_buf());
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&out, output_bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }

    print!("{}", String::from_utf8_lossy(&output_bytes));
    Ok(())
}

fn print_extensions_report(
    bundle_path: &Path,
    warmup_frames: u64,
    keys: &BTreeSet<String>,
    occurrences: &BTreeMap<String, Vec<ExtensionOccurrence>>,
) -> String {
    let mut out = String::new();
    out.push_str("extensions:\n");
    out.push_str(&format!("  bundle_artifact: {}\n", bundle_path.display()));
    out.push_str(&format!("  warmup_frames: {warmup_frames}\n"));
    out.push_str(&format!("  keys_total: {}\n", keys.len()));
    if keys.is_empty() {
        out.push_str("  keys: (none)\n");
        return out;
    }

    out.push_str("  keys:\n");
    let mut sorted_keys: Vec<&String> = keys.iter().collect();
    sorted_keys.sort();
    for k in sorted_keys {
        let hits = occurrences.get(k.as_str()).map(|v| v.len()).unwrap_or(0);
        out.push_str(&format!("    - {k} (occurrences={hits})\n"));
    }
    out
}

fn collect_debug_extensions_from_bundle_value(
    bundle: &Value,
    warmup_frames: u64,
) -> (BTreeSet<String>, BTreeMap<String, Vec<ExtensionOccurrence>>) {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    let mut out: BTreeMap<String, Vec<ExtensionOccurrence>> = BTreeMap::new();

    let Some(windows) = bundle.get("windows").and_then(|v| v.as_array()) else {
        return (keys, out);
    };

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());
        let Some(s) = crate::json_bundle::pick_last_snapshot_after_warmup(snaps, warmup_frames)
        else {
            continue;
        };

        let frame_id = crate::json_bundle::snapshot_frame_id(s);
        let window_snapshot_seq = crate::json_bundle::snapshot_window_snapshot_seq(s);
        let Some(ext) = s
            .get("debug")
            .and_then(|d| d.get("extensions"))
            .and_then(|v| v.as_object())
        else {
            continue;
        };

        for (k, v) in ext {
            keys.insert(k.clone());
            let bytes = serde_json::to_vec(v).map(|b| b.len()).unwrap_or(0);
            let clipped = v.get("_clipped").and_then(|v| v.as_bool()).unwrap_or(false);
            out.entry(k.clone()).or_default().push(ExtensionOccurrence {
                window,
                frame_id,
                window_snapshot_seq,
                bytes,
                clipped,
                value: v.clone(),
            });
        }
    }

    (keys, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_keys_from_last_snapshot_after_warmup() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {"frame_id": 0, "debug": {"extensions": {"a.v1": {"x": 1}}}},
                    {"frame_id": 10, "debug": {"extensions": {"b.v1": {"y": 2}, "a.v1": {"x": 3}}}},
                ]
            }]
        });

        let (keys, occ) = collect_debug_extensions_from_bundle_value(&bundle, 5);
        assert!(keys.contains("a.v1"));
        assert!(keys.contains("b.v1"));
        assert_eq!(occ.get("a.v1").unwrap().len(), 1);
        assert_eq!(occ.get("b.v1").unwrap().len(), 1);
        assert_eq!(occ.get("a.v1").unwrap()[0].frame_id, 10);
    }

    #[test]
    fn reports_clipped_marker() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {"frame_id": 1, "debug": {"extensions": {"big.v1": {"_clipped": true, "_max_bytes": 1}}}},
                ]
            }]
        });
        let (_keys, occ) = collect_debug_extensions_from_bundle_value(&bundle, 0);
        assert!(occ.get("big.v1").unwrap()[0].clipped);
    }
}
