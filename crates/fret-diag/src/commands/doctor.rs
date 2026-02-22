use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1};
use serde_json::{Value, json};

use super::sidecars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorStatus {
    Ok,
    Missing,
    Invalid,
}

#[derive(Debug, Clone)]
struct DoctorItem {
    label: &'static str,
    kind: &'static str,
    file_name: &'static str,
    candidates: Vec<PathBuf>,
    status: DoctorStatus,
    resolved_path: Option<PathBuf>,
    error: Option<String>,
    suggest: String,
    notes: Vec<String>,
}

fn resolve_latest_bundle_dir_path(out_dir: &Path) -> Result<PathBuf, String> {
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(latest)
}

fn normalize_bundle_dir(dir: &Path) -> PathBuf {
    if dir.file_name().and_then(|s| s.to_str()) == Some("_root") {
        return dir.parent().unwrap_or(dir).to_path_buf();
    }
    dir.to_path_buf()
}

fn resolve_bundle_dir_for_report(src: &Path) -> PathBuf {
    let dir = if src.is_file() {
        src.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    } else {
        src.to_path_buf()
    };
    normalize_bundle_dir(&dir)
}

fn candidate_paths_for_sidecar(bundle_dir: &Path, file_name: &str) -> Vec<PathBuf> {
    let dir = bundle_dir;
    let mut out: Vec<PathBuf> = Vec::new();

    out.push(dir.join(file_name));

    out.push(dir.join("_root").join(file_name));

    out.sort();
    out.dedup();
    out
}

fn bundle_index_has_script_markers(v: &serde_json::Value) -> bool {
    v.get("script")
        .and_then(|v| v.get("steps"))
        .and_then(|v| v.as_array())
        .is_some_and(|steps| !steps.is_empty())
}

fn try_read_sidecar_from_candidates(
    candidates: &[PathBuf],
    kind: &'static str,
    warmup_frames: u64,
) -> (
    DoctorStatus,
    Option<PathBuf>,
    Option<serde_json::Value>,
    Option<String>,
) {
    let mut first_invalid: Option<String> = None;
    for path in candidates {
        if !path.is_file() {
            continue;
        }
        match sidecars::read_sidecar_json_v1(path, kind, warmup_frames) {
            Ok(v) => return (DoctorStatus::Ok, Some(path.clone()), Some(v), None),
            Err(e) => {
                if first_invalid.is_none() {
                    first_invalid = Some(e.to_string());
                }
            }
        }
    }

    if first_invalid.is_some() {
        (DoctorStatus::Invalid, None, None, first_invalid)
    } else {
        (DoctorStatus::Missing, None, None, None)
    }
}

fn build_item(
    bundle_dir: &Path,
    label: &'static str,
    kind: &'static str,
    file_name: &'static str,
    warmup_frames: u64,
    suggest: String,
) -> DoctorItem {
    let candidates = candidate_paths_for_sidecar(bundle_dir, file_name);
    let (status, resolved_path, payload, error) =
        try_read_sidecar_from_candidates(&candidates, kind, warmup_frames);

    let mut notes: Vec<String> = Vec::new();
    if status == DoctorStatus::Ok && candidates.len() > 1 {
        if let Some(path) = &resolved_path {
            if path
                .components()
                .any(|c| c.as_os_str().to_str() == Some("_root"))
            {
                notes.push("resolved from _root/".to_string());
            }
        }
    }

    if kind == "bundle_index" && status == DoctorStatus::Ok {
        if let Some(v) = payload {
            let has_script = bundle_index_has_script_markers(&v);
            let script_result = bundle_dir.join("script.result.json");
            if script_result.is_file() && !has_script {
                notes.push("index missing script markers (script.result.json exists; rerun `diag index` to upgrade)".to_string());
            }
        }
    }

    DoctorItem {
        label,
        kind,
        file_name,
        candidates,
        status,
        resolved_path,
        error,
        suggest,
        notes,
    }
}

pub(crate) fn cmd_doctor(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if rest.len() > 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let mut bundle_dir = if let Some(src) = rest.first().cloned() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(src));
        resolve_bundle_dir_for_report(&src)
    } else {
        resolve_latest_bundle_dir_path(out_dir)?
    };

    // If the user points at an out-dir root (no bundle artifacts directly), prefer the latest
    // bundle directory so `doctor` produces a useful report without requiring another argument.
    let has_bundleish_artifact = bundle_dir.join("bundle.json").is_file()
        || bundle_dir.join("_root").join("bundle.json").is_file()
        || bundle_dir.join("bundle.index.json").is_file()
        || bundle_dir.join("_root").join("bundle.index.json").is_file()
        || bundle_dir.join("bundle.meta.json").is_file()
        || bundle_dir.join("_root").join("bundle.meta.json").is_file()
        || bundle_dir.join("test_ids.index.json").is_file()
        || bundle_dir
            .join("_root")
            .join("test_ids.index.json")
            .is_file()
        || bundle_dir.join("manifest.json").is_file()
        || bundle_dir.join("_root").join("manifest.json").is_file();
    if !has_bundleish_artifact {
        if let Ok(latest) = resolve_latest_bundle_dir_path(&bundle_dir) {
            bundle_dir = latest;
        }
    }

    let (items, required_ok, ok) = doctor_items(&bundle_dir, warmup_frames);

    if stats_json {
        let payload = doctor_report_json(&bundle_dir, warmup_frames);
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        println!("{pretty}");
        return Ok(());
    }

    println!("bundle_dir: {}", bundle_dir.display());
    println!("warmup_frames: {}", warmup_frames);
    println!("required_ok: {required_ok}");
    println!("ok: {ok}");
    let report = doctor_report_json(&bundle_dir, warmup_frames);
    let repairs = report
        .get("repairs")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    for r in repairs {
        let code = r.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let command = r.get("command").and_then(|v| v.as_str());
        let note = r.get("note").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(command) = command {
            println!("repair: {code} ({note})");
            println!("  command: {command}");
        } else if !note.is_empty() {
            println!("repair: {code} ({note})");
        }
    }
    for it in &items {
        match it.status {
            DoctorStatus::Ok => {
                let path = it
                    .resolved_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                println!("{}: ok ({})", it.label, path);
            }
            DoctorStatus::Missing => {
                println!("{}: missing", it.label);
                println!("  candidates:");
                for p in &it.candidates {
                    println!("    - {}", p.display());
                }
                println!("  fix: {}", it.suggest);
            }
            DoctorStatus::Invalid => {
                println!("{}: invalid", it.label);
                if let Some(err) = &it.error {
                    println!("  error: {err}");
                }
                println!("  fix: {}", it.suggest);
            }
        }
        for note in &it.notes {
            println!("  note: {note}");
        }
    }

    Ok(())
}

fn doctor_items(bundle_dir: &Path, warmup_frames: u64) -> (Vec<DoctorItem>, bool, bool) {
    let mut items: Vec<DoctorItem> = Vec::new();
    items.push(build_item(
        bundle_dir,
        "bundle.index.json",
        "bundle_index",
        "bundle.index.json",
        warmup_frames,
        format!(
            "fretboard diag index {} --warmup-frames {}",
            bundle_dir.display(),
            warmup_frames
        ),
    ));
    items.push(build_item(
        bundle_dir,
        "bundle.meta.json",
        "bundle_meta",
        "bundle.meta.json",
        warmup_frames,
        format!(
            "fretboard diag meta {} --warmup-frames {}",
            bundle_dir.display(),
            warmup_frames
        ),
    ));
    items.push(build_item(
        bundle_dir,
        "test_ids.index.json",
        "test_ids_index",
        "test_ids.index.json",
        warmup_frames,
        format!(
            "fretboard diag test-ids-index {} --warmup-frames {}",
            bundle_dir.display(),
            warmup_frames
        ),
    ));

    let required_ok = items
        .iter()
        .filter(|it| it.kind != "test_ids_index")
        .all(|it| it.status == DoctorStatus::Ok);
    let ok = items.iter().all(|it| it.status == DoctorStatus::Ok);
    (items, required_ok, ok)
}

fn resolve_bundle_json_path_no_materialize(bundle_dir: &Path) -> Option<PathBuf> {
    let direct = bundle_dir.join("bundle.json");
    if direct.is_file() {
        return Some(direct);
    }
    let root = bundle_dir.join("_root").join("bundle.json");
    if root.is_file() {
        return Some(root);
    }
    None
}

fn resolve_script_result_path(bundle_dir: &Path) -> Option<PathBuf> {
    let direct = bundle_dir.join("script.result.json");
    if direct.is_file() {
        return Some(direct);
    }
    if let Some(parent) = bundle_dir.parent() {
        let from_parent = parent.join("script.result.json");
        if from_parent.is_file() {
            return Some(from_parent);
        }
    }
    None
}

fn try_read_script_result_v1(path: &Path) -> Option<UiScriptResultV1> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()
}

fn resolve_manifest_path(bundle_dir: &Path) -> Option<PathBuf> {
    let direct = bundle_dir.join("manifest.json");
    if direct.is_file() {
        return Some(direct);
    }
    let root = bundle_dir.join("_root").join("manifest.json");
    if root.is_file() {
        return Some(root);
    }
    None
}

fn read_json_value(path: &Path) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn file_bytes(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

fn manifest_bundle_json_chunks_summary(manifest_dir: &Path, manifest: &Value) -> Option<Value> {
    let bundle_json = manifest.get("bundle_json")?;
    let chunks = bundle_json.get("chunks")?.as_array()?;
    if chunks.is_empty() {
        return None;
    }

    let mut missing: u64 = 0;
    let mut bytes_mismatch: u64 = 0;
    let mut total_expected_bytes: u64 = 0;
    for c in chunks {
        let Some(rel) = c.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let expected = c.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
        total_expected_bytes = total_expected_bytes.saturating_add(expected);
        let p = manifest_dir.join(rel);
        if !p.is_file() {
            missing = missing.saturating_add(1);
            continue;
        }
        if let Some(actual) = file_bytes(&p)
            && expected != 0
            && actual != expected
        {
            bytes_mismatch = bytes_mismatch.saturating_add(1);
        }
    }

    Some(json!({
        "chunks_total": chunks.len(),
        "chunks_missing": missing,
        "chunks_bytes_mismatch": bytes_mismatch,
        "total_expected_bytes": total_expected_bytes,
    }))
}

pub(crate) fn doctor_report_json(bundle_dir: &Path, warmup_frames: u64) -> Value {
    let normalized = resolve_bundle_dir_for_report(bundle_dir);
    let bundle_dir = normalized.as_path();

    let (items, required_ok, ok) = doctor_items(bundle_dir, warmup_frames);
    let bundle_json = resolve_bundle_json_path_no_materialize(bundle_dir);

    let script_result_path = resolve_script_result_path(bundle_dir);
    let script_result = script_result_path
        .as_deref()
        .and_then(try_read_script_result_v1)
        .map(|r| {
            let stage = match r.stage {
                UiScriptStageV1::Queued => "queued",
                UiScriptStageV1::Running => "running",
                UiScriptStageV1::Passed => "passed",
                UiScriptStageV1::Failed => "failed",
            };
            json!({
                "schema_version": r.schema_version,
                "run_id": r.run_id,
                "stage": stage,
                "reason_code": r.reason_code,
                "reason": r.reason,
                "last_bundle_dir": r.last_bundle_dir,
                "bundle_json_bytes": r.last_bundle_artifact.and_then(|a| a.bundle_json_bytes),
            })
        });

    let manifest_path = resolve_manifest_path(bundle_dir);
    let manifest = manifest_path.as_deref().and_then(read_json_value);
    let manifest_dir = manifest_path
        .as_deref()
        .and_then(|p| p.parent())
        .unwrap_or(bundle_dir);
    let manifest_chunks = manifest
        .as_ref()
        .and_then(|m| manifest_bundle_json_chunks_summary(manifest_dir, m));

    let mut repairs: Vec<Value> = Vec::new();
    for it in &items {
        if it.status != DoctorStatus::Ok {
            repairs.push(json!({
                "code": "regen_sidecar",
                "kind": it.kind,
                "file": it.file_name,
                "command": it.suggest,
            }));
        }
    }

    if bundle_json.is_none() {
        if let Some(chunks) = &manifest_chunks {
            let missing = chunks
                .get("chunks_missing")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let bytes_mismatch = chunks
                .get("chunks_bytes_mismatch")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if missing == 0 && bytes_mismatch == 0 {
                repairs.push(json!({
                    "code": "materialize_bundle_json",
                    "note": "bundle.json is missing but manifest chunks look complete; running a command that resolves bundle.json from chunks should restore it",
                    "command": format!("fretboard diag index {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                }));
            } else {
                repairs.push(json!({
                    "code": "incomplete_chunks",
                    "note": "bundle.json is missing and manifest chunks are incomplete; re-extract the share zip or re-capture the bundle",
                }));
            }
        } else {
            repairs.push(json!({
                "code": "missing_bundle_json",
                "note": "bundle.json is missing and no manifest chunks were found; re-capture the bundle",
            }));
        }
    }

    json!({
        "ok": ok,
        "required_ok": required_ok,
        "bundle_dir": bundle_dir.display().to_string(),
        "bundle_json": bundle_json.as_ref().map(|p| p.display().to_string()),
        "bundle_json_bytes": bundle_json.as_deref().and_then(file_bytes),
        "warmup_frames": warmup_frames,
        "script_result_path": script_result_path.as_ref().map(|p| p.display().to_string()),
        "script_result": script_result,
        "manifest_path": manifest_path.as_ref().map(|p| p.display().to_string()),
        "manifest_chunks": manifest_chunks,
        "repairs": repairs,
        "items": items.iter().map(|it| {
            json!({
                "label": it.label,
                "kind": it.kind,
                "file_name": it.file_name,
                "status": match it.status {
                    DoctorStatus::Ok => "ok",
                    DoctorStatus::Missing => "missing",
                    DoctorStatus::Invalid => "invalid",
                },
                "resolved_path": it.resolved_path.as_ref().map(|p| p.display().to_string()),
                "candidates": it.candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "error": it.error,
                "suggest": it.suggest,
                "notes": it.notes,
            })
        }).collect::<Vec<_>>(),
    })
}
