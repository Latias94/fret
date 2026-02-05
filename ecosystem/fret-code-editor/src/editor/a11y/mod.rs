//! Accessibility helpers for the code editor surface.

use super::CodeEditorState;
use fret_code_editor_buffer::TextBuffer;

const A11Y_WINDOW_BYTES_BEFORE: usize = 4096;
const A11Y_WINDOW_BYTES_AFTER: usize = 4096;

pub(super) fn a11y_composed_text_window(
    st: &CodeEditorState,
) -> (String, Option<(u32, u32)>, Option<(u32, u32)>) {
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

        let (mut a, mut b) = preedit
            .cursor
            .unwrap_or_else(|| (preedit.text.len(), preedit.text.len()));
        a = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, a).min(preedit.text.len());
        b = fret_code_editor_view::clamp_to_char_boundary(&preedit.text, b).min(preedit.text.len());
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }

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
