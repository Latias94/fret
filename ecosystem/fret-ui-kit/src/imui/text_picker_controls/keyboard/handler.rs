use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_runtime::Model;
use fret_ui::UiHost;

use super::InputTextPickerKeyboardState;

mod navigation;
mod pick;

pub(in crate::imui::text_picker_controls) fn install_picker_keyboard_handler<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    input_id: fret_ui::GlobalElementId,
    model: Model<String>,
    popup_open: Model<bool>,
    state: Model<InputTextPickerKeyboardState>,
    visible_candidates: Vec<(usize, Arc<str>)>,
    keyboard_repeat: bool,
) {
    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing
                || down.modifiers != Modifiers::default()
                || (down.repeat && !keyboard_repeat)
            {
                return false;
            }

            if visible_candidates.is_empty() {
                return false;
            }

            match down.key {
                KeyCode::ArrowDown | KeyCode::ArrowUp => navigation::move_picker_highlight(
                    host,
                    action_cx,
                    down.key == KeyCode::ArrowDown,
                    &state,
                    &visible_candidates,
                ),
                KeyCode::Enter | KeyCode::NumpadEnter => pick::commit_picker_highlight(
                    host,
                    action_cx,
                    &model,
                    &popup_open,
                    &state,
                    &visible_candidates,
                ),
                _ => false,
            }
        }),
    );
}
