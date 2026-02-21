use std::path::{Path, PathBuf};

use crate::json_bundle::{
    SemanticsResolver, pick_last_snapshot_with_semantics_after_warmup, snapshot_frame_id,
    snapshot_window_snapshot_seq,
};

use super::slice_payload::build_test_id_slice_payload_from_snapshot_and_nodes;
use super::slice_streaming::try_build_test_id_slice_payload_streaming_inline;

pub(crate) fn build_test_id_slice_payload_from_bundle(
    bundle_path: &Path,
    bundle: &serde_json::Value,
    semantics: &SemanticsResolver<'_>,
    warmup_frames: u64,
    test_id: &str,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    window_id: Option<u64>,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<serde_json::Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    struct Picked<'a> {
        window: u64,
        snapshot: &'a serde_json::Value,
    }

    fn snapshot_has_test_id(
        semantics: &SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
        target: &str,
    ) -> bool {
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);
        nodes.iter().any(|n| {
            n.get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.trim() == target)
        })
    }

    let mut picked: Option<Picked<'_>> = None;
    for w in windows {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(req) = window_id
            && req != w_id
        {
            continue;
        }
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let snapshot = if let Some(req_frame) = frame_id {
            snaps.iter().find(|s| snapshot_frame_id(s) == req_frame)
        } else if let Some(req_seq) = window_snapshot_seq {
            snaps
                .iter()
                .find(|s| snapshot_window_snapshot_seq(s) == Some(req_seq))
        } else {
            pick_last_snapshot_with_semantics_after_warmup(snaps, warmup_frames)
        };
        let Some(snapshot) = snapshot else {
            continue;
        };
        if semantics.nodes(snapshot).is_none() {
            if frame_id.is_some() || window_snapshot_seq.is_some() {
                return Err("selected snapshot has no exported semantics (try a different --frame-id/--snapshot-seq, or ensure semantics export is enabled)".to_string());
            }
            continue;
        }
        if snapshot_has_test_id(semantics, snapshot, test_id) {
            picked = Some(Picked {
                window: w_id,
                snapshot,
            });
            break;
        }
        if picked.is_none() {
            picked = Some(Picked {
                window: w_id,
                snapshot,
            });
        }
    }

    let Some(picked) = picked else {
        return Err("bundle.json contains no windows".to_string());
    };

    let snapshot = picked.snapshot;
    let nodes = semantics
        .nodes(snapshot)
        .ok_or_else(|| "bundle snapshot missing semantics nodes".to_string())?;

    build_test_id_slice_payload_from_snapshot_and_nodes(
        bundle_path,
        warmup_frames,
        picked.window,
        snapshot,
        nodes,
        test_id,
        max_matches,
        max_ancestors,
    )
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn sanitize_test_id_for_filename(test_id: &str) -> String {
    crate::util::sanitize_for_filename(test_id, 80, "test_id")
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

struct BundleIndexSnapshotMatch {
    has_semantics: bool,
    semantics_source: Option<String>,
}

#[derive(Debug, Clone)]
struct BundleIndexSnapshotSelection {
    window: u64,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    semantics_source: Option<String>,
}

fn try_read_bundle_index(bundle_path: &Path) -> Option<serde_json::Value> {
    let index_path = crate::bundle_index::default_bundle_index_path(bundle_path);
    let bytes = std::fs::read(index_path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return None;
    }
    Some(v)
}

fn find_snapshot_in_bundle_index(
    idx: &serde_json::Value,
    window_id: Option<u64>,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
) -> Option<BundleIndexSnapshotMatch> {
    let windows = idx.get("windows")?.as_array()?;
    for w in windows {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(req) = window_id
            && req != w_id
        {
            continue;
        }
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        for s in snaps {
            let matches = if let Some(req_frame) = frame_id {
                s.get("frame_id").and_then(|v| v.as_u64()) == Some(req_frame)
            } else if let Some(req_seq) = window_snapshot_seq {
                s.get("window_snapshot_seq").and_then(|v| v.as_u64()) == Some(req_seq)
            } else {
                false
            };
            if !matches {
                continue;
            }

            let has_semantics = s
                .get("has_semantics")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let semantics_source = s
                .get("semantics_source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return Some(BundleIndexSnapshotMatch {
                has_semantics,
                semantics_source,
            });
        }
    }
    None
}

fn pick_default_snapshot_in_bundle_index(
    idx: &serde_json::Value,
    window_id: Option<u64>,
) -> Option<BundleIndexSnapshotSelection> {
    let windows = idx.get("windows")?.as_array()?;
    for w in windows {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(req) = window_id
            && req != w_id
        {
            continue;
        }

        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut best_any: Option<BundleIndexSnapshotSelection> = None;
        for s in snaps.iter().rev() {
            let has_semantics = s
                .get("has_semantics")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !has_semantics {
                continue;
            }

            let is_warmup = s
                .get("is_warmup")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let semantics_source = s
                .get("semantics_source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64());
            let window_snapshot_seq = s.get("window_snapshot_seq").and_then(|v| v.as_u64());

            let sel = BundleIndexSnapshotSelection {
                window: w_id,
                frame_id,
                window_snapshot_seq,
                semantics_source,
            };

            if !is_warmup {
                return Some(sel);
            }
            best_any.get_or_insert(sel);
        }

        if best_any.is_some() {
            return best_any;
        }
    }
    None
}

fn try_read_slice_payload(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("slice.test_id") {
        return None;
    }
    Some(v)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_slice(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    slice_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut test_id: Option<String> = None;
    let mut frame_id: Option<u64> = None;
    let mut window_snapshot_seq: Option<u64> = None;
    let mut window_id: Option<u64> = None;
    let mut max_matches: usize = 20;
    let mut max_ancestors: usize = 64;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--test-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --test-id".to_string());
                };
                test_id = Some(v);
                i += 1;
            }
            "--frame-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --frame-id".to_string());
                };
                frame_id = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --frame-id (expected u64)".to_string())?,
                );
                i += 1;
            }
            "--snapshot-seq" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --snapshot-seq".to_string());
                };
                window_snapshot_seq =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --snapshot-seq (expected u64)".to_string()
                    })?);
                i += 1;
            }
            "--window" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --window".to_string());
                };
                window_id = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --window (expected u64)".to_string())?,
                );
                i += 1;
            }
            "--max-matches" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --max-matches".to_string());
                };
                max_matches = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --max-matches (expected usize)".to_string())?;
                i += 1;
            }
            "--max-ancestors" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --max-ancestors".to_string());
                };
                max_ancestors = v.parse::<usize>().map_err(|_| {
                    "invalid value for --max-ancestors (expected usize)".to_string()
                })?;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for slice: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    // allow positional bundle arg if it exists as a path
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else if test_id.is_none() {
                        test_id = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else if test_id.is_none() {
                    test_id = Some(other.to_string());
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let test_id = test_id.ok_or_else(|| {
        "missing --test-id (try: fretboard diag slice --test-id <test_id>)".to_string()
    })?;

    if let Some(bundle_arg) = bundle_arg.as_deref() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(bundle_arg));
        if src.is_dir() {
            let stem = crate::util::sanitize_for_filename(test_id.as_str(), 80, "test_id");
            let candidates = [
                src.join(format!("slice.test_id.{stem}.json")),
                src.join(format!("slice.{stem}.json")),
            ];

            let found = candidates
                .iter()
                .find(|p| p.is_file())
                .and_then(|p| try_read_slice_payload(p).map(|v| (p.to_path_buf(), v)));

            if let Some((slice_path, payload)) = found {
                let req_window = window_id;
                let req_frame = frame_id;
                let req_seq = window_snapshot_seq;

                if req_window.is_some() || req_frame.is_some() || req_seq.is_some() {
                    let got_window = payload.get("window").and_then(|v| v.as_u64());
                    let got_frame = payload.get("frame_id").and_then(|v| v.as_u64());
                    let got_seq = payload.get("window_snapshot_seq").and_then(|v| v.as_u64());

                    if req_window.is_some_and(|w| Some(w) != got_window)
                        || req_frame.is_some_and(|f| Some(f) != got_frame)
                        || req_seq.is_some_and(|s| Some(s) != got_seq)
                    {
                        return Err(format!(
                            "found precomputed slice, but it doesn't match the requested selection (requested window={req_window:?} frame_id={req_frame:?} snapshot_seq={req_seq:?}; slice has window={got_window:?} frame_id={got_frame:?} snapshot_seq={got_seq:?})"
                        ));
                    }
                }

                let out = slice_out
                    .map(|p| crate::resolve_path(workspace_root, p))
                    .unwrap_or_else(|| slice_path.clone());

                if out != slice_path {
                    if let Some(parent) = out.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let pretty =
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
                }

                if stats_json {
                    println!(
                        "{}",
                        std::fs::read_to_string(&out).map_err(|e| e.to_string())?
                    );
                } else {
                    println!("{}", out.display());
                }
                return Ok(());
            }
        }
    }

    let bundle_path =
        resolve_bundle_json_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;

    let bundle_index = try_read_bundle_index(&bundle_path);

    let index_default = if frame_id.is_none() && window_snapshot_seq.is_none() {
        bundle_index
            .as_ref()
            .and_then(|idx| pick_default_snapshot_in_bundle_index(idx, window_id))
    } else {
        None
    };

    let explicit_selector = frame_id.is_some() || window_snapshot_seq.is_some();
    let (stream_frame_id, stream_window_snapshot_seq, stream_window_id, stream_semantics_source) =
        if let Some(sel) = &index_default {
            let (frame_id, window_snapshot_seq) = if sel.window_snapshot_seq.is_some() {
                (None, sel.window_snapshot_seq)
            } else {
                (sel.frame_id, None)
            };

            (
                frame_id,
                window_snapshot_seq,
                Some(sel.window),
                sel.semantics_source.clone(),
            )
        } else {
            (frame_id, window_snapshot_seq, window_id, None)
        };

    let mut allow_streaming = explicit_selector || index_default.is_some();
    if allow_streaming && explicit_selector {
        if let Some(idx) = bundle_index.as_ref() {
            let m = find_snapshot_in_bundle_index(&idx, window_id, frame_id, window_snapshot_seq)
                .ok_or_else(|| {
                    let mut hint = String::new();
                    if let Some(w) = window_id {
                        hint.push_str(&format!(" --window {w}"));
                    }
                    if let Some(f) = frame_id {
                        hint.push_str(&format!(" --frame-id {f}"));
                    }
                    if let Some(s) = window_snapshot_seq {
                        hint.push_str(&format!(" --snapshot-seq {s}"));
                    }
                    format!("no matching snapshot found in bundle.index.json (try regenerating it via `fretboard diag index <bundle.json>`), args:{hint}")
                })?;
            if !m.has_semantics {
                let source = m.semantics_source.unwrap_or_else(|| "none".to_string());
                return Err(format!(
                    "selected snapshot has no exported semantics (bundle.index.json semantics_source={source}; try a different --frame-id/--snapshot-seq, or ensure semantics export is enabled)"
                ));
            }
            if let Some(source) = m.semantics_source.as_deref() {
                if source != "inline" {
                    allow_streaming = false;
                }
            }
        }
    }

    if allow_streaming && !explicit_selector {
        if let Some(source) = stream_semantics_source.as_deref() {
            if source != "inline" {
                allow_streaming = false;
            }
        }
    }

    let build_from_bundle = || -> Result<serde_json::Value, String> {
        let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
        let bundle: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let semantics = SemanticsResolver::new(&bundle);
        Ok(build_test_id_slice_payload_from_bundle(
            &bundle_path,
            &bundle,
            &semantics,
            warmup_frames,
            &test_id,
            frame_id,
            window_snapshot_seq,
            window_id,
            max_matches,
            max_ancestors,
        )?)
    };

    let payload = if allow_streaming {
        if let Some(payload) = try_build_test_id_slice_payload_streaming_inline(
            &bundle_path,
            warmup_frames,
            test_id.as_str(),
            stream_frame_id,
            stream_window_snapshot_seq,
            stream_window_id,
            max_matches,
            max_ancestors,
        )? {
            let should_fallback_to_find_match = !explicit_selector
                && index_default.is_some()
                && payload
                    .get("matches")
                    .and_then(|v| v.as_array())
                    .is_some_and(|v| v.is_empty());

            if should_fallback_to_find_match {
                build_from_bundle()?
            } else {
                payload
            }
        } else {
            build_from_bundle()?
        }
    } else {
        build_from_bundle()?
    };

    let out = slice_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| {
            let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
            dir.join(format!(
                "slice.{}.json",
                sanitize_test_id_for_filename(test_id.as_str())
            ))
        });

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_default_snapshot_prefers_last_non_warmup_with_semantics() {
        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 1, "window_snapshot_seq": 1, "is_warmup": true, "has_semantics": true, "semantics_source": "inline" },
                        { "frame_id": 2, "window_snapshot_seq": 2, "is_warmup": false, "has_semantics": false, "semantics_source": "none" },
                        { "frame_id": 3, "window_snapshot_seq": 3, "is_warmup": false, "has_semantics": true, "semantics_source": "inline" }
                    ]
                }
            ]
        });

        let sel = pick_default_snapshot_in_bundle_index(&idx, None).expect("selection");
        assert_eq!(sel.window, 1);
        assert_eq!(sel.frame_id, Some(3));
        assert_eq!(sel.window_snapshot_seq, Some(3));
        assert_eq!(sel.semantics_source.as_deref(), Some("inline"));
    }

    #[test]
    fn pick_default_snapshot_respects_window_filter() {
        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 9, "is_warmup": false, "has_semantics": true, "semantics_source": "inline" }
                    ]
                },
                {
                    "window": 2,
                    "snapshots": [
                        { "frame_id": 10, "is_warmup": false, "has_semantics": true, "semantics_source": "inline" }
                    ]
                }
            ]
        });

        let sel = pick_default_snapshot_in_bundle_index(&idx, Some(2)).expect("selection");
        assert_eq!(sel.window, 2);
        assert_eq!(sel.frame_id, Some(10));
    }
}
