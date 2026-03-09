use super::*;
use crate::ui::NodeChromeHint;
use crate::ui::PortChromeHint;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn visible_node_ids_for_render(
        &self,
        geom: &CanvasGeometry,
        index: &CanvasSpatialDerived,
        cull: Option<Rect>,
    ) -> (usize, Vec<GraphNodeId>) {
        if let Some(cull_rect) = cull {
            let mut candidates: Vec<GraphNodeId> = Vec::new();
            index.query_nodes_in_rect(cull_rect, &mut candidates);

            let mut visible: Vec<GraphNodeId> = Vec::with_capacity(candidates.len());
            for node in candidates.iter().copied() {
                let Some(node_geom) = geom.nodes.get(&node) else {
                    continue;
                };
                if rects_intersect(node_geom.rect, cull_rect) {
                    visible.push(node);
                }
            }

            visible.sort_unstable_by_key(|node| {
                (geom.node_rank.get(node).copied().unwrap_or(u32::MAX), *node)
            });

            (candidates.len(), visible)
        } else {
            (geom.order.len(), geom.order.clone())
        }
    }

    pub(in super::super) fn node_render_label_overhead(&self) -> f32 {
        let node_pad = self.style.geometry.node_padding;
        let pin_gap = 8.0;
        let pin_r = self.style.geometry.pin_radius;
        2.0 * node_pad + 2.0 * (pin_r + pin_gap)
    }

    pub(in super::super) fn append_node_render_data(
        &self,
        graph: &Graph,
        geom: &CanvasGeometry,
        presenter: &dyn NodeGraphPresenter,
        out: &mut RenderData,
        node: GraphNodeId,
        is_selected: bool,
        zoom: f32,
        label_overhead: f32,
    ) {
        let Some(node_geom) = geom.nodes.get(&node) else {
            return;
        };

        let hint = if let Some(skin) = self.skin.as_ref() {
            skin.node_chrome_hint(graph, node, &self.style, is_selected)
        } else {
            NodeChromeHint::default()
        };
        let title = presenter.node_title(graph, node);
        let (inputs, outputs) = node_ports(graph, node);
        let pin_rows = inputs.len().max(outputs.len());
        let body = presenter.node_body_label(graph, node);
        let resize_handles = presenter.node_resize_handles(graph, node, &self.style);
        out.nodes.push((
            node,
            node_geom.rect,
            is_selected,
            title,
            body,
            pin_rows,
            resize_handles,
            hint,
        ));
        out.metrics.node_visible = out.metrics.node_visible.saturating_add(1);

        let screen_w = node_geom.rect.size.width.0 * zoom;
        let screen_max = (screen_w - label_overhead).max(0.0);
        let max_w = Px(screen_max / zoom);

        for port_id in inputs.iter().chain(outputs.iter()).copied() {
            let Some(handle) = geom.ports.get(&port_id) else {
                continue;
            };
            out.port_centers.insert(port_id, handle.center);
            out.port_labels.insert(
                port_id,
                PortLabelRender {
                    label: presenter.port_label(graph, port_id),
                    dir: handle.dir,
                    max_width: max_w,
                },
            );
            let color = presenter.port_color(graph, port_id, &self.style);
            let hint = if let Some(skin) = self.skin.as_ref() {
                skin.port_chrome_hint(graph, port_id, &self.style, color)
            } else {
                PortChromeHint::default()
            };
            let fill = hint.fill.unwrap_or(color);
            out.pins.push((port_id, handle.bounds, fill, hint));
        }
    }
}
