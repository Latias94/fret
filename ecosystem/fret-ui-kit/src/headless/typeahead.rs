//! Typeahead buffer + prefix matching helpers (APG-inspired).
//!
//! This is used for menus / listbox / select-style widgets where users can type a sequence of
//! characters to jump to the next matching item.

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TypeaheadBuffer {
    timeout_ticks: u64,
    last_tick: Option<u64>,
    buffer: String,
}

impl TypeaheadBuffer {
    pub fn new(timeout_ticks: u64) -> Self {
        Self {
            timeout_ticks,
            last_tick: None,
            buffer: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.last_tick = None;
        self.buffer.clear();
    }

    pub fn push_char(&mut self, ch: char, now: u64) {
        if ch.is_whitespace() {
            return;
        }

        if let Some(prev) = self.last_tick
            && self.timeout_ticks > 0
            && now.saturating_sub(prev) > self.timeout_ticks
        {
            self.buffer.clear();
        }

        self.last_tick = Some(now);
        self.buffer.extend(ch.to_lowercase());
    }

    pub fn query(&self, now: u64) -> Option<&str> {
        if self.buffer.is_empty() {
            return None;
        }
        if let Some(prev) = self.last_tick
            && self.timeout_ticks > 0
            && now.saturating_sub(prev) > self.timeout_ticks
        {
            return None;
        }
        Some(self.buffer.as_str())
    }
}

pub fn match_prefix<'a>(
    labels: &'a [&'a str],
    disabled: &[bool],
    query: &str,
    current_match: Option<usize>,
    wrap: bool,
) -> Option<usize> {
    let len = labels.len();
    if len == 0 || query.is_empty() {
        return None;
    }

    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    let query = normalize_repeated_search(query);
    let exclude_current_match = query.chars().count() == 1 && current_match.is_some();

    let is_disabled = |idx: usize| disabled.get(idx).copied().unwrap_or(false);

    let matches = |idx: usize| -> bool {
        if is_disabled(idx) {
            return false;
        }
        let label = labels.get(idx).copied().unwrap_or_default().trim_start();
        label
            .to_ascii_lowercase()
            .starts_with(&query.to_ascii_lowercase())
    };

    if wrap {
        let start = current_match.unwrap_or(0);
        let start = if exclude_current_match {
            start.saturating_add(1)
        } else {
            start
        };
        for offset in 0..len {
            let idx = (start + offset) % len;
            if matches(idx) {
                return Some(idx);
            }
        }
        None
    } else {
        let start = current_match.unwrap_or(0);
        let start = if exclude_current_match {
            start.saturating_add(1)
        } else {
            start
        };
        if start >= len {
            return None;
        }
        (start..len).find(|&idx| matches(idx))
    }
}

pub fn match_prefix_arc_str(
    labels: &[Arc<str>],
    disabled: &[bool],
    query: &str,
    current_match: Option<usize>,
    wrap: bool,
) -> Option<usize> {
    let len = labels.len();
    if len == 0 || query.is_empty() {
        return None;
    }

    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    let query = normalize_repeated_search(query);
    let exclude_current_match = query.chars().count() == 1 && current_match.is_some();

    let is_disabled = |idx: usize| disabled.get(idx).copied().unwrap_or(false);

    let matches = |idx: usize| -> bool {
        if is_disabled(idx) {
            return false;
        }
        let label = labels.get(idx).map(|s| s.as_ref()).unwrap_or_default();
        label
            .trim_start()
            .to_ascii_lowercase()
            .starts_with(&query.to_ascii_lowercase())
    };

    if wrap {
        let start = current_match.unwrap_or(0);
        let start = if exclude_current_match {
            start.saturating_add(1)
        } else {
            start
        };
        for offset in 0..len {
            let idx = (start + offset) % len;
            if matches(idx) {
                return Some(idx);
            }
        }
        None
    } else {
        let start = current_match.unwrap_or(0);
        let start = if exclude_current_match {
            start.saturating_add(1)
        } else {
            start
        };
        if start >= len {
            return None;
        }
        (start..len).find(|&idx| matches(idx))
    }
}

fn normalize_repeated_search(query: &str) -> &str {
    let mut it = query.chars();
    let Some(first) = it.next() else {
        return query;
    };

    let mut count = 1usize;
    for c in it {
        count += 1;
        if c != first {
            return query;
        }
    }

    if count <= 1 {
        return query;
    }

    // For repeated characters (e.g. "aaa"), match as if it was just the first character.
    first_char_slice(query)
}

fn first_char_slice(s: &str) -> &str {
    let mut it = s.char_indices();
    let _ = it.next();
    match it.next() {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_expires_after_timeout() {
        let mut buf = TypeaheadBuffer::new(3);
        buf.push_char('a', 0);
        assert_eq!(buf.query(0), Some("a"));
        assert_eq!(buf.query(3), Some("a"));
        assert_eq!(buf.query(4), None);
    }

    #[test]
    fn match_prefix_skips_disabled_and_wraps() {
        let labels = ["Alpha", "Beta", "Alpine"];
        let disabled = [false, true, false];

        assert_eq!(
            match_prefix(&labels, &disabled, "a", Some(0), true),
            Some(2)
        );
        assert_eq!(
            match_prefix(&labels, &disabled, "a", Some(2), true),
            Some(0)
        );
        assert_eq!(match_prefix(&labels, &disabled, "b", None, true), None);
    }

    #[test]
    fn match_prefix_arc_str_matches_prefix_and_wraps() {
        let labels: Vec<Arc<str>> =
            vec![Arc::from("Alpha"), Arc::from("Beta"), Arc::from("Alpine")];
        let disabled = [false, true, false];

        assert_eq!(
            match_prefix_arc_str(&labels, &disabled, "al", Some(0), true),
            Some(0)
        );
        assert_eq!(
            match_prefix_arc_str(&labels, &disabled, "a", Some(0), true),
            Some(2)
        );
        assert_eq!(
            match_prefix_arc_str(&labels, &disabled, "a", Some(2), true),
            Some(0)
        );
        assert_eq!(
            match_prefix_arc_str(&labels, &disabled, "be", None, true),
            None
        );
    }

    #[test]
    fn multi_char_query_keeps_current_match_when_still_matching() {
        let labels = ["Alpha", "Alpine", "Beta"];
        let disabled = [false, false, false];
        assert_eq!(
            match_prefix(&labels, &disabled, "al", Some(0), true),
            Some(0)
        );
    }

    #[test]
    fn repeated_char_query_cycles_like_single_char() {
        let labels = ["Alpha", "Alpine", "Beta"];
        let disabled = [false, false, false];
        assert_eq!(
            match_prefix(&labels, &disabled, "aa", Some(0), true),
            Some(1)
        );
    }
}
