use super::*;
use crate::editor::input;

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
