use std::collections::HashMap;

use super::super::*;
use crate::ui::canvas::widget::paint_render_data::EdgeRender;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn collect_custom_edge_paths<H: UiHost>(
        &self,
        host: &H,
        edges: &[EdgeRender],
        zoom: f32,
    ) -> HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath> {
        let Some(edge_types) = self.edge_types.as_ref().filter(|t| t.has_custom_paths()) else {
            return HashMap::new();
        };
        let style = self.style.clone();
        self.graph
            .read_ref(host, |graph| {
                let mut out: HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath> =
                    HashMap::new();
                for edge in edges {
                    if let Some(custom) = edge_types.custom_path(
                        graph,
                        edge.id,
                        &style,
                        &edge.hint,
                        crate::ui::edge_types::EdgePathInput {
                            from: edge.from,
                            to: edge.to,
                            zoom,
                        },
                    ) {
                        out.insert(edge.id, custom);
                    }
                }
                out
            })
            .ok()
            .unwrap_or_default()
    }
}
