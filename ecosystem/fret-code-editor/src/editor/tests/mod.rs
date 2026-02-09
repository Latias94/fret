use super::*;
use super::{input, paint};
use fret_core::Point;
use std::sync::Arc;

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
        st.drag_autoscroll_timer = Some(TimerToken(123));
        st.drag_autoscroll_viewport_pos = Some(fret_core::Point::new(Px(12.0), Px(34.0)));
        st.row_text_cache.insert(
            0,
            (
                RowTextCacheEntry {
                    text: Arc::from("hello"),
                    range: 0..5,
                    fold_map: None,
                },
                1,
            ),
        );
        st.row_text_cache_queue.push_back((0, 1));
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..5,
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
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
    assert!(st.drag_autoscroll_timer.is_none());
    assert!(st.drag_autoscroll_viewport_pos.is_none());
    assert_eq!(st.row_text_cache.len(), 0);
    assert_eq!(st.row_text_cache_queue.len(), 0);
    assert_eq!(st.row_geom_cache.len(), 0);
    assert_eq!(st.row_geom_cache_queue.len(), 0);
}

#[test]
fn drag_autoscroll_delta_y_is_zero_inside_safe_band() {
    let delta = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(50.0));
    assert_eq!(delta, Px(0.0));
}

#[test]
fn drag_autoscroll_delta_y_uses_direction_and_clamp() {
    let up = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(20.0));
    assert!(up.0 < 0.0);

    let down = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(80.0));
    assert!(down.0 > 0.0);

    let capped = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(10_000.0));
    assert!((capped.0 - 3.0).abs() < 1e-6);
}

#[test]
fn drag_autoscroll_delta_y_handles_zero_sizes() {
    assert_eq!(
        drag_autoscroll_delta_y(Px(0.0), Px(100.0), Px(20.0)),
        Px(0.0)
    );
    assert_eq!(
        drag_autoscroll_delta_y(Px(20.0), Px(0.0), Px(20.0)),
        Px(0.0)
    );
}

#[test]
fn display_row_for_pointer_y_clamps_outside_viewport() {
    let bounds = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(100.0), Px(60.0)),
    );

    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(5.0), 5),
        Some(0)
    );
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(25.0), 5),
        Some(0)
    );
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(95.0), 5),
        Some(4)
    );
}

#[test]
fn display_row_for_pointer_y_rejects_invalid_inputs() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(1.0), 0),
        None
    );
    assert_eq!(display_row_for_pointer_y(bounds, Px(0.0), Px(1.0), 2), None);
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
fn text_boundary_mode_override_can_be_cleared() {
    let handle = CodeEditorHandle::new("hello");
    assert_eq!(
        handle.text_boundary_mode_override(),
        Some(TextBoundaryMode::Identifier)
    );

    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);
    assert_eq!(
        handle.text_boundary_mode_override(),
        Some(TextBoundaryMode::UnicodeWord)
    );

    handle.set_text_boundary_mode_override(None);
    assert_eq!(handle.text_boundary_mode_override(), None);
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
        blob: fret_core::TextBlobId::default(),
        caret_stops: Vec::new(),
        fold_map: None,
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
fn row_fold_map_maps_between_buffer_and_display() {
    let map = geom::RowFoldMap::new(vec![geom::RowFoldSpan {
        buffer_range: 1..4,
        // U+2026 is 3 bytes in UTF-8, so a placeholder "…" at offset 1 occupies [1,4).
        display_range: 1..4,
    }]);

    assert_eq!(map.buffer_local_to_display_local(0), 0);
    assert_eq!(map.buffer_local_to_display_local(1), 1);
    assert_eq!(map.buffer_local_to_display_local(2), 1);
    assert_eq!(map.buffer_local_to_display_local(3), 1);
    assert_eq!(map.buffer_local_to_display_local(4), 4);
    assert_eq!(map.buffer_local_to_display_local(5), 5);

    assert_eq!(map.display_local_to_buffer_local(0), 0);
    assert_eq!(map.display_local_to_buffer_local(1), 1);
    assert_eq!(map.display_local_to_buffer_local(2), 1);
    assert_eq!(map.display_local_to_buffer_local(3), 1);
    assert_eq!(map.display_local_to_buffer_local(4), 4);
    assert_eq!(map.display_local_to_buffer_local(5), 5);
}

#[test]
fn row_fold_map_handles_inlay_insertions() {
    let map = geom::RowFoldMap::new(vec![geom::RowFoldSpan {
        buffer_range: 2..2,
        display_range: 2..6,
    }]);

    assert_eq!(map.buffer_local_to_display_local(2), 2);
    assert_eq!(map.buffer_local_to_display_local(3), 7);

    assert_eq!(map.display_local_to_buffer_local(2), 2);
    assert_eq!(map.display_local_to_buffer_local(3), 2);
    assert_eq!(map.display_local_to_buffer_local(6), 2);
    assert_eq!(map.display_local_to_buffer_local(7), 3);
}

#[test]
fn caret_left_right_skips_folded_ranges() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        input::move_caret_right(&mut st, false);
        assert_eq!(st.selection.caret(), 4);

        input::move_caret_left(&mut st, false);
        assert_eq!(st.selection.caret(), 1);
    }
}

#[test]
fn caret_left_right_skips_folded_ranges_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        input::move_caret_right(&mut st, false);
        assert_eq!(st.selection.caret(), 4);

        input::move_caret_left(&mut st, false);
        assert_eq!(st.selection.caret(), 1);
    }
}

#[test]
fn enabling_folds_snaps_caret_out_of_folded_range() {
    let handle = CodeEditorHandle::new("abcdef");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
    }

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    let st = handle.state.borrow();
    assert_eq!(st.selection.caret(), 1);
}

#[test]
fn enabling_folds_snaps_caret_out_of_folded_range_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
    }

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }],
    );

    let st = handle.state.borrow();
    assert_eq!(st.selection.caret(), 1);
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
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
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
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
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
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![
                        (0, Px(0.0)),
                        (1, Px(10.0)),
                        (2, Px(20.0)),
                        (3, Px(30.0)),
                        (4, Px(40.0)),
                    ],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: None,
                },
                1,
            ),
        );

        input::move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 7, "row 1, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));

        input::move_caret_vertical(&mut st, 1, false, Px(8.0));
        assert_eq!(st.selection.caret(), 12, "row 2, local index 2");
        assert_eq!(st.caret_preferred_x, Some(Px(20.0)));
    }
}

#[test]
fn row_geom_cache_is_shifted_across_single_line_soft_wrap_edits() {
    let handle = CodeEditorHandle::new("aaaaaa\nbbbbbb\ncccccc");
    handle.set_soft_wrap_cols(Some(4));

    let before = {
        let mut st = handle.state.borrow_mut();
        assert_eq!(
            st.display_map.row_count(),
            6,
            "3 lines * 2 wrapped rows each"
        );

        // Seed geometry for the second and third logical lines (rows 2..6).
        for row in 2..6 {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.row_geom_cache.insert(
                row,
                (
                    RowGeom {
                        row_range: range.clone(),
                        blob: fret_core::TextBlobId::default(),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache_queue.push_back((row, 1));
        }

        // Capture the original ranges so we can assert on the shifted values after the edit.
        (2..6)
            .map(|row| {
                (
                    row,
                    st.row_geom_cache.get(&row).unwrap().0.row_range.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    {
        let mut st = handle.state.borrow_mut();
        let at = 6; // End of the first logical line.
        let edit = Edit::Insert {
            at,
            text: "zzzz".to_string(),
        };
        let caret = at + 4;
        input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            edit,
            Selection {
                anchor: caret,
                focus: caret,
            },
        )
        .expect("edit must apply");
        assert_eq!(
            st.display_map.row_count(),
            7,
            "inserting four chars grows the first line from 2 -> 3 wrapped rows"
        );
    }

    let st = handle.state.borrow();
    assert_eq!(
        st.row_geom_cache.len(),
        4,
        "unaffected lines keep row geometry cached"
    );

    for (old_row, old_range) in before {
        let new_row = old_row + 1;
        let (geom, _) = st
            .row_geom_cache
            .get(&new_row)
            .expect("shifted row present");
        assert_eq!(
            geom.row_range,
            (old_range.start + 4)..(old_range.end + 4),
            "byte ranges shift by the inserted text length"
        );
    }
}

#[test]
fn row_geom_cache_is_byte_shifted_for_single_line_non_wrap_edits() {
    let handle = CodeEditorHandle::new("hello\nworld\nagain");

    let before = {
        let mut st = handle.state.borrow_mut();
        assert_eq!(st.display_map.row_count(), 3);

        for row in 1..3 {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.row_geom_cache.insert(
                row,
                (
                    RowGeom {
                        row_range: range.clone(),
                        blob: fret_core::TextBlobId::default(),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        preedit: None,
                    },
                    1,
                ),
            );
            st.row_geom_cache_queue.push_back((row, 1));
        }
        (1..3)
            .map(|row| {
                (
                    row,
                    st.row_geom_cache.get(&row).unwrap().0.row_range.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    {
        let mut st = handle.state.borrow_mut();
        let edit = Edit::Insert {
            at: 0,
            text: "123".to_string(),
        };
        input::apply_and_record_edit(
            &mut st,
            UndoGroupKind::Typing,
            edit,
            Selection {
                anchor: 3,
                focus: 3,
            },
        )
        .expect("edit must apply");
        assert_eq!(
            st.display_map.row_count(),
            3,
            "non-wrapping edits keep row count stable"
        );
    }

    let st = handle.state.borrow();
    assert_eq!(st.row_geom_cache.len(), 2);
    for (row, old_range) in before {
        let (geom, _) = st.row_geom_cache.get(&row).expect("row present");
        assert_eq!(
            geom.row_range,
            (old_range.start + 3)..(old_range.end + 3),
            "byte ranges shift by the inserted text length"
        );
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

        let a = paint::cached_row_text(&mut st, 0, 8);
        let b = paint::cached_row_text(&mut st, 0, 8);

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

    let handled = input::handle_key_down(
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
fn read_only_allows_navigation_but_blocks_edits() {
    let handle = CodeEditorHandle::new("hello");
    handle.set_caret(5);
    handle.set_interaction(CodeEditorInteractionOptions::read_only());

    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let cell_w = Cell::new(Px(10.0));

    let handled = input::handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::Backspace,
        Modifiers::default(),
    );
    assert!(handled);
    assert_eq!(handle.with_buffer(|b| b.text_string()), "hello");
    assert_eq!(handle.selection().caret(), 5);

    let handled = input::handle_key_down(
        &mut host,
        action_cx,
        &handle.state,
        Px(16.0),
        &scroll,
        &cell_w,
        KeyCode::ArrowLeft,
        Modifiers::default(),
    );
    assert!(handled);
    assert_eq!(handle.selection().caret(), 4);

    {
        let mut st = handle.state.borrow_mut();
        assert!(input::insert_text(&mut st, "x").is_none());
        assert!(!input::undo(&mut st));
        assert!(!input::redo(&mut st));
    }
    assert_eq!(handle.with_buffer(|b| b.text_string()), "hello");
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

    let mut st = handle.state.borrow_mut();
    let rect =
        caret_rect_for_selection(&mut st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");

    assert_eq!(rect.origin.x, Px(20.0), "2 cols * 10px");
    assert_eq!(rect.origin.y, Px(0.0));
}

#[test]
fn caret_rect_ignores_stale_row_geom_with_preedit_mapping() {
    let handle = CodeEditorHandle::new("abc");
    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        st.preedit = None;
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..3,
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![(0, Px(0.0)), (1, Px(100.0)), (2, Px(200.0)), (3, Px(300.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: Some(RowPreeditMapping {
                        insert_at: 0,
                        preedit_len: 2,
                    }),
                },
                1,
            ),
        );
    }

    let mut st = handle.state.borrow_mut();
    let rect =
        caret_rect_for_selection(&mut st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");
    assert_eq!(rect.origin.x, Px(10.0), "fallback col (1) * cell_w (10px)");
}

#[test]
fn caret_for_pointer_ignores_stale_row_geom_with_preedit_mapping() {
    let handle = CodeEditorHandle::new("abc");
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    {
        let mut st = handle.state.borrow_mut();
        st.preedit = None;
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0..3,
                    blob: fret_core::TextBlobId::default(),
                    caret_stops: vec![(0, Px(0.0)), (1, Px(100.0)), (2, Px(200.0)), (3, Px(300.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    preedit: Some(RowPreeditMapping {
                        insert_at: 0,
                        preedit_len: 2,
                    }),
                },
                1,
            ),
        );
    }

    let mut st = handle.state.borrow_mut();
    let caret = caret_for_pointer(
        &mut st,
        0,
        bounds,
        fret_core::Point::new(Px(15.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(
        caret, 1,
        "expected fallback monospace hit-test (x=15 -> col 1)"
    );
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

    let rich = paint::materialize_preedit_rich_text("hello".into(), 2, &preedit, fg, selection_bg);
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
fn a11y_window_includes_preedit_and_reports_composition_range() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
        st.preedit = Some(PreeditState {
            text: "ab".to_string(),
            cursor: Some((0, "a".len())),
        });
    }

    let st = handle.state.borrow();
    let (value, selection, composition) = a11y_composed_text_window(&st);
    assert_eq!(value.as_str(), "heabllo");
    assert_eq!(composition, Some((2, 2 + "ab".len() as u32)));
    assert_eq!(selection, Some((2, 2 + "a".len() as u32)));
}

#[test]
fn a11y_window_maps_offsets_back_to_buffer_selection_with_preedit() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        st.preedit = Some(PreeditState {
            text: "AB".to_string(),
            cursor: Some((2, 2)),
        });
    }

    let st = handle.state.borrow();
    let (value, _selection, composition) = a11y_composed_text_window(&st);
    assert_eq!(value.as_str(), "ABhello");
    assert_eq!(composition, Some((0, 2)));

    let text_len = st.buffer.len_bytes();
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(text_len));
    let (start, end) = a11y_text_window_bounds(&st.buffer, caret);

    let mapped = map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 3);
    assert_eq!(
        mapped, 1,
        "display offset after preedit should map into base text"
    );

    let inside_preedit =
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 1);
    assert_eq!(
        inside_preedit, 0,
        "display offset inside preedit snaps to insertion caret"
    );

    let clamped_end =
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, u32::MAX);
    assert_eq!(clamped_end, st.buffer.len_bytes());
}

#[test]
fn a11y_preedit_offset_mapping_honors_window_start() {
    let handle = CodeEditorHandle::new("0123456789");
    let st = handle.state.borrow();

    let start = 2;
    let end = 8;
    let caret = 5;
    let preedit_len = 2;

    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 0),
        start
    );
    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 3),
        caret
    );
    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 4),
        caret
    );
    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 5),
        caret
    );
    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 6),
        caret + 1
    );
    assert_eq!(
        map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, preedit_len, 7),
        caret + 2
    );
}

#[test]
fn pointer_down_double_click_selects_word_and_cancels_preedit() {
    let handle = CodeEditorHandle::new("foo_bar baz");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        st.preedit = Some(PreeditState {
            text: "x".to_string(),
            cursor: Some((0, 1)),
        });

        let caret = "foo_".len();
        let (expect_start, expect_end) =
            select_word_range_in_buffer(&st.buffer, caret, st.active_text_boundary_mode);

        input::apply_pointer_down_selection(&mut st, 0, caret, 2, false);

        assert_eq!(st.preedit, None);
        assert_eq!(
            st.selection,
            Selection {
                anchor: expect_start,
                focus: expect_end,
            }
        );
    }
}

#[test]
fn pointer_down_triple_click_selects_logical_line_including_newline_and_cancels_preedit() {
    let handle = CodeEditorHandle::new("abc\ndef\n");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        st.preedit = Some(PreeditState {
            text: "x".to_string(),
            cursor: Some((0, 1)),
        });

        let row = 1;
        let caret = "abc\n".len() + 1;
        input::apply_pointer_down_selection(&mut st, row, caret, 3, false);

        assert_eq!(st.preedit, None);
        assert_eq!(
            st.selection.normalized(),
            "abc\n".len()..("abc\ndef\n".len())
        );
    }
}

#[test]
fn pointer_down_shift_click_extends_selection_and_cancels_preedit() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        st.preedit = Some(PreeditState {
            text: "x".to_string(),
            cursor: Some((0, 1)),
        });

        input::apply_pointer_down_selection(&mut st, 0, 4, 1, true);

        assert_eq!(st.preedit, None);
        assert_eq!(
            st.selection,
            Selection {
                anchor: 1,
                focus: 4,
            }
        );
    }
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
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 2);

    // Row 1 col 0 -> Down => row 2 col 0 (next logical line "ef").
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), 5);

    // Row 2 is the last display row; another Down should clamp.
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
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
        input::apply_and_record_edit(
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
        input::apply_and_record_edit(
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

        input::apply_and_record_edit(
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
    input::move_caret_home_end(&mut st, true, false, false);
    assert_eq!(st.selection.caret(), 2);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, false, false);
    assert_eq!(st.selection.caret(), 4);

    // Ctrl+Home/End should clamp to document bounds.
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, true, true, false);
    assert_eq!(st.selection.caret(), 0);

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, true, false);
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

    input::move_caret_page(&mut st, 1, false, row_h, &scroll, Px(10.0));

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

    input::delete_word_backward(&mut st);
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

    input::delete_word_forward(&mut st);
    assert_eq!(st.buffer.text_string(), " world");
    assert_eq!(st.selection.caret(), 0);
}

#[test]
fn move_word_right_respects_text_boundary_mode_for_apostrophe() {
    let handle = CodeEditorHandle::new("can't");

    handle.set_text_boundary_mode(TextBoundaryMode::UnicodeWord);
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        input::move_word(&mut st, 1, false);
        assert_eq!(
            st.selection.caret(),
            "can't".len(),
            "UnicodeWord should treat \"can't\" as a single word"
        );
    }

    handle.set_text_boundary_mode(TextBoundaryMode::Identifier);
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 0,
            focus: 0,
        };
        input::move_word(&mut st, 1, false);
        assert_eq!(
            st.selection.caret(),
            3,
            "Identifier should split \"can't\" around the apostrophe"
        );
    }
}

#[test]
fn pointer_down_double_click_matches_identifier_boundary_under_soft_wrap_for_mixed_scripts() {
    let mixed_identifier = format!("{}_foo42", "\u{53D8}\u{91CF}");
    let text = format!("can't {mixed_identifier}.bar");
    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_text_boundary_mode(TextBoundaryMode::Identifier);
    handle.set_soft_wrap_cols(Some(6));

    let mut st = handle.state.borrow_mut();
    st.preedit = Some(PreeditState {
        text: "AB".to_string(),
        cursor: Some((2, 2)),
    });

    let caret = text.find("foo42").expect("expected mixed identifier token") + 1;
    let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let (expect_start, expect_end) =
        select_word_range_in_buffer(&st.buffer, caret, st.active_text_boundary_mode);

    input::apply_pointer_down_selection(&mut st, row, caret, 2, false);

    let selected = st
        .buffer
        .slice_to_string(expect_start..expect_end)
        .unwrap_or_default();
    assert_eq!(st.preedit, None);
    assert_eq!(selected.as_str(), mixed_identifier.as_str());
    assert_eq!(
        st.selection,
        Selection {
            anchor: expect_start,
            focus: expect_end,
        }
    );
}

#[test]
fn move_word_navigation_uses_same_boundaries_under_soft_wrap_with_punctuation() {
    let text = format!("can't {}_foo42.bar", "\u{53D8}\u{91CF}");
    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_text_boundary_mode(TextBoundaryMode::Identifier);
    handle.set_soft_wrap_cols(Some(5));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    let mut expected = 0usize;
    for _ in 0..4 {
        expected = move_word_right_in_buffer(&st.buffer, expected, st.active_text_boundary_mode)
            .min(st.buffer.len_bytes());
        assert!(input::move_word(&mut st, 1, false));
        assert_eq!(st.selection.anchor, st.selection.focus);
        assert_eq!(st.selection.caret(), expected);
    }

    for _ in 0..4 {
        expected = move_word_left_in_buffer(&st.buffer, expected, st.active_text_boundary_mode)
            .min(st.buffer.len_bytes());
        assert!(input::move_word(&mut st, -1, false));
        assert_eq!(st.selection.anchor, st.selection.focus);
        assert_eq!(st.selection.caret(), expected);
    }
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
        let spans = paint::cached_row_syntax_spans(&mut st, row, 256);
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
    let _ = paint::cached_row_syntax_spans(&mut st, 0, max_entries);
    let _ = paint::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
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

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_newline_insertion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let _ = paint::cached_row_syntax_spans(&mut st, 0, max_entries);
    let spans_150 = paint::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "\n".to_string(),
        },
        Selection {
            anchor: 1,
            focus: 1,
        },
    )
    .expect("apply edit");

    let shifted_row = 151;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let spans_after = paint::cached_row_syntax_spans(&mut st, shifted_row, max_entries);
    assert!(
        st.cache_stats.syntax_hits > hits_before,
        "expected shifted far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&spans_after, &spans_150));
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_newline_deletion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let newline = text.find('\n').expect("expected a newline");

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let _ = paint::cached_row_syntax_spans(&mut st, 0, max_entries);
    let spans_150 = paint::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Delete {
            range: newline..newline + 1,
        },
        Selection {
            anchor: newline,
            focus: newline,
        },
    )
    .expect("apply edit");

    let shifted_row = 149;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let spans_after = paint::cached_row_syntax_spans(&mut st, shifted_row, max_entries);
    assert!(
        st.cache_stats.syntax_hits > hits_before,
        "expected shifted far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&spans_after, &spans_150));
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_invalidates_bounded_window_around_edit() {
    let mut text = String::new();
    for _ in 0..300 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let line_10 = paint::cached_row_syntax_spans(&mut st, 10, max_entries);
    let line_50 = paint::cached_row_syntax_spans(&mut st, 50, max_entries);
    let line_150 = paint::cached_row_syntax_spans(&mut st, 150, max_entries);
    let line_200 = paint::cached_row_syntax_spans(&mut st, 200, max_entries);

    assert!(st.syntax_row_cache.contains_key(&10));
    assert!(st.syntax_row_cache.contains_key(&50));
    assert!(st.syntax_row_cache.contains_key(&150));
    assert!(st.syntax_row_cache.contains_key(&200));

    let edit_line = 100usize;
    let at = st
        .buffer
        .line_start(edit_line)
        .expect("expected line start");

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at,
            text: "x".to_string(),
        },
        Selection {
            anchor: at + 1,
            focus: at + 1,
        },
    )
    .expect("apply edit");

    assert!(
        st.syntax_row_cache.contains_key(&10),
        "expected far-row entry outside the invalidation window to survive"
    );
    assert!(
        st.syntax_row_cache.contains_key(&200),
        "expected far-row entry outside the invalidation window to survive"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&50),
        "expected entry inside the lookback/lookahead invalidation window to be evicted"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&150),
        "expected entry inside the lookback/lookahead invalidation window to be evicted"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let line_10_after = paint::cached_row_syntax_spans(&mut st, 10, max_entries);
    let line_200_after = paint::cached_row_syntax_spans(&mut st, 200, max_entries);
    assert!(
        st.cache_stats.syntax_hits >= hits_before + 2,
        "expected preserved far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&line_10_after, &line_10));
    assert!(Arc::ptr_eq(&line_200_after, &line_200));

    let _ = line_50;
    let _ = line_150;
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_multiple_newline_insertion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;
    let spans_150 = paint::cached_row_syntax_spans(&mut st, 150, max_entries);

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "\n\n\n".to_string(),
        },
        Selection {
            anchor: 3,
            focus: 3,
        },
    )
    .expect("apply edit");

    let shifted_row = 153;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_multiple_line_deletion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let spans_150 = paint::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    let end = st.buffer.line_start(3).expect("expected a line start");
    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Delete { range: 0..end },
        Selection {
            anchor: 0,
            focus: 0,
        },
    )
    .expect("apply edit");

    let shifted_row = 147;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
}
