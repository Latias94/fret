//! Input, editing, and command handling for the code editor surface.

use super::*;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;

#[cfg(feature = "syntax")]
use super::paint::invalidate_syntax_row_cache_for_delta;

pub(super) fn scroll_caret_into_view(
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

pub(super) fn push_caret_rect_effect(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &mut CodeEditorState,
    row_h: Px,
    cell_w: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
) {
    let Some(bounds) = st.last_bounds else {
        return;
    };
    let cell_w = if cell_w.0 > 0.0 { cell_w } else { Px(8.0) };
    if let Some(rect) = caret_rect_for_selection(st, row_h, cell_w, bounds, scroll_handle) {
        host.push_effect(Effect::ImeSetCursorArea {
            window: action_cx.window,
            rect,
        });
    }
}

pub(super) fn insert_text(st: &mut CodeEditorState, text: &str) -> Option<()> {
    insert_text_with_kind(st, text, UndoGroupKind::Typing)
}

pub(super) fn insert_text_with_kind(
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

pub(super) fn apply_pointer_down_selection(
    st: &mut CodeEditorState,
    row: usize,
    caret: usize,
    click_count: u8,
    shift: bool,
) {
    st.preedit = None;

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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_key_down(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: ActionCx,
    state: &Rc<RefCell<CodeEditorState>>,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    cell_w: &Cell<Px>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    let mut st = state.borrow_mut();
    let shift = modifiers.shift;
    let ctrl_or_meta = modifiers.ctrl || modifiers.meta;
    let word = modifiers.ctrl || modifiers.alt;
    let meta = modifiers.meta;

    if st.preedit.is_some() {
        let cancel_preedit = match key {
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Enter
            | KeyCode::Tab => true,
            KeyCode::PageUp | KeyCode::PageDown => !ctrl_or_meta,
            _ => false,
        };
        if cancel_preedit {
            st.preedit = None;
        }
    }

    // Let workspace keymaps handle global page navigation (e.g. tab switching).
    if ctrl_or_meta && matches!(key, KeyCode::PageUp | KeyCode::PageDown) {
        return false;
    }

    let cell_w_px = cell_w.get();

    match key {
        KeyCode::ArrowLeft => {
            if meta {
                move_caret_home_end(&mut st, true, false, shift);
            } else if word {
                move_word(&mut st, -1, shift);
            } else {
                move_caret_left(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowRight => {
            if meta {
                move_caret_home_end(&mut st, false, false, shift);
            } else if word {
                move_word(&mut st, 1, shift);
            } else {
                move_caret_right(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowUp => {
            if meta {
                move_caret_home_end(&mut st, true, true, shift);
            } else {
                move_caret_vertical(&mut st, -1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowDown => {
            if meta {
                move_caret_home_end(&mut st, false, true, shift);
            } else {
                move_caret_vertical(&mut st, 1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::Home => {
            move_caret_home_end(&mut st, true, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::End => {
            move_caret_home_end(&mut st, false, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::PageUp => {
            move_caret_page(&mut st, -1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::PageDown => {
            move_caret_page(&mut st, 1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::Backspace => {
            if word {
                delete_word_backward(&mut st);
            } else {
                delete_backward(&mut st);
            }
        }
        KeyCode::Delete => {
            if word {
                delete_word_forward(&mut st);
            } else {
                delete_forward(&mut st);
            }
        }
        KeyCode::Enter => {
            let _ = insert_text(&mut st, "\n");
        }
        KeyCode::Tab => {
            let _ = insert_text(&mut st, "\t");
        }
        KeyCode::KeyC if ctrl_or_meta => copy_selection(host, &st),
        KeyCode::KeyV if ctrl_or_meta => request_paste(host, action_cx),
        _ => return false,
    }

    scroll_caret_into_view(&st, row_h, scroll_handle);
    push_caret_rect_effect(host, action_cx, &mut st, row_h, cell_w_px, scroll_handle);

    host.notify(action_cx);
    host.request_redraw(action_cx.window);
    true
}

pub(super) fn page_rows(row_h: Px, scroll_handle: &fret_ui::scroll::ScrollHandle) -> usize {
    if row_h.0 <= 0.0 {
        return 1;
    }
    let viewport = scroll_handle.viewport_size();
    ((viewport.height.0 / row_h.0).floor() as usize).max(1)
}

pub(super) fn move_caret_page(
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

pub(super) fn move_caret_home_end(
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

pub(super) fn copy_selection(host: &mut dyn UiActionHost, st: &CodeEditorState) {
    let range = st.selection.normalized();
    if range.is_empty() {
        return;
    }
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let Some(text) = st.buffer.slice_to_string(start..end) else {
        return;
    };
    host.push_effect(Effect::ClipboardSetText { text });
}

pub(super) fn request_paste(host: &mut dyn UiActionHost, action_cx: ActionCx) {
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardGetText {
        window: action_cx.window,
        token,
    });
}

pub(super) fn delete_word_backward(st: &mut CodeEditorState) {
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

pub(super) fn delete_word_forward(st: &mut CodeEditorState) {
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

pub(super) fn delete_backward(st: &mut CodeEditorState) {
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

pub(super) fn delete_forward(st: &mut CodeEditorState) {
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

pub(super) fn move_caret_left(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let new = st.buffer.prev_char_boundary(caret);
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

pub(super) fn move_caret_right(st: &mut CodeEditorState, extend: bool) {
    let caret = st.selection.caret().min(st.buffer.len_bytes());
    let new = st.buffer.next_char_boundary(caret);
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

pub(super) fn move_caret_vertical(st: &mut CodeEditorState, delta: i32, extend: bool, cell_w: Px) {
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
    let next = if let Some((geom, _)) = st.row_geom_cache.get(&next_row)
        && !geom.caret_stops.is_empty()
        && geom.preedit.is_some() == st.preedit.is_some()
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, desired_x);
        let byte = map_row_local_to_buffer_byte(&st.buffer, geom, local);
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
    if extend {
        st.selection.focus = next;
    } else {
        st.selection = Selection {
            anchor: next,
            focus: next,
        };
    }
}

pub(super) fn apply_and_record_edit(
    st: &mut CodeEditorState,
    kind: UndoGroupKind,
    edit: Edit,
    next_selection: Selection,
) -> Option<()> {
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

    st.preedit = None;
    let delta = {
        let group = st.undo_group.as_mut().expect("undo group must exist");
        st.buffer.apply_in_transaction(&mut group.tx, edit).ok()?
    };
    if st.display_wrap_cols.is_some() || delta.lines.old_count != delta.lines.new_count {
        st.refresh_display_map();
    }
    #[cfg(feature = "syntax")]
    invalidate_syntax_row_cache_for_delta(st, delta);
    #[cfg(not(feature = "syntax"))]
    let _ = delta;
    st.selection = next_selection;
    st.caret_preferred_x = None;

    let can_shift_row_geom_cache = edit_is_single_line
        && before_wrap_cols == st.display_wrap_cols
        && delta.lines.old_count == 1
        && delta.lines.new_count == 1
        && delta.lines.start == before_line;
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
    before_line_rows: std::ops::Range<usize>,
    after_line_rows: std::ops::Range<usize>,
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

pub(super) fn undo(st: &mut CodeEditorState) -> bool {
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
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
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

pub(super) fn redo(st: &mut CodeEditorState) -> bool {
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
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
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

pub(super) fn move_word(st: &mut CodeEditorState, dir: i32, extend: bool) -> bool {
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
    st.preedit = None;
    true
}

pub(super) fn cut_selection(host: &mut dyn UiActionHost, st: &mut CodeEditorState) -> bool {
    let range = st.selection.normalized();
    if range.is_empty() {
        return false;
    }
    copy_selection(host, st);
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let out = apply_and_record_edit(
        st,
        UndoGroupKind::Cut,
        Edit::Delete { range: start..end },
        Selection {
            anchor: start,
            focus: start,
        },
    )
    .is_some();
    if out {
        st.caret_preferred_x = None;
    }
    out
}
