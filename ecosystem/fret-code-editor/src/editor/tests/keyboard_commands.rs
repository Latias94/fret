use super::*;
use crate::editor::input;

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
