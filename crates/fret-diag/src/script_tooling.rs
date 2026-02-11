use fret_diag_protocol::{UiActionScriptV1, UiActionScriptV2, UiActionStepV1, UiActionStepV2};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) struct NormalizedScript {
    pub(crate) normalized: String,
    pub(crate) changed: bool,
}

pub(crate) struct ScriptSchemaReport {
    pub(crate) payload: Value,
    pub(crate) error_scripts: usize,
}

pub(crate) struct ScriptLintReport {
    pub(crate) payload: Value,
    pub(crate) error_scripts: usize,
}

pub(crate) fn normalize_script_from_path(path: &Path) -> Result<NormalizedScript, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    canonicalize_json_value(&mut value);
    let mut normalized = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    normalized.push('\n');

    let mut original = String::from_utf8_lossy(&bytes).to_string();
    original = original.replace("\r\n", "\n");
    if !original.ends_with('\n') {
        original.push('\n');
    }

    Ok(NormalizedScript {
        changed: original != normalized,
        normalized,
    })
}

pub(crate) fn validate_scripts(scripts: &[PathBuf]) -> ScriptSchemaReport {
    let mut error_scripts: usize = 0;
    let mut entries: Vec<Value> = Vec::with_capacity(scripts.len());

    for src in scripts {
        let entry = match validate_script(src) {
            Ok(mut ok) => {
                ok["status"] = Value::String("passed".to_string());
                ok
            }
            Err(err) => {
                error_scripts += 1;
                serde_json::json!({
                    "path": src.display().to_string(),
                    "status": "failed",
                    "error": err,
                })
            }
        };
        entries.push(entry);
    }

    let status = if error_scripts == 0 {
        "passed"
    } else {
        "failed"
    };
    ScriptSchemaReport {
        payload: serde_json::json!({
            "schema_version": 1,
            "status": status,
            "error_scripts": error_scripts,
            "scripts": entries,
        }),
        error_scripts,
    }
}

pub(crate) fn lint_scripts(scripts: &[PathBuf]) -> ScriptLintReport {
    let mut error_scripts: usize = 0;
    let mut entries: Vec<Value> = Vec::with_capacity(scripts.len());

    for src in scripts {
        let entry = match lint_script(src) {
            Ok(v) => v,
            Err(err) => {
                error_scripts += 1;
                serde_json::json!({
                    "path": src.display().to_string(),
                    "status": "failed",
                    "error": err,
                })
            }
        };
        entries.push(entry);
    }

    let status = if error_scripts == 0 {
        "passed"
    } else {
        "failed"
    };
    ScriptLintReport {
        payload: serde_json::json!({
            "schema_version": 1,
            "status": status,
            "error_scripts": error_scripts,
            "scripts": entries,
        }),
        error_scripts,
    }
}

fn validate_script(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32;

    match schema_version {
        1 => {
            let _script: UiActionScriptV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "path": path.display().to_string(),
                "schema_version": 1,
            }))
        }
        2 => {
            let _script: UiActionScriptV2 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "path": path.display().to_string(),
                "schema_version": 2,
            }))
        }
        _ => Err(format!(
            "unknown script schema_version (expected 1 or 2): {}",
            schema_version
        )),
    }
}

fn lint_script(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32;

    let mut findings: Vec<Value> = Vec::new();
    let (declared_required, inferred_required, step_summary) = match schema_version {
        1 => {
            let script: UiActionScriptV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            (
                script
                    .meta
                    .as_ref()
                    .map(|m| m.required_capabilities.clone())
                    .unwrap_or_default(),
                infer_required_capabilities_v1(&script),
                summarize_steps_v1(&script),
            )
        }
        2 => {
            let script: UiActionScriptV2 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            (
                script
                    .meta
                    .as_ref()
                    .map(|m| m.required_capabilities.clone())
                    .unwrap_or_default(),
                infer_required_capabilities_v2(&script),
                summarize_steps_v2(&script),
            )
        }
        _ => {
            return Err(format!(
                "unknown script schema_version (expected 1 or 2): {}",
                schema_version
            ));
        }
    };

    let mut required = declared_required.clone();
    for cap in &inferred_required {
        if !required.iter().any(|c| c == cap) {
            required.push(cap.clone());
        }
    }

    if inferred_required.iter().any(|c| c == "diag.screenshot_png")
        && !declared_required.iter().any(|c| c == "diag.screenshot_png")
    {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "capability.inferred_missing",
            "message": "script uses capture_screenshot, which implies required capability diag.screenshot_png",
        }));
    }

    if inferred_required.iter().any(|c| c == "diag.script_v2")
        && !declared_required.iter().any(|c| c == "diag.script_v2")
    {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "capability.inferred_missing",
            "message": "script schema_version=2 implies required capability diag.script_v2",
        }));
    }

    if step_summary["capture_bundle_count"].as_u64().unwrap_or(0) == 0 {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.no_capture_bundle",
            "message": "script has no capture_bundle step; successful runs may produce no portable bundle.json",
        }));
    }

    let wait_frames_max = step_summary["wait_frames_max"].as_u64().unwrap_or(0);
    if wait_frames_max >= 120 {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.sleep_wait_frames",
            "message": format!("script contains wait_frames up to {wait_frames_max}; prefer wait_until/click_stable/wait_bounds_stable"),
        }));
    }

    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "status": "passed",
        "schema_version": schema_version,
        "capabilities": {
            "declared_required": declared_required,
            "inferred_required": inferred_required,
            "required": required,
        },
        "steps": step_summary,
        "findings": findings,
    }))
}

fn infer_required_capabilities_v1(script: &UiActionScriptV1) -> Vec<String> {
    let mut caps: Vec<String> = Vec::new();
    for step in &script.steps {
        if matches!(step, UiActionStepV1::CaptureScreenshot { .. }) {
            push_cap(&mut caps, "diag.screenshot_png");
        }
    }
    caps
}

fn infer_required_capabilities_v2(script: &UiActionScriptV2) -> Vec<String> {
    let mut caps: Vec<String> = Vec::new();
    push_cap(&mut caps, "diag.script_v2");
    for step in &script.steps {
        if matches!(step, UiActionStepV2::CaptureScreenshot { .. }) {
            push_cap(&mut caps, "diag.screenshot_png");
        }
    }
    caps
}

fn summarize_steps_v1(script: &UiActionScriptV1) -> Value {
    let mut capture_bundle_count: u64 = 0;
    let mut wait_frames_max: u64 = 0;
    for step in &script.steps {
        match step {
            UiActionStepV1::CaptureBundle { .. } => capture_bundle_count += 1,
            UiActionStepV1::WaitFrames { n } => wait_frames_max = wait_frames_max.max(*n as u64),
            _ => {}
        }
    }
    serde_json::json!({
        "count": script.steps.len(),
        "capture_bundle_count": capture_bundle_count,
        "wait_frames_max": wait_frames_max,
    })
}

fn summarize_steps_v2(script: &UiActionScriptV2) -> Value {
    let mut capture_bundle_count: u64 = 0;
    let mut wait_frames_max: u64 = 0;
    let mut click_stable_count: u64 = 0;
    let mut wait_bounds_stable_count: u64 = 0;

    for step in &script.steps {
        match step {
            UiActionStepV2::CaptureBundle { .. } => capture_bundle_count += 1,
            UiActionStepV2::WaitFrames { n } => wait_frames_max = wait_frames_max.max(*n as u64),
            UiActionStepV2::ClickStable { .. } => click_stable_count += 1,
            UiActionStepV2::WaitBoundsStable { .. } => wait_bounds_stable_count += 1,
            _ => {}
        }
    }
    serde_json::json!({
        "count": script.steps.len(),
        "capture_bundle_count": capture_bundle_count,
        "wait_frames_max": wait_frames_max,
        "click_stable_count": click_stable_count,
        "wait_bounds_stable_count": wait_bounds_stable_count,
    })
}

fn push_cap(caps: &mut Vec<String>, cap: &str) {
    if caps.iter().any(|c| c == cap) {
        return;
    }
    caps.push(cap.to_string());
}

pub(crate) fn canonicalize_json_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut old = std::mem::take(map);
            let mut keys: Vec<String> = old.keys().cloned().collect();
            keys.sort();
            for k in keys {
                if let Some(mut v) = old.remove(&k) {
                    canonicalize_json_value(&mut v);
                    map.insert(k, v);
                }
            }
        }
        Value::Array(values) => {
            for v in values {
                canonicalize_json_value(v);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_sorts_keys_recursively() {
        let mut value = serde_json::json!({
            "b": 1,
            "a": {"z": 1, "y": 2},
            "c": [{"b": 2, "a": 1}]
        });
        canonicalize_json_value(&mut value);
        let s = serde_json::to_string_pretty(&value).expect("pretty json should succeed");
        assert!(s.contains("\"a\""));
        assert!(s.find("\"a\"").unwrap() < s.find("\"b\"").unwrap());
        assert!(s.find("\"y\"").unwrap() < s.find("\"z\"").unwrap());
    }

    #[test]
    fn lint_infers_screenshot_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::CaptureScreenshot {
                label: None,
                timeout_frames: 1,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.script_v2"));
        assert!(inferred.iter().any(|c| c == "diag.screenshot_png"));
    }
}
