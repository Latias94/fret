use std::collections::{HashMap, HashSet};

use fret_core::AppWindowId;

use super::inspect;

#[derive(Default)]
pub(super) struct InspectState {
    pub(super) focus_node_id: HashMap<AppWindowId, u64>,
    pub(super) focus_selector_json: HashMap<AppWindowId, String>,
    pub(super) focus_down_stack: HashMap<AppWindowId, Vec<u64>>,
    pub(super) pending_nav: HashMap<AppWindowId, inspect::InspectNavCommand>,
    pub(super) focus_summary_line: HashMap<AppWindowId, String>,
    pub(super) focus_path_line: HashMap<AppWindowId, String>,
    pub(super) locked_windows: HashSet<AppWindowId>,

    pub(super) help_open_windows: HashSet<AppWindowId>,
    pub(super) help_search_query: HashMap<AppWindowId, String>,
    pub(super) help_suppress_next_text_input: HashMap<AppWindowId, char>,
    pub(super) help_match_node_ids: HashMap<AppWindowId, Vec<u64>>,
    pub(super) help_selected_match_index: HashMap<AppWindowId, usize>,
    pub(super) help_scroll_offset: HashMap<AppWindowId, usize>,

    pub(super) tree_open_windows: HashSet<AppWindowId>,
    pub(super) tree_expanded_node_ids: HashMap<AppWindowId, HashSet<u64>>,
    pub(super) tree_flat_node_ids: HashMap<AppWindowId, Vec<u64>>,
    pub(super) tree_selected_index: HashMap<AppWindowId, usize>,
    pub(super) tree_selected_node_id: HashMap<AppWindowId, u64>,

    pub(super) pending_copy_selector_windows: HashSet<AppWindowId>,
    pub(super) pending_copy_details_windows: HashSet<AppWindowId>,
    pub(super) pending_copy_details_payload: HashMap<AppWindowId, String>,
    pub(super) toast: HashMap<AppWindowId, inspect::InspectToast>,
}

impl InspectState {
    pub(super) fn clear_for_window(&mut self, window: AppWindowId) {
        self.focus_node_id.remove(&window);
        self.focus_selector_json.remove(&window);
        self.focus_down_stack.remove(&window);
        self.pending_nav.remove(&window);
        self.focus_summary_line.remove(&window);
        self.focus_path_line.remove(&window);
        self.locked_windows.remove(&window);

        self.help_open_windows.remove(&window);
        self.help_search_query.remove(&window);
        self.help_suppress_next_text_input.remove(&window);
        self.help_match_node_ids.remove(&window);
        self.help_selected_match_index.remove(&window);
        self.help_scroll_offset.remove(&window);

        self.tree_open_windows.remove(&window);
        self.tree_expanded_node_ids.remove(&window);
        self.tree_flat_node_ids.remove(&window);
        self.tree_selected_index.remove(&window);
        self.tree_selected_node_id.remove(&window);

        self.pending_copy_selector_windows.remove(&window);
        self.pending_copy_details_windows.remove(&window);
        self.pending_copy_details_payload.remove(&window);
        self.toast.remove(&window);
    }

    pub(super) fn clear_all(&mut self) {
        self.focus_node_id.clear();
        self.focus_selector_json.clear();
        self.focus_down_stack.clear();
        self.pending_nav.clear();
        self.focus_summary_line.clear();
        self.focus_path_line.clear();
        self.locked_windows.clear();

        self.help_open_windows.clear();
        self.help_search_query.clear();
        self.help_suppress_next_text_input.clear();
        self.help_match_node_ids.clear();
        self.help_selected_match_index.clear();
        self.help_scroll_offset.clear();

        self.tree_open_windows.clear();
        self.tree_expanded_node_ids.clear();
        self.tree_flat_node_ids.clear();
        self.tree_selected_index.clear();
        self.tree_selected_node_id.clear();

        self.pending_copy_selector_windows.clear();
        self.pending_copy_details_windows.clear();
        self.pending_copy_details_payload.clear();
        self.toast.clear();
    }
}
