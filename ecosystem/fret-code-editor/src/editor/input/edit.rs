use super::*;

#[cfg(feature = "syntax")]
use crate::editor::syntax::invalidate_syntax_row_cache_for_delta;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;

pub(in crate::editor) fn insert_text(st: &mut CodeEditorState, text: &str) -> Option<()> {
    insert_text_with_kind(st, text, UndoGroupKind::Typing)
}

pub(in crate::editor) fn insert_text_with_kind(
    st: &mut CodeEditorState,
    text: &str,
    kind: UndoGroupKind,
) -> Option<()> {
    if text.is_empty() {
        return None;
    }
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let caret = start.saturating_add(text.len()).min(st.buffer.len_bytes());
    apply_and_record_edit(
        st,
        kind,
        Edit::Replace {
            range: start..end,
            text: text.to_string(),
        },
        Selection {
            anchor: caret,
            focus: caret,
        },
    )?;
    st.caret_preferred_x = None;
    Some(())
}

pub(in crate::editor) fn apply_ime_delete_surrounding(
    st: &mut CodeEditorState,
    before_bytes: usize,
    after_bytes: usize,
) -> Option<()> {
    if before_bytes == 0 && after_bytes == 0 {
        return None;
    }

    let len = st.buffer.len_bytes();
    let range = st.selection.normalized();
    let start = range.start.min(len);
    let end = range.end.min(len);

    let mut start_before = start.saturating_sub(before_bytes);
    while start_before > 0 && !st.buffer.is_char_boundary(start_before) {
        start_before = start_before.saturating_sub(1);
    }

    let mut end_after = end.saturating_add(after_bytes).min(len);
    while end_after < len && !st.buffer.is_char_boundary(end_after) {
        end_after = end_after.saturating_add(1);
    }

    if start_before == start && end_after == end {
        return None;
    }

    let kept = st.buffer.slice_to_string(start..end).unwrap_or_default();
    let deleted_before = start.saturating_sub(start_before);
    let next_selection = Selection {
        anchor: st.selection.anchor.saturating_sub(deleted_before),
        focus: st.selection.focus.saturating_sub(deleted_before),
    };
    let kind = if before_bytes > 0 {
        UndoGroupKind::Backspace
    } else {
        UndoGroupKind::DeleteForward
    };

    apply_and_record_edit_inner(
        st,
        kind,
        Edit::Replace {
            range: start_before..end_after,
            text: kept,
        },
        next_selection,
        true,
    )
}

pub(in crate::editor) fn delete_word_backward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::Backspace,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }

    let prev = move_word_left_in_buffer(&st.buffer, caret, st.active_text_boundary_mode).min(caret);
    if prev == caret {
        return;
    }

    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::Backspace,
        Edit::Delete { range: prev..caret },
        Selection {
            anchor: prev,
            focus: prev,
        },
    );
    st.caret_preferred_x = None;
}

pub(in crate::editor) fn delete_word_forward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::DeleteForward,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = move_word_right_in_buffer(&st.buffer, caret, st.active_text_boundary_mode)
        .max(caret)
        .min(st.buffer.len_bytes());
    if next == caret {
        return;
    }

    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::DeleteForward,
        Edit::Delete { range: caret..next },
        Selection {
            anchor: caret,
            focus: caret,
        },
    );
    st.caret_preferred_x = None;
}

pub(in crate::editor) fn delete_backward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::Backspace,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    if caret == 0 {
        return;
    }
    let prev = st.buffer.prev_char_boundary(caret);
    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::Backspace,
        Edit::Delete { range: prev..caret },
        Selection {
            anchor: prev,
            focus: prev,
        },
    );
    st.caret_preferred_x = None;
}

pub(in crate::editor) fn delete_forward(st: &mut CodeEditorState) {
    let range = st.selection.normalized();
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    if start != end {
        let _ = apply_and_record_edit(
            st,
            UndoGroupKind::DeleteForward,
            Edit::Delete { range: start..end },
            Selection {
                anchor: start,
                focus: start,
            },
        );
        st.caret_preferred_x = None;
        return;
    }

    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let next = st.buffer.next_char_boundary(caret);
    if next == caret {
        return;
    }
    let _ = apply_and_record_edit(
        st,
        UndoGroupKind::DeleteForward,
        Edit::Delete { range: caret..next },
        Selection {
            anchor: caret,
            focus: caret,
        },
    );
    st.caret_preferred_x = None;
}

pub(in crate::editor) fn apply_and_record_edit(
    st: &mut CodeEditorState,
    kind: UndoGroupKind,
    edit: Edit,
    next_selection: Selection,
) -> Option<()> {
    apply_and_record_edit_inner(st, kind, edit, next_selection, false)
}

fn apply_and_record_edit_inner(
    st: &mut CodeEditorState,
    kind: UndoGroupKind,
    edit: Edit,
    next_selection: Selection,
    preserve_preedit: bool,
) -> Option<()> {
    if !st.interaction.enabled || !st.interaction.editable {
        return None;
    }
    let (edit_start, edit_old_end, edit_byte_delta, edit_is_single_line) =
        edit_cache_shift_params(&st.buffer, &edit);
    let before_wrap_cols = st.display_wrap_cols;
    let before_line = st
        .buffer
        .line_index_at_byte(edit_start.min(st.buffer.len_bytes()));
    let before_line_rows = st.display_map.line_display_row_range(before_line);

    if !st.selection.is_caret() {
        st.undo_group = None;
    }
    if st.undo_group.as_ref().is_none_or(|g| g.kind != kind) {
        st.undo_group = Some(UndoGroup {
            kind,
            before_selection: st.selection,
            tx: TextBufferTransaction::default(),
            coalesce_key: kind.coalesce_key(),
        });
    }

    if !preserve_preedit {
        st.set_preedit(None);
    }
    let delta = {
        let group = st.undo_group.as_mut().expect("undo group must exist");
        st.buffer.apply_in_transaction(&mut group.tx, edit).ok()?
    };
    if st.display_wrap_cols.is_some() || delta.lines.old_count != delta.lines.new_count {
        st.refresh_display_map();
    }
    st.clear_feature_payloads_for_buffer_change();
    #[cfg(feature = "syntax")]
    invalidate_syntax_row_cache_for_delta(st, delta);
    #[cfg(not(feature = "syntax"))]
    let _ = delta;
    let can_shift_row_text_cache = edit_is_single_line
        && before_wrap_cols == st.display_wrap_cols
        && delta.lines.old_count == 1
        && delta.lines.new_count == 1
        && delta.lines.start == before_line
        && st.line_folds.is_empty()
        && st.line_inlays.is_empty();
    if can_shift_row_text_cache {
        let after_line_rows = st.display_map.line_display_row_range(before_line);
        if after_line_rows.start == before_line_rows.start {
            crate::editor::paint::shift_row_text_cache_for_single_line_edit(
                st,
                before_line_rows.clone(),
                after_line_rows.clone(),
                edit_old_end,
                edit_byte_delta,
            );
            crate::editor::paint::shift_row_scene_cache_for_single_line_edit(
                st,
                before_line_rows.clone(),
                after_line_rows,
                edit_old_end,
                edit_byte_delta,
            );
        }
    }
    st.selection = next_selection;
    st.caret_preferred_x = None;

    let can_shift_row_geom_cache = edit_is_single_line
        && before_wrap_cols == st.display_wrap_cols
        && delta.lines.old_count == 1
        && delta.lines.new_count == 1
        && delta.lines.start == before_line
        && st.line_folds.is_empty();
    if can_shift_row_geom_cache {
        let after_line_rows = st.display_map.line_display_row_range(before_line);
        if after_line_rows.start == before_line_rows.start {
            shift_row_geom_cache_for_single_line_edit(
                st,
                before_line_rows,
                after_line_rows,
                edit_old_end,
                edit_byte_delta,
            );
        } else {
            st.row_geom_cache_tick = 0;
            st.row_geom_cache.clear();
            st.row_geom_cache_queue.clear();
        }
    } else {
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
    st.row_geom_cache_rev = st.buffer.revision();
    st.row_geom_cache_wrap_cols = st.display_wrap_cols;
    st.row_geom_cache_folds_epoch = st.folds_epoch;

    let (buffer_tx, inverse_selection, coalesce_key) = {
        let group = st.undo_group.as_ref().expect("undo group must exist");
        (
            group.tx.snapshot(),
            group.before_selection,
            group.coalesce_key.clone(),
        )
    };
    let record = UndoRecord::new(CodeEditorTx {
        buffer_tx,
        selection: next_selection,
        inverse_selection,
    })
    .coalesce_key(coalesce_key);
    st.undo.record_or_coalesce(record);
    Some(())
}

fn edit_cache_shift_params(buf: &TextBuffer, edit: &Edit) -> (usize, usize, isize, bool) {
    let (start, old_end, delta, inserted_text) = match edit {
        Edit::Insert { at, text } => (*at, *at, text.len() as isize, text.as_str()),
        Edit::Delete { range } => (
            range.start,
            range.end,
            -((range.end.saturating_sub(range.start)) as isize),
            "",
        ),
        Edit::Replace { range, text } => (
            range.start,
            range.end,
            text.len() as isize - (range.end.saturating_sub(range.start) as isize),
            text.as_str(),
        ),
    };

    let inserted_is_single_line = !inserted_text.contains('\n');
    let start_line = buf.line_index_at_byte(start.min(buf.len_bytes()));
    let end_line = buf.line_index_at_byte(old_end.min(buf.len_bytes()));
    let is_single_line = inserted_is_single_line && start_line == end_line;
    (start, old_end, delta, is_single_line)
}

fn shift_row_geom_cache_for_single_line_edit(
    st: &mut CodeEditorState,
    before_line_rows: Range<usize>,
    after_line_rows: Range<usize>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) {
    let row_delta = after_line_rows.len() as isize - before_line_rows.len() as isize;

    let old_cache = std::mem::take(&mut st.row_geom_cache);
    let old_queue = std::mem::take(&mut st.row_geom_cache_queue);

    let mut new_cache = HashMap::with_capacity(old_cache.len());
    for (row, (mut geom, tick)) in old_cache {
        if before_line_rows.contains(&row) {
            continue;
        }

        if row >= before_line_rows.end {
            geom.row_range =
                shift_range_for_single_line_edit(geom.row_range, edit_old_end, edit_byte_delta);
        }
        let new_row = if row >= before_line_rows.end {
            shift_usize(row, row_delta)
        } else {
            row
        };
        new_cache.insert(new_row, (geom, tick));
    }

    let mut new_queue = VecDeque::with_capacity(old_queue.len());
    for (row, tick) in old_queue {
        if before_line_rows.contains(&row) {
            continue;
        }
        let new_row = if row >= before_line_rows.end {
            shift_usize(row, row_delta)
        } else {
            row
        };
        new_queue.push_back((new_row, tick));
    }

    st.row_geom_cache = new_cache;
    st.row_geom_cache_queue = new_queue;
}

fn shift_usize(value: usize, delta: isize) -> usize {
    if delta >= 0 {
        value.saturating_add(delta as usize)
    } else {
        value.saturating_sub((-delta) as usize)
    }
}

fn shift_range_for_single_line_edit(
    range: Range<usize>,
    edit_old_end: usize,
    delta: isize,
) -> Range<usize> {
    if range.end <= edit_old_end || delta == 0 {
        return range;
    }
    let start = shift_usize(range.start, delta);
    let end = shift_usize(range.end, delta);
    start..end.max(start)
}

pub(in crate::editor) fn undo(st: &mut CodeEditorState) -> bool {
    if !st.interaction.enabled || !st.interaction.editable {
        return false;
    }
    st.undo_group = None;
    st.caret_preferred_x = None;
    let (buffer, selection, preedit, history) = (
        &mut st.buffer,
        &mut st.selection,
        &mut st.preedit,
        &mut st.undo,
    );
    let mut applied = false;
    let _ = history.undo_invertible(|record| {
        *preedit = None;
        if buffer.apply_tx(&record.tx.buffer_tx).is_ok() {
            *selection = record.tx.selection;
            applied = true;
        }
        Ok::<_, ()>(())
    });
    if applied {
        st.refresh_display_map();
        st.clear_feature_payloads_for_buffer_change();
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
    #[cfg(feature = "syntax")]
    {
        if applied {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
    }
    applied
}

pub(in crate::editor) fn redo(st: &mut CodeEditorState) -> bool {
    if !st.interaction.enabled || !st.interaction.editable {
        return false;
    }
    st.undo_group = None;
    st.caret_preferred_x = None;
    let (buffer, selection, preedit, history) = (
        &mut st.buffer,
        &mut st.selection,
        &mut st.preedit,
        &mut st.undo,
    );
    let mut applied = false;
    let _ = history.redo_invertible(|record| {
        *preedit = None;
        if buffer.apply_tx(&record.tx.buffer_tx).is_ok() {
            *selection = record.tx.selection;
            applied = true;
        }
        Ok::<_, ()>(())
    });
    if applied {
        st.refresh_display_map();
        st.clear_feature_payloads_for_buffer_change();
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
    }
    #[cfg(feature = "syntax")]
    {
        if applied {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
        }
    }
    applied
}
