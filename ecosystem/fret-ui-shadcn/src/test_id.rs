use std::sync::Arc;

use fret_ui::element::AnyElement;

pub(crate) fn attach_test_id(el: AnyElement, test_id: Arc<str>) -> AnyElement {
    el.test_id(test_id)
}

pub(crate) fn attach_test_id_suffix(
    el: AnyElement,
    prefix: Option<&Arc<str>>,
    suffix: &'static str,
) -> AnyElement {
    let Some(prefix) = prefix else {
        return el;
    };
    attach_test_id(el, Arc::<str>::from(format!("{prefix}-{suffix}")))
}

/// Converts an arbitrary identifier into a stable `test_id`-safe slug.
///
/// This is intentionally conservative: keep ASCII alphanumerics, lowercase them, and replace all
/// other characters with `-`. Callers should treat this as an automation-only surface.
pub(crate) fn test_id_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}
