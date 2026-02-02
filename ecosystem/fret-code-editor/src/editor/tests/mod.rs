use super::*;

#[derive(Default)]
struct TestHost {
    models: fret_runtime::ModelStore,
    next_timer: u64,
    next_clipboard: u64,
}

impl fret_ui::action::UiActionHost for TestHost {
    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
        &mut self.models
    }

    fn push_effect(&mut self, _effect: fret_runtime::Effect) {}

    fn request_redraw(&mut self, _window: fret_core::AppWindowId) {}

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        self.next_timer = self.next_timer.saturating_add(1);
        fret_runtime::TimerToken(self.next_timer)
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        self.next_clipboard = self.next_clipboard.saturating_add(1);
        fret_runtime::ClipboardToken(self.next_clipboard)
    }
}

impl fret_ui::action::UiFocusActionHost for TestHost {
    fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
}

#[test]
fn replace_buffer_resets_state() {
    let handle = CodeEditorHandle::new("hello");

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 3,
        };
        st.dragging = true;
        st.drag_pointer = Some(fret_core::PointerId(1));
        st.row_text_cache.insert(0, ("hello".into(), 1));
        st.row_text_cache_queue.push_back((0, 1));
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..5,
                    caret_stops: vec![(0, Px(0.0))],
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: None,
                },
                1,
            ),
        );
        st.row_geom_cache_queue.push_back((0, 1));
    }

    let doc = DocId::new();
    let buffer = TextBuffer::new(doc, "world".to_string()).unwrap();
    handle.replace_buffer(buffer);

    let st = handle.state.borrow();
    assert_eq!(st.buffer.text_string(), "world");
    assert_eq!(st.selection, Selection::default());
    assert_eq!(st.preedit, None);
    assert!(st.undo_group.is_none());
    assert!(!st.dragging);
    assert_eq!(st.drag_pointer, None);
    assert_eq!(st.row_text_cache.len(), 0);
    assert_eq!(st.row_text_cache_queue.len(), 0);
    assert_eq!(st.row_geom_cache.len(), 0);
    assert_eq!(st.row_geom_cache_queue.len(), 0);
}

#[test]
fn replace_buffer_preserves_text_boundary_mode() {
    let handle = CodeEditorHandle::new("hello");
    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

    let doc = DocId::new();
    let buffer = TextBuffer::new(doc, "world".to_string()).unwrap();
    handle.replace_buffer(buffer);

    assert_eq!(handle.text_boundary_mode(), TextBoundaryMode::UnicodeWord);
}

#[test]
fn caret_stops_hit_test_picks_nearest_stop() {
    let stops = vec![(0, Px(0.0)), (1, Px(10.0)), (2, Px(20.0)), (3, Px(30.0))];
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-5.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(0.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(4.9)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(5.1)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(14.9)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(15.1)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(999.0)), 3);
}

#[test]
fn caret_stops_hit_test_handles_decreasing_x() {
    let stops = vec![(0, Px(30.0)), (1, Px(20.0)), (2, Px(10.0)), (3, Px(0.0))];
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(35.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(30.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(24.0)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(15.0)), 1);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(6.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-5.0)), 3);
}

#[test]
fn caret_stops_hit_test_handles_non_monotonic_x() {
    // Non-monotonic caret stops can happen on mixed-direction lines (bidi).
    let stops = vec![(0, Px(0.0)), (1, Px(30.0)), (2, Px(10.0)), (3, Px(20.0))];
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(-100.0)), 0);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(9.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(11.0)), 2);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(19.0)), 3);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(21.0)), 3);
    assert_eq!(hit_test_index_from_caret_stops(&stops, Px(999.0)), 1);
}

#[test]
fn map_row_local_to_buffer_byte_snaps_inside_preedit() {
    let doc = DocId::new();
    let buffer = TextBuffer::new(doc, "hello".to_string()).unwrap();
    let geom = RowGeom {
        row_range: 0..buffer.len_bytes(),
        caret_stops: Vec::new(),
        caret_rect_top: None,
        caret_rect_height: None,
        preedit: Some(RowPreeditMapping {
            insert_at: 2,
            preedit_len: 2,
        }),
    };

    // Before the injection point maps 1:1.
    assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 0), 0);
    assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 2), 2);

    // Inside the injected preedit snaps to the injection point.
    assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 3), 2);

    // After the injected preedit shifts by `preedit_len`.
    assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 4), 2);
    assert_eq!(map_row_local_to_buffer_byte(&buffer, &geom, 5), 3);
}

#[test]
fn caret_preferred_x_is_preserved_across_vertical_moves() {
    let handle = CodeEditorHandle::new("aaaa\nbbbb\ncccc");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };

        // Synthetic caret stops: 10px per byte.
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..4,
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: None,
                },
                1,
            ),
        );
        st.row_geom_cache.insert(
            1,
            (
                RowGeom {
                    row_range: 5..9,
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: None,
                },
                1,
            ),
        );
        st.row_geom_cache.insert(
            2,
            (
                RowGeom {
                    row_range: 10..14,
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: None,
                },
                1,
            ),
        );

        move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 7, "row 1, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));

        move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 12, "row 2, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));
    }
}

#[test]
fn row_text_cache_stats_tracks_hits_and_misses() {
    let handle = CodeEditorHandle::new("hello\nworld");
    handle.reset_cache_stats();

    {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.cache_stats.row_text_get_calls, 0);
        assert_eq!(st.cache_stats.row_text_hits, 0);
        assert_eq!(st.cache_stats.row_text_misses, 0);

        let a = cached_row_text(&mut st, 0, 8);
        let b = cached_row_text(&mut st, 0, 8);

        assert_eq!(a.as_ref(), "hello");
        assert_eq!(b.as_ref(), "hello");
        assert_eq!(st.cache_stats.row_text_get_calls, 2);
        assert_eq!(st.cache_stats.row_text_hits, 1);
        assert_eq!(st.cache_stats.row_text_misses, 1);
    }
}

#[test]
fn ctrl_page_down_bubbles_and_keeps_preedit() {
    let handle = CodeEditorHandle::new("hello\nworld");
    let preedit = PreeditState {
        text: "世界".to_string(),
        cursor: Some((0, "世".len())),
    };
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
        st.preedit = Some(preedit.clone());
    }

    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let cell_w = Cell::new(Px(10.0));

    let handled = handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::PageDown,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );

    assert!(!handled);
    let st = handle.state.borrow();
    assert_eq!(st.preedit, Some(preedit));
    assert_eq!(
        st.selection,
        Selection {
            anchor: 2,
            focus: 2
        }
    );
}

#[test]
fn caret_rect_offsets_for_preedit_cursor() {
    let handle = CodeEditorHandle::new("hello");
    let preedit = PreeditState {
        text: "ab".to_string(),
        cursor: Some((0, 2)),
    };
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        st.preedit = Some(preedit.clone());
    }

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    let st = handle.state.borrow();
    let rect =
        caret_rect_for_selection(&st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");

    assert_eq!(rect.origin.x, Px(20.0), "2 cols * 10px");
    assert_eq!(rect.origin.y, Px(0.0));
}

#[test]
fn preedit_rich_text_inserts_and_underlines() {
    let preedit = PreeditState {
        text: "世界".to_string(),
        cursor: Some((0, "世".len())),
    };
    let fg = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let selection_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };

    let rich = materialize_preedit_rich_text("hello".into(), 2, &preedit, fg, selection_bg);
    assert_eq!(rich.text.as_ref(), "he世界llo");
    assert!(rich.is_valid());
    assert!(
        rich.spans.iter().any(|s| s.paint.underline.is_some()),
        "expected preedit spans to be underlined"
    );
    assert!(
        rich.spans.iter().any(|s| s.paint.bg.is_some()),
        "expected cursor range to be highlighted"
    );
}

#[test]
fn a11y_window_maps_offsets_back_to_buffer_selection() {
    let handle = CodeEditorHandle::new("hello 😀 world");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: "hello 😀 ".len(),
            focus: "hello 😀 ".len(),
        };
        st.preedit = None;
    }

    let st = handle.state.borrow();
    let (value, selection, composition) = a11y_composed_text_window(&st);
    assert_eq!(composition, None);
    assert_eq!(value.as_str(), "hello 😀 world");
    assert_eq!(
        selection,
        Some(("hello 😀 ".len() as u32, "hello 😀 ".len() as u32))
    );

    let text_len = st.buffer.len_bytes();
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(text_len));
    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);
    assert_eq!(start, 0);
    assert_eq!(end, text_len);

    let anchor = 0u32;
    let focus = u32::try_from("hello".len()).unwrap();
    let new_anchor = map_a11y_offset_to_buffer(&st.buffer, start, end, anchor);
    let new_focus = map_a11y_offset_to_buffer(&st.buffer, start, end, focus);
    assert_eq!(new_anchor, 0);
    assert_eq!(new_focus, "hello".len());
}

#[test]
fn move_caret_vertical_clamps_in_display_row_space_when_wrapped() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    // Row 0 col 0 -> Down => row 1 col 0 (within the wrapped "abcd").
    move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 2);

    // Row 1 col 0 -> Down => row 2 col 0 (next logical line "ef").
    move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 5);

    // Row 2 is the last display row; another Down should clamp.
    move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 5);
}

#[test]
fn apply_and_record_edit_refreshes_display_map_only_when_needed() {
    let handle = CodeEditorHandle::new("ab\nc");

    {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.display_wrap_cols, None);
        assert_eq!(st.display_map.row_count(), 2);

        // No newline, no wrap => row_count should remain correct without forcing a refresh.
        apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: 0,
                text: "x".to_string(),
            },
            Selection {
                anchor: 1,
                focus: 1,
            },
        )
        .expect("apply edit");
        assert_eq!(st.buffer.line_count(), 2);
        assert_eq!(st.display_map.row_count(), 2);

        // Newline => line count changes, so the map must refresh.
        let insert_at = st.buffer.text_string().find('\n').unwrap_or(0);
        apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: insert_at,
                text: "\n".to_string(),
            },
            Selection {
                anchor: insert_at + 1,
                focus: insert_at + 1,
            },
        )
        .expect("apply edit");
        assert_eq!(st.buffer.line_count(), 3);
        assert_eq!(st.display_map.row_count(), 3);
    }

    // With wrap enabled, edits can change display rows even if line count is stable.
    let handle = CodeEditorHandle::new("ab");
    handle.set_soft_wrap_cols(Some(2));
    {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.display_map.row_count(), 1);

        apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            Edit::Insert {
                at: 2,
                text: "c".to_string(),
            },
            Selection {
                anchor: 3,
                focus: 3,
            },
        )
        .expect("apply edit");
        assert_eq!(st.display_map.row_count(), 2);
    }
}

#[test]
fn home_end_move_within_wrapped_display_rows() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };

    // caret at byte 3 is in the second wrapped row ("cd"): row start is byte 2, end is byte 4.
    move_caret_home_end(&mut st, true, false, false);
    assert_eq!(st.selection.caret(), 2);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    move_caret_home_end(&mut st, false, false, false);
    assert_eq!(st.selection.caret(), 4);

    // Ctrl+Home/End should clamp to document bounds.
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    move_caret_home_end(&mut st, true, true, false);
    assert_eq!(st.selection.caret(), 0);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    move_caret_home_end(&mut st, false, true, false);
    assert_eq!(st.selection.caret(), st.buffer.len_bytes());
}

#[test]
fn page_down_moves_by_viewport_rows_and_scrolls() {
    let handle = CodeEditorHandle::new("abcd\nefgh\nijkl\nmnop\nqrst\n");
    handle.set_soft_wrap_cols(Some(2));

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let row_h = Px(10.0);
    scroll.set_viewport_size(Size::new(Px(100.0), Px(25.0))); // 2 rows
    scroll.set_content_size(Size::new(Px(100.0), Px(10_000.0)));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    move_caret_page(&mut st, 1, false, row_h, &scroll, Px(10.0));

    let expected = st
        .display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(2, 0));
    assert_eq!(st.selection.caret(), expected);
    assert_eq!(scroll.offset().y, Px(20.0));
}

#[test]
fn delete_word_backward_removes_previous_word() {
    let handle = CodeEditorHandle::new("hello world");
    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

    let mut st = handle.state.borrow_mut();
    let end = st.buffer.len_bytes();
    st.selection = Selection {
        anchor: end,
        focus: end,
    };

    delete_word_backward(&mut st);
    assert_eq!(st.buffer.text_string(), "hello ");
    assert_eq!(st.selection.caret(), "hello ".len());
}

#[test]
fn delete_word_forward_removes_next_word() {
    let handle = CodeEditorHandle::new("hello world");
    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    delete_word_forward(&mut st);
    assert_eq!(st.buffer.text_string(), " world");
    assert_eq!(st.selection.caret(), 0);
}

#[cfg(feature = "syntax-rust")]
#[test]
fn rust_syntax_spans_are_materialized_for_rows() {
    let handle = CodeEditorHandle::new("fn main() {\n    let x = 1;\n}\n");
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let line_count = st.buffer.line_count();
    assert!(line_count > 0);

    let mut any_highlight = false;
    for row in 0..line_count {
        let spans = cached_row_syntax_spans(&mut st, row, 256);
        if !spans.is_empty() {
            any_highlight = true;
            break;
        }
    }
    assert!(
        any_highlight,
        "expected at least one highlighted span for rust"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_preserves_far_rows_on_inline_edit() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;
    let _ = cached_row_syntax_spans(&mut st, 0, max_entries);
    let _ = cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "x".to_string(),
        },
        Selection {
            anchor: 1,
            focus: 1,
        },
    )
    .expect("apply edit");

    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to survive inline edit invalidation"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );
}
