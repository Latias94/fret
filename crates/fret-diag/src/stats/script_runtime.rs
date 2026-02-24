use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::util::{
    read_pick_result, read_pick_result_run_id, read_script_result, read_script_result_run_id,
    touch, write_script,
};

#[derive(Debug, Clone)]
pub(crate) struct ScriptResultSummary {
    pub(crate) run_id: u64,
    pub(crate) stage: Option<String>,
    pub(crate) step_index: Option<u64>,
    pub(crate) reason_code: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PickResultSummary {
    pub(crate) run_id: u64,
    pub(crate) stage: Option<String>,
    pub(crate) reason: Option<String>,
    pub(crate) last_bundle_dir: Option<String>,
    pub(crate) selector: Option<serde_json::Value>,
}

pub(crate) fn run_script_and_wait(
    src: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ScriptResultSummary, String> {
    fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
        // Give the app a little time to observe the initial trigger file state. On cold start,
        // the first observed stamp is treated as a baseline (not a trigger) to avoid replaying
        // stale scripts when the diagnostics directory is reused.
        //
        // If the external driver touches the file before the app has observed it once, the touch
        // can be consumed as the baseline and the script will never run unless the stamp advances
        // again. We mitigate this by re-touching once after a short grace period if no run starts.
        let baseline_race_ms = poll_ms.saturating_mul(4).clamp(250, 5_000);
        baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
    }

    let prev_run_id = read_script_result_run_id(script_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    write_script(src, script_path)?;
    touch(script_trigger_path)?;

    let start_deadline =
        Instant::now() + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms));
    let mut next_retouch_at = start_deadline;
    let mut retouch_interval_ms: u64 = 2_000;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for script result (result: {}, trigger: {})",
                script_result_path.display(),
                script_result_trigger_path.display()
            ));
        }

        if target_run_id.is_none() && Instant::now() >= next_retouch_at {
            // See comment in `start_grace_ms`.
            //
            // In `--launch` mode the demo process may start significantly later than the driver
            // touches the trigger (e.g. due to compilation). The in-app contract intentionally
            // treats the first observed stamp as a baseline. Periodic re-touching ensures at
            // least one trigger lands after the app has observed its baseline value, without
            // requiring any changes to the in-app polling contract.
            touch(script_trigger_path)?;
            retouch_interval_ms = (retouch_interval_ms.saturating_mul(2)).min(10_000);
            next_retouch_at = Instant::now() + Duration::from_millis(retouch_interval_ms);
        }

        if let Some(result) = read_script_result(script_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("passed") | Some("failed")) {
                    let step_index = result.get("step_index").and_then(|v| v.as_u64());
                    let reason_code = result
                        .get("reason_code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(ScriptResultSummary {
                        run_id,
                        stage,
                        step_index,
                        reason_code,
                        reason,
                        last_bundle_dir,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(crate) fn clear_script_result_files(
    script_result_path: &Path,
    script_result_trigger_path: &Path,
) {
    let _ = std::fs::remove_file(script_result_path);
    let _ = std::fs::remove_file(script_result_trigger_path);
}

pub(crate) fn report_result_and_exit(result: &ScriptResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("passed") => {
            println!("PASS (run_id={})", result.run_id);
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason}",
                        result.run_id, step
                    );
                } else {
                    eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
                }
            } else if let Some(step) = result.step_index {
                eprintln!(
                    "FAIL (run_id={}) step={} reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id, step
                );
            } else {
                eprintln!(
                    "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id
                );
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected script stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_pick_and_wait(
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PickResultSummary, String> {
    fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
        // Same baseline-race mitigation as `run_script_and_wait`.
        let baseline_race_ms = poll_ms.saturating_mul(4).clamp(250, 5_000);
        baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
    }

    let prev_run_id = read_pick_result_run_id(pick_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;
    let mut did_retouch = false;

    touch(pick_trigger_path)?;

    let start_deadline =
        Instant::now() + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms));
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for pick result (result: {}, trigger: {})",
                pick_result_path.display(),
                pick_result_trigger_path.display()
            ));
        }

        if !did_retouch && target_run_id.is_none() && Instant::now() >= start_deadline {
            touch(pick_trigger_path)?;
            did_retouch = true;
        }

        if let Some(result) = read_pick_result(pick_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("picked")) {
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let selector = result
                        .get("selection")
                        .and_then(|v| v.get("selectors"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .cloned();

                    return Ok(PickResultSummary {
                        run_id,
                        stage,
                        reason,
                        last_bundle_dir,
                        selector,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(crate) fn report_pick_result_and_exit(result: &PickResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("picked") => {
            if let Some(sel) = result.selector.as_ref() {
                println!("{}", serde_json::to_string(sel).unwrap_or_default());
            } else {
                println!("PICKED (run_id={})", result.run_id);
            }
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
            } else {
                eprintln!(
                    "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id
                );
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected pick stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn expected_failure_dump_suffixes(result: &ScriptResultSummary) -> Vec<String> {
    let Some(step_index) = result.step_index else {
        return Vec::new();
    };
    let Some(reason) = result.reason.as_deref() else {
        return Vec::new();
    };

    match reason {
        "wait_until_timeout" => vec![format!("script-step-{step_index:04}-wait_until-timeout")],
        "assert_failed" => vec![format!("script-step-{step_index:04}-assert-failed")],
        "no_semantics_snapshot" => vec![
            format!("script-step-{step_index:04}-wait_until-no-semantics"),
            format!("script-step-{step_index:04}-assert-no-semantics"),
        ],
        _ => Vec::new(),
    }
}

pub(crate) fn wait_for_failure_dump_bundle(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let suffixes = expected_failure_dump_suffixes(result);
    if suffixes.is_empty() {
        return None;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(250, 5_000));
    while Instant::now() < deadline {
        for suffix in &suffixes {
            if let Some(dir) = find_latest_export_dir_with_suffix(out_dir, suffix)
                && crate::resolve_bundle_artifact_path(&dir).is_file()
            {
                return Some(dir);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn find_latest_export_dir_with_suffix(out_dir: &Path, suffix: &str) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(suffix) {
            continue;
        }
        let Some((ts_str, _)) = name.split_once('-') else {
            continue;
        };
        let Ok(ts) = ts_str.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

pub(crate) fn write_pick_script(selector: &serde_json::Value, dst: &Path) -> Result<(), String> {
    let script = serde_json::json!({
        "schema_version": 1,
        "steps": [
            { "type": "click", "target": selector },
            { "type": "wait_frames", "frames": 2 },
            { "type": "capture_bundle", "label": "after-picked-click" }
        ]
    });

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

pub(crate) fn apply_pick_to_script(
    src: &Path,
    dst: &Path,
    json_pointer: &str,
    selector: serde_json::Value,
) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let mut script: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    crate::util::json_pointer_set(&mut script, json_pointer, selector)?;

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}
