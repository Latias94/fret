use fret_core::Point;

use crate::registry::{DndItemId, Droppable, RegistrySnapshot};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndCollision {
    pub id: DndItemId,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionStrategy {
    PointerWithin,
    ClosestCenter,
}

pub fn pointer_within_over(snapshot: &RegistrySnapshot, pointer: Point) -> Option<DndItemId> {
    let mut best: Option<(DndItemId, i32)> = None;
    for droppable in snapshot.droppables.iter() {
        if droppable.disabled || !droppable.rect.contains(pointer) {
            continue;
        }

        match best {
            None => best = Some((droppable.id, droppable.z_index)),
            Some((best_id, best_z)) => {
                if droppable.z_index > best_z
                    || (droppable.z_index == best_z && droppable.id < best_id)
                {
                    best = Some((droppable.id, droppable.z_index));
                }
            }
        }
    }

    best.map(|(id, _z)| id)
}

pub fn closest_center_over(snapshot: &RegistrySnapshot, pointer: Point) -> Option<DndItemId> {
    let mut best: Option<DndCollision> = None;
    for droppable in snapshot.droppables.iter() {
        if droppable.disabled {
            continue;
        }

        let c = collision_distance_to_center(droppable, pointer);
        match best {
            None => best = Some(c),
            Some(best_c) => {
                if c.score < best_c.score || (c.score == best_c.score && c.id < best_c.id) {
                    best = Some(c);
                }
            }
        }
    }

    best.map(|c| c.id)
}

pub fn pointer_within_collisions(snapshot: &RegistrySnapshot, pointer: Point) -> Vec<DndCollision> {
    let mut out: Vec<(i32, DndCollision)> = snapshot
        .droppables
        .iter()
        .filter(|d| !d.disabled && d.rect.contains(pointer))
        .map(|d| {
            (
                d.z_index,
                DndCollision {
                    id: d.id,
                    score: 0.0,
                },
            )
        })
        .collect();

    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));

    out.into_iter().map(|(_, c)| c).collect()
}

pub fn closest_center_collisions(snapshot: &RegistrySnapshot, pointer: Point) -> Vec<DndCollision> {
    let mut out: Vec<DndCollision> = snapshot
        .droppables
        .iter()
        .filter(|d| !d.disabled)
        .map(|d| collision_distance_to_center(d, pointer))
        .collect();

    out.sort_by(|a, b| a.score.total_cmp(&b.score).then_with(|| a.id.cmp(&b.id)));
    out
}

fn collision_distance_to_center(droppable: &Droppable, pointer: Point) -> DndCollision {
    let cx = droppable.rect.origin.x.0 + droppable.rect.size.width.0 * 0.5;
    let cy = droppable.rect.origin.y.0 + droppable.rect.size.height.0 * 0.5;
    let dx = pointer.x.0 - cx;
    let dy = pointer.y.0 - cy;
    DndCollision {
        id: droppable.id,
        score: dx * dx + dy * dy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Rect, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn pointer_within_respects_z_index() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                Droppable {
                    id: DndItemId(2),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 10,
                },
            ],
        };

        let cols = pointer_within_collisions(&snapshot, Point::new(Px(5.0), Px(5.0)));
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].id, DndItemId(2));
        assert_eq!(cols[1].id, DndItemId(1));
    }

    #[test]
    fn closest_center_sorts_by_distance() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                Droppable {
                    id: DndItemId(2),
                    rect: rect(100.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
            ],
        };

        let cols = closest_center_collisions(&snapshot, Point::new(Px(6.0), Px(5.0)));
        assert_eq!(cols[0].id, DndItemId(1));
        assert_eq!(cols[1].id, DndItemId(2));
    }

    #[test]
    fn pointer_within_over_picks_topmost() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                Droppable {
                    id: DndItemId(2),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 10,
                },
            ],
        };

        assert_eq!(
            pointer_within_over(&snapshot, Point::new(Px(5.0), Px(5.0))),
            Some(DndItemId(2))
        );
    }

    #[test]
    fn closest_center_over_picks_nearest() {
        let snapshot = RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                Droppable {
                    id: DndItemId(2),
                    rect: rect(100.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
            ],
        };

        assert_eq!(
            closest_center_over(&snapshot, Point::new(Px(6.0), Px(5.0))),
            Some(DndItemId(1))
        );
    }
}
