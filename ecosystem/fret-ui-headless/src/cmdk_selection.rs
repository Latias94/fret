//! Headless cmdk-style "active option" selection helpers.
//!
//! This is intentionally small and deterministic: it only provides index math for keeping focus in
//! the input field while moving a highlighted row in a results list.

/// Returns the next active index given the current active index, disabled flags, and direction.
///
/// - When `current` is `None`, this picks the first/last enabled item depending on `forward`.
/// - When `wrap` is `false`, reaching an edge keeps the current index (if valid).
/// - If every item is disabled, returns `None`.
pub fn next_active_index(
    disabled: &[bool],
    current: Option<usize>,
    forward: bool,
    wrap: bool,
) -> Option<usize> {
    let len = disabled.len();
    if len == 0 {
        return None;
    }

    let is_enabled = |idx: usize| disabled.get(idx).copied() == Some(false);
    let first = disabled.iter().position(|d| !*d)?;
    let last = disabled.iter().rposition(|d| !*d)?;

    let Some(current) = current.filter(|&i| i < len && is_enabled(i)) else {
        return Some(if forward { first } else { last });
    };

    if wrap {
        for step in 1..=len {
            let idx = if forward {
                (current + step) % len
            } else {
                (current + len - (step % len)) % len
            };
            if is_enabled(idx) {
                return Some(idx);
            }
        }
        None
    } else if forward {
        ((current + 1)..len)
            .find(|&i| is_enabled(i))
            .or(Some(current))
    } else if current > 0 {
        (0..current)
            .rev()
            .find(|&i| is_enabled(i))
            .or(Some(current))
    } else {
        Some(current)
    }
}

/// Clamps an active index to a valid, enabled index.
///
/// If `current` is out of range or disabled, this falls back to the first enabled item.
pub fn clamp_active_index(disabled: &[bool], current: Option<usize>) -> Option<usize> {
    if let Some(current) = current
        && disabled.get(current).copied() == Some(false)
    {
        return Some(current);
    }
    disabled.iter().position(|d| !*d)
}

/// Returns the first enabled index, or `None` if all items are disabled.
pub fn first_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|d| !*d)
}

/// Returns the last enabled index, or `None` if all items are disabled.
pub fn last_enabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|d| !*d)
}

/// Moves the active index by `amount` steps, skipping disabled items.
///
/// This is a convenience wrapper around `next_active_index` for PageUp/PageDown style navigation.
pub fn advance_active_index(
    disabled: &[bool],
    current: Option<usize>,
    forward: bool,
    wrap: bool,
    amount: usize,
) -> Option<usize> {
    let mut cur = clamp_active_index(disabled, current);
    if amount == 0 {
        return cur;
    }

    for _ in 0..amount {
        let next = next_active_index(disabled, cur, forward, wrap);
        if next == cur {
            break;
        }
        cur = next;
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_picks_first_or_last_when_none() {
        let disabled = [true, false, false];
        assert_eq!(next_active_index(&disabled, None, true, true), Some(1));
        assert_eq!(next_active_index(&disabled, None, false, true), Some(2));
    }

    #[test]
    fn next_wraps_and_skips_disabled() {
        let disabled = [false, true, false];
        assert_eq!(next_active_index(&disabled, Some(0), true, true), Some(2));
        assert_eq!(next_active_index(&disabled, Some(2), true, true), Some(0));
        assert_eq!(next_active_index(&disabled, Some(0), false, true), Some(2));
    }

    #[test]
    fn next_does_not_wrap_and_clamps_to_edges() {
        let disabled = [false, true, false];
        assert_eq!(next_active_index(&disabled, Some(0), false, false), Some(0));
        assert_eq!(next_active_index(&disabled, Some(2), true, false), Some(2));
        assert_eq!(next_active_index(&disabled, Some(0), true, false), Some(2));
    }

    #[test]
    fn clamp_falls_back_to_first_enabled() {
        let disabled = [true, false, true];
        assert_eq!(clamp_active_index(&disabled, Some(0)), Some(1));
        assert_eq!(clamp_active_index(&disabled, Some(2)), Some(1));
        assert_eq!(clamp_active_index(&disabled, None), Some(1));
    }

    #[test]
    fn all_disabled_returns_none() {
        let disabled = [true, true];
        assert_eq!(next_active_index(&disabled, None, true, true), None);
        assert_eq!(clamp_active_index(&disabled, Some(0)), None);
    }

    #[test]
    fn first_and_last_enabled_work() {
        let disabled = [true, false, true, false];
        assert_eq!(first_enabled(&disabled), Some(1));
        assert_eq!(last_enabled(&disabled), Some(3));
        assert_eq!(first_enabled(&[true, true]), None);
        assert_eq!(last_enabled(&[true, true]), None);
    }

    #[test]
    fn advance_moves_multiple_steps() {
        let disabled = [false, true, false, false];
        assert_eq!(
            advance_active_index(&disabled, Some(0), true, true, 2),
            Some(3)
        );
        assert_eq!(
            advance_active_index(&disabled, Some(3), false, true, 3),
            Some(3)
        );
    }
}
