use super::*;

mod label;
mod plan;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn align_or_distribute_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        mode: AlignDistributeMode,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let geom = self.canvas_geometry(&*host, snapshot);

        let ops = self
            .graph
            .read_ref(host, |g| {
                plan::plan_ops(
                    g,
                    geom.as_ref(),
                    &selected_nodes,
                    &selected_groups,
                    snapshot,
                    mode,
                )
            })
            .ok()
            .unwrap_or_default();

        if ops.is_empty() {
            return;
        }

        let label = label::label_for_mode(mode);
        let _ = self.commit_ops(host, window, Some(label), ops);
    }
}
