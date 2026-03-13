use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn record_recent_kind(&mut self, kind: &NodeKindKey) {
        super::searcher_rows::record_recent_kind(self, kind)
    }

    pub(super) fn searcher_is_selectable_row(row: &SearcherRow) -> bool {
        super::searcher_rows::searcher_is_selectable_row(row)
    }

    pub(super) fn searcher_first_selectable_row(rows: &[SearcherRow]) -> usize {
        super::searcher_rows::searcher_first_selectable_row(rows)
    }

    pub(super) fn rebuild_searcher_rows(searcher: &mut SearcherState) {
        super::searcher_rows::rebuild_searcher_rows::<M>(searcher)
    }

    pub(super) fn ensure_searcher_active_visible(searcher: &mut SearcherState) {
        super::searcher_rows::ensure_searcher_active_visible(searcher)
    }

    pub(super) fn try_activate_searcher_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
    ) -> bool {
        super::searcher_row_activation::try_activate_searcher_row(self, cx, row_ix)
    }

    pub(super) fn open_insert_node_picker<H: UiHost>(&mut self, host: &mut H, at: CanvasPoint) {
        super::searcher_picker::open_insert_node_picker(self, host, at)
    }

    pub(super) fn open_connection_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
    ) {
        super::searcher_picker::open_connection_insert_node_picker(self, host, from, at)
    }

    pub(super) fn open_edge_insert_node_picker<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        edge: EdgeId,
        invoked_at: Point,
    ) {
        super::searcher_picker::open_edge_insert_node_picker(self, host, window, edge, invoked_at)
    }
}
