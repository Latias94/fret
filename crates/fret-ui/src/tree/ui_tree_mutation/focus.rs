use super::super::*;

impl<H: UiHost> UiTree<H> {
    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(parent)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub(crate) fn children_ref(&self, parent: NodeId) -> &[NodeId] {
        self.nodes
            .get(parent)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Count retained parent-pointer drift from layer-root child edges without mutating nodes.
    ///
    /// This is the deletion oracle for the removed normal repair pass. Normal runtime queries must
    /// use current child-edge topology instead of relying on this retained storage field.
    pub(crate) fn parent_pointers_would_repair_from_layer_roots(&self) -> u32 {
        let roots = self.all_layer_roots();
        if roots.is_empty() {
            return 0;
        }

        let mut would_repair: u32 = 0;
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(Option<NodeId>, NodeId)> = Vec::with_capacity(roots.len());
        for root in roots {
            stack.push((None, root));
        }

        while let Some((expected_parent, node)) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }

            let Some(n) = self.nodes.get(node) else {
                continue;
            };
            if n.parent != expected_parent {
                would_repair = would_repair.saturating_add(1);
            }

            for &child in &n.children {
                stack.push((Some(node), child));
            }
        }

        would_repair
    }

    #[cfg(test)]
    pub(crate) fn debug_node_parent_storage(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.parent)
    }

    pub fn debug_node_measured_size(&self, node: NodeId) -> Option<Size> {
        self.nodes.get(node).map(|n| n.measured_size)
    }

    /// Debug helper for mapping a `NodeId` back to the declarative `ElementInstance` kind (when
    /// the node is driven by the declarative renderer).
    pub fn debug_declarative_instance_kind(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<&'static str> {
        crate::declarative::element_record_for_node(app, window, node)
            .map(|record| record.instance.kind_name())
    }

    pub fn first_focusable_ancestor_including_declarative(
        &self,
        app: &mut H,
        window: AppWindowId,
        start: NodeId,
    ) -> Option<NodeId> {
        let mut node = Some(start);
        while let Some(id) = node {
            let focusable = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::TextInputRegion(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    _ => false,
                }
            } else {
                self.nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable())
            };

            if focusable {
                return Some(id);
            }

            node = self.parent_in_layer_forest_via_children(id);
        }
        None
    }

    pub fn first_focusable_descendant(&self, root: NodeId) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let focusable = self
                .nodes
                .get(id)
                .and_then(|n| n.widget.as_ref())
                .is_some_and(|w| w.is_focusable());
            if focusable {
                return Some(id);
            }

            if let Some(node) = self.nodes.get(id) {
                let traverse_children = node
                    .widget
                    .as_ref()
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                if traverse_children {
                    for &child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }
            }
        }
        None
    }

    /// Like `first_focusable_descendant`, but also considers declarative element instances that
    /// haven't run layout yet.
    ///
    /// This is needed because declarative nodes derive focusability from their element instance
    /// (`PressableProps.focusable`, `TextInput`, ...), and the `ElementHostWidget` only caches that
    /// information during layout. Overlay policies commonly want to set initial focus immediately
    /// after installing an overlay root, before layout runs.
    pub fn first_focusable_descendant_including_declarative(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
    ) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let (focusable, traverse_children) = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                let focusable = match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::TextInputRegion(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    crate::declarative::ElementInstance::Semantics(p) => {
                        p.focusable && !p.disabled && !p.hidden
                    }
                    _ => false,
                };
                let traverse_children = match &record.instance {
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled,
                    crate::declarative::ElementInstance::InteractivityGate(p) => {
                        p.present && p.interactive
                    }
                    crate::declarative::ElementInstance::Spinner(_) => false,
                    _ => true,
                };
                (focusable, traverse_children)
            } else {
                let traverse_children = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                let focusable = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable());
                (focusable, traverse_children)
            };

            if focusable {
                return Some(id);
            }

            if traverse_children && let Some(node) = self.nodes.get(id) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }

    /// Like `first_focusable_descendant_including_declarative`, but treats `InteractivityGate`
    /// as a *pointer/activation* gate, not a traversal boundary for initial focus.
    ///
    /// This is useful for overlay autofocus policies where content may be temporarily
    /// non-interactive (e.g. during motion) but still present and should be eligible for focus.
    pub fn first_focusable_descendant_including_declarative_present_only(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
    ) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let (focusable, traverse_children) = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                let focusable = match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::TextInputRegion(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    crate::declarative::ElementInstance::Semantics(p) => {
                        p.focusable && !p.disabled && !p.hidden
                    }
                    _ => false,
                };
                let traverse_children = match &record.instance {
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled,
                    crate::declarative::ElementInstance::InteractivityGate(p) => p.present,
                    crate::declarative::ElementInstance::Spinner(_) => false,
                    _ => true,
                };
                (focusable, traverse_children)
            } else {
                let traverse_children = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                let focusable = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable());
                (focusable, traverse_children)
            };

            if focusable {
                return Some(id);
            }

            if traverse_children && let Some(node) = self.nodes.get(id) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }
}
