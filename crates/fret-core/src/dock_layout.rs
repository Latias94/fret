use serde::{Deserialize, Serialize};

use crate::{Axis, PanelKey};

pub const DOCK_LAYOUT_VERSION_V1: u32 = 1;

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
pub struct DockLayoutWindowV1 {
    pub logical_window_id: String,
    pub root: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<DockWindowPlacementV1>,
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
