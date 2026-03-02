use std::collections::HashMap;

use fret_core::AppWindowId;

use super::inspect_state::InspectState;

#[derive(Default)]
pub(super) struct InspectController {
    pub(super) enabled: bool,
    pub(super) consume_clicks: bool,

    pub(super) last_picked_node_id: HashMap<AppWindowId, u64>,
    pub(super) last_picked_selector_json: HashMap<AppWindowId, String>,
    pub(super) last_hovered_node_id: HashMap<AppWindowId, u64>,
    pub(super) last_hovered_selector_json: HashMap<AppWindowId, String>,

    pub(super) state: InspectState,
    pub(super) pick_overlay_grace_frames: HashMap<AppWindowId, u32>,

    pub(super) pick_armed_run_id: Option<u64>,
    pub(super) pending_pick: Option<super::PendingPick>,
}

impl InspectController {
    pub(super) fn clear_for_window(&mut self, window: AppWindowId) {
        self.last_picked_node_id.remove(&window);
        self.last_picked_selector_json.remove(&window);
        self.last_hovered_node_id.remove(&window);
        self.last_hovered_selector_json.remove(&window);
        self.pick_overlay_grace_frames.remove(&window);
        self.state.clear_for_window(window);

        if self
            .pending_pick
            .as_ref()
            .is_some_and(|p| p.window == window)
        {
            self.pending_pick = None;
        }
    }
}
