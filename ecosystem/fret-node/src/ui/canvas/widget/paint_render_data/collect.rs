use super::*;
impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_render_data<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: Arc<CanvasGeometry>,
        index: Arc<CanvasSpatialDerived>,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        hovered_edge: Option<EdgeId>,
        include_groups: bool,
        include_nodes: bool,
        include_edges: bool,
    ) -> RenderData {
        let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
        let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();
        let selected_groups: HashSet<crate::core::GroupId> =
            snapshot.selected_groups.iter().copied().collect();

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let cull = render_cull_rect;
        let this = self;

        this.graph
            .read_ref(host, |graph| {
                let mut out = RenderData::default();

                let geom = geom.as_ref();
                let index = index.as_ref();
                let label_overhead = this.node_render_label_overhead();

                if include_groups {
                    this.collect_group_render_data(
                        graph,
                        snapshot,
                        &selected_groups,
                        cull,
                        &mut out,
                    );
                }

                if include_nodes {
                    out.metrics.node_total = geom.order.len();
                    let (node_candidates, visible_nodes) =
                        this.visible_node_ids_for_render(geom, index, cull);
                    out.metrics.node_candidates = node_candidates;

                    for node in visible_nodes {
                        this.append_node_render_data(
                            graph,
                            geom,
                            presenter,
                            &mut out,
                            node,
                            selected.contains(&node),
                            zoom,
                            label_overhead,
                        );
                    }
                }

                if include_edges {
                    this.collect_edge_render_data(
                        graph,
                        snapshot,
                        geom,
                        index,
                        presenter,
                        &selected_edges,
                        hovered_edge,
                        cull,
                        zoom,
                        &mut out,
                    );
                }

                out
            })
            .unwrap_or_default()
    }
}
