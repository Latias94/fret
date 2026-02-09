use super::*;

impl DockGraph {
    pub fn move_panel(
        &mut self,
        window: AppWindowId,
        panel: PanelKey,
        target_tabs: DockNodeId,
        zone: DropZone,
    ) -> bool {
        self.move_panel_ex(window, panel, target_tabs, zone, None)
    }

    pub fn move_panel_ex(
        &mut self,
        window: AppWindowId,
        panel: PanelKey,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    ) -> bool {
        self.move_panel_between_windows(window, panel, window, target_tabs, zone, insert_index)
    }

    pub fn move_panel_between_windows(
        &mut self,
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    ) -> bool {
        let Some((source_tabs, source_index)) = self.find_panel_in_window(source_window, &panel)
        else {
            return false;
        };

        if zone == DropZone::Center
            && source_window == target_window
            && source_tabs == target_tabs
            && insert_index.is_none()
        {
            return true;
        }

        if !self.remove_panel_from_tabs(source_tabs, source_index) {
            return false;
        }

        if zone == DropZone::Center {
            let mut index = insert_index;
            if source_window == target_window
                && source_tabs == target_tabs
                && let Some(i) = index.as_mut()
                && *i > source_index
            {
                *i = i.saturating_sub(1);
            }

            let ok = self.insert_panel_into_tabs_at(target_tabs, panel, index);
            self.collapse_empty_tabs_upwards(source_window, source_tabs);
            self.remove_empty_floating_windows(source_window);
            return ok;
        }

        let axis = match zone {
            DropZone::Left | DropZone::Right => Axis::Horizontal,
            DropZone::Top | DropZone::Bottom => Axis::Vertical,
            DropZone::Center => unreachable!(),
        };

        let new_tabs = self.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });

        let (first, second) = match zone {
            DropZone::Left | DropZone::Top => (new_tabs, target_tabs),
            DropZone::Right | DropZone::Bottom => (target_tabs, new_tabs),
            DropZone::Center => unreachable!(),
        };

        let split = self.insert_node(DockNode::Split {
            axis,
            children: vec![first, second],
            fractions: vec![0.5, 0.5],
        });

        self.replace_node_in_window_tree(target_window, target_tabs, split);
        self.collapse_empty_tabs_upwards(source_window, source_tabs);
        self.remove_empty_floating_windows(source_window);
        true
    }

    pub fn move_tabs_between_windows(
        &mut self,
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    ) -> bool {
        if zone == DropZone::Center && source_window == target_window && source_tabs == target_tabs
        {
            return true;
        }

        if self
            .root_for_node_in_window_forest(source_window, source_tabs)
            .is_none()
        {
            return false;
        }
        if self
            .root_for_node_in_window_forest(target_window, target_tabs)
            .is_none()
        {
            return false;
        }

        let (panels, active) = match self.nodes.get(source_tabs) {
            Some(DockNode::Tabs { tabs, active }) if !tabs.is_empty() => (tabs.clone(), *active),
            _ => return false,
        };
        let active = active.min(panels.len().saturating_sub(1));

        if zone == DropZone::Center
            && !matches!(self.nodes.get(target_tabs), Some(DockNode::Tabs { .. }))
        {
            return false;
        }

        if let Some(DockNode::Tabs { tabs, active }) = self.nodes.get_mut(source_tabs) {
            tabs.clear();
            *active = 0;
        }
        if self.window_root(source_window) == Some(source_tabs) {
            let _ = self.remove_window_root(source_window);
        }
        self.collapse_empty_tabs_upwards(source_window, source_tabs);
        self.remove_empty_floating_windows(source_window);

        if zone == DropZone::Center {
            let ok = self.insert_panels_into_tabs_at(target_tabs, &panels, insert_index, active);
            self.remove_empty_floating_windows(target_window);
            return ok;
        }

        let axis = match zone {
            DropZone::Left | DropZone::Right => Axis::Horizontal,
            DropZone::Top | DropZone::Bottom => Axis::Vertical,
            DropZone::Center => unreachable!(),
        };

        let new_tabs = self.insert_node(DockNode::Tabs {
            tabs: panels,
            active,
        });

        let (first, second) = match zone {
            DropZone::Left | DropZone::Top => (new_tabs, target_tabs),
            DropZone::Right | DropZone::Bottom => (target_tabs, new_tabs),
            DropZone::Center => unreachable!(),
        };

        let split = self.insert_node(DockNode::Split {
            axis,
            children: vec![first, second],
            fractions: vec![0.5, 0.5],
        });

        self.replace_node_in_window_tree(target_window, target_tabs, split);
        self.remove_empty_floating_windows(target_window);
        true
    }

    pub fn close_panel(&mut self, window: AppWindowId, panel: PanelKey) -> bool {
        let Some((tabs, index)) = self.find_panel_in_window(window, &panel) else {
            return false;
        };
        if !self.remove_panel_from_tabs(tabs, index) {
            return false;
        }
        self.collapse_empty_tabs_upwards(window, tabs);
        self.remove_empty_floating_windows(window);
        true
    }

    pub fn float_panel_to_window(
        &mut self,
        source_window: AppWindowId,
        panel: PanelKey,
        new_window: AppWindowId,
    ) -> bool {
        let Some((source_tabs, source_index)) = self.find_panel_in_window(source_window, &panel)
        else {
            return false;
        };
        if !self.remove_panel_from_tabs(source_tabs, source_index) {
            return false;
        }

        let tabs = self.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        self.set_window_root(new_window, tabs);
        self.collapse_empty_tabs_upwards(source_window, source_tabs);
        self.remove_empty_floating_windows(source_window);
        true
    }

    pub fn float_panel_in_window(
        &mut self,
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        rect: Rect,
    ) -> bool {
        let Some((source_tabs, source_index)) = self.find_panel_in_window(source_window, &panel)
        else {
            return false;
        };
        if !self.remove_panel_from_tabs(source_tabs, source_index) {
            return false;
        }

        let tabs = self.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        let floating = self.insert_node(DockNode::Floating { child: tabs });
        self.floating_windows_mut(target_window)
            .push(DockFloatingWindow { floating, rect });

        self.collapse_empty_tabs_upwards(source_window, source_tabs);
        self.remove_empty_floating_windows(source_window);
        true
    }

    pub fn float_tabs_in_window(
        &mut self,
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
        rect: Rect,
    ) -> bool {
        if self
            .root_for_node_in_window_forest(source_window, source_tabs)
            .is_none()
        {
            return false;
        }

        let (panels, active) = match self.nodes.get(source_tabs) {
            Some(DockNode::Tabs { tabs, active }) if !tabs.is_empty() => (tabs.clone(), *active),
            _ => return false,
        };
        let active = active.min(panels.len().saturating_sub(1));

        if let Some(DockNode::Tabs { tabs, active }) = self.nodes.get_mut(source_tabs) {
            tabs.clear();
            *active = 0;
        }
        if self.window_root(source_window) == Some(source_tabs) {
            let _ = self.remove_window_root(source_window);
        }
        self.collapse_empty_tabs_upwards(source_window, source_tabs);
        self.remove_empty_floating_windows(source_window);

        let tabs = self.insert_node(DockNode::Tabs {
            tabs: panels,
            active,
        });
        let floating = self.insert_node(DockNode::Floating { child: tabs });
        self.floating_windows_mut(target_window)
            .push(DockFloatingWindow { floating, rect });
        self.remove_empty_floating_windows(target_window);
        true
    }

    pub fn set_floating_rect(
        &mut self,
        window: AppWindowId,
        floating: DockNodeId,
        rect: Rect,
    ) -> bool {
        let Some(list) = self.window_floatings.get_mut(&window) else {
            return false;
        };
        let Some(entry) = list.iter_mut().find(|w| w.floating == floating) else {
            return false;
        };
        entry.rect = rect;
        true
    }

    pub fn raise_floating(&mut self, window: AppWindowId, floating: DockNodeId) -> bool {
        let Some(list) = self.window_floatings.get_mut(&window) else {
            return false;
        };
        let Some(index) = list.iter().position(|w| w.floating == floating) else {
            return false;
        };
        if index + 1 == list.len() {
            return true;
        }
        let entry = list.remove(index);
        list.push(entry);
        true
    }

    pub fn merge_floating_into(
        &mut self,
        window: AppWindowId,
        floating: DockNodeId,
        target_tabs: DockNodeId,
    ) -> bool {
        let Some(list) = self.window_floatings.get(&window) else {
            return false;
        };
        if !list.iter().any(|w| w.floating == floating) {
            return false;
        }

        let panels = self.collect_panels_in_subtree(floating);
        for panel in panels {
            let _ = self.move_panel_between_windows(
                window,
                panel,
                window,
                target_tabs,
                DropZone::Center,
                None,
            );
        }

        if let Some(list) = self.window_floatings.get_mut(&window)
            && let Some(index) = list.iter().position(|w| w.floating == floating)
        {
            list.remove(index);
        }
        true
    }

    pub fn set_active_tab(&mut self, tabs: DockNodeId, active: usize) -> bool {
        let Some(DockNode::Tabs {
            tabs: list,
            active: cur,
        }) = self.nodes.get_mut(tabs)
        else {
            return false;
        };
        if list.is_empty() {
            *cur = 0;
            return true;
        }
        *cur = active.min(list.len() - 1);
        true
    }

    pub fn update_split_two(&mut self, split: DockNodeId, first_fraction: f32) -> bool {
        let Some(DockNode::Split {
            children,
            fractions,
            ..
        }) = self.nodes.get_mut(split)
        else {
            return false;
        };
        if children.len() != 2 || fractions.len() != 2 {
            return false;
        }
        let f0 = first_fraction.clamp(0.0, 1.0);
        fractions[0] = f0;
        fractions[1] = 1.0 - f0;
        true
    }

    pub fn update_split_fractions(&mut self, split: DockNodeId, mut next: Vec<f32>) -> bool {
        let Some(DockNode::Split {
            children,
            fractions,
            ..
        }) = self.nodes.get_mut(split)
        else {
            return false;
        };
        if children.len() < 2 || next.len() != children.len() {
            return false;
        }

        for f in &mut next {
            if !f.is_finite() {
                *f = 0.0;
            }
            *f = (*f).max(0.0);
        }
        let sum: f32 = next.iter().sum();
        if !sum.is_finite() || sum <= f32::EPSILON {
            next = vec![1.0 / next.len() as f32; next.len()];
        } else {
            for f in &mut next {
                *f /= sum;
            }
            let len = next.len();
            if len >= 1 {
                let rest: f32 = next.iter().take(len.saturating_sub(1)).sum();
                next[len - 1] = (1.0 - rest).clamp(0.0, 1.0);
            }
        }

        *fractions = next;
        true
    }

    fn insert_panel_into_tabs_at(
        &mut self,
        tabs: DockNodeId,
        panel: PanelKey,
        index: Option<usize>,
    ) -> bool {
        let Some(DockNode::Tabs { tabs: list, active }) = self.nodes.get_mut(tabs) else {
            return false;
        };
        if list.contains(&panel) {
            return true;
        }

        match index {
            Some(i) => {
                let i = i.min(list.len());
                list.insert(i, panel);
                *active = i;
            }
            None => {
                list.push(panel);
                *active = list.len().saturating_sub(1);
            }
        }
        true
    }

    fn insert_panels_into_tabs_at(
        &mut self,
        tabs: DockNodeId,
        panels: &[PanelKey],
        index: Option<usize>,
        active_in_group: usize,
    ) -> bool {
        let Some(DockNode::Tabs { tabs: list, active }) = self.nodes.get_mut(tabs) else {
            return false;
        };
        if panels.is_empty() {
            return true;
        }

        let mut insert_at = index.unwrap_or(list.len()).min(list.len());
        for panel in panels {
            if list.contains(panel) {
                continue;
            }
            list.insert(insert_at, panel.clone());
            insert_at = insert_at.saturating_add(1);
        }

        if let Some(active_panel) = panels.get(active_in_group)
            && let Some(ix) = list.iter().position(|p| p == active_panel)
        {
            *active = ix;
        }
        if list.is_empty() {
            *active = 0;
        } else if *active >= list.len() {
            *active = list.len().saturating_sub(1);
        }

        true
    }

    fn remove_panel_from_tabs(&mut self, tabs: DockNodeId, index: usize) -> bool {
        let Some(DockNode::Tabs { tabs: list, active }) = self.nodes.get_mut(tabs) else {
            return false;
        };
        if index >= list.len() {
            return false;
        }

        list.remove(index);
        if list.is_empty() {
            *active = 0;
        } else if *active >= list.len() {
            *active = list.len().saturating_sub(1);
        } else if index < *active {
            *active = active.saturating_sub(1);
        }
        true
    }

    fn replace_node_in_window_tree(
        &mut self,
        window: AppWindowId,
        old: DockNodeId,
        new: DockNodeId,
    ) {
        if self.window_root(window) == Some(old) {
            self.set_window_root(window, new);
            return;
        }
        if let Some(list) = self.window_floatings.get_mut(&window) {
            for w in list {
                if w.floating == old {
                    w.floating = new;
                    return;
                }
            }
        }

        let Some(root) = self.window_root(window) else {
            return;
        };
        if let Some(parent) = self.find_parent_in_subtree(root, old) {
            self.replace_child_in_node(parent, old, new);
            return;
        }
        if let Some(list) = self.window_floatings.get(&window) {
            for w in list {
                if let Some(parent) = self.find_parent_in_subtree(w.floating, old) {
                    self.replace_child_in_node(parent, old, new);
                    return;
                }
            }
        }
    }

    fn replace_child_in_node(
        &mut self,
        node: DockNodeId,
        old: DockNodeId,
        new: DockNodeId,
    ) -> bool {
        let Some(n) = self.nodes.get_mut(node) else {
            return false;
        };
        match n {
            DockNode::Split { children, .. } => {
                let Some(index) = children.iter().position(|c| *c == old) else {
                    return false;
                };
                children[index] = new;
                true
            }
            DockNode::Floating { child } => {
                if *child != old {
                    return false;
                }
                *child = new;
                true
            }
            DockNode::Tabs { .. } => false,
        }
    }

    fn find_parent_in_subtree(&self, node: DockNodeId, target: DockNodeId) -> Option<DockNodeId> {
        let n = self.nodes.get(node)?;
        match n {
            DockNode::Tabs { .. } => None,
            DockNode::Split { children, .. } => {
                if children.contains(&target) {
                    return Some(node);
                }
                children
                    .iter()
                    .copied()
                    .find_map(|child| self.find_parent_in_subtree(child, target))
            }
            DockNode::Floating { child } => {
                if *child == target {
                    return Some(node);
                }
                self.find_parent_in_subtree(*child, target)
            }
        }
    }

    fn collapse_empty_tabs_upwards(&mut self, window: AppWindowId, start_tabs: DockNodeId) {
        let Some(root) = self.root_for_node_in_window_forest(window, start_tabs) else {
            return;
        };
        let mut current = start_tabs;
        loop {
            let Some(parent) = self.find_parent_in_subtree(root, current) else {
                break;
            };

            let only_child = match self.nodes.get(parent) {
                Some(DockNode::Split { children, .. }) => {
                    if children.len() != 2 {
                        break;
                    }
                    if children[0] == current {
                        children[1]
                    } else {
                        children[0]
                    }
                }
                Some(DockNode::Floating { child }) => *child,
                _ => break,
            };

            match self.nodes.get(current) {
                Some(DockNode::Tabs { tabs, .. }) if tabs.is_empty() => {}
                _ => break,
            }

            if parent == root {
                if self.window_root(window) == Some(root) {
                    self.set_window_root(window, only_child);
                } else if let Some(list) = self.window_floatings.get_mut(&window) {
                    for w in list {
                        if w.floating == root {
                            w.floating = only_child;
                            break;
                        }
                    }
                } else if let Some(DockNode::Floating { child }) = self.nodes.get_mut(root) {
                    *child = only_child;
                }
                break;
            }
            self.replace_node_in_window_tree(window, parent, only_child);
            current = parent;
        }
    }

    fn root_for_node_in_window_forest(
        &self,
        window: AppWindowId,
        target: DockNodeId,
    ) -> Option<DockNodeId> {
        fn contains(graph: &DockGraph, root: DockNodeId, target: DockNodeId) -> bool {
            if root == target {
                return true;
            }
            let Some(n) = graph.nodes.get(root) else {
                return false;
            };
            match n {
                DockNode::Tabs { .. } => false,
                DockNode::Split { children, .. } => {
                    children.iter().copied().any(|c| contains(graph, c, target))
                }
                DockNode::Floating { child } => contains(graph, *child, target),
            }
        }

        if let Some(root) = self.window_root(window)
            && contains(self, root, target)
        {
            return Some(root);
        }
        if let Some(list) = self.window_floatings.get(&window) {
            for w in list {
                if contains(self, w.floating, target) {
                    return Some(w.floating);
                }
            }
        }
        None
    }

    fn remove_empty_floating_windows(&mut self, window: AppWindowId) {
        let Some(mut list) = self.window_floatings.remove(&window) else {
            return;
        };
        list.retain(|w| !self.collect_panels_in_subtree(w.floating).is_empty());
        if !list.is_empty() {
            self.window_floatings.insert(window, list);
        }
    }
}
