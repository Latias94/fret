use std::ffi::OsString;

fn parse_strict_runtime_value(value: OsString) -> bool {
    let s = value.to_string_lossy();
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    !matches!(s, "0" | "false" | "no" | "off")
}

/// Returns whether `FRET_STRICT_RUNTIME` enables strict runtime behavior.
///
/// This is intentionally shared across kernel crates so "strict mode" toggles behave consistently.
pub fn strict_runtime_enabled_from_env() -> bool {
    std::env::var_os("FRET_STRICT_RUNTIME").is_some_and(parse_strict_runtime_value)
}
