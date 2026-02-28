use fret_diag_protocol::{
    UiActionScriptV1, UiActionScriptV2, UiActionStepV1, UiActionStepV2, UiPointerKindV1,
    UiWindowTargetV1,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "resolved_path": resolved.write_path.display().to_string(),
        "redirect_chain": resolved.redirect_chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
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
    }
    caps
}

fn infer_required_capabilities_v2(script: &UiActionScriptV2) -> Vec<String> {
    let mut caps: Vec<String> = Vec::new();
    fn window_target_requires_multi_window(window: &UiWindowTargetV1) -> bool {
        matches!(
            window,
            UiWindowTargetV1::FirstSeenOther
                | UiWindowTargetV1::LastSeenOther
                | UiWindowTargetV1::WindowFfi { .. }
        )
    }
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
            | UiActionStepV2::WaitUntil { window, .. }
            | UiActionStepV2::Assert { window, .. }
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
            | UiActionStepV2::SetWindowOuterPosition { window, .. }
            | UiActionStepV2::SetCursorInWindow { window, .. }
            | UiActionStepV2::SetCursorInWindowLogical { window, .. }
            | UiActionStepV2::SetMouseButtons { window, .. }
            | UiActionStepV2::RaiseWindow { window, .. }
            | UiActionStepV2::DragPointerUntil { window, .. } => window.as_ref(),
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
    use fret_diag_protocol::{UiMouseButtonV1, UiSelectorV1};

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
                },
                modifiers: None,
            }],
        };
        let inferred = infer_required_capabilities_v2(&script);
        assert!(inferred.iter().any(|c| c == "diag.gesture_tap"));
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
}
