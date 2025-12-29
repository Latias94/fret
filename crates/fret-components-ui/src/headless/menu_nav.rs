//! Menu/list navigation helpers (APG-aligned index math).
//!
//! This module is intentionally headless and deterministic: it provides index selection math for
//! keyboard navigation and leaves event wiring / focus requests to the UI runtime.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    Prev,
    Next,
    Home,
    End,
}

pub fn next_enabled_index(
    disabled: &[bool],
    current: Option<usize>,
    action: NavAction,
    wrap: bool,
) -> Option<usize> {
    let len = disabled.len();
    if len == 0 {
        return None;
    }

    let is_disabled = |idx: usize| disabled.get(idx).copied().unwrap_or(false);
    let first = || (0..len).find(|&i| !is_disabled(i));
    let last = || (0..len).rev().find(|&i| !is_disabled(i));

    match action {
        NavAction::Home => first(),
        NavAction::End => last(),
        NavAction::Next => {
            let Some(cur) = current else {
                return first();
            };
            if cur >= len {
                return first();
            }
            if wrap {
                for step in 1..=len {
                    let idx = (cur + step) % len;
                    if !is_disabled(idx) {
                        return Some(idx);
                    }
                }
                None
            } else {
                ((cur + 1)..len).find(|&i| !is_disabled(i))
            }
        }
        NavAction::Prev => {
            let Some(cur) = current else {
                return last();
            };
            if cur >= len {
                return last();
            }
            if wrap {
                for step in 1..=len {
                    let idx = (cur + len - (step % len)) % len;
                    if !is_disabled(idx) {
                        return Some(idx);
                    }
                }
                None
            } else if cur > 0 {
                (0..cur).rev().find(|&i| !is_disabled(i))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_and_skips_disabled() {
        let disabled = [false, true, false];
        assert_eq!(
            next_enabled_index(&disabled, Some(0), NavAction::Next, true),
            Some(2)
        );
        assert_eq!(
            next_enabled_index(&disabled, Some(2), NavAction::Next, true),
            Some(0)
        );
        assert_eq!(
            next_enabled_index(&disabled, Some(0), NavAction::Prev, true),
            Some(2)
        );
    }

    #[test]
    fn non_wrapping_stops_at_edges() {
        let disabled = [false, false, false];
        assert_eq!(
            next_enabled_index(&disabled, Some(2), NavAction::Next, false),
            None
        );
        assert_eq!(
            next_enabled_index(&disabled, Some(0), NavAction::Prev, false),
            None
        );
    }

    #[test]
    fn home_end_pick_first_last_enabled() {
        let disabled = [true, false, true, false];
        assert_eq!(
            next_enabled_index(&disabled, None, NavAction::Home, true),
            Some(1)
        );
        assert_eq!(
            next_enabled_index(&disabled, None, NavAction::End, true),
            Some(3)
        );
    }
}
