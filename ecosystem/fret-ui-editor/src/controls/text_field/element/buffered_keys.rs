use std::sync::{Arc, Mutex};

use fret_core::KeyCode;
use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::OnTextFieldOutcome;
use super::super::buffered::{self, BufferedTextFieldState};

pub(super) enum TextFieldBufferedKeyMode {
    SingleLine { submit_command: Option<CommandId> },
    Multiline,
}

pub(super) struct TextFieldBufferedKeyHandlerArgs {
    pub(super) entry_id: GlobalElementId,
    pub(super) mode: TextFieldBufferedKeyMode,
    pub(super) model: Model<String>,
    pub(super) draft: Model<String>,
    pub(super) buffered_state: Arc<Mutex<BufferedTextFieldState>>,
    pub(super) on_outcome: Option<OnTextFieldOutcome>,
}

pub(super) fn install_buffered_text_field_key_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: TextFieldBufferedKeyHandlerArgs,
) {
    let TextFieldBufferedKeyHandlerArgs {
        entry_id,
        mode,
        model,
        draft,
        buffered_state,
        on_outcome,
    } = args;

    cx.key_add_on_key_down_capture_for(
        entry_id,
        Arc::new(
            move |host: &mut dyn UiFocusActionHost, action_cx: ActionCx, down| {
                if down.ime_composing || down.repeat {
                    return false;
                }

                match down.key {
                    KeyCode::Enter | KeyCode::NumpadEnter => match &mode {
                        TextFieldBufferedKeyMode::SingleLine { submit_command } => {
                            buffered::commit_buffered_text_field(
                                host,
                                action_cx,
                                &model,
                                &draft,
                                &buffered_state,
                                on_outcome.as_ref(),
                                submit_command.as_ref(),
                            )
                        }
                        TextFieldBufferedKeyMode::Multiline
                            if buffered::is_multiline_buffered_commit_shortcut(down) =>
                        {
                            buffered::commit_buffered_text_field(
                                host,
                                action_cx,
                                &model,
                                &draft,
                                &buffered_state,
                                on_outcome.as_ref(),
                                None,
                            )
                        }
                        TextFieldBufferedKeyMode::Multiline => false,
                    },
                    KeyCode::Escape => buffered::cancel_buffered_text_field(
                        host,
                        action_cx,
                        &model,
                        &draft,
                        &buffered_state,
                        on_outcome.as_ref(),
                    ),
                    _ => false,
                }
            },
        ),
    );
}
