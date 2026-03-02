use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::args::{looks_like_path, resolve_bundle_artifact_path_or_latest};
use super::resolve;
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
        "overlay-placement-trace"
        | "overlay_placement_trace"
        | "overlay-placement"
        | "overlay_placement" => cmd_query_overlay_placement_trace(
            &rest[1..],
            workspace_root,
            out_dir,
            query_out,
            stats_json,
        ),
        other => Err(format!("unknown query kind: {other}")),
    }
}

fn find_nearest_script_result_json_preferring_evidence(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    let mut first_found: Option<PathBuf> = None;
    for _ in 0..10 {
        let Some(dir) = cur else { break };
        let direct = dir.join("script.result.json");
        if direct.is_file() {
            if first_found.is_none() {
                first_found = Some(direct.clone());
            }
            if let Some(result) = read_script_result_typed(&direct)
                && result.evidence.is_some()
            {
                return Some(direct);
            }
        }
        cur = dir.parent();
    }
    first_found
}

fn read_script_result_typed(path: &Path) -> Option<fret_diag_protocol::UiScriptResultV1> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice::<fret_diag_protocol::UiScriptResultV1>(&bytes).ok()
}

fn parse_overlay_side(s: &str) -> Option<fret_diag_protocol::UiOverlaySideV1> {
    match s.trim().to_ascii_lowercase().as_str() {
        "top" => Some(fret_diag_protocol::UiOverlaySideV1::Top),
        "bottom" => Some(fret_diag_protocol::UiOverlaySideV1::Bottom),
        "left" => Some(fret_diag_protocol::UiOverlaySideV1::Left),
        "right" => Some(fret_diag_protocol::UiOverlaySideV1::Right),
        _ => None,
    }
}

fn parse_overlay_align(s: &str) -> Option<fret_diag_protocol::UiOverlayAlignV1> {
    match s.trim().to_ascii_lowercase().as_str() {
        "start" => Some(fret_diag_protocol::UiOverlayAlignV1::Start),
        "center" => Some(fret_diag_protocol::UiOverlayAlignV1::Center),
        "end" => Some(fret_diag_protocol::UiOverlayAlignV1::End),
        _ => None,
    }
}

fn parse_overlay_sticky(s: &str) -> Option<fret_diag_protocol::UiOverlayStickyModeV1> {
    match s.trim().to_ascii_lowercase().as_str() {
        "partial" => Some(fret_diag_protocol::UiOverlayStickyModeV1::Partial),
        "always" => Some(fret_diag_protocol::UiOverlayStickyModeV1::Always),
        _ => None,
    }
}

fn parse_overlay_kind(s: &str) -> Option<fret_diag_protocol::UiOverlayPlacementTraceKindV1> {
    match s.trim().to_ascii_lowercase().as_str() {
        "anchored_panel" | "anchored-panel" => {
            Some(fret_diag_protocol::UiOverlayPlacementTraceKindV1::AnchoredPanel)
        }
        "placed_rect" | "placed-rect" => {
            Some(fret_diag_protocol::UiOverlayPlacementTraceKindV1::PlacedRect)
        }
        _ => None,
    }
}

fn overlay_entry_matches_query(
    entry: &fret_diag_protocol::UiOverlayPlacementTraceEntryV1,
    q: &fret_diag_protocol::UiOverlayPlacementTraceQueryV1,
) -> bool {
    let kind = match entry {
        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::AnchoredPanel { .. } => {
            fret_diag_protocol::UiOverlayPlacementTraceKindV1::AnchoredPanel
        }
        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::PlacedRect { .. } => {
            fret_diag_protocol::UiOverlayPlacementTraceKindV1::PlacedRect
        }
    };

    if let Some(want) = q.kind
        && want != kind
    {
        return false;
    }

    fn opt_str_matches(have: &Option<String>, want: &Option<String>) -> bool {
        match want.as_deref() {
            None => true,
            Some(w) => have.as_deref().is_some_and(|h| h == w),
        }
    }

    match entry {
        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::AnchoredPanel {
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            preferred_side,
            chosen_side,
            align,
            sticky,
            ..
        } => {
            if !opt_str_matches(overlay_root_name, &q.overlay_root_name) {
                return false;
            }
            if !opt_str_matches(anchor_test_id, &q.anchor_test_id) {
                return false;
            }
            if !opt_str_matches(content_test_id, &q.content_test_id) {
                return false;
            }
            if let Some(want) = q.preferred_side
                && want != *preferred_side
            {
                return false;
            }
            if let Some(want) = q.chosen_side
                && want != *chosen_side
            {
                return false;
            }
            if let Some(want) = q.flipped {
                let flipped = *preferred_side != *chosen_side;
                if want != flipped {
                    return false;
                }
            }
            if let Some(want) = q.align
                && want != *align
            {
                return false;
            }
            if let Some(want) = q.sticky
                && want != *sticky
            {
                return false;
            }
            true
        }
        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::PlacedRect {
            overlay_root_name,
            anchor_test_id,
            content_test_id,
            side,
            ..
        } => {
            if !opt_str_matches(overlay_root_name, &q.overlay_root_name) {
                return false;
            }
            if !opt_str_matches(anchor_test_id, &q.anchor_test_id) {
                return false;
            }
            if !opt_str_matches(content_test_id, &q.content_test_id) {
                return false;
            }
            if let Some(want) = q.chosen_side {
                // Best-effort: only `PlacedRect` has an optional `side`.
                if side.is_some_and(|s| s != want) || side.is_none() {
                    return false;
                }
            }
            true
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_query_overlay_placement_trace(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    stats_json: bool,
) -> Result<(), String> {
    let mut top: usize = 50;
    let mut q = fret_diag_protocol::UiOverlayPlacementTraceQueryV1::default();

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
            "--kind" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --kind".to_string());
                };
                q.kind = Some(parse_overlay_kind(v.as_str()).ok_or_else(|| {
                    "invalid value for --kind (expected anchored_panel|placed_rect)".to_string()
                })?);
                i += 1;
            }
            "--overlay-root-name" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --overlay-root-name".to_string());
                };
                q.overlay_root_name = Some(v);
                i += 1;
            }
            "--anchor-test-id" | "--anchor" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --anchor-test-id".to_string());
                };
                q.anchor_test_id = Some(v);
                i += 1;
            }
            "--content-test-id" | "--content" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --content-test-id".to_string());
                };
                q.content_test_id = Some(v);
                i += 1;
            }
            "--preferred-side" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --preferred-side".to_string());
                };
                q.preferred_side = Some(parse_overlay_side(v.as_str()).ok_or_else(|| {
                    "invalid value for --preferred-side (expected top|bottom|left|right)"
                        .to_string()
                })?);
                i += 1;
            }
            "--chosen-side" | "--side" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --chosen-side".to_string());
                };
                q.chosen_side = Some(parse_overlay_side(v.as_str()).ok_or_else(|| {
                    "invalid value for --chosen-side (expected top|bottom|left|right)".to_string()
                })?);
                i += 1;
            }
            "--flipped" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --flipped".to_string());
                };
                q.flipped = match v.trim() {
                    "1" | "true" => Some(true),
                    "0" | "false" => Some(false),
                    _ => {
                        return Err("invalid value for --flipped (expected true|false)".to_string());
                    }
                };
                i += 1;
            }
            "--align" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --align".to_string());
                };
                q.align = Some(parse_overlay_align(v.as_str()).ok_or_else(|| {
                    "invalid value for --align (expected start|center|end)".to_string()
                })?);
                i += 1;
            }
            "--sticky" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --sticky".to_string());
                };
                q.sticky = Some(parse_overlay_sticky(v.as_str()).ok_or_else(|| {
                    "invalid value for --sticky (expected partial|always)".to_string()
                })?);
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!(
                    "unknown flag for query overlay-placement-trace: {other}"
                ));
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

    let script_result_path = if let Some(src) = positionals.first() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(src));
        if src.is_dir() {
            let direct = src.join("script.result.json");
            if direct.is_file() {
                find_nearest_script_result_json_preferring_evidence(&src).ok_or_else(|| {
                    format!(
                        "failed to locate script.result.json near: {}\n\
hint: pass a diagnostics out dir (or bundle dir) that contains script.result.json",
                        src.display()
                    )
                })?
            } else {
                let resolved =
                    resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&src);
                let start = if resolved.is_dir() {
                    resolved.as_path()
                } else {
                    resolved.parent().unwrap_or_else(|| resolved.as_path())
                };
                find_nearest_script_result_json_preferring_evidence(start).ok_or_else(|| {
                    format!(
                        "failed to locate script.result.json near: {}\n\
hint: pass a diagnostics out dir (or bundle dir) that contains script.result.json",
                        src.display()
                    )
                })?
            }
        } else if src.is_file()
            && src
                .file_name()
                .is_some_and(|s| s.eq_ignore_ascii_case("script.result.json"))
        {
            let start = src.parent().unwrap_or_else(|| Path::new("."));
            find_nearest_script_result_json_preferring_evidence(start).ok_or_else(|| {
                format!(
                    "failed to locate script.result.json near: {}\n\
hint: pass a diagnostics out dir (or bundle dir) that contains script.result.json",
                    src.display()
                )
            })?
        } else {
            let resolved =
                resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&src);
            let start = if resolved.is_dir() {
                resolved.as_path()
            } else {
                resolved.parent().unwrap_or_else(|| resolved.as_path())
            };
            find_nearest_script_result_json_preferring_evidence(start).ok_or_else(|| {
                format!(
                    "failed to locate script.result.json near: {}\n\
hint: pass a diagnostics out dir (or bundle dir) that contains script.result.json",
                    src.display()
                )
            })?
        }
    } else {
        let bundle_path = resolve_bundle_artifact_path_or_latest(None, workspace_root, out_dir)?;
        let start = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        find_nearest_script_result_json_preferring_evidence(start).ok_or_else(|| {
            format!(
                "failed to locate script.result.json near latest bundle: {}",
                bundle_path.display()
            )
        })?
    };

    let result = read_script_result_typed(&script_result_path).ok_or_else(|| {
        format!(
            "script.result.json is missing or invalid (expected UiScriptResultV1 JSON)\n  path: {}",
            script_result_path.display()
        )
    })?;

    let evidence = result.evidence.as_ref().ok_or_else(|| {
        format!(
            "script.result.json has no evidence (missing `evidence` field)\n\
hint: overlay placement evidence is only captured when scripts require `diag.overlay_placement_trace` (e.g. via `wait_overlay_placement_trace`)\n  path: {}",
            script_result_path.display()
        )
    })?;

    let mut rows: Vec<&fret_diag_protocol::UiOverlayPlacementTraceEntryV1> = evidence
        .overlay_placement_trace
        .iter()
        .filter(|e| overlay_entry_matches_query(e, &q))
        .collect();

    if top > 0 && rows.len() > top {
        rows.truncate(top);
    }

    if stats_json {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "query.overlay_placement_trace",
            "script_result": script_result_path.display().to_string(),
            "top": top,
            "query": serde_json::to_value(&q).unwrap_or_else(|_| serde_json::json!({})),
            "results": rows.iter().map(|entry| serde_json::to_value(entry).unwrap_or_else(|_| serde_json::json!({ "error": "serialize_failed" }))).collect::<Vec<_>>(),
        });

        if let Some(out) = query_out.map(|p| crate::resolve_path(workspace_root, p)) {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
            println!("{}", out.display());
            return Ok(());
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    if rows.is_empty() {
        println!("(no matching overlay placement trace entries)");
        return Ok(());
    }

    for entry in rows {
        match entry {
            fret_diag_protocol::UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                step_index,
                frame_id,
                anchor_test_id,
                content_test_id,
                preferred_side,
                chosen_side,
                final_rect,
                shift_delta,
                ..
            } => {
                println!(
                    "anchored_panel step={} frame={} anchor_test_id={:?} content_test_id={:?} preferred={:?} chosen={:?} final=({:.1},{:.1},{:.1},{:.1}) shift=({:.1},{:.1})",
                    step_index,
                    frame_id,
                    anchor_test_id,
                    content_test_id,
                    preferred_side,
                    chosen_side,
                    final_rect.x_px,
                    final_rect.y_px,
                    final_rect.w_px,
                    final_rect.h_px,
                    shift_delta.x_px,
                    shift_delta.y_px
                );
            }
            fret_diag_protocol::UiOverlayPlacementTraceEntryV1::PlacedRect {
                step_index,
                frame_id,
                anchor_test_id,
                content_test_id,
                placed,
                side,
                ..
            } => {
                println!(
                    "placed_rect step={} frame={} anchor_test_id={:?} content_test_id={:?} side={:?} placed=({:.1},{:.1},{:.1},{:.1})",
                    step_index,
                    frame_id,
                    anchor_test_id,
                    content_test_id,
                    side,
                    placed.x_px,
                    placed.y_px,
                    placed.w_px,
                    placed.h_px,
                );
            }
        }
    }

    Ok(())
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
                    "missing pattern (try: fretboard diag query test-id <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> <pattern>)"
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("{prefix}-{}", crate::util::now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_script_result_with_overlay_trace(dir: &Path) -> PathBuf {
        use fret_diag_protocol::{
            UiOverlayPlacementTraceEntryV1, UiOverlaySideV1, UiRectV1, UiScriptEvidenceV1,
            UiScriptResultV1, UiScriptStageV1,
        };

        let rect = UiRectV1 {
            x_px: 1.0,
            y_px: 2.0,
            w_px: 3.0,
            h_px: 4.0,
        };

        let evidence = UiScriptEvidenceV1 {
            overlay_placement_trace: vec![
                UiOverlayPlacementTraceEntryV1::PlacedRect {
                    step_index: 10,
                    note: None,
                    frame_id: 100,
                    overlay_root_name: Some("root".to_string()),
                    anchor_element: None,
                    anchor_test_id: Some("anchor-a".to_string()),
                    content_element: None,
                    content_test_id: Some("panel-a".to_string()),
                    outer: rect,
                    anchor: rect,
                    placed: rect,
                    side: Some(UiOverlaySideV1::Top),
                },
                UiOverlayPlacementTraceEntryV1::PlacedRect {
                    step_index: 20,
                    note: None,
                    frame_id: 200,
                    overlay_root_name: Some("root".to_string()),
                    anchor_element: None,
                    anchor_test_id: Some("anchor-b".to_string()),
                    content_element: None,
                    content_test_id: Some("panel-b".to_string()),
                    outer: rect,
                    anchor: rect,
                    placed: rect,
                    side: Some(UiOverlaySideV1::Bottom),
                },
            ],
            ..UiScriptEvidenceV1::default()
        };

        let payload = UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(evidence),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };

        let path = dir.join("script.result.json");
        let bytes = serde_json::to_vec_pretty(&payload).expect("serialize script.result");
        std::fs::write(&path, bytes).expect("write script.result");
        path
    }

    fn write_script_result_without_evidence(dir: &Path) -> PathBuf {
        use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1};

        let payload = UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };

        let path = dir.join("script.result.json");
        let bytes = serde_json::to_vec_pretty(&payload).expect("serialize script.result");
        std::fs::write(&path, bytes).expect("write script.result");
        path
    }

    #[test]
    fn query_overlay_placement_trace_filters_by_anchor_and_side_and_writes_json() {
        let out_dir = make_temp_dir("fret-diag-query-overlay");
        let script_result = write_script_result_with_overlay_trace(&out_dir);

        let query_out = out_dir.join("out.json");
        cmd_query_overlay_placement_trace(
            &[
                script_result.display().to_string(),
                "--anchor-test-id".to_string(),
                "anchor-a".to_string(),
                "--chosen-side".to_string(),
                "top".to_string(),
            ],
            Path::new("."),
            &out_dir,
            Some(query_out.clone()),
            true,
        )
        .expect("query ok");

        let bytes = std::fs::read(&query_out).expect("read out.json");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse out.json");
        let results = v
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].get("kind").and_then(|v| v.as_str()),
            Some("placed_rect")
        );
    }

    #[test]
    fn query_overlay_placement_trace_accepts_out_dir_with_script_result_json() {
        let out_dir = make_temp_dir("fret-diag-query-overlay-dir");
        let _script_result = write_script_result_with_overlay_trace(&out_dir);

        let query_out = out_dir.join("out.json");
        cmd_query_overlay_placement_trace(
            &[
                out_dir.display().to_string(),
                "--anchor-test-id".to_string(),
                "anchor-b".to_string(),
                "--chosen-side".to_string(),
                "bottom".to_string(),
            ],
            Path::new("."),
            &out_dir,
            Some(query_out.clone()),
            true,
        )
        .expect("query ok");

        let bytes = std::fs::read(&query_out).expect("read out.json");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse out.json");
        let results = v
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn query_overlay_placement_trace_prefers_parent_script_result_with_evidence() {
        let out_dir = make_temp_dir("fret-diag-query-overlay-prefers-evidence");
        let parent = out_dir.join("sessions").join("sid");
        std::fs::create_dir_all(&parent).expect("create parent");
        let child = parent.join("bundle-subdir");
        std::fs::create_dir_all(&child).expect("create child");

        // Simulate the common layout where bundle dump dirs contain a script.result.json without
        // evidence, but the session root contains the evidence-bearing script.result.json.
        let _parent_script_result = write_script_result_with_overlay_trace(&parent);
        let _child_script_result = write_script_result_without_evidence(&child);

        let query_out = out_dir.join("out.json");
        cmd_query_overlay_placement_trace(
            &[
                child.display().to_string(),
                "--anchor-test-id".to_string(),
                "anchor-a".to_string(),
                "--chosen-side".to_string(),
                "top".to_string(),
            ],
            Path::new("."),
            &out_dir,
            Some(query_out.clone()),
            true,
        )
        .expect("query ok");

        let bytes = std::fs::read(&query_out).expect("read out.json");
        let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse out.json");
        let results = v
            .get("results")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(results.len(), 1);
    }
}
