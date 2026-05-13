use super::*;
use crate::editor::{input, paint};

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
fn move_caret_vertical_steps_through_code_wrap_policy_rows() {
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

    // Expect the first logical line to be split into three wrapped display rows.
    let rows: Vec<String> = (0..st.display_map.row_count())
        .map(|row| {
            let range = st.display_map.display_row_byte_range(&st.buffer, row);
            st.buffer.slice_to_string(range).unwrap_or_default()
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            "left->".to_string(),
            "right".to_string(),
            "->tail".to_string(),
            "zzz".to_string()
        ]
    );

    // Down should walk display rows (not logical lines) until it reaches the next line.
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->".len());
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->right".len());
    input::move_caret_vertical(&mut st, 1, false, Px(10.0));
    assert_eq!(st.selection.caret(), "left->right->tail\n".len());
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
fn shift_home_end_extends_selection_within_wrapped_display_rows() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };

    input::move_caret_home_end(&mut st, true, false, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 3,
            focus: 2,
        }
    );

    st.selection = Selection {
        anchor: 3,
        focus: 3,
    };
    input::move_caret_home_end(&mut st, false, false, true);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 3,
            focus: 4,
        }
    );
}

#[test]
fn shift_vertical_extends_selection_in_display_row_space_when_wrapped() {
    let handle = CodeEditorHandle::new("abcd\nef");
    handle.set_soft_wrap_cols(Some(2));

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    input::move_caret_vertical(&mut st, 1, true, Px(10.0));
    assert_eq!(
        st.selection,
        Selection {
            anchor: 0,
            focus: 2,
        }
    );

    input::move_caret_vertical(&mut st, 1, true, Px(10.0));
    assert_eq!(
        st.selection,
        Selection {
            anchor: 0,
            focus: 5,
        }
    );
}

#[test]
fn home_end_respects_code_wrap_policy_row_boundaries() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    let mut st = handle.state.borrow_mut();
    // Place the caret inside the "right" segment.
    st.selection = Selection {
        anchor: "left->r".len(),
        focus: "left->r".len(),
    };

    // Sanity: End should produce a caret in the same display-map space as the policy segmentation.
    input::move_caret_home_end(&mut st, false, false, false);

    let mut rows: Vec<String> = Vec::new();
    for row in 0..st.display_map.row_count() {
        let range = st.display_map.display_row_byte_range(&st.buffer, row);
        rows.push(st.buffer.slice_to_string(range).unwrap_or_default());
    }
    assert_eq!(rows.join(""), st.buffer.text_string());

    for w in rows.windows(2) {
        assert!(
            !(w[0].ends_with('-') && w[1].starts_with('>')),
            "expected `->` to not be split across a wrap boundary: rows={rows:?}"
        );
    }
}

#[test]
fn move_caret_vertical_uses_row_fold_map_for_inlay_insertions_under_soft_wrap() {
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

    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 3);
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };

    // Prefer a stable target x so the vertical move hit-tests a specific caret stop.
    st.caret_preferred_x = Some(Px(10.0));

    // Seed geometry for the next row ("X") so the move path uses the geometry cache.
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 1, 64);
    assert_eq!(
        row_range,
        st.display_map.display_row_byte_range(&st.buffer, 1),
        "cached row text range must match the display map"
    );
    assert_eq!(row_text.as_ref(), "X");
    assert!(
        fold_map.is_some(),
        "expected an insertion span for the inlay"
    );

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

    input::move_caret_vertical(&mut st, 1, false, Px(8.0));
    assert_eq!(
        st.selection.caret(),
        3,
        "expected caret to snap to the inlay insertion point (before 'd')"
    );
}

#[test]
fn move_caret_vertical_uses_row_fold_map_for_fold_placeholders_under_soft_wrap() {
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

    let mut st = handle.state.borrow_mut();
    assert_eq!(st.display_map.row_count(), 2);

    // Seed geometry for the first wrapped row ("a.") so the vertical move path uses the cached
    // caret stop mapping (which must respect fold placeholders).
    let (row_range, row_text, fold_map, _preedit_range, _spans) =
        paint::cached_row_text_with_range(&mut st, 0, 64);
    assert_eq!(
        row_range,
        st.display_map.display_row_byte_range(&st.buffer, 0),
        "cached row text range must match the display map"
    );
    assert_eq!(row_text.as_ref(), "a.");
    assert!(fold_map.is_some());

    // Start on the second wrapped row and move up. The requested x targets the placeholder in the
    // first wrapped row, which must snap to the fold start in the base buffer.
    st.selection = Selection {
        anchor: 5,
        focus: 5,
    };
    st.caret_preferred_x = Some(Px(10.0));

    let caret_stops: Vec<(usize, Px)> = (0..=row_text.len())
        .map(|idx| (idx, Px(idx as f32 * 10.0)))
        .collect();
    st.row_geom_cache.insert(
        0,
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

    input::move_caret_vertical(&mut st, -1, false, Px(8.0));
    assert_eq!(
        st.selection.caret(),
        1,
        "expected caret to snap to the fold start when targeting the placeholder"
    );
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
