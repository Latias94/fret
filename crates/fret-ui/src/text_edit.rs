use fret_core::{ImeEvent, TickId};

pub(crate) mod utf8 {
    pub(crate) fn clamp_to_char_boundary(text: &str, idx: usize) -> usize {
        if idx >= text.len() {
            return text.len();
        }
        if text.is_char_boundary(idx) {
            return idx;
        }
        let mut i = idx;
        while i > 0 && !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    pub(crate) fn prev_char_boundary(text: &str, idx: usize) -> usize {
        let idx = clamp_to_char_boundary(text, idx);
        if idx == 0 {
            return 0;
        }
        let slice = &text[..idx];
        slice.char_indices().last().map(|(i, _)| i).unwrap_or(0)
    }

    pub(crate) fn next_char_boundary(text: &str, idx: usize) -> usize {
        let idx = clamp_to_char_boundary(text, idx);
        if idx >= text.len() {
            return text.len();
        }
        let ch = text[idx..].chars().next().unwrap();
        idx + ch.len_utf8()
    }

    pub(crate) fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    pub(crate) fn move_word_left(text: &str, idx: usize) -> usize {
        let mut i = prev_char_boundary(text, idx);
        while i > 0 {
            let prev = prev_char_boundary(text, i);
            let ch = text[prev..i].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = prev;
        }
        while i > 0 {
            let prev = prev_char_boundary(text, i);
            let ch = text[prev..i].chars().next().unwrap_or(' ');
            if !is_word_char(ch) {
                break;
            }
            i = prev;
        }
        i
    }

    pub(crate) fn move_word_right(text: &str, idx: usize) -> usize {
        let mut i = next_char_boundary(text, idx);
        while i < text.len() {
            let next = next_char_boundary(text, i);
            let ch = text[i..next].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i = next;
        }
        while i < text.len() {
            let next = next_char_boundary(text, i);
            let ch = text[i..next].chars().next().unwrap_or(' ');
            if !is_word_char(ch) {
                break;
            }
            i = next;
        }
        i
    }
}

pub(crate) mod buffer {
    pub(crate) fn selection_range(selection_anchor: usize, caret: usize) -> (usize, usize) {
        let a = selection_anchor.min(caret);
        let b = selection_anchor.max(caret);
        (a, b)
    }

    pub(crate) fn has_selection(selection_anchor: usize, caret: usize) -> bool {
        selection_anchor != caret
    }

    pub(crate) fn replace_selection(
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
        insert: &str,
    ) {
        let (a, b) = selection_range(*selection_anchor, *caret);
        if a != b {
            text.replace_range(a..b, insert);
            *caret = a + insert.len();
            *selection_anchor = *caret;
        } else {
            text.insert_str(*caret, insert);
            *caret += insert.len();
            *selection_anchor = *caret;
        }
    }

    pub(crate) fn delete_selection_if_any(
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
    ) -> bool {
        let (a, b) = selection_range(*selection_anchor, *caret);
        if a == b {
            return false;
        }
        text.replace_range(a..b, "");
        *caret = a;
        *selection_anchor = *caret;
        true
    }
}

pub(crate) mod ime {
    use super::buffer;
    use super::{ImeEvent, TickId};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ApplyResult {
        Noop,
        Cleared,
        PreeditUpdated { starting: bool },
        CommitApplied,
        CommitDuplicate,
    }

    pub(crate) fn is_composing(preedit: &str, preedit_cursor: Option<(usize, usize)>) -> bool {
        !preedit.is_empty() || preedit_cursor.is_some()
    }

    pub(crate) fn preedit_cursor_end(
        preedit: &str,
        preedit_cursor: Option<(usize, usize)>,
    ) -> usize {
        preedit_cursor
            .map(|(_, end)| end.min(preedit.len()))
            .unwrap_or(preedit.len())
    }

    pub(crate) fn clear_state(
        preedit: &mut String,
        preedit_cursor: &mut Option<(usize, usize)>,
        ime_replace_range: &mut Option<(usize, usize)>,
    ) {
        preedit.clear();
        *preedit_cursor = None;
        *ime_replace_range = None;
    }

    fn normalize_newlines_to_lf(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }

    pub(crate) fn apply_event(
        ime: &ImeEvent,
        tick: TickId,
        normalize_newlines: bool,
        last_text_input_tick: Option<TickId>,
        last_text_input_text: Option<&str>,
        text: &mut String,
        caret: &mut usize,
        selection_anchor: &mut usize,
        preedit: &mut String,
        preedit_cursor: &mut Option<(usize, usize)>,
        ime_replace_range: &mut Option<(usize, usize)>,
        last_ime_commit_tick: &mut Option<TickId>,
        last_ime_commit_text: &mut Option<String>,
    ) -> ApplyResult {
        match ime {
            ImeEvent::Enabled => ApplyResult::Noop,
            ImeEvent::Disabled => {
                clear_state(preedit, preedit_cursor, ime_replace_range);
                ApplyResult::Cleared
            }
            ImeEvent::Commit(text_in) => {
                let committed = if normalize_newlines && text_in.contains('\r') {
                    normalize_newlines_to_lf(text_in)
                } else {
                    text_in.clone()
                };

                if last_text_input_tick == Some(tick)
                    && last_text_input_text == Some(committed.as_str())
                {
                    clear_state(preedit, preedit_cursor, ime_replace_range);
                    return ApplyResult::CommitDuplicate;
                }

                *last_ime_commit_tick = Some(tick);
                *last_ime_commit_text = Some(committed.clone());

                if let Some((start, end)) = ime_replace_range.take() {
                    *selection_anchor = start;
                    *caret = end;
                }

                buffer::replace_selection(text, caret, selection_anchor, &committed);
                clear_state(preedit, preedit_cursor, ime_replace_range);
                ApplyResult::CommitApplied
            }
            ImeEvent::Preedit { text: next, cursor } => {
                if next.is_empty() && cursor.is_none() {
                    clear_state(preedit, preedit_cursor, ime_replace_range);
                    return ApplyResult::Cleared;
                }

                let starting = !is_composing(preedit, *preedit_cursor);
                if starting {
                    let (start, end) = buffer::selection_range(*selection_anchor, *caret);
                    if start != end {
                        *ime_replace_range = Some((start, end));
                        *caret = start;
                        *selection_anchor = start;
                    } else {
                        *ime_replace_range = None;
                    }
                }

                *preedit = next.clone();
                *preedit_cursor = *cursor;
                ApplyResult::PreeditUpdated { starting }
            }
        }
    }
}
