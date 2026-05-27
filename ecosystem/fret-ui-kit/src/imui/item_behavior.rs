//! Private shared item behavior for immediate-mode pressable controls.

use fret_core::{Modifiers, Point};
use fret_runtime::Model;

use super::interaction_runtime::{
    ImUiActiveItemState, ImUiLifecycleSessionState, LongPressSignalState,
};

mod install;
mod response;

pub(super) use install::{
    install_pressable_item_behavior, install_pressable_item_behavior_with_options,
};
pub(super) use response::populate_pressable_item_response;

pub(super) struct PressableItemBehavior {
    pub(super) active_item_model: Model<ImUiActiveItemState>,
    pub(super) context_anchor_model: Model<Option<Point>>,
    pub(super) long_press_signal_model: Model<LongPressSignalState>,
    pub(super) lifecycle_model: Model<ImUiLifecycleSessionState>,
    pointer_click_modifiers_model: Option<Model<Modifiers>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PressableItemBehaviorOptions {
    pub(super) report_pointer_click: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PressableItemResponseInput {
    pub(super) enabled: bool,
    pub(super) clicked: bool,
    pub(super) changed: bool,
    pub(super) lifecycle_edited: bool,
}
