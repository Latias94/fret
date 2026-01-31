use crate::{
    PanelKey,
    dock_op::DockOp,
    geometry::{Point, Px, Rect, Size},
    ids::{AppWindowId, DockNodeId},
};
use slotmap::{Key, SlotMap};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropZone {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum DockNode {
    Split {
        axis: Axis,
        children: Vec<DockNodeId>,
        fractions: Vec<f32>,
    },
    Tabs {
        tabs: Vec<PanelKey>,
        active: usize,
    },
    /// An in-window floating dock container (ImGui docking, viewports disabled).
    ///
    /// The container node is stable: docking within the floating window replaces `child` while
    /// keeping the container id stable. Window metadata (rect, z-order) is stored in `DockGraph`.
    Floating {
        child: DockNodeId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockFloatingWindow {
    pub floating: DockNodeId,
    pub rect: Rect,
}

#[derive(Debug, Default)]
pub struct DockGraph {
    nodes: SlotMap<DockNodeId, DockNode>,
    window_roots: HashMap<AppWindowId, DockNodeId>,
    window_floatings: HashMap<AppWindowId, Vec<DockFloatingWindow>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockOpApplyError {
    pub kind: DockOpApplyErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockOpApplyErrorKind {
    UnsupportedOp,
    TabsNodeNotFound {
        tabs: DockNodeId,
    },
    NodeIsNotTabs {
        node: DockNodeId,
    },
    ActiveOutOfBounds {
        tabs: DockNodeId,
        active: usize,
        len: usize,
    },
    PanelNotFound {
        window: AppWindowId,
        panel: PanelKey,
    },
    OperationFailed,
}

impl std::fmt::Display for DockOpApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dock op apply error: {:?}", self.kind)
    }
}

impl std::error::Error for DockOpApplyError {}

impl DockGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_node(&mut self, node: DockNode) -> DockNodeId {
        self.nodes.insert(node)
    }

    pub fn node(&self, id: DockNodeId) -> Option<&DockNode> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: DockNodeId) -> Option<&mut DockNode> {
        self.nodes.get_mut(id)
    }

    pub fn set_window_root(&mut self, window: AppWindowId, root: DockNodeId) {
        self.window_roots.insert(window, root);
    }

    pub fn window_root(&self, window: AppWindowId) -> Option<DockNodeId> {
        self.window_roots.get(&window).copied()
    }

    pub fn remove_window_root(&mut self, window: AppWindowId) -> Option<DockNodeId> {
        self.window_roots.remove(&window)
    }

    pub fn floating_windows(&self, window: AppWindowId) -> &[DockFloatingWindow] {
        self.window_floatings
            .get(&window)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn floating_windows_mut(&mut self, window: AppWindowId) -> &mut Vec<DockFloatingWindow> {
        self.window_floatings.entry(window).or_default()
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

    fn collect_panels_in_subtree(&self, node: DockNodeId) -> Vec<PanelKey> {
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
        let removed_before_active = index < *active;
        list.remove(index);
        if list.is_empty() {
            *active = 0;
        } else {
            if removed_before_active {
                *active = active.saturating_sub(1);
            }
            if *active >= list.len() {
                *active = list.len() - 1;
            }
        }
        true
    }

    fn replace_node_in_window_tree(
        &mut self,
        window: AppWindowId,
        old: DockNodeId,
        new: DockNodeId,
    ) {
        if let Some(root) = self.window_root(window) {
            if root == old {
                self.set_window_root(window, new);
                return;
            }
            if let Some(parent) = self.find_parent_in_subtree(root, old)
                && self.replace_child_in_node(parent, old, new)
            {
                return;
            }
        }

        let mut floating_index: Option<usize> = None;
        let mut floating_parent: Option<DockNodeId> = None;
        let mut floating_root_is_old = false;
        if let Some(list) = self.window_floatings.get(&window) {
            for (i, w) in list.iter().enumerate() {
                if w.floating == old {
                    floating_index = Some(i);
                    floating_root_is_old = true;
                    break;
                }
                if let Some(parent) = self.find_parent_in_subtree(w.floating, old) {
                    floating_index = Some(i);
                    floating_parent = Some(parent);
                    break;
                }
            }
        }

        let Some(index) = floating_index else {
            return;
        };

        if floating_root_is_old {
            if let Some(list) = self.window_floatings.get_mut(&window)
                && index < list.len()
            {
                list[index].floating = new;
            }
            return;
        }

        let Some(parent) = floating_parent else {
            return;
        };
        let _ = self.replace_child_in_node(parent, old, new);
    }

    fn replace_child_in_node(
        &mut self,
        parent: DockNodeId,
        old: DockNodeId,
        new: DockNodeId,
    ) -> bool {
        match self.nodes.get_mut(parent) {
            Some(DockNode::Split { children, .. }) => {
                for child in children.iter_mut() {
                    if *child == old {
                        *child = new;
                        return true;
                    }
                }
                false
            }
            Some(DockNode::Floating { child }) => {
                if *child == old {
                    *child = new;
                    return true;
                }
                false
            }
            _ => false,
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

        let is_empty_tabs = matches!(
            self.nodes.get(start_tabs),
            Some(DockNode::Tabs { tabs, .. }) if tabs.is_empty()
        );
        if !is_empty_tabs {
            return;
        }

        let mut current = start_tabs;
        loop {
            let Some(parent) = self.find_parent_in_subtree(root, current) else {
                break;
            };

            if matches!(self.nodes.get(parent), Some(DockNode::Floating { .. })) {
                if self.collect_panels_in_subtree(parent).is_empty() {
                    let _ = self.remove_floating_window(window, parent);
                }
                break;
            }

            let Some(DockNode::Split {
                children,
                fractions,
                ..
            }) = self.nodes.get_mut(parent)
            else {
                break;
            };

            if let Some(pos) = children.iter().position(|&c| c == current) {
                children.remove(pos);
                if pos < fractions.len() {
                    fractions.remove(pos);
                }
            }

            if children.len() != 1 {
                break;
            }

            let only_child = children[0];
            if root == parent {
                if self.window_root(window) == Some(root) {
                    self.set_window_root(window, only_child);
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

    fn remove_floating_window(&mut self, window: AppWindowId, floating: DockNodeId) -> bool {
        let Some(list) = self.window_floatings.get_mut(&window) else {
            return false;
        };
        let Some(index) = list.iter().position(|w| w.floating == floating) else {
            return false;
        };
        list.remove(index);
        true
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

    pub fn apply_op_checked(&mut self, op: &DockOp) -> Result<bool, DockOpApplyError> {
        match op {
            DockOp::SetActiveTab { tabs, active } => {
                let Some(node) = self.nodes.get(*tabs) else {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::TabsNodeNotFound { tabs: *tabs },
                    });
                };
                let DockNode::Tabs { tabs: list, .. } = node else {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::NodeIsNotTabs { node: *tabs },
                    });
                };
                if *active >= list.len() {
                    return Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::ActiveOutOfBounds {
                            tabs: *tabs,
                            active: *active,
                            len: list.len(),
                        },
                    });
                }
                Ok(self.set_active_tab(*tabs, *active))
            }
            DockOp::ClosePanel { window, panel } => {
                if self.close_panel(*window, panel.clone()) {
                    Ok(true)
                } else {
                    Err(DockOpApplyError {
                        kind: DockOpApplyErrorKind::PanelNotFound {
                            window: *window,
                            panel: panel.clone(),
                        },
                    })
                }
            }
            DockOp::RequestFloatPanelToNewWindow { .. } => Err(DockOpApplyError {
                kind: DockOpApplyErrorKind::UnsupportedOp,
            }),
            _ => Ok(self.apply_op(op)),
        }
    }

    pub fn apply_op(&mut self, op: &DockOp) -> bool {
        match op {
            DockOp::SetActiveTab { tabs, active } => self.set_active_tab(*tabs, *active),
            DockOp::ClosePanel { window, panel } => self.close_panel(*window, panel.clone()),
            DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs,
                zone,
                insert_index,
            } => self.move_panel_between_windows(
                *source_window,
                panel.clone(),
                *target_window,
                *target_tabs,
                *zone,
                *insert_index,
            ),
            DockOp::MoveTabs {
                source_window,
                source_tabs,
                target_window,
                target_tabs,
                zone,
                insert_index,
            } => self.move_tabs_between_windows(
                *source_window,
                *source_tabs,
                *target_window,
                *target_tabs,
                *zone,
                *insert_index,
            ),
            DockOp::FloatPanelToWindow {
                source_window,
                panel,
                new_window,
            } => self.float_panel_to_window(*source_window, panel.clone(), *new_window),
            DockOp::RequestFloatPanelToNewWindow { .. } => false,
            DockOp::FloatPanelInWindow {
                source_window,
                panel,
                target_window,
                rect,
            } => self.float_panel_in_window(*source_window, panel.clone(), *target_window, *rect),
            DockOp::FloatTabsInWindow {
                source_window,
                source_tabs,
                target_window,
                rect,
            } => self.float_tabs_in_window(*source_window, *source_tabs, *target_window, *rect),
            DockOp::SetFloatingRect {
                window,
                floating,
                rect,
            } => self.set_floating_rect(*window, *floating, *rect),
            DockOp::RaiseFloating { window, floating } => self.raise_floating(*window, *floating),
            DockOp::MergeFloatingInto {
                window,
                floating,
                target_tabs,
            } => self.merge_floating_into(*window, *floating, *target_tabs),
            DockOp::MergeWindowInto {
                source_window,
                target_window,
                target_tabs,
            } => {
                let panels = self.collect_panels_in_window(*source_window);
                for panel in panels {
                    let _ = self.move_panel_between_windows(
                        *source_window,
                        panel,
                        *target_window,
                        *target_tabs,
                        DropZone::Center,
                        None,
                    );
                }
                let _ = self.remove_window_root(*source_window);
                let _ = self.window_floatings.remove(source_window);
                true
            }
            DockOp::SetSplitFractions { split, fractions } => {
                self.update_split_fractions(*split, fractions.clone())
            }
            DockOp::SetSplitFractionsMany { updates } => {
                let mut changed = false;
                for u in updates {
                    changed |= self.update_split_fractions(u.split, u.fractions.clone());
                }
                changed
            }
            DockOp::SetSplitFractionTwo {
                split,
                first_fraction,
            } => self.update_split_two(*split, *first_fraction),
        }
    }

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

        Ok(imported_any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn window(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn float_panel_in_window_creates_floating_container() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");

        let mut g = DockGraph::new();
        let tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 0,
        });
        g.set_window_root(w, tabs);

        let ok = g.apply_op(&DockOp::FloatPanelInWindow {
            source_window: w,
            panel: panel_b.clone(),
            target_window: w,
            rect: rect(10.0, 20.0, 300.0, 200.0),
        });
        assert!(ok);
        assert_eq!(g.collect_panels_in_window(w).len(), 2);
        assert!(
            g.floating_windows(w)
                .iter()
                .any(|f| { g.collect_panels_in_subtree(f.floating).contains(&panel_b) })
        );
        assert!(g.find_panel_in_window(w, &panel_b).is_some());
    }

    #[test]
    fn merge_floating_into_moves_panels_and_removes_floating_entry() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");

        let mut g = DockGraph::new();
        let main_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 0,
        });
        g.set_window_root(w, main_tabs);

        assert!(g.apply_op(&DockOp::FloatPanelInWindow {
            source_window: w,
            panel: panel_b.clone(),
            target_window: w,
            rect: rect(0.0, 0.0, 200.0, 160.0),
        }));

        let floating = g.floating_windows(w).first().unwrap().floating;
        assert!(g.apply_op(&DockOp::MergeFloatingInto {
            window: w,
            floating,
            target_tabs: main_tabs,
        }));

        assert!(g.floating_windows(w).is_empty());
        let (tabs, _i) = g
            .find_panel_in_window(w, &panel_b)
            .expect("panel in window");
        assert_eq!(tabs, main_tabs);
    }

    #[test]
    fn float_tabs_in_window_creates_floating_container_with_tabs() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let panel_c = PanelKey::new("test.c");

        let mut g = DockGraph::new();
        let tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
            active: 1,
        });
        g.set_window_root(w, tabs);

        assert!(g.apply_op(&DockOp::FloatTabsInWindow {
            source_window: w,
            source_tabs: tabs,
            target_window: w,
            rect: rect(10.0, 20.0, 300.0, 200.0),
        }));

        assert!(
            g.window_root(w).is_none(),
            "expected dock root to be removed"
        );
        let floatings = g.floating_windows(w);
        assert_eq!(floatings.len(), 1);
        assert_eq!(floatings[0].rect, rect(10.0, 20.0, 300.0, 200.0));
        assert_eq!(
            g.collect_panels_in_subtree(floatings[0].floating),
            vec![panel_a, panel_b, panel_c]
        );
    }

    #[test]
    fn move_tabs_merges_into_target_tabs_and_preserves_active() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let panel_c = PanelKey::new("test.c");

        let mut g = DockGraph::new();
        let source_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 1,
        });
        let target_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_c.clone()],
            active: 0,
        });
        let root = g.insert_node(DockNode::Split {
            axis: Axis::Horizontal,
            children: vec![source_tabs, target_tabs],
            fractions: vec![0.5, 0.5],
        });
        g.set_window_root(w, root);

        assert!(g.apply_op(&DockOp::MoveTabs {
            source_window: w,
            source_tabs,
            target_window: w,
            target_tabs,
            zone: DropZone::Center,
            insert_index: Some(0),
        }));

        assert_eq!(g.window_root(w), Some(target_tabs));
        let DockNode::Tabs { tabs, active } = g.node(target_tabs).expect("target tabs exists")
        else {
            unreachable!();
        };
        assert_eq!(
            tabs,
            &vec![panel_a.clone(), panel_b.clone(), panel_c.clone()]
        );
        assert_eq!(*active, 1);
    }

    #[test]
    fn layout_roundtrips_floatings_with_rect_and_order() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let panel_c = PanelKey::new("test.c");

        let mut g = DockGraph::new();
        let main_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone()],
            active: 0,
        });
        g.set_window_root(w, main_tabs);

        let f0_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_b.clone()],
            active: 0,
        });
        let f0 = g.insert_node(DockNode::Floating { child: f0_tabs });
        g.floating_windows_mut(w).push(DockFloatingWindow {
            floating: f0,
            rect: rect(10.0, 20.0, 300.0, 200.0),
        });

        let f1_tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_c.clone()],
            active: 0,
        });
        let f1 = g.insert_node(DockNode::Floating { child: f1_tabs });
        g.floating_windows_mut(w).push(DockFloatingWindow {
            floating: f1,
            rect: rect(40.0, 60.0, 200.0, 120.0),
        });

        let windows = vec![(w, "main".to_string())];
        let layout = g.export_layout(&windows);

        let mut g2 = DockGraph::new();
        assert!(g2.import_layout_for_windows(&layout, &windows));
        assert_eq!(
            g2.collect_panels_in_window(w),
            vec![panel_a, panel_b, panel_c]
        );
        let floatings = g2.floating_windows(w);
        assert_eq!(floatings.len(), 2);
        assert_eq!(floatings[0].rect, rect(10.0, 20.0, 300.0, 200.0));
        assert_eq!(floatings[1].rect, rect(40.0, 60.0, 200.0, 120.0));
    }

    #[test]
    fn import_layout_degrades_unmapped_windows_into_floating_containers() {
        let window_a = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");

        let layout = crate::DockLayout::new(
            vec![
                crate::DockLayoutWindow {
                    logical_window_id: "main".to_string(),
                    root: 1,
                    placement: None,
                    floatings: Vec::new(),
                },
                crate::DockLayoutWindow {
                    logical_window_id: "extra".to_string(),
                    root: 2,
                    placement: Some(crate::DockWindowPlacement {
                        width: 400,
                        height: 300,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    }),
                    floatings: Vec::new(),
                },
            ],
            vec![
                crate::DockLayoutNode::Tabs {
                    id: 1,
                    tabs: vec![panel_a.clone()],
                    active: 0,
                },
                crate::DockLayoutNode::Tabs {
                    id: 2,
                    tabs: vec![panel_b.clone()],
                    active: 0,
                },
            ],
        );

        let mut g = DockGraph::new();
        assert!(g.import_layout_for_windows_with_fallback_floatings(
            &layout,
            &[(window_a, "main".to_string())],
            window_a
        ));

        assert!(g.find_panel_in_window(window_a, &panel_a).is_some());
        assert!(g.find_panel_in_window(window_a, &panel_b).is_some());
        assert_eq!(g.floating_windows(window_a).len(), 1);
    }

    #[test]
    fn close_panel_before_active_preserves_active_panel() {
        let w = window(1);
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let panel_c = PanelKey::new("test.c");

        let mut g = DockGraph::new();
        let tabs = g.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
            active: 1,
        });
        g.set_window_root(w, tabs);

        assert!(g.apply_op(&DockOp::ClosePanel {
            window: w,
            panel: panel_a.clone(),
        }));

        let DockNode::Tabs { tabs: list, active } = g.node(tabs).expect("tabs node must exist")
        else {
            unreachable!();
        };

        assert_eq!(list, &vec![panel_b.clone(), panel_c.clone()]);
        assert_eq!(*active, 0, "expected active panel (b) to remain active");
        assert_eq!(
            g.find_panel_in_window(w, &panel_b),
            Some((tabs, 0)),
            "expected the previously-active panel to remain selected"
        );
    }
}
