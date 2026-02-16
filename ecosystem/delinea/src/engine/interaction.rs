use crate::ids::AxisId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisInteractionLocks {
    pub pan_locked: bool,
    pub zoom_locked: bool,
}

impl AxisInteractionLocks {
    pub fn toggle_pan(&mut self) {
        self.pan_locked = !self.pan_locked;
    }

    pub fn toggle_zoom(&mut self) {
        self.zoom_locked = !self.zoom_locked;
    }
}

pub fn lock_entry(
    map: &mut std::collections::BTreeMap<AxisId, AxisInteractionLocks>,
    axis: AxisId,
) -> &mut AxisInteractionLocks {
    map.entry(axis).or_default()
}
