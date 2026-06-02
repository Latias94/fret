use fret_core::KeyCode;
use fret_runtime::CommandId;
use fret_ui::action::KeyDownCx;

mod snapshot;

pub(in crate::imui::text_controls) use snapshot::InputTextPolicyCommands;

pub(in crate::imui::text_controls) fn resolve_input_text_policy_command(
    commands: &InputTextPolicyCommands,
    down: KeyDownCx,
) -> Option<CommandId> {
    if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
        return None;
    }

    if down.modifiers.ctrl {
        match down.key {
            KeyCode::KeyZ
                if !down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.undo.clone()
            }
            KeyCode::KeyY
                if !down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.redo.clone()
            }
            KeyCode::KeyZ
                if down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.redo.clone()
            }
            _ => None,
        }
    } else if !down.modifiers.shift {
        match down.key {
            KeyCode::Tab if !down.repeat || commands.completion_repeat => {
                commands.completion.clone()
            }
            KeyCode::ArrowUp if !down.repeat || commands.history_repeat => {
                commands.history_previous.clone()
            }
            KeyCode::ArrowDown if !down.repeat || commands.history_repeat => {
                commands.history_next.clone()
            }
            _ => None,
        }
    } else {
        None
    }
}
