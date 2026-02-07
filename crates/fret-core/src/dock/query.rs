use super::*;

impl DockGraph {
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
                let count = children.len().min(fractions.len());
                if count == 0 {
                    return;
                }

                let total: f32 = fractions.iter().take(count).sum();
                let total = if total <= 0.0 { 1.0 } else { total };

                let mut cursor = 0.0;
                for i in 0..count {
                    let f = fractions[i] / total;
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
        let root = self.window_root(window)?;
        self.find_panel_in_subtree(root, panel).or_else(|| {
            self.window_floatings.get(&window).and_then(|list| {
                list.iter()
                    .find_map(|w| self.find_panel_in_subtree(w.floating, panel))
            })
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
        let root = self.window_root(window)?;
        self.first_tabs_in_subtree(root)
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
