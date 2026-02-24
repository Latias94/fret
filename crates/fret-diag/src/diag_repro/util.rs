use super::super::*;

pub(super) fn read_tooling_reason_code(path: &Path) -> Option<String> {
    read_json_value(path).and_then(|v| {
        v.get("reason_code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })
}

pub(super) fn pack_defaults_with_fallback(
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
) -> (bool, bool, bool) {
    let mut pack_defaults = (
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
    );
    if !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
        pack_defaults = (true, true, true);
    }
    pack_defaults
}
