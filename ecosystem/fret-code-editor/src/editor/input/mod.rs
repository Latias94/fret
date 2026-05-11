//! Input, editing, and command handling for the code editor surface.

use super::geom::map_row_display_local_to_buffer_byte;
use super::*;

mod edit;

pub(super) use edit::{
    apply_and_record_edit, apply_ime_delete_surrounding, insert_text, insert_text_with_kind, redo,
    undo,
};

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

pub(super) fn apply_pointer_down_selection(
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
    if !st.interaction.enabled || !st.interaction.focusable || !st.interaction.selectable {
        return false;
    }
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
            st.set_preedit(None);
        }
    }

    // Let workspace keymaps handle global page navigation (e.g. tab switching).
    if ctrl_or_meta && matches!(key, KeyCode::PageUp | KeyCode::PageDown) {
        return false;
    }

    let cell_w_px = cell_w.get();

    match key {
        KeyCode::KeyA if ctrl_or_meta => {
            st.set_preedit(None);
            let end = st.buffer.len_bytes();
            st.selection = Selection {
                anchor: 0,
                focus: end,
            };
            st.caret_preferred_x = None;
            st.undo_group = None;
        }
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
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            if word {
                delete_word_backward(&mut st);
            } else {
                delete_backward(&mut st);
            }
        }
        KeyCode::Delete => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            if word {
                delete_word_forward(&mut st);
            } else {
                delete_forward(&mut st);
            }
        }
        KeyCode::Enter => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            let _ = insert_text(&mut st, "\n");
        }
        KeyCode::Tab => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            let _ = insert_text(&mut st, "\t");
        }
        KeyCode::KeyC if ctrl_or_meta => copy_selection(host, action_cx, &st),
        KeyCode::KeyV if ctrl_or_meta => {
            if st.interaction.editable {
                request_paste(host, action_cx);
            } else {
                st.set_preedit(None);
                st.undo_group = None;
            }
        }
        _ => return false,
    }

    scroll_caret_into_view(&st, row_h, scroll_handle);

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

pub(super) fn copy_selection(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &CodeEditorState,
) {
    let range = st.selection.normalized();
    if range.is_empty() {
        return;
    }
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let Some(text) = st.buffer.slice_to_string(start..end) else {
        return;
    };
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardWriteText {
        window: action_cx.window,
        token,
        text,
    });
}

pub(super) fn request_paste(host: &mut dyn UiActionHost, action_cx: ActionCx) {
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardReadText {
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

pub(super) fn move_caret_right(st: &mut CodeEditorState, extend: bool) {
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

pub(super) fn clamp_selection_out_of_folds(st: &mut CodeEditorState) {
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

pub(super) fn cut_selection(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &mut CodeEditorState,
) -> bool {
    let range = st.selection.normalized();
    if range.is_empty() {
        return false;
    }
    copy_selection(host, action_cx, st);
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
