use super::geom::map_row_display_local_to_buffer_byte;
use super::*;
use super::{input, paint};
use fret_core::Point;
use std::sync::Arc;

#[derive(Default)]
struct TestHost {
    models: fret_runtime::ModelStore,
    next_timer: u64,
    next_clipboard: u64,
    next_share_sheet: u64,
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

    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
        self.next_share_sheet = self.next_share_sheet.saturating_add(1);
        fret_runtime::ShareSheetToken(self.next_share_sheet)
    }
}

impl fret_ui::action::UiFocusActionHost for TestHost {
    fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
}

fn row_geom_key_for_tests(text: &Arc<str>) -> geom::RowGeomKey {
    geom::RowGeomKey::for_plain(
        text,
        &TextStyle::default(),
        (
            None,
            TextWrap::None,
            TextOverflow::Clip,
            fret_core::TextAlign::Start,
            1.0,
        ),
        fret_runtime::TextFontStackKey(0),
    )
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
                    preedit_range: None,
                    row_spans: Arc::from([]),
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
                    key: row_geom_key_for_tests(&Arc::from("hello")),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
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
fn composition_selection_replacement_is_reflected_in_a11y_window() {
    let handle = CodeEditorHandle::new("hello world");
    let text_cache_max_entries = 64;

    let (base_value, base_selection, base_composition) = {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 6,
            focus: 11,
        };
        super::a11y::a11y_composed_text_window(&mut st, text_cache_max_entries)
    };

    assert_eq!(base_value.as_str(), "hello world");
    assert_eq!(base_selection, Some((6, 11)));
    assert_eq!(base_composition, None);

    let did = {
        let mut st = handle.state.borrow_mut();
        super::platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            text_cache_max_entries,
            base_value.as_str(),
            fret_runtime::Utf16Range::new(6, 11),
            "X",
            Some(fret_runtime::Utf16Range::new(6, 7)),
        )
    };
    assert!(did, "expected replace-and-mark to update editor state");

    let (value, selection, composition) = {
        let mut st = handle.state.borrow_mut();
        super::a11y::a11y_composed_text_window(&mut st, text_cache_max_entries)
    };
    assert_eq!(value.as_str(), "hello X");
    assert_eq!(selection, Some((6, 7)));
    assert_eq!(composition, Some((6, 7)));

    {
        let mut st = handle.state.borrow_mut();
        let start = super::a11y::map_a11y_offset_to_buffer_in_current_window(
            &mut st,
            text_cache_max_entries,
            6,
        );
        let after_preedit = super::a11y::map_a11y_offset_to_buffer_in_current_window(
            &mut st,
            text_cache_max_entries,
            7,
        );
        assert_eq!(start, 6);
        assert_eq!(after_preedit, 11);
    }

    {
        let mut st = handle.state.borrow_mut();
        st.set_preedit(None);
        assert!(st.preedit_replace_range.is_none());
    }
}

#[test]
fn cached_row_text_hits_and_reuses_arc_for_repeated_calls() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (a, b, stats) = {
        let mut st = handle.state.borrow_mut();
        let (_range_a, a, _folds_a, _preedit_a, _spans_a) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        let (_range_b, b, _folds_b, _preedit_b, _spans_b) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        (a, b, st.cache_stats)
    };

    assert!(
        Arc::ptr_eq(&a, &b),
        "expected row text cache to reuse Arc<str>"
    );
    assert_eq!(stats.row_text_misses, 1);
    assert_eq!(stats.row_text_hits, 1);
}

#[test]
fn cached_row_text_invalidates_on_buffer_revision_change() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (before, after, resets) = {
        let mut st = handle.state.borrow_mut();
        let (_range, before, _folds, _preedit, _spans) =
            paint::cached_row_text_with_range(&mut st, 0, 64);

        let mut tx = st.buffer.transaction_begin();
        st.buffer
            .transaction_update(
                &mut tx,
                Edit::Insert {
                    at: 0,
                    text: "!".to_string(),
                },
            )
            .expect("edit");
        let _ = st.buffer.transaction_commit(tx);
        st.refresh_display_map();

        let (_range, after, _folds, _preedit, _spans) =
            paint::cached_row_text_with_range(&mut st, 0, 64);
        (before, after, st.cache_stats.row_text_resets)
    };

    assert!(
        !Arc::ptr_eq(&before, &after),
        "expected row text cache to invalidate when buffer revision changes"
    );
    assert!(resets > 0, "expected row text cache resets to be recorded");
}

#[test]
fn cached_row_text_lru_eviction_rebuilds_evicted_rows() {
    let handle = CodeEditorHandle::new("hello\nworld");

    let (first0, first1, second0, stats) = {
        let mut st = handle.state.borrow_mut();
        let (_range0, first0, _folds0, _preedit0, _spans0) =
            paint::cached_row_text_with_range(&mut st, 0, 1);
        let (_range1, first1, _folds1, _preedit1, _spans1) =
            paint::cached_row_text_with_range(&mut st, 1, 1);
        let (_range0, second0, _folds0, _preedit0, _spans0) =
            paint::cached_row_text_with_range(&mut st, 0, 1);
        (first0, first1, second0, st.cache_stats)
    };

    assert_eq!(first0.as_ref(), "hello");
    assert_eq!(first1.as_ref(), "world");
    assert!(
        !Arc::ptr_eq(&first0, &second0),
        "expected row 0 to be rebuilt after eviction under max_entries=1"
    );
    assert!(
        stats.row_text_evictions > 0,
        "expected at least one eviction"
    );
}

#[test]
fn paint_source_does_not_materialize_whole_buffer_string() {
    // Regression guard: the editor paint path should never call `TextBuffer::text_string()`.
    // Materializing the entire rope would scale with document size and defeat row virtualization.
    const SRC: &str = include_str!("../paint/mod.rs");
    assert!(
        !SRC.contains(".text_string("),
        "paint/mod.rs must not call TextBuffer::text_string()"
    );
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
fn map_row_display_local_to_buffer_byte_snaps_inside_preedit() {
    let doc = DocId::new();
    let buffer = TextBuffer::new(doc, "hello".to_string()).unwrap();
    let geom = RowGeom {
        row_range: 0..buffer.len_bytes(),
        key: row_geom_key_for_tests(&Arc::from("hello")),
        caret_stops: Vec::new(),
        fold_map: None,
        caret_rect_top: None,
        caret_rect_height: None,
        has_preedit: true,
        preedit: Some(RowPreeditMapping {
            insert_at: 2,
            preedit_len: 2,
        }),
    };

    // Before the injection point maps 1:1.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 0), 0);
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 2), 2);

    // Inside the injected preedit snaps to the injection point.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 3), 2);

    // After the injected preedit shifts by `preedit_len`.
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 4), 2);
    assert_eq!(map_row_display_local_to_buffer_byte(&buffer, &geom, 5), 3);
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
                    key: row_geom_key_for_tests(&Arc::from("aaaa")),
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
                    has_preedit: false,
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
                    key: row_geom_key_for_tests(&Arc::from("bbbb")),
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
                    has_preedit: false,
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
                    key: row_geom_key_for_tests(&Arc::from("cccc")),
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
                    has_preedit: false,
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
                        key: row_geom_key_for_tests(&Arc::from("")),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        has_preedit: false,
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
                        key: row_geom_key_for_tests(&Arc::from("")),
                        caret_stops: vec![(0, Px(0.0))],
                        fold_map: None,
                        caret_rect_top: None,
                        caret_rect_height: None,
                        has_preedit: false,
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
fn ctrl_a_selects_all() {
    let handle = CodeEditorHandle::new("hello\nworld");
    handle.set_caret(3);

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
        KeyCode::KeyA,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(handled);

    let st = handle.state.borrow();
    assert_eq!(st.selection.anchor, 0);
    assert_eq!(st.selection.focus, st.buffer.len_bytes());
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
fn platform_marked_range_utf16_maps_to_preedit_cursor_bytes() {
    let text = "a😀b";
    let base = 10u32;

    let (bs, be) = preedit_cursor_bytes_for_marked_range_utf16(
        base,
        fret_runtime::Utf16Range::new(base + 1, base + 3),
        text,
    );
    assert_eq!(&text[bs..be], "😀");

    let (bs, be) = preedit_cursor_bytes_for_marked_range_utf16(
        base,
        fret_runtime::Utf16Range::new(base + 2, base + 2),
        text,
    );
    assert_eq!(&text[bs..be], "😀", "clamps inside surrogate pair");

    let (bs, be) = preedit_cursor_bytes_for_marked_range_utf16(
        base,
        fret_runtime::Utf16Range::new(base, base + 1),
        text,
    );
    assert_eq!(&text[bs..be], "a");
}

#[test]
fn platform_replace_and_mark_non_empty_range_replaces_in_composed_view_without_mutating_base() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 4,
        };
        st.preedit = None;
    }

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);

    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(1, 4),
        "XY",
        Some(fret_runtime::Utf16Range::new(1, 3)),
    );
    assert!(did);
    assert_eq!(st.buffer.text_string(), "hello");
    assert_eq!(
        st.preedit.as_ref().map(|p| p.text.as_str()),
        Some("XY"),
        "composing text remains preedit-only"
    );
    assert_eq!(st.selection.caret(), 1);

    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "hXYo");
    assert_eq!(composition, Some((1, 3)));
    assert_eq!(selection, Some((1, 3)));

    let (_range, row_text, _folds, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 0, 1024);
    assert_eq!(
        row_text.as_ref(),
        "hXYo",
        "expected view-composed row text to match the platform-facing composed window"
    );
    assert_eq!(preedit_range, Some(1..3));
}

#[test]
fn platform_replace_and_mark_with_marked_none_behaves_like_replace() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 4,
        };
        st.preedit = Some(PreeditState {
            text: "AB".to_string(),
            cursor: Some((0, 2)),
        });
    }

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);

    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(1, 4),
        "X",
        None,
    );
    assert!(did);
    assert_eq!(st.buffer.text_string(), "hXo");
    assert_eq!(st.preedit, None);
    assert_eq!(st.selection.caret(), 2);
}

#[test]
fn font_stack_key_change_clears_geometry_caches() {
    let handle = CodeEditorHandle::new("hello");
    let mut st = handle.state.borrow_mut();

    st.font_stack_key = fret_runtime::TextFontStackKey(1);
    st.row_geom_cache.insert(
        0,
        (
            RowGeom {
                row_range: 0..5,
                key: row_geom_key_for_tests(&Arc::from("hello")),
                caret_stops: vec![(0, Px(0.0)), (5, Px(50.0))],
                fold_map: None,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );
    st.baseline_measure_cache = Some(BaselineMeasureCache {
        max_width: Px(100.0),
        row_h: Px(20.0),
        scale_bits: 0,
        text_style: TextStyle {
            font: FontId::monospace(),
            size: Px(12.0),
            ..Default::default()
        },
        metrics: fret_core::TextMetrics {
            size: Size::new(Px(0.0), Px(0.0)),
            baseline: Px(0.0),
        },
        measured_h: Px(0.0),
    });

    st.update_font_stack_key(fret_runtime::TextFontStackKey(1));
    assert!(!st.row_geom_cache.is_empty());
    assert!(st.baseline_measure_cache.is_some());

    st.update_font_stack_key(fret_runtime::TextFontStackKey(2));
    assert!(st.row_geom_cache.is_empty());
    assert!(st.baseline_measure_cache.is_none());
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
                    key: row_geom_key_for_tests(&Arc::from("abc")),
                    caret_stops: vec![(0, Px(0.0)), (1, Px(100.0)), (2, Px(200.0)), (3, Px(300.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: true,
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
                    key: row_geom_key_for_tests(&Arc::from("abc")),
                    caret_stops: vec![(0, Px(0.0)), (1, Px(100.0)), (2, Px(200.0)), (3, Px(300.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: true,
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
fn row_geom_key_ignores_paint_only_changes() {
    let text: Arc<str> = Arc::<str>::from("let x = 1;");
    let base = TextStyle::default();
    let constraints = (
        Some(Px(200.0)),
        TextWrap::None,
        TextOverflow::Clip,
        fret_core::TextAlign::Start,
        1.0,
    );
    let font_stack_key = fret_runtime::TextFontStackKey(7);

    let mk_rich = |kw: Color, ident: Color| {
        let spans = vec![
            TextSpan {
                len: "let".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(kw),
                    ..Default::default()
                },
            },
            TextSpan::new(" ".len()),
            TextSpan {
                len: "x".len(),
                shaping: Default::default(),
                paint: TextPaintStyle {
                    fg: Some(ident),
                    ..Default::default()
                },
            },
            TextSpan::new(" = 1;".len()),
        ];
        AttributedText::new(Arc::clone(&text), Arc::<[TextSpan]>::from(spans))
    };

    let rich_a = mk_rich(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
    );
    let rich_b = mk_rich(
        Color {
            r: 0.2,
            g: 0.2,
            b: 1.0,
            a: 1.0,
        },
        Color {
            r: 0.8,
            g: 0.8,
            b: 0.0,
            a: 1.0,
        },
    );

    assert!(
        rich_a.shaping_eq(&rich_b),
        "sanity: shaping_eq should ignore paint-only changes"
    );

    let key_a = geom::RowGeomKey::for_attributed(&rich_a, &base, constraints, font_stack_key);
    let key_b = geom::RowGeomKey::for_attributed(&rich_b, &base, constraints, font_stack_key);
    assert_eq!(
        key_a, key_b,
        "row geometry cache key must ignore paint-only changes"
    );

    let mut spans_c = rich_b.spans.as_ref().to_vec();
    spans_c[0].shaping = spans_c[0]
        .shaping
        .clone()
        .with_weight(fret_core::FontWeight(700));
    let rich_c = AttributedText::new(Arc::clone(&text), Arc::<[TextSpan]>::from(spans_c));
    let key_c = geom::RowGeomKey::for_attributed(&rich_c, &base, constraints, font_stack_key);
    assert_ne!(key_a, key_c, "shaping changes must affect geometry key");
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

    let mut st = handle.state.borrow_mut();
    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
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
    let (start, end) = super::a11y::a11y_text_window_bounds(&st.buffer, caret);
    assert_eq!(start, 0);
    assert_eq!(end, text_len);

    let anchor = 0u32;
    let focus = u32::try_from("hello".len()).unwrap();
    let new_anchor = super::a11y::map_a11y_offset_to_buffer(&st.buffer, start, end, anchor);
    let new_focus = super::a11y::map_a11y_offset_to_buffer(&st.buffer, start, end, focus);
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

    let mut st = handle.state.borrow_mut();
    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
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

    let mut st = handle.state.borrow_mut();
    let (value, _selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "ABhello");
    assert_eq!(composition, Some((0, 2)));

    let text_len = st.buffer.len_bytes();
    let caret = st
        .buffer
        .clamp_to_char_boundary_left(st.selection.caret().min(text_len));
    let (start, end) = super::a11y::a11y_text_window_bounds(&st.buffer, caret);

    let mapped =
        super::a11y::map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 3);
    assert_eq!(
        mapped, 1,
        "display offset after preedit should map into base text"
    );

    let inside_preedit =
        super::a11y::map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 1);
    assert_eq!(
        inside_preedit, 0,
        "display offset inside preedit snaps to insertion caret"
    );

    let clamped_end = super::a11y::map_a11y_offset_to_buffer_with_preedit(
        &st.buffer,
        start,
        end,
        caret,
        2,
        u32::MAX,
    );
    assert_eq!(clamped_end, st.buffer.len_bytes());
}

#[test]
fn a11y_current_window_maps_buffer_offsets_and_roundtrips() {
    let handle = CodeEditorHandle::new("hello 😀 world");
    {
        let mut st = handle.state.borrow_mut();
        let caret = "hello 😀 ".len();
        st.selection = Selection {
            anchor: caret,
            focus: caret,
        };
        st.preedit = None;
        st.compose_inline_preedit = false;
    }

    let mut st = handle.state.borrow_mut();
    let (value, _selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(composition, None);
    assert_eq!(value.as_str(), "hello 😀 world");

    let byte = "hello".len();
    let a11y_offset = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, byte);
    let back = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, a11y_offset);
    assert_eq!(back, byte);
}

#[test]
fn a11y_current_window_mapping_accounts_for_preedit_injection() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
        st.preedit = Some(PreeditState {
            text: "AB".to_string(),
            cursor: Some((0, 0)),
        });
        st.compose_inline_preedit = false;
    }

    let mut st = handle.state.borrow_mut();
    let (value, _selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "heABllo");
    assert_eq!(composition, Some((2, 4)));

    let before = 1usize;
    let before_a11y = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, before);
    assert_eq!(before_a11y, 1);
    let before_back = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, before_a11y);
    assert_eq!(before_back, 1);

    let after = 3usize;
    let after_a11y = a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, after);
    assert_eq!(
        after_a11y,
        u32::try_from(after + "AB".len()).unwrap(),
        "bytes after caret include injected preedit segment"
    );
    let inside_preedit = a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, 3);
    assert_eq!(
        inside_preedit, 2,
        "offset inside preedit snaps to insertion caret"
    );
}

#[test]
fn a11y_window_includes_decorations_when_composed() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(4));
    handle.set_allow_decorations_under_inline_preedit(true);
    handle.set_compose_inline_preedit(true);

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..3,
            placeholder: Arc::<str>::from("…"),
        }],
    );
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 1,
            text: Arc::<str>::from("<inlay>"),
        }],
    );

    handle.set_caret(1);
    handle.set_preedit_debug("XY", Some((1, 1)));

    let mut st = handle.state.borrow_mut();
    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert!(value.contains("<inlay>"));
    assert!(value.contains("…"));
    assert!(value.contains("XY"));
    assert_eq!(composition, Some((1, 3)));
    assert_eq!(selection, Some((2, 2)));

    let (mapped_anchor, mapped_focus) = map_a11y_offsets_to_buffer_composed(&mut st, 1024, 2, 2);
    assert_eq!((mapped_anchor, mapped_focus), (1, 1));
}

#[test]
fn a11y_window_composed_selection_preserves_direction_for_preedit_cursor() {
    let handle = CodeEditorHandle::new("hello");
    handle.set_compose_inline_preedit(true);
    handle.set_caret("hello".len());
    handle.set_preedit_debug("yo", Some((2, 0)));

    let mut st = handle.state.borrow_mut();
    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "helloyo");
    assert_eq!(
        composition,
        Some(("hello".len() as u32, "helloyo".len() as u32))
    );
    assert_eq!(
        selection,
        Some(("helloyo".len() as u32, "hello".len() as u32)),
        "ADR 0071: preserve (anchor, focus) directionality"
    );
}

#[test]
fn a11y_window_composed_mapping_clamps_inside_utf8_scalars() {
    let handle = CodeEditorHandle::new("a😀b");
    handle.set_compose_inline_preedit(true);
    handle.set_caret(0);

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "a😀b");

    // Offset 2 lands inside the UTF-8 bytes of 😀 (starts at 1, ends at 5).
    let (mapped_anchor, mapped_focus) = map_a11y_offsets_to_buffer_composed(&mut st, 1024, 2, 2);
    assert_eq!((mapped_anchor, mapped_focus), (1, 1));
}

#[test]
fn a11y_window_composed_newline_offsets_map_to_line_end() {
    let handle = CodeEditorHandle::new("ab\ncd");
    handle.set_compose_inline_preedit(true);
    handle.set_caret(0);

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "ab\ncd");

    // In the composed display window, a newline is inserted between lines. Both the byte offset
    // at the end of the first line and the offset "inside the inserted newline" should map to
    // the same buffer boundary (the newline byte index).
    let newline_byte = "ab".len();
    let (at_end, _) = map_a11y_offsets_to_buffer_composed(
        &mut st,
        1024,
        u32::try_from(newline_byte).unwrap(),
        u32::try_from(newline_byte).unwrap(),
    );
    let (after_nl, _) = map_a11y_offsets_to_buffer_composed(
        &mut st,
        1024,
        u32::try_from(newline_byte + 1).unwrap(),
        u32::try_from(newline_byte + 1).unwrap(),
    );
    assert_eq!(at_end, newline_byte);
    assert_eq!(after_nl, newline_byte);
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
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            0,
        ),
        start
    );
    assert_eq!(
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            3,
        ),
        caret
    );
    assert_eq!(
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            4,
        ),
        caret
    );
    assert_eq!(
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            5,
        ),
        caret
    );
    assert_eq!(
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            6,
        ),
        caret + 1
    );
    assert_eq!(
        super::a11y::map_a11y_offset_to_buffer_with_preedit(
            &st.buffer,
            start,
            end,
            caret,
            preedit_len,
            7,
        ),
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
fn set_language_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("fn main() {\n    let x = 1;\n}\n");

    assert_eq!(
        handle.cache_stats().syntax_resets,
        0,
        "new handles should not reset syntax caches"
    );

    handle.set_language(Some(Arc::<str>::from("rust")));

    {
        let mut st = handle.state.borrow_mut();
        let _ = paint::cached_row_syntax_spans(&mut st, 0, 256);
        let _ = paint::cached_row_syntax_spans(&mut st, 1, 256);
        assert!(
            st.syntax_row_cache.contains_key(&0),
            "expected syntax cache entry for row 0"
        );
        assert!(
            st.syntax_row_cache.contains_key(&1),
            "expected syntax cache entry for row 1"
        );
    }

    let resets_before = handle.cache_stats().syntax_resets;

    // The UI layer may call set_language during render; that must be a no-op when the language is
    // unchanged to avoid per-frame cache resets and re-highlighting work.
    handle.set_language(Some(Arc::<str>::from("rust")));
    assert_eq!(
        handle.cache_stats().syntax_resets,
        resets_before,
        "idempotent set_language must not reset syntax caches"
    );

    {
        let st = handle.state.borrow();
        assert!(
            st.syntax_row_cache.contains_key(&0),
            "expected syntax cache entry for row 0 to survive idempotent set_language"
        );
        assert!(
            st.syntax_row_cache.contains_key(&1),
            "expected syntax cache entry for row 1 to survive idempotent set_language"
        );
    }
}

#[test]
fn set_line_folds_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("abcdef\n");

    let placeholder = Arc::<str>::from("…");
    let spans = vec![FoldSpan {
        range: 1..3,
        placeholder,
    }];

    handle.set_line_folds(0, spans.clone());

    let (folds_epoch_before, row_text_resets_before) = {
        let st = handle.state.borrow();
        (st.folds_epoch, st.cache_stats.row_text_resets)
    };

    handle.set_line_folds(0, spans);

    let st = handle.state.borrow();
    assert_eq!(
        st.folds_epoch, folds_epoch_before,
        "idempotent set_line_folds must not bump folds_epoch"
    );
    assert_eq!(
        st.cache_stats.row_text_resets, row_text_resets_before,
        "idempotent set_line_folds must not reset row text caches"
    );
}

#[test]
fn set_line_inlays_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("abcdef\n");

    let spans = vec![InlaySpan {
        byte: 2,
        text: Arc::<str>::from("<inlay>"),
    }];

    handle.set_line_inlays(0, spans.clone());

    let (inlays_epoch_before, row_text_resets_before) = {
        let st = handle.state.borrow();
        (st.inlays_epoch, st.cache_stats.row_text_resets)
    };

    handle.set_line_inlays(0, spans);

    let st = handle.state.borrow();
    assert_eq!(
        st.inlays_epoch, inlays_epoch_before,
        "idempotent set_line_inlays must not bump inlays_epoch"
    );
    assert_eq!(
        st.cache_stats.row_text_resets, row_text_resets_before,
        "idempotent set_line_inlays must not reset row text caches"
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
