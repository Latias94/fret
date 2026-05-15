use super::*;
use crate::editor::input;

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
fn set_code_wrap_policy_clears_row_geom_cache_when_wrapped() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));

    {
        let mut st = handle.state.borrow_mut();
        st.row_geom_cache.insert(
            0,
            (
                geom::RowGeom {
                    row_range: 0.."left->".len(),
                    key: row_geom_key_for_tests(&Arc::from("left->")),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
                    preedit: None,
                },
                0,
            ),
        );
        assert!(!st.row_geom_cache.is_empty());
    }

    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Conservative,
        ),
    ));

    let st = handle.state.borrow();
    assert!(st.row_geom_cache.is_empty());
    assert_eq!(st.row_geom_cache_wrap_cols, st.display_wrap_cols);
}

#[test]
fn set_soft_wrap_cols_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("left->right->tail");
    handle.set_soft_wrap_cols(Some(6));

    let (display_map_epoch_before, row_scene_resets_before) = {
        let mut st = handle.state.borrow_mut();
        st.row_geom_cache.insert(
            0,
            (
                RowGeom {
                    row_range: 0.."left->".len(),
                    key: row_geom_key_for_tests(&Arc::from("left->")),
                    caret_stops: vec![(0, Px(0.0))],
                    fold_map: None,
                    caret_rect_top: None,
                    caret_rect_height: None,
                    has_preedit: false,
                    preedit: None,
                },
                0,
            ),
        );
        (st.display_map_epoch, st.cache_stats.row_scene_resets)
    };

    handle.set_soft_wrap_cols(Some(6));

    let st = handle.state.borrow();
    assert_eq!(
        st.display_map_epoch, display_map_epoch_before,
        "idempotent set_soft_wrap_cols must not rebuild the display map"
    );
    assert_eq!(
        st.cache_stats.row_scene_resets, row_scene_resets_before,
        "idempotent set_soft_wrap_cols must not reset row scene caches"
    );
    assert!(
        st.row_geom_cache.contains_key(&0),
        "idempotent set_soft_wrap_cols must preserve row geometry cache entries"
    );
}
