use super::*;

pub(super) use super::inspect_controller::{InspectNavCommand, InspectToast};

#[derive(Debug, Default, Clone)]
pub(super) struct InspectOverlayClipboardPayloads {
    pub(super) copy_details: Option<String>,
    pub(super) copy_selector: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct InspectOverlayModel {
    pub(super) pointer_pos: Option<Point>,
    pub(super) picked_node_id: Option<u64>,
    pub(super) focus_node_id: Option<u64>,

    pub(super) redact_text: bool,

    pub(super) pick_armed: bool,
    pub(super) pick_pending: bool,

    pub(super) inspect_enabled: bool,
    pub(super) help_open: bool,
    pub(super) consume_clicks: bool,
    pub(super) locked: bool,

    pub(super) help_search_query: Option<String>,
    pub(super) help_selected_match_index: Option<usize>,
    pub(super) help_scroll_offset: usize,
    pub(super) tree_open: bool,

    pub(super) toast_message: Option<String>,
    pub(super) best_selector_json: Option<String>,
    pub(super) focus_summary_line: Option<String>,
    pub(super) focus_path_line: Option<String>,
}

impl UiDiagnosticsService {
    pub(super) fn poll_inspector_controls(&mut self) {
        self.poll_pick_trigger();
        self.poll_inspect_trigger();
    }

    pub(super) fn take_inspect_overlay_clipboard_payloads(
        &mut self,
        window: AppWindowId,
    ) -> InspectOverlayClipboardPayloads {
        InspectOverlayClipboardPayloads {
            copy_details: self.inspector.take_pending_copy_details_payload(window),
            copy_selector: self.inspector.take_pending_copy_selector_payload(window),
        }
    }

    pub(super) fn inspect_overlay_model(&self, window: AppWindowId) -> InspectOverlayModel {
        InspectOverlayModel {
            pointer_pos: self.last_pointer_position(window),
            picked_node_id: self.last_picked_node_id(window),
            focus_node_id: self.inspector.focus_node_id(window),
            redact_text: self.redact_text(),
            pick_armed: self.pick_is_armed(),
            pick_pending: self.pick_is_pending(window),
            inspect_enabled: self.inspector.enabled,
            help_open: self.inspector.help_is_open(window),
            consume_clicks: self.inspector.consume_clicks,
            locked: self.inspector.is_locked(window),
            help_search_query: self
                .inspector
                .help_search_query(window)
                .map(|s| s.to_string()),
            help_selected_match_index: self.inspector.help_selected_match_index(window),
            help_scroll_offset: self.inspector.help_scroll_offset(window),
            tree_open: self.inspector.tree_is_open(window),
            toast_message: self.inspector.toast_message(window).map(|s| s.to_string()),
            best_selector_json: self
                .inspector
                .best_selector_json(window)
                .map(|s| s.to_string()),
            focus_summary_line: self
                .inspector
                .focus_summary_line(window)
                .map(|s| s.to_string()),
            focus_path_line: self
                .inspector
                .focus_path_line(window)
                .map(|s| s.to_string()),
        }
    }

    pub(super) fn ensure_inspect_tree_state_initialized(
        &mut self,
        window: AppWindowId,
        snapshot: &fret_core::SemanticsSnapshot,
        index: &super::selector::SemanticsIndex<'_>,
        focus_node_id: Option<u64>,
        picked_node_id: Option<u64>,
    ) {
        self.inspector.ensure_tree_state_initialized(
            window,
            snapshot,
            index,
            focus_node_id,
            picked_node_id,
        );
    }

    pub(super) fn inspect_tree_state_snapshot(
        &self,
        window: AppWindowId,
    ) -> (std::collections::HashSet<u64>, Option<u64>) {
        self.inspector.tree_state_snapshot(window)
    }

    pub(super) fn set_inspect_enabled(&mut self, enabled: bool, consume_clicks: bool) {
        self.inspector.set_enabled(enabled, consume_clicks);
    }

    pub fn inspect_is_enabled(&self) -> bool {
        self.inspector.enabled
    }

    pub fn inspect_consume_clicks(&self) -> bool {
        self.inspector.consume_clicks
    }

    pub fn inspect_is_locked(&self, window: AppWindowId) -> bool {
        self.inspector.is_locked(window)
    }

    pub fn inspect_help_is_open(&self, window: AppWindowId) -> bool {
        self.inspector.help_is_open(window)
    }

    pub fn inspect_help_search_query(&self, window: AppWindowId) -> Option<&str> {
        self.inspector.help_search_query(window)
    }

    #[cfg(test)]
    pub(super) fn inspect_help_scroll_offset(&self, window: AppWindowId) -> usize {
        self.inspector.help_scroll_offset(window)
    }

    pub(super) fn set_inspect_help_scroll_offset(&mut self, window: AppWindowId, offset: usize) {
        self.inspector.set_help_scroll_offset(window, offset);
    }

    pub(super) fn set_inspect_help_matches(&mut self, window: AppWindowId, matches: Vec<u64>) {
        self.inspector.set_help_matches(window, matches);
    }

    pub(super) fn set_inspect_tree_items(&mut self, window: AppWindowId, items: Vec<u64>) {
        self.inspector.set_tree_items(window, items);
    }

    pub fn inspect_focus_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.inspector.focus_node_id(window)
    }

    pub fn inspect_focus_summary_line(&self, window: AppWindowId) -> Option<&str> {
        self.inspector.focus_summary_line(window)
    }

    pub fn inspect_focus_path_line(&self, window: AppWindowId) -> Option<&str> {
        self.inspector.focus_path_line(window)
    }

    pub fn inspect_toast_message(&self, window: AppWindowId) -> Option<&str> {
        self.inspector.toast_message(window)
    }

    pub fn inspect_best_selector_json(&self, window: AppWindowId) -> Option<&str> {
        self.inspector.best_selector_json(window)
    }

    pub fn wants_inspection_active(&mut self, window: AppWindowId) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_inspector_controls();

        self.inspector.wants_inspection_active(window)
    }

    pub fn maybe_intercept_event_for_inspect_shortcuts(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.inspector
            .maybe_intercept_event_for_shortcuts(&self.cfg, app, window, event)
    }

    /// Returns `true` if the event was consumed by diagnostics picking.
    ///
    /// When a pick is armed, the next pointer down is intercepted (not dispatched to the UI tree)
    /// to avoid triggering app behavior while selecting a target (GPUI/Zed inspect style).
    pub fn maybe_intercept_event_for_picking(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        let Event::Pointer(PointerEvent::Down { position, .. }) = event else {
            return false;
        };

        let decision = self
            .inspector
            .on_pointer_down_for_picking(window, *position);
        if !decision.intercepted {
            return false;
        }
        if decision.request_redraw {
            app.request_redraw(window);
        }
        decision.consumed
    }

    pub fn update_inspect_hover(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        hovered_node_id: Option<u64>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }

        self.inspector.update_hover(
            &self.cfg,
            window,
            snapshot,
            hovered_node_id,
            element_runtime,
        );
    }

    pub fn apply_inspect_navigation(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }

        self.inspector
            .apply_navigation(&self.cfg, window, snapshot, element_runtime);
    }

    pub(super) fn update_inspect_focus_lines(
        &mut self,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.is_enabled() {
            return;
        }

        self.inspector
            .update_focus_lines(&self.cfg, window, snapshot, element_runtime);
    }
}
