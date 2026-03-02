use std::collections::HashMap;

use fret_app::{App, Effect};
use fret_core::AppWindowId;
use fret_core::{Event, KeyCode};
use fret_diag_protocol::UiInspectConfigV1;

use super::UiDiagnosticsConfig;
use super::inspect_state::InspectState;

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
                app.push_effect(Effect::ClipboardSetText { text: payload });
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
