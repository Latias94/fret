use super::*;
use crate::ui::canvas::geometry::node_size_default_px;
use fret_core::TextStyle;

pub(super) fn measure_node_sizes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    nodes: &[super::auto_measure_collect::NodeMeasureInput],
) -> Vec<(GraphNodeId, (f32, f32))> {
    let text_style = canvas.style.geometry.context_menu_text_style.clone();
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: cx.scale_factor,
    };

    let node_pad = canvas.style.geometry.node_padding;
    let pin_gap = 8.0;
    let pin_r = canvas.style.geometry.pin_radius;
    let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

    let mut measured: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let title_w = text_width(canvas, cx, &text_style, constraints, &node.title);
        let max_in = max_text_width(canvas, cx, &text_style, constraints, &node.inputs);
        let max_out = max_text_width(canvas, cx, &text_style, constraints, &node.outputs);

        let w_by_title = title_w + 2.0 * node_pad;
        let w_by_labels = max_in.max(max_out) + label_overhead;
        let w = canvas
            .style
            .geometry
            .node_width
            .max(w_by_title)
            .max(w_by_labels);

        let (_default_w, h) =
            node_size_default_px(node.inputs.len(), node.outputs.len(), &canvas.style);
        measured.push((node.node, (w, h)));
    }
    measured
}

pub(super) fn apply_measured_sizes<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    measured: Vec<(GraphNodeId, (f32, f32))>,
) {
    let keep: std::collections::BTreeSet<GraphNodeId> = measured.iter().map(|(n, _)| *n).collect();

    let _ = canvas
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

fn max_text_width<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    text_style: &TextStyle,
    constraints: TextConstraints,
    labels: &[Arc<str>],
) -> f32 {
    labels
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| text_width(canvas, cx, text_style, constraints, s))
        .fold(0.0, f32::max)
}

fn text_width<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    text_style: &TextStyle,
    constraints: TextConstraints,
    text: &Arc<str>,
) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    canvas
        .paint_cache
        .text_metrics(cx.services, text.clone(), text_style, constraints)
        .size
        .width
        .0
}
