//! Input, editing, and command handling for the code editor surface.

use super::*;

pub(super) fn push_caret_rect_effect(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &CodeEditorState,
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

    push_caret_rect_effect(host, action_cx, &st, row_h, cell_w_px, scroll_handle);

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

    let prev = move_word_left_in_buffer(&st.buffer, caret, st.text_boundary_mode).min(caret);
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
    let next = move_word_right_in_buffer(&st.buffer, caret, st.text_boundary_mode)
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
    {
        let local = hit_test_index_from_caret_stops(&geom.caret_stops, desired_x);
        let byte = map_row_local_to_buffer_byte(&st.buffer, geom, local);
        st.buffer
            .clamp_to_char_boundary_left(byte.min(st.buffer.len_bytes()))
    } else {
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
    st.row_geom_cache_rev = st.buffer.revision();
    st.row_geom_cache_wrap_cols = st.display_wrap_cols;
    st.row_geom_cache_tick = 0;
    st.row_geom_cache.clear();
    st.row_geom_cache_queue.clear();

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
    let mode = st.text_boundary_mode;
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
