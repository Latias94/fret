use fret_diag_protocol::{
    UiActionScriptV1, UiActionScriptV2, UiActionStepV1, UiActionStepV2, UiPointerKindV1,
    UiPredicateV1, UiWindowTargetV1,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn step_type_string_from_serializable<S: serde::Serialize>(step: &S) -> String {
    serde_json::to_value(step)
        .ok()
        .and_then(|v| {
            v.get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) struct NormalizedScript {
    pub(crate) normalized: String,
    pub(crate) changed: bool,
    pub(crate) write_path: PathBuf,
    pub(crate) redirect_chain: Vec<PathBuf>,
}

pub(crate) struct ScriptSchemaReport {
    pub(crate) payload: Value,
    pub(crate) error_scripts: usize,
}

pub(crate) struct ScriptLintReport {
    pub(crate) payload: Value,
    pub(crate) error_scripts: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedScriptJson {
    pub(crate) value: Value,
    pub(crate) write_path: PathBuf,
    pub(crate) redirect_chain: Vec<PathBuf>,
}

pub(crate) fn normalize_script_from_path(path: &Path) -> Result<NormalizedScript, String> {
    let resolved = read_script_json_resolving_redirects(path)?;

    let mut value = resolved.value;
    canonicalize_json_value(&mut value);
    let mut normalized = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    normalized.push('\n');

    let bytes = std::fs::read(&resolved.write_path).map_err(|e| e.to_string())?;
    let mut original = String::from_utf8_lossy(&bytes).to_string();
    original = original.replace("\r\n", "\n");
    if !original.ends_with('\n') {
        original.push('\n');
    }

    Ok(NormalizedScript {
        changed: original != normalized,
        normalized,
        write_path: resolved.write_path,
        redirect_chain: resolved.redirect_chain,
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

pub(crate) fn preflight_strict_termination_issues(
    scripts: &[PathBuf],
) -> Result<Vec<Value>, String> {
    let mut issues: Vec<Value> = Vec::new();

    for src in scripts {
        let resolved = read_script_json_resolving_redirects(src)?;
        let value = resolved.value;

        let schema_version = crate::compat::script::script_schema_version_from_value(&value);

        let mut push_issue = |code: &'static str,
                              severity: &'static str,
                              message: String,
                              step_index: Option<u64>| {
            issues.push(serde_json::json!({
                "path": src.display().to_string(),
                "resolved_path": resolved.write_path.display().to_string(),
                "schema_version": schema_version,
                "severity": severity,
                "code": code,
                "message": message,
                "step_index": step_index,
            }));
        };

        match schema_version {
            1 => {
                let script: UiActionScriptV1 =
                    serde_json::from_value(value).map_err(|e| e.to_string())?;
                let last_capture_bundle_index: Option<usize> =
                    script.steps.iter().enumerate().rev().find_map(|(i, step)| {
                        matches!(step, UiActionStepV1::CaptureBundle { .. }).then_some(i)
                    });

                if let Some(last) = script.steps.last()
                    && matches!(last, UiActionStepV1::WaitFrames { .. })
                {
                    let idx = script.steps.len().saturating_sub(1) as u64;
                    push_issue(
                        "script.ends_with_wait_frames",
                        "error",
                        "script ends with wait_frames; smoke/gate suites require deterministic termination (prefer wait_until + capture_bundle as the final step)".to_string(),
                        Some(idx),
                    );
                }

                if let Some(capture_idx) = last_capture_bundle_index {
                    if let Some((i, _)) = script
                        .steps
                        .iter()
                        .enumerate()
                        .skip(capture_idx + 1)
                        .find(|(_i, step)| matches!(step, UiActionStepV1::WaitFrames { .. }))
                    {
                        push_issue(
                            "script.wait_frames_after_last_capture_bundle",
                            "error",
                            "script has wait_frames after the final capture_bundle; this can stall indefinitely under occlusion/idle".to_string(),
                            Some(i as u64),
                        );
                    }
                }
            }
            2 => {
                let script: UiActionScriptV2 =
                    serde_json::from_value(value).map_err(|e| e.to_string())?;
                let last_capture_bundle_index: Option<usize> =
                    script.steps.iter().enumerate().rev().find_map(|(i, step)| {
                        matches!(step, UiActionStepV2::CaptureBundle { .. }).then_some(i)
                    });

                if let Some(last) = script.steps.last()
                    && matches!(last, UiActionStepV2::WaitFrames { .. })
                {
                    let idx = script.steps.len().saturating_sub(1) as u64;
                    push_issue(
                        "script.ends_with_wait_frames",
                        "error",
                        "script ends with wait_frames; smoke/gate suites require deterministic termination (prefer wait_until + capture_bundle as the final step)".to_string(),
                        Some(idx),
                    );
                }

                if let Some(last) = script.steps.last()
                    && matches!(last, UiActionStepV2::WaitMs { .. })
                {
                    let idx = script.steps.len().saturating_sub(1) as u64;
                    push_issue(
                        "script.ends_with_wait_ms",
                        "error",
                        "script ends with wait_ms; smoke/gate suites require deterministic termination (prefer wait_until + capture_bundle as the final step)".to_string(),
                        Some(idx),
                    );
                }

                if let Some(capture_idx) = last_capture_bundle_index {
                    if let Some((i, _)) = script
                        .steps
                        .iter()
                        .enumerate()
                        .skip(capture_idx + 1)
                        .find(|(_i, step)| matches!(step, UiActionStepV2::WaitFrames { .. }))
                    {
                        push_issue(
                            "script.wait_frames_after_last_capture_bundle",
                            "error",
                            "script has wait_frames after the final capture_bundle; this can stall indefinitely under occlusion/idle".to_string(),
                            Some(i as u64),
                        );
                    }
                    if let Some((i, _)) = script
                        .steps
                        .iter()
                        .enumerate()
                        .skip(capture_idx + 1)
                        .find(|(_i, step)| matches!(step, UiActionStepV2::WaitMs { .. }))
                    {
                        push_issue(
                            "script.wait_ms_after_last_capture_bundle",
                            "error",
                            "script has wait_ms after the final capture_bundle; this can stall indefinitely under occlusion/idle".to_string(),
                            Some(i as u64),
                        );
                    }
                }
            }
            _ => {
                push_issue(
                    "script.schema_version_unknown",
                    "error",
                    format!("unknown script schema_version (expected 1 or 2): {schema_version}"),
                    None,
                );
            }
        }
    }

    Ok(issues)
}

fn validate_script(path: &Path) -> Result<Value, String> {
    let resolved = read_script_json_resolving_redirects(path)?;
    let value = resolved.value;

    let schema_version = crate::compat::script::script_schema_version_from_value(&value);

    match schema_version {
        1 => {
            let _script: UiActionScriptV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "path": path.display().to_string(),
                "resolved_path": resolved.write_path.display().to_string(),
                "redirect_chain": resolved.redirect_chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "schema_version": 1,
            }))
        }
        2 => {
            let _script: UiActionScriptV2 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "path": path.display().to_string(),
                "resolved_path": resolved.write_path.display().to_string(),
                "redirect_chain": resolved.redirect_chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
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
    let resolved = read_script_json_resolving_redirects(path)?;
    let value = resolved.value;

    let schema_version = crate::compat::script::script_schema_version_from_value(&value);

    let mut findings: Vec<Value> = Vec::new();
    let (declared_required, inferred_required, step_summary, meta_tags) = match schema_version {
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
                script
                    .meta
                    .as_ref()
                    .map(|m| m.tags.clone())
                    .unwrap_or_default(),
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
                script
                    .meta
                    .as_ref()
                    .map(|m| m.tags.clone())
                    .unwrap_or_default(),
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

    if !resolved.redirect_chain.is_empty() {
        findings.push(serde_json::json!({
            "severity": "info",
            "code": "script.redirect_resolved",
            "message": "script path is a redirect stub; tooling resolved it before execution",
            "resolved_path": resolved.write_path.display().to_string(),
            "redirect_chain": resolved.redirect_chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        }));
    }

    if schema_version == 1 {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.schema_v1_deprecated",
            "message": "script schema_version=1 is deprecated; prefer schema_version=2 (tooling upgrades v1→v2 on execution)",
        }));
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
            "message": "script has no capture_bundle step; successful runs may produce no portable bundle artifact",
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

    let wait_ms_max = step_summary["wait_ms_max"].as_u64().unwrap_or(0);
    if wait_ms_max >= 2_000 {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.sleep_wait_ms",
            "message": format!("script contains wait_ms up to {wait_ms_max}; prefer wait_until on semantic predicates for deterministic gates"),
        }));
    }

    if step_summary["ends_with_wait_frames"].as_bool() == Some(true) {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.ends_with_wait_frames",
            "message": "script ends with wait_frames; prefer wait_until, or end with capture_bundle/assert so termination is deterministic under occlusion/idle",
        }));
    }

    if step_summary["ends_with_wait_ms"].as_bool() == Some(true) {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.ends_with_wait_ms",
            "message": "script ends with wait_ms; prefer wait_until, or end with capture_bundle/assert so termination is deterministic under occlusion/idle",
        }));
    }

    if step_summary["wait_frames_after_last_capture_bundle"].as_bool() == Some(true) {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.wait_frames_after_last_capture_bundle",
            "message": "script has wait_frames after the final capture_bundle; this can stall indefinitely if the remaining window becomes occluded/idle and stops producing frames",
        }));
    }

    if step_summary["wait_ms_after_last_capture_bundle"].as_bool() == Some(true) {
        findings.push(serde_json::json!({
            "severity": "warning",
            "code": "script.wait_ms_after_last_capture_bundle",
            "message": "script has wait_ms after the final capture_bundle; this can stall indefinitely if the remaining window becomes occluded/idle and stops producing frames",
        }));
    }

    let has_wait_after_final_capture_bundle =
        step_summary["wait_frames_after_last_capture_bundle"].as_bool() == Some(true)
            || step_summary["wait_ms_after_last_capture_bundle"].as_bool() == Some(true);
    if !has_wait_after_final_capture_bundle
        && step_summary["capture_bundle_not_last"].as_bool() == Some(true)
    {
        findings.push(serde_json::json!({
            "severity": "info",
            "code": "script.capture_bundle_not_last",
            "message": "final capture_bundle is not the last step; if this is a gate script, prefer capture_bundle as the final step so evidence always represents the terminal state",
        }));
    }

    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "resolved_path": resolved.write_path.display().to_string(),
        "redirect_chain": resolved.redirect_chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "status": "passed",
        "schema_version": schema_version,
        "meta": {
            "tags": meta_tags,
        },
        "capabilities": {
            "declared_required": declared_required,
            "inferred_required": inferred_required,
            "required": required,
        },
        "steps": step_summary,
        "findings": findings,
    }))
}

pub(crate) fn read_script_json_resolving_redirects(
    path: &Path,
) -> Result<ResolvedScriptJson, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    resolve_script_json_redirects_from_value(path, value)
}

pub(crate) fn resolve_script_json_redirects_from_value(
    path: &Path,
    mut value: Value,
) -> Result<ResolvedScriptJson, String> {
    const MAX_REDIRECT_DEPTH: usize = 8;

    fn find_repo_root_for_path(p: &Path) -> Option<PathBuf> {
        let mut cur = p;
        while let Some(parent) = cur.parent() {
            if parent.join("Cargo.toml").is_file() {
                return Some(parent.to_path_buf());
            }
            cur = parent;
        }
        None
    }

    fn resolve_to_path(from: &Path, to: &str) -> PathBuf {
        let to_path = PathBuf::from(to);
        if to_path.is_absolute() {
            return to_path;
        }

        let looks_repo_relative = to.starts_with("tools/")
            || to.starts_with("crates/")
            || to.starts_with("apps/")
            || to.starts_with("docs/")
            || to.starts_with(".fret/");

        if looks_repo_relative {
            if let Some(root) = find_repo_root_for_path(from) {
                return root.join(to_path);
            }
        }

        from.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(to_path)
    }

    let mut read_path = path.to_path_buf();
    let mut write_path = path.to_path_buf();
    let mut redirect_chain: Vec<PathBuf> = Vec::new();
    let mut visited: BTreeSet<PathBuf> = BTreeSet::new();

    for _ in 0..=MAX_REDIRECT_DEPTH {
        let canon = std::fs::canonicalize(&read_path).unwrap_or_else(|_| read_path.clone());
        if !visited.insert(canon.clone()) {
            return Err(format!(
                "script redirect loop detected (revisiting): {}",
                canon.display()
            ));
        }

        let is_redirect = value
            .get("kind")
            .and_then(|v| v.as_str())
            .map(|s| s == "script_redirect")
            .unwrap_or(false);
        if !is_redirect {
            return Ok(ResolvedScriptJson {
                value,
                write_path,
                redirect_chain,
            });
        }

        let schema_version = crate::compat::script::script_schema_version_from_value(&value);
        if schema_version != 1 {
            return Err(format!(
                "invalid script_redirect schema_version (expected 1): {schema_version}"
            ));
        }

        let to = value
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "invalid script_redirect (missing string field: to)".to_string())?;

        redirect_chain.push(read_path.clone());
        read_path = resolve_to_path(&read_path, to);
        write_path = read_path.clone();

        let bytes = std::fs::read(&read_path).map_err(|e| e.to_string())?;
        value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    }

    Err(format!(
        "script redirect depth exceeded (max_depth={MAX_REDIRECT_DEPTH})"
    ))
}

fn infer_required_capabilities_v1(script: &UiActionScriptV1) -> Vec<String> {
    let mut caps: Vec<String> = Vec::new();
    for step in &script.steps {
        if matches!(step, UiActionStepV1::CaptureScreenshot { .. }) {
            push_cap(&mut caps, "diag.screenshot_png");
        }
        match step {
            UiActionStepV1::WaitUntil { predicate, .. } | UiActionStepV1::Assert { predicate } => {
                infer_required_capabilities_from_predicate(&mut caps, predicate);
            }
            _ => {}
        }
    }
    caps
}

fn window_target_requires_multi_window(window: &UiWindowTargetV1) -> bool {
    matches!(
        window,
        UiWindowTargetV1::FirstSeenOther
            | UiWindowTargetV1::LastSeenOther
            | UiWindowTargetV1::WindowFfi { .. }
    )
}

fn infer_required_capabilities_from_predicate(caps: &mut Vec<String>, predicate: &UiPredicateV1) {
    match predicate {
        UiPredicateV1::PlatformWindowReceiverAtCursorIs { window } => {
            push_cap(caps, "diag.platform_window_receiver_at_cursor_v1");
            if window_target_requires_multi_window(window) {
                push_cap(caps, "diag.multi_window");
            }
        }
        UiPredicateV1::WindowStyleEffectiveIs { window, .. } => {
            push_cap(caps, "diag.window_style_snapshot");
            if window_target_requires_multi_window(window) {
                push_cap(caps, "diag.multi_window");
            }
        }
        UiPredicateV1::WindowBackgroundMaterialEffectiveIs { window, .. } => {
            push_cap(caps, "diag.window_background_material_snapshot");
            if window_target_requires_multi_window(window) {
                push_cap(caps, "diag.multi_window");
            }
        }
        _ => {}
    }
}

fn infer_required_capabilities_v2(script: &UiActionScriptV2) -> Vec<String> {
    let mut caps: Vec<String> = Vec::new();
    fn step_window_target(step: &UiActionStepV2) -> Option<&UiWindowTargetV1> {
        match step {
            UiActionStepV2::Click { window, .. }
            | UiActionStepV2::Tap { window, .. }
            | UiActionStepV2::Pinch { window, .. }
            | UiActionStepV2::MovePointer { window, .. }
            | UiActionStepV2::PointerDown { window, .. }
            | UiActionStepV2::DragPointer { window, .. }
            | UiActionStepV2::PointerMove { window, .. }
            | UiActionStepV2::PointerUp { window, .. }
            | UiActionStepV2::MovePointerSweep { window, .. }
            | UiActionStepV2::Wheel { window, .. }
            | UiActionStepV2::WheelBurst { window, .. }
            | UiActionStepV2::WaitUntil { window, .. }
            | UiActionStepV2::Assert { window, .. }
            | UiActionStepV2::Activate { window, .. }
            | UiActionStepV2::Focus { window, .. }
            | UiActionStepV2::ClickStable { window, .. }
            | UiActionStepV2::ClickSelectableTextSpanStable { window, .. }
            | UiActionStepV2::WaitBoundsStable { window, .. }
            | UiActionStepV2::EnsureVisible { window, .. }
            | UiActionStepV2::ScrollIntoView { window, .. }
            | UiActionStepV2::TypeTextInto { window, .. }
            | UiActionStepV2::MenuSelect { window, .. }
            | UiActionStepV2::MenuSelectPath { window, .. }
            | UiActionStepV2::DragTo { window, .. }
            | UiActionStepV2::SetSliderValue { window, .. }
            | UiActionStepV2::SetWindowInnerSize { window, .. }
            | UiActionStepV2::SetWindowStyle { window, .. }
            | UiActionStepV2::SetWindowOuterPosition { window, .. }
            | UiActionStepV2::SetCursorInWindow { window, .. }
            | UiActionStepV2::SetCursorInWindowLogical { window, .. }
            | UiActionStepV2::SetMouseButtons { window, .. }
            | UiActionStepV2::RaiseWindow { window, .. }
            | UiActionStepV2::DragPointerUntil { window, .. } => window.as_ref(),
            _ => None,
        }
    }
    fn step_predicate(step: &UiActionStepV2) -> Option<&UiPredicateV1> {
        match step {
            UiActionStepV2::WaitUntil { predicate, .. }
            | UiActionStepV2::Assert { predicate, .. }
            | UiActionStepV2::DragPointerUntil { predicate, .. } => Some(predicate),
            _ => None,
        }
    }
    fn step_pointer_kind(step: &UiActionStepV2) -> Option<UiPointerKindV1> {
        match step {
            UiActionStepV2::Click { pointer_kind, .. }
            | UiActionStepV2::Tap { pointer_kind, .. }
            | UiActionStepV2::LongPress { pointer_kind, .. }
            | UiActionStepV2::Swipe { pointer_kind, .. }
            | UiActionStepV2::Pinch { pointer_kind, .. }
            | UiActionStepV2::MovePointer { pointer_kind, .. }
            | UiActionStepV2::PointerDown { pointer_kind, .. }
            | UiActionStepV2::DragPointer { pointer_kind, .. }
            | UiActionStepV2::PointerMove { pointer_kind, .. }
            | UiActionStepV2::PointerUp { pointer_kind, .. }
            | UiActionStepV2::MovePointerSweep { pointer_kind, .. }
            | UiActionStepV2::Wheel { pointer_kind, .. }
            | UiActionStepV2::WheelBurst { pointer_kind, .. }
            | UiActionStepV2::ClickStable { pointer_kind, .. }
            | UiActionStepV2::ClickSelectableTextSpanStable { pointer_kind, .. }
            | UiActionStepV2::ScrollIntoView { pointer_kind, .. }
            | UiActionStepV2::TypeTextInto { pointer_kind, .. }
            | UiActionStepV2::MenuSelect { pointer_kind, .. }
            | UiActionStepV2::MenuSelectPath { pointer_kind, .. }
            | UiActionStepV2::DragTo { pointer_kind, .. }
            | UiActionStepV2::SetSliderValue { pointer_kind, .. }
            | UiActionStepV2::DragPointerUntil { pointer_kind, .. } => *pointer_kind,
            _ => None,
        }
    }
    push_cap(&mut caps, "diag.script_v2");
    for step in &script.steps {
        if let Some(window) = step_window_target(step)
            && window_target_requires_multi_window(window)
        {
            push_cap(&mut caps, "diag.multi_window");
        }
        if let Some(predicate) = step_predicate(step) {
            infer_required_capabilities_from_predicate(&mut caps, predicate);
        }
        if matches!(step, UiActionStepV2::CaptureScreenshot { .. }) {
            push_cap(&mut caps, "diag.screenshot_png");
        }
        if matches!(step, UiActionStepV2::Ime { .. }) {
            push_cap(&mut caps, "diag.inject_ime");
        }
        if matches!(step, UiActionStepV2::WaitShortcutRoutingTrace { .. }) {
            push_cap(&mut caps, "diag.shortcut_routing_trace");
        }
        if matches!(step, UiActionStepV2::WaitOverlayPlacementTrace { .. }) {
            push_cap(&mut caps, "diag.overlay_placement_trace");
        }
        if matches!(step, UiActionStepV2::WheelBurst { .. }) {
            push_cap(&mut caps, "diag.wheel_burst_inject");
        }
        if matches!(
            step,
            UiActionStepV2::SetCursorScreenPos { .. }
                | UiActionStepV2::SetCursorInWindow { .. }
                | UiActionStepV2::SetCursorInWindowLogical { .. }
        ) {
            push_cap(&mut caps, "diag.cursor_screen_pos_override");
        }
        if matches!(step, UiActionStepV2::SetMouseButtons { .. }) {
            push_cap(&mut caps, "diag.mouse_buttons_override");
        }
        if matches!(
            step,
            UiActionStepV2::SetClipboardText { .. } | UiActionStepV2::AssertClipboardText { .. }
        ) {
            push_cap(&mut caps, "diag.clipboard_text");
        }
        if matches!(step, UiActionStepV2::SetClipboardForceUnavailable { .. }) {
            push_cap(&mut caps, "diag.clipboard_force_unavailable");
        }
        if matches!(step, UiActionStepV2::InjectIncomingOpen { .. }) {
            push_cap(&mut caps, "diag.incoming_open_inject");
        }
        if matches!(step, UiActionStepV2::SetWindowStyle { .. }) {
            push_cap(&mut caps, "diag.window_style_patch_v1");
        }
        if matches!(step, UiActionStepV2::Tap { .. }) {
            push_cap(&mut caps, "diag.gesture_tap");
        }
        if matches!(step, UiActionStepV2::LongPress { .. }) {
            push_cap(&mut caps, "diag.gesture_long_press");
        }
        if matches!(step, UiActionStepV2::Swipe { .. }) {
            push_cap(&mut caps, "diag.gesture_swipe");
        }
        if matches!(step, UiActionStepV2::Pinch { .. }) {
            push_cap(&mut caps, "diag.gesture_pinch");
        }
        if matches!(step_pointer_kind(step), Some(UiPointerKindV1::Touch)) {
            push_cap(&mut caps, "diag.pointer_kind_touch");
        }
        if matches!(step_pointer_kind(step), Some(UiPointerKindV1::Pen)) {
            push_cap(&mut caps, "diag.pointer_kind_pen");
        }
    }
    caps
}

pub(crate) fn infer_required_capabilities_from_value(value: &Value) -> Vec<String> {
    let schema_version = crate::compat::script::script_schema_version_from_value(value);
    match schema_version {
        1 => serde_json::from_value::<UiActionScriptV1>(value.clone())
            .ok()
            .map(|script| infer_required_capabilities_v1(&script))
            .unwrap_or_default(),
        2 => serde_json::from_value::<UiActionScriptV2>(value.clone())
            .ok()
            .map(|script| infer_required_capabilities_v2(&script))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn summarize_steps_v1(script: &UiActionScriptV1) -> Value {
    let mut capture_bundle_count: u64 = 0;
    let mut wait_frames_max: u64 = 0;
    let mut last_capture_bundle_index: Option<u64> = None;

    for (i, step) in script.steps.iter().enumerate() {
        match step {
            UiActionStepV1::CaptureBundle { .. } => {
                capture_bundle_count += 1;
                last_capture_bundle_index = Some(i as u64);
            }
            UiActionStepV1::WaitFrames { n } => wait_frames_max = wait_frames_max.max(*n as u64),
            _ => {}
        }
    }

    let ends_with_wait_frames =
        matches!(script.steps.last(), Some(UiActionStepV1::WaitFrames { .. }));
    let last_step_type = script
        .steps
        .last()
        .map(step_type_string_from_serializable)
        .unwrap_or_else(|| "none".to_string());

    let (
        capture_bundle_not_last,
        wait_frames_after_last_capture_bundle,
        tail_step_count,
        tail_step_types,
    ) = if let Some(last_capture_bundle_index) = last_capture_bundle_index {
        let idx = last_capture_bundle_index as usize;
        let capture_bundle_not_last = idx + 1 < script.steps.len();
        let mut wait_frames_after_last_capture_bundle = false;
        let mut tail_step_types: Vec<String> = Vec::new();
        for step in script.steps.iter().skip(idx + 1) {
            if matches!(step, UiActionStepV1::WaitFrames { .. }) {
                wait_frames_after_last_capture_bundle = true;
            }
            let t = step_type_string_from_serializable(step);
            if tail_step_types
                .last()
                .map(|prev| prev != &t)
                .unwrap_or(true)
            {
                tail_step_types.push(t);
            }
        }
        (
            capture_bundle_not_last,
            wait_frames_after_last_capture_bundle,
            (script.steps.len().saturating_sub(idx + 1)) as u64,
            tail_step_types,
        )
    } else {
        (false, false, 0, Vec::new())
    };

    serde_json::json!({
        "count": script.steps.len(),
        "capture_bundle_count": capture_bundle_count,
        "last_capture_bundle_index": last_capture_bundle_index,
        "capture_bundle_not_last": capture_bundle_not_last,
        "tail_step_count_after_last_capture_bundle": tail_step_count,
        "tail_step_types_after_last_capture_bundle": tail_step_types,
        "wait_frames_after_last_capture_bundle": wait_frames_after_last_capture_bundle,
        "wait_frames_max": wait_frames_max,
        "ends_with_wait_frames": ends_with_wait_frames,
        "last_step_type": last_step_type,
    })
}

fn summarize_steps_v2(script: &UiActionScriptV2) -> Value {
    let mut capture_bundle_count: u64 = 0;
    let mut wait_frames_max: u64 = 0;
    let mut wait_ms_max: u64 = 0;
    let mut click_stable_count: u64 = 0;
    let mut wait_bounds_stable_count: u64 = 0;
    let mut last_capture_bundle_index: Option<u64> = None;

    for (i, step) in script.steps.iter().enumerate() {
        match step {
            UiActionStepV2::CaptureBundle { .. } => {
                capture_bundle_count += 1;
                last_capture_bundle_index = Some(i as u64);
            }
            UiActionStepV2::WaitFrames { n, .. } => {
                wait_frames_max = wait_frames_max.max(*n as u64)
            }
            UiActionStepV2::WaitMs { n_ms, .. } => wait_ms_max = wait_ms_max.max(*n_ms as u64),
            UiActionStepV2::ClickStable { .. } => click_stable_count += 1,
            UiActionStepV2::WaitBoundsStable { .. } => wait_bounds_stable_count += 1,
            _ => {}
        }
    }

    let ends_with_wait_frames =
        matches!(script.steps.last(), Some(UiActionStepV2::WaitFrames { .. }));
    let ends_with_wait_ms = matches!(script.steps.last(), Some(UiActionStepV2::WaitMs { .. }));
    let last_step_type = script
        .steps
        .last()
        .map(step_type_string_from_serializable)
        .unwrap_or_else(|| "none".to_string());

    let (
        capture_bundle_not_last,
        wait_frames_after_last_capture_bundle,
        wait_ms_after_last_capture_bundle,
        tail_step_count,
        tail_step_types,
    ) = if let Some(last_capture_bundle_index) = last_capture_bundle_index {
        let idx = last_capture_bundle_index as usize;
        let capture_bundle_not_last = idx + 1 < script.steps.len();
        let mut wait_frames_after_last_capture_bundle = false;
        let mut wait_ms_after_last_capture_bundle = false;
        let mut tail_step_types: Vec<String> = Vec::new();
        for step in script.steps.iter().skip(idx + 1) {
            if matches!(step, UiActionStepV2::WaitFrames { .. }) {
                wait_frames_after_last_capture_bundle = true;
            }
            if matches!(step, UiActionStepV2::WaitMs { .. }) {
                wait_ms_after_last_capture_bundle = true;
            }
            let t = step_type_string_from_serializable(step);
            if tail_step_types
                .last()
                .map(|prev| prev != &t)
                .unwrap_or(true)
            {
                tail_step_types.push(t);
            }
        }
        (
            capture_bundle_not_last,
            wait_frames_after_last_capture_bundle,
            wait_ms_after_last_capture_bundle,
            (script.steps.len().saturating_sub(idx + 1)) as u64,
            tail_step_types,
        )
    } else {
        (false, false, false, 0, Vec::new())
    };

    serde_json::json!({
        "count": script.steps.len(),
        "capture_bundle_count": capture_bundle_count,
        "last_capture_bundle_index": last_capture_bundle_index,
        "capture_bundle_not_last": capture_bundle_not_last,
        "tail_step_count_after_last_capture_bundle": tail_step_count,
        "tail_step_types_after_last_capture_bundle": tail_step_types,
        "wait_frames_after_last_capture_bundle": wait_frames_after_last_capture_bundle,
        "wait_ms_after_last_capture_bundle": wait_ms_after_last_capture_bundle,
        "wait_frames_max": wait_frames_max,
        "wait_ms_max": wait_ms_max,
        "click_stable_count": click_stable_count,
        "wait_bounds_stable_count": wait_bounds_stable_count,
        "ends_with_wait_frames": ends_with_wait_frames,
        "ends_with_wait_ms": ends_with_wait_ms,
        "last_step_type": last_step_type,
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
    use fret_diag_protocol::{
        UiMouseButtonV1, UiSelectorV1, UiWindowHitTestPatchV1, UiWindowStylePatchV1,
    };

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
                timeout_ms: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.script_v2"));
        assert!(inferred.iter().any(|c| c == "diag.screenshot_png"));
    }

    #[test]
    fn lint_infers_cursor_override_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::SetCursorInWindowLogical {
                window: None,
                x_px: 1.0,
                y_px: 2.0,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(
            inferred
                .iter()
                .any(|c| c == "diag.cursor_screen_pos_override")
        );
    }

    #[test]
    fn lint_infers_pointer_kind_touch_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::Click {
                window: None,
                pointer_kind: Some(UiPointerKindV1::Touch),
                target: UiSelectorV1::TestId {
                    id: "touch-target".to_string(),
                    root_z_index: None,
                },
                button: UiMouseButtonV1::Left,
                click_count: 1,
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.pointer_kind_touch"));
    }

    #[test]
    fn lint_infers_pointer_kind_pen_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::Click {
                window: None,
                pointer_kind: Some(UiPointerKindV1::Pen),
                target: UiSelectorV1::TestId {
                    id: "pen-target".to_string(),
                    root_z_index: None,
                },
                button: UiMouseButtonV1::Left,
                click_count: 1,
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.pointer_kind_pen"));
    }

    #[test]
    fn lint_infers_gesture_tap_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::Tap {
                window: None,
                pointer_kind: None,
                target: UiSelectorV1::TestId {
                    id: "tap-target".to_string(),
                    root_z_index: None,
                },
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.gesture_tap"));
    }

    #[test]
    fn summarize_detects_capture_bundle_trailing_wait_frames_v2() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![
                UiActionStepV2::CaptureBundle {
                    label: Some("end".to_string()),
                    max_snapshots: None,
                },
                UiActionStepV2::WaitFrames { window: None, n: 1 },
            ],
        };

        let summary = summarize_steps_v2(&script);
        assert_eq!(summary["capture_bundle_count"].as_u64(), Some(1));
        assert_eq!(summary["last_capture_bundle_index"].as_u64(), Some(0));
        assert_eq!(summary["capture_bundle_not_last"].as_bool(), Some(true));
        assert_eq!(
            summary["wait_frames_after_last_capture_bundle"].as_bool(),
            Some(true)
        );
        assert_eq!(summary["ends_with_wait_frames"].as_bool(), Some(true));
        assert_eq!(summary["last_step_type"].as_str(), Some("wait_frames"));
    }

    #[test]
    fn strict_termination_preflight_reports_trailing_wait_frames() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-script-tooling-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let script_path = root.join("script.json");
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![
                UiActionStepV2::CaptureBundle {
                    label: Some("end".to_string()),
                    max_snapshots: None,
                },
                UiActionStepV2::WaitFrames { window: None, n: 1 },
            ],
        };
        std::fs::write(
            &script_path,
            serde_json::to_vec_pretty(&script).expect("script json"),
        )
        .unwrap();

        let issues = preflight_strict_termination_issues(&[script_path.clone()]).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|it| {
            it.get("code").and_then(|v| v.as_str()).is_some_and(|c| {
                c == "script.ends_with_wait_frames"
                    || c == "script.wait_frames_after_last_capture_bundle"
            })
        }));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn lint_infers_gesture_pinch_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::Pinch {
                window: None,
                pointer_kind: None,
                target: UiSelectorV1::TestId {
                    id: "pinch-target".to_string(),
                    root_z_index: None,
                },
                delta: 0.25,
                steps: 8,
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.gesture_pinch"));
    }

    #[test]
    fn lint_infers_gesture_long_press_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::LongPress {
                window: None,
                pointer_kind: None,
                target: UiSelectorV1::TestId {
                    id: "press-target".to_string(),
                    root_z_index: None,
                },
                duration_ms: 125,
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.gesture_long_press"));
    }

    #[test]
    fn lint_infers_gesture_swipe_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::Swipe {
                window: None,
                pointer_kind: None,
                target: UiSelectorV1::TestId {
                    id: "swipe-target".to_string(),
                    root_z_index: None,
                },
                delta_x: 0.0,
                delta_y: 100.0,
                steps: 8,
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.gesture_swipe"));
    }

    #[test]
    fn lint_infers_window_style_patch_capability() {
        let script = UiActionScriptV2 {
            schema_version: 2,
            meta: None,
            steps: vec![UiActionStepV2::SetWindowStyle {
                window: None,
                style: UiWindowStylePatchV1 {
                    hit_test: Some(UiWindowHitTestPatchV1::PassthroughAll),
                    ..UiWindowStylePatchV1::default()
                },
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.window_style_patch_v1"));
    }
}
