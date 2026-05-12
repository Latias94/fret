use super::*;
use crate::editor::{
    ime_cursor_area_for_text_input_region, platform_replace_and_mark_text_in_range_utf16,
    preedit_cursor_bytes_for_marked_range_utf16,
};

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
fn ime_cursor_area_matches_caret_rect_for_selection_under_preedit_and_wrap() {
    let handle = CodeEditorHandle::new("a馃榾bcd");
    handle.set_soft_wrap_cols(Some(2));
    handle.set_code_wrap_policy(Some(
        fret_code_editor_view::code_wrap_policy::CodeWrapPolicy::preset(
            fret_code_editor_view::code_wrap_policy::CodeWrapPreset::Balanced,
        ),
    ));

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: "a馃榾".len(),
            focus: "a馃榾".len(),
        };
        st.preedit = Some(PreeditState {
            text: "XY".to_string(),
            cursor: Some((1, 1)),
        });
    }

    let scroll = fret_ui::scroll::ScrollHandle::default();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(120.0)),
    );

    let mut st = handle.state.borrow_mut();
    let expected =
        caret_rect_for_selection(&mut st, Px(20.0), Px(10.0), bounds, &scroll).expect("caret rect");
    let actual =
        ime_cursor_area_for_text_input_region(&mut st, Px(20.0), Px(10.0), bounds, &scroll)
            .expect("ime cursor area");
    assert_eq!(actual, expected);
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
        None,
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
fn platform_replace_and_mark_empty_text_cancels_and_restores_selection() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 4,
        };
        st.preedit = None;
        st.preedit_replace_range = None;
        st.preedit_saved_selection = None;
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
        None,
    );
    assert!(did);
    assert_eq!(st.buffer.text_string(), "hello");
    assert!(st.preedit.is_some());
    assert!(st.preedit_replace_range.is_some());

    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "hXYo");
    assert_eq!(composition, Some((1, 3)));
    assert_eq!(selection, Some((1, 3)));

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(1, 3),
        "",
        Some(fret_runtime::Utf16Range::new(1, 1)),
        None,
    );
    assert!(did, "cancel must update state");
    assert_eq!(
        st.buffer.text_string(),
        "hello",
        "cancel must not mutate base buffer"
    );
    assert_eq!(st.preedit, None);
    assert!(st.preedit_replace_range.is_none());
    assert!(st.preedit_saved_selection.is_none());
    assert_eq!(
        st.selection,
        Selection {
            anchor: 1,
            focus: 4
        },
        "cancel must restore the pre-composition selection"
    );

    let (value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "hello");
    assert_eq!(composition, None);
    assert_eq!(selection, Some((1, 4)));
}

#[test]
fn platform_replace_and_mark_empty_text_cancels_and_restores_caret() {
    let handle = CodeEditorHandle::new("hello");
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 2,
            focus: 2,
        };
        st.preedit = None;
        st.preedit_replace_range = None;
        st.preedit_saved_selection = None;
    }

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);

    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(2, 2),
        "Z",
        Some(fret_runtime::Utf16Range::new(2, 3)),
        None,
    );
    assert!(did);
    assert!(st.preedit.is_some());

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(2, 3),
        "",
        Some(fret_runtime::Utf16Range::new(2, 2)),
        None,
    );
    assert!(did);
    assert_eq!(st.preedit, None);
    assert_eq!(
        st.selection,
        Selection {
            anchor: 2,
            focus: 2
        },
        "cancel must restore the pre-composition caret"
    );
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
        st.preedit_saved_selection = None;
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
        None,
    );
    assert!(did);
    assert_eq!(st.buffer.text_string(), "hXo");
    assert_eq!(st.preedit, None);
    assert_eq!(st.selection.caret(), 2);
}

#[test]
fn ime_delete_surrounding_preserves_selection_direction_and_preedit() {
    let handle = CodeEditorHandle::new("abcdef");
    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 4,
        focus: 2,
    };
    st.preedit = Some(PreeditState {
        text: "XY".to_string(),
        cursor: Some((0, 2)),
    });

    let did = input::apply_ime_delete_surrounding(&mut st, 1, 1);
    assert!(did.is_some());
    assert_eq!(st.buffer.text_string(), "acdf");
    assert_eq!(
        st.selection,
        Selection {
            anchor: 3,
            focus: 1,
        }
    );
    assert_eq!(
        st.preedit.as_ref().map(|p| p.text.as_str()),
        Some("XY"),
        "delete-surrounding should not clear the active preedit state"
    );
}

#[test]
fn platform_replace_and_mark_range_spanning_newline_is_clamped_to_anchor_line() {
    // Staging contract: selection-replacing composition ranges that span a newline in the
    // platform-facing composed window are clamped to the anchor logical line in the view model.
    // This keeps IME replacement deterministic while we stage multi-line composition support.
    let handle = CodeEditorHandle::new("ab\ncd");
    handle.set_compose_inline_preedit(true);
    handle.set_caret(0);

    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 0,
        focus: 0,
    };
    st.preedit = None;
    st.preedit_replace_range = None;
    st.preedit_saved_selection = None;

    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(value.as_str(), "ab\ncd");

    // Range 1..4 covers "b\nc" in UTF-16 (ASCII here), i.e. it spans the newline boundary.
    let did = platform_replace_and_mark_text_in_range_utf16(
        &mut st,
        1024,
        value.as_str(),
        fret_runtime::Utf16Range::new(1, 4),
        "X",
        Some(fret_runtime::Utf16Range::new(1, 2)),
        None,
    );
    assert!(did);
    assert_eq!(
        st.buffer.text_string(),
        "ab\ncd",
        "base buffer stays unchanged"
    );

    // Clamp to the end of the first line (newline byte index == 2), replacing only "b".
    assert_eq!(st.preedit_replace_range, Some(1..2));
    assert_eq!(
        st.preedit.as_ref().map(|p| p.text.as_str()),
        Some("X"),
        "composing text remains preedit-only"
    );
    assert_eq!(st.selection.caret(), 1);

    let (next_value, selection, composition) = a11y_composed_text_window(&mut st, 1024);
    assert_eq!(next_value.as_str(), "aX\ncd");
    assert_eq!(composition, Some((1, 2)));
    assert_eq!(selection, Some((1, 2)));
}
