use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn drain_edit_queue<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) {
        let Some(queue) = self.edit_queue.as_ref() else {
            return;
        };
        let Some(rev) = queue.revision(host) else {
            return;
        };
        if self.edit_queue_key == Some(rev) {
            return;
        }
        self.edit_queue_key = Some(rev);

        let Ok(txs) = queue.update(host, |q, _cx| q.drain()) else {
            return;
        };
        for tx in txs {
            let _ = self.commit_transaction(host, window, &tx);
        }
    }

    pub(in super::super) fn drain_view_queue<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) -> bool {
        let Some(queue) = self.view_queue.as_ref() else {
            return false;
        };
        let Some(rev) = queue.revision(host) else {
            return false;
        };
        if self.view_queue_key == Some(rev) {
            return false;
        }
        self.view_queue_key = Some(rev);

        let Ok(reqs) = queue.update(host, |q, _cx| q.drain()) else {
            return false;
        };
        if reqs.is_empty() {
            return false;
        }

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let mut did = false;
        for req in reqs {
            match req {
                crate::ui::NodeGraphViewRequest::FrameNodes { nodes, options } => {
                    did |= self.frame_nodes_in_view_with_options(
                        host,
                        window,
                        bounds,
                        &nodes,
                        Some(&options),
                    );
                }
                crate::ui::NodeGraphViewRequest::SetViewport { pan, zoom, options } => {
                    did |= self.set_viewport_with_options(host, window, pan, zoom, Some(&options));
                }
            }
        }
        did
    }
}
