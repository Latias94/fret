use super::*;

pub(super) fn build_semantics_value<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
