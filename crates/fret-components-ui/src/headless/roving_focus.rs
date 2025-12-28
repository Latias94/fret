//! Small helpers for APG-aligned "roving tabindex" style navigation.
//!
//! This module is intentionally lightweight: it only provides index math and selection fallback
//! decisions. Event handling and focus requests live in the runtime substrate (`fret-ui`).

pub fn first_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|d| !*d)
}

pub fn last_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|d| !*d)
}

pub fn next_enabled(disabled: &[bool], current: usize, forward: bool, wrap: bool) -> Option<usize> {
    let len = disabled.len();
    if len == 0 || current >= len {
        return None;
    }

    let is_disabled = |idx: usize| disabled.get(idx).copied().unwrap_or(false);

    if wrap {
        for step in 1..=len {
            let idx = if forward {
                (current + step) % len
            } else {
                (current + len - (step % len)) % len
            };
            if !is_disabled(idx) {
                return Some(idx);
            }
        }
        None
    } else if forward {
        ((current + 1)..len).find(|&i| !is_disabled(i))
    } else if current > 0 {
        (0..current).rev().find(|&i| !is_disabled(i))
    } else {
        None
    }
}

pub fn active_index_from_str_keys(
    keys: &[std::sync::Arc<str>],
    selected: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    if keys.len() != disabled.len() {
        return first_enabled(disabled);
    }

    if let Some(selected) = selected
        && let Some(idx) = keys.iter().position(|k| k.as_ref() == selected)
        && !disabled.get(idx).copied().unwrap_or(true)
    {
        return Some(idx);
    }

    first_enabled(disabled)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn next_enabled_wrap_skips_disabled_and_wraps() {
        let disabled = [false, true, false];
        assert_eq!(next_enabled(&disabled, 0, true, true), Some(2));
        assert_eq!(next_enabled(&disabled, 2, true, true), Some(0));
        assert_eq!(next_enabled(&disabled, 2, false, true), Some(0));
    }

    #[test]
    fn active_index_prefers_selected_when_enabled() {
        let keys: Vec<Arc<str>> = vec![Arc::from("a"), Arc::from("b"), Arc::from("c")];
        let disabled = [false, true, false];
        assert_eq!(
            active_index_from_str_keys(&keys, Some("c"), &disabled),
            Some(2)
        );
        assert_eq!(
            active_index_from_str_keys(&keys, Some("b"), &disabled),
            Some(0)
        );
        assert_eq!(active_index_from_str_keys(&keys, None, &disabled), Some(0));
    }
}
