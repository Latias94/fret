use super::*;
use crate::editor::geom::map_row_display_local_to_buffer_byte;

pub(in crate::editor) fn scroll_caret_into_view(
    st: &CodeEditorState,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) {
    if row_h.0 <= 0.0 {
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let caret = st.buffer.clamp_to_char_boundary_left(caret);
    let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let y = Px(row_h.0 * row as f32);

    // Keep a small vertical margin so the caret does not sit flush against the viewport edge.
    let margin = Px(row_h.0 * 2.0);
    scroll_handle.scroll_to_range_y(
        Px(y.0 - margin.0),
        Px(y.0 + row_h.0 + margin.0),
        fret_ui::scroll::ScrollStrategy::Nearest,
    );
}

pub(in crate::editor) fn page_rows(
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) -> usize {
    if row_h.0 <= 0.0 {
        return 1;
    }
    let viewport = scroll_handle.viewport_size();
    ((viewport.height.0 / row_h.0).floor() as usize).max(1)
}

pub(in crate::editor) fn move_caret_page(
    st: &mut CodeEditorState,
    pages: i32,
    extend: bool,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    cell_w: Px,
) {
    let rows = page_rows(row_h, scroll_handle);
    let delta = pages.saturating_mul(rows as i32);
    if delta != 0 {
        move_caret_vertical(st, delta, extend, cell_w);
    }

    // Keep the viewport moving with the caret for page navigation.
    let offset = scroll_handle.offset();
    let dy = row_h.0 * rows as f32;
    let next_y = if pages < 0 {
        offset.y.0 - dy * pages.unsigned_abs() as f32
    } else {
        offset.y.0 + dy * pages as f32
    };
    scroll_handle.scroll_to_offset(fret_core::Point::new(offset.x, Px(next_y)));
}

pub(in crate::editor) fn move_caret_home_end(
    st: &mut CodeEditorState,
    home: bool,
    ctrl_or_meta: bool,
    extend: bool,
) {
    let sel = st.selection.normalized();
    let mut caret = st.selection.caret().min(st.buffer.len_bytes());
    if !st.selection.is_caret() && !extend {
        caret = if home { sel.start } else { sel.end };
    }

    let target = if ctrl_or_meta {
        if home { 0 } else { st.buffer.len_bytes() }
    } else {
        let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
        let row_range = st.display_map.display_row_byte_range(&st.buffer, row);
        if home { row_range.start } else { row_range.end }
    };

    st.caret_preferred_x = None;
    if extend {
        if st.selection.is_caret() {
            st.selection.anchor = caret;
        }
        st.selection.focus = target;
    } else {
        st.selection = Selection {
            anchor: target,
            focus: target,
        };
    }
}

pub(in crate::editor) fn move_caret_left(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let mut new = st.buffer.prev_char_boundary(caret);
    new = clamp_byte_out_of_folds(st, new, FoldSnap::Start);
    st.caret_preferred_x = None;
    if extend {
        st.selection.focus = new;
    } else {
        st.selection = Selection {
            anchor: new,
            focus: new,
        };
    }
}

pub(in crate::editor) fn move_caret_right(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let mut new = st.buffer.next_char_boundary(caret);
    new = clamp_byte_out_of_folds(st, new, FoldSnap::End);
    st.caret_preferred_x = None;
    if extend {
        st.selection.focus = new;
    } else {
        st.selection = Selection {
            anchor: new,
            focus: new,
        };
    }
}

pub(in crate::editor) fn move_caret_vertical(
    st: &mut CodeEditorState,
    delta: i32,
    extend: bool,
    cell_w: Px,
) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let pt = st.display_map.byte_to_display_point(&st.buffer, caret);

    let desired_x = st
        .caret_preferred_x
        .or_else(|| caret_x_for_buffer_byte_in_row(st, pt.row, caret))
        .unwrap_or_else(|| Px(pt.col as f32 * cell_w.0));
    st.caret_preferred_x = Some(desired_x);

    let next_row = if delta < 0 {
        pt.row.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        pt.row.saturating_add(delta as usize)
    };
    let max_row = st.display_map.row_count().saturating_sub(1);
    let next_row = next_row.min(max_row);
    let next_row_has_preedit = st.preedit.is_some() && pt.row == next_row;
    let next = if let Some((geom, _)) = st.row_geom_cache.get(&next_row)
        && !geom.caret_stops.is_empty()
        && geom.has_preedit == next_row_has_preedit
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, desired_x);
        let byte = map_row_display_local_to_buffer_byte(&st.buffer, geom, local);
        st.buffer
            .clamp_to_char_boundary_left(byte.min(st.buffer.len_bytes()))
    } else {
        st.cache_stats.geom_vertical_move_fallbacks = st
            .cache_stats
            .geom_vertical_move_fallbacks
            .saturating_add(1);
        st.display_map
            .display_point_to_byte(&st.buffer, DisplayPoint::new(next_row, pt.col))
    };
    let next = clamp_byte_out_of_folds(st, next, FoldSnap::Start);
    if extend {
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
}

pub(in crate::editor) fn move_word(st: &mut CodeEditorState, dir: i32, extend: bool) -> bool {
    let mode = st.active_text_boundary_mode;
    st.undo_group = None;
    st.caret_preferred_x = None;

    let (sel_start, sel_end) = {
        let r = st.selection.normalized();
        (r.start, r.end)
    };
    let mut caret = st.selection.caret().min(st.buffer.len_bytes());
    if !st.selection.is_caret() && !extend {
        caret = if dir < 0 { sel_start } else { sel_end };
    }

    let next = if dir < 0 {
        move_word_left_in_buffer(&st.buffer, caret, mode)
    } else {
        move_word_right_in_buffer(&st.buffer, caret, mode)
    };
    let next = if dir < 0 {
        clamp_byte_out_of_folds(st, next, FoldSnap::Start)
    } else {
        clamp_byte_out_of_folds(st, next, FoldSnap::End)
    };

    if extend {
        if st.selection.is_caret() {
            st.selection.anchor = caret;
        }
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
    st.set_preedit(None);
    true
}

pub(in crate::editor) fn clamp_selection_out_of_folds(st: &mut CodeEditorState) {
    if st.preedit.is_some() || st.line_folds.is_empty() {
        return;
    }

    if st.selection.is_caret() {
        let caret = st.selection.caret().min(st.buffer.len_bytes());
        let caret = clamp_byte_out_of_folds(st, caret, FoldSnap::Start);
        st.selection = Selection {
            anchor: caret,
            focus: caret,
        };
        return;
    }

    let normalized = st.selection.normalized();
    let start = clamp_byte_out_of_folds(st, normalized.start, FoldSnap::Start);
    let end = clamp_byte_out_of_folds(st, normalized.end, FoldSnap::End);
    let (anchor, focus) = if st.selection.anchor <= st.selection.focus {
        (start, end)
    } else {
        (end, start)
    };
    st.selection.anchor = anchor;
    st.selection.focus = focus;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoldSnap {
    Start,
    End,
}

fn clamp_byte_out_of_folds(st: &CodeEditorState, byte: usize, snap: FoldSnap) -> usize {
    if st.preedit.is_some() || st.line_folds.is_empty() {
        return byte;
    }

    let line = st.buffer.line_index_at_byte(byte);
    let Some(folds) = st.line_folds.get(&line) else {
        return byte;
    };

    let Some(line_range) = st.buffer.line_byte_range(line) else {
        return byte;
    };
    if byte <= line_range.start || byte >= line_range.end {
        return byte;
    }

    let Some(line_text) = st.buffer.line_text(line) else {
        return byte;
    };
    if fret_code_editor_view::validate_fold_spans(&line_text, folds.as_ref()).is_err() {
        return byte;
    }

    let local = byte.saturating_sub(line_range.start);
    for span in folds.iter() {
        let start = span.range.start;
        let end = span.range.end.max(start);
        if local > start && local < end {
            return match snap {
                FoldSnap::Start => line_range.start.saturating_add(start),
                FoldSnap::End => line_range.start.saturating_add(end),
            };
        }
    }

    byte
}
