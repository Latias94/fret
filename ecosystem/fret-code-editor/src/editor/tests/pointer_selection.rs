use super::*;
use crate::editor::{input, paint};

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
fn caret_for_pointer_snaps_inside_inlay_only_row_to_insertion_point_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(3));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 3,
            text: Arc::<str>::from("X"),
        }],
    );

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut st = handle.state.borrow_mut();

    // Ensure we have an "inlay-only" row: base slice empty, but composed text non-empty.
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 1, 64);
    assert_eq!(
        row_range,
        st.display_map.display_row_byte_range(&st.buffer, 1),
        "cached row text range must match the display map"
    );
    assert_eq!(row_text.as_ref(), "X");
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        1,
        (
            RowGeom {
                row_range: row_range.clone(),
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    // Pointer hit-test should clamp inside the insertion span to the insertion point in the base
    // buffer.
    let caret = caret_for_pointer(
        &mut st,
        1,
        bounds,
        fret_core::Point::new(Px(9.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret, 3);
}

#[test]
fn caret_for_pointer_snaps_inside_preedit_replacement_span_under_wrap_and_code_wrap_policy() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: "left->".len(),
        focus: "left->right".len(),
    };
    st.preedit = None;
    st.preedit_replace_range = None;
    st.preedit_saved_selection = None;

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), st.buffer.text_string());

    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new("left->".len() as u32, "left->right".len() as u32),
        "X",
        Some(fret_runtime::Utf16Range::new(
            "left->".len() as u32,
            ("left->".len() + 1) as u32,
        )),
        None,
    );
    assert!(did);
    assert!(st.preedit.is_some());
    assert!(st.preedit_replace_range.is_some());

    let caret = st.selection.caret();
    let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let (row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, row, 64);
    let preedit_range = preedit_range.expect("expected a visible preedit range on the caret row");
    assert!(row_text.as_ref().contains('X'));
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        row,
        (
            RowGeom {
                row_range: row_range.clone(),
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: true,
                preedit: None,
            },
            1,
        ),
    );

    let x = Px(preedit_range.start as f32 * 10.0);
    let caret = caret_for_pointer(
        &mut st,
        row,
        bounds,
        fret_core::Point::new(x, Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret, "left->".len());
}

#[test]
fn shift_click_extends_selection_to_inlay_insertion_point_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(3));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 3,
            text: Arc::<str>::from("X"),
        }],
    );

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    let row = 1;
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, row, 64);
    assert_eq!(row_text.as_ref(), "X");
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    let caret = caret_for_pointer(
        &mut st,
        row,
        bounds,
        fret_core::Point::new(Px(9.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(
        caret, 3,
        "clicking inside the inlay must map to its insertion point"
    );

    input::apply_pointer_down_selection(&mut st, row, caret, 1, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 0,
            focus: 3
        }
    );
}

#[test]
fn shift_drag_preserves_anchor_when_dragging_across_fold_placeholder_mapping() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..5,
            placeholder: Arc::<str>::from("."),
        }],
    );

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 2);
    st.selection = Selection {
        anchor: 5,
        focus: 5,
    };

    // Seed geometry for the first wrapped row ("a.") so caret_for_pointer goes through fold_map.
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 0, 64);
    assert_eq!(row_text.as_ref(), "a.");
    assert!(fold_map.is_some());
    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        0,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    let caret_on_placeholder = caret_for_pointer(
        &mut st,
        0,
        bounds,
        fret_core::Point::new(Px(10.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret_on_placeholder, 1);

    input::apply_pointer_down_selection(&mut st, 0, caret_on_placeholder, 1, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 5,
            focus: 1
        }
    );

    // Simulate dragging to the start of the next wrapped row ("f").
    let caret_next_row = st
        .display_map
        .display_point_to_byte(&st.buffer, DisplayPoint::new(1, 0));
    st.selection.focus = caret_next_row;
    assert_eq!(st.selection.anchor, 5);
}

#[test]
fn pointer_down_cancels_preedit_replacement_and_snaps_to_replace_start() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };
    st.preedit = None;
    st.preedit_replace_range = None;
    st.preedit_saved_selection = None;

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let start = "left->".len();
    let end = "left->right".len();

    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(start as u32, end as u32),
        "X",
        Some(fret_runtime::Utf16Range::new(
            start as u32,
            (start + 1) as u32,
        )),
        None,
    );
    assert!(did);
    assert!(st.preedit.is_some());
    assert!(st.preedit_replace_range.is_some());

    let caret = st.selection.caret();
    let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;
    let (_row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, row, 64);
    let preedit_range = preedit_range.expect("expected a visible preedit range on the caret row");
    assert!(row_text.as_ref().contains('X'));
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    let row_range = st.display_map.display_row_byte_range(&st.buffer, row);
    st.row_geom_cache.insert(
        row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: true,
                preedit: None,
            },
            1,
        ),
    );

    let caret_on_preedit = caret_for_pointer(
        &mut st,
        row,
        bounds,
        fret_core::Point::new(Px(preedit_range.start as f32 * 10.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret_on_preedit, start);

    input::apply_pointer_down_selection(&mut st, row, caret_on_preedit, 1, false);
    assert!(st.preedit.is_none());
    assert!(st.preedit_replace_range.is_none());
    assert!(st.preedit_saved_selection.is_none());
    assert_eq!(
        st.selection,
        Selection {
            anchor: start,
            focus: start
        }
    );
}

#[test]
fn triple_click_selects_logical_line_on_inlay_only_row_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef\nzzz");
    handle.set_soft_wrap_cols(Some(3));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 3,
            text: Arc::<str>::from("X"),
        }],
    );

    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 4);

    let (row_range, row_text, _fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 1, 64);
    assert_eq!(row_range, 3..3);
    assert_eq!(row_text.as_ref(), "X");

    input::apply_pointer_down_selection(&mut st, 1, 0, 3, false);

    let expected = st
        .buffer
        .line_byte_range_including_newline(0)
        .expect("line 0");
    assert_eq!(
        st.selection,
        Selection {
            anchor: expected.start,
            focus: expected.end,
        }
    );
}

#[test]
fn triple_click_cancels_preedit_replacement_and_selects_logical_line() {
    let handle = CodeEditorHandle::new("left->right->tail\nzzz");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };
    st.preedit = None;
    st.preedit_replace_range = None;
    st.preedit_saved_selection = None;

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let start = "left->".len();
    let end = "left->right".len();
    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(start as u32, end as u32),
        "X",
        Some(fret_runtime::Utf16Range::new(
            start as u32,
            (start + 1) as u32,
        )),
        None,
    );
    assert!(did);
    assert!(st.preedit.is_some());
    assert!(st.preedit_replace_range.is_some());

    let caret = st.selection.caret();
    let row = st.display_map.byte_to_display_point(&st.buffer, caret).row;

    input::apply_pointer_down_selection(&mut st, row, caret, 3, false);
    assert!(st.preedit.is_none());
    assert!(st.preedit_replace_range.is_none());
    assert!(st.preedit_saved_selection.is_none());

    let expected = st
        .buffer
        .line_byte_range_including_newline(0)
        .expect("line 0");
    assert_eq!(
        st.selection,
        Selection {
            anchor: expected.start,
            focus: expected.end,
        }
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
fn pointer_down_double_click_selects_word_on_inlay_only_row_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(3));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 3,
            text: Arc::<str>::from("X"),
        }],
    );

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    let mut st = handle.state.borrow_mut();

    let row = 1;
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, row, 64);
    assert_eq!(row_text.as_ref(), "X");
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: false,
                preedit: None,
            },
            1,
        ),
    );

    let caret = caret_for_pointer(
        &mut st,
        row,
        bounds,
        fret_core::Point::new(Px(9.0), Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret, 3);

    let (expect_start, expect_end) =
        select_word_range_in_buffer(&st.buffer, caret, st.active_text_boundary_mode);
    input::apply_pointer_down_selection(&mut st, row, caret, 2, false);
    assert_eq!(
        st.selection,
        Selection {
            anchor: expect_start,
            focus: expect_end,
        }
    );
}

#[test]
fn pointer_down_double_click_cancels_preedit_replacement_and_selects_word() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };
    st.preedit = None;
    st.preedit_replace_range = None;
    st.preedit_saved_selection = None;

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let start = "left->".len();
    let end = "left->right".len();
    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(start as u32, end as u32),
        "X",
        Some(fret_runtime::Utf16Range::new(
            start as u32,
            (start + 1) as u32,
        )),
        None,
    );
    assert!(did);
    assert!(st.preedit.is_some());
    assert!(st.preedit_replace_range.is_some());

    let row = st.display_map.byte_to_display_point(&st.buffer, start).row;
    let (row_range, row_text, fold_map, preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, row, 64);
    let preedit_range = preedit_range.expect("expected visible preedit on caret row");
    assert!(row_text.as_ref().contains('X'));
    assert!(fold_map.is_some());

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        row,
        (
            RowGeom {
                row_range,
                key: row_geom_key_for_tests(&row_text),
                caret_stops,
                fold_map,
                caret_rect_top: None,
                caret_rect_height: None,
                has_preedit: true,
                preedit: None,
            },
            1,
        ),
    );

    let x = Px(preedit_range.start as f32 * 10.0);
    let caret = caret_for_pointer(
        &mut st,
        row,
        bounds,
        fret_core::Point::new(x, Px(5.0)),
        Px(10.0),
    );
    assert_eq!(caret, start);

    let (expect_start, expect_end) =
        select_word_range_in_buffer(&st.buffer, caret, st.active_text_boundary_mode);
    input::apply_pointer_down_selection(&mut st, row, caret, 2, false);
    assert!(st.preedit.is_none());
    assert!(st.preedit_replace_range.is_none());
    assert!(st.preedit_saved_selection.is_none());
    assert_eq!(
        st.selection,
        Selection {
            anchor: expect_start,
            focus: expect_end,
        }
    );
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
