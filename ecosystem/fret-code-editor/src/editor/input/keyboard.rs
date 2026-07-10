use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::editor) struct CommandDispatchResult {
    pub handled: bool,
    pub did: bool,
}

impl CommandDispatchResult {
    fn not_handled() -> Self {
        Self {
            handled: false,
            did: false,
        }
    }

    fn handled(did: bool) -> Self {
        Self { handled: true, did }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorCommand {
    Undo,
    Redo,
    SelectAll,
    Copy,
    Cut,
    Paste,
    MoveWordLeft,
    MoveWordRight,
    SelectWordLeft,
    SelectWordRight,
}

impl EditorCommand {
    fn parse(command: &str) -> Option<Self> {
        match command {
            "edit.undo" => Some(Self::Undo),
            "edit.redo" => Some(Self::Redo),
            "edit.select_all" => Some(Self::SelectAll),
            "edit.copy" => Some(Self::Copy),
            "edit.cut" => Some(Self::Cut),
            "edit.paste" => Some(Self::Paste),
            "text.move_word_left" => Some(Self::MoveWordLeft),
            "text.move_word_right" => Some(Self::MoveWordRight),
            "text.select_word_left" => Some(Self::SelectWordLeft),
            "text.select_word_right" => Some(Self::SelectWordRight),
            _ => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::editor) fn handle_key_down(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: ActionCx,
    state: &Rc<RefCell<CodeEditorState>>,
    row_h: Px,
    scroll_handle: &fret_ui::scroll::ScrollHandle,
    cell_w: &Cell<Px>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    let mut st = state.borrow_mut();
    if !st.interaction.enabled || !st.interaction.focusable || !st.interaction.selectable {
        return false;
    }
    let shift = modifiers.shift;
    let ctrl_or_meta = modifiers.ctrl || modifiers.meta;
    let word = modifiers.ctrl || modifiers.alt;
    let meta = modifiers.meta;

    if st.preedit.is_some() {
        let cancel_preedit = match key {
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Enter
            | KeyCode::Tab => true,
            KeyCode::PageUp | KeyCode::PageDown => !ctrl_or_meta,
            _ => false,
        };
        if cancel_preedit {
            st.set_preedit(None);
        }
    }

    // Let workspace keymaps handle global page navigation (e.g. tab switching).
    if ctrl_or_meta && matches!(key, KeyCode::PageUp | KeyCode::PageDown) {
        return false;
    }

    let cell_w_px = cell_w.get();

    match key {
        KeyCode::KeyA if ctrl_or_meta => {
            st.set_preedit(None);
            let end = st.buffer.len_bytes();
            st.selection = Selection {
                anchor: 0,
                focus: end,
            };
            st.caret_preferred_x = None;
            st.undo_group = None;
        }
        KeyCode::ArrowLeft => {
            if meta {
                move_caret_home_end(&mut st, true, false, shift);
            } else if word {
                move_word(&mut st, -1, shift);
            } else {
                move_caret_left(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowRight => {
            if meta {
                move_caret_home_end(&mut st, false, false, shift);
            } else if word {
                move_word(&mut st, 1, shift);
            } else {
                move_caret_right(&mut st, shift);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowUp => {
            if meta {
                move_caret_home_end(&mut st, true, true, shift);
            } else {
                move_caret_vertical(&mut st, -1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::ArrowDown => {
            if meta {
                move_caret_home_end(&mut st, false, true, shift);
            } else {
                move_caret_vertical(&mut st, 1, shift, cell_w_px);
            }
            st.undo_group = None;
        }
        KeyCode::Home => {
            move_caret_home_end(&mut st, true, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::End => {
            move_caret_home_end(&mut st, false, ctrl_or_meta, shift);
            st.undo_group = None;
        }
        KeyCode::PageUp => {
            move_caret_page(&mut st, -1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::PageDown => {
            move_caret_page(&mut st, 1, shift, row_h, scroll_handle, cell_w_px);
            st.undo_group = None;
        }
        KeyCode::Backspace => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            if word {
                delete_word_backward(&mut st);
            } else {
                delete_backward(&mut st);
            }
        }
        KeyCode::Delete => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            if word {
                delete_word_forward(&mut st);
            } else {
                delete_forward(&mut st);
            }
        }
        KeyCode::Enter => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            let _ = insert_text(&mut st, "\n");
        }
        KeyCode::Tab => {
            if !st.interaction.editable {
                st.set_preedit(None);
                st.undo_group = None;
                return true;
            }
            let _ = insert_text(&mut st, "\t");
        }
        KeyCode::KeyC if ctrl_or_meta => copy_selection(host, action_cx, &st),
        KeyCode::KeyV if ctrl_or_meta => {
            if st.interaction.editable {
                request_paste(host, action_cx);
            } else {
                st.set_preedit(None);
                st.undo_group = None;
            }
        }
        _ => return false,
    }

    scroll_caret_into_view(&st, row_h, scroll_handle);

    host.notify(action_cx);
    host.request_redraw(action_cx.window);
    true
}

pub(in crate::editor) fn handle_command(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: ActionCx,
    st: &mut CodeEditorState,
    command: &str,
) -> CommandDispatchResult {
    if !st.interaction.enabled || !st.interaction.focusable {
        return CommandDispatchResult::not_handled();
    }

    let Some(command) = EditorCommand::parse(command) else {
        return CommandDispatchResult::not_handled();
    };

    match command {
        EditorCommand::Undo => {
            if !st.interaction.editable {
                return CommandDispatchResult::handled(false);
            }
            CommandDispatchResult::handled(undo(st))
        }
        EditorCommand::Redo => {
            if !st.interaction.editable {
                return CommandDispatchResult::handled(false);
            }
            CommandDispatchResult::handled(redo(st))
        }
        EditorCommand::SelectAll => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            let end = st.buffer.len_bytes();
            st.selection = Selection {
                anchor: 0,
                focus: end,
            };
            st.set_preedit(None);
            st.undo_group = None;
            CommandDispatchResult::handled(true)
        }
        EditorCommand::Copy => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            copy_selection(host, action_cx, st);
            CommandDispatchResult::handled(true)
        }
        EditorCommand::Cut => {
            if !st.interaction.editable {
                return CommandDispatchResult::handled(false);
            }
            CommandDispatchResult::handled(cut_selection(host, action_cx, st))
        }
        EditorCommand::Paste => {
            if !st.interaction.editable {
                return CommandDispatchResult::handled(false);
            }
            request_paste(host, action_cx);
            CommandDispatchResult::handled(true)
        }
        EditorCommand::MoveWordLeft => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            st.set_preedit(None);
            CommandDispatchResult::handled(move_word(st, -1, false))
        }
        EditorCommand::MoveWordRight => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            st.set_preedit(None);
            CommandDispatchResult::handled(move_word(st, 1, false))
        }
        EditorCommand::SelectWordLeft => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            st.set_preedit(None);
            CommandDispatchResult::handled(move_word(st, -1, true))
        }
        EditorCommand::SelectWordRight => {
            if !st.interaction.selectable {
                return CommandDispatchResult::handled(false);
            }
            st.set_preedit(None);
            CommandDispatchResult::handled(move_word(st, 1, true))
        }
    }
}

pub(in crate::editor) fn command_availability(
    st: &CodeEditorState,
    input_ctx: &fret_runtime::InputContext,
    command: &str,
) -> fret_ui::CommandAvailability {
    if !st.interaction.enabled || !st.interaction.focusable {
        return fret_ui::CommandAvailability::NotHandled;
    }

    let has_selection = !st.selection.normalized().is_empty();
    let has_text = st.buffer.len_bytes() > 0;
    let clipboard_read = input_ctx.caps.clipboard.text.read;
    let clipboard_write = input_ctx.caps.clipboard.text.write;

    let Some(command) = EditorCommand::parse(command) else {
        return fret_ui::CommandAvailability::NotHandled;
    };

    match command {
        EditorCommand::Undo => {
            if st.interaction.editable && st.undo.can_undo() {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::Redo => {
            if st.interaction.editable && st.undo.can_redo() {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::SelectAll => {
            if st.interaction.selectable && has_text {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::Copy => {
            if st.interaction.selectable && has_selection && clipboard_write {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::Cut => {
            if st.interaction.editable && has_selection && clipboard_write {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::Paste => {
            if st.interaction.editable && clipboard_read {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
        EditorCommand::MoveWordLeft
        | EditorCommand::MoveWordRight
        | EditorCommand::SelectWordLeft
        | EditorCommand::SelectWordRight => {
            if st.interaction.selectable && has_text {
                fret_ui::CommandAvailability::Available
            } else {
                fret_ui::CommandAvailability::Blocked
            }
        }
    }
}
