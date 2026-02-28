use std::collections::BTreeMap;

/// Env prefixes that tooling scrubs from the inherited parent environment in `--launch` mode.
///
/// The runtime's config resolution is "env overrides config", so leaving *any* `FRET_DIAG_*`
/// key set in the parent shell can silently override the per-run `diag.config.json` that tooling writes.
pub(crate) const TOOL_LAUNCH_SCRUB_ENV_PREFIXES: &[&str] = &["FRET_DIAG_"];

/// Env keys that tooling treats as reserved (tooling-owned) in `--launch` mode.
///
/// These must not be overridden via `--env KEY=VALUE` because tooling sets them to enforce a
/// deterministic per-run layout (paths + config file).
pub(crate) const TOOL_LAUNCH_RESERVED_ENV_KEYS: &[&str] = &[
    "FRET_DIAG",
    "FRET_DIAG_DIR",
    "FRET_DIAG_TRIGGER_PATH",
    "FRET_DIAG_READY_PATH",
    "FRET_DIAG_EXIT_PATH",
    "FRET_DIAG_CONFIG_PATH",
    "FRET_DIAG_SCRIPT_PATH",
    "FRET_DIAG_SCRIPT_TRIGGER_PATH",
    "FRET_DIAG_SCRIPT_RESULT_PATH",
    "FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH",
    "FRET_DIAG_PICK_TRIGGER_PATH",
    "FRET_DIAG_PICK_RESULT_PATH",
    "FRET_DIAG_PICK_RESULT_TRIGGER_PATH",
    "FRET_DIAG_INSPECT_PATH",
    "FRET_DIAG_INSPECT_TRIGGER_PATH",
    "FRET_DIAG_SCREENSHOT_REQUEST_PATH",
    "FRET_DIAG_SCREENSHOT_TRIGGER_PATH",
    "FRET_DIAG_SCREENSHOT_RESULT_PATH",
    "FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH",
];

pub(crate) fn tool_launch_env_key_is_reserved(key: &str) -> bool {
    TOOL_LAUNCH_RESERVED_ENV_KEYS.iter().any(|k| *k == key)
}

pub(crate) fn scrub_inherited_env_for_tool_launch(
    base: &BTreeMap<String, String>,
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut env = base.clone();
    let mut scrubbed: Vec<String> = Vec::new();

    let keys_to_remove: Vec<String> = env
        .keys()
        .filter(|k| {
            TOOL_LAUNCH_SCRUB_ENV_PREFIXES
                .iter()
                .any(|prefix| k.starts_with(prefix))
        })
        .cloned()
        .collect();

    for key in keys_to_remove {
        if env.remove(&key).is_some() {
            scrubbed.push(key);
        }
    }
    scrubbed.sort();
    (env, scrubbed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_removes_all_fret_diag_prefix_keys_and_reports_them() {
        let mut base = BTreeMap::<String, String>::new();
        base.insert("FRET_DIAG_MAX_SNAPSHOTS".to_string(), "999".to_string());
        base.insert("FRET_DIAG_RENDERER_PERF".to_string(), "1".to_string());
        base.insert(
            "FRET_DIAG_BUNDLE_WRITE_SCHEMA2".to_string(),
            "1".to_string(),
        );
        base.insert("FRET_SOMETHING_ELSE".to_string(), "1".to_string());

        let (scrubbed, keys) = scrub_inherited_env_for_tool_launch(&base);
        assert!(scrubbed.get("FRET_DIAG_MAX_SNAPSHOTS").is_none());
        assert!(scrubbed.get("FRET_DIAG_RENDERER_PERF").is_none());
        assert!(scrubbed.get("FRET_DIAG_BUNDLE_WRITE_SCHEMA2").is_none());
        assert_eq!(
            scrubbed.get("FRET_SOMETHING_ELSE").map(|v| v.as_str()),
            Some("1")
        );
        assert!(keys.iter().any(|k| k == "FRET_DIAG_MAX_SNAPSHOTS"));
        assert!(keys.iter().any(|k| k == "FRET_DIAG_RENDERER_PERF"));
        assert!(keys.iter().any(|k| k == "FRET_DIAG_BUNDLE_WRITE_SCHEMA2"));
    }
}
