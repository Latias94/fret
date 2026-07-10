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

fn input_context_with_clipboard(read: bool, write: bool) -> fret_runtime::InputContext {
    let mut input_ctx = fret_runtime::InputContext::default();
    input_ctx.caps.clipboard.text.read = read;
    input_ctx.caps.clipboard.text.write = write;
    input_ctx
}

#[test]
fn command_availability_reports_selection_clipboard_and_navigation_state() {
    let handle = CodeEditorHandle::new("hello");
    let mut input_ctx = input_context_with_clipboard(true, true);

    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.select_all"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.copy"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "text.move_word_left"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "workspace.close_tab"),
            fret_ui::CommandAvailability::NotHandled
        );
    }

    handle.set_selection(Selection {
        anchor: 0,
        focus: 5,
    });
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.copy"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.cut"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.paste"),
            fret_ui::CommandAvailability::Available
        );
    }

    input_ctx.caps.clipboard.text.write = false;
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.copy"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.cut"),
            fret_ui::CommandAvailability::Blocked
        );
    }

    input_ctx.caps.clipboard.text.read = false;
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.paste"),
            fret_ui::CommandAvailability::Blocked
        );
    }
}

#[test]
fn command_availability_reports_undo_redo_and_read_only_state() {
    let handle = CodeEditorHandle::new("hello");
    let input_ctx = input_context_with_clipboard(true, true);

    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.undo"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.redo"),
            fret_ui::CommandAvailability::Blocked
        );
    }

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 5,
            focus: 5,
        };
        assert!(input::insert_text(&mut st, "!").is_some());
    }
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.undo"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.redo"),
            fret_ui::CommandAvailability::Blocked
        );
    }

    {
        let mut st = handle.state.borrow_mut();
        assert!(input::undo(&mut st));
    }
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.redo"),
            fret_ui::CommandAvailability::Available
        );
    }

    handle.set_selection(Selection {
        anchor: 0,
        focus: 5,
    });
    handle.set_interaction(CodeEditorInteractionOptions::read_only());
    {
        let st = handle.state.borrow();
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.copy"),
            fret_ui::CommandAvailability::Available
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.cut"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.paste"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "edit.undo"),
            fret_ui::CommandAvailability::Blocked
        );
        assert_eq!(
            input::command_availability(&st, &input_ctx, "text.select_word_right"),
            fret_ui::CommandAvailability::Available
        );
    }
}

#[test]
fn edit_undo_redo_commands_use_editor_local_history() {
    let handle = CodeEditorHandle::new("hello");
    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };

    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 5,
            focus: 5,
        };
        assert!(input::insert_text(&mut st, "!").is_some());
        assert_eq!(st.buffer.text_string(), "hello!");

        let undo_result = input::handle_command(&mut host, action_cx, &mut st, "edit.undo");
        assert!(undo_result.handled);
        assert!(undo_result.did);
        assert_eq!(st.buffer.text_string(), "hello");

        let redo_result = input::handle_command(&mut host, action_cx, &mut st, "edit.redo");
        assert!(redo_result.handled);
        assert!(redo_result.did);
        assert_eq!(st.buffer.text_string(), "hello!");
    }
}

#[test]
fn retired_text_edit_aliases_are_not_owned_by_code_editor() {
    let handle = CodeEditorHandle::new("hello");
    let input_ctx = input_context_with_clipboard(true, true);
    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let mut st = handle.state.borrow_mut();

    for command in [
        "text.undo",
        "text.redo",
        "text.select_all",
        "text.copy",
        "text.cut",
        "text.paste",
    ] {
        assert_eq!(
            input::command_availability(&st, &input_ctx, command),
            fret_ui::CommandAvailability::NotHandled,
            "retired command `{command}` must not remain on the editor surface"
        );
        let result = input::handle_command(&mut host, action_cx, &mut st, command);
        assert!(
            !result.handled,
            "retired command `{command}` was dispatched"
        );
        assert!(
            !result.did,
            "retired command `{command}` changed editor state"
        );
    }
}

#[test]
fn core_edit_registry_commands_reach_editor_availability_and_dispatch() {
    let mut registry = fret_runtime::CommandRegistry::default();
    fret_app::core_commands::register_text_edit_commands(&mut registry);

    let handle = CodeEditorHandle::new("hello");
    let input_ctx = input_context_with_clipboard(true, true);
    let mut host = TestHost::default();
    let action_cx = ActionCx {
        window: fret_core::AppWindowId::default(),
        target: fret_ui::GlobalElementId(0),
    };
    let mut st = handle.state.borrow_mut();
    st.selection = Selection {
        anchor: 5,
        focus: 5,
    };
    assert!(input::insert_text(&mut st, "!").is_some());

    for command in [
        "edit.undo",
        "edit.redo",
        "edit.select_all",
        "edit.copy",
        "edit.cut",
        "edit.paste",
    ] {
        assert!(
            registry
                .get(fret_runtime::CommandId::from(command))
                .is_some(),
            "core registry is missing `{command}`"
        );
        assert_ne!(
            input::command_availability(&st, &input_ctx, command),
            fret_ui::CommandAvailability::NotHandled,
            "editor availability does not own `{command}`"
        );
        assert!(
            input::handle_command(&mut host, action_cx, &mut st, command).handled,
            "editor dispatch does not own `{command}`"
        );
    }
}
