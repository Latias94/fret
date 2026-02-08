use super::*;
use crate::ui::canvas::geometry::node_size_default_px;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn update_auto_measured_node_sizes<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let graph_rev = self.graph.revision(cx.app).unwrap_or(0);
        let scale_bits = cx.scale_factor.to_bits();
        let key = (graph_rev, scale_bits);
        if self.auto_measured_key == Some(key) {
            return;
        }
        self.auto_measured_key = Some(key);

        #[derive(Debug)]
        struct NodeMeasureInput {
            node: GraphNodeId,
            title: Arc<str>,
            inputs: Vec<Arc<str>>,
            outputs: Vec<Arc<str>>,
        }

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let Some(nodes) = self
            .graph
            .read_ref(cx.app, |graph| {
                let mut out: Vec<NodeMeasureInput> = Vec::new();
                out.reserve(graph.nodes.len());

                for node_id in graph.nodes.keys().copied() {
                    let title = presenter.node_title(graph, node_id);
                    let (inputs, outputs) = node_ports(graph, node_id);
                    let inputs = inputs
                        .into_iter()
                        .map(|p| presenter.port_label(graph, p))
                        .collect();
                    let outputs = outputs
                        .into_iter()
                        .map(|p| presenter.port_label(graph, p))
                        .collect();
                    out.push(NodeMeasureInput {
                        node: node_id,
                        title,
                        inputs,
                        outputs,
                    });
                }

                out
            })
            .ok()
        else {
            return;
        };

        let text_style = self.style.context_menu_text_style.clone();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let node_pad = self.style.node_padding;
        let pin_gap = 8.0;
        let pin_r = self.style.pin_radius;
        let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

        let mut measured: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let title_w = if node.title.is_empty() {
                0.0
            } else {
                self.paint_cache
                    .text_metrics(cx.services, node.title.clone(), &text_style, constraints)
                    .size
                    .width
                    .0
            };
            let max_in = node
                .inputs
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    self.paint_cache
                        .text_metrics(cx.services, s.clone(), &text_style, constraints)
                        .size
                        .width
                        .0
                })
                .fold(0.0, f32::max);
            let max_out = node
                .outputs
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    self.paint_cache
                        .text_metrics(cx.services, s.clone(), &text_style, constraints)
                        .size
                        .width
                        .0
                })
                .fold(0.0, f32::max);

            let w_by_title = title_w + 2.0 * node_pad;
            let w_by_labels = max_in.max(max_out) + label_overhead;
            let w = self.style.node_width.max(w_by_title).max(w_by_labels);

            let (_default_w, h) =
                node_size_default_px(node.inputs.len(), node.outputs.len(), &self.style);

            measured.push((node.node, (w, h)));
        }

        let keep: std::collections::BTreeSet<GraphNodeId> =
            measured.iter().map(|(n, _)| *n).collect();

        let _ = self
            .auto_measured
            .update_if_changed(|node_sizes, _anchors| {
                let mut changed = false;

                node_sizes.retain(|id, _| {
                    let ok = keep.contains(id);
                    if !ok {
                        changed = true;
                    }
                    ok
                });

                for (node, size) in &measured {
                    let needs = match node_sizes.get(node) {
                        Some(old) => (old.0 - size.0).abs() > 0.1 || (old.1 - size.1).abs() > 0.1,
                        None => true,
                    };
                    if needs {
                        node_sizes.insert(*node, *size);
                        changed = true;
                    }
                }

                changed
            });
    }
}
