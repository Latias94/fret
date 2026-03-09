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

                let label_overhead = this.node_render_label_overhead();

                for node in selected_nodes.iter().copied() {
                    let Some(node_geom) = geom.nodes.get(&node) else {
                        continue;
                    };
                    if cull.is_some_and(|c| !rects_intersect(node_geom.rect, c)) {
                        continue;
                    }

                    this.append_node_render_data(
                        graph,
                        geom,
                        presenter,
                        &mut out,
                        node,
                        true,
                        zoom,
                        label_overhead,
                    );
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
