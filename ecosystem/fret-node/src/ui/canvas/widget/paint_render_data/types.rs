use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::canvas::widget) struct RenderMetrics {
    pub(in crate::ui::canvas::widget) group_total: usize,
    pub(in crate::ui::canvas::widget) group_candidates: usize,
    pub(in crate::ui::canvas::widget) group_visible: usize,
    pub(in crate::ui::canvas::widget) node_total: usize,
    pub(in crate::ui::canvas::widget) node_candidates: usize,
    pub(in crate::ui::canvas::widget) node_visible: usize,
    pub(in crate::ui::canvas::widget) edge_total: usize,
    pub(in crate::ui::canvas::widget) edge_candidates: usize,
    pub(in crate::ui::canvas::widget) edge_visible: usize,
}

#[derive(Debug, Default)]
pub(in super::super) struct RenderData {
    pub(in super::super) metrics: RenderMetrics,
    pub(in super::super) groups: Vec<(Rect, Arc<str>, bool)>,
    pub(in super::super) edges: Vec<EdgeRender>,
    pub(in super::super) nodes: Vec<(
        GraphNodeId,
        Rect,
        bool,
        Arc<str>,
        Option<Arc<str>>,
        usize,
        NodeResizeHandleSet,
    )>,
    pub(in super::super) pins: Vec<(PortId, Rect, Color)>,
    pub(in super::super) port_labels: HashMap<PortId, PortLabelRender>,
    pub(in super::super) port_centers: HashMap<PortId, Point>,
}

#[derive(Debug, Clone)]
pub(in super::super) struct EdgeRender {
    pub(in super::super) id: EdgeId,
    pub(in super::super) rank: u32,
    pub(in super::super) from: Point,
    pub(in super::super) to: Point,
    pub(in super::super) color: Color,
    pub(in super::super) hint: EdgeRenderHint,
    pub(in super::super) selected: bool,
    pub(in super::super) hovered: bool,
}

#[derive(Debug, Clone)]
pub(in super::super) struct PortLabelRender {
    pub(in super::super) label: Arc<str>,
    pub(in super::super) dir: PortDirection,
    pub(in super::super) max_width: Px,
}
