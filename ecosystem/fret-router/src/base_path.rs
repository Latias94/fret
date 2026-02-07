use crate::path::normalize_path;

/// Normalize a deployment base path.
///
/// Canonical form rules:
/// - empty input and `/` become `""` (no base path)
/// - always starts with `/` when non-empty
/// - duplicate separators are collapsed
/// - trailing `/` is removed
pub fn normalize_base_path(base_path: &str) -> String {
    let trimmed = base_path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let without_fragment = trimmed.split('#').next().unwrap_or_default();
    let without_query = without_fragment.split('?').next().unwrap_or_default();
    let normalized = normalize_path(without_query);

    if normalized == "/" {
        String::new()
    } else {
        normalized
    }
}

/// Apply a normalized base path to an app-relative path.
pub fn apply_base_path(path: &str, base_path: &str) -> String {
    let base = normalize_base_path(base_path);
    let normalized_path = normalize_path(path);

    if base.is_empty() {
        return normalized_path;
    }

    if normalized_path == "/" {
        return base;
    }

    format!("{base}{normalized_path}")
}

/// Strip deployment base path from an absolute path.
///
/// Returns `None` when `path` is not under `base_path`.
pub fn strip_base_path(path: &str, base_path: &str) -> Option<String> {
    let base = normalize_base_path(base_path);
    let normalized_path = normalize_path(path);

    if base.is_empty() {
        return Some(normalized_path);
    }

    if normalized_path == base {
        return Some("/".to_string());
    }

    let prefix = format!("{base}/");
    if normalized_path.starts_with(prefix.as_str()) {
        Some(normalized_path[base.len()..].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_base_path, normalize_base_path, strip_base_path};

    #[test]
    fn normalize_base_path_collapses_to_canonical_form() {
        assert_eq!(normalize_base_path(""), "");
        assert_eq!(normalize_base_path("/"), "");
        assert_eq!(normalize_base_path("app"), "/app");
        assert_eq!(normalize_base_path("///app//v1/"), "/app/v1");
        assert_eq!(normalize_base_path("/app?v=1#x"), "/app");
    }

    #[test]
    fn apply_base_path_joins_paths() {
        assert_eq!(apply_base_path("/", "/app"), "/app");
        assert_eq!(apply_base_path("/users/42", "/app"), "/app/users/42");
        assert_eq!(apply_base_path("/users/42", ""), "/users/42");
    }

    #[test]
    fn strip_base_path_extracts_app_relative_path() {
        assert_eq!(strip_base_path("/app", "/app"), Some("/".to_string()));
        assert_eq!(
            strip_base_path("/app/users/42", "/app"),
            Some("/users/42".to_string())
        );
        assert_eq!(strip_base_path("/other/users/42", "/app"), None);
        assert_eq!(
            strip_base_path("/users/42", ""),
            Some("/users/42".to_string())
        );
    }
}
