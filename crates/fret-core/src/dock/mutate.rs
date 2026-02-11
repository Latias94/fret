use super::*;

impl DockGraph {
    /// Simplify and canonicalize the docking forest for a window.
    ///
    /// Canonical form (v1):
    ///
    /// - `Tabs` nodes are non-empty (empty tabs are pruned; roots may be removed).
    /// - `Split` nodes have `children.len() == fractions.len()`.
    /// - `Split` fractions are finite, non-negative, and normalized (sum ~= 1.0).
    /// - Single-child splits are pruned.
    /// - Nested same-axis splits are flattened (bounded-depth property).
    /// - `Floating` nodes keep their container identity, but their `child` is simplified.
    fn simplify_window_forest(&mut self, window: AppWindowId) {
        if let Some(root) = self.window_root(window) {
            match self.simplify_subtree(root) {
                Some(next_root) => self.set_window_root(window, next_root),
                None => {
                    let _ = self.remove_window_root(window);
                }
            }
        }

        let Some(mut floatings) = self.window_floatings.remove(&window) else {
            return;
        };

        floatings.retain_mut(|w| match self.simplify_subtree(w.floating) {
            Some(next_root) => {
                w.floating = next_root;
                true
            }
            None => false,
        });

        if !floatings.is_empty() {
            self.window_floatings.insert(window, floatings);
        }
    }

    fn simplify_subtree(&mut self, node: DockNodeId) -> Option<DockNodeId> {
        let n = self.nodes.get(node)?.clone();
        match n {
            DockNode::Tabs { tabs, mut active } => {
                if tabs.is_empty() {
                    return None;
                }
                if active >= tabs.len() {
                    active = tabs.len().saturating_sub(1);
                }
                if let Some(DockNode::Tabs {
                    tabs: list,
                    active: cur,
                }) = self.nodes.get_mut(node)
                {
                    *list = tabs;
                    *cur = active;
                }
                Some(node)
            }
            DockNode::Floating { child } => {
                let child = self.simplify_subtree(child)?;
                if let Some(DockNode::Floating { child: cur }) = self.nodes.get_mut(node) {
                    *cur = child;
                }
                Some(node)
            }
            DockNode::Split {
                axis,
                children,
                fractions,
            } => {
                let mut next_children: Vec<DockNodeId> = Vec::new();
                let mut next_fractions: Vec<f32> = Vec::new();

                // Repair mismatched lengths conservatively (treat missing fractions as 1.0 shares).
                for (i, child) in children.into_iter().enumerate() {
                    let Some(child) = self.simplify_subtree(child) else {
                        continue;
                    };
                    let f = fractions.get(i).copied().unwrap_or(1.0);
                    next_children.push(child);
                    next_fractions.push(f);
                }

                if next_children.is_empty() {
                    return None;
                }
                if next_children.len() == 1 {
                    return Some(next_children[0]);
                }

                self.flatten_same_axis_splits(axis, &mut next_children, &mut next_fractions);

                if next_children.is_empty() {
                    return None;
                }
                if next_children.len() == 1 {
                    return Some(next_children[0]);
                }

                normalize_shares(&mut next_fractions);
                debug_assert_eq!(next_children.len(), next_fractions.len());

                if let Some(DockNode::Split {
                    children: cur_children,
                    fractions: cur_fractions,
                    ..
                }) = self.nodes.get_mut(node)
                {
                    *cur_children = next_children;
                    *cur_fractions = next_fractions;
                }

                Some(node)
            }
        }
    }

    fn flatten_same_axis_splits(
        &mut self,
        axis: Axis,
        children: &mut Vec<DockNodeId>,
        fractions: &mut Vec<f32>,
    ) {
        let mut changed = true;
        while changed {
            changed = false;

            let mut out_children: Vec<DockNodeId> = Vec::with_capacity(children.len());
            let mut out_fractions: Vec<f32> = Vec::with_capacity(fractions.len());

            for (child, parent_share) in children.iter().copied().zip(fractions.iter().copied()) {
                let Some(DockNode::Split {
                    axis: child_axis,
                    children: grand_children,
                    fractions: grand_fractions,
                }) = self.nodes.get(child)
                else {
                    out_children.push(child);
                    out_fractions.push(parent_share);
                    continue;
                };

                if *child_axis != axis {
                    out_children.push(child);
                    out_fractions.push(parent_share);
                    continue;
                }

                // Flatten nested same-axis split by distributing the parent share across the grand-children.
                changed = true;

                let mut grand_shares = grand_fractions.clone();
                normalize_shares(&mut grand_shares);
                debug_assert_eq!(grand_children.len(), grand_shares.len());

                for (&gc, &gs) in grand_children.iter().zip(grand_shares.iter()) {
                    out_children.push(gc);
                    out_fractions.push(parent_share * gs);
                }
            }

            *children = out_children;
            *fractions = out_fractions;
        }
    }

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
            self.simplify_window_forest(source_window);
            if target_window != source_window {
                self.simplify_window_forest(target_window);
            }
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

        if self.insert_edge_child_prefer_same_axis_split(
            target_window,
            target_tabs,
            axis,
            zone,
            new_tabs,
        ) {
            self.simplify_window_forest(source_window);
            if target_window != source_window {
                self.simplify_window_forest(target_window);
            }
            return true;
        }

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
        self.simplify_window_forest(source_window);
        if target_window != source_window {
            self.simplify_window_forest(target_window);
        }
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
            self.simplify_window_forest(target_window);
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

        if self.insert_edge_child_prefer_same_axis_split(
            target_window,
            target_tabs,
            axis,
            zone,
            new_tabs,
        ) {
            self.simplify_window_forest(target_window);
            return true;
        }

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
        self.simplify_window_forest(target_window);
        true
    }

    pub fn close_panel(&mut self, window: AppWindowId, panel: PanelKey) -> bool {
        let Some((tabs, index)) = self.find_panel_in_window(window, &panel) else {
            return false;
        };
        if !self.remove_panel_from_tabs(tabs, index) {
            return false;
        }
        self.simplify_window_forest(window);
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
        self.simplify_window_forest(source_window);
        self.simplify_window_forest(new_window);
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

        self.simplify_window_forest(source_window);
        self.simplify_window_forest(target_window);
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
        self.simplify_window_forest(source_window);

        let tabs = self.insert_node(DockNode::Tabs {
            tabs: panels,
            active,
        });
        let floating = self.insert_node(DockNode::Floating { child: tabs });
        self.floating_windows_mut(target_window)
            .push(DockFloatingWindow { floating, rect });
        self.simplify_window_forest(target_window);
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
        self.simplify_window_forest(window);
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

    fn insert_edge_child_prefer_same_axis_split(
        &mut self,
        window: AppWindowId,
        target: DockNodeId,
        axis: Axis,
        zone: DropZone,
        new_child: DockNodeId,
    ) -> bool {
        // Keep core commit semantics and docking preview semantics aligned: use the shared, pure
        // decision helper.
        let Some(decision) = self.edge_dock_decision(window, target, zone) else {
            return false;
        };
        let EdgeDockDecision::InsertIntoSplit {
            split,
            anchor_index,
            insert_index,
        } = decision
        else {
            return false;
        };

        let Some(DockNode::Split {
            axis: split_axis,
            children,
            fractions,
        }) = self.nodes.get_mut(split)
        else {
            return false;
        };
        if *split_axis != axis || children.len() != fractions.len() || children.is_empty() {
            return false;
        }

        split_share_and_insert(children, fractions, anchor_index, insert_index, new_child);
        true
    }

    pub(super) fn find_nearest_same_axis_split_and_anchor(
        &self,
        root: DockNodeId,
        target: DockNodeId,
        axis: Axis,
    ) -> Option<(DockNodeId, usize)> {
        let mut path_rev: Vec<DockNodeId> = Vec::new();
        if !self.build_path_rev(root, target, &mut path_rev) {
            return None;
        }
        path_rev.reverse();
        if path_rev.len() < 2 {
            return None;
        }

        for i in (0..path_rev.len().saturating_sub(1)).rev() {
            let ancestor = path_rev[i];
            let child_on_path = path_rev[i + 1];

            let Some(DockNode::Split {
                axis: split_axis,
                children,
                fractions,
            }) = self.nodes.get(ancestor)
            else {
                continue;
            };
            if *split_axis != axis || children.len() != fractions.len() || children.is_empty() {
                continue;
            }
            let Some(ix) = children.iter().position(|c| *c == child_on_path) else {
                continue;
            };
            return Some((ancestor, ix));
        }

        None
    }

    fn build_path_rev(
        &self,
        node: DockNodeId,
        target: DockNodeId,
        out: &mut Vec<DockNodeId>,
    ) -> bool {
        if node == target {
            out.push(node);
            return true;
        }

        let Some(n) = self.nodes.get(node) else {
            return false;
        };
        match n {
            DockNode::Tabs { .. } => false,
            DockNode::Split { children, .. } => {
                for &child in children {
                    if self.build_path_rev(child, target, out) {
                        out.push(node);
                        return true;
                    }
                }
                false
            }
            DockNode::Floating { child } => {
                if self.build_path_rev(*child, target, out) {
                    out.push(node);
                    true
                } else {
                    false
                }
            }
        }
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
        let Some(_root) = self.root_for_node_in_window_forest(window, start_tabs) else {
            return;
        };

        // Historical helper: this used to assume binary splits. Keep the API, but delegate to the
        // canonical simplifier (which is N-ary safe).
        let _ = start_tabs;
        self.simplify_window_forest(window);
    }

    pub(super) fn root_for_node_in_window_forest(
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
        // Kept for compatibility with older call sites; the canonical simplifier already prunes
        // empty floatings.
        self.simplify_window_forest(window);
    }
}

fn normalize_shares(shares: &mut Vec<f32>) {
    for f in shares.iter_mut() {
        if !f.is_finite() {
            *f = 0.0;
        }
        if *f < 0.0 {
            *f = 0.0;
        }
    }

    let sum: f32 = shares.iter().sum();
    if !sum.is_finite() || sum <= f32::EPSILON {
        let n = shares.len().max(1);
        *shares = vec![1.0 / n as f32; n];
        return;
    }

    for f in shares.iter_mut() {
        *f /= sum;
    }

    // Clamp drift on the last element to keep sum stable.
    let len = shares.len();
    if len >= 1 {
        let rest: f32 = shares.iter().take(len.saturating_sub(1)).sum();
        shares[len - 1] = (1.0 - rest).clamp(0.0, 1.0);
    }
}

fn split_share_and_insert(
    children: &mut Vec<DockNodeId>,
    fractions: &mut Vec<f32>,
    anchor_index: usize,
    insert_index: usize,
    new_child: DockNodeId,
) {
    debug_assert!(!children.is_empty());
    debug_assert_eq!(children.len(), fractions.len());
    debug_assert!(anchor_index < fractions.len());
    debug_assert!(insert_index <= fractions.len());

    // Default ratio for v1: split the anchor child share in half.
    let k = 0.5_f32;

    let old = fractions[anchor_index];
    let keep = old * (1.0 - k);
    let take = old * k;

    fractions[anchor_index] = keep;
    children.insert(insert_index, new_child);
    fractions.insert(insert_index, take);
}
