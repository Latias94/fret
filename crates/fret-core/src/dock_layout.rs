use serde::{Deserialize, Serialize};

use crate::{AppWindowId, Axis, DockGraph, DockNode, DockNodeId, PanelKey};

pub const DOCK_LAYOUT_VERSION_V1: u32 = 1;
pub const DOCK_LAYOUT_VERSION_V2: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutV1 {
    pub layout_version: u32,
    pub windows: Vec<DockLayoutWindowV1>,
    pub nodes: Vec<DockLayoutNodeV1>,
}

impl DockLayoutV1 {
    pub fn new_v1(windows: Vec<DockLayoutWindowV1>, nodes: Vec<DockLayoutNodeV1>) -> Self {
        Self {
            layout_version: DOCK_LAYOUT_VERSION_V1,
            windows,
            nodes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutV2 {
    pub layout_version: u32,
    pub windows: Vec<DockLayoutWindowV2>,
    pub nodes: Vec<DockLayoutNodeV1>,
}

impl DockLayoutV2 {
    pub fn new_v2(windows: Vec<DockLayoutWindowV2>, nodes: Vec<DockLayoutNodeV1>) -> Self {
        Self {
            layout_version: DOCK_LAYOUT_VERSION_V2,
            windows,
            nodes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutWindowV1 {
    pub logical_window_id: String,
    pub root: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<DockWindowPlacementV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutWindowV2 {
    pub logical_window_id: String,
    pub root: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<DockWindowPlacementV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floatings: Vec<DockLayoutFloatingWindowV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutFloatingWindowV2 {
    /// Root node id within `nodes` for the floating dock tree (tabs/splits).
    pub root: u32,
    /// Floating window rect in logical pixels, relative to the host window's inner content origin.
    pub rect: DockRectV2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DockRectV2 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl DockRectV2 {
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
pub struct DockWindowPlacementV1 {
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
pub enum DockLayoutNodeV1 {
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
}
