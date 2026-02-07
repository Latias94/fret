mod apply;
pub(super) mod layout;
mod mutate;
pub(super) mod op;
mod persistence;
mod query;
use self::op::DockOp;
use crate::{
    PanelKey,
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

    // DockOp application lives in `apply.rs` to keep the main dock graph module focused on the
    // runtime tree and core mutation primitives.
}

#[cfg(test)]
mod tests;
