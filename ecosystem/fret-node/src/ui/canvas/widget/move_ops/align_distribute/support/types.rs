use crate::ui::canvas::widget::move_ops::*;

#[derive(Clone, Copy)]
pub(super) enum ElementId {
    Node(GraphNodeId),
    Group(crate::core::GroupId),
}

#[derive(Clone, Copy)]
pub(in super::super) struct Elem {
    pub(super) id: ElementId,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
}

pub(in super::super) struct TargetBounds {
    pub(super) left: f32,
    pub(super) top: f32,
    pub(super) right: f32,
    pub(super) bottom: f32,
    pub(super) center_x: f32,
    pub(super) center_y: f32,
}

pub(in super::super) struct DeltaPlan {
    pub(in super::super) per_group_delta:
        std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    pub(in super::super) per_node_delta: std::collections::HashMap<GraphNodeId, CanvasPoint>,
}

#[derive(Clone, Copy)]
pub(in super::super) struct ModeFlags {
    pub(in super::super) aligns: bool,
    pub(in super::super) affects_x: bool,
    pub(in super::super) affects_y: bool,
}

impl ModeFlags {
    pub(in super::super) fn for_mode(mode: AlignDistributeMode) -> Self {
        Self {
            aligns: matches!(
                mode,
                AlignDistributeMode::AlignLeft
                    | AlignDistributeMode::AlignRight
                    | AlignDistributeMode::AlignTop
                    | AlignDistributeMode::AlignBottom
                    | AlignDistributeMode::AlignCenterX
                    | AlignDistributeMode::AlignCenterY
            ),
            affects_x: matches!(
                mode,
                AlignDistributeMode::AlignLeft
                    | AlignDistributeMode::AlignRight
                    | AlignDistributeMode::AlignCenterX
            ),
            affects_y: matches!(
                mode,
                AlignDistributeMode::AlignTop
                    | AlignDistributeMode::AlignBottom
                    | AlignDistributeMode::AlignCenterY
            ),
        }
    }
}
