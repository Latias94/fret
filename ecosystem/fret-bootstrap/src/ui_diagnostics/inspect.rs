use super::*;

pub(super) use super::inspect_controller::{InspectNavCommand, InspectToast};

impl UiDiagnosticsService {
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

    pub(super) fn inspect_help_selected_match_index(&self, window: AppWindowId) -> Option<usize> {
        self.inspector.help_selected_match_index(window)
    }

    pub(super) fn inspect_help_scroll_offset(&self, window: AppWindowId) -> usize {
        self.inspector.help_scroll_offset(window)
    }

    pub(super) fn set_inspect_help_scroll_offset(&mut self, window: AppWindowId, offset: usize) {
        self.inspector.set_help_scroll_offset(window, offset);
    }

    pub(super) fn set_inspect_help_matches(&mut self, window: AppWindowId, matches: Vec<u64>) {
        self.inspector.set_help_matches(window, matches);
    }

    pub(super) fn inspect_tree_is_open(&self, window: AppWindowId) -> bool {
        self.inspector.tree_is_open(window)
    }

    pub(super) fn set_inspect_tree_items(&mut self, window: AppWindowId, items: Vec<u64>) {
        self.inspector.set_tree_items(window, items);
    }

    pub(super) fn inspect_tree_selected_node_id(&self, window: AppWindowId) -> Option<u64> {
        self.inspector.tree_selected_node_id(window)
    }

    fn inspect_help_selected_match_node_id(&self, window: AppWindowId) -> Option<u64> {
        let list = self.inspector.state.help_match_node_ids.get(&window)?;
        if list.is_empty() {
            return None;
        }
        let idx = self
            .inspector
            .state
            .help_selected_match_index
            .get(&window)
            .copied()
            .unwrap_or(0)
            .min(list.len().saturating_sub(1));
        list.get(idx).copied()
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

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        self.inspector.wants_inspection_active(window)
    }

    /// Returns `true` if the event was consumed by inspect-mode shortcuts.
    pub fn maybe_intercept_event_for_inspect_shortcuts(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        return self
            .inspector
            .maybe_intercept_event_for_shortcuts(&self.cfg, app, window, event);

        #[allow(unreachable_code)]
        {
            if let Event::TextInput(text) = event {
                let inspection_active =
                    self.inspector.pick_armed_run_id.is_some() || self.inspector.enabled;
                if !inspection_active {
                    return false;
                }

                if !self.inspect_help_is_open(window) {
                    return false;
                }

                const MAX_QUERY_BYTES: usize = 64;
                let q = self
                    .inspector
                    .state
                    .help_search_query
                    .entry(window)
                    .or_default();
                let mut chars: Vec<char> = text.chars().collect();
                if let Some(expected) = self
                    .inspector
                    .state
                    .help_suppress_next_text_input
                    .remove(&window)
                {
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
                    self.inspector.state.help_search_query.remove(&window);
                    self.inspector
                        .state
                        .help_suppress_next_text_input
                        .remove(&window);
                }
                self.inspector
                    .state
                    .help_selected_match_index
                    .insert(window, 0);
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

            // Allow toggling inspection without filesystem triggers / DevTools WS.
            //
            // Keep this chord gated (Alt required) so we don't conflict with common app shortcuts.
            match *key {
                KeyCode::KeyI if wants_diag_command => {
                    let next_enabled = !self.inspector.enabled;
                    if !next_enabled {
                        self.inspector.pick_armed_run_id.take();
                    }

                    self.set_inspect_enabled(next_enabled, self.inspector.consume_clicks);

                    let _ = write_json(
                        self.cfg.inspect_path.clone(),
                        &UiInspectConfigV1 {
                            schema_version: 1,
                            enabled: next_enabled,
                            consume_clicks: self.inspector.consume_clicks,
                        },
                    );
                    let _ = touch_file(&self.cfg.inspect_trigger_path);

                    let msg = if next_enabled {
                        "inspect: enabled"
                    } else {
                        "inspect: disabled"
                    };
                    self.push_inspect_toast(window, msg.to_string());
                    app.request_redraw(window);
                    return true;
                }
                KeyCode::KeyH if wants_diag_command => {
                    if !self.inspector.enabled {
                        self.set_inspect_enabled(true, self.inspector.consume_clicks);

                        let _ = write_json(
                            self.cfg.inspect_path.clone(),
                            &UiInspectConfigV1 {
                                schema_version: 1,
                                enabled: true,
                                consume_clicks: self.inspector.consume_clicks,
                            },
                        );
                        let _ = touch_file(&self.cfg.inspect_trigger_path);
                    }

                    let help_open = if self.inspector.state.help_open_windows.remove(&window) {
                        self.inspector.state.help_search_query.remove(&window);
                        self.inspector
                            .state
                            .help_suppress_next_text_input
                            .remove(&window);
                        self.inspector.state.help_match_node_ids.remove(&window);
                        self.inspector
                            .state
                            .help_selected_match_index
                            .remove(&window);
                        self.inspector.state.help_scroll_offset.remove(&window);
                        self.inspector.state.tree_open_windows.remove(&window);
                        self.inspector.state.tree_expanded_node_ids.remove(&window);
                        self.inspector.state.tree_flat_node_ids.remove(&window);
                        self.inspector.state.tree_selected_index.remove(&window);
                        self.inspector.state.tree_selected_node_id.remove(&window);
                        false
                    } else {
                        self.inspector.state.help_open_windows.insert(window);
                        self.inspector.state.help_scroll_offset.remove(&window);
                        true
                    };

                    let msg = if help_open {
                        "inspect: help shown"
                    } else {
                        "inspect: help hidden"
                    };
                    self.push_inspect_toast(window, msg.to_string());
                    app.request_redraw(window);
                    return true;
                }
                _ => {}
            }

            let inspection_active =
                self.inspector.pick_armed_run_id.is_some() || self.inspector.enabled;
            if !inspection_active {
                return false;
            }

            // Help mode: allow `Ctrl/Cmd+Enter` while search is active to lock the selected match and
            // immediately copy the best selector (without requiring a separate `Ctrl/Cmd+C` press).
            if self.inspect_help_is_open(window)
                && (modifiers.ctrl || modifiers.meta)
                && !(modifiers.alt || modifiers.alt_gr)
                && *key == KeyCode::Enter
            {
                if self.inspect_help_search_query(window).is_some() {
                    if let Some(node_id) = self.inspect_help_selected_match_node_id(window) {
                        self.inspector
                            .state
                            .focus_down_stack
                            .insert(window, Vec::new());
                        self.inspector.state.locked_windows.insert(window);
                        self.inspector
                            .state
                            .pending_nav
                            .insert(window, InspectNavCommand::SelectNode(node_id));
                        self.inspector
                            .state
                            .pending_copy_selector_windows
                            .insert(window);
                        self.push_inspect_toast(
                            window,
                            "inspect: locked match and copied selector".to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                } else if self.inspect_tree_is_open(window) {
                    if let Some(node_id) = self.inspect_tree_selected_node_id(window) {
                        self.inspector
                            .state
                            .focus_down_stack
                            .insert(window, Vec::new());
                        self.inspector.state.locked_windows.insert(window);
                        self.inspector
                            .state
                            .pending_nav
                            .insert(window, InspectNavCommand::SelectNode(node_id));
                        self.inspector
                            .state
                            .pending_copy_selector_windows
                            .insert(window);
                        self.push_inspect_toast(
                            window,
                            "inspect: locked node and copied selector".to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                }

                if self
                    .inspector
                    .state
                    .help_search_query
                    .remove(&window)
                    .is_some()
                {
                    self.inspector
                        .state
                        .help_suppress_next_text_input
                        .remove(&window);
                    self.inspector.state.help_match_node_ids.remove(&window);
                    self.inspector
                        .state
                        .help_selected_match_index
                        .remove(&window);
                    self.push_inspect_toast(window, "inspect: search cleared".to_string());
                }
                app.request_redraw(window);
                return true;
            }

            if self.inspect_help_is_open(window)
                && (modifiers.ctrl || modifiers.meta)
                && !(modifiers.alt || modifiers.alt_gr)
                && *key == KeyCode::KeyT
            {
                let tree_open = if self.inspector.state.tree_open_windows.remove(&window) {
                    self.inspector.state.tree_expanded_node_ids.remove(&window);
                    self.inspector.state.tree_flat_node_ids.remove(&window);
                    self.inspector.state.tree_selected_index.remove(&window);
                    self.inspector.state.tree_selected_node_id.remove(&window);
                    self.set_inspect_help_scroll_offset(window, 0);
                    false
                } else {
                    self.inspector.state.tree_open_windows.insert(window);
                    // Jump towards the bottom so the tree section is visible even when the help output
                    // grows (explainability + neighborhood + matches).
                    self.set_inspect_help_scroll_offset(window, usize::MAX / 4);
                    true
                };

                self.push_inspect_toast(
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

            if self.inspect_help_is_open(window)
                && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
            {
                match *key {
                    KeyCode::ArrowUp => {
                        if self.inspect_help_search_query(window).is_some()
                            && self
                                .inspector
                                .state
                                .help_match_node_ids
                                .get(&window)
                                .is_some_and(|m| !m.is_empty())
                        {
                            let len = self
                                .inspector
                                .state
                                .help_match_node_ids
                                .get(&window)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            let idx = self
                                .inspector
                                .state
                                .help_selected_match_index
                                .get(&window)
                                .copied()
                                .unwrap_or(0);
                            let next = if idx == 0 { len - 1 } else { idx - 1 };
                            self.inspector
                                .state
                                .help_selected_match_index
                                .insert(window, next);
                            app.request_redraw(window);
                            return true;
                        }

                        if self.inspect_help_search_query(window).is_none()
                            && self.inspect_tree_is_open(window)
                            && self
                                .inspector
                                .state
                                .tree_flat_node_ids
                                .get(&window)
                                .is_some_and(|v| !v.is_empty())
                        {
                            let items = self
                                .inspector
                                .state
                                .tree_flat_node_ids
                                .get(&window)
                                .unwrap();
                            let len = items.len();
                            let idx = self
                                .inspector
                                .state
                                .tree_selected_index
                                .get(&window)
                                .copied()
                                .unwrap_or(0)
                                .min(len.saturating_sub(1));
                            let next = if idx == 0 { len - 1 } else { idx - 1 };
                            self.inspector
                                .state
                                .tree_selected_index
                                .insert(window, next);
                            if let Some(id) = items.get(next).copied() {
                                self.inspector
                                    .state
                                    .tree_selected_node_id
                                    .insert(window, id);
                            }
                            app.request_redraw(window);
                            return true;
                        }
                    }
                    KeyCode::ArrowDown => {
                        if self.inspect_help_search_query(window).is_some()
                            && self
                                .inspector
                                .state
                                .help_match_node_ids
                                .get(&window)
                                .is_some_and(|m| !m.is_empty())
                        {
                            let len = self
                                .inspector
                                .state
                                .help_match_node_ids
                                .get(&window)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            let idx = self
                                .inspector
                                .state
                                .help_selected_match_index
                                .get(&window)
                                .copied()
                                .unwrap_or(0);
                            let next = (idx + 1) % len.max(1);
                            self.inspector
                                .state
                                .help_selected_match_index
                                .insert(window, next);
                            app.request_redraw(window);
                            return true;
                        }

                        if self.inspect_help_search_query(window).is_none()
                            && self.inspect_tree_is_open(window)
                            && self
                                .inspector
                                .state
                                .tree_flat_node_ids
                                .get(&window)
                                .is_some_and(|v| !v.is_empty())
                        {
                            let items = self
                                .inspector
                                .state
                                .tree_flat_node_ids
                                .get(&window)
                                .unwrap();
                            let len = items.len();
                            let idx = self
                                .inspector
                                .state
                                .tree_selected_index
                                .get(&window)
                                .copied()
                                .unwrap_or(0)
                                .min(len.saturating_sub(1));
                            let next = (idx + 1) % len.max(1);
                            self.inspector
                                .state
                                .tree_selected_index
                                .insert(window, next);
                            if let Some(id) = items.get(next).copied() {
                                self.inspector
                                    .state
                                    .tree_selected_node_id
                                    .insert(window, id);
                            }
                            app.request_redraw(window);
                            return true;
                        }
                    }
                    KeyCode::ArrowRight => {
                        if self.inspect_help_search_query(window).is_none()
                            && self.inspect_tree_is_open(window)
                        {
                            if let Some(node_id) = self.inspect_tree_selected_node_id(window) {
                                self.inspector
                                    .state
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
                        if self.inspect_help_search_query(window).is_none()
                            && self.inspect_tree_is_open(window)
                        {
                            if let Some(node_id) = self.inspect_tree_selected_node_id(window) {
                                if self
                                    .inspector
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
                        let offset = self.inspect_help_scroll_offset(window);
                        self.set_inspect_help_scroll_offset(window, offset.saturating_sub(20));
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::PageDown => {
                        let offset = self.inspect_help_scroll_offset(window);
                        self.set_inspect_help_scroll_offset(window, offset.saturating_add(20));
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::Home => {
                        self.set_inspect_help_scroll_offset(window, 0);
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::End => {
                        self.set_inspect_help_scroll_offset(window, usize::MAX / 4);
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::Backspace => {
                        if let Some(q) = self.inspector.state.help_search_query.get_mut(&window) {
                            q.pop();
                            if q.trim().is_empty() {
                                self.inspector.state.help_search_query.remove(&window);
                                self.inspector
                                    .state
                                    .help_suppress_next_text_input
                                    .remove(&window);
                            }
                        }
                        self.inspector
                            .state
                            .help_selected_match_index
                            .insert(window, 0);
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::Enter => {
                        if self.inspect_help_search_query(window).is_some() {
                            if let Some(node_id) = self.inspect_help_selected_match_node_id(window)
                            {
                                self.inspector
                                    .state
                                    .focus_down_stack
                                    .insert(window, Vec::new());
                                self.inspector.state.locked_windows.insert(window);
                                let wants_copy = wants_command;
                                self.inspector
                                    .state
                                    .pending_nav
                                    .insert(window, InspectNavCommand::SelectNode(node_id));
                                if wants_copy {
                                    self.inspector
                                        .state
                                        .pending_copy_selector_windows
                                        .insert(window);
                                }
                                if wants_copy {
                                    self.push_inspect_toast(
                                        window,
                                        "inspect: locked match and copied selector".to_string(),
                                    );
                                } else {
                                    self.push_inspect_toast(
                                    window,
                                    "inspect: locked match selection (press Ctrl/Cmd+C to copy selector)"
                                        .to_string(),
                                );
                                }
                                app.request_redraw(window);
                                return true;
                            }
                        } else if self.inspect_tree_is_open(window) {
                            if let Some(node_id) = self.inspect_tree_selected_node_id(window) {
                                self.inspector
                                    .state
                                    .focus_down_stack
                                    .insert(window, Vec::new());
                                self.inspector.state.locked_windows.insert(window);
                                self.inspector
                                    .state
                                    .pending_nav
                                    .insert(window, InspectNavCommand::SelectNode(node_id));
                                self.push_inspect_toast(
                                window,
                                "inspect: locked node selection (press Ctrl/Cmd+C to copy selector)"
                                    .to_string(),
                            );
                                app.request_redraw(window);
                                return true;
                            }
                        }

                        if self
                            .inspector
                            .state
                            .help_search_query
                            .remove(&window)
                            .is_some()
                        {
                            self.inspector
                                .state
                                .help_suppress_next_text_input
                                .remove(&window);
                            self.inspector.state.help_match_node_ids.remove(&window);
                            self.inspector
                                .state
                                .help_selected_match_index
                                .remove(&window);
                            self.push_inspect_toast(window, "inspect: search cleared".to_string());
                        }
                        app.request_redraw(window);
                        return true;
                    }
                    KeyCode::Space => {
                        const MAX_QUERY_BYTES: usize = 64;
                        let q = self
                            .inspector
                            .state
                            .help_search_query
                            .entry(window)
                            .or_default();
                        if q.len() < MAX_QUERY_BYTES {
                            q.push(' ');
                        }
                        if q.trim().is_empty() {
                            self.inspector.state.help_search_query.remove(&window);
                            self.inspector
                                .state
                                .help_suppress_next_text_input
                                .remove(&window);
                        }
                        self.inspector
                            .state
                            .help_selected_match_index
                            .insert(window, 0);
                        app.request_redraw(window);
                        return true;
                    }
                    _ => {
                        if let Some(ch) = fret_core::keycode_to_ascii_lowercase(*key) {
                            const MAX_QUERY_BYTES: usize = 64;
                            let q = self
                                .inspector
                                .state
                                .help_search_query
                                .entry(window)
                                .or_default();
                            if q.len() < MAX_QUERY_BYTES {
                                let ok = ch.is_ascii_alphanumeric() || ch == '-';
                                if ok {
                                    q.push(ch);
                                    self.inspector
                                        .state
                                        .help_suppress_next_text_input
                                        .insert(window, ch);
                                }
                            }
                            if q.trim().is_empty() {
                                self.inspector.state.help_search_query.remove(&window);
                                self.inspector
                                    .state
                                    .help_suppress_next_text_input
                                    .remove(&window);
                            }
                            self.inspector
                                .state
                                .help_selected_match_index
                                .insert(window, 0);
                            app.request_redraw(window);
                            return true;
                        }
                    }
                }
            }

            match *key {
                KeyCode::Escape => {
                    if self.inspector.pick_armed_run_id.take().is_some() {
                        self.push_inspect_toast(window, "inspect: pick disarmed".to_string());
                        app.request_redraw(window);
                        return true;
                    }

                    if self.inspector.enabled {
                        self.set_inspect_enabled(false, self.inspector.consume_clicks);

                        let _ = write_json(
                            self.cfg.inspect_path.clone(),
                            &UiInspectConfigV1 {
                                schema_version: 1,
                                enabled: false,
                                consume_clicks: self.inspector.consume_clicks,
                            },
                        );
                        let _ = touch_file(&self.cfg.inspect_trigger_path);

                        self.push_inspect_toast(window, "inspect: disabled".to_string());
                        app.request_redraw(window);
                        return true;
                    }
                    false
                }
                KeyCode::KeyL => {
                    if self.inspector.state.locked_windows.remove(&window) {
                        self.inspector.state.focus_down_stack.remove(&window);
                        self.push_inspect_toast(window, "inspect: unlocked".to_string());
                    } else if let Some(hovered) =
                        self.inspector.last_hovered_node_id.get(&window).copied()
                    {
                        self.inspector.last_picked_node_id.insert(window, hovered);
                        if let Some(sel) = self
                            .inspector
                            .last_hovered_selector_json
                            .get(&window)
                            .cloned()
                        {
                            self.inspector.last_picked_selector_json.insert(window, sel);
                        }
                        self.inspector.state.focus_node_id.insert(window, hovered);
                        if let Some(sel) = self
                            .inspector
                            .last_hovered_selector_json
                            .get(&window)
                            .cloned()
                        {
                            self.inspector.state.focus_selector_json.insert(window, sel);
                        }
                        self.inspector
                            .state
                            .focus_down_stack
                            .insert(window, Vec::new());
                        self.inspector.state.locked_windows.insert(window);
                        self.push_inspect_toast(window, "inspect: locked selection".to_string());
                    } else {
                        self.push_inspect_toast(window, "inspect: nothing to lock".to_string());
                    }
                    app.request_redraw(window);
                    true
                }
                KeyCode::KeyC => {
                    let wants_copy = wants_command;
                    if !wants_copy {
                        return false;
                    }
                    if modifiers.shift {
                        self.inspector
                            .state
                            .pending_copy_details_windows
                            .insert(window);
                        self.push_inspect_toast(window, "inspect: copy requested".to_string());
                        app.request_redraw(window);
                        return true;
                    }

                    let Some(payload) = self
                        .inspect_best_selector_json(window)
                        .map(|s| s.to_string())
                    else {
                        self.push_inspect_toast(window, "inspect: no selector to copy".to_string());
                        app.request_redraw(window);
                        return true;
                    };
                    app.push_effect(Effect::ClipboardSetText { text: payload });
                    self.push_inspect_toast(window, "inspect: copied selector".to_string());
                    app.request_redraw(window);
                    true
                }
                KeyCode::KeyF => {
                    if !self.inspector.enabled {
                        return false;
                    }
                    self.inspector
                        .state
                        .pending_nav
                        .insert(window, InspectNavCommand::Focus);
                    self.push_inspect_toast(window, "inspect: select focused node".to_string());
                    app.request_redraw(window);
                    true
                }
                KeyCode::ArrowUp => {
                    if !modifiers.alt {
                        return false;
                    }
                    if !self.inspect_is_locked(window) {
                        self.push_inspect_toast(
                            window,
                            "inspect: lock selection first (press L)".to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                    self.inspector
                        .state
                        .pending_nav
                        .insert(window, InspectNavCommand::Up);
                    app.request_redraw(window);
                    true
                }
                KeyCode::ArrowDown => {
                    if !modifiers.alt {
                        return false;
                    }
                    if !self.inspect_is_locked(window) {
                        self.push_inspect_toast(
                            window,
                            "inspect: lock selection first (press L)".to_string(),
                        );
                        app.request_redraw(window);
                        return true;
                    }
                    self.inspector
                        .state
                        .pending_nav
                        .insert(window, InspectNavCommand::Down);
                    app.request_redraw(window);
                    true
                }
                _ => false,
            }
        }
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

        self.poll_pick_trigger();
        self.poll_inspect_trigger();

        let Event::Pointer(PointerEvent::Down { position, .. }) = event else {
            return false;
        };

        if let Some(run_id) = self.inspector.pick_armed_run_id.take() {
            self.inspector.pending_pick = Some(PendingPick {
                run_id,
                window,
                position: *position,
            });
            app.request_redraw(window);
            return true;
        }

        if !self.inspector.enabled {
            return false;
        }

        let run_id = self.next_pick_run_id();

        self.inspector.pending_pick = Some(PendingPick {
            run_id,
            window,
            position: *position,
        });
        app.request_redraw(window);
        self.inspector.consume_clicks
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
        if !self.inspector.enabled {
            return;
        }
        let Some(snapshot) = snapshot else {
            return;
        };
        let Some(hovered_id) = hovered_node_id else {
            self.inspector.last_hovered_node_id.remove(&window);
            self.inspector.last_hovered_selector_json.remove(&window);
            return;
        };
        if self.inspect_is_locked(window) {
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
        let selector = best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            &self.cfg,
        )
        .or_else(|| best_selector_for_node(snapshot, node, element, &self.cfg));
        let Some(selector) = selector else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.inspector
                .last_hovered_node_id
                .insert(window, hovered_id);
            self.inspector
                .last_hovered_selector_json
                .insert(window, json);
            self.inspector
                .state
                .focus_node_id
                .insert(window, hovered_id);
            if let Some(sel) = self
                .inspector
                .last_hovered_selector_json
                .get(&window)
                .cloned()
            {
                self.inspector.state.focus_selector_json.insert(window, sel);
            }
            self.inspector
                .state
                .focus_down_stack
                .insert(window, Vec::new());
        }
    }

    fn push_inspect_toast(&mut self, window: AppWindowId, message: String) {
        self.inspector.state.toast.insert(
            window,
            InspectToast {
                message,
                remaining_frames: 90,
            },
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
        if !self.inspector.enabled {
            self.inspector.state.pending_nav.remove(&window);
            return;
        }
        let Some(cmd) = self.inspector.state.pending_nav.remove(&window) else {
            return;
        };
        let Some(snapshot) = snapshot else {
            self.push_inspect_toast(window, "inspect: no semantics snapshot".to_string());
            return;
        };

        match cmd {
            InspectNavCommand::Focus => {
                let Some(node) = snapshot.focus else {
                    self.push_inspect_toast(window, "inspect: no focused node".to_string());
                    return;
                };
                let id = node.data().as_ffi();
                self.inspector
                    .state
                    .focus_down_stack
                    .insert(window, Vec::new());
                self.inspector.state.locked_windows.insert(window);
                self.set_inspect_focus(window, snapshot, id, element_runtime);
            }
            InspectNavCommand::SelectNode(node_id) => {
                self.inspector
                    .state
                    .focus_down_stack
                    .insert(window, Vec::new());
                self.inspector.state.locked_windows.insert(window);
                self.set_inspect_focus(window, snapshot, node_id, element_runtime);
            }
            InspectNavCommand::Up => {
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }

                let current = self
                    .inspector
                    .state
                    .focus_node_id
                    .get(&window)
                    .copied()
                    .or_else(|| self.inspector.last_picked_node_id.get(&window).copied())
                    .or_else(|| self.inspector.last_hovered_node_id.get(&window).copied());
                let Some(current) = current else {
                    self.push_inspect_toast(window, "inspect: no focused node".to_string());
                    return;
                };

                let Some(parent) = parent_node_id(snapshot, current) else {
                    self.push_inspect_toast(window, "inspect: reached root".to_string());
                    return;
                };
                self.inspector
                    .state
                    .focus_down_stack
                    .entry(window)
                    .or_default()
                    .push(current);
                self.set_inspect_focus(window, snapshot, parent, element_runtime);
                self.push_inspect_toast(window, "inspect: parent".to_string());
            }
            InspectNavCommand::Down => {
                if !self.inspect_is_locked(window) {
                    self.push_inspect_toast(
                        window,
                        "inspect: lock selection first (press L)".to_string(),
                    );
                    return;
                }
                let Some(prev) = self
                    .inspector
                    .state
                    .focus_down_stack
                    .get_mut(&window)
                    .and_then(|s| s.pop())
                else {
                    self.push_inspect_toast(window, "inspect: no child history".to_string());
                    return;
                };
                self.set_inspect_focus(window, snapshot, prev, element_runtime);
                self.push_inspect_toast(window, "inspect: child".to_string());
            }
        }
    }

    fn set_inspect_focus(
        &mut self,
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
        let selector = best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            &self.cfg,
        )
        .or_else(|| best_selector_for_node(snapshot, node, element, &self.cfg));
        let Some(selector) = selector else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.inspector.state.focus_node_id.insert(window, node_id);
            self.inspector
                .state
                .focus_selector_json
                .insert(window, json.clone());
            self.inspector.last_picked_node_id.insert(window, node_id);
            self.inspector
                .last_picked_selector_json
                .insert(window, json);
        }
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
        let Some(snapshot) = snapshot else {
            self.inspector.state.focus_summary_line.remove(&window);
            self.inspector.state.focus_path_line.remove(&window);
            if self
                .inspector
                .state
                .pending_copy_details_windows
                .remove(&window)
            {
                self.push_inspect_toast(window, "inspect: no semantics snapshot".to_string());
            }
            return;
        };

        let node_id = self
            .inspector
            .state
            .focus_node_id
            .get(&window)
            .copied()
            .or_else(|| self.inspector.last_picked_node_id.get(&window).copied())
            .or_else(|| self.inspector.last_hovered_node_id.get(&window).copied());
        let Some(node_id) = node_id else {
            self.inspector.state.focus_summary_line.remove(&window);
            self.inspector.state.focus_path_line.remove(&window);
            if self
                .inspector
                .state
                .pending_copy_details_windows
                .remove(&window)
            {
                self.push_inspect_toast(window, "inspect: no focused node".to_string());
            }
            return;
        };

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            self.inspector.state.focus_summary_line.remove(&window);
            self.inspector.state.focus_path_line.remove(&window);
            if self
                .inspector
                .state
                .pending_copy_details_windows
                .remove(&window)
            {
                self.push_inspect_toast(window, "inspect: focused node missing".to_string());
            }
            return;
        };

        let role = semantics_role_label(node.role);
        let mut summary = format!("focus: {role} node={node_id}");

        if let Some(runtime) = element_runtime
            && let Some(element) = runtime.element_for_node(window, node.id)
        {
            summary.push_str(&format!(" element={}", element.0));
            if let Some(path) = runtime.debug_path_for_element(window, element) {
                let path = truncate_debug_value(&path, 200);
                summary.push_str(&format!(" element_path={path}"));
            }
        }
        if let Some(test_id) = node.test_id.as_deref() {
            summary.push_str(&format!(" test_id={test_id}"));
        }
        if !self.cfg.redact_text
            && let Some(label) = node.label.as_deref()
        {
            let label = truncate_debug_value(label, 120);
            summary.push_str(&format!(" label={label}"));
        }

        let path = format_inspect_path(snapshot, node_id, self.cfg.redact_text, 10);

        self.inspector
            .state
            .focus_summary_line
            .insert(window, summary);
        if let Some(path) = path {
            self.inspector.state.focus_path_line.insert(window, path);
        } else {
            self.inspector.state.focus_path_line.remove(&window);
        }

        if !self
            .inspector
            .state
            .pending_copy_details_windows
            .remove(&window)
        {
            return;
        }

        let element = element_runtime
            .and_then(|runtime| runtime.element_for_node(window, node.id))
            .map(|id| id.0);

        let best = best_selector_for_node_validated(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            &self.cfg,
        )
        .or_else(|| best_selector_for_node(snapshot, node, element, &self.cfg));

        let mut lines: Vec<String> = Vec::new();
        if let Some(best) = best.as_ref().and_then(|s| serde_json::to_string(s).ok()) {
            lines.push(format!("selector: {best}"));
        }
        if let Some(summary) = self.inspect_focus_summary_line(window) {
            lines.push(summary.to_string());
        }
        if let Some(path) = self.inspect_focus_path_line(window) {
            lines.push(path.to_string());
        }

        let report = inspect_selector_candidates_report(
            snapshot,
            window,
            element_runtime,
            node,
            element,
            &self.cfg,
        );
        if !report.trim().is_empty() {
            lines.push(String::new());
            lines.push("selector_candidates:".to_string());
            lines.extend(report.lines().map(|l: &str| l.to_string()));
        }

        let payload = lines.join("\n");
        if payload.trim().is_empty() {
            self.push_inspect_toast(window, "inspect: no details available".to_string());
            return;
        }

        self.inspector
            .state
            .pending_copy_details_payload
            .insert(window, payload);
        self.push_inspect_toast(window, "inspect: details copied".to_string());
    }
}
