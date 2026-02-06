use super::*;

#[derive(Clone, Copy)]
pub(super) struct EdgePathContext<'a> {
    pub(super) style: &'a NodeGraphStyle,
    presenter: &'a dyn NodeGraphPresenter,
    edge_types: Option<&'a NodeGraphEdgeTypes>,
}

impl<'a> EdgePathContext<'a> {
    pub(super) fn new(
        style: &'a NodeGraphStyle,
        presenter: &'a dyn NodeGraphPresenter,
        edge_types: Option<&'a NodeGraphEdgeTypes>,
    ) -> Self {
        Self {
            style,
            presenter,
            edge_types,
        }
    }

    pub(super) fn has_custom_paths(self) -> bool {
        self.edge_types
            .is_some_and(|edge_types| edge_types.has_custom_paths())
    }

    pub(super) fn edge_render_hint(self, graph: &Graph, edge_id: EdgeId) -> EdgeRenderHint {
        let base = self.presenter.edge_render_hint(graph, edge_id, self.style);
        if let Some(edge_types) = self.edge_types {
            edge_types.apply(graph, edge_id, self.style, base)
        } else {
            base
        }
    }

    pub(super) fn edge_render_hint_normalized(
        self,
        graph: &Graph,
        edge_id: EdgeId,
    ) -> EdgeRenderHint {
        self.edge_render_hint(graph, edge_id).normalized()
    }

    pub(super) fn edge_custom_path(
        self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> Option<crate::ui::edge_types::EdgeCustomPath> {
        self.edge_types?.custom_path(
            graph,
            edge_id,
            self.style,
            hint,
            crate::ui::edge_types::EdgePathInput { from, to, zoom },
        )
    }
}
