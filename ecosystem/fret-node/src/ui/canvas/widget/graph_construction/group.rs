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
mod tests {
    use super::*;

    #[test]
    fn centered_group_origin_offsets_half_size() {
        let size = default_group_size();
        let origin = centered_group_origin(CanvasPoint { x: 100.0, y: 200.0 }, size);

        assert_eq!(
            origin,
            CanvasPoint {
                x: 100.0 - 0.5 * GROUP_WIDTH,
                y: 200.0 - 0.5 * GROUP_HEIGHT,
            }
        );
    }

    #[test]
    fn create_group_ops_builds_single_add_group_op() {
        let (group_id, ops) = create_group_ops(CanvasPoint { x: 100.0, y: 200.0 });

        let [GraphOp::AddGroup { id, group }] = ops.as_slice() else {
            panic!("expected one add group op");
        };
        assert_eq!(*id, group_id);
        assert_eq!(group.title, "Group");
        assert_eq!(group.rect.size, default_group_size());
        assert_eq!(
            group.rect.origin,
            CanvasPoint {
                x: 100.0 - 0.5 * GROUP_WIDTH,
                y: 200.0 - 0.5 * GROUP_HEIGHT,
            }
        );
    }

    #[test]
    fn select_created_group_in_view_state_clears_other_selection_kinds() {
        let group_id = crate::core::GroupId::from_u128(9);
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_nodes.push(GraphNodeId::from_u128(1));
        view_state.selected_edges.push(EdgeId::from_u128(2));
        view_state
            .selected_groups
            .push(crate::core::GroupId::from_u128(3));
        view_state.group_draw_order.extend([
            crate::core::GroupId::from_u128(1),
            group_id,
            crate::core::GroupId::from_u128(2),
        ]);

        select_created_group_in_view_state(&mut view_state, group_id);

        assert!(view_state.selected_nodes.is_empty());
        assert!(view_state.selected_edges.is_empty());
        assert_eq!(view_state.selected_groups, vec![group_id]);
        assert_eq!(
            view_state.group_draw_order,
            vec![
                crate::core::GroupId::from_u128(1),
                crate::core::GroupId::from_u128(2),
                group_id,
            ]
        );
    }
}
