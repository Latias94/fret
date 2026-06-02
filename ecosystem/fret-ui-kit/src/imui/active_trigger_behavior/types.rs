use fret_runtime::Model;

use super::super::interaction_runtime::{ImUiActiveItemState, ImUiLifecycleSessionState};

pub(in crate::imui) struct ActiveTriggerBehavior {
    pub(in crate::imui) active_item_model: Model<ImUiActiveItemState>,
    pub(in crate::imui) context_anchor_model: Model<Option<fret_core::Point>>,
    pub(in crate::imui) lifecycle_model: Model<ImUiLifecycleSessionState>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::imui) struct ActiveTriggerBehaviorOptions {
    pub(in crate::imui) primary_active: bool,
    pub(in crate::imui) request_focus_on_press: bool,
    pub(in crate::imui) clear_pointer_move: bool,
}

impl Default for ActiveTriggerBehaviorOptions {
    fn default() -> Self {
        Self {
            primary_active: true,
            request_focus_on_press: true,
            clear_pointer_move: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::imui) struct ActiveTriggerResponseInput {
    pub(in crate::imui) enabled: bool,
    pub(in crate::imui) clicked: bool,
    pub(in crate::imui) changed: bool,
    pub(in crate::imui) lifecycle_edited: bool,
}
