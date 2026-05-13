use super::*;
use crate::editor::input;

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
