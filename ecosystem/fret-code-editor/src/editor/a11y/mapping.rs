use super::window::{
    A11yDisplayWindow, a11y_text_window_bounds, build_a11y_display_window,
    map_buffer_offset_to_a11y_global,
};
use super::*;

pub(in crate::editor) fn map_a11y_offsets_to_buffer_composed(
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

pub(in crate::editor) fn map_a11y_offset_to_buffer_in_current_window(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
    offset: u32,
) -> usize {
    if st.compose_inline_preedit {
        let (mapped, _) =
            map_a11y_offsets_to_buffer_composed(st, text_cache_max_entries, offset, offset);
        return mapped;
    }

    let caret = st
        .preedit_replace_range
        .as_ref()
        .map(|r| r.start)
        .unwrap_or_else(|| st.selection.caret().min(st.buffer.len_bytes()));
    let caret = st.buffer.clamp_to_char_boundary_left(caret);
    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

    if let Some(preedit) = st.preedit.as_ref() {
        let preedit_len = preedit.text.len();
        if let Some(r) = st.preedit_replace_range.as_ref()
            && !r.is_empty()
        {
            map_a11y_offset_to_buffer_with_preedit_replacing_range(
                &st.buffer,
                start,
                end,
                caret,
                r.end.min(st.buffer.len_bytes()),
                preedit_len,
                offset,
            )
        } else {
            map_a11y_offset_to_buffer_with_preedit(
                &st.buffer,
                start,
                end,
                caret,
                preedit_len,
                offset,
            )
        }
    } else {
        map_a11y_offset_to_buffer(&st.buffer, start, end, offset)
    }
}

pub(in crate::editor) fn map_buffer_offset_to_a11y_offset(
    st: &mut CodeEditorState,
    text_cache_max_entries: usize,
    byte: usize,
) -> u32 {
    if st.compose_inline_preedit {
        let caret = st
            .buffer
            .clamp_to_char_boundary_left(st.selection.caret().min(st.buffer.len_bytes()));
        let caret_row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
        let window = build_a11y_display_window(st, caret_row, text_cache_max_entries);
        let global = map_buffer_offset_to_a11y_global(&window, &st.buffer, byte);
        return u32::try_from(global).unwrap_or(u32::MAX);
    }

    let caret = st
        .preedit_replace_range
        .as_ref()
        .map(|r| r.start)
        .unwrap_or_else(|| st.selection.caret().min(st.buffer.len_bytes()));
    let caret = st.buffer.clamp_to_char_boundary_left(caret);
    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

    let mut byte = byte.min(end).max(start);
    byte = st.buffer.clamp_to_char_boundary_left(byte);

    let base = byte.saturating_sub(start);
    let out = if let Some(preedit) = st.preedit.as_ref() {
        let preedit_len = preedit.text.len();
        if let Some(r) = st.preedit_replace_range.as_ref()
            && !r.is_empty()
        {
            map_buffer_offset_to_a11y_offset_with_preedit_replacing_range(
                &st.buffer,
                start,
                end,
                caret,
                r.end.min(st.buffer.len_bytes()),
                preedit_len,
                byte,
            )
        } else if byte <= caret {
            base
        } else {
            base.saturating_add(preedit_len)
        }
    } else {
        base
    };

    u32::try_from(out).unwrap_or(u32::MAX)
}

fn map_a11y_offset_to_buffer_with_preedit_replacing_range(
    buf: &TextBuffer,
    window_start: usize,
    window_end: usize,
    replace_start: usize,
    replace_end: usize,
    preedit_len: usize,
    offset: u32,
) -> usize {
    let window_start = window_start.min(buf.len_bytes());
    let window_end = window_end.min(buf.len_bytes()).max(window_start);
    let replace_start = buf
        .clamp_to_char_boundary_left(replace_start.min(window_end))
        .max(window_start);
    let replace_end = buf
        .clamp_to_char_boundary_left(replace_end.min(window_end))
        .max(replace_start);

    let before_len = replace_start.saturating_sub(window_start);
    let removed_len = replace_end.saturating_sub(replace_start);
    let base_len = window_end.saturating_sub(window_start);
    let display_len = base_len
        .saturating_sub(removed_len)
        .saturating_add(preedit_len);
    let display_offset = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(display_len);

    let base_offset = if preedit_len == 0 {
        display_offset
    } else if display_offset <= before_len {
        display_offset
    } else if display_offset < before_len.saturating_add(preedit_len) {
        before_len
    } else {
        display_offset
            .saturating_sub(preedit_len)
            .saturating_add(removed_len)
    };

    let byte = window_start
        .saturating_add(base_offset.min(base_len))
        .min(window_end);
    buf.clamp_to_char_boundary_left(byte).min(buf.len_bytes())
}

fn map_buffer_offset_to_a11y_offset_with_preedit_replacing_range(
    buf: &TextBuffer,
    window_start: usize,
    window_end: usize,
    replace_start: usize,
    replace_end: usize,
    preedit_len: usize,
    byte: usize,
) -> usize {
    let window_start = window_start.min(buf.len_bytes());
    let window_end = window_end.min(buf.len_bytes()).max(window_start);
    let replace_start = buf
        .clamp_to_char_boundary_left(replace_start.min(window_end))
        .max(window_start);
    let replace_end = buf
        .clamp_to_char_boundary_left(replace_end.min(window_end))
        .max(replace_start);

    let before_len = replace_start.saturating_sub(window_start);
    let removed_len = replace_end.saturating_sub(replace_start);

    let mut byte = byte.min(window_end).max(window_start);
    byte = buf.clamp_to_char_boundary_left(byte);

    if byte <= replace_start {
        byte.saturating_sub(window_start)
    } else if byte < replace_end {
        before_len
    } else {
        byte.saturating_sub(window_start)
            .saturating_sub(removed_len)
            .saturating_add(preedit_len)
    }
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

pub(in crate::editor) fn map_a11y_offset_to_buffer(
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

pub(in crate::editor) fn map_a11y_offset_to_buffer_with_preedit(
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
