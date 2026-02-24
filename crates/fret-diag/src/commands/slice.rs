use std::path::{Path, PathBuf};

use crate::json_bundle::{
    SemanticsResolver, pick_last_snapshot_with_resolved_semantics_after_warmup, snapshot_frame_id,
    snapshot_window_snapshot_seq,
};
use crate::test_id_bloom::TestIdBloomV1;

use super::sidecars;
use super::slice_payload::build_test_id_slice_payload_from_snapshot_and_nodes;
use super::slice_streaming::{
    try_build_test_id_slice_payload_streaming_inline,
    try_build_test_id_slice_payload_streaming_table,
};

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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    struct Picked<'a> {
        window: u64,
        snapshot: &'a serde_json::Value,
    }

    fn snapshot_has_test_id(
        semantics: &SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
        target: &str,
    ) -> bool {
        crate::json_bundle::semantics_node_for_test_id_trimmed(semantics, snapshot, target)
            .is_some()
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
            pick_last_snapshot_with_resolved_semantics_after_warmup(snaps, warmup_frames, semantics)
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
        return Err("bundle contains no windows".to_string());
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

pub(crate) fn build_test_id_slice_payload_from_bundle_path(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    window_id: Option<u64>,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<serde_json::Value, String> {
    let bundle_index = try_read_bundle_index(bundle_path, warmup_frames);

    let index_default = if frame_id.is_none() && window_snapshot_seq.is_none() {
        bundle_index
            .as_ref()
            .and_then(|idx| pick_default_snapshot_in_bundle_index(idx, window_id, Some(test_id)))
    } else {
        None
    };

    let explicit_selector = frame_id.is_some() || window_snapshot_seq.is_some();
    let (
        stream_frame_id,
        stream_window_snapshot_seq,
        stream_window_id,
        mut stream_semantics_source,
        mut stream_semantics_fingerprint,
    ) = if let Some(sel) = &index_default {
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
            sel.semantics_fingerprint,
        )
    } else {
        (frame_id, window_snapshot_seq, window_id, None, None)
    };

    let mut allow_streaming = explicit_selector || index_default.is_some();
    if allow_streaming && explicit_selector {
        if let Some(idx) = bundle_index.as_ref() {
            let m = find_snapshot_in_bundle_index(idx, window_id, frame_id, window_snapshot_seq)
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
                    format!("no matching snapshot found in bundle.index.json (try regenerating it via `fretboard diag index <bundle_dir|bundle.json|bundle.schema2.json>`), args:{hint}")
                })?;
            if !m.has_semantics {
                let source = m.semantics_source.unwrap_or_else(|| "none".to_string());
                return Err(format!(
                    "selected snapshot has no exported semantics (bundle.index.json semantics_source={source}; try a different --frame-id/--snapshot-seq, or ensure semantics export is enabled)"
                ));
            }
            let source = m.semantics_source.as_deref().unwrap_or("none");
            if source != "inline" && source != "table" {
                allow_streaming = false;
            }
            if stream_semantics_source.is_none() {
                stream_semantics_source = m.semantics_source;
            }
            if stream_semantics_fingerprint.is_none() {
                stream_semantics_fingerprint = m.semantics_fingerprint;
            }
        }
    }

    if allow_streaming && !explicit_selector {
        if let Some(source) = stream_semantics_source.as_deref() {
            if source != "inline" && source != "table" {
                allow_streaming = false;
            }
        }
    }

    let build_from_bundle = || -> Result<serde_json::Value, String> {
        let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
        let bundle: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let semantics = SemanticsResolver::new(&bundle);
        Ok(build_test_id_slice_payload_from_bundle(
            bundle_path,
            &bundle,
            &semantics,
            warmup_frames,
            test_id,
            frame_id,
            window_snapshot_seq,
            window_id,
            max_matches,
            max_ancestors,
        )?)
    };

    if allow_streaming {
        let streaming_res = if stream_semantics_source.as_deref() == Some("table") {
            try_build_test_id_slice_payload_streaming_table(
                bundle_path,
                warmup_frames,
                test_id,
                stream_frame_id,
                stream_window_snapshot_seq,
                stream_window_id,
                stream_semantics_fingerprint,
                max_matches,
                max_ancestors,
            )?
        } else {
            try_build_test_id_slice_payload_streaming_inline(
                bundle_path,
                warmup_frames,
                test_id,
                stream_frame_id,
                stream_window_snapshot_seq,
                stream_window_id,
                max_matches,
                max_ancestors,
            )?
        };

        if let Some(payload) = streaming_res {
            let should_fallback_to_find_match = !explicit_selector
                && index_default.is_some()
                && payload
                    .get("matches")
                    .and_then(|v| v.as_array())
                    .is_some_and(|v| v.is_empty());

            if should_fallback_to_find_match {
                if let Some(idx) = bundle_index.as_ref() {
                    const MAX_CANDIDATES: usize = 6;
                    let skip =
                        stream_window_id.map(|w| (w, stream_frame_id, stream_window_snapshot_seq));
                    if let Some(hit) = try_find_hit_payload_by_streaming_candidates(
                        bundle_path,
                        idx,
                        warmup_frames,
                        test_id,
                        window_id,
                        MAX_CANDIDATES,
                        max_matches,
                        max_ancestors,
                        skip,
                    )? {
                        Ok(hit)
                    } else {
                        build_from_bundle()
                    }
                } else {
                    build_from_bundle()
                }
            } else {
                Ok(payload)
            }
        } else {
            build_from_bundle()
        }
    } else {
        build_from_bundle()
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn sanitize_test_id_for_filename(test_id: &str) -> String {
    crate::util::sanitize_for_filename(test_id, 80, "test_id")
}

fn resolve_bundle_artifact_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_artifact_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_artifact_path(&latest))
}

struct BundleIndexSnapshotMatch {
    has_semantics: bool,
    semantics_source: Option<String>,
    semantics_fingerprint: Option<u64>,
}

#[derive(Debug, Clone)]
struct BundleIndexSnapshotSelection {
    window: u64,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    semantics_source: Option<String>,
    semantics_fingerprint: Option<u64>,
}

#[derive(Debug, Clone)]
struct BundleIndexSliceCandidate {
    window: u64,
    window_order: usize,
    selector_frame_id: Option<u64>,
    selector_window_snapshot_seq: Option<u64>,
    semantics_source: Option<String>,
    semantics_fingerprint: Option<u64>,
    is_warmup: bool,
    bloom_might_contain: Option<bool>,
    sort_window_snapshot_seq: Option<u64>,
    sort_frame_id: Option<u64>,
    sort_timestamp_unix_ms: Option<u64>,
}

fn try_read_bundle_index(bundle_path: &Path, warmup_frames: u64) -> Option<serde_json::Value> {
    let index_path = crate::bundle_index::default_bundle_index_path(bundle_path);
    sidecars::try_read_sidecar_json_v1(&index_path, "bundle_index", warmup_frames)
}

fn bundle_index_has_script_markers(v: &serde_json::Value) -> bool {
    v.get("script")
        .and_then(|v| v.get("steps"))
        .and_then(|v| v.as_array())
        .is_some_and(|steps| !steps.is_empty())
}

fn read_bundle_index_for_step_index(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<Option<serde_json::Value>, String> {
    let v = try_read_bundle_index(bundle_path, warmup_frames);
    if v.as_ref().is_some_and(bundle_index_has_script_markers) {
        return Ok(v);
    }
    let index_path = crate::bundle_index::ensure_bundle_index_json(bundle_path, warmup_frames)?;
    Ok(sidecars::try_read_sidecar_json_v1(
        &index_path,
        "bundle_index",
        warmup_frames,
    ))
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
            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
            return Some(BundleIndexSnapshotMatch {
                has_semantics,
                semantics_source,
                semantics_fingerprint,
            });
        }
    }
    None
}

fn pick_default_snapshot_in_bundle_index(
    idx: &serde_json::Value,
    window_id: Option<u64>,
    test_id: Option<&str>,
) -> Option<BundleIndexSnapshotSelection> {
    let semantics_blooms = crate::bundle_index::semantics_bloom_index_from_bundle_index_json(idx);
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
        let mut best_inline_any: Option<BundleIndexSnapshotSelection> = None;
        let mut best_non_warmup_any: Option<BundleIndexSnapshotSelection> = None;
        let mut best_non_warmup_inline: Option<BundleIndexSnapshotSelection> = None;

        let target = test_id.unwrap_or_default().trim().to_string();
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
            let is_inline = semantics_source.as_deref() == Some("inline");
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64());
            let window_snapshot_seq = s.get("window_snapshot_seq").and_then(|v| v.as_u64());
            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());

            let sel = BundleIndexSnapshotSelection {
                window: w_id,
                frame_id,
                window_snapshot_seq,
                semantics_source,
                semantics_fingerprint,
            };

            if !is_warmup {
                best_non_warmup_any.get_or_insert(sel.clone());
                if is_inline {
                    best_non_warmup_inline.get_or_insert(sel.clone());
                }
            }
            if is_inline {
                best_inline_any.get_or_insert(sel.clone());
            }
            best_any.get_or_insert(sel.clone());

            if !target.is_empty() && !is_warmup {
                let mut might_contain: Option<bool> = None;
                if let Some(hex) = s.get("test_id_bloom_hex").and_then(|v| v.as_str()) {
                    might_contain = TestIdBloomV1::from_hex(hex).map(|b| b.might_contain(&target));
                }
                if might_contain.is_none() {
                    if let (Some(fp), Some(src)) =
                        (semantics_fingerprint, sel.semantics_source.as_deref())
                    {
                        let source_tag = if src == "inline" { 0u8 } else { 1u8 };
                        if let Some(b) = semantics_blooms.get(&(w_id, fp, source_tag)) {
                            might_contain = Some(b.might_contain(&target));
                        }
                    }
                }
                if might_contain == Some(true) {
                    return Some(sel);
                }
            }
        }

        if best_non_warmup_inline.is_some() {
            return best_non_warmup_inline;
        }
        if best_non_warmup_any.is_some() {
            return best_non_warmup_any;
        }
        if best_inline_any.is_some() {
            return best_inline_any;
        }
        if best_any.is_some() {
            return best_any;
        }
    }
    None
}

fn slice_candidates_from_bundle_index(
    idx: &serde_json::Value,
    window_id: Option<u64>,
    test_id: &str,
    max_candidates: usize,
) -> Vec<BundleIndexSliceCandidate> {
    let semantics_blooms = crate::bundle_index::semantics_bloom_index_from_bundle_index_json(idx);
    let target = test_id.trim();

    let empty = Vec::new();
    let windows = idx
        .get("windows")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);

    let mut out: Vec<BundleIndexSliceCandidate> = Vec::new();

    for (w_idx, w) in windows.iter().enumerate() {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(req) = window_id
            && req != w_id
        {
            continue;
        }

        let snaps_empty = Vec::new();
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .unwrap_or(&snaps_empty);

        for s in snaps {
            let has_semantics = s
                .get("has_semantics")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !has_semantics {
                continue;
            }

            let semantics_source = s
                .get("semantics_source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let source = semantics_source.as_deref().unwrap_or("none");
            if source != "inline" && source != "table" {
                continue;
            }

            let is_warmup = s
                .get("is_warmup")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let selector_window_snapshot_seq =
                s.get("window_snapshot_seq").and_then(|v| v.as_u64());
            let selector_frame_id = s.get("frame_id").and_then(|v| v.as_u64());

            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());

            let bloom_might_contain = if !target.is_empty() {
                if let Some(hex) = s.get("test_id_bloom_hex").and_then(|v| v.as_str()) {
                    TestIdBloomV1::from_hex(hex).map(|b| b.might_contain(target))
                } else if let (Some(fp), Some(src)) =
                    (semantics_fingerprint, semantics_source.as_deref())
                {
                    let source_tag = if src == "inline" { 0u8 } else { 1u8 };
                    semantics_blooms
                        .get(&(w_id, fp, source_tag))
                        .map(|b| b.might_contain(target))
                } else {
                    None
                }
            } else {
                None
            };

            out.push(BundleIndexSliceCandidate {
                window: w_id,
                window_order: w_idx,
                selector_frame_id,
                selector_window_snapshot_seq,
                semantics_source,
                semantics_fingerprint,
                is_warmup,
                bloom_might_contain,
                sort_window_snapshot_seq: selector_window_snapshot_seq,
                sort_frame_id: selector_frame_id,
                sort_timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
            });
        }
    }

    fn source_rank(s: Option<&str>) -> i32 {
        match s {
            Some("inline") => 2,
            Some("table") => 1,
            _ => 0,
        }
    }

    out.sort_by(|a, b| {
        let a_hit = a.bloom_might_contain.unwrap_or(false);
        let b_hit = b.bloom_might_contain.unwrap_or(false);
        b_hit
            .cmp(&a_hit)
            .then_with(|| a.is_warmup.cmp(&b.is_warmup))
            .then_with(|| {
                source_rank(b.semantics_source.as_deref())
                    .cmp(&source_rank(a.semantics_source.as_deref()))
            })
            .then_with(|| b.sort_window_snapshot_seq.cmp(&a.sort_window_snapshot_seq))
            .then_with(|| b.sort_frame_id.cmp(&a.sort_frame_id))
            .then_with(|| b.sort_timestamp_unix_ms.cmp(&a.sort_timestamp_unix_ms))
            .then_with(|| a.window_order.cmp(&b.window_order))
    });

    if max_candidates > 0 && out.len() > max_candidates {
        out.truncate(max_candidates);
    }

    out
}

fn try_find_hit_payload_by_streaming_candidates(
    bundle_path: &Path,
    idx: &serde_json::Value,
    warmup_frames: u64,
    test_id: &str,
    window_id: Option<u64>,
    max_candidates: usize,
    max_matches: usize,
    max_ancestors: usize,
    skip: Option<(u64, Option<u64>, Option<u64>)>,
) -> Result<Option<serde_json::Value>, String> {
    let candidates = slice_candidates_from_bundle_index(idx, window_id, test_id, max_candidates);

    for c in candidates {
        if let Some((w, fid, seq)) = skip {
            if c.window == w && c.selector_frame_id == fid && c.selector_window_snapshot_seq == seq
            {
                continue;
            }
        }

        let selector_frame_id = if c.selector_window_snapshot_seq.is_some() {
            None
        } else {
            c.selector_frame_id
        };
        let selector_window_snapshot_seq = c.selector_window_snapshot_seq;

        let payload = if c.semantics_source.as_deref() == Some("table") {
            try_build_test_id_slice_payload_streaming_table(
                bundle_path,
                warmup_frames,
                test_id,
                selector_frame_id,
                selector_window_snapshot_seq,
                Some(c.window),
                c.semantics_fingerprint,
                max_matches,
                max_ancestors,
            )?
        } else {
            try_build_test_id_slice_payload_streaming_inline(
                bundle_path,
                warmup_frames,
                test_id,
                selector_frame_id,
                selector_window_snapshot_seq,
                Some(c.window),
                max_matches,
                max_ancestors,
            )?
        };

        let Some(payload) = payload else {
            continue;
        };

        let hit = payload
            .get("matches")
            .and_then(|v| v.as_array())
            .is_some_and(|v| !v.is_empty());
        if hit {
            return Ok(Some(payload));
        }
    }

    Ok(None)
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
    let mut step_index: Option<u32> = None;
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
            "--step-index" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --step-index".to_string());
                };
                step_index =
                    Some(v.parse::<u32>().map_err(|_| {
                        "invalid value for --step-index (expected u32)".to_string()
                    })?);
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

    if step_index.is_some() && (frame_id.is_some() || window_snapshot_seq.is_some()) {
        return Err("--step-index cannot be combined with --frame-id/--snapshot-seq".to_string());
    }

    fn try_read_bundle_index_from_dir(dir: &Path, warmup_frames: u64) -> Option<serde_json::Value> {
        let direct = dir.join("bundle.index.json");
        if direct.is_file() {
            if let Some(v) =
                sidecars::try_read_sidecar_json_v1(&direct, "bundle_index", warmup_frames)
            {
                return Some(v);
            }
        }
        let root = dir.join("_root").join("bundle.index.json");
        if root.is_file() {
            if let Some(v) =
                sidecars::try_read_sidecar_json_v1(&root, "bundle_index", warmup_frames)
            {
                return Some(v);
            }
        }
        None
    }

    fn resolve_step_selector_from_bundle_index(
        idx: &serde_json::Value,
        step_index: u32,
    ) -> Option<(u64, Option<u64>, Option<u64>)> {
        let steps = idx
            .get("script")
            .and_then(|v| v.get("steps"))
            .and_then(|v| v.as_array())?;
        let step = steps.iter().find(|s| {
            s.get("step_index")
                .and_then(|v| v.as_u64())
                .is_some_and(|v| v == step_index as u64)
        })?;
        let window = step.get("window")?.as_u64()?;
        let frame_id = step.get("frame_id").and_then(|v| v.as_u64());
        let window_snapshot_seq = step.get("window_snapshot_seq").and_then(|v| v.as_u64());
        Some((window, frame_id, window_snapshot_seq))
    }

    // Resolve `--step-index` as early as possible so it can validate / select precomputed slices.
    if let (Some(step_index), Some(bundle_arg)) = (step_index, bundle_arg.as_deref()) {
        let src = crate::resolve_path(workspace_root, PathBuf::from(bundle_arg));
        if src.is_dir() {
            let bundle_path = crate::resolve_bundle_artifact_path(&src);
            let idx = if bundle_path.is_file() {
                read_bundle_index_for_step_index(&bundle_path, warmup_frames)
                    .ok()
                    .flatten()
            } else {
                try_read_bundle_index_from_dir(&src, warmup_frames)
            };

            if let Some(idx) = idx
                && let Some((step_window, step_frame_id, step_seq)) =
                    resolve_step_selector_from_bundle_index(&idx, step_index)
            {
                if let Some(req) = window_id
                    && req != step_window
                {
                    return Err(format!(
                        "--step-index {step_index} resolved to window={step_window}, but --window {req} was requested"
                    ));
                }
                window_id = Some(step_window);
                if step_seq.is_some() {
                    window_snapshot_seq = step_seq;
                } else {
                    frame_id = step_frame_id;
                }
            }
        }
    }

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
        resolve_bundle_artifact_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;

    if let Some(step_index) = step_index {
        let bundle_index = read_bundle_index_for_step_index(&bundle_path, warmup_frames)?;
        let Some(idx) = bundle_index else {
            return Err(format!(
                "unable to resolve --step-index {step_index}: missing bundle.index.json"
            ));
        };
        let Some((step_window, step_frame_id, step_seq)) =
            resolve_step_selector_from_bundle_index(&idx, step_index)
        else {
            return Err(format!(
                "bundle.index.json is missing script step markers for step_index={step_index} (tip: run `fretboard diag index <out_dir>/<run_id>` so it can see script.result.json)"
            ));
        };
        if let Some(req) = window_id
            && req != step_window
        {
            return Err(format!(
                "--step-index {step_index} resolved to window={step_window}, but --window {req} was requested"
            ));
        }
        window_id = Some(step_window);
        if step_seq.is_some() {
            window_snapshot_seq = step_seq;
        } else {
            frame_id = step_frame_id;
        }
    }

    let payload = build_test_id_slice_payload_from_bundle_path(
        &bundle_path,
        warmup_frames,
        test_id.as_str(),
        frame_id,
        window_snapshot_seq,
        window_id,
        max_matches,
        max_ancestors,
    )?;

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
    use crate::test_id_bloom::TestIdBloomV1;

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

        let sel = pick_default_snapshot_in_bundle_index(&idx, None, None).expect("selection");
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

        let sel = pick_default_snapshot_in_bundle_index(&idx, Some(2), None).expect("selection");
        assert_eq!(sel.window, 2);
        assert_eq!(sel.frame_id, Some(10));
    }

    #[test]
    fn pick_default_snapshot_prefers_bloom_hit_for_test_id() {
        let mut hit = TestIdBloomV1::new();
        hit.add("target");
        let hit_hex = hit.to_hex();

        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 1, "window_snapshot_seq": 1, "is_warmup": false, "has_semantics": true, "semantics_source": "inline", "test_id_bloom_hex": hit_hex },
                        { "frame_id": 2, "window_snapshot_seq": 2, "is_warmup": false, "has_semantics": true, "semantics_source": "inline", "test_id_bloom_hex": null }
                    ]
                }
            ]
        });

        let sel = pick_default_snapshot_in_bundle_index(&idx, None, Some("target")).expect("sel");
        assert_eq!(sel.frame_id, Some(1));
    }

    #[test]
    fn pick_default_snapshot_uses_semantics_blooms_when_snapshot_bloom_missing() {
        let mut hit = TestIdBloomV1::new();
        hit.add("target");
        let hit_hex = hit.to_hex();

        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "semantics_blooms": {
                "schema_version": 1,
                "m_bits": 1024,
                "k": 4,
                "computed_from": "resolved_semantics_nodes",
                "max_keys_per_window": 2048,
                "keys_total": 1,
                "windows": [
                    { "window": 1, "items": [
                        { "semantics_fingerprint": 42, "semantics_source": "inline", "test_id_bloom_hex": hit_hex }
                    ]}
                ]
            },
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 1, "window_snapshot_seq": 1, "is_warmup": false, "has_semantics": true, "semantics_source": "inline", "semantics_fingerprint": 7, "test_id_bloom_hex": null },
                        { "frame_id": 2, "window_snapshot_seq": 2, "is_warmup": false, "has_semantics": true, "semantics_source": "inline", "semantics_fingerprint": 42, "test_id_bloom_hex": null }
                    ]
                }
            ]
        });

        let sel = pick_default_snapshot_in_bundle_index(&idx, None, Some("target")).expect("sel");
        assert_eq!(sel.frame_id, Some(2));
    }

    #[test]
    fn streaming_candidates_can_find_older_snapshot_with_match_inline() {
        let bundle = r#"{
  "schema_version": 1,
  "windows": [
    {
      "window": 1,
      "snapshots": [
        {
          "frame_id": 1,
          "window_snapshot_seq": 1,
          "timestamp_unix_ms": 100,
          "debug": {
            "semantics": {
              "nodes": [
                { "id": 10, "test_id": "target", "role": "button" }
              ]
            }
          }
        },
        {
          "frame_id": 2,
          "window_snapshot_seq": 2,
          "timestamp_unix_ms": 200,
          "debug": {
            "semantics": {
              "nodes": [
                { "id": 11, "test_id": "other", "role": "button" }
              ]
            }
          }
        }
      ]
    }
  ]
}"#;

        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 1, "window_snapshot_seq": 1, "timestamp_unix_ms": 100, "is_warmup": false, "has_semantics": true, "semantics_source": "inline" },
                        { "frame_id": 2, "window_snapshot_seq": 2, "timestamp_unix_ms": 200, "is_warmup": false, "has_semantics": true, "semantics_source": "inline" }
                    ]
                }
            ]
        });

        let tmp = std::env::temp_dir().join(format!(
            "fret-diag-slice-streaming-candidates-inline-{}.bundle.json",
            crate::util::now_unix_ms()
        ));
        std::fs::write(&tmp, bundle.as_bytes()).unwrap();

        let out = try_find_hit_payload_by_streaming_candidates(
            &tmp, &idx, 0, "target", None, 4, 20, 64, None,
        )
        .unwrap()
        .expect("payload");

        assert_eq!(out.get("frame_id").and_then(|v| v.as_u64()), Some(1));
        assert!(
            out.get("matches")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
        );

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn build_test_id_slice_payload_picks_last_snapshot_with_resolved_semantics() {
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 1, "window_snapshot_seq": 1, "timestamp_unix_ms": 100, "window": 1, "semantics_fingerprint": 1, "debug": {} },
                    { "frame_id": 2, "window_snapshot_seq": 2, "timestamp_unix_ms": 200, "window": 1, "semantics_fingerprint": 2, "debug": { "semantics": null } }
                ]
            }],
            "tables": {
                "semantics": {
                    "schema_version": 1,
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 1,
                        "semantics": { "nodes": [{ "id": 10, "test_id": "target", "role": "button" }] }
                    }, {
                        "window": 1,
                        "semantics_fingerprint": 2,
                        "semantics": { "nodes": [{ "id": 11, "test_id": "target", "role": "button" }] }
                    }]
                }
            }
        });

        let semantics = SemanticsResolver::new(&bundle);

        let payload = build_test_id_slice_payload_from_bundle(
            Path::new("bundle.json"),
            &bundle,
            &semantics,
            0,
            "target",
            None,
            None,
            None,
            10,
            10,
        )
        .expect("expected payload");

        assert_eq!(payload["kind"].as_str(), Some("slice.test_id"));
        assert_eq!(payload["frame_id"].as_u64(), Some(1));
    }

    #[test]
    fn streaming_candidates_can_find_older_snapshot_with_match_table() {
        let bundle = r#"{
  "schema_version": 2,
  "windows": [
    {
      "window": 1,
      "snapshots": [
        {
          "frame_id": 1,
          "window_snapshot_seq": 1,
          "timestamp_unix_ms": 100,
          "semantics_fingerprint": 41,
          "debug": { "stats": { "total_time_us": 1 } }
        },
        {
          "frame_id": 2,
          "window_snapshot_seq": 2,
          "timestamp_unix_ms": 200,
          "semantics_fingerprint": 42,
          "debug": { "stats": { "total_time_us": 1 } }
        }
      ]
    }
  ],
  "tables": {
    "semantics": {
      "entries": [
        {
          "window": 1,
          "semantics_fingerprint": 41,
          "semantics": { "nodes": [ { "id": 10, "test_id": "target" } ] }
        },
        {
          "window": 1,
          "semantics_fingerprint": 42,
          "semantics": { "nodes": [ { "id": 11, "test_id": "other" } ] }
        }
      ]
    }
  }
}"#;

        let idx = serde_json::json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        { "frame_id": 1, "window_snapshot_seq": 1, "timestamp_unix_ms": 100, "is_warmup": false, "has_semantics": true, "semantics_source": "table", "semantics_fingerprint": 41 },
                        { "frame_id": 2, "window_snapshot_seq": 2, "timestamp_unix_ms": 200, "is_warmup": false, "has_semantics": true, "semantics_source": "table", "semantics_fingerprint": 42 }
                    ]
                }
            ]
        });

        let tmp = std::env::temp_dir().join(format!(
            "fret-diag-slice-streaming-candidates-table-{}.bundle.json",
            crate::util::now_unix_ms()
        ));
        std::fs::write(&tmp, bundle.as_bytes()).unwrap();

        let out = try_find_hit_payload_by_streaming_candidates(
            &tmp, &idx, 0, "target", None, 4, 20, 64, None,
        )
        .unwrap()
        .expect("payload");

        assert_eq!(out.get("frame_id").and_then(|v| v.as_u64()), Some(1));
        assert!(
            out.get("matches")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
        );

        let _ = std::fs::remove_file(&tmp);
    }
}
