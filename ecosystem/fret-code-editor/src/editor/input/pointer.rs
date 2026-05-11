use super::*;

pub(in crate::editor) fn apply_pointer_down_selection(
    st: &mut CodeEditorState,
    row: usize,
    caret: usize,
    click_count: u8,
    shift: bool,
) {
    st.set_preedit(None);

    let caret = st
        .buffer
        .clamp_to_char_boundary_left(caret.min(st.buffer.len_bytes()));

    match click_count {
        2 => {
            let (start, end) =
                select_word_range_in_buffer(&st.buffer, caret, st.active_text_boundary_mode);
            st.selection = Selection {
                anchor: start,
                focus: end,
            };
        }
        3 => {
            let start = st
                .display_map
                .display_point_to_byte(&st.buffer, DisplayPoint::new(row, 0));
            let line = st.buffer.line_index_at_byte(start);
            if let Some(range) = st.buffer.line_byte_range_including_newline(line) {
                st.selection = Selection {
                    anchor: range.start,
                    focus: range.end,
                };
            }
        }
        _ => {
            if shift {
                st.selection.focus = caret;
            } else {
                st.selection = Selection {
                    anchor: caret,
                    focus: caret,
                };
            }
        }
    }

    st.caret_preferred_x = None;
}
