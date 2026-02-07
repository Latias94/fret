use serde::{Deserialize, Serialize};

use crate::{AppWindowId, Axis, DockGraph, DockNode, DockNodeId, PanelKey};

pub const DOCK_LAYOUT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayout {
    pub layout_version: u32,
    pub windows: Vec<DockLayoutWindow>,
    pub nodes: Vec<DockLayoutNode>,
}

impl DockLayout {
    pub fn new(windows: Vec<DockLayoutWindow>, nodes: Vec<DockLayoutNode>) -> Self {
        Self {
            layout_version: DOCK_LAYOUT_VERSION,
            windows,
            nodes,
        }
    }

    pub fn validate(&self) -> Result<(), DockLayoutValidationError> {
        use std::collections::HashMap;

        if self.layout_version != DOCK_LAYOUT_VERSION {
            return Err(DockLayoutValidationError {
                kind: DockLayoutValidationErrorKind::UnsupportedVersion {
                    expected: DOCK_LAYOUT_VERSION,
                    found: self.layout_version,
                },
            });
        }

        let mut by_id: HashMap<u32, &DockLayoutNode> = HashMap::new();
        for node in &self.nodes {
            let id = match node {
                DockLayoutNode::Split { id, .. } => *id,
                DockLayoutNode::Tabs { id, .. } => *id,
            };
            if by_id.insert(id, node).is_some() {
                return Err(DockLayoutValidationError {
                    kind: DockLayoutValidationErrorKind::DuplicateNodeId { id },
                });
            }
        }

        for (id, node) in &by_id {
            match node {
                DockLayoutNode::Tabs { tabs, active, .. } => {
                    if tabs.is_empty() {
                        return Err(DockLayoutValidationError {
                            kind: DockLayoutValidationErrorKind::EmptyTabs { id: *id },
                        });
                    }
                    if *active >= tabs.len() {
                        return Err(DockLayoutValidationError {
                            kind: DockLayoutValidationErrorKind::TabsActiveOutOfBounds {
                                id: *id,
                                active: *active,
                                len: tabs.len(),
                            },
                        });
                    }
                }
                DockLayoutNode::Split {
                    children,
                    fractions,
                    ..
                } => {
                    if children.is_empty() {
                        return Err(DockLayoutValidationError {
                            kind: DockLayoutValidationErrorKind::EmptySplitChildren { id: *id },
                        });
                    }
                    if children.len() != fractions.len() {
                        return Err(DockLayoutValidationError {
                            kind: DockLayoutValidationErrorKind::SplitFractionsLenMismatch {
                                id: *id,
                                children_len: children.len(),
                                fractions_len: fractions.len(),
                            },
                        });
                    }
                    for (index, f) in fractions.iter().copied().enumerate() {
                        if !f.is_finite() {
                            return Err(DockLayoutValidationError {
                                kind: DockLayoutValidationErrorKind::SplitNonFiniteFraction {
                                    id: *id,
                                    index,
                                    value: f,
                                },
                            });
                        }
                        if f < 0.0 {
                            return Err(DockLayoutValidationError {
                                kind: DockLayoutValidationErrorKind::SplitNegativeFraction {
                                    id: *id,
                                    index,
                                    value: f,
                                },
                            });
                        }
                    }
                }
            }
        }

        for node in by_id.values() {
            if let DockLayoutNode::Split { children, .. } = node {
                for child in children {
                    if !by_id.contains_key(child) {
                        return Err(DockLayoutValidationError {
                            kind: DockLayoutValidationErrorKind::MissingNodeId { id: *child },
                        });
                    }
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Mark {
            Visiting,
            Done,
        }
        let mut marks: HashMap<u32, Mark> = HashMap::new();

        for start in by_id.keys().copied() {
            if marks.contains_key(&start) {
                continue;
            }

            #[derive(Clone, Copy)]
            enum Step {
                Enter(u32),
                Exit(u32),
            }

            let mut stack: Vec<Step> = vec![Step::Enter(start)];
            while let Some(step) = stack.pop() {
                match step {
                    Step::Enter(id) => {
                        if marks.get(&id) == Some(&Mark::Done) {
                            continue;
                        }
                        if marks.get(&id) == Some(&Mark::Visiting) {
                            return Err(DockLayoutValidationError {
                                kind: DockLayoutValidationErrorKind::CycleDetected { id },
                            });
                        }
                        marks.insert(id, Mark::Visiting);
                        stack.push(Step::Exit(id));

                        if let Some(DockLayoutNode::Split { children, .. }) = by_id.get(&id) {
                            for child in children.iter().rev().copied() {
                                stack.push(Step::Enter(child));
                            }
                        }
                    }
                    Step::Exit(id) => {
                        marks.insert(id, Mark::Done);
                    }
                }
            }
        }

        for w in &self.windows {
            if !by_id.contains_key(&w.root) {
                return Err(DockLayoutValidationError {
                    kind: DockLayoutValidationErrorKind::WindowRootMissing {
                        logical_window_id: w.logical_window_id.clone(),
                        root: w.root,
                    },
                });
            }
            for f in &w.floatings {
                if !by_id.contains_key(&f.root) {
                    return Err(DockLayoutValidationError {
                        kind: DockLayoutValidationErrorKind::FloatingRootMissing {
                            logical_window_id: w.logical_window_id.clone(),
                            root: f.root,
                        },
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DockLayoutValidationError {
    pub kind: DockLayoutValidationErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DockLayoutValidationErrorKind {
    UnsupportedVersion {
        expected: u32,
        found: u32,
    },
    DuplicateNodeId {
        id: u32,
    },
    MissingNodeId {
        id: u32,
    },
    CycleDetected {
        id: u32,
    },
    EmptyTabs {
        id: u32,
    },
    TabsActiveOutOfBounds {
        id: u32,
        active: usize,
        len: usize,
    },
    EmptySplitChildren {
        id: u32,
    },
    SplitFractionsLenMismatch {
        id: u32,
        children_len: usize,
        fractions_len: usize,
    },
    SplitNonFiniteFraction {
        id: u32,
        index: usize,
        value: f32,
    },
    SplitNegativeFraction {
        id: u32,
        index: usize,
        value: f32,
    },
    WindowRootMissing {
        logical_window_id: String,
        root: u32,
    },
    FloatingRootMissing {
        logical_window_id: String,
        root: u32,
    },
}

impl std::fmt::Display for DockLayoutValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dock layout validation error: {:?}", self.kind)
    }
}

impl std::error::Error for DockLayoutValidationError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutWindow {
    pub logical_window_id: String,
    pub root: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<DockWindowPlacement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floatings: Vec<DockLayoutFloatingWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutFloatingWindow {
    /// Root node id within `nodes` for the floating dock tree (tabs/splits).
    pub root: u32,
    /// Floating window rect in logical pixels, relative to the host window's inner content origin.
    pub rect: DockRect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DockRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl DockRect {
    pub fn from_rect(rect: crate::Rect) -> Self {
        Self {
            x: rect.origin.x.0,
            y: rect.origin.y.0,
            w: rect.size.width.0,
            h: rect.size.height.0,
        }
    }

    pub fn to_rect(self) -> crate::Rect {
        crate::Rect::new(
            crate::Point::new(crate::Px(self.x), crate::Px(self.y)),
            crate::Size::new(crate::Px(self.w), crate::Px(self.h)),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockWindowPlacement {
    pub width: u32,
    pub height: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DockLayoutNode {
    #[serde(rename = "split")]
    Split {
        id: u32,
        axis: Axis,
        children: Vec<u32>,
        fractions: Vec<f32>,
    },
    #[serde(rename = "tabs")]
    Tabs {
        id: u32,
        tabs: Vec<PanelKey>,
        active: usize,
    },
}

#[derive(Debug, Clone)]
pub struct EditorDockLayoutSpec {
    pub left_tabs: Vec<PanelKey>,
    pub main_tabs: Vec<PanelKey>,
    pub bottom_tabs: Vec<PanelKey>,
    pub left_fraction: f32,
    pub main_fraction: f32,
    pub active_left: usize,
    pub active_main: usize,
    pub active_bottom: usize,
}

impl EditorDockLayoutSpec {
    pub fn new(
        left_tabs: Vec<PanelKey>,
        main_tabs: Vec<PanelKey>,
        bottom_tabs: Vec<PanelKey>,
    ) -> Self {
        Self {
            left_tabs,
            main_tabs,
            bottom_tabs,
            left_fraction: 0.26,
            main_fraction: 0.72,
            active_left: 0,
            active_main: 0,
            active_bottom: 0,
        }
    }

    pub fn with_fractions(mut self, left_fraction: f32, main_fraction: f32) -> Self {
        self.left_fraction = left_fraction;
        self.main_fraction = main_fraction;
        self
    }
}

/// Convenience helpers to build a `DockGraph` (runtime dock tree) without manually calling
/// `DockGraph::insert_node` everywhere.
#[derive(Debug, Default)]
pub struct DockLayoutBuilder {
    graph: DockGraph,
}

impl DockLayoutBuilder {
    pub fn new() -> Self {
        Self {
            graph: DockGraph::new(),
        }
    }

    pub fn into_graph(self) -> DockGraph {
        self.graph
    }

    pub fn tabs(&mut self, tabs: Vec<PanelKey>, active: usize) -> DockNodeId {
        self.graph.insert_node(DockNode::Tabs { tabs, active })
    }

    pub fn split_h(
        &mut self,
        left: DockNodeId,
        right: DockNodeId,
        left_fraction: f32,
    ) -> DockNodeId {
        self.graph.insert_node(DockNode::Split {
            axis: Axis::Horizontal,
            children: vec![left, right],
            fractions: vec![left_fraction, (1.0 - left_fraction).max(0.0)],
        })
    }

    pub fn split_v(
        &mut self,
        top: DockNodeId,
        bottom: DockNodeId,
        top_fraction: f32,
    ) -> DockNodeId {
        self.graph.insert_node(DockNode::Split {
            axis: Axis::Vertical,
            children: vec![top, bottom],
            fractions: vec![top_fraction, (1.0 - top_fraction).max(0.0)],
        })
    }

    pub fn set_window_root(&mut self, window: AppWindowId, root: DockNodeId) {
        self.graph.set_window_root(window, root);
    }

    /// Builds a Unity-like editor default layout:
    /// - left: (Hierarchy, Project, ...)
    /// - right: top (Scene, Game, ...), bottom (Inspector, Console/Text Probe, ...)
    pub fn default_editor_layout(window: AppWindowId, spec: EditorDockLayoutSpec) -> DockGraph {
        let mut b = DockLayoutBuilder::new();
        let left = b.tabs(spec.left_tabs, spec.active_left);
        let top = b.tabs(spec.main_tabs, spec.active_main);
        let bottom = b.tabs(spec.bottom_tabs, spec.active_bottom);
        let right = b.split_v(top, bottom, spec.main_fraction);
        let root = b.split_h(left, right, spec.left_fraction);
        b.set_window_root(window, root);
        b.into_graph()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_default_editor_layout_sets_window_root() {
        let window = AppWindowId::default();
        let spec = EditorDockLayoutSpec::new(
            vec![
                PanelKey::new("core.hierarchy"),
                PanelKey::new("core.project"),
            ],
            vec![PanelKey::new("core.scene"), PanelKey::new("core.game")],
            vec![
                PanelKey::new("core.inspector"),
                PanelKey::new("core.text_probe"),
            ],
        );
        let graph = DockLayoutBuilder::default_editor_layout(window, spec);
        assert!(graph.window_root(window).is_some());
    }

    #[test]
    fn validate_rejects_duplicate_node_ids() {
        let layout = DockLayout {
            layout_version: DOCK_LAYOUT_VERSION,
            windows: vec![DockLayoutWindow {
                logical_window_id: "main".into(),
                root: 1,
                placement: None,
                floatings: Vec::new(),
            }],
            nodes: vec![
                DockLayoutNode::Tabs {
                    id: 1,
                    tabs: vec![PanelKey::new("core.a")],
                    active: 0,
                },
                DockLayoutNode::Tabs {
                    id: 1,
                    tabs: vec![PanelKey::new("core.b")],
                    active: 0,
                },
            ],
        };

        let err = layout.validate().expect_err("duplicate ids should fail");
        assert!(matches!(
            err.kind,
            DockLayoutValidationErrorKind::DuplicateNodeId { id: 1 }
        ));
    }

    #[test]
    fn validate_rejects_cycles() {
        let layout = DockLayout {
            layout_version: DOCK_LAYOUT_VERSION,
            windows: vec![DockLayoutWindow {
                logical_window_id: "main".into(),
                root: 1,
                placement: None,
                floatings: Vec::new(),
            }],
            nodes: vec![DockLayoutNode::Split {
                id: 1,
                axis: Axis::Horizontal,
                children: vec![1],
                fractions: vec![1.0],
            }],
        };

        let err = layout.validate().expect_err("cycles should fail");
        assert!(matches!(
            err.kind,
            DockLayoutValidationErrorKind::CycleDetected { id: 1 }
        ));
    }

    #[test]
    fn validate_rejects_tabs_active_out_of_bounds() {
        let layout = DockLayout {
            layout_version: DOCK_LAYOUT_VERSION,
            windows: vec![DockLayoutWindow {
                logical_window_id: "main".into(),
                root: 1,
                placement: None,
                floatings: Vec::new(),
            }],
            nodes: vec![DockLayoutNode::Tabs {
                id: 1,
                tabs: vec![PanelKey::new("core.a")],
                active: 2,
            }],
        };

        let err = layout
            .validate()
            .expect_err("active out of bounds should fail");
        assert!(matches!(
            err.kind,
            DockLayoutValidationErrorKind::TabsActiveOutOfBounds {
                id: 1,
                active: 2,
                len: 1
            }
        ));
    }
}
