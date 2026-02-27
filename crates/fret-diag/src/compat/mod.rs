pub(crate) mod bundle;
pub(crate) mod script;

pub(crate) fn normalize_capability(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if raw.contains('.') {
        return Some(raw.to_string());
    }

    let mapped = match raw {
        "script_v2" => "diag.script_v2",
        "screenshot_png" => "diag.screenshot_png",
        "multi_window" => "diag.multi_window",
        "pointer_kind_touch" => "diag.pointer_kind_touch",
        "gesture_pinch" => "diag.gesture_pinch",
        _ => raw,
    };
    Some(mapped.to_string())
}

pub(crate) fn normalize_capability_lossy(raw: &str) -> String {
    normalize_capability(raw).unwrap_or_default()
}
