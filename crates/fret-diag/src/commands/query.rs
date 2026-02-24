use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::args::{looks_like_path, resolve_bundle_artifact_path_or_latest};
use super::sidecars;

use crate::test_id_bloom::TestIdBloomV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryMode {
    Contains,
    Prefix,
    Glob,
}

impl QueryMode {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "contains" => Some(Self::Contains),
            "prefix" => Some(Self::Prefix),
            "glob" => Some(Self::Glob),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Prefix => "prefix",
            Self::Glob => "glob",
        }
    }
}

fn try_read_test_ids_index_json(path: &Path, warmup_frames: u64) -> Option<serde_json::Value> {
    sidecars::try_read_sidecar_json_v1(path, "test_ids_index", warmup_frames)
}

fn try_read_bundle_index_json(path: &Path, warmup_frames: u64) -> Option<serde_json::Value> {
    sidecars::try_read_sidecar_json_v1(path, "bundle_index", warmup_frames)
}

fn bundle_index_has_script_markers(v: &serde_json::Value) -> bool {
    v.get("script")
        .and_then(|v| v.get("steps"))
        .and_then(|v| v.as_array())
        .is_some_and(|steps| !steps.is_empty())
}

fn bundle_index_matches_request(
    v: &serde_json::Value,
    warmup_frames: u64,
    require_script_markers: bool,
) -> bool {
    let _ = warmup_frames;
    // schema_version + warmup_frames are validated by `try_read_bundle_index_json`.
    if require_script_markers && !bundle_index_has_script_markers(v) {
        return false;
    }
    true
}

fn resolve_test_ids_index_from_src(
    src: &Path,
    warmup_frames: u64,
) -> Result<(String, PathBuf, serde_json::Value), String> {
    if src.is_dir() {
        let direct = src.join("test_ids.index.json");
        if direct.is_file() {
            if let Some(v) = try_read_test_ids_index_json(&direct, warmup_frames) {
                let bundle = v
                    .get("bundle")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| direct.display().to_string());
                return Ok((bundle, direct, v));
            }
        }

        let root = src.join("_root").join("test_ids.index.json");
        if root.is_file() {
            if let Some(v) = try_read_test_ids_index_json(&root, warmup_frames) {
                let bundle = v
                    .get("bundle")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| root.display().to_string());
                return Ok((bundle, root, v));
            }
        }

        let bundle_path = crate::resolve_bundle_artifact_path(src);
        let index_path =
            crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
        let v = try_read_test_ids_index_json(&index_path, warmup_frames)
            .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
        return Ok((bundle_path.display().to_string(), index_path, v));
    }

    if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "test_ids.index.json")
    {
        if let Some(v) = try_read_test_ids_index_json(src, warmup_frames) {
            let bundle = v
                .get("bundle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| src.display().to_string());
            return Ok((bundle, src.to_path_buf(), v));
        }

        if let Some(bundle_path) = sidecars::adjacent_bundle_path_for_sidecar(src) {
            let index_path =
                crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
            let v = try_read_test_ids_index_json(&index_path, warmup_frames)
                .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
            let bundle = v
                .get("bundle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| bundle_path.display().to_string());
            return Ok((bundle, index_path, v));
        }

        return Err(format!(
            "invalid test_ids.index.json (expected schema_version=1 warmup_frames={warmup_frames}) and no adjacent bundle artifact was found to regenerate it\n  index: {}",
            src.display()
        ));
    }

    let bundle_path = crate::resolve_bundle_artifact_path(src);
    let index_path = crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
    let v = try_read_test_ids_index_json(&index_path, warmup_frames)
        .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
    Ok((bundle_path.display().to_string(), index_path, v))
}

fn resolve_bundle_index_from_src(
    src: &Path,
    warmup_frames: u64,
    require_script_markers: bool,
) -> Result<(String, PathBuf, serde_json::Value), String> {
    if src.is_dir() {
        let direct = src.join("bundle.index.json");
        if direct.is_file() {
            if let Some(v) = try_read_bundle_index_json(&direct, warmup_frames) {
                if bundle_index_matches_request(&v, warmup_frames, require_script_markers) {
                    let bundle = v
                        .get("bundle")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| direct.display().to_string());
                    return Ok((bundle, direct, v));
                }
            }
        }

        let root = src.join("_root").join("bundle.index.json");
        if root.is_file() {
            if let Some(v) = try_read_bundle_index_json(&root, warmup_frames) {
                if bundle_index_matches_request(&v, warmup_frames, require_script_markers) {
                    let bundle = v
                        .get("bundle")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| root.display().to_string());
                    return Ok((bundle, root, v));
                }
            }
        }

        let bundle_path = crate::resolve_bundle_artifact_path(src);
        let index_path =
            crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
        let v = try_read_bundle_index_json(&index_path, warmup_frames)
            .ok_or_else(|| "invalid bundle.index.json".to_string())?;
        return Ok((bundle_path.display().to_string(), index_path, v));
    }

    if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "bundle.index.json")
    {
        let v = try_read_bundle_index_json(src, warmup_frames)
            .ok_or_else(|| format!("invalid bundle.index.json: {}", src.display()))?;
        if bundle_index_matches_request(&v, warmup_frames, require_script_markers) {
            let bundle = v
                .get("bundle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| src.display().to_string());
            return Ok((bundle, src.to_path_buf(), v));
        }

        // Attempt recovery by regenerating from an adjacent bundle artifact (bundle.json or
        // bundle.schema2.json).
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(parent) = src.parent() {
            candidates.push(parent.to_path_buf());
            if parent.file_name().and_then(|s| s.to_str()) == Some("_root") {
                if let Some(grandparent) = parent.parent() {
                    candidates.push(grandparent.to_path_buf());
                }
            }
        }
        for candidate in candidates {
            let bundle_path = crate::resolve_bundle_artifact_path(&candidate);
            if !bundle_path.is_file() {
                continue;
            }
            let index_path =
                crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
            let v = try_read_bundle_index_json(&index_path, warmup_frames)
                .ok_or_else(|| "invalid bundle.index.json".to_string())?;
            if !bundle_index_matches_request(&v, warmup_frames, require_script_markers) {
                continue;
            }
            let bundle = v
                .get("bundle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| bundle_path.display().to_string());
            return Ok((bundle, index_path, v));
        }

        let mut missing_bits: Vec<&'static str> = Vec::new();
        if v.get("warmup_frames").and_then(|v| v.as_u64()) != Some(warmup_frames) {
            missing_bits.push("warmup_frames mismatch");
        }
        if require_script_markers && !bundle_index_has_script_markers(&v) {
            missing_bits.push("script markers missing");
        }
        let note = if missing_bits.is_empty() {
            "does not match request".to_string()
        } else {
            missing_bits.join(", ")
        };

        return Err(format!(
            "bundle.index.json {note} and no adjacent bundle artifact was found to regenerate it (tip: run `fretboard diag index <bundle_dir|bundle.json|bundle.schema2.json>`)\n  index: {}",
            src.display()
        ));
    }

    let bundle_path = crate::resolve_bundle_artifact_path(src);
    let index_path = crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
    let v = try_read_bundle_index_json(&index_path, warmup_frames)
        .ok_or_else(|| "invalid bundle.index.json".to_string())?;
    Ok((bundle_path.display().to_string(), index_path, v))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_query(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(kind) = rest.first().map(|s| s.as_str()) else {
        return Err("missing query kind (try: fretboard diag query test-id <pattern>)".to_string());
    };

    match kind {
        "test-id" | "test_ids" => cmd_query_test_id(
            &rest[1..],
            workspace_root,
            out_dir,
            query_out,
            warmup_frames,
            stats_json,
        ),
        "snapshots" | "snapshot" => cmd_query_snapshots(
            &rest[1..],
            workspace_root,
            out_dir,
            query_out,
            warmup_frames,
            stats_json,
        ),
        other => Err(format!("unknown query kind: {other}")),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_query_snapshots(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    let mut top: usize = 20;
    let mut window_id: Option<u64> = None;
    let mut non_warmup_only: bool = true;
    let mut has_semantics_only: bool = true;
    let mut semantics_source: Option<String> = None; // inline|table|any (None => any)
    let mut test_id: Option<String> = None;
    let mut step_index: Option<u32> = None;

    let mut positionals: Vec<String> = Vec::new();
    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--top" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --top (expected usize)".to_string())?;
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
            "--include-warmup" => {
                non_warmup_only = false;
                i += 1;
            }
            "--include-missing-semantics" => {
                has_semantics_only = false;
                i += 1;
            }
            "--semantics-source" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --semantics-source".to_string());
                };
                match v.as_str() {
                    "any" => semantics_source = None,
                    "inline" | "table" | "none" => semantics_source = Some(v),
                    _ => {
                        return Err(
                            "invalid value for --semantics-source (expected any|inline|table|none)"
                                .to_string(),
                        );
                    }
                }
                i += 1;
            }
            "--test-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --test-id".to_string());
                };
                test_id = Some(v);
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
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for query snapshots: {other}"));
            }
            other => {
                positionals.push(other.to_string());
                i += 1;
            }
        }
    }

    if positionals.len() > 1 {
        return Err(format!(
            "unexpected arguments: {}",
            positionals[1..].join(" ")
        ));
    }

    let (bundle_label, index_path, index) = if let Some(bundle_src) = positionals.first() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(bundle_src));
        resolve_bundle_index_from_src(&src, warmup_frames, step_index.is_some())?
    } else {
        let bundle_path = resolve_bundle_artifact_path_or_latest(None, workspace_root, out_dir)?;
        let index_path =
            crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
        let index = try_read_bundle_index_json(&index_path, warmup_frames)
            .ok_or_else(|| "invalid bundle.index.json".to_string())?;
        (bundle_path.display().to_string(), index_path, index)
    };

    #[derive(Debug, Clone)]
    struct SnapRow {
        window: u64,
        frame_id: Option<u64>,
        window_snapshot_seq: Option<u64>,
        timestamp_unix_ms: Option<u64>,
        is_warmup: bool,
        semantics_source: Option<String>,
        has_semantics: bool,
        bloom_might_contain: Option<bool>,
    }

    fn best_effort_verify_semantics_in_bundle_index(
        index_path: &Path,
        warmup_frames: u64,
        rows: &[SnapRow],
    ) -> Vec<String> {
        // Keep this cheap and safe: avoid opening very large bundles, and cap the number of verified
        // snapshots. The query command should stay sidecar-first.
        const MAX_VERIFY_BUNDLE_BYTES: u64 = 64 * 1024 * 1024;
        const MAX_VERIFY_ROWS: usize = 10;

        let Some(bundle_path) = sidecars::adjacent_bundle_path_for_sidecar(index_path) else {
            return vec![format!(
                "skipped semantics verification: no adjacent bundle artifact found for index: {}",
                index_path.display()
            )];
        };

        let bundle_bytes = std::fs::metadata(&bundle_path)
            .map(|m| m.len())
            .unwrap_or(0);
        if bundle_bytes > MAX_VERIFY_BUNDLE_BYTES {
            return vec![format!(
                "skipped semantics verification: bundle artifact is large (bytes={} > {})\n  bundle: {}",
                bundle_bytes,
                MAX_VERIFY_BUNDLE_BYTES,
                bundle_path.display()
            )];
        }

        let bytes = match std::fs::read(&bundle_path) {
            Ok(v) => v,
            Err(err) => {
                return vec![format!(
                    "skipped semantics verification: failed to read bundle artifact: {err}\n  bundle: {}",
                    bundle_path.display()
                )];
            }
        };
        let bundle: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(err) => {
                return vec![format!(
                    "skipped semantics verification: failed to parse bundle artifact: {err}\n  bundle: {}",
                    bundle_path.display()
                )];
            }
        };

        let semantics = crate::json_bundle::SemanticsResolver::new(&bundle);

        let windows = bundle
            .get("windows")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut warnings: Vec<String> = Vec::new();
        for r in rows.iter().take(MAX_VERIFY_ROWS) {
            let Some(window_obj) = windows.iter().find(|w| {
                w.get("window")
                    .and_then(|v| v.as_u64())
                    .is_some_and(|w| w == r.window)
            }) else {
                continue;
            };
            let snaps = window_obj
                .get("snapshots")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let snap = if let Some(seq) = r.window_snapshot_seq {
                snaps
                    .iter()
                    .find(|s| crate::json_bundle::snapshot_window_snapshot_seq(s) == Some(seq))
            } else if let Some(fid) = r.frame_id {
                snaps
                    .iter()
                    .find(|s| crate::json_bundle::snapshot_frame_id(s) == fid)
            } else {
                None
            };
            let Some(snap) = snap else {
                continue;
            };

            let actual_has_semantics = semantics.nodes(snap).is_some();
            let actual_source = if crate::json_bundle::snapshot_semantics_nodes(snap).is_some() {
                Some("inline")
            } else if actual_has_semantics {
                Some("table")
            } else {
                Some("none")
            };

            let index_source = r.semantics_source.as_deref().unwrap_or("unknown");
            if r.has_semantics != actual_has_semantics
                || index_source != actual_source.unwrap_or("")
            {
                warnings.push(format!(
                    "bundle.index.json semantics mismatch for window={} frame_id={:?} snapshot_seq={:?} (warmup_frames={warmup_frames})\n  index: has_semantics={} semantics_source={}\n  bundle: has_semantics={} semantics_source={}\n  hint: regenerate sidecars via `fretboard diag index <bundle_dir|bundle.json|bundle.schema2.json> --warmup-frames {warmup_frames}`",
                    r.window,
                    r.frame_id,
                    r.window_snapshot_seq,
                    r.has_semantics,
                    index_source,
                    actual_has_semantics,
                    actual_source.unwrap_or("unknown"),
                ));
            }
            if warnings.len() >= 5 {
                break;
            }
        }

        warnings
    }

    let windows = index
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    let step_selector: Option<(u64, Option<u64>, Option<u64>)> = if let Some(step_index) =
        step_index
    {
        let steps = index
            .get("script")
            .and_then(|v| v.get("steps"))
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let step = steps.iter().find(|s| {
            s.get("step_index")
                .and_then(|v| v.as_u64())
                .is_some_and(|v| v == step_index as u64)
        });
        let Some(step) = step else {
            return Err(format!(
                "bundle.index.json is missing script step markers for step_index={step_index} (tip: run `fretboard diag index <out_dir>/<run_id>` so it can see script.result.json)"
            ));
        };
        let window = step.get("window").and_then(|v| v.as_u64()).ok_or_else(|| {
            "invalid bundle.index.json: script step marker missing window".to_string()
        })?;
        let frame_id = step.get("frame_id").and_then(|v| v.as_u64());
        let window_snapshot_seq = step.get("window_snapshot_seq").and_then(|v| v.as_u64());
        Some((window, frame_id, window_snapshot_seq))
    } else {
        None
    };

    let mut rows: Vec<SnapRow> = Vec::new();
    let target = test_id.as_deref().unwrap_or_default().trim();
    let semantics_blooms =
        crate::bundle_index::semantics_bloom_index_from_bundle_index_json(&index);
    for w in windows {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some((req_window, _, _)) = step_selector
            && req_window != w_id
        {
            continue;
        }
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
            let is_warmup = s
                .get("is_warmup")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if non_warmup_only && is_warmup {
                continue;
            }

            if let Some((_, req_frame_id, req_seq)) = step_selector {
                let got_frame_id = s.get("frame_id").and_then(|v| v.as_u64());
                let got_seq = s.get("window_snapshot_seq").and_then(|v| v.as_u64());
                let matches = if let Some(req_seq) = req_seq {
                    got_seq == Some(req_seq)
                } else if let Some(req_frame_id) = req_frame_id {
                    got_frame_id == Some(req_frame_id)
                } else {
                    false
                };
                if !matches {
                    continue;
                }
            }

            let has_semantics = s
                .get("has_semantics")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if has_semantics_only && !has_semantics {
                continue;
            }

            let src = s
                .get("semantics_source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(req) = semantics_source.as_deref() {
                if src.as_deref() != Some(req) {
                    continue;
                }
            }

            let bloom_might_contain = if !target.is_empty() {
                if let Some(hex) = s.get("test_id_bloom_hex").and_then(|v| v.as_str()) {
                    TestIdBloomV1::from_hex(hex).map(|b| b.might_contain(target))
                } else {
                    let fp = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
                    if let (Some(fp), Some(src)) = (fp, src.as_deref()) {
                        let source_tag = if src == "inline" { 0u8 } else { 1u8 };
                        semantics_blooms
                            .get(&(w_id, fp, source_tag))
                            .map(|b| b.might_contain(target))
                    } else {
                        None
                    }
                }
            } else {
                None
            };

            if !target.is_empty() && bloom_might_contain == Some(false) {
                continue;
            }

            rows.push(SnapRow {
                window: w_id,
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()),
                window_snapshot_seq: s.get("window_snapshot_seq").and_then(|v| v.as_u64()),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                is_warmup,
                semantics_source: src,
                has_semantics,
                bloom_might_contain,
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

    rows.sort_by(|a, b| {
        let a_hit = a.bloom_might_contain.unwrap_or(false);
        let b_hit = b.bloom_might_contain.unwrap_or(false);
        b_hit
            .cmp(&a_hit)
            .then_with(|| {
                source_rank(b.semantics_source.as_deref())
                    .cmp(&source_rank(a.semantics_source.as_deref()))
            })
            .then_with(|| b.window_snapshot_seq.cmp(&a.window_snapshot_seq))
            .then_with(|| b.frame_id.cmp(&a.frame_id))
            .then_with(|| b.timestamp_unix_ms.cmp(&a.timestamp_unix_ms))
            .then_with(|| a.window.cmp(&b.window))
    });

    if top > 0 && rows.len() > top {
        rows.truncate(top);
    }

    let warnings = if stats_json {
        best_effort_verify_semantics_in_bundle_index(&index_path, warmup_frames, &rows)
    } else {
        Vec::new()
    };

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "query.snapshots",
        "bundle": bundle_label,
        "index": index_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "warnings": warnings,
        "top": top,
        "window": window_id,
        "non_warmup_only": non_warmup_only,
        "has_semantics_only": has_semantics_only,
        "semantics_source": semantics_source.as_deref().unwrap_or("any"),
        "test_id": test_id.as_deref(),
        "results": rows.iter().map(|r| serde_json::json!({
            "window": r.window,
            "frame_id": r.frame_id,
            "window_snapshot_seq": r.window_snapshot_seq,
            "timestamp_unix_ms": r.timestamp_unix_ms,
            "is_warmup": r.is_warmup,
            "semantics_source": r.semantics_source,
            "has_semantics": r.has_semantics,
            "bloom_might_contain_test_id": r.bloom_might_contain,
        })).collect::<Vec<_>>(),
    });

    if let Some(out) = query_out.map(|p| crate::resolve_path(workspace_root, p)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
        if !stats_json {
            println!("{}", out.display());
            return Ok(());
        }
    }

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    for r in rows {
        let mut selector = format!("--window {} ", r.window);
        if let Some(seq) = r.window_snapshot_seq {
            selector.push_str(&format!("--snapshot-seq {seq} "));
        } else if let Some(fid) = r.frame_id {
            selector.push_str(&format!("--frame-id {fid} "));
        }
        let src = r.semantics_source.unwrap_or_else(|| "unknown".to_string());
        if let Some(hit) = r.bloom_might_contain {
            println!(
                "{selector}(frame_id={:?} snapshot_seq={:?} warmup={} semantics_source={} bloom_hit={})",
                r.frame_id, r.window_snapshot_seq, r.is_warmup, src, hit
            );
        } else {
            println!(
                "{selector}(frame_id={:?} snapshot_seq={:?} warmup={} semantics_source={})",
                r.frame_id, r.window_snapshot_seq, r.is_warmup, src
            );
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_query_test_id(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    let mut mode: QueryMode = QueryMode::Contains;
    let mut top: usize = 50;
    let mut case_sensitive: bool = false;

    let mut positionals: Vec<String> = Vec::new();
    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--mode" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --mode".to_string());
                };
                mode = QueryMode::from_str(v.as_str()).ok_or_else(|| {
                    "invalid value for --mode (expected contains|prefix|glob)".to_string()
                })?;
                i += 1;
            }
            "--top" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --top (expected usize)".to_string())?;
                i += 1;
            }
            "--case-sensitive" => {
                case_sensitive = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for query test-id: {other}"));
            }
            other => {
                positionals.push(other.to_string());
                i += 1;
            }
        }
    }

    if positionals.is_empty() {
        return Err("missing pattern (try: fretboard diag query test-id <pattern>)".to_string());
    }
    if positionals.len() > 2 {
        return Err(format!(
            "unexpected arguments: {}",
            positionals[2..].join(" ")
        ));
    }

    let (bundle_path, pattern) = match positionals.as_slice() {
        [bundle_src, pattern] => {
            let bundle_src = crate::resolve_path(workspace_root, PathBuf::from(bundle_src));
            (
                crate::resolve_bundle_artifact_path(&bundle_src),
                pattern.to_string(),
            )
        }
        [pattern] => {
            let maybe_path = crate::resolve_path(workspace_root, PathBuf::from(pattern));
            if looks_like_path(pattern) && (maybe_path.is_file() || maybe_path.is_dir()) {
                return Err(
                    "missing pattern (try: fretboard diag query test-id <bundle_dir|bundle.json|bundle.schema2.json> <pattern>)"
                        .to_string(),
                );
            }
            (
                resolve_bundle_artifact_path_or_latest(None, workspace_root, out_dir)?,
                pattern.to_string(),
            )
        }
        _ => unreachable!(),
    };

    let (bundle_label, index_path, index) = if positionals.len() == 2 {
        let src = crate::resolve_path(workspace_root, PathBuf::from(&positionals[0]));
        resolve_test_ids_index_from_src(&src, warmup_frames)?
    } else {
        let index_path =
            crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
        let index = try_read_test_ids_index_json(&index_path, warmup_frames)
            .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
        (bundle_path.display().to_string(), index_path, index)
    };

    let truncated = index
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let max_unique_test_ids_budget = index
        .get("max_unique_test_ids_budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let windows = index
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    #[derive(Debug, Clone)]
    struct Agg {
        total: u64,
        windows_present: u32,
    }

    let mut by_id: HashMap<String, Agg> = HashMap::new();
    for w in windows {
        let items_empty = Vec::new();
        let items = w
            .get("items")
            .and_then(|v| v.as_array())
            .unwrap_or(&items_empty);

        let mut seen_in_window: HashSet<&str> = HashSet::new();
        for it in items {
            let Some(test_id) = it.get("test_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let count = it.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let entry = by_id.entry(test_id.to_string()).or_insert(Agg {
                total: 0,
                windows_present: 0,
            });
            entry.total = entry.total.saturating_add(count);
            seen_in_window.insert(test_id);
        }
        for test_id in seen_in_window {
            let entry = by_id.entry(test_id.to_string()).or_insert(Agg {
                total: 0,
                windows_present: 0,
            });
            entry.windows_present = entry.windows_present.saturating_add(1);
        }
    }

    let pat_norm = if case_sensitive {
        pattern.clone()
    } else {
        pattern.to_lowercase()
    };
    let glob_pat = if mode == QueryMode::Glob {
        Some(glob::Pattern::new(&pattern).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut matches: Vec<(String, Agg)> = by_id
        .into_iter()
        .filter(|(test_id, _agg)| match mode {
            QueryMode::Contains => {
                if case_sensitive {
                    test_id.contains(&pattern)
                } else {
                    test_id.to_lowercase().contains(&pat_norm)
                }
            }
            QueryMode::Prefix => {
                if case_sensitive {
                    test_id.starts_with(&pattern)
                } else {
                    test_id.to_lowercase().starts_with(&pat_norm)
                }
            }
            QueryMode::Glob => glob_pat
                .as_ref()
                .is_some_and(|p| p.matches(test_id.as_str())),
        })
        .collect();

    matches.sort_by(|(a_id, a), (b_id, b)| {
        b.total
            .cmp(&a.total)
            .then_with(|| b.windows_present.cmp(&a.windows_present))
            .then_with(|| a_id.cmp(b_id))
    });
    if top > 0 && matches.len() > top {
        matches.truncate(top);
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "query.test_id",
        "bundle": bundle_label,
        "index": index_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "mode": mode.as_str(),
        "pattern": pattern,
        "case_sensitive": case_sensitive,
        "top": top,
        "truncated_index": truncated,
        "max_unique_test_ids_budget": max_unique_test_ids_budget,
        "results": matches.iter().map(|(test_id, agg)| serde_json::json!({
            "test_id": test_id,
            "count_total": agg.total,
            "windows_present": agg.windows_present,
        })).collect::<Vec<_>>(),
    });

    if let Some(out) = query_out.map(|p| crate::resolve_path(workspace_root, p)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
        if !stats_json {
            println!("{}", out.display());
            return Ok(());
        }
    }

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    if truncated {
        eprintln!(
            "warning: test_ids.index.json truncated at max_unique_test_ids_budget={}",
            max_unique_test_ids_budget
        );
    }
    for (test_id, agg) in matches {
        println!(
            "{test_id} count_total={} windows_present={}",
            agg.total, agg.windows_present
        );
    }
    Ok(())
}
