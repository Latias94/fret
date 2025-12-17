use crate::{
    geometry::{Point, Px, Rect, Size},
    ids::{AppWindowId, DockNodeId, PanelId},
};
use slotmap::SlotMap;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        tabs: Vec<PanelId>,
        active: usize,
    },
}

#[derive(Debug, Default)]
pub struct DockGraph {
    nodes: SlotMap<DockNodeId, DockNode>,
    window_roots: HashMap<AppWindowId, DockNodeId>,
}

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

    pub fn move_panel(
        &mut self,
        window: AppWindowId,
        panel: PanelId,
        target_tabs: DockNodeId,
        zone: DropZone,
    ) -> bool {
        self.move_panel_ex(window, panel, target_tabs, zone, None)
    }

    pub fn move_panel_ex(
        &mut self,
        window: AppWindowId,
        panel: PanelId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    ) -> bool {
        self.move_panel_between_windows(window, panel, window, target_tabs, zone, insert_index)
    }

    pub fn move_panel_between_windows(
        &mut self,
        source_window: AppWindowId,
        panel: PanelId,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    ) -> bool {
        let Some((source_tabs, source_index)) = self.find_panel_in_window(source_window, panel)
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
            if source_window == target_window && source_tabs == target_tabs {
                if let Some(i) = index.as_mut() {
                    if *i > source_index {
                        *i = i.saturating_sub(1);
                    }
                }
            }

            let ok = self.insert_panel_into_tabs_at(target_tabs, panel, index);
            self.collapse_empty_tabs_upwards(source_window, source_tabs);
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
        true
    }

    pub fn float_panel_to_window(
        &mut self,
        source_window: AppWindowId,
        panel: PanelId,
        new_window: AppWindowId,
    ) -> bool {
        let Some((source_tabs, source_index)) = self.find_panel_in_window(source_window, panel)
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
        }
    }

    fn find_panel_in_window(
        &self,
        window: AppWindowId,
        panel: PanelId,
    ) -> Option<(DockNodeId, usize)> {
        let root = self.window_root(window)?;
        self.find_panel_in_subtree(root, panel)
    }

    pub fn collect_panels_in_window(&self, window: AppWindowId) -> Vec<PanelId> {
        let Some(root) = self.window_root(window) else {
            return Vec::new();
        };
        self.collect_panels_in_subtree(root)
    }

    pub fn first_tabs_in_window(&self, window: AppWindowId) -> Option<DockNodeId> {
        let root = self.window_root(window)?;
        self.first_tabs_in_subtree(root)
    }

    fn collect_panels_in_subtree(&self, node: DockNodeId) -> Vec<PanelId> {
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
        }
    }

    fn first_tabs_in_subtree(&self, node: DockNodeId) -> Option<DockNodeId> {
        let Some(n) = self.nodes.get(node) else {
            return None;
        };
        match n {
            DockNode::Tabs { .. } => Some(node),
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .find_map(|child| self.first_tabs_in_subtree(child)),
        }
    }

    fn find_panel_in_subtree(
        &self,
        node: DockNodeId,
        panel: PanelId,
    ) -> Option<(DockNodeId, usize)> {
        let Some(n) = self.nodes.get(node) else {
            return None;
        };
        match n {
            DockNode::Tabs { tabs, .. } => tabs.iter().position(|&p| p == panel).map(|i| (node, i)),
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .find_map(|child| self.find_panel_in_subtree(child, panel)),
        }
    }

    fn insert_panel_into_tabs_at(
        &mut self,
        tabs: DockNodeId,
        panel: PanelId,
        index: Option<usize>,
    ) -> bool {
        let Some(DockNode::Tabs { tabs: list, active }) = self.nodes.get_mut(tabs) else {
            return false;
        };
        if list.iter().any(|&p| p == panel) {
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
            *active = list.len() - 1;
        }
        true
    }

    fn replace_node_in_window_tree(
        &mut self,
        window: AppWindowId,
        old: DockNodeId,
        new: DockNodeId,
    ) {
        let Some(root) = self.window_root(window) else {
            return;
        };
        if root == old {
            self.set_window_root(window, new);
            return;
        }
        let Some(parent) = self.find_parent_in_subtree(root, old) else {
            return;
        };
        let Some(DockNode::Split { children, .. }) = self.nodes.get_mut(parent) else {
            return;
        };
        for child in children.iter_mut() {
            if *child == old {
                *child = new;
                break;
            }
        }
    }

    fn find_parent_in_subtree(&self, node: DockNodeId, target: DockNodeId) -> Option<DockNodeId> {
        let Some(n) = self.nodes.get(node) else {
            return None;
        };
        match n {
            DockNode::Tabs { .. } => None,
            DockNode::Split { children, .. } => {
                if children.iter().any(|&c| c == target) {
                    return Some(node);
                }
                children
                    .iter()
                    .copied()
                    .find_map(|child| self.find_parent_in_subtree(child, target))
            }
        }
    }

    fn collapse_empty_tabs_upwards(&mut self, window: AppWindowId, start_tabs: DockNodeId) {
        let Some(root) = self.window_root(window) else {
            return;
        };
        if root == start_tabs {
            return;
        }

        let is_empty_tabs = matches!(
            self.nodes.get(start_tabs),
            Some(DockNode::Tabs { tabs, .. }) if tabs.is_empty()
        );
        if !is_empty_tabs {
            return;
        }

        let mut current = start_tabs;
        loop {
            let root = self.window_root(window).unwrap();
            let Some(parent) = self.find_parent_in_subtree(root, current) else {
                break;
            };

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
                self.set_window_root(window, only_child);
                break;
            }
            self.replace_node_in_window_tree(window, parent, only_child);
            current = parent;
        }
    }
}
