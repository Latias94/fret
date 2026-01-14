use std::sync::Arc;

/// Returns `Some(message)` when `value` is empty after trimming whitespace.
pub fn required_trimmed(value: &str, message: impl Into<Arc<str>>) -> Option<Arc<str>> {
    value.trim().is_empty().then(|| message.into())
}

/// Returns `Some(message)` when `value` is shorter than `min_len`.
pub fn min_len(value: &str, min_len: usize, message: impl Into<Arc<str>>) -> Option<Arc<str>> {
    (value.chars().count() < min_len).then(|| message.into())
}

/// Returns the first validation error (if any).
pub fn first_error(errors: impl IntoIterator<Item = Option<Arc<str>>>) -> Option<Arc<str>> {
    errors.into_iter().find_map(|e| e)
}
