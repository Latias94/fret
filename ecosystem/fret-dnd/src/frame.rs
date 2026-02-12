use fret_core::{Point, Rect};

use crate::{
    AutoScrollConfig, AutoScrollRequest, CollisionStrategy, DndCollision, DndItemId,
    RegistrySnapshot, closest_center_collisions, closest_center_over, compute_autoscroll,
    pointer_within_collisions, pointer_within_over,
};

#[derive(Debug, Clone)]
pub struct DndFrameOutput {
    pub collisions: Vec<DndCollision>,
    pub over: Option<DndItemId>,
    pub autoscroll: Option<AutoScrollRequest>,
}

pub fn compute_dnd_over(
    snapshot: &RegistrySnapshot,
    pointer: Point,
    collision_strategy: CollisionStrategy,
) -> Option<DndItemId> {
    match collision_strategy {
        CollisionStrategy::PointerWithin => pointer_within_over(snapshot, pointer),
        CollisionStrategy::ClosestCenter => closest_center_over(snapshot, pointer),
    }
}

pub fn compute_dnd_frame(
    snapshot: &RegistrySnapshot,
    pointer: Point,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndFrameOutput {
    let collisions = match collision_strategy {
        CollisionStrategy::PointerWithin => pointer_within_collisions(snapshot, pointer),
        CollisionStrategy::ClosestCenter => closest_center_collisions(snapshot, pointer),
    };
    let over = collisions.first().map(|c| c.id);

    let autoscroll =
        autoscroll.and_then(|(container, cfg)| compute_autoscroll(cfg, container, pointer));

    DndFrameOutput {
        collisions,
        over,
        autoscroll,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn compute_dnd_frame_picks_over_from_first_collision() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                crate::Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                crate::Droppable {
                    id: DndItemId(2),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 10,
                },
            ],
        };

        let out = compute_dnd_frame(
            &snapshot,
            Point::new(Px(5.0), Px(5.0)),
            CollisionStrategy::PointerWithin,
            None,
        );

        assert_eq!(out.over, Some(DndItemId(2)));
        assert_eq!(out.collisions.first().map(|c| c.id), Some(DndItemId(2)));
    }

    #[test]
    fn compute_dnd_over_matches_pointer_within_topmost() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                crate::Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                crate::Droppable {
                    id: DndItemId(2),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 10,
                },
            ],
        };

        let over = compute_dnd_over(
            &snapshot,
            Point::new(Px(5.0), Px(5.0)),
            CollisionStrategy::PointerWithin,
        );
        assert_eq!(over, Some(DndItemId(2)));
    }
}
