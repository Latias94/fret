use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_selected_nodes_render_data<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        render_cull_rect: Option<Rect>,
        zoom: f32,
    ) -> RenderData {
        let selected_nodes = snapshot.selected_nodes.clone();
        if selected_nodes.is_empty() {
            return RenderData::default();
        }

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let cull = render_cull_rect;

        let this = self;
        this.graph
            .read_ref(host, |graph| {
                let mut out = RenderData::default();

                let node_pad = this.style.node_padding;
                let pin_gap = 8.0;
                let pin_r = this.style.pin_radius;
                let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

                for node in selected_nodes.iter().copied() {
                    let Some(node_geom) = geom.nodes.get(&node) else {
                        continue;
                    };
                    if cull.is_some_and(|c| !rects_intersect(node_geom.rect, c)) {
                        continue;
                    }

                    let title = presenter.node_title(graph, node);
                    let (inputs, outputs) = node_ports(graph, node);
                    let pin_rows = inputs.len().max(outputs.len());
                    let body = presenter.node_body_label(graph, node);
                    let resize_handles = presenter.node_resize_handles(graph, node, &this.style);
                    out.nodes.push((
                        node,
                        node_geom.rect,
                        true,
                        title,
                        body,
                        pin_rows,
                        resize_handles,
                    ));

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
                        let color = presenter.port_color(graph, port_id, &this.style);
                        out.pins.push((port_id, handle.bounds, color));
                    }
                }

                out.nodes.sort_unstable_by(|a, b| {
                    let rank_a = geom.node_rank.get(&a.0).copied().unwrap_or(u32::MAX);
                    let rank_b = geom.node_rank.get(&b.0).copied().unwrap_or(u32::MAX);
                    rank_a.cmp(&rank_b).then_with(|| a.0.cmp(&b.0))
                });

                out
            })
            .unwrap_or_default()
    }
}
