//! Text navigation helpers shared across Fret surfaces.
//!
//! This crate centralizes v1 "word" and "line" semantics used by:
//! - core text widgets (`TextInput`, `TextArea`, `SelectableText`)
//! - ecosystem code editor surfaces
//!
//! The active word-boundary mode (`UnicodeWord` vs `Identifier`) remains a policy input
//! (`TextBoundaryMode`). This crate only provides deterministic algorithms for those modes.
//!
//! Normative behavior is defined by:
//! - ADR 0179: Text Navigation and Word Boundaries (v1)
//! - ADR 0044: Text editing command vocabulary and UTF-8 byte indices (clamping rules)

use fret_runtime::TextBoundaryMode;
use unicode_segmentation::UnicodeSegmentation;

pub fn clamp_to_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    if text.is_char_boundary(idx) {
        return idx;
    }
    let mut i = idx;
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

pub fn prev_char_boundary(text: &str, idx: usize) -> usize {
    let idx = clamp_to_char_boundary(text, idx);
    if idx == 0 {
        return 0;
    }

    // Avoid scanning from the start (which is O(n)). Back up to the previous UTF-8 char boundary.
    let mut i = idx.saturating_sub(1);
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

pub fn next_char_boundary(text: &str, idx: usize) -> usize {
    let idx = clamp_to_char_boundary(text, idx);
    if idx >= text.len() {
        return text.len();
    }
    let ch = text[idx..].chars().next().unwrap_or('\0');
    idx.saturating_add(ch.len_utf8()).min(text.len())
}

pub fn is_grapheme_boundary(text: &str, idx: usize) -> bool {
    let idx = idx.min(text.len());
    if idx == 0 || idx == text.len() {
        return true;
    }
    text.grapheme_indices(true).any(|(start, _)| start == idx)
}

pub fn prev_grapheme_boundary(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if idx == 0 {
        return 0;
    }

    let mut prev = 0usize;
    for (start, _) in text.grapheme_indices(true) {
        if start >= idx {
            break;
        }
        prev = start;
    }
    prev
}

pub fn next_grapheme_boundary(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if idx >= text.len() {
        return text.len();
    }

    for (start, g) in text.grapheme_indices(true) {
        let end = start + g.len();
        if idx < end {
            return end;
        }
    }
    text.len()
}

pub fn clamp_to_grapheme_boundary(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        return idx;
    }

    // Prefer the closest grapheme boundary; ties clamp down.
    for (start, g) in text.grapheme_indices(true) {
        let end = start + g.len();
        if idx < end {
            return if idx - start <= end - idx { start } else { end };
        }
    }

    text.len()
}

pub fn clamp_to_grapheme_boundary_down(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        idx
    } else {
        prev_grapheme_boundary(text, idx)
    }
}

pub fn clamp_to_grapheme_boundary_up(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        idx
    } else {
        next_grapheme_boundary(text, idx)
    }
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_continue(ch)
}

fn char_at(text: &str, idx: usize) -> Option<char> {
    let idx = clamp_to_char_boundary(text, idx);
    text.get(idx..)?.chars().next()
}

fn is_unicode_word_char(text: &str, idx: usize) -> bool {
    let idx = clamp_to_char_boundary(text, idx);
    text.unicode_word_indices()
        .any(|(start, word)| (start..start + word.len()).contains(&idx))
}

fn unicode_word_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
    let idx = clamp_to_char_boundary(text, idx);
    for (start, word) in text.unicode_word_indices() {
        let end = start + word.len();
        if (start..end).contains(&idx) {
            return Some((start, end));
        }
    }
    None
}

fn identifier_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
    let idx = clamp_to_char_boundary(text, idx);
    let ch = char_at(text, idx)?;
    if !is_identifier_char(ch) {
        return None;
    }

    let mut start = idx;
    while start > 0 {
        let prev = prev_char_boundary(text, start);
        let prev_ch = char_at(text, prev).unwrap_or(' ');
        if !is_identifier_char(prev_ch) {
            break;
        }
        start = prev;
    }

    let mut end = next_char_boundary(text, idx);
    while end < text.len() {
        let next_ch = char_at(text, end).unwrap_or(' ');
        if !is_identifier_char(next_ch) {
            break;
        }
        end = next_char_boundary(text, end);
    }

    Some((start, end))
}

pub fn select_word_range(text: &str, idx: usize, mode: TextBoundaryMode) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let mut idx = clamp_to_grapheme_boundary(text, idx).min(text.len());
    if idx >= text.len() {
        idx = prev_grapheme_boundary(text, idx);
    }

    // Prefer selecting the previous word when clicking just after it.
    if char_at(text, idx).is_some_and(|c| c.is_whitespace()) && idx > 0 {
        let prev = prev_grapheme_boundary(text, idx);
        let prev_is_word = match mode {
            TextBoundaryMode::UnicodeWord => is_unicode_word_char(text, prev),
            TextBoundaryMode::Identifier => char_at(text, prev).is_some_and(is_identifier_char),
        };
        if prev_is_word {
            idx = prev;
        }
    }

    let Some(ch) = char_at(text, idx) else {
        return (0, 0);
    };

    if ch.is_whitespace() {
        let mut start = idx;
        while start > 0 {
            let prev = prev_grapheme_boundary(text, start);
            if char_at(text, prev).is_some_and(|c| c.is_whitespace()) {
                start = prev;
            } else {
                break;
            }
        }
        let mut end = next_grapheme_boundary(text, idx);
        while end < text.len() {
            if char_at(text, end).is_some_and(|c| c.is_whitespace()) {
                end = next_grapheme_boundary(text, end);
            } else {
                break;
            }
        }
        return (
            clamp_to_grapheme_boundary_down(text, start),
            clamp_to_grapheme_boundary_up(text, end),
        );
    }

    let (start, end) = match mode {
        TextBoundaryMode::UnicodeWord => {
            unicode_word_range_at(text, idx).unwrap_or((idx, next_grapheme_boundary(text, idx)))
        }
        TextBoundaryMode::Identifier => {
            identifier_range_at(text, idx).unwrap_or((idx, next_grapheme_boundary(text, idx)))
        }
    };

    (
        clamp_to_grapheme_boundary_down(text, start),
        clamp_to_grapheme_boundary_up(text, end),
    )
}

pub fn select_line_range(text: &str, idx: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let idx = clamp_to_grapheme_boundary(text, idx).min(text.len());
    let start = text[..idx]
        .rfind('\n')
        .map(|i| (i + 1).min(text.len()))
        .unwrap_or(0);
    let end = text[idx..]
        .find('\n')
        .map(|i| (idx + i + 1).min(text.len()))
        .unwrap_or(text.len());
    (
        clamp_to_grapheme_boundary_down(text, start),
        clamp_to_grapheme_boundary_up(text, end),
    )
}

pub fn move_word_left(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    let mut i = clamp_to_grapheme_boundary(text, idx);
    while i > 0 {
        let prev = prev_grapheme_boundary(text, i);
        let ch = text[prev..i].chars().next().unwrap_or(' ');
        if !ch.is_whitespace() {
            break;
        }
        i = prev;
    }

    if i == 0 {
        return 0;
    }

    // `i` is the boundary after any trailing whitespace. Anchor inside the grapheme immediately
    // to the left so we always query a position inside the word/token.
    let anchor = prev_grapheme_boundary(text, i);

    let next = match mode {
        TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, anchor)
            .map(|(start, _)| start)
            .unwrap_or(anchor),
        TextBoundaryMode::Identifier => identifier_range_at(text, anchor)
            .map(|(start, _)| start)
            .unwrap_or(anchor),
    };
    clamp_to_grapheme_boundary(text, next)
}

pub fn move_word_right(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    let mut i = next_grapheme_boundary(text, idx);
    while i < text.len() {
        let next = next_grapheme_boundary(text, i);
        let ch = text[i..next].chars().next().unwrap_or(' ');
        if !ch.is_whitespace() {
            break;
        }
        i = next;
    }

    if i >= text.len() {
        return text.len();
    }

    let next = match mode {
        TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, i)
            .map(|(_, end)| end)
            .unwrap_or(i),
        TextBoundaryMode::Identifier => identifier_range_at(text, i)
            .map(|(_, end)| end)
            .unwrap_or(i),
    };
    clamp_to_grapheme_boundary(text, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_word_right_distinguishes_unicode_word_and_identifier_for_apostrophe() {
        let text = "can't";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::UnicodeWord),
            text.len(),
            "UnicodeWord should treat \"can't\" as a single word"
        );
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::Identifier),
            3,
            "Identifier should split \"can't\" around the apostrophe"
        );
    }

    #[test]
    fn select_word_range_identifier_uses_xid_continue() {
        assert_eq!(
            select_word_range("αβγ δ", 1, TextBoundaryMode::Identifier),
            (0, "αβγ".len())
        );
        assert_eq!(
            select_word_range("a_b c", 1, TextBoundaryMode::Identifier),
            (0, "a_b".len())
        );
    }

    #[test]
    fn select_word_range_prefers_previous_word_when_clicking_whitespace_after_word() {
        let text = "foo bar";
        let idx = "foo".len();
        assert_eq!(
            select_word_range(text, idx, TextBoundaryMode::UnicodeWord),
            (0, "foo".len())
        );
        assert_eq!(
            select_word_range(text, idx, TextBoundaryMode::Identifier),
            (0, "foo".len())
        );
    }

    #[test]
    fn select_word_range_selects_whitespace_runs() {
        let text = "foo   bar";
        let idx = "foo ".len();
        assert_eq!(
            select_word_range(text, idx, TextBoundaryMode::UnicodeWord),
            ("foo".len(), "foo   ".len())
        );
        assert_eq!(
            select_word_range(text, idx, TextBoundaryMode::Identifier),
            ("foo".len(), "foo   ".len())
        );
    }

    #[test]
    fn select_word_range_unicode_word_handles_cjk_runs() {
        let text = "世界 hello";
        assert_eq!(
            select_word_range(text, 0, TextBoundaryMode::UnicodeWord),
            (0, "世".len())
        );
        assert_eq!(
            select_word_range(text, "世".len(), TextBoundaryMode::UnicodeWord),
            ("世".len(), "世界".len())
        );
    }

    #[test]
    fn select_word_range_unicode_word_falls_back_to_single_grapheme_on_emoji() {
        let text = "hi😀there";
        let emoji_start = "hi".len();
        let emoji_end = emoji_start + "😀".len();
        assert_eq!(
            select_word_range(text, emoji_start, TextBoundaryMode::UnicodeWord),
            (emoji_start, emoji_end)
        );
    }

    #[test]
    fn select_word_range_identifier_includes_digits_and_underscores() {
        let text = "foo123_bar baz";
        assert_eq!(
            select_word_range(text, 2, TextBoundaryMode::Identifier),
            (0, "foo123_bar".len())
        );
    }

    #[test]
    fn select_word_range_identifier_falls_back_to_single_grapheme_on_punctuation() {
        let text = "foo.bar";
        let dot = "foo".len();
        assert_eq!(
            select_word_range(text, dot, TextBoundaryMode::Identifier),
            (dot, dot + ".".len())
        );
    }

    #[test]
    fn select_word_range_unicode_word_falls_back_to_single_grapheme_on_zwj_emoji() {
        let emoji = "👩‍💻";
        let text = format!("a{emoji}b");
        let start = "a".len();
        assert_eq!(
            select_word_range(&text, start, TextBoundaryMode::UnicodeWord),
            (start, start + emoji.len())
        );
    }

    #[test]
    fn move_word_identifier_treats_punctuation_as_delimiter() {
        let text = "foo.bar";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::Identifier),
            "foo".len()
        );
        assert_eq!(
            move_word_left(text, text.len(), TextBoundaryMode::Identifier),
            "foo.".len()
        );
    }

    #[test]
    fn move_word_left_skips_whitespace_and_moves_to_word_start() {
        let text = "foo   bar";
        assert_eq!(
            move_word_left(text, text.len(), TextBoundaryMode::UnicodeWord),
            6
        );
        assert_eq!(
            move_word_left(text, "foo   ".len(), TextBoundaryMode::UnicodeWord),
            0
        );
    }

    #[test]
    fn move_word_right_skips_whitespace_and_moves_to_word_end() {
        let text = "foo   bar";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::UnicodeWord),
            "foo".len()
        );
        assert_eq!(
            move_word_right(text, "foo".len(), TextBoundaryMode::UnicodeWord),
            text.len()
        );
    }

    #[test]
    fn select_line_range_includes_trailing_newline_when_present() {
        let text = "a\nb\nc";
        assert_eq!(select_line_range(text, 0), (0, "a\n".len()));
        assert_eq!(select_line_range(text, "a".len()), (0, "a\n".len()));

        let b_idx = "a\n".len();
        assert_eq!(select_line_range(text, b_idx), (b_idx, "a\nb\n".len()));
        assert_eq!(
            select_line_range(text, b_idx + "b".len()),
            (b_idx, "a\nb\n".len())
        );

        let c_idx = "a\nb\n".len();
        assert_eq!(select_line_range(text, c_idx), (c_idx, text.len()));
    }
}
