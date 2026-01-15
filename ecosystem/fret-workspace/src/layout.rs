use std::sync::Arc;

use fret_core::Axis;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A lightweight "workspace shell" layout that can be persisted by apps.
///
/// This intentionally does not embed docking internals:
/// - Dock layout persistence is covered by docking contracts (ADR 0013).
/// - Workspace layout focuses on editor chrome: document groups (tabs) and pane splits.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceLayout {
    pub windows: Vec<WorkspaceWindowLayout>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceWindowLayout {
    pub id: Arc<str>,
    pub pane_tree: WorkspacePaneTree,
    pub active_pane: Option<Arc<str>>,
}

impl WorkspaceWindowLayout {
    pub fn new(id: impl Into<Arc<str>>, root_pane_id: impl Into<Arc<str>>) -> Self {
        let root_pane_id: Arc<str> = root_pane_id.into();
        Self {
            id: id.into(),
            pane_tree: WorkspacePaneTree::leaf(root_pane_id.clone()),
            active_pane: Some(root_pane_id),
        }
    }

    pub fn active_pane_id(&self) -> Option<&Arc<str>> {
        self.active_pane.as_ref()
    }

    pub fn active_pane_mut(&mut self) -> Option<&mut WorkspacePaneLayout> {
        let active = self.active_pane.clone()?;
        self.pane_tree.find_pane_mut(active.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct WorkspacePaneLayout {
    pub id: Arc<str>,
    pub tabs: crate::tabs::WorkspaceTabs,
}

impl WorkspacePaneLayout {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            tabs: crate::tabs::WorkspaceTabs::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkspacePaneTree {
    Leaf(WorkspacePaneLayout),
    Split {
        axis: Axis,
        /// Fraction of the available space given to `a`.
        fraction: f32,
        a: Box<WorkspacePaneTree>,
        b: Box<WorkspacePaneTree>,
    },
}

impl WorkspacePaneTree {
    pub fn leaf(id: impl Into<Arc<str>>) -> Self {
        Self::Leaf(WorkspacePaneLayout::new(id))
    }

    pub fn split(axis: Axis, fraction: f32, a: WorkspacePaneTree, b: WorkspacePaneTree) -> Self {
        Self::Split {
            axis,
            fraction: clamp_fraction(fraction),
            a: Box::new(a),
            b: Box::new(b),
        }
    }

    pub fn find_pane(&self, id: &str) -> Option<&WorkspacePaneLayout> {
        match self {
            WorkspacePaneTree::Leaf(pane) => (pane.id.as_ref() == id).then_some(pane),
            WorkspacePaneTree::Split { a, b, .. } => a.find_pane(id).or_else(|| b.find_pane(id)),
        }
    }

    pub fn find_pane_mut(&mut self, id: &str) -> Option<&mut WorkspacePaneLayout> {
        match self {
            WorkspacePaneTree::Leaf(pane) => (pane.id.as_ref() == id).then_some(pane),
            WorkspacePaneTree::Split { a, b, .. } => {
                a.find_pane_mut(id).or_else(|| b.find_pane_mut(id))
            }
        }
    }

    pub fn first_leaf_id(&self) -> Option<&Arc<str>> {
        match self {
            WorkspacePaneTree::Leaf(pane) => Some(&pane.id),
            WorkspacePaneTree::Split { a, .. } => a.first_leaf_id(),
        }
    }
}

fn clamp_fraction(fraction: f32) -> f32 {
    fraction.clamp(0.05, 0.95)
}

pub const WORKSPACE_LAYOUT_VERSION_V1: u32 = 1;

/// Persistable snapshot of `WorkspaceLayout` (V1).
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspaceLayoutV1 {
    pub layout_version: u32,
    pub windows: Vec<WorkspaceWindowLayoutV1>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspaceWindowLayoutV1 {
    pub id: Arc<str>,
    pub pane_tree: WorkspacePaneTreeV1,
    pub active_pane: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkspacePaneLayoutV1 {
    pub id: Arc<str>,
    pub tabs: crate::tabs::WorkspaceTabsV1,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WorkspacePaneTreeV1 {
    Leaf(WorkspacePaneLayoutV1),
    Split {
        axis: Axis,
        fraction: f32,
        a: Box<WorkspacePaneTreeV1>,
        b: Box<WorkspacePaneTreeV1>,
    },
}

impl From<&WorkspaceLayout> for WorkspaceLayoutV1 {
    fn from(value: &WorkspaceLayout) -> Self {
        Self {
            layout_version: WORKSPACE_LAYOUT_VERSION_V1,
            windows: value
                .windows
                .iter()
                .map(|w| WorkspaceWindowLayoutV1 {
                    id: w.id.clone(),
                    pane_tree: WorkspacePaneTreeV1::from(&w.pane_tree),
                    active_pane: w.active_pane.clone(),
                })
                .collect(),
        }
    }
}

impl From<&WorkspacePaneTree> for WorkspacePaneTreeV1 {
    fn from(value: &WorkspacePaneTree) -> Self {
        match value {
            WorkspacePaneTree::Leaf(pane) => WorkspacePaneTreeV1::Leaf(WorkspacePaneLayoutV1 {
                id: pane.id.clone(),
                tabs: pane.tabs.snapshot_v1(),
            }),
            WorkspacePaneTree::Split {
                axis,
                fraction,
                a,
                b,
            } => WorkspacePaneTreeV1::Split {
                axis: *axis,
                fraction: clamp_fraction(*fraction),
                a: Box::new(WorkspacePaneTreeV1::from(a.as_ref())),
                b: Box::new(WorkspacePaneTreeV1::from(b.as_ref())),
            },
        }
    }
}

impl WorkspaceLayoutV1 {
    pub fn into_layout(self) -> WorkspaceLayout {
        let mut windows: Vec<WorkspaceWindowLayout> = self
            .windows
            .into_iter()
            .map(|w| {
                let pane_tree = w.pane_tree.into_tree();
                let mut window = WorkspaceWindowLayout {
                    id: w.id,
                    pane_tree,
                    active_pane: w.active_pane,
                };

                if let Some(active) = window.active_pane.clone() {
                    if window.pane_tree.find_pane(active.as_ref()).is_none() {
                        window.active_pane =
                            window.pane_tree.first_leaf_id().cloned().or(Some(active));
                    }
                } else {
                    window.active_pane = window.pane_tree.first_leaf_id().cloned();
                }

                window
            })
            .collect();

        windows.retain(|w| w.pane_tree.first_leaf_id().is_some());

        WorkspaceLayout { windows }
    }
}

impl WorkspacePaneTreeV1 {
    pub fn into_tree(self) -> WorkspacePaneTree {
        match self {
            WorkspacePaneTreeV1::Leaf(pane) => WorkspacePaneTree::Leaf(WorkspacePaneLayout {
                id: pane.id,
                tabs: crate::tabs::WorkspaceTabs::from_snapshot_v1(pane.tabs),
            }),
            WorkspacePaneTreeV1::Split {
                axis,
                fraction,
                a,
                b,
            } => WorkspacePaneTree::Split {
                axis,
                fraction: clamp_fraction(fraction),
                a: Box::new(a.into_tree()),
                b: Box::new(b.into_tree()),
            },
        }
    }
}
