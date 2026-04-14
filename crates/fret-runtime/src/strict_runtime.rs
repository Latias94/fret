use std::ffi::OsString;

fn parse_strict_runtime_str(value: &str) -> bool {
    let s = value.trim();
    if s.is_empty() {
        return false;
    }
    !matches!(s, "0" | "false" | "no" | "off")
}

fn parse_strict_runtime_value(value: OsString) -> bool {
    parse_strict_runtime_str(&value.to_string_lossy())
}

fn strict_runtime_enabled_from_build_default() -> bool {
    option_env!("FRET_STRICT_RUNTIME_DEFAULT").is_some_and(parse_strict_runtime_str)
}

/// Returns whether `FRET_STRICT_RUNTIME` enables strict runtime behavior.
///
/// This is intentionally shared across kernel crates so "strict mode" toggles behave consistently.
/// Runtime env wins. The build default exists so browser/wasm dev runners can opt into strict mode
/// even though there is no normal process environment at runtime.
pub fn strict_runtime_enabled_from_env() -> bool {
    std::env::var_os("FRET_STRICT_RUNTIME")
        .map(parse_strict_runtime_value)
        .unwrap_or_else(strict_runtime_enabled_from_build_default)
}

#[cfg(test)]
mod tests {
    use super::parse_strict_runtime_str;

    #[test]
    fn parse_strict_runtime_str_treats_common_false_values_as_disabled() {
        for raw in ["", "0", "false", "no", "off", " 0 ", " false "] {
            assert!(!parse_strict_runtime_str(raw), "raw={raw:?}");
        }
    }

    #[test]
    fn parse_strict_runtime_str_treats_other_values_as_enabled() {
        for raw in ["1", "true", "yes", "on", "debug"] {
            assert!(parse_strict_runtime_str(raw), "raw={raw:?}");
        }
    }
}
