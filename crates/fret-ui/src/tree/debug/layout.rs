use super::super::*;

#[derive(Debug, Clone, Default)]
pub struct UiDebugLayoutEngineMeasureHotspot {
    pub node: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub cache_hits: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub top_children: Vec<UiDebugLayoutEngineMeasureChildHotspot>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiDebugLayoutEngineMeasureChildHotspot {
    pub child: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutEngineSolve {
    pub root: NodeId,
    pub solve_time: Duration,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    pub measure_time: Duration,
    pub top_measures: Vec<UiDebugLayoutEngineMeasureHotspot>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugWidgetMeasureHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugPaintWidgetHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
    pub inclusive_scene_ops_delta: u32,
    pub exclusive_scene_ops_delta: u32,
}
