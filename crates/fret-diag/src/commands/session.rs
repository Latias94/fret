use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1};

use crate::run_artifacts::{write_run_id_bundle_json, write_run_id_script_result};
use crate::util::{now_unix_ms, touch};

use super::args::resolve_latest_bundle_dir_path;
use super::resolve;

pub(crate) fn cmd_poke(
    rest: &[String],
    pack_after_run: bool,
    out_dir: &Path,
    trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut label: Option<String> = None;
    let mut max_snapshots: Option<u32> = None;
    let mut request_id: Option<u64> = None;
    let mut wait = false;
    let mut record_run = false;
    let mut run_id_override: Option<u64> = None;

    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--label" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --label".to_string());
                };
                label = Some(v.to_string());
                i += 2;
            }
            "--max-snapshots" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --max-snapshots".to_string());
                };
                max_snapshots = v.parse::<u32>().ok();
                if max_snapshots.is_none() {
                    return Err("invalid value for --max-snapshots".to_string());
                }
                i += 2;
            }
            "--request-id" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --request-id".to_string());
                };
                request_id = v.parse::<u64>().ok();
                if request_id.is_none() {
                    return Err("invalid value for --request-id".to_string());
                }
                i += 2;
            }
            "--wait" => {
                wait = true;
                i += 1;
            }
            "--record-run" => {
                record_run = true;
                i += 1;
            }
            "--run-id" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --run-id".to_string());
                };
                run_id_override = v.parse::<u64>().ok();
                if run_id_override.is_none() {
                    return Err("invalid value for --run-id".to_string());
                }
                i += 2;
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    let prev_latest = std::fs::read_to_string(out_dir.join("latest.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if label.is_some() || max_snapshots.is_some() || request_id.is_some() {
        let payload = serde_json::json!({
            "schema_version": 1,
            "label": label.as_deref(),
            "max_snapshots": max_snapshots,
            "request_id": request_id.or_else(|| Some(now_unix_ms())),
        });
        let _ = crate::util::write_json_value(&out_dir.join("dump.request.json"), &payload);
    }

    touch(trigger_path)?;

    if !wait && !record_run {
        println!("{}", trigger_path.display());
        return Ok(());
    }

    let bundle_dir =
        wait_for_new_latest_dump_dir(out_dir, prev_latest.as_deref(), timeout_ms, poll_ms)?;
    if !record_run {
        println!("{}", bundle_dir.display());
        if out_dir.join(crate::session::SESSIONS_DIRNAME).is_dir() {
            eprintln!(
                "hint: if `{}` is a base dir with multiple sessions, prefer `fretboard diag resolve latest --dir {}` to avoid relying on base-level latest.txt",
                out_dir.display(),
                out_dir.display()
            );
        }
        return Ok(());
    }

    let run_id = run_id_override.unwrap_or_else(next_manual_run_id);
    let bundle_artifact_path = resolve_bundle_artifact_path_for_dir(&bundle_dir);

    let rel_bundle_dir = bundle_dir
        .strip_prefix(out_dir)
        .unwrap_or(&bundle_dir)
        .to_string_lossy()
        .replace('\\', "/");

    let result = UiScriptResultV1 {
        schema_version: 1,
        run_id,
        updated_unix_ms: now_unix_ms(),
        window: None,
        stage: UiScriptStageV1::Passed,
        step_index: None,
        reason_code: Some("manual.poke.dump".to_string()),
        reason: Some("manual diagnostics dump triggered via trigger.touch".to_string()),
        evidence: None,
        last_bundle_dir: Some(rel_bundle_dir),
        last_bundle_artifact: None,
    };

    write_run_id_script_result(out_dir, run_id, &result);
    write_run_id_bundle_json(out_dir, run_id, &bundle_artifact_path);

    let run_dir = out_dir.join(run_id.to_string());
    println!("{}", run_dir.display());
    Ok(())
}

fn resolve_bundle_artifact_path_for_dir(dir: &Path) -> PathBuf {
    let raw = dir.join("bundle.json");
    if raw.is_file() {
        return raw;
    }
    let schema2 = dir.join("bundle.schema2.json");
    if schema2.is_file() {
        return schema2;
    }
    raw
}

fn wait_for_new_latest_dump_dir(
    out_dir: &Path,
    prev_latest: Option<&str>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PathBuf, String> {
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for latest.txt to update (out_dir={})",
                out_dir.display()
            ));
        }

        if let Ok(text) = std::fs::read_to_string(out_dir.join("latest.txt")) {
            let latest = text.trim();
            if !latest.is_empty() && Some(latest) != prev_latest {
                let dir = out_dir.join(latest);
                if dir.is_dir() {
                    return Ok(dir);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

fn next_manual_run_id() -> u64 {
    // Keep manual poke runs disjoint from runtime script run ids (which are based on unix_ms).
    const HI: u64 = 1u64 << 63;
    HI.saturating_add(now_unix_ms())
}

pub(crate) fn cmd_path(
    rest: &[String],
    pack_after_run: bool,
    trigger_path: &Path,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    println!("{}", trigger_path.display());
    Ok(())
}

pub(crate) fn cmd_latest(
    rest: &[String],
    pack_after_run: bool,
    out_dir: &Path,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    let resolved = resolve::resolve_latest_bundle_dir_from_base_or_session_out_dir(out_dir, None);
    let path = resolved
        .as_ref()
        .map(|(p, _session_id, _source)| p.clone())
        .or_else(|_| resolve_latest_bundle_dir_path(out_dir))?;
    println!("{}", path.display());

    if let Ok((_p, session_id, source)) = resolved {
        if let Some(session_id) = session_id {
            eprintln!(
                "diag latest: resolved via sessions (session_id={} source={})",
                session_id, source
            );
        }
    }
    Ok(())
}
