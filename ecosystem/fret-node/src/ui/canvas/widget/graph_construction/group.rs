use super::super::*;

const GROUP_WIDTH: f32 = 480.0;
const GROUP_HEIGHT: f32 = 320.0;

fn default_group_size() -> crate::core::CanvasSize {
    crate::core::CanvasSize {
        width: GROUP_WIDTH,
        height: GROUP_HEIGHT,
    }
}

fn centered_group_origin(at: CanvasPoint, size: crate::core::CanvasSize) -> CanvasPoint {
    CanvasPoint {
        x: at.x - 0.5 * size.width,
        y: at.y - 0.5 * size.height,
    }
}

fn create_group_ops(at: CanvasPoint) -> (crate::core::GroupId, Vec<GraphOp>) {
    let size = default_group_size();
    let origin = centered_group_origin(at, size);
    let group = crate::core::Group {
        title: "Group".to_string(),
        rect: crate::core::CanvasRect { origin, size },
        color: None,
    };
    let group_id = crate::core::GroupId::new();
    (
        group_id,
        vec![GraphOp::AddGroup {
            id: group_id,
            group,
        }],
    )
}

fn select_created_group_in_view_state(
    view_state: &mut NodeGraphViewState,
    group_id: crate::core::GroupId,
) {
    view_state.selected_nodes.clear();
    view_state.selected_edges.clear();
    view_state.selected_groups.clear();
    view_state.selected_groups.push(group_id);
    view_state.group_draw_order.retain(|id| *id != group_id);
    view_state.group_draw_order.push(group_id);
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn create_group_at<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let (group_id, ops) = create_group_ops(at);
        if self.commit_ops(host, window, Some("Create Group"), ops) {
            self.update_view_state(host, |view_state| {
                select_created_group_in_view_state(view_state, group_id);
            });
        }
    }
}

#[cfg(test)]
mod tests;
