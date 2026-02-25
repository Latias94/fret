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

    /// Best-effort repair pass for parent pointers based on child edges from layer roots.
    ///
    /// Parent pointers are used for cache-root discovery (`nearest_view_cache_root`) and for
    /// determining whether nodes are attached to any layer (`node_layer`). If a bug or GC edge
    /// case leaves a reachable node with a missing/incorrect `parent`, this can cascade into
    /// incorrect invalidation truncation and overly-aggressive subtree sweeping.
    ///
    /// This intentionally only walks nodes reachable from the installed layer roots; it does not
    /// attempt to "rescue" detached islands.
    pub(crate) fn repair_parent_pointers_from_layer_roots(&mut self) -> u32 {
        let roots = self.all_layer_roots();
        if roots.is_empty() {
            return 0;
        }

        let mut repaired: u32 = 0;
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(Option<NodeId>, NodeId)> = Vec::with_capacity(roots.len());
        for root in roots {
            stack.push((None, root));
        }

        while let Some((expected_parent, node)) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }

            let (current_parent, children) = match self.nodes.get(node) {
                Some(n) => (n.parent, n.children.clone()),
                None => continue,
            };

            if current_parent != expected_parent
                && let Some(n) = self.nodes.get_mut(node)
            {
                n.parent = expected_parent;
                repaired = repaired.saturating_add(1);
            }

            for child in children {
                stack.push((Some(node), child));
            }
        }

        repaired
    }

    pub fn node_parent(&self, node: NodeId) -> Option<NodeId> {
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

            node = self.nodes.get(id).and_then(|n| n.parent);
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
