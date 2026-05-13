use super::*;
use crate::editor::a11y as editor_a11y;
use crate::editor::platform_replace_and_mark_text_in_range_utf16;

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
        a11y_composed_text_window(&mut st, text_cache_max_entries)
    };

    assert_eq!(base_value.as_str(), "hello world");
    assert_eq!(base_selection, Some((6, 11)));
    assert_eq!(base_composition, None);

    let did = {
        let mut st = handle.state.borrow_mut();
        platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            text_cache_max_entries,
            base_value.as_str(),
            fret_runtime::Utf16Range::new(6, 11),
            "X",
            Some(fret_runtime::Utf16Range::new(6, 7)),
            None,
        )
    };
    assert!(did, "expected replace-and-mark to update editor state");

    let (value, selection, composition) = {
        let mut st = handle.state.borrow_mut();
        a11y_composed_text_window(&mut st, text_cache_max_entries)
    };
    assert_eq!(value.as_str(), "hello X");
    assert_eq!(selection, Some((6, 7)));
    assert_eq!(composition, Some((6, 7)));

    {
        let mut st = handle.state.borrow_mut();
        let start = editor_a11y::map_a11y_offset_to_buffer_in_current_window(
            &mut st,
            text_cache_max_entries,
            6,
        );
        let after_preedit = editor_a11y::map_a11y_offset_to_buffer_in_current_window(
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
fn a11y_source_does_not_materialize_whole_buffer_string() {
    // Regression guard: the editor a11y composed-window path should never call
    // `TextBuffer::text_string()`. Materializing the entire rope would scale with document size
    // and defeat windowed platform text input.
    const SRC: &str = include_str!("../a11y/mod.rs");
    assert!(
        !SRC.contains(".text_string("),
        "a11y/mod.rs must not call TextBuffer::text_string()"
    );
}

#[test]
fn a11y_composed_window_is_bounded_for_large_documents() {
    // Regression guard: the composed a11y window must remain bounded even for large documents.
    // This defends against accidental full-document materialization during platform queries.
    let mut text = String::with_capacity(300_000);
    for _ in 0..50_000 {
        text.push_str("abcd\n");
    }
    assert!(text.len() > 200_000);

    let handle = CodeEditorHandle::new(text.clone());
    handle.debug_set_compose_inline_preedit(true);
    handle.set_caret(text.len() / 2);

    let mut st = handle.state.borrow_mut();
    let (value, _selection, _composition) = a11y_composed_text_window(&mut st, 8);
    assert!(
        value.len() < text.len() / 10,
        "expected composed a11y value to be a bounded window, not a full-document snapshot"
    );
    assert!(
        value.len() < 20_000,
        "expected composed a11y value to remain bounded by window budgets"
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
    let (start, end) = editor_a11y::a11y_text_window_bounds(&st.buffer, caret);
    assert_eq!(start, 0);
    assert_eq!(end, text_len);

    let anchor = 0u32;
    let focus = u32::try_from("hello".len()).unwrap();
    let new_anchor = editor_a11y::map_a11y_offset_to_buffer(&st.buffer, start, end, anchor);
    let new_focus = editor_a11y::map_a11y_offset_to_buffer(&st.buffer, start, end, focus);
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
    let (start, end) = editor_a11y::a11y_text_window_bounds(&st.buffer, caret);

    let mapped =
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 3);
    assert_eq!(
        mapped, 1,
        "display offset after preedit should map into base text"
    );

    let inside_preedit =
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(&st.buffer, start, end, caret, 2, 1);
    assert_eq!(
        inside_preedit, 0,
        "display offset inside preedit snaps to insertion caret"
    );

    let clamped_end = editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
    let a11y_offset = editor_a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, byte);
    let back = editor_a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, a11y_offset);
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
    let before_a11y = editor_a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, before);
    assert_eq!(before_a11y, 1);
    let before_back =
        editor_a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, before_a11y);
    assert_eq!(before_back, 1);

    let after = 3usize;
    let after_a11y = editor_a11y::map_buffer_offset_to_a11y_offset(&mut st, 1024, after);
    assert_eq!(
        after_a11y,
        u32::try_from(after + "AB".len()).unwrap(),
        "bytes after caret include injected preedit segment"
    );
    let inside_preedit = editor_a11y::map_a11y_offset_to_buffer_in_current_window(&mut st, 1024, 3);
    assert_eq!(
        inside_preedit, 2,
        "offset inside preedit snaps to insertion caret"
    );
}

#[test]
fn a11y_window_includes_decorations_when_composed() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(4));
    handle.debug_set_allow_decorations_under_inline_preedit(true);
    handle.debug_set_compose_inline_preedit(true);

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
    handle.debug_set_compose_inline_preedit(true);
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
    handle.debug_set_compose_inline_preedit(true);
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
    handle.debug_set_compose_inline_preedit(true);
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
        editor_a11y::map_a11y_offset_to_buffer_with_preedit(
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
