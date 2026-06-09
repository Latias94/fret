use std::sync::Arc;

use super::super::keyboard::InputTextPickerKeyboardPick;
use super::types::InputTextPickerPopupResult;

pub(super) struct PreparedTextPickerPopupResult {
    picked_index: Option<usize>,
    picked: Option<Arc<str>>,
}

impl PreparedTextPickerPopupResult {
    pub(super) fn from_pending(pending_pick: Option<&InputTextPickerKeyboardPick>) -> Self {
        Self {
            picked_index: pending_pick.map(|pick| pick.source_index),
            picked: pending_pick.map(|pick| pick.value.clone()),
        }
    }

    pub(super) fn merge_item_pick(&mut self, item_pick: InputTextPickerKeyboardPick) {
        self.picked_index = Some(item_pick.source_index);
        self.picked = Some(item_pick.value);
    }

    pub(super) fn finish(self, opened: bool) -> InputTextPickerPopupResult {
        InputTextPickerPopupResult {
            opened,
            picked_index: self.picked_index,
            picked: self.picked,
        }
    }
}
