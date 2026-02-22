use super::*;

impl UiDiagnosticsService {
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
        if !self.inspect_enabled {
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
        let Some(selector) = best_selector_for_node(snapshot, node, element, &self.cfg) else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.last_hovered_node_id.insert(window, hovered_id);
            self.last_hovered_selector_json.insert(window, json);
            self.inspect_focus_node_id.insert(window, hovered_id);
            if let Some(sel) = self.last_hovered_selector_json.get(&window).cloned() {
                self.inspect_focus_selector_json.insert(window, sel);
            }
            self.inspect_focus_down_stack.insert(window, Vec::new());
        }
    }

    pub(super) fn push_inspect_toast(&mut self, window: AppWindowId, message: String) {
        self.inspect_toast.insert(
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
        if !self.inspect_enabled {
            self.inspect_pending_nav.remove(&window);
            return;
        }
        let Some(cmd) = self.inspect_pending_nav.remove(&window) else {
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
                self.inspect_focus_down_stack.insert(window, Vec::new());
                self.inspect_locked_windows.insert(window);
                self.set_inspect_focus(window, snapshot, id, element_runtime);
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
                    .inspect_focus_node_id
                    .get(&window)
                    .copied()
                    .or_else(|| self.last_picked_node_id.get(&window).copied())
                    .or_else(|| self.last_hovered_node_id.get(&window).copied());
                let Some(current) = current else {
                    self.push_inspect_toast(window, "inspect: no focused node".to_string());
                    return;
                };

                let Some(parent) = parent_node_id(snapshot, current) else {
                    self.push_inspect_toast(window, "inspect: reached root".to_string());
                    return;
                };
                self.inspect_focus_down_stack
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
                    .inspect_focus_down_stack
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
        let Some(selector) = best_selector_for_node(snapshot, node, element, &self.cfg) else {
            return;
        };
        if let Ok(json) = serde_json::to_string(&selector) {
            self.inspect_focus_node_id.insert(window, node_id);
            self.inspect_focus_selector_json
                .insert(window, json.clone());
            self.last_picked_node_id.insert(window, node_id);
            self.last_picked_selector_json.insert(window, json);
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
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
            return;
        };

        let node_id = self
            .inspect_focus_node_id
            .get(&window)
            .copied()
            .or_else(|| self.last_picked_node_id.get(&window).copied())
            .or_else(|| self.last_hovered_node_id.get(&window).copied());
        let Some(node_id) = node_id else {
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
            return;
        };

        let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == node_id)
        else {
            self.inspect_focus_summary_line.remove(&window);
            self.inspect_focus_path_line.remove(&window);
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

        self.inspect_focus_summary_line.insert(window, summary);
        if let Some(path) = path {
            self.inspect_focus_path_line.insert(window, path);
        } else {
            self.inspect_focus_path_line.remove(&window);
        }
    }

    pub(super) fn inspect_copy_details_payload(&self, window: AppWindowId) -> String {
        let selector = self.inspect_best_selector_json(window);
        let summary = self.inspect_focus_summary_line(window);
        let path = self.inspect_focus_path_line(window);

        let mut lines: Vec<String> = Vec::new();
        if let Some(selector) = selector {
            lines.push(format!("selector: {selector}"));
        }
        if let Some(summary) = summary {
            lines.push(summary.to_string());
        }
        if let Some(path) = path {
            lines.push(path.to_string());
        }
        lines.join("\n")
    }
}
