use std::path::{Path, PathBuf};

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

fn resolve_latest_bundle_json_path(out_dir: &Path) -> Result<PathBuf, String> {
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

fn candidate_paths_for_sidecar(bundle_path: &Path, file_name: &str) -> Vec<PathBuf> {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let mut out: Vec<PathBuf> = Vec::new();

    out.push(dir.join(file_name));

    if dir.file_name().and_then(|s| s.to_str()) == Some("_root") {
        if let Some(grandparent) = dir.parent() {
            out.push(grandparent.join(file_name));
        }
    } else {
        out.push(dir.join("_root").join(file_name));
    }

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
    bundle_path: &Path,
    label: &'static str,
    kind: &'static str,
    file_name: &'static str,
    warmup_frames: u64,
    suggest: String,
) -> DoctorItem {
    let candidates = candidate_paths_for_sidecar(bundle_path, file_name);
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
            let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
            let script_result = dir.join("script.result.json");
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

    let bundle_path = if let Some(src) = rest.first().cloned() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(src));
        if src.is_file() {
            let file_name = src.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(
                file_name,
                "bundle.index.json" | "bundle.meta.json" | "test_ids.index.json"
            ) {
                if let Some(bundle_path) = sidecars::adjacent_bundle_json_path_for_sidecar(&src) {
                    bundle_path
                } else {
                    return Err(format!(
                        "unable to locate adjacent bundle.json for {}\n  tip: pass the bundle directory or bundle.json path instead",
                        src.display()
                    ));
                }
            } else {
                crate::resolve_bundle_json_path(&src)
            }
        } else {
            crate::resolve_bundle_json_path(&src)
        }
    } else {
        resolve_latest_bundle_json_path(out_dir)?
    };

    if !bundle_path.is_file() {
        return Err(format!(
            "missing bundle.json\n  bundle: {}",
            bundle_path.display()
        ));
    }

    let (items, required_ok, ok) = doctor_items(&bundle_path, warmup_frames);

    if stats_json {
        let payload = doctor_report_json(&bundle_path, warmup_frames);
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        println!("{pretty}");
        return Ok(());
    }

    println!("bundle: {}", bundle_path.display());
    println!("warmup_frames: {}", warmup_frames);
    println!("required_ok: {required_ok}");
    println!("ok: {ok}");
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

fn doctor_items(bundle_path: &Path, warmup_frames: u64) -> (Vec<DoctorItem>, bool, bool) {
    let mut items: Vec<DoctorItem> = Vec::new();
    items.push(build_item(
        bundle_path,
        "bundle.index.json",
        "bundle_index",
        "bundle.index.json",
        warmup_frames,
        format!(
            "fretboard diag index {} --warmup-frames {}",
            bundle_path.display(),
            warmup_frames
        ),
    ));
    items.push(build_item(
        bundle_path,
        "bundle.meta.json",
        "bundle_meta",
        "bundle.meta.json",
        warmup_frames,
        format!(
            "fretboard diag meta {} --warmup-frames {}",
            bundle_path.display(),
            warmup_frames
        ),
    ));
    items.push(build_item(
        bundle_path,
        "test_ids.index.json",
        "test_ids_index",
        "test_ids.index.json",
        warmup_frames,
        format!(
            "fretboard diag test-ids-index {} --warmup-frames {}",
            bundle_path.display(),
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

pub(crate) fn doctor_report_json(bundle_path: &Path, warmup_frames: u64) -> Value {
    let (items, required_ok, ok) = doctor_items(bundle_path, warmup_frames);
    json!({
        "ok": ok,
        "required_ok": required_ok,
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
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
