use crate::ui::canvas::widget::move_ops::*;

use super::types::{DeltaPlan, Elem, ElementId, TargetBounds};

pub(in super::super) fn plan_deltas(
    elems: &[Elem],
    targets: &TargetBounds,
    mode: AlignDistributeMode,
) -> Option<DeltaPlan> {
    let mut per_group_delta = std::collections::HashMap::new();
    let mut per_node_delta = std::collections::HashMap::new();

    match mode {
        AlignDistributeMode::AlignLeft => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: targets.left - elem.x,
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignRight => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: (targets.right - elem.w) - elem.x,
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignTop => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: targets.top - elem.y,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignBottom => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: (targets.bottom - elem.h) - elem.y,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignCenterX => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: targets.center_x - (elem.x + 0.5 * elem.w),
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::AlignCenterY => {
            for elem in elems {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: targets.center_y - (elem.y + 0.5 * elem.h),
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::DistributeX => {
            if elems.len() < 3 {
                return None;
            }
            let mut sorted = elems.to_vec();
            sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            let first = sorted.first().copied().unwrap();
            let last = sorted.last().copied().unwrap();
            let c0 = first.x + 0.5 * first.w;
            let c1 = last.x + 0.5 * last.w;
            let span = c1 - c0;
            if !span.is_finite() || span.abs() <= 1.0e-6 {
                return None;
            }
            let step = span / (sorted.len() as f32 - 1.0);
            for (index, elem) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: (c0 + (index as f32) * step) - (elem.x + 0.5 * elem.w),
                        y: 0.0,
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
        AlignDistributeMode::DistributeY => {
            if elems.len() < 3 {
                return None;
            }
            let mut sorted = elems.to_vec();
            sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
            let first = sorted.first().copied().unwrap();
            let last = sorted.last().copied().unwrap();
            let c0 = first.y + 0.5 * first.h;
            let c1 = last.y + 0.5 * last.h;
            let span = c1 - c0;
            if !span.is_finite() || span.abs() <= 1.0e-6 {
                return None;
            }
            let step = span / (sorted.len() as f32 - 1.0);
            for (index, elem) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                assign_delta(
                    elem.id,
                    CanvasPoint {
                        x: 0.0,
                        y: (c0 + (index as f32) * step) - (elem.y + 0.5 * elem.h),
                    },
                    &mut per_group_delta,
                    &mut per_node_delta,
                );
            }
        }
    }

    Some(DeltaPlan {
        per_group_delta,
        per_node_delta,
    })
}

fn assign_delta(
    element_id: ElementId,
    delta: CanvasPoint,
    per_group_delta: &mut std::collections::HashMap<crate::core::GroupId, CanvasPoint>,
    per_node_delta: &mut std::collections::HashMap<GraphNodeId, CanvasPoint>,
) {
    if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
        return;
    }
    match element_id {
        ElementId::Group(id) => {
            per_group_delta.insert(id, delta);
        }
        ElementId::Node(id) => {
            per_node_delta.insert(id, delta);
        }
    }
}
