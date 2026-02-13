use super::*;

impl DockGraph {
    pub fn export_layout(&self, windows: &[(AppWindowId, String)]) -> crate::DockLayout {
        self.export_layout_with_placement(windows, |_| None)
    }

    pub fn export_layout_with_placement(
        &self,
        windows: &[(AppWindowId, String)],
        mut placement: impl FnMut(AppWindowId) -> Option<crate::DockWindowPlacement>,
    ) -> crate::DockLayout {
        use crate::{DockLayoutFloatingWindow, DockLayoutNode, DockLayoutWindow};
        use std::collections::HashMap;

        fn visit(
            graph: &DockGraph,
            node: DockNodeId,
            next_id: &mut u32,
            ids: &mut HashMap<DockNodeId, u32>,
            out: &mut Vec<DockLayoutNode>,
        ) {
            if ids.contains_key(&node) {
                return;
            }

            let id = *next_id;
            *next_id = next_id.saturating_add(1);
            ids.insert(node, id);

            let Some(n) = graph.nodes.get(node) else {
                return;
            };

            match n {
                DockNode::Tabs { tabs, active } => {
                    out.push(DockLayoutNode::Tabs {
                        id,
                        tabs: tabs.clone(),
                        active: *active,
                    });
                }
                DockNode::Split {
                    axis,
                    children,
                    fractions,
                } => {
                    for child in children {
                        visit(graph, *child, next_id, ids, out);
                    }
                    let child_ids: Vec<u32> = children
                        .iter()
                        .filter_map(|c| ids.get(c).copied())
                        .collect();
                    out.push(DockLayoutNode::Split {
                        id,
                        axis: *axis,
                        children: child_ids,
                        fractions: fractions.clone(),
                    });
                }
                DockNode::Floating { child } => {
                    visit(graph, *child, next_id, ids, out);
                    if let Some(&child_id) = ids.get(child) {
                        ids.insert(node, child_id);
                    }
                }
            }
        }

        let mut next_id: u32 = 1;
        let mut ids: HashMap<DockNodeId, u32> = HashMap::new();
        let mut nodes: Vec<DockLayoutNode> = Vec::new();
        let mut out_windows: Vec<DockLayoutWindow> = Vec::new();

        for (window, logical_window_id) in windows {
            let Some(root) = self.window_root(*window) else {
                continue;
            };
            visit(self, root, &mut next_id, &mut ids, &mut nodes);
            let Some(root_id) = ids.get(&root).copied() else {
                continue;
            };

            let mut floatings: Vec<DockLayoutFloatingWindow> = Vec::new();
            for floating in self.floating_windows(*window) {
                visit(self, floating.floating, &mut next_id, &mut ids, &mut nodes);
                let Some(floating_root) = ids.get(&floating.floating).copied() else {
                    continue;
                };
                floatings.push(DockLayoutFloatingWindow {
                    root: floating_root,
                    rect: crate::DockRect::from_rect(floating.rect),
                });
            }

            out_windows.push(DockLayoutWindow {
                logical_window_id: logical_window_id.clone(),
                root: root_id,
                placement: placement(*window),
                floatings,
            });
        }

        crate::DockLayout::new(out_windows, nodes)
    }

    pub fn import_subtree_from_layout(
        &mut self,
        layout: &crate::DockLayout,
        root: u32,
    ) -> Option<DockNodeId> {
        use crate::DockLayoutNode;
        use std::collections::HashMap;

        if layout.layout_version != crate::DOCK_LAYOUT_VERSION {
            return None;
        }

        let mut by_id: HashMap<u32, &DockLayoutNode> = HashMap::new();
        for node in &layout.nodes {
            let id = match node {
                DockLayoutNode::Split { id, .. } => *id,
                DockLayoutNode::Tabs { id, .. } => *id,
            };
            by_id.insert(id, node);
        }

        fn build(
            graph: &mut DockGraph,
            by_id: &HashMap<u32, &DockLayoutNode>,
            id: u32,
            visiting: &mut HashMap<u32, ()>,
        ) -> Option<DockNodeId> {
            if visiting.contains_key(&id) {
                return None;
            }
            visiting.insert(id, ());

            let node = by_id.get(&id)?;
            let out = match node {
                DockLayoutNode::Tabs { tabs, active, .. } => {
                    Some(graph.insert_node(DockNode::Tabs {
                        tabs: tabs.clone(),
                        active: *active,
                    }))
                }
                DockLayoutNode::Split {
                    axis,
                    children,
                    fractions,
                    ..
                } => {
                    let mut child_nodes: Vec<DockNodeId> = Vec::new();
                    for child in children {
                        child_nodes.push(build(graph, by_id, *child, visiting)?);
                    }
                    Some(graph.insert_node(DockNode::Split {
                        axis: *axis,
                        children: child_nodes,
                        fractions: fractions.clone(),
                    }))
                }
            };

            visiting.remove(&id);
            out
        }

        let mut visiting: HashMap<u32, ()> = HashMap::new();
        build(self, &by_id, root, &mut visiting)
    }

    pub fn import_subtree_from_layout_checked(
        &mut self,
        layout: &crate::DockLayout,
        root: u32,
    ) -> Result<DockNodeId, crate::DockLayoutValidationError> {
        use crate::DockLayoutNode;
        use std::collections::HashMap;

        layout.validate()?;

        let mut by_id: HashMap<u32, &DockLayoutNode> = HashMap::new();
        for node in &layout.nodes {
            let id = match node {
                DockLayoutNode::Split { id, .. } => *id,
                DockLayoutNode::Tabs { id, .. } => *id,
            };
            by_id.insert(id, node);
        }

        let mut built: HashMap<u32, DockNodeId> = HashMap::new();
        fn build_checked(
            graph: &mut DockGraph,
            by_id: &HashMap<u32, &DockLayoutNode>,
            built: &mut HashMap<u32, DockNodeId>,
            id: u32,
        ) -> DockNodeId {
            if let Some(&node) = built.get(&id) {
                return node;
            }

            let node = by_id
                .get(&id)
                .copied()
                .expect("layout.validate ensures node id exists");

            let out = match node {
                DockLayoutNode::Tabs { tabs, active, .. } => graph.insert_node(DockNode::Tabs {
                    tabs: tabs.clone(),
                    active: *active,
                }),
                DockLayoutNode::Split {
                    axis,
                    children,
                    fractions,
                    ..
                } => {
                    let mut child_nodes: Vec<DockNodeId> = Vec::new();
                    for child in children {
                        child_nodes.push(build_checked(graph, by_id, built, *child));
                    }
                    graph.insert_node(DockNode::Split {
                        axis: *axis,
                        children: child_nodes,
                        fractions: fractions.clone(),
                    })
                }
            };

            built.insert(id, out);
            out
        }

        if !by_id.contains_key(&root) {
            return Err(crate::DockLayoutValidationError {
                kind: crate::DockLayoutValidationErrorKind::MissingNodeId { id: root },
            });
        }

        Ok(build_checked(self, &by_id, &mut built, root))
    }

    pub fn import_layout_for_windows(
        &mut self,
        layout: &crate::DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> bool {
        use std::collections::HashMap;

        if layout.layout_version != crate::DOCK_LAYOUT_VERSION {
            return false;
        }

        let mut by_logical: HashMap<&str, AppWindowId> = HashMap::new();
        for (window, logical_id) in windows {
            by_logical.insert(logical_id.as_str(), *window);
        }

        let mut imported_any = false;
        for w in &layout.windows {
            let Some(window) = by_logical.get(w.logical_window_id.as_str()).copied() else {
                continue;
            };

            let Some(root) = self.import_subtree_from_layout(layout, w.root) else {
                continue;
            };
            self.set_window_root(window, root);

            self.floating_windows_mut(window).clear();
            for f in &w.floatings {
                let Some(child) = self.import_subtree_from_layout(layout, f.root) else {
                    continue;
                };
                let floating = self.insert_node(DockNode::Floating { child });
                self.floating_windows_mut(window).push(DockFloatingWindow {
                    floating,
                    rect: f.rect.to_rect(),
                });
            }

            self.simplify_window_forest(window);
            imported_any = true;
        }

        imported_any
    }

    pub fn import_layout_for_windows_checked(
        &mut self,
        layout: &crate::DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> Result<bool, crate::DockLayoutValidationError> {
        use std::collections::HashMap;

        layout.validate()?;

        let mut by_logical: HashMap<&str, AppWindowId> = HashMap::new();
        for (window, logical_id) in windows {
            by_logical.insert(logical_id.as_str(), *window);
        }

        let mut imported_any = false;
        for w in &layout.windows {
            let Some(window) = by_logical.get(w.logical_window_id.as_str()).copied() else {
                continue;
            };

            let root = self.import_subtree_from_layout_checked(layout, w.root)?;
            self.set_window_root(window, root);

            self.floating_windows_mut(window).clear();
            for f in &w.floatings {
                let child = self.import_subtree_from_layout_checked(layout, f.root)?;
                let floating = self.insert_node(DockNode::Floating { child });
                self.floating_windows_mut(window).push(DockFloatingWindow {
                    floating,
                    rect: f.rect.to_rect(),
                });
            }

            self.simplify_window_forest(window);
            imported_any = true;
        }

        Ok(imported_any)
    }

    /// Import a dock layout for a set of known windows, degrading any unmapped logical windows
    /// into in-window floating containers inside `fallback_window`.
    ///
    /// This enables loading multi-window layouts on platforms that do not support multiple OS
    /// windows (wasm/mobile). The extra logical windows become floating dock containers rendered
    /// by the dock host in `fallback_window`.
    pub fn import_layout_for_windows_with_fallback_floatings(
        &mut self,
        layout: &crate::DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> bool {
        use std::collections::HashMap;

        if layout.layout_version != crate::DOCK_LAYOUT_VERSION {
            return false;
        }

        fn offset_rect(rect: Rect, delta: Point) -> Rect {
            Rect::new(
                Point::new(
                    Px(rect.origin.x.0 + delta.x.0),
                    Px(rect.origin.y.0 + delta.y.0),
                ),
                rect.size,
            )
        }

        fn rect_for_unmapped_window(w: &crate::DockLayoutWindow, index: usize) -> Rect {
            let default_w = 640.0;
            let default_h = 480.0;
            let (w_px, h_px) = w
                .placement
                .as_ref()
                .map(|p| (p.width as f32, p.height as f32))
                .unwrap_or((default_w, default_h));

            let width = w_px.clamp(240.0, 1400.0);
            let height = h_px.clamp(180.0, 1000.0);

            let stagger = (index as f32).min(12.0) * 24.0;
            Rect::new(
                Point::new(Px(32.0 + stagger), Px(32.0 + stagger)),
                Size::new(Px(width), Px(height)),
            )
        }

        let mut by_logical: HashMap<&str, AppWindowId> = HashMap::new();
        for (window, logical_id) in windows {
            by_logical.insert(logical_id.as_str(), *window);
        }

        // Clear and re-import all floating windows for `fallback_window` so the resulting state is
        // deterministic when this method is used as a "load persisted layout" entry point.
        self.floating_windows_mut(fallback_window).clear();

        let mut imported_any = false;
        let mut unmapped_index: usize = 0;

        for w in &layout.windows {
            if let Some(window) = by_logical.get(w.logical_window_id.as_str()).copied() {
                let Some(root) = self.import_subtree_from_layout(layout, w.root) else {
                    continue;
                };
                self.set_window_root(window, root);

                self.floating_windows_mut(window).clear();
                for f in &w.floatings {
                    let Some(child) = self.import_subtree_from_layout(layout, f.root) else {
                        continue;
                    };
                    let floating = self.insert_node(DockNode::Floating { child });
                    self.floating_windows_mut(window).push(DockFloatingWindow {
                        floating,
                        rect: f.rect.to_rect(),
                    });
                }

                self.simplify_window_forest(window);
                imported_any = true;
                continue;
            }

            // Unmapped logical window: import it as a floating container inside `fallback_window`.
            let Some(child) = self.import_subtree_from_layout(layout, w.root) else {
                continue;
            };
            let window_rect = rect_for_unmapped_window(w, unmapped_index);
            let floating = self.insert_node(DockNode::Floating { child });
            self.floating_windows_mut(fallback_window)
                .push(DockFloatingWindow {
                    floating,
                    rect: window_rect,
                });

            for f in &w.floatings {
                let Some(child) = self.import_subtree_from_layout(layout, f.root) else {
                    continue;
                };
                let floating = self.insert_node(DockNode::Floating { child });
                self.floating_windows_mut(fallback_window)
                    .push(DockFloatingWindow {
                        floating,
                        rect: offset_rect(f.rect.to_rect(), window_rect.origin),
                    });
            }

            unmapped_index = unmapped_index.saturating_add(1);
            imported_any = true;
        }

        self.simplify_window_forest(fallback_window);
        imported_any
    }

    pub fn import_layout_for_windows_with_fallback_floatings_checked(
        &mut self,
        layout: &crate::DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> Result<bool, crate::DockLayoutValidationError> {
        use std::collections::HashMap;

        layout.validate()?;

        fn offset_rect(rect: Rect, delta: Point) -> Rect {
            Rect::new(
                Point::new(
                    Px(rect.origin.x.0 + delta.x.0),
                    Px(rect.origin.y.0 + delta.y.0),
                ),
                rect.size,
            )
        }

        fn rect_for_unmapped_window(w: &crate::DockLayoutWindow, index: usize) -> Rect {
            let default_w = 640.0;
            let default_h = 480.0;
            let (w_px, h_px) = w
                .placement
                .as_ref()
                .map(|p| (p.width as f32, p.height as f32))
                .unwrap_or((default_w, default_h));

            let width = w_px.clamp(240.0, 1400.0);
            let height = h_px.clamp(180.0, 1000.0);

            let stagger = (index as f32).min(12.0) * 24.0;
            Rect::new(
                Point::new(Px(32.0 + stagger), Px(32.0 + stagger)),
                Size::new(Px(width), Px(height)),
            )
        }

        let mut by_logical: HashMap<&str, AppWindowId> = HashMap::new();
        for (window, logical_id) in windows {
            by_logical.insert(logical_id.as_str(), *window);
        }

        self.floating_windows_mut(fallback_window).clear();

        let mut imported_any = false;
        let mut unmapped_index: usize = 0;

        for w in &layout.windows {
            if let Some(window) = by_logical.get(w.logical_window_id.as_str()).copied() {
                let root = self.import_subtree_from_layout_checked(layout, w.root)?;
                self.set_window_root(window, root);

                self.floating_windows_mut(window).clear();
                for f in &w.floatings {
                    let child = self.import_subtree_from_layout_checked(layout, f.root)?;
                    let floating = self.insert_node(DockNode::Floating { child });
                    self.floating_windows_mut(window).push(DockFloatingWindow {
                        floating,
                        rect: f.rect.to_rect(),
                    });
                }

                self.simplify_window_forest(window);
                imported_any = true;
                continue;
            }

            let child = self.import_subtree_from_layout_checked(layout, w.root)?;
            let window_rect = rect_for_unmapped_window(w, unmapped_index);
            let floating = self.insert_node(DockNode::Floating { child });
            self.floating_windows_mut(fallback_window)
                .push(DockFloatingWindow {
                    floating,
                    rect: window_rect,
                });

            for f in &w.floatings {
                let child = self.import_subtree_from_layout_checked(layout, f.root)?;
                let floating = self.insert_node(DockNode::Floating { child });
                self.floating_windows_mut(fallback_window)
                    .push(DockFloatingWindow {
                        floating,
                        rect: offset_rect(f.rect.to_rect(), window_rect.origin),
                    });
            }

            unmapped_index = unmapped_index.saturating_add(1);
            imported_any = true;
        }

        self.simplify_window_forest(fallback_window);
        Ok(imported_any)
    }
}
