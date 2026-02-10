//! Accessibility helpers for the code editor surface.

use super::CodeEditorState;
use super::geom::RowFoldMap;
use super::paint::cached_row_text_with_range;
use fret_code_editor_buffer::TextBuffer;
use std::ops::Range;
use std::sync::Arc;

const A11Y_WINDOW_BYTES_BEFORE: usize = 4096;
const A11Y_WINDOW_BYTES_AFTER: usize = 4096;

pub(super) fn a11y_composed_text_window(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
) -> (String, Option<(u32, u32)>, Option<(u32, u32)>) {
    if st.compose_inline_preedit {
        return a11y_display_text_window_composed(st, text_cache_max_entries);
    }

    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));

    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

    let before = st.buffer.slice_to_string(start..caret).unwrap_or_default();
    let after = st.buffer.slice_to_string(caret..end).unwrap_or_default();

    if let Some(preedit) = st.preedit.as_ref() {
        let mut display = String::with_capacity(before.len() + preedit.text.len() + after.len());
        display.push_str(before.as_str());
        display.push_str(preedit.text.as_str());
        display.push_str(after.as_str());

        let before_len: u32 = before.len().try_into().unwrap_or(u32::MAX);
        let preedit_len: u32 = preedit.text.len().try_into().unwrap_or(u32::MAX);

        let composition = Some((before_len, before_len.saturating_add(preedit_len)));

        // ADR 0071: selection offsets are (anchor, focus) byte offsets into the semantics value.
        // Preserve directionality when the IME reports a cursor range.
        let (mut a, mut b) = preedit
            .cursor
            .unwrap_or_else(|| (preedit.text.len(), preedit.text.len()));
        a = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, a).min(preedit.text.len());
        b = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, b).min(preedit.text.len());

        let selection = Some((
            before_len.saturating_add(a as u32),
            before_len.saturating_add(b as u32),
        ));

        return (display, selection, composition);
    }

    let mut display = String::with_capacity(before.len() + after.len());
    display.push_str(before.as_str());
    display.push_str(after.as_str());

    let map = |offset: usize| -> u32 {
        let offset = offset.min(end).max(start);
        let offset = st.buffer.clamp_to_char_boundary_left(offset);
        u32::try_from(offset.saturating_sub(start)).unwrap_or(u32::MAX)
    };
    let selection = Some((map(st.selection.anchor), map(st.selection.focus)));

    (display, selection, None)
}

#[derive(Debug, Clone)]
struct A11yDisplayRow {
    line: usize,
    row_range: Range<usize>,
    text: Arc<str>,
    fold_map: Option<RowFoldMap>,
    preedit_range: Option<Range<usize>>,
}

#[derive(Debug, Clone)]
struct A11yDisplayWindow {
    rows: Vec<A11yDisplayRow>,
    row_global_starts: Vec<usize>,
    newline_after_row: Vec<bool>,
    value: String,
}

fn a11y_display_text_window_composed(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
) -> (String, Option<(u32, u32)>, Option<(u32, u32)>) {
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
    let caret_row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let window = build_a11y_display_window(st, caret_row, text_cache_max_entries);

    let selection = if let Some(preedit) = st.preedit.as_ref() {
        let mut selection = None::<(u32, u32)>;
        let mut composition = None::<(u32, u32)>;

        for (idx, row) in window.rows.iter().enumerate() {
            let Some(preedit_range) = row.preedit_range.clone() else {
                continue;
            };
            let before_len: u32 = window
                .row_global_starts
                .get(idx)
                .copied()
                .unwrap_or(0)
                .saturating_add(preedit_range.start)
                .try_into()
                .unwrap_or(u32::MAX);
            let preedit_len: u32 = preedit_range.len().try_into().unwrap_or(u32::MAX);
            composition = Some((before_len, before_len.saturating_add(preedit_len)));

            // ADR 0071: selection offsets are (anchor, focus) byte offsets into the semantics
            // value. Preserve directionality when the IME reports a cursor range.
            let (mut a, mut b) = preedit
                .cursor
                .unwrap_or_else(|| (preedit.text.len(), preedit.text.len()));
            a = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, a)
                .min(preedit.text.len());
            b = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, b)
                .min(preedit.text.len());
            selection = Some((
                before_len.saturating_add(a as u32),
                before_len.saturating_add(b as u32),
            ));
            return (window.value, selection, composition);
        }

        (window.value, selection, composition)
    } else {
        let map = |offset: usize| -> u32 {
            let global = map_buffer_offset_to_a11y_global(&window, &st.buffer, offset);
            u32::try_from(global).unwrap_or(u32::MAX)
        };
        let selection = Some((map(st.selection.anchor), map(st.selection.focus)));
        (window.value, selection, None)
    };

    selection
}

fn build_a11y_display_window(
    st: &mut CodeEditorState,
    caret_row: usize,
    text_cache_max_entries: usize,
) -> A11yDisplayWindow {
    let row_count = st.display_map.row_count().max(1);
    let caret_row = caret_row.min(row_count.saturating_sub(1));

    let mut before_bytes = 0usize;
    let mut start_row = caret_row;
    while start_row > 0 && before_bytes < A11Y_WINDOW_BYTES_BEFORE {
        let prev = start_row.saturating_sub(1);
        let range = st.display_map.display_row_byte_range(&st.buffer, prev);
        before_bytes = before_bytes.saturating_add(range.len());
        if st.display_map.display_row_line(prev) != st.display_map.display_row_line(start_row) {
            before_bytes = before_bytes.saturating_add(1);
        }
        start_row = prev;
    }

    let mut after_bytes = 0usize;
    let mut end_row = caret_row;
    while end_row.saturating_add(1) < row_count && after_bytes < A11Y_WINDOW_BYTES_AFTER {
        let next = end_row.saturating_add(1);
        let range = st.display_map.display_row_byte_range(&st.buffer, next);
        after_bytes = after_bytes.saturating_add(range.len());
        if st.display_map.display_row_line(next) != st.display_map.display_row_line(end_row) {
            after_bytes = after_bytes.saturating_add(1);
        }
        end_row = next;
    }

    let mut rows = Vec::<A11yDisplayRow>::new();
    rows.reserve(end_row.saturating_sub(start_row).saturating_add(1));

    for row in start_row..=end_row {
        let line = st.display_map.display_row_line(row);
        let (row_range, text, fold_map, preedit_range) =
            cached_row_text_with_range(st, row, text_cache_max_entries);
        rows.push(A11yDisplayRow {
            line,
            row_range,
            text,
            fold_map,
            preedit_range,
        });
    }

    if rows.is_empty() {
        return A11yDisplayWindow {
            rows,
            row_global_starts: Vec::new(),
            newline_after_row: Vec::new(),
            value: String::new(),
        };
    }

    let mut cap = 0usize;
    for i in 0..rows.len() {
        cap = cap.saturating_add(rows[i].text.len());
        if i.saturating_add(1) < rows.len() && rows[i + 1].line != rows[i].line {
            cap = cap.saturating_add(1);
        }
    }

    let mut value = String::with_capacity(cap);
    let mut row_global_starts = Vec::<usize>::with_capacity(rows.len());
    let mut newline_after_row = Vec::<bool>::with_capacity(rows.len());
    for i in 0..rows.len() {
        row_global_starts.push(value.len());
        value.push_str(rows[i].text.as_ref());

        let needs_nl = i.saturating_add(1) < rows.len() && rows[i + 1].line != rows[i].line;
        newline_after_row.push(needs_nl);
        if needs_nl {
            value.push('\n');
        }
    }

    A11yDisplayWindow {
        rows,
        row_global_starts,
        newline_after_row,
        value,
    }
}

fn map_buffer_offset_to_a11y_global(
    window: &A11yDisplayWindow,
    buf: &TextBuffer,
    byte: usize,
) -> usize {
    let Some(first) = window.rows.first() else {
        return 0;
    };
    let Some(last) = window.rows.last() else {
        return 0;
    };

    let window_start = first.row_range.start.min(buf.len_bytes());
    let window_end = last.row_range.end.min(buf.len_bytes()).max(window_start);
    let mut byte = byte.min(window_end).max(window_start);
    byte = buf.clamp_to_char_boundary_left(byte);

    for (idx, row) in window.rows.iter().enumerate() {
        if byte < row.row_range.start {
            continue;
        }
        let start = *window.row_global_starts.get(idx).unwrap_or(&0);
        let end = start.saturating_add(row.text.len());

        if byte < row.row_range.end {
            let buffer_local = byte.saturating_sub(row.row_range.start);
            let display_local = row
                .fold_map
                .as_ref()
                .map(|m| m.buffer_local_to_display_local(buffer_local))
                .unwrap_or(buffer_local)
                .min(row.text.len());
            return start.saturating_add(display_local);
        }

        if byte == row.row_range.end {
            if window.newline_after_row.get(idx).copied().unwrap_or(false) {
                return end;
            }
            return end;
        }
    }

    window.value.len()
}

pub(super) fn map_a11y_offsets_to_buffer_composed(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
    anchor: u32,
    focus: u32,
) -> (usize, usize) {
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
    let caret_row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let window = build_a11y_display_window(st, caret_row, text_cache_max_entries);
    (
        map_a11y_offset_to_buffer_composed(&window, &st.buffer, anchor),
        map_a11y_offset_to_buffer_composed(&window, &st.buffer, focus),
    )
}

fn map_a11y_offset_to_buffer_composed(
    window: &A11yDisplayWindow,
    buf: &TextBuffer,
    offset: u32,
) -> usize {
    let offset = usize::try_from(offset).unwrap_or(usize::MAX);
    let offset = offset.min(window.value.len());

    for (idx, row) in window.rows.iter().enumerate() {
        let start = *window.row_global_starts.get(idx).unwrap_or(&0);
        let end = start.saturating_add(row.text.len());
        if offset <= end {
            let local_display = offset.saturating_sub(start).min(row.text.len());
            let buffer_local = row
                .fold_map
                .as_ref()
                .map(|m| m.display_local_to_buffer_local(local_display))
                .unwrap_or(local_display)
                .min(row.row_range.len());
            return buf
                .clamp_to_char_boundary_left(row.row_range.start.saturating_add(buffer_local))
                .min(buf.len_bytes());
        }

        if window.newline_after_row.get(idx).copied().unwrap_or(false) && offset == end + 1 {
            return buf
                .clamp_to_char_boundary_left(row.row_range.end)
                .min(buf.len_bytes());
        }
    }

    buf.len_bytes()
}

pub(super) fn a11y_text_window_bounds(buf: &TextBuffer, caret: usize) -> (usize, usize) {
    let caret = buf.clamp_to_char_boundary_left(caret.min(buf.len_bytes()));
    let start = buf.clamp_to_char_boundary_left(caret.saturating_sub(A11Y_WINDOW_BYTES_BEFORE));
    let end = buf.clamp_to_char_boundary_left(
        caret
            .saturating_add(A11Y_WINDOW_BYTES_AFTER)
            .min(buf.len_bytes()),
    );
    (start, end)
}

pub(super) fn map_a11y_offset_to_buffer(
    buf: &TextBuffer,
    window_start: usize,
    window_end: usize,
    offset: u32,
) -> usize {
    let window_start = window_start.min(buf.len_bytes());
    let window_end = window_end.min(buf.len_bytes()).max(window_start);
    let window_len = window_end.saturating_sub(window_start);
    let offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(window_len);
    let byte = window_start.saturating_add(offset).min(window_end);
    buf.clamp_to_char_boundary_left(byte).min(buf.len_bytes())
}

pub(super) fn map_a11y_offset_to_buffer_with_preedit(
    buf: &TextBuffer,
    window_start: usize,
    window_end: usize,
    caret: usize,
    preedit_len: usize,
    offset: u32,
) -> usize {
    let window_start = window_start.min(buf.len_bytes());
    let window_end = window_end.min(buf.len_bytes()).max(window_start);
    let caret = buf
        .clamp_to_char_boundary_left(caret)
        .min(window_end)
        .max(window_start);

    let anchor = caret.saturating_sub(window_start);
    let display_len = window_end
        .saturating_sub(window_start)
        .saturating_add(preedit_len);
    let display_offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(display_len);

    let base_offset = if preedit_len == 0 {
        display_offset
    } else if display_offset <= anchor {
        display_offset
    } else if display_offset >= anchor.saturating_add(preedit_len) {
        display_offset.saturating_sub(preedit_len)
    } else {
        anchor
    };

    let byte = window_start.saturating_add(base_offset).min(window_end);
    buf.clamp_to_char_boundary_left(byte).min(buf.len_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::{CodeEditorHandle, PreeditState, Selection};
    use super::a11y_composed_text_window;

    #[test]
    fn a11y_window_selection_preserves_direction_without_preedit() {
        let handle = CodeEditorHandle::new("hello world");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 8,
                focus: 3,
            };
        }

        let mut st = handle.state.borrow_mut();
        let (_value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
        assert_eq!(composition, None);
        assert_eq!(selection, Some((8, 3)));
    }

    #[test]
    fn a11y_window_selection_preserves_direction_for_preedit_cursor() {
        let handle = CodeEditorHandle::new("hello world");
        {
            let mut st = handle.state.borrow_mut();
            st.selection = Selection {
                anchor: 5,
                focus: 5,
            };
            st.preedit = Some(PreeditState {
                text: "yo".to_string(),
                cursor: Some((2, 0)),
            });
        }

        let mut st = handle.state.borrow_mut();
        let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
        assert_eq!(value, "helloyo world");
        assert_eq!(composition, Some((5, 7)));
        assert_eq!(selection, Some((7, 5)));
    }
}
