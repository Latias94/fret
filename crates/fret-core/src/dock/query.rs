use super::*;

#[derive(Debug, Default)]
struct DockParentIndex {
    root_for: HashMap<DockNodeId, DockNodeId>,
    parent: HashMap<DockNodeId, DockNodeId>,
    split_child_index: HashMap<DockNodeId, usize>,
}

impl DockGraph {
    fn build_parent_index_for_window(&self, window: AppWindowId) -> DockParentIndex {
        fn index_subtree(graph: &DockGraph, root: DockNodeId, index: &mut DockParentIndex) {
            let mut stack: Vec<DockNodeId> = vec![root];
            while let Some(node) = stack.pop() {
                if index.root_for.contains_key(&node) {
                    continue;
                }
                index.root_for.insert(node, root);

                let Some(n) = graph.nodes.get(node) else {
                    continue;
                };
                match n {
                    DockNode::Tabs { .. } => {}
                    DockNode::Floating { child } => {
                        index.parent.insert(*child, node);
                        stack.push(*child);
                    }
                    DockNode::Split { children, .. } => {
                        for (i, child) in children.iter().copied().enumerate() {
                            index.parent.insert(child, node);
                            index.split_child_index.insert(child, i);
                            stack.push(child);
                        }
                    }
                }
            }
        }

        let mut index = DockParentIndex::default();
        if let Some(root) = self.window_root(window) {
            index_subtree(self, root, &mut index);
        }
        if let Some(list) = self.window_floatings.get(&window) {
            for w in list {
                index_subtree(self, w.floating, &mut index);
            }
        }
        index
    }

    /// Decide whether an edge-dock into `target` will insert into an existing same-axis split, or
    /// wrap the target in a new split.
    ///
    /// Returns `None` for `DropZone::Center` (not an edge dock) or if the target is not present in
    /// the window's dock forest.
    pub fn edge_dock_decision(
        &self,
        window: AppWindowId,
        target: DockNodeId,
        zone: DropZone,
    ) -> Option<EdgeDockDecision> {
        if zone == DropZone::Center {
            return None;
        }

        let axis = match zone {
            DropZone::Left | DropZone::Right => Axis::Horizontal,
            DropZone::Top | DropZone::Bottom => Axis::Vertical,
            DropZone::Center => unreachable!(),
        };

        // Index parent links for the window's docking forest so we can answer:
        // - is the target in the forest?
        // - what is the nearest same-axis split ancestor?
        //
        // This avoids repeated subtree scans in edge-dock hot paths.
        let index = self.build_parent_index_for_window(window);
        if !index.root_for.contains_key(&target) {
            return None;
        }

        // Outer docking can target an existing split container. In that case we can insert at the
        // boundary without searching for an ancestor.
        if let Some(DockNode::Split {
            axis: split_axis,
            children,
            fractions,
        }) = self.nodes.get(target)
            && *split_axis == axis
            && !children.is_empty()
            && children.len() == fractions.len()
        {
            let len = children.len();
            let (anchor_index, insert_index) = match zone {
                DropZone::Left | DropZone::Top => (0, 0),
                DropZone::Right | DropZone::Bottom => {
                    let last = len.saturating_sub(1);
                    (last, last.saturating_add(1))
                }
                DropZone::Center => unreachable!(),
            };
            return Some(EdgeDockDecision::InsertIntoSplit {
                split: target,
                anchor_index,
                insert_index,
            });
        }

        let mut cur = target;
        while let Some(parent) = index.parent.get(&cur).copied() {
            let Some(DockNode::Split {
                axis: split_axis,
                children,
                fractions,
            }) = self.nodes.get(parent)
            else {
                cur = parent;
                continue;
            };

            if *split_axis == axis && !children.is_empty() && children.len() == fractions.len() {
                let Some(anchor_index) = index.split_child_index.get(&cur).copied() else {
                    break;
                };

                let insert_index = match zone {
                    DropZone::Left | DropZone::Top => anchor_index,
                    DropZone::Right | DropZone::Bottom => anchor_index.saturating_add(1),
                    DropZone::Center => unreachable!(),
                };

                return Some(EdgeDockDecision::InsertIntoSplit {
                    split: parent,
                    anchor_index,
                    insert_index,
                });
            }

            cur = parent;
        }

        Some(EdgeDockDecision::WrapNewSplit)
    }

    pub fn compute_layout(
        &self,
        root: DockNodeId,
        bounds: Rect,
        out: &mut HashMap<DockNodeId, Rect>,
    ) {
        let Some(node) = self.nodes.get(root) else {
            return;
        };

        out.insert(root, bounds);
        match node {
            DockNode::Tabs { .. } => {}
            DockNode::Split {
                axis,
                children,
                fractions,
            } => {
                if children.is_empty() {
                    return;
                }

                // Layout should not silently truncate children if invariants are violated. If the
                // split is non-canonical (mismatched lengths, non-finite values), repair the shares
                // locally for deterministic layout.
                let cleaned_share_at = |i: usize| -> f32 {
                    let raw = fractions.get(i).copied().unwrap_or(1.0);
                    if raw.is_finite() && raw > 0.0 {
                        raw
                    } else {
                        0.0
                    }
                };

                let mut total = 0.0;
                for i in 0..children.len() {
                    total += cleaned_share_at(i);
                }
                let uniform = 1.0 / children.len() as f32;
                let inv_total = if total > 0.0 { 1.0 / total } else { 0.0 };

                let mut cursor = 0.0;
                for i in 0..children.len() {
                    let f = if total > 0.0 {
                        cleaned_share_at(i) * inv_total
                    } else {
                        uniform
                    };
                    let (child_rect, next_cursor) = match axis {
                        Axis::Horizontal => {
                            let w = bounds.size.width.0 * f;
                            let rect = Rect {
                                origin: Point::new(Px(bounds.origin.x.0 + cursor), bounds.origin.y),
                                size: Size::new(Px(w), bounds.size.height),
                            };
                            (rect, cursor + w)
                        }
                        Axis::Vertical => {
                            let h = bounds.size.height.0 * f;
                            let rect = Rect {
                                origin: Point::new(bounds.origin.x, Px(bounds.origin.y.0 + cursor)),
                                size: Size::new(bounds.size.width, Px(h)),
                            };
                            (rect, cursor + h)
                        }
                    };

                    cursor = next_cursor;
                    self.compute_layout(children[i], child_rect, out);
                }
            }
            DockNode::Floating { child } => {
                self.compute_layout(*child, bounds, out);
            }
        }
    }

    pub fn find_panel_in_window(
        &self,
        window: AppWindowId,
        panel: &PanelKey,
    ) -> Option<(DockNodeId, usize)> {
        if let Some(root) = self.window_root(window)
            && let Some(found) = self.find_panel_in_subtree(root, panel)
        {
            return Some(found);
        }

        self.window_floatings.get(&window).and_then(|list| {
            list.iter()
                .find_map(|w| self.find_panel_in_subtree(w.floating, panel))
        })
    }

    pub fn windows(&self) -> Vec<AppWindowId> {
        let mut windows: Vec<AppWindowId> = self.window_roots.keys().copied().collect();
        windows.sort_by_key(|w| w.data().as_ffi());
        windows
    }

    pub fn collect_panels_in_window(&self, window: AppWindowId) -> Vec<PanelKey> {
        let mut out: Vec<PanelKey> = Vec::new();
        if let Some(root) = self.window_root(window) {
            out.extend(self.collect_panels_in_subtree(root));
        }
        if let Some(list) = self.window_floatings.get(&window) {
            for w in list {
                out.extend(self.collect_panels_in_subtree(w.floating));
            }
        }
        out
    }

    pub fn first_tabs_in_window(&self, window: AppWindowId) -> Option<DockNodeId> {
        if let Some(root) = self.window_root(window)
            && let Some(tabs) = self.first_tabs_in_subtree(root)
        {
            return Some(tabs);
        }

        self.window_floatings.get(&window).and_then(|list| {
            list.iter()
                .find_map(|w| self.first_tabs_in_subtree(w.floating))
        })
    }

    pub(super) fn collect_panels_in_subtree(&self, node: DockNodeId) -> Vec<PanelKey> {
        let Some(n) = self.nodes.get(node) else {
            return Vec::new();
        };
        match n {
            DockNode::Tabs { tabs, .. } => tabs.clone(),
            DockNode::Split { children, .. } => {
                let mut out = Vec::new();
                for child in children {
                    out.extend(self.collect_panels_in_subtree(*child));
                }
                out
            }
            DockNode::Floating { child } => self.collect_panels_in_subtree(*child),
        }
    }

    fn first_tabs_in_subtree(&self, node: DockNodeId) -> Option<DockNodeId> {
        let n = self.nodes.get(node)?;
        match n {
            DockNode::Tabs { .. } => Some(node),
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .find_map(|child| self.first_tabs_in_subtree(child)),
            DockNode::Floating { child } => self.first_tabs_in_subtree(*child),
        }
    }

    fn find_panel_in_subtree(
        &self,
        node: DockNodeId,
        panel: &PanelKey,
    ) -> Option<(DockNodeId, usize)> {
        let n = self.nodes.get(node)?;
        match n {
            DockNode::Tabs { tabs, .. } => tabs.iter().position(|p| p == panel).map(|i| (node, i)),
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .find_map(|child| self.find_panel_in_subtree(child, panel)),
            DockNode::Floating { child } => self.find_panel_in_subtree(*child, panel),
        }
    }
}
