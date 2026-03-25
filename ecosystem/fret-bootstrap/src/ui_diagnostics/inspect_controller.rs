use std::collections::{HashMap, HashSet};

use fret_app::{App, Effect};
use fret_core::AppWindowId;
use fret_core::{Event, KeyCode, Point};
use fret_diag_protocol::UiInspectConfigV1;
use fret_ui::elements::ElementRuntime;
use slotmap::Key as _;

use super::UiDiagnosticsConfig;
use super::inspect_state::InspectState;

#[derive(Debug, Clone)]
pub(super) struct PendingPick {
    pub(super) run_id: u64,
    pub(super) window: AppWindowId,
    pub(super) position: Point,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PickInterceptDecision {
    pub(super) intercepted: bool,
    pub(super) consumed: bool,
    pub(super) request_redraw: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScriptInspectTreeValidationFailure {
    NodeNotInTree,
    SelectedMarkerMissing,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum InspectNavCommand {
    Up,
    Down,
    Focus,
    SelectNode(u64),
}

#[derive(Debug, Clone)]
pub(super) struct InspectToast {
    pub(super) message: String,
    pub(super) remaining_frames: u32,
}

#[derive(Default)]
pub(super) struct InspectController {
    pub(super) enabled: bool,
    pub(super) consume_clicks: bool,

    pub(super) last_pick_run_id: u64,
    pub(super) last_picked_node_id: HashMap<AppWindowId, u64>,
    pub(super) last_picked_selector_json: HashMap<AppWindowId, String>,
    pub(super) last_hovered_node_id: HashMap<AppWindowId, u64>,
    pub(super) last_hovered_selector_json: HashMap<AppWindowId, String>,

    pub(super) state: InspectState,
    pub(super) pick_overlay_grace_frames: HashMap<AppWindowId, u32>,

    pub(super) pick_armed_run_id: Option<u64>,
    pub(super) pending_pick: Option<PendingPick>,
}

impl InspectController {
    pub(super) fn next_pick_run_id(&mut self) -> u64 {
        let mut id = super::unix_ms_now();
        if id <= self.last_pick_run_id {
            id = self.last_pick_run_id.saturating_add(1);
        }
        self.last_pick_run_id = id;
        id
    }

    pub(super) fn arm_pick(&mut self, run_id: u64) {
        self.pending_pick = None;
        self.pick_armed_run_id = Some(run_id);
    }

    pub(super) fn on_pointer_down_for_picking(
        &mut self,
        window: AppWindowId,
        position: Point,
    ) -> PickInterceptDecision {
        if self.try_consume_armed_pick(window, position) {
            return PickInterceptDecision {
                intercepted: true,
                consumed: true,
                request_redraw: true,
            };
        }

        if !self.enabled {
            return PickInterceptDecision {
                intercepted: false,
                consumed: false,
                request_redraw: false,
            };
        }

        let run_id = self.next_pick_run_id();
        let consumed = self.start_pending_pick(window, run_id, position);
        PickInterceptDecision {
            intercepted: true,
            consumed,
            request_redraw: true,
        }
    }

    pub(super) fn set_enabled(&mut self, enabled: bool, consume_clicks: bool) {
        self.enabled = enabled;
        self.consume_clicks = consume_clicks;
        if !enabled {
            self.pick_overlay_grace_frames.clear();
            self.state.clear_all();
            self.last_hovered_node_id.clear();
            self.last_hovered_selector_json.clear();
            self.last_picked_node_id.clear();
            self.last_picked_selector_json.clear();
        }
    }

    pub(super) fn take_pending_pick_for_window(
        &mut self,
        window: AppWindowId,
    ) -> Option<PendingPick> {
        if self
            .pending_pick
            .as_ref()
            .is_some_and(|pending| pending.window == window)
        {
            return self.pending_pick.take();
        }
        None
    }

    pub(super) fn on_pick_success(
        &mut self,
        window: AppWindowId,
        node_id: u64,
        selector_json: Option<String>,
    ) {
        self.last_picked_node_id.insert(window, node_id);
        if let Some(json) = selector_json {
            self.last_picked_selector_json.insert(window, json.clone());
            self.state.focus_node_id.insert(window, node_id);
            self.state.focus_selector_json.insert(window, json);
            self.state.focus_down_stack.insert(window, Vec::new());
        }
        self.pick_overlay_grace_frames.insert(window, 10);
    }

    pub(super) fn script_lock_window(&mut self, window: AppWindowId) {
        self.state.focus_down_stack.insert(window, Vec::new());
        self.state.locked_windows.insert(window);
    }

    pub(super) fn script_set_focus_and_best_selector_json(
        &mut self,
        window: AppWindowId,
        node_id: u64,
        selector_json: String,
    ) {
        self.state.focus_node_id.insert(window, node_id);
        self.state
            .focus_selector_json
            .insert(window, selector_json.clone());
        self.last_picked_node_id.insert(window, node_id);
        self.last_picked_selector_json.insert(window, selector_json);
    }

    pub(super) fn script_set_toast(&mut self, window: AppWindowId, message: String) {
        self.state.toast.insert(
            window,
            InspectToast {
                message,
                remaining_frames: 90,
            },
        );
    }

    pub(super) fn script_finish_lock_and_copy_selector(
        &mut self,
        window: AppWindowId,
        token: fret_core::ClipboardToken,
        node_id: u64,
        selector_json: String,
        toast_message: String,
    ) -> Effect {
        self.script_set_focus_and_best_selector_json(window, node_id, selector_json.clone());
        self.script_set_toast(window, toast_message);
        Effect::ClipboardWriteText {
            window,
            token,
            text: selector_json,
        }
    }

    pub(super) fn script_prepare_help_search(&mut self, window: AppWindowId, query: String) {
        self.state.help_open_windows.insert(window);
        self.state.help_search_query.insert(window, query);
        self.state.help_selected_match_index.insert(window, 0);
    }

    pub(super) fn script_prepare_help_tree_search(&mut self, window: AppWindowId, query: String) {
        self.script_prepare_help_search(window, query);
        self.state.tree_open_windows.insert(window);
        self.set_help_scroll_offset(window, usize::MAX / 4);
    }

    pub(super) fn script_select_tree_node_and_expand_ancestors(
        &mut self,
        window: AppWindowId,
        node_id: u64,
        index: &super::selector::SemanticsIndex<'_>,
    ) {
        self.state.tree_selected_node_id.insert(window, node_id);

        let expanded = self.state.tree_expanded_node_ids.entry(window).or_default();
        let mut cur = Some(node_id);
        for _ in 0..64 {
            let Some(id) = cur else {
                break;
            };
            expanded.insert(id);
            cur = index
                .by_id
                .get(&id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
    }

    pub(super) fn take_pending_copy_details_payload(
        &mut self,
        window: AppWindowId,
    ) -> Option<String> {
        self.state.pending_copy_details_payload.remove(&window)
    }

    pub(super) fn take_pending_copy_selector_payload(
        &mut self,
        window: AppWindowId,
    ) -> Option<String> {
        if !self.state.pending_copy_selector_windows.contains(&window) {
            return None;
        }

        let payload = self.best_selector_json(window).map(|s| s.to_string());
        if payload.is_some() {
            self.state.pending_copy_selector_windows.remove(&window);
        }
        payload
    }

    pub(super) fn ensure_tree_state_initialized(
        &mut self,
        window: AppWindowId,
        snapshot: &fret_core::SemanticsSnapshot,
        index: &super::selector::SemanticsIndex<'_>,
        focus_node_id: Option<u64>,
        picked_node_id: Option<u64>,
    ) {
        if !self.tree_is_open(window) {
            return;
        }

        let expanded = self.state.tree_expanded_node_ids.entry(window).or_default();
        let selected = self.state.tree_selected_node_id.get(&window).copied();
        let anchor = selected.or(focus_node_id).or(picked_node_id);

        if selected.is_none() {
            if let Some(anchor) = anchor {
                self.state.tree_selected_node_id.insert(window, anchor);
            }
        }

        if expanded.is_empty() {
            if let Some(anchor) = anchor {
                let mut cur = Some(anchor);
                for _ in 0..64 {
                    let Some(id) = cur else {
                        break;
                    };
                    expanded.insert(id);
                    cur = index
                        .by_id
                        .get(&id)
                        .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
                }
            } else {
                for r in snapshot.roots.iter().filter(|r| r.visible) {
                    expanded.insert(r.root.data().as_ffi());
                }
            }
        }
    }

    pub(super) fn tree_state_snapshot(&self, window: AppWindowId) -> (HashSet<u64>, Option<u64>) {
        (
            self.state
                .tree_expanded_node_ids
                .get(&window)
                .cloned()
                .unwrap_or_default(),
            self.state.tree_selected_node_id.get(&window).copied(),
        )
    }

    pub(super) fn script_validate_tree_model_selected(
        &self,
        model: &super::inspect_tree::InspectTreeModel,
        node_id: u64,
    ) -> Result<(), ScriptInspectTreeValidationFailure> {
        if !model.flat_node_ids.iter().any(|id| *id == node_id) {
            return Err(ScriptInspectTreeValidationFailure::NodeNotInTree);
        }

        let want = format!("node={node_id}");
        let ok = model
            .lines
            .iter()
            .any(|line| line.contains("> ") && line.contains(&want));
        if !ok {
            return Err(ScriptInspectTreeValidationFailure::SelectedMarkerMissing);
        }

        Ok(())
    }

    pub(super) fn is_locked(&self, window: AppWindowId) -> bool {
        self.state.locked_windows.contains(&window)
    }

    pub(super) fn help_is_open(&self, window: AppWindowId) -> bool {
        self.state.help_open_windows.contains(&window)
    }

    pub(super) fn help_search_query(&self, window: AppWindowId) -> Option<&str> {
        self.state
            .help_search_query
            .get(&window)
            .map(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
    }

    pub(super) fn help_selected_match_index(&self, window: AppWindowId) -> Option<usize> {
        self.state.help_selected_match_index.get(&window).copied()
    }

    pub(super) fn help_scroll_offset(&self, window: AppWindowId) -> usize {
        self.state
            .help_scroll_offset
            .get(&window)
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn set_help_scroll_offset(&mut self, window: AppWindowId, offset: usize) {
        if offset == 0 {
            self.state.help_scroll_offset.remove(&window);
        } else {
            self.state.help_scroll_offset.insert(window, offset);
        }
    }

    pub(super) fn set_help_matches(&mut self, window: AppWindowId, matches: Vec<u64>) {
        if matches.is_empty() {
            self.state.help_match_node_ids.remove(&window);
            self.state.help_selected_match_index.remove(&window);
            return;
        }

        self.state.help_match_node_ids.insert(window, matches);
        let len = self
            .state
            .help_match_node_ids
            .get(&window)
            .map(|v| v.len())
            .unwrap_or(0);
        if len == 0 {
            self.state.help_selected_match_index.remove(&window);
            return;
        }

        let idx = self
            .state
            .help_selected_match_index
            .get(&window)
            .copied()
            .unwrap_or(0)
            .min(len.saturating_sub(1));
        self.state.help_selected_match_index.insert(window, idx);
    }

    pub(super) fn tree_is_open(&self, window: AppWindowId) -> bool {
        self.state.tree_open_windows.contains(&window)
    }

    pub(super) fn set_tree_items(&mut self, window: AppWindowId, items: Vec<u64>) {
        if items.is_empty() {
            self.state.tree_flat_node_ids.remove(&window);
            self.state.tree_selected_index.remove(&window);
            self.state.tree_selected_node_id.remove(&window);
            return;
        }

        self.state.tree_flat_node_ids.insert(window, items);
        let items = self
            .state
            .tree_flat_node_ids
            .get(&window)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if items.is_empty() {
            self.state.tree_selected_index.remove(&window);
            self.state.tree_selected_node_id.remove(&window);
            return;
        }

        let mut idx = self
            .state
            .tree_selected_index
            .get(&window)
            .copied()
            .unwrap_or(0)
            .min(items.len().saturating_sub(1));

        if let Some(want_id) = self.state.tree_selected_node_id.get(&window).copied() {
            if let Some(pos) = items.iter().position(|id| *id == want_id) {
                idx = pos;
            }
        }

        let selected_id = items.get(idx).copied().unwrap_or_else(|| items[0]);
        self.state.tree_selected_index.insert(window, idx);
        self.state.tree_selected_node_id.insert(window, selected_id);
    }

    pub(super) fn tree_selected_node_id(&self, window: AppWindowId) -> Option<u64> {
        let items = self.state.tree_flat_node_ids.get(&window)?;
        if items.is_empty() {
            return None;
        }
        let idx = self
            .state
            .tree_selected_index
            .get(&window)
            .copied()
            .unwrap_or(0)
            .min(items.len().saturating_sub(1));
        items.get(idx).copied()
    }

    pub(super) fn focus_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.state.focus_node_id.get(&window).copied()
    }

    pub(super) fn focus_summary_line(&self, window: AppWindowId) -> Option<&str> {
        self.state
            .focus_summary_line
            .get(&window)
            .map(|s| s.as_str())
    }

    pub(super) fn focus_path_line(&self, window: AppWindowId) -> Option<&str> {
        self.state.focus_path_line.get(&window).map(|s| s.as_str())
    }

    pub(super) fn toast_message(&self, window: AppWindowId) -> Option<&str> {
        self.state.toast.get(&window).map(|t| t.message.as_str())
    }

    pub(super) fn best_selector_json(&self, window: AppWindowId) -> Option<&str> {
        self.state
            .focus_selector_json
            .get(&window)
            .map(|s| s.as_str())
            .or_else(|| {
                self.last_picked_selector_json
                    .get(&window)
                    .map(|s| s.as_str())
            })
            .or_else(|| {
                self.last_hovered_selector_json
                    .get(&window)
                    .map(|s| s.as_str())
            })
    }

    pub(super) fn wants_inspection_active(&mut self, window: AppWindowId) -> bool {
        let grace = self
            .pick_overlay_grace_frames
            .get(&window)
            .copied()
            .unwrap_or(0);
        if grace > 0 {
            let next = grace.saturating_sub(1);
            if next == 0 {
                self.pick_overlay_grace_frames.remove(&window);
            } else {
                self.pick_overlay_grace_frames.insert(window, next);
            }
        }

        if let Some(toast) = self.state.toast.get_mut(&window) {
            toast.remaining_frames = toast.remaining_frames.saturating_sub(1);
            if toast.remaining_frames == 0 {
                self.state.toast.remove(&window);
            }
        }

        self.pick_armed_run_id.is_some()
            || grace > 0
            || self.enabled
            || self.state.toast.contains_key(&window)
            || self
                .pending_pick
                .as_ref()
                .is_some_and(|p| p.window == window)
    }

    pub(super) fn maybe_intercept_event_for_shortcuts(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if let Event::TextInput(text) = event {
            let inspection_active = self.pick_armed_run_id.is_some() || self.enabled;
            if !inspection_active {
                return false;
            }

            if !self.help_is_open(window) {
                return false;
            }

            const MAX_QUERY_BYTES: usize = 64;
            let q = self.state.help_search_query.entry(window).or_default();
            let mut chars: Vec<char> = text.chars().collect();
            if let Some(expected) = self.state.help_suppress_next_text_input.remove(&window) {
                if chars.first().copied() == Some(expected) {
                    chars.remove(0);
                }
            }

            for ch in chars {
                if q.len() >= MAX_QUERY_BYTES {
                    break;
                }

                let ch = ch.to_ascii_lowercase();
                let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == ' ';
                if ok {
                    q.push(ch);
                }
            }

            if q.trim().is_empty() {
                self.state.help_search_query.remove(&window);
                self.state.help_suppress_next_text_input.remove(&window);
            }
            self.state.help_selected_match_index.insert(window, 0);
            app.request_redraw(window);
            return true;
        }

        let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
        else {
            return false;
        };
        if *repeat {
            return false;
        }

        let wants_command = modifiers.ctrl || modifiers.meta;
        let wants_diag_command = wants_command && modifiers.alt;

        match *key {
            KeyCode::KeyI if wants_diag_command => {
                let next_enabled = !self.enabled;
                if !next_enabled {
                    self.pick_armed_run_id.take();
                }

                self.set_enabled(next_enabled, self.consume_clicks);

                let _ = super::write_json(
                    cfg.inspect_path.clone(),
                    &UiInspectConfigV1 {
                        schema_version: 1,
                        enabled: next_enabled,
                        consume_clicks: self.consume_clicks,
                    },
                );
                let _ = super::touch_file(&cfg.inspect_trigger_path);

                let msg = if next_enabled {
                    "inspect: enabled"
                } else {
                    "inspect: disabled"
                };
                self.push_toast(window, msg.to_string());
                app.request_redraw(window);
                return true;
            }
            KeyCode::KeyH if wants_diag_command => {
                if !self.enabled {
                    self.set_enabled(true, self.consume_clicks);

                    let _ = super::write_json(
                        cfg.inspect_path.clone(),
                        &UiInspectConfigV1 {
                            schema_version: 1,
                            enabled: true,
                            consume_clicks: self.consume_clicks,
                        },
                    );
                    let _ = super::touch_file(&cfg.inspect_trigger_path);
                }

                let help_open = if self.state.help_open_windows.remove(&window) {
                    self.state.help_search_query.remove(&window);
                    self.state.help_suppress_next_text_input.remove(&window);
                    self.state.help_match_node_ids.remove(&window);
                    self.state.help_selected_match_index.remove(&window);
                    self.state.help_scroll_offset.remove(&window);
                    self.state.tree_open_windows.remove(&window);
                    self.state.tree_expanded_node_ids.remove(&window);
                    self.state.tree_flat_node_ids.remove(&window);
                    self.state.tree_selected_index.remove(&window);
                    self.state.tree_selected_node_id.remove(&window);
                    false
                } else {
                    self.state.help_open_windows.insert(window);
                    self.state.help_scroll_offset.remove(&window);
                    true
                };

                let msg = if help_open {
                    "inspect: help shown"
                } else {
                    "inspect: help hidden"
                };
                self.push_toast(window, msg.to_string());
                app.request_redraw(window);
                return true;
            }
            _ => {}
        }

        let inspection_active = self.pick_armed_run_id.is_some() || self.enabled;
        if !inspection_active {
            return false;
        }

        if self.help_is_open(window)
            && (modifiers.ctrl || modifiers.meta)
            && !(modifiers.alt || modifiers.alt_gr)
            && *key == KeyCode::Enter
        {
            if self.help_search_query(window).is_some() {
                if let Some(node_id) = self.help_selected_match_node_id(window) {
                    self.state.focus_down_stack.insert(window, Vec::new());
                    self.state.locked_windows.insert(window);
                    self.state
                        .pending_nav
                        .insert(window, InspectNavCommand::SelectNode(node_id));
                    self.state.pending_copy_selector_windows.insert(window);
                    self.push_toast(
                        window,
                        "inspect: locked match and copied selector".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
            } else if self.tree_is_open(window) {
                if let Some(node_id) = self.tree_selected_node_id(window) {
                    self.state.focus_down_stack.insert(window, Vec::new());
                    self.state.locked_windows.insert(window);
                    self.state
                        .pending_nav
                        .insert(window, InspectNavCommand::SelectNode(node_id));
                    self.state.pending_copy_selector_windows.insert(window);
                    self.push_toast(
                        window,
                        "inspect: locked node and copied selector".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
            }

            if self.state.help_search_query.remove(&window).is_some() {
                self.state.help_suppress_next_text_input.remove(&window);
                self.state.help_match_node_ids.remove(&window);
                self.state.help_selected_match_index.remove(&window);
                self.push_toast(window, "inspect: search cleared".to_string());
            }
            app.request_redraw(window);
            return true;
        }

        if self.help_is_open(window)
            && (modifiers.ctrl || modifiers.meta)
            && !(modifiers.alt || modifiers.alt_gr)
            && *key == KeyCode::KeyT
        {
            let tree_open = if self.state.tree_open_windows.remove(&window) {
                self.state.tree_expanded_node_ids.remove(&window);
                self.state.tree_flat_node_ids.remove(&window);
                self.state.tree_selected_index.remove(&window);
                self.state.tree_selected_node_id.remove(&window);
                self.set_help_scroll_offset(window, 0);
                false
            } else {
                self.state.tree_open_windows.insert(window);
                self.set_help_scroll_offset(window, usize::MAX / 4);
                true
            };

            self.push_toast(
                window,
                if tree_open {
                    "inspect: tree shown".to_string()
                } else {
                    "inspect: tree hidden".to_string()
                },
            );
            app.request_redraw(window);
            return true;
        }

        if self.help_is_open(window)
            && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
        {
            if self.handle_help_mode_key(app, window, *key, wants_command) {
                return true;
            }
        }

        match *key {
            KeyCode::Escape => {
                if self.pick_armed_run_id.take().is_some() {
                    self.push_toast(window, "inspect: pick disarmed".to_string());
                    app.request_redraw(window);
                    return true;
                }

                if self.enabled {
                    self.set_enabled(false, self.consume_clicks);

                    let _ = super::write_json(
                        cfg.inspect_path.clone(),
                        &UiInspectConfigV1 {
                            schema_version: 1,
                            enabled: false,
                            consume_clicks: self.consume_clicks,
                        },
                    );
                    let _ = super::touch_file(&cfg.inspect_trigger_path);

                    self.push_toast(window, "inspect: disabled".to_string());
                    app.request_redraw(window);
                    return true;
                }
                false
            }
            KeyCode::KeyL => {
                if self.state.locked_windows.remove(&window) {
                    self.state.focus_down_stack.remove(&window);
                    self.push_toast(window, "inspect: unlocked".to_string());
                } else if let Some(hovered) = self.last_hovered_node_id.get(&window).copied() {
                    self.last_picked_node_id.insert(window, hovered);
                    if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                        self.last_picked_selector_json.insert(window, sel);
                    }
                    self.state.focus_node_id.insert(window, hovered);
                    if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                        self.state.focus_selector_json.insert(window, sel);
                    }
                    self.state.focus_down_stack.insert(window, Vec::new());
                    self.state.locked_windows.insert(window);
                    self.push_toast(window, "inspect: locked selection".to_string());
                } else {
                    self.push_toast(window, "inspect: nothing to lock".to_string());
                }
                app.request_redraw(window);
                true
            }
            KeyCode::KeyC => {
                if !wants_command {
                    return false;
                }
                if modifiers.shift {
                    self.state.pending_copy_details_windows.insert(window);
                    self.push_toast(window, "inspect: copy requested".to_string());
                    app.request_redraw(window);
                    return true;
                }

                let Some(payload) = self.best_selector_json(window).map(|s| s.to_string()) else {
                    self.push_toast(window, "inspect: no selector to copy".to_string());
                    app.request_redraw(window);
                    return true;
                };
                let token = app.next_clipboard_token();
                app.push_effect(Effect::ClipboardWriteText {
                    window,
                    token,
                    text: payload,
                });
                self.push_toast(window, "inspect: copied selector".to_string());
                app.request_redraw(window);
                true
            }
            KeyCode::KeyF => {
                if !self.enabled {
                    return false;
                }
                self.state
                    .pending_nav
                    .insert(window, InspectNavCommand::Focus);
                self.push_toast(window, "inspect: select focused node".to_string());
                app.request_redraw(window);
                true
            }
            KeyCode::ArrowUp => {
                if !modifiers.alt {
                    return false;
                }
                if !self.is_locked(window) {
                    self.push_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
                self.state.pending_nav.insert(window, InspectNavCommand::Up);
                app.request_redraw(window);
                true
            }
            KeyCode::ArrowDown => {
                if !modifiers.alt {
                    return false;
                }
                if !self.is_locked(window) {
                    self.push_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    app.request_redraw(window);
                    return true;
                }
                self.state
                    .pending_nav
                    .insert(window, InspectNavCommand::Down);
                app.request_redraw(window);
                true
            }
            _ => false,
        }
    }

    pub(super) fn try_consume_armed_pick(&mut self, window: AppWindowId, position: Point) -> bool {
        let Some(run_id) = self.pick_armed_run_id.take() else {
            return false;
        };
        self.pending_pick = Some(PendingPick {
            run_id,
            window,
            position,
        });
        true
    }

    pub(super) fn start_pending_pick(
        &mut self,
        window: AppWindowId,
        run_id: u64,
        position: Point,
    ) -> bool {
        self.pending_pick = Some(PendingPick {
            run_id,
            window,
            position,
        });
        self.consume_clicks
    }

    pub(super) fn update_hover(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        hovered_node_id: Option<u64>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.enabled {
            return;
        }
        let Some(snapshot) = snapshot else {
            return;
        };
        let Some(hovered_id) = hovered_node_id else {
            self.last_hovered_node_id.remove(&window);
            self.last_hovered_selector_json.remove(&window);
            return;
        };
        if self.is_locked(window) {
            return;
        }

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == hovered_id)
        else {
            return;
        };
        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);
        let selector = super::best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            cfg,
        )
        .or_else(|| super::best_selector_for_node(snapshot, node, element, cfg));
        let Some(selector) = selector else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.last_hovered_node_id.insert(window, hovered_id);
            self.last_hovered_selector_json.insert(window, json.clone());

            self.state.focus_node_id.insert(window, hovered_id);
            self.state.focus_selector_json.insert(window, json);
            self.state.focus_down_stack.insert(window, Vec::new());
        }
    }

    pub(super) fn apply_navigation(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        if !self.enabled {
            self.state.pending_nav.remove(&window);
            return;
        }
        let Some(cmd) = self.state.pending_nav.remove(&window) else {
            return;
        };
        let Some(snapshot) = snapshot else {
            self.push_toast(window, "inspect: no semantics snapshot".to_string());
            return;
        };

        match cmd {
            InspectNavCommand::Focus => {
                let Some(node) = snapshot.focus else {
                    self.push_toast(window, "inspect: no focused node".to_string());
                    return;
                };
                let id = node.data().as_ffi();
                self.state.focus_down_stack.insert(window, Vec::new());
                self.state.locked_windows.insert(window);
                self.set_focus(cfg, window, snapshot, id, element_runtime);
            }
            InspectNavCommand::SelectNode(node_id) => {
                self.state.focus_down_stack.insert(window, Vec::new());
                self.state.locked_windows.insert(window);
                self.set_focus(cfg, window, snapshot, node_id, element_runtime);
            }
            InspectNavCommand::Up => {
                if !self.is_locked(window) {
                    self.push_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }

                let current = self
                    .state
                    .focus_node_id
                    .get(&window)
                    .copied()
                    .or_else(|| self.last_picked_node_id.get(&window).copied())
                    .or_else(|| self.last_hovered_node_id.get(&window).copied());
                let Some(current) = current else {
                    self.push_toast(window, "inspect: no focused node".to_string());
                    return;
                };

                let Some(parent) = super::parent_node_id(snapshot, current) else {
                    self.push_toast(window, "inspect: reached root".to_string());
                    return;
                };
                self.state
                    .focus_down_stack
                    .entry(window)
                    .or_default()
                    .push(current);
                self.set_focus(cfg, window, snapshot, parent, element_runtime);
                self.push_toast(window, "inspect: parent".to_string());
            }
            InspectNavCommand::Down => {
                if !self.is_locked(window) {
                    self.push_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }
                let Some(prev) = self
                    .state
                    .focus_down_stack
                    .get_mut(&window)
                    .and_then(|s| s.pop())
                else {
                    self.push_toast(window, "inspect: no child history".to_string());
                    return;
                };
                self.set_focus(cfg, window, snapshot, prev, element_runtime);
                self.push_toast(window, "inspect: child".to_string());
            }
        }
    }

    pub(super) fn update_focus_lines(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        window: AppWindowId,
        snapshot: Option<&fret_core::SemanticsSnapshot>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        let Some(snapshot) = snapshot else {
            self.state.focus_summary_line.remove(&window);
            self.state.focus_path_line.remove(&window);
            if self.state.pending_copy_details_windows.remove(&window) {
                self.push_toast(window, "inspect: no semantics snapshot".to_string());
            }
            return;
        };

        let node_id = self
            .state
            .focus_node_id
            .get(&window)
            .copied()
            .or_else(|| self.last_picked_node_id.get(&window).copied())
            .or_else(|| self.last_hovered_node_id.get(&window).copied());
        let Some(node_id) = node_id else {
            self.state.focus_summary_line.remove(&window);
            self.state.focus_path_line.remove(&window);
            if self.state.pending_copy_details_windows.remove(&window) {
                self.push_toast(window, "inspect: no focused node".to_string());
            }
            return;
        };

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            self.state.focus_summary_line.remove(&window);
            self.state.focus_path_line.remove(&window);
            if self.state.pending_copy_details_windows.remove(&window) {
                self.push_toast(window, "inspect: focused node missing".to_string());
            }
            return;
        };

        let role = super::semantics_role_label(node.role);
        let mut summary = format!("focus: {role} node={node_id}");

        if let Some(runtime) = element_runtime
            && let Some(element) = runtime.element_for_node(window, node.id)
        {
            summary.push_str(&format!(" element={}", element.0));
            if let Some(path) = runtime.debug_path_for_element(window, element) {
                let path = super::truncate_debug_value(&path, 200);
                summary.push_str(&format!(" element_path={path}"));
            }
        }
        if let Some(test_id) = node.test_id.as_deref() {
            summary.push_str(&format!(" test_id={test_id}"));
        }
        if !cfg.redact_text
            && let Some(label) = node.label.as_deref()
        {
            let label = super::truncate_debug_value(label, 120);
            summary.push_str(&format!(" label={label}"));
        }

        let path_line = super::format_inspect_path(snapshot, node_id, cfg.redact_text, 10);

        self.state
            .focus_summary_line
            .insert(window, summary.clone());
        if let Some(path) = path_line.as_ref() {
            self.state.focus_path_line.insert(window, path.clone());
        } else {
            self.state.focus_path_line.remove(&window);
        }

        if !self.state.pending_copy_details_windows.remove(&window) {
            return;
        }

        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);

        let best = super::best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            cfg,
        )
        .or_else(|| super::best_selector_for_node(snapshot, node, element, cfg));

        let mut lines: Vec<String> = Vec::new();
        if let Some(best) = best.as_ref().and_then(|s| serde_json::to_string(s).ok()) {
            lines.push(format!("selector: {best}"));
        }
        lines.push(summary);
        if let Some(path) = path_line {
            lines.push(path);
        }

        let report = super::inspect_selector_candidates_report(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            cfg,
        );
        if !report.trim().is_empty() {
            lines.push(String::new());
            lines.push("selector_candidates:".to_string());
            lines.extend(report.lines().map(|l: &str| l.to_string()));
        }

        let payload = lines.join("\n");
        if payload.trim().is_empty() {
            self.push_toast(window, "inspect: no details available".to_string());
            return;
        }

        self.state
            .pending_copy_details_payload
            .insert(window, payload);
        self.push_toast(window, "inspect: details copied".to_string());
    }

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

    fn set_focus(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        window: AppWindowId,
        snapshot: &fret_core::SemanticsSnapshot,
        node_id: u64,
        element_runtime: Option<&ElementRuntime>,
    ) {
        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            return;
        };
        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);
        let selector = super::best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            cfg,
        )
        .or_else(|| super::best_selector_for_node(snapshot, node, element, cfg));
        let Some(selector) = selector else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.state.focus_node_id.insert(window, node_id);
            self.state.focus_selector_json.insert(window, json.clone());
            self.last_picked_node_id.insert(window, node_id);
            self.last_picked_selector_json.insert(window, json);
        }
    }

    fn handle_help_mode_key(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        key: KeyCode,
        wants_command: bool,
    ) -> bool {
        match key {
            KeyCode::ArrowUp => {
                if self.help_search_query(window).is_some()
                    && self
                        .state
                        .help_match_node_ids
                        .get(&window)
                        .is_some_and(|m| !m.is_empty())
                {
                    let len = self
                        .state
                        .help_match_node_ids
                        .get(&window)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    let idx = self
                        .state
                        .help_selected_match_index
                        .get(&window)
                        .copied()
                        .unwrap_or(0);
                    let next = if idx == 0 { len - 1 } else { idx - 1 };
                    self.state.help_selected_match_index.insert(window, next);
                    app.request_redraw(window);
                    return true;
                }

                if self.help_search_query(window).is_none()
                    && self.tree_is_open(window)
                    && self
                        .state
                        .tree_flat_node_ids
                        .get(&window)
                        .is_some_and(|v| !v.is_empty())
                {
                    let items = self.state.tree_flat_node_ids.get(&window).unwrap();
                    let len = items.len();
                    let idx = self
                        .state
                        .tree_selected_index
                        .get(&window)
                        .copied()
                        .unwrap_or(0)
                        .min(len.saturating_sub(1));
                    let next = if idx == 0 { len - 1 } else { idx - 1 };
                    self.state.tree_selected_index.insert(window, next);
                    if let Some(id) = items.get(next).copied() {
                        self.state.tree_selected_node_id.insert(window, id);
                    }
                    app.request_redraw(window);
                    return true;
                }
            }
            KeyCode::ArrowDown => {
                if self.help_search_query(window).is_some()
                    && self
                        .state
                        .help_match_node_ids
                        .get(&window)
                        .is_some_and(|m| !m.is_empty())
                {
                    let len = self
                        .state
                        .help_match_node_ids
                        .get(&window)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    let idx = self
                        .state
                        .help_selected_match_index
                        .get(&window)
                        .copied()
                        .unwrap_or(0);
                    let next = (idx + 1) % len.max(1);
                    self.state.help_selected_match_index.insert(window, next);
                    app.request_redraw(window);
                    return true;
                }

                if self.help_search_query(window).is_none()
                    && self.tree_is_open(window)
                    && self
                        .state
                        .tree_flat_node_ids
                        .get(&window)
                        .is_some_and(|v| !v.is_empty())
                {
                    let items = self.state.tree_flat_node_ids.get(&window).unwrap();
                    let len = items.len();
                    let idx = self
                        .state
                        .tree_selected_index
                        .get(&window)
                        .copied()
                        .unwrap_or(0)
                        .min(len.saturating_sub(1));
                    let next = (idx + 1) % len.max(1);
                    self.state.tree_selected_index.insert(window, next);
                    if let Some(id) = items.get(next).copied() {
                        self.state.tree_selected_node_id.insert(window, id);
                    }
                    app.request_redraw(window);
                    return true;
                }
            }
            KeyCode::ArrowRight => {
                if self.help_search_query(window).is_none() && self.tree_is_open(window) {
                    if let Some(node_id) = self.tree_selected_node_id(window) {
                        self.state
                            .tree_expanded_node_ids
                            .entry(window)
                            .or_default()
                            .insert(node_id);
                        app.request_redraw(window);
                        return true;
                    }
                }
            }
            KeyCode::ArrowLeft => {
                if self.help_search_query(window).is_none() && self.tree_is_open(window) {
                    if let Some(node_id) = self.tree_selected_node_id(window) {
                        if self
                            .state
                            .tree_expanded_node_ids
                            .entry(window)
                            .or_default()
                            .remove(&node_id)
                        {
                            app.request_redraw(window);
                            return true;
                        }
                    }
                }
            }
            KeyCode::PageUp => {
                let offset = self.help_scroll_offset(window);
                self.set_help_scroll_offset(window, offset.saturating_sub(20));
                app.request_redraw(window);
                return true;
            }
            KeyCode::PageDown => {
                let offset = self.help_scroll_offset(window);
                self.set_help_scroll_offset(window, offset.saturating_add(20));
                app.request_redraw(window);
                return true;
            }
            KeyCode::Home => {
                self.set_help_scroll_offset(window, 0);
                app.request_redraw(window);
                return true;
            }
            KeyCode::End => {
                self.set_help_scroll_offset(window, usize::MAX / 4);
                app.request_redraw(window);
                return true;
            }
            KeyCode::Backspace => {
                if let Some(q) = self.state.help_search_query.get_mut(&window) {
                    q.pop();
                    if q.trim().is_empty() {
                        self.state.help_search_query.remove(&window);
                        self.state.help_suppress_next_text_input.remove(&window);
                    }
                }
                self.state.help_selected_match_index.insert(window, 0);
                app.request_redraw(window);
                return true;
            }
            KeyCode::Enter => {
                if self.help_search_query(window).is_some() {
                    if let Some(node_id) = self.help_selected_match_node_id(window) {
                        self.state.focus_down_stack.insert(window, Vec::new());
                        self.state.locked_windows.insert(window);
                        self.state
                            .pending_nav
                            .insert(window, InspectNavCommand::SelectNode(node_id));
                        if wants_command {
                            self.state.pending_copy_selector_windows.insert(window);
                            self.push_toast(
                                window,
                                "inspect: locked match and copied selector".to_string(),
                            );
                        } else {
                            self.push_toast(
                                window,
                                "inspect: locked match selection (press Ctrl/Cmd+C to copy selector)"
                                    .to_string(),
                            );
                        }
                        app.request_redraw(window);
                        return true;
                    }
                } else if self.tree_is_open(window) {
                    if let Some(node_id) = self.tree_selected_node_id(window) {
                        self.state.focus_down_stack.insert(window, Vec::new());
                        self.state.locked_windows.insert(window);
                        self.state
                            .pending_nav
                            .insert(window, InspectNavCommand::SelectNode(node_id));
                        self.push_toast(
                            window,
                            "inspect: locked node selection (press Ctrl/Cmd+C to copy selector)"
                                .to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                }

                if self.state.help_search_query.remove(&window).is_some() {
                    self.state.help_suppress_next_text_input.remove(&window);
                    self.state.help_match_node_ids.remove(&window);
                    self.state.help_selected_match_index.remove(&window);
                    self.push_toast(window, "inspect: search cleared".to_string());
                }
                app.request_redraw(window);
                return true;
            }
            KeyCode::Space => {
                const MAX_QUERY_BYTES: usize = 64;
                let q = self.state.help_search_query.entry(window).or_default();
                if q.len() < MAX_QUERY_BYTES {
                    q.push(' ');
                }
                if q.trim().is_empty() {
                    self.state.help_search_query.remove(&window);
                    self.state.help_suppress_next_text_input.remove(&window);
                }
                self.state.help_selected_match_index.insert(window, 0);
                app.request_redraw(window);
                return true;
            }
            _ => {
                if let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) {
                    const MAX_QUERY_BYTES: usize = 64;
                    let q = self.state.help_search_query.entry(window).or_default();
                    if q.len() < MAX_QUERY_BYTES {
                        let ok = ch.is_ascii_alphanumeric() || ch == '-';
                        if ok {
                            q.push(ch);
                            self.state.help_suppress_next_text_input.insert(window, ch);
                        }
                    }
                    if q.trim().is_empty() {
                        self.state.help_search_query.remove(&window);
                        self.state.help_suppress_next_text_input.remove(&window);
                    }
                    self.state.help_selected_match_index.insert(window, 0);
                    app.request_redraw(window);
                    return true;
                }
            }
        }
        false
    }

    fn help_selected_match_node_id(&self, window: AppWindowId) -> Option<u64> {
        let list = self.state.help_match_node_ids.get(&window)?;
        if list.is_empty() {
            return None;
        }
        let idx = self
            .state
            .help_selected_match_index
            .get(&window)
            .copied()
            .unwrap_or(0)
            .min(list.len().saturating_sub(1));
        list.get(idx).copied()
    }

    fn push_toast(&mut self, window: AppWindowId, message: String) {
        self.state.toast.insert(
            window,
            InspectToast {
                message,
                remaining_frames: 90,
            },
        );
    }
}
