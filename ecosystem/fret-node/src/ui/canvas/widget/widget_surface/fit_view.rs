use super::*;

use crate::ui::NodeGraphFitViewOptions;
use crate::ui::view_queue::NodeGraphViewQueueFitViewOptions;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn fit_view_on_mount_node_ids<H: UiHost>(
        &self,
        host: &mut H,
        include_hidden_nodes: bool,
    ) -> Vec<GraphNodeId> {
        self.graph
            .read_ref(host, |graph| {
                graph
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        if node.hidden && !include_hidden_nodes {
                            None
                        } else {
                            Some(*id)
                        }
                    })
                    .collect()
            })
            .ok()
            .unwrap_or_default()
    }

    pub fn with_fit_view_on_mount(self) -> Self {
        self.with_fit_view_on_mount_options(NodeGraphFitViewOptions::default())
    }

    pub fn with_fit_view_on_mount_options(mut self, options: NodeGraphFitViewOptions) -> Self {
        self.fit_view_on_mount = Some(NodeGraphViewQueueFitViewOptions::from(options));
        self.did_fit_view_on_mount = false;
        self
    }

    pub(in super::super) fn maybe_fit_view_on_mount<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        did_drain_view_queue: bool,
    ) -> bool {
        if did_drain_view_queue || self.did_fit_view_on_mount {
            return false;
        }

        let Some(options) = self.fit_view_on_mount.clone() else {
            return false;
        };

        let node_ids = self.fit_view_on_mount_node_ids(host, options.include_hidden_nodes);
        if node_ids.is_empty() {
            return false;
        }

        let did =
            self.frame_nodes_in_view_with_options(host, window, bounds, &node_ids, Some(&options));
        if did {
            self.did_fit_view_on_mount = true;
        }
        did
    }
}
