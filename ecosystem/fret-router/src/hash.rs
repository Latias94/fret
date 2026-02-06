use crate::query::decode_component;

/// Parse a token-style hash value.
///
/// Examples:
/// - `#ui_gallery` -> `Some("ui_gallery")`
/// - `#/ui_gallery` -> `Some("ui_gallery")`
/// - `#demo=ui_gallery` -> `None` (query-style hash)
pub fn hash_token(hash: &str) -> Option<String> {
    let body = hash.strip_prefix('#').unwrap_or(hash).trim();
    if body.is_empty() {
        return None;
    }
    if body.contains('=') || body.contains('&') {
        return None;
    }

    let token = body
        .trim_start_matches('/')
        .split('/')
        .find(|segment| !segment.is_empty())?;
    let token = decode_component(token);
    if token.is_empty() { None } else { Some(token) }
}

/// Backward-compatible helper for legacy `#token` matching behavior.
pub fn hash_contains_token(hash: &str, token: &str) -> bool {
    let body = hash.strip_prefix('#').unwrap_or(hash);
    !body.is_empty() && body.contains(token)
}

#[cfg(test)]
mod tests {
    use super::hash_token;

    #[test]
    fn hash_token_handles_plain_and_path_forms() {
        assert_eq!(hash_token("#ui_gallery").as_deref(), Some("ui_gallery"));
        assert_eq!(hash_token("#/ui_gallery").as_deref(), Some("ui_gallery"));
        assert_eq!(hash_token("#demo=ui_gallery"), None);
    }
}
