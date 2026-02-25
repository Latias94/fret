use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1};
use serde_json::{Value, json};

use super::args::resolve_latest_bundle_dir_path;
use super::sidecars;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DoctorRunOptions {
    pub(crate) fix_bundle_json: bool,
    pub(crate) fix_schema2: bool,
    pub(crate) fix_sidecars: bool,
    pub(crate) fix_dry_run: bool,
    pub(crate) check_required: bool,
    pub(crate) check_all: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DoctorRunResult {
    pub(crate) bundle_dir: PathBuf,
    pub(crate) report: Value,
    pub(crate) fixes_planned: Vec<String>,
    pub(crate) fixes_applied: Vec<String>,
}

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

pub(crate) fn run_doctor_for_bundle_dir(
    bundle_dir: &Path,
    warmup_frames: u64,
    opts: DoctorRunOptions,
) -> Result<DoctorRunResult, String> {
    let bundle_dir = normalize_bundle_dir(bundle_dir);

    let mut fixes_applied: Vec<String> = Vec::new();
    let mut fixes_planned: Vec<String> = Vec::new();

    let plan_report = doctor_report_json(&bundle_dir, warmup_frames);
    if opts.fix_bundle_json {
        let has_bundle_artifact = plan_report
            .get("bundle_artifact")
            .is_some_and(|v| !v.is_null());
        if !has_bundle_artifact {
            let chunks = plan_report.get("manifest_chunks");
            let can_materialize = chunks.is_some_and(|c| {
                c.get("chunks_missing").and_then(|v| v.as_u64()) == Some(0)
                    && c.get("chunks_bytes_mismatch").and_then(|v| v.as_u64()) == Some(0)
            });
            if can_materialize {
                fixes_planned.push("materialize raw bundle.json from manifest chunks".to_string());
            } else {
                fixes_planned.push(
                    "materialize raw bundle.json from manifest chunks (blocked: incomplete/missing manifest chunks)"
                        .to_string(),
                );
            }
        }
    }
    if opts.fix_sidecars {
        let items = plan_report
            .get("items")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        for it in items {
            let status = it.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if status == "ok" {
                let notes = it
                    .get("notes")
                    .and_then(|v| v.as_array())
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
                let needs_upgrade = notes.iter().any(|n| {
                    n.as_str()
                        .is_some_and(|s| s.contains("missing script markers"))
                });
                if needs_upgrade {
                    fixes_planned
                        .push("regenerate bundle.index.json to add script markers".to_string());
                }
                continue;
            }
            let file = it.get("file_name").and_then(|v| v.as_str()).unwrap_or("");
            if !file.is_empty() {
                fixes_planned.push(format!("regenerate {file}"));
            }
        }
    }
    if opts.fix_schema2 {
        let schema2_exists =
            crate::resolve_bundle_schema2_artifact_path_no_materialize(&bundle_dir).is_some();
        if !schema2_exists {
            if crate::resolve_raw_bundle_artifact_path_no_materialize(&bundle_dir).is_some() {
                fixes_planned.push(
                    "write bundle.schema2.json from raw bundle.json (--mode last)".to_string(),
                );
            } else {
                fixes_planned.push(
                    "write bundle.schema2.json from raw bundle.json (blocked: missing raw bundle.json)"
                        .to_string(),
                );
            }
        }
    }

    if opts.fix_dry_run {
        return Ok(DoctorRunResult {
            bundle_dir,
            report: plan_report,
            fixes_planned,
            fixes_applied,
        });
    }

    if opts.fix_bundle_json {
        if crate::resolve_bundle_artifact_path_no_materialize(&bundle_dir).is_none() {
            let attempts = [bundle_dir.clone(), bundle_dir.join("_root")];
            for dir in &attempts {
                match crate::run_artifacts::materialize_bundle_json_from_manifest_chunks_if_missing(
                    dir,
                ) {
                    Ok(Some(out)) => {
                        fixes_applied.push(format!(
                            "materialized raw bundle.json from chunks ({})",
                            out.display()
                        ));
                        break;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        fixes_applied.push(format!(
                            "attempted to materialize raw bundle.json from chunks, but failed ({})",
                            err
                        ));
                    }
                }
            }
        }
    }

    if opts.fix_schema2 {
        let schema2_exists =
            crate::resolve_bundle_schema2_artifact_path_no_materialize(&bundle_dir).is_some();
        if !schema2_exists {
            let Some(bundle_json) =
                crate::resolve_raw_bundle_artifact_path_no_materialize(&bundle_dir)
            else {
                fixes_applied.push(
                    "skipped writing bundle.schema2.json (missing raw bundle.json)".to_string(),
                );
                let report = doctor_report_json(&bundle_dir, warmup_frames);
                return Ok(DoctorRunResult {
                    bundle_dir,
                    report,
                    fixes_planned,
                    fixes_applied,
                });
            };

            let out_dir = bundle_json.parent().unwrap_or(&bundle_dir);
            let out = out_dir.join("bundle.schema2.json");
            match super::bundle_v2::write_bundle_schema2_json_from_path(
                &bundle_json,
                &out,
                super::bundle_v2::WriteBundleSchema2Options {
                    mode: "last",
                    pretty: false,
                    force: false,
                },
            ) {
                Ok(res) => {
                    fixes_applied.push(format!(
                        "wrote bundle.schema2.json (input_schema_version={}, output_bytes={})",
                        res.input_schema_version, res.output_bytes
                    ));
                }
                Err(err) => {
                    fixes_applied.push(format!(
                        "attempted to write bundle.schema2.json, but failed ({err})"
                    ));
                }
            }
        }
    }

    if opts.fix_sidecars {
        let bundle_artifact =
            crate::resolve_bundle_artifact_path_no_materialize(&bundle_dir).ok_or_else(|| {
            "unable to regenerate sidecars: missing bundle artifact (bundle.json or bundle.schema2.json) (tip: re-run with --fix-bundle-json, or provide a bundle dir that contains one of those files)".to_string()
        })?;
        let _ = crate::bundle_index::ensure_bundle_meta_json(&bundle_artifact, warmup_frames)
            .map(|p| fixes_applied.push(format!("regenerated bundle.meta.json ({})", p.display())));
        let _ = crate::bundle_index::ensure_test_ids_index_json(&bundle_artifact, warmup_frames)
            .map(|p| {
                fixes_applied.push(format!("regenerated test_ids.index.json ({})", p.display()))
            });
        let _ = crate::bundle_index::ensure_bundle_index_json(&bundle_artifact, warmup_frames).map(
            |p| fixes_applied.push(format!("regenerated bundle.index.json ({})", p.display())),
        );
        let _ = crate::frames_index::ensure_frames_index_json(&bundle_artifact, warmup_frames).map(
            |p| fixes_applied.push(format!("regenerated frames.index.json ({})", p.display())),
        );
    }

    let report = doctor_report_json(&bundle_dir, warmup_frames);
    Ok(DoctorRunResult {
        bundle_dir,
        report,
        fixes_planned,
        fixes_applied,
    })
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

    let mut fix_bundle_json: bool = false;
    let mut fix_schema2: bool = false;
    let mut fix_sidecars: bool = false;
    let mut fix_dry_run: bool = false;
    let mut check_required: bool = false;
    let mut check_all: bool = false;
    let mut positionals: Vec<String> = Vec::new();

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fix" => {
                fix_bundle_json = true;
                fix_sidecars = true;
                i += 1;
            }
            "--fix-dry-run" | "--fix-plan" => {
                fix_bundle_json = true;
                fix_sidecars = true;
                fix_dry_run = true;
                i += 1;
            }
            "--fix-schema2" => {
                fix_schema2 = true;
                i += 1;
            }
            "--fix-bundle-json" => {
                fix_bundle_json = true;
                i += 1;
            }
            "--fix-sidecars" => {
                fix_sidecars = true;
                i += 1;
            }
            "--check" | "--check-required" => {
                check_required = true;
                i += 1;
            }
            "--check-all" | "--strict" => {
                check_all = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for doctor: {other}"));
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

    let mut bundle_dir = if let Some(src) = positionals.first().cloned() {
        let src = crate::resolve_path(workspace_root, PathBuf::from(src));
        resolve_bundle_dir_for_report(&src)
    } else {
        resolve_latest_bundle_dir_path(out_dir)?
    };

    // If the user points at an out-dir root (no bundle artifacts directly), prefer the latest
    // bundle directory so `doctor` produces a useful report without requiring another argument.
    let has_bundleish_artifact = crate::resolve_bundle_artifact_path_no_materialize(&bundle_dir)
        .is_some()
        || [
            "bundle.index.json",
            "bundle.meta.json",
            "test_ids.index.json",
            "frames.index.json",
        ]
        .into_iter()
        .any(|name| {
            bundle_dir.join(name).is_file() || bundle_dir.join("_root").join(name).is_file()
        })
        || bundle_dir.join("manifest.json").is_file()
        || bundle_dir.join("_root").join("manifest.json").is_file();
    if !has_bundleish_artifact {
        if let Ok(latest) = resolve_latest_bundle_dir_path(&bundle_dir) {
            bundle_dir = latest;
        }
    }

    let opts = DoctorRunOptions {
        fix_bundle_json,
        fix_schema2,
        fix_sidecars,
        fix_dry_run,
        check_required,
        check_all,
    };
    let run = run_doctor_for_bundle_dir(&bundle_dir, warmup_frames, opts)?;
    let bundle_dir = run.bundle_dir;
    let fixes_planned = run.fixes_planned;
    let fixes_applied = run.fixes_applied;
    let report = run.report;

    let ok = report.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let required_ok = report
        .get("required_ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let (items, _required_ok_items, _ok_items) = doctor_items(&bundle_dir, warmup_frames);

    if stats_json {
        let mut payload = report;
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "fixes_planned".to_string(),
                serde_json::Value::Array(
                    fixes_planned
                        .iter()
                        .map(|s| serde_json::Value::String(s.to_string()))
                        .collect(),
                ),
            );
            obj.insert(
                "fixes_applied".to_string(),
                serde_json::Value::Array(
                    fixes_applied
                        .iter()
                        .map(|s| serde_json::Value::String(s.to_string()))
                        .collect(),
                ),
            );
            obj.insert(
                "fix_dry_run".to_string(),
                serde_json::Value::Bool(fix_dry_run),
            );
            obj.insert("check".to_string(), serde_json::Value::Bool(check_required));
            obj.insert("check_all".to_string(), serde_json::Value::Bool(check_all));
        }
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        println!("{pretty}");
        if fix_dry_run {
            if (check_all && !ok) || (check_required && !required_ok) {
                std::process::exit(1);
            }
            return Ok(());
        }
        if (check_all && !ok) || (check_required && !required_ok) {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("bundle_dir: {}", bundle_dir.display());
    println!("warmup_frames: {}", warmup_frames);
    if fix_dry_run {
        for f in &fixes_planned {
            println!("plan: {f}");
        }
        return Ok(());
    }
    if !fixes_applied.is_empty() {
        for f in &fixes_applied {
            println!("fixed: {f}");
        }
    }
    if !fixes_planned.is_empty() {
        for f in &fixes_planned {
            println!("plan: {f}");
        }
    }
    println!("required_ok: {required_ok}");
    println!("ok: {ok}");
    let warnings = report
        .get("warnings")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    for w in warnings {
        if let Some(w) = w.as_str() {
            println!("warning: {w}");
        }
    }
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

    if check_all && !ok {
        std::process::exit(1);
    }
    if check_required && !required_ok {
        std::process::exit(1);
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
    items.push(build_item(
        bundle_dir,
        "frames.index.json",
        "frames_index",
        "frames.index.json",
        warmup_frames,
        format!(
            "fretboard diag frames-index {} --warmup-frames {}",
            bundle_dir.display(),
            warmup_frames
        ),
    ));

    let required_ok = items
        .iter()
        .filter(|it| it.kind != "test_ids_index" && it.kind != "frames_index")
        .all(|it| it.status == DoctorStatus::Ok);
    let ok = items.iter().all(|it| it.status == DoctorStatus::Ok);
    (items, required_ok, ok)
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

fn read_json_value_result(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn file_bytes(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

fn sniff_schema_version_from_json_prefix(bytes: &[u8]) -> Option<u64> {
    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn parse_number_after_key(bytes: &[u8], key_offset: usize, key_len: usize) -> Option<u64> {
        let mut i = key_offset.saturating_add(key_len);
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i = i.saturating_add(1);
        }
        if bytes.get(i).copied() != Some(b':') {
            return None;
        }
        i = i.saturating_add(1);
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i = i.saturating_add(1);
        }

        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i = i.saturating_add(1);
        }
        if start == i {
            return None;
        }
        std::str::from_utf8(&bytes[start..i])
            .ok()?
            .parse::<u64>()
            .ok()
    }

    for key in [&br#""schema_version""#[..], &br#""schemaVersion""#[..]] {
        let Some(off) = find_subslice(bytes, key) else {
            continue;
        };
        if let Some(v) = parse_number_after_key(bytes, off, key.len()) {
            return Some(v);
        }
    }

    None
}

fn sniff_bundle_schema_version(bundle_json_path: &Path) -> Result<Option<u64>, String> {
    // Only read a prefix: schema_version is expected near the top-level object.
    const MAX_PREFIX_BYTES: usize = 64 * 1024;
    let mut bytes = std::fs::read(bundle_json_path).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PREFIX_BYTES {
        bytes.truncate(MAX_PREFIX_BYTES);
    }
    Ok(sniff_schema_version_from_json_prefix(&bytes))
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
    let mut missing_paths_sample: Vec<String> = Vec::new();
    let mut bytes_mismatch_paths_sample: Vec<String> = Vec::new();
    const MAX_SAMPLE: usize = 8;
    for c in chunks {
        let Some(rel) = c.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let expected = c.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
        total_expected_bytes = total_expected_bytes.saturating_add(expected);
        let p = manifest_dir.join(rel);
        if !p.is_file() {
            missing = missing.saturating_add(1);
            if missing_paths_sample.len() < MAX_SAMPLE {
                missing_paths_sample.push(rel.to_string());
            }
            continue;
        }
        if let Some(actual) = file_bytes(&p)
            && expected != 0
            && actual != expected
        {
            bytes_mismatch = bytes_mismatch.saturating_add(1);
            if bytes_mismatch_paths_sample.len() < MAX_SAMPLE {
                bytes_mismatch_paths_sample.push(rel.to_string());
            }
        }
    }

    Some(json!({
        "chunks_total": chunks.len(),
        "chunks_missing": missing,
        "chunks_bytes_mismatch": bytes_mismatch,
        "total_expected_bytes": total_expected_bytes,
        "missing_paths_sample": missing_paths_sample,
        "bytes_mismatch_paths_sample": bytes_mismatch_paths_sample,
    }))
}

pub(crate) fn doctor_report_json(bundle_dir: &Path, warmup_frames: u64) -> Value {
    let normalized = resolve_bundle_dir_for_report(bundle_dir);
    let bundle_dir = normalized.as_path();

    let (items, required_ok, ok) = doctor_items(bundle_dir, warmup_frames);
    let bundle_artifact = crate::resolve_bundle_artifact_path_no_materialize(bundle_dir);
    let bundle_artifact_bytes = bundle_artifact.as_deref().and_then(file_bytes);
    let raw_bundle_json = crate::resolve_raw_bundle_artifact_path_no_materialize(bundle_dir);
    let raw_bundle_json_bytes = raw_bundle_json.as_deref().and_then(file_bytes);
    let schema2_exists =
        crate::resolve_bundle_schema2_artifact_path_no_materialize(bundle_dir).is_some();
    let bundle_is_raw_bundle_json = bundle_artifact
        .as_deref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        == Some("bundle.json");
    let (bundle_schema_version, bundle_schema_error) =
        if let Some(path) = bundle_artifact.as_deref() {
            match sniff_bundle_schema_version(path) {
                Ok(v) => (v, None),
                Err(e) => (None, Some(e)),
            }
        } else {
            (None, None)
        };

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
    let (manifest, manifest_error) = if let Some(path) = manifest_path.as_deref() {
        match read_json_value_result(path) {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e)),
        }
    } else {
        (None, None)
    };
    let manifest_dir = manifest_path
        .as_deref()
        .and_then(|p| p.parent())
        .unwrap_or(bundle_dir);
    let manifest_chunks = manifest
        .as_ref()
        .and_then(|m| manifest_bundle_json_chunks_summary(manifest_dir, m));
    let manifest_schema_version = manifest
        .as_ref()
        .and_then(|m| m.get("schema_version"))
        .and_then(|v| v.as_u64());
    let manifest_run_id = manifest
        .as_ref()
        .and_then(|m| m.get("run_id"))
        .and_then(|v| v.as_u64());

    let mut repairs: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();
    if !items.iter().all(|it| it.status == DoctorStatus::Ok) {
        repairs.push(json!({
            "code": "fix_sidecars",
            "note": "regenerate common sidecars in one step",
            "command": format!("fretboard diag doctor --fix-sidecars {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
        }));
    }
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

    if let Some(err) = &manifest_error {
        repairs.push(json!({
            "code": "invalid_manifest_json",
            "note": "manifest.json exists but could not be parsed as JSON",
            "error": err,
            "repair_hint": "re-extract the share zip (ensure manifest.json is intact) or re-capture the bundle",
        }));
    }

    if let Some(schema_version) = bundle_schema_version {
        if schema_version != 1 && schema_version != 2 {
            warnings.push(Value::String(format!(
                "unsupported bundle schema_version={schema_version} (expected 1 or 2)"
            )));
            repairs.push(json!({
                "code": "unsupported_bundle_schema_version",
                "note": "bundle schema_version is not supported by in-tree tooling",
                "schema_version": schema_version,
                "repair_hint": "re-capture the bundle or regenerate it from a known-good source",
            }));
        }
        if schema_version == 1 {
            if let Some(bytes) = bundle_artifact_bytes {
                const SUGGEST_V2_MIN_BYTES: u64 = 64 * 1024 * 1024;
                if bytes >= SUGGEST_V2_MIN_BYTES {
                    warnings.push(Value::String(
                        "bundle uses schema v1; consider converting to schema v2 to reduce bundle size and enable semantics tables"
                            .to_string(),
                    ));
                    repairs.push(json!({
                        "code": "suggest_bundle_v2",
                        "note": "convert to schema v2 (writes bundle.schema2.json)",
                        "command": format!("fretboard diag doctor --fix-schema2 {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                    }));
                }
            }
        }
        if schema_version == 2 && !schema2_exists && bundle_is_raw_bundle_json {
            if let Some(bytes) = bundle_artifact_bytes {
                const SUGGEST_SCHEMA2_MIN_BYTES: u64 = 64 * 1024 * 1024;
                if bytes >= SUGGEST_SCHEMA2_MIN_BYTES {
                    warnings.push(Value::String(
                        "raw bundle.json is large; consider writing bundle.schema2.json to keep tooling and AI loops fast"
                            .to_string(),
                    ));
                    repairs.push(json!({
                        "code": "suggest_bundle_schema2",
                        "note": "write a compact schema2 bundle (writes bundle.schema2.json)",
                        "command": format!("fretboard diag doctor --fix-schema2 {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                    }));
                }
            }
        }
    } else if bundle_artifact.is_some() {
        warnings.push(Value::String(
            "bundle is present but schema_version could not be detected from the file prefix"
                .to_string(),
        ));
        repairs.push(json!({
            "code": "missing_bundle_schema_version",
            "note": "bundle is present but schema_version could not be detected",
                "repair_hint": "re-capture the bundle or regenerate it; ensure schema_version is at the top-level object",
        }));
    }
    if let Some(err) = &bundle_schema_error {
        warnings.push(Value::String(format!(
            "bundle schema version sniff failed: {err}"
        )));
    }

    if bundle_artifact.is_none() {
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
                    "note": "bundle artifact is missing; manifest chunks look complete, so materialize raw bundle.json from chunks",
                    "command": format!("fretboard diag doctor --fix-bundle-json {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                }));
            } else {
                repairs.push(json!({
                    "code": "incomplete_chunks",
                    "note": "bundle artifact is missing and manifest chunks are incomplete; re-extract the share zip (ensure all chunk files are present) or re-capture the bundle",
                    "chunks_missing": missing,
                    "chunks_bytes_mismatch": bytes_mismatch,
                    "missing_paths_sample": chunks.get("missing_paths_sample"),
                    "bytes_mismatch_paths_sample": chunks.get("bytes_mismatch_paths_sample"),
                }));
            }
        } else {
            repairs.push(json!({
                "code": "missing_bundle_json",
                "note": "bundle artifact is missing and no manifest chunks were found; re-capture the bundle",
            }));
        }
    }

    if let Some(bundle_json_bytes) = bundle_artifact_bytes {
        const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
        if bundle_json_bytes > DEFAULT_MAX_FILE_BYTES {
            repairs.push(json!({
                "code": "bundle_json_too_large",
                "note": "bundle is very large; prefer generating a small ai-packet for agentic triage",
                "bundle_json_bytes": bundle_json_bytes,
                "command": format!("fretboard diag ai-packet {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
            }));
        }
    }

    json!({
        "ok": ok,
        "required_ok": required_ok,
        "bundle_dir": bundle_dir.display().to_string(),
        "bundle_json": bundle_artifact.as_ref().map(|p| p.display().to_string()),
        "bundle_json_bytes": bundle_artifact_bytes,
        "bundle_artifact": bundle_artifact.as_ref().map(|p| p.display().to_string()),
        "bundle_artifact_bytes": bundle_artifact_bytes,
        "raw_bundle_json": raw_bundle_json.as_ref().map(|p| p.display().to_string()),
        "raw_bundle_json_bytes": raw_bundle_json_bytes,
        "bundle_schema_version": bundle_schema_version,
        "bundle_schema_error": bundle_schema_error,
        "warmup_frames": warmup_frames,
        "script_result_path": script_result_path.as_ref().map(|p| p.display().to_string()),
        "script_result": script_result,
        "manifest_path": manifest_path.as_ref().map(|p| p.display().to_string()),
        "manifest_chunks": manifest_chunks,
        "manifest_schema_version": manifest_schema_version,
        "manifest_run_id": manifest_run_id,
        "manifest_error": manifest_error,
        "warnings": warnings,
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
