use super::*;

pub(super) fn sync_semantics<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut SemanticsCx<'_, H>,
) {
    let theme = Theme::global(&*cx.app).snapshot();
    canvas.sync_style_from_color_mode(theme, None);
    canvas.sync_skin(None);
    canvas.sync_paint_overrides(None);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    cx.set_role(fret_core::SemanticsRole::Viewport);
    cx.set_focusable(true);
    cx.set_label(canvas.presenter.a11y_canvas_label().as_ref());
    cx.set_test_id("node_graph.canvas");
    cx.set_active_descendant(active_descendant(canvas, cx));
    cx.set_value(build_semantics_value(canvas, cx, &snapshot));
}

fn active_descendant<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &SemanticsCx<'_, H>,
) -> Option<fret_core::NodeId> {
    match (
        canvas.interaction.focused_port.is_some(),
        canvas.interaction.focused_edge.is_some(),
        canvas.interaction.focused_node.is_some(),
    ) {
        (true, _, _) => cx.children.first().copied(),
        (false, true, _) => cx.children.get(1).copied(),
        (false, false, true) => cx.children.get(2).copied(),
        _ => None,
    }
}

fn build_semantics_value<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &SemanticsCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> String {
    let (focused_node, focused_port, focused_edge) = (
        canvas.interaction.focused_node,
        canvas.interaction.focused_port,
        canvas.interaction.focused_edge,
    );
    let style = canvas.style.clone();
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut parts: Vec<String> = vec![
                format!("zoom {:.3}", snapshot.zoom),
                format!("panning {}", canvas.interaction.panning),
                format!(
                    "selected nodes {}, edges {}, groups {}",
                    snapshot.selected_nodes.len(),
                    snapshot.selected_edges.len(),
                    snapshot.selected_groups.len(),
                ),
            ];

            if canvas.interaction.wire_drag.is_some() {
                parts.push("connecting".to_string());
            }

            push_focus_label(
                &mut parts,
                focused_node,
                "focused node",
                |id| format!("focused node {:?}", id),
                |graph, id| canvas.presenter.a11y_node_label(graph, id),
                graph,
            );
            push_focus_label(
                &mut parts,
                focused_port,
                "focused port",
                |id| format!("focused port {:?}", id),
                |graph, id| canvas.presenter.a11y_port_label(graph, id),
                graph,
            );
            push_focus_label(
                &mut parts,
                focused_edge,
                "focused edge",
                |id| format!("focused edge {:?}", id),
                |graph, id| canvas.presenter.a11y_edge_label(graph, id, &style),
                graph,
            );

            parts.join("; ")
        })
        .ok()
        .unwrap_or_else(|| format!("zoom {:.3}", snapshot.zoom))
}

fn push_focus_label<Id, GraphLabel, Fallback>(
    parts: &mut Vec<String>,
    focused: Option<Id>,
    prefix: &str,
    fallback: Fallback,
    graph_label: GraphLabel,
    graph: &Graph,
) where
    Id: Copy + core::fmt::Debug,
    GraphLabel: Fn(&Graph, Id) -> Option<Arc<str>>,
    Fallback: Fn(Id) -> String,
{
    let Some(id) = focused else {
        return;
    };
    if let Some(label) = graph_label(graph, id) {
        parts.push(format!("{prefix} {label}"));
    } else {
        parts.push(fallback(id));
    }
}
