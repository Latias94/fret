use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnKeyDown};
use fret_ui_kit::primitives::combobox as kit_combobox;

#[cfg(test)]
mod tests;

#[allow(clippy::arc_with_non_send_sync)]
pub(super) fn enum_select_trigger_open_keys(
    enabled: bool,
    open: Model<bool>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
) -> OnKeyDown {
    Arc::new(move |host, action_cx: ActionCx, down| {
        if !enabled {
            return false;
        }

        match enum_select_trigger_key_intent(down.key) {
            EnumSelectTriggerKeyIntent::Open => {
                let _ = host.models_mut().update(&open_change_reason, |v| {
                    *v = Some(kit_combobox::ComboboxOpenChangeReason::TriggerPress);
                });
                let _ = host.models_mut().update(&open, |v| *v = true);
                host.request_redraw(action_cx.window);
                true
            }
            EnumSelectTriggerKeyIntent::Close => {
                let was_open = host.models_mut().get_copied(&open).unwrap_or(false);
                if was_open {
                    let _ = host.models_mut().update(&open_change_reason, |v| {
                        *v = Some(kit_combobox::ComboboxOpenChangeReason::EscapeKey);
                    });
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    true
                } else {
                    false
                }
            }
            EnumSelectTriggerKeyIntent::Ignore => false,
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnumSelectTriggerKeyIntent {
    Open,
    Close,
    Ignore,
}

fn enum_select_trigger_key_intent(key: KeyCode) -> EnumSelectTriggerKeyIntent {
    match key {
        KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space | KeyCode::ArrowDown => {
            EnumSelectTriggerKeyIntent::Open
        }
        KeyCode::Escape => EnumSelectTriggerKeyIntent::Close,
        _ => EnumSelectTriggerKeyIntent::Ignore,
    }
}
