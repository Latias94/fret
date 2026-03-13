use std::collections::BTreeMap;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, Graph, Group, GroupId};

pub(super) fn best_parent_group(
    rect: Rect,
    graph: &Graph,
    group_overrides: &BTreeMap<GroupId, CanvasRect>,
) -> Option<GroupId> {
    let rect_min_x = rect.origin.x.0;
    let rect_min_y = rect.origin.y.0;
    let rect_max_x = rect.origin.x.0 + rect.size.width.0;
    let rect_max_y = rect.origin.y.0 + rect.size.height.0;

    let mut best: Option<(GroupId, f32)> = None;
    for (group_id, group) in &graph.groups {
        let group_rect = group_overrides.get(group_id).copied().unwrap_or(group.rect);
        let gx0 = group_rect.origin.x;
        let gy0 = group_rect.origin.y;
        let gx1 = group_rect.origin.x + group_rect.size.width;
        let gy1 = group_rect.origin.y + group_rect.size.height;
        if rect_min_x >= gx0 && rect_min_y >= gy0 && rect_max_x <= gx1 && rect_max_y <= gy1 {
            let area = (group_rect.size.width.max(0.0)) * (group_rect.size.height.max(0.0));
            match best {
                Some((_id, best_area)) if best_area <= area => {}
                _ => best = Some((*group_id, area)),
            }
        }
    }

    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests;
