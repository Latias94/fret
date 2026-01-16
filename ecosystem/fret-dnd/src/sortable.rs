use fret_core::{Point, Rect};

use crate::{
    Axis, CollisionStrategy, DndItemId, RegistrySnapshot, closest_center_collisions,
    pointer_within_collisions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionSide {
    Before,
    After,
}

pub fn insertion_side_for_pointer(pointer: Point, rect: Rect, axis: Axis) -> InsertionSide {
    let center_x = rect.origin.x.0 + rect.size.width.0 * 0.5;
    let center_y = rect.origin.y.0 + rect.size.height.0 * 0.5;

    match axis {
        Axis::X => {
            if pointer.x.0 < center_x {
                InsertionSide::Before
            } else {
                InsertionSide::After
            }
        }
        Axis::Y => {
            if pointer.y.0 < center_y {
                InsertionSide::Before
            } else {
                InsertionSide::After
            }
        }
    }
}

pub fn sortable_insertion(
    snapshot: &RegistrySnapshot,
    pointer: Point,
    axis: Axis,
    collision_strategy: CollisionStrategy,
) -> Option<(DndItemId, InsertionSide)> {
    let collisions = match collision_strategy {
        CollisionStrategy::PointerWithin => pointer_within_collisions(snapshot, pointer),
        CollisionStrategy::ClosestCenter => closest_center_collisions(snapshot, pointer),
    };
    let over = collisions.first()?.id;

    let rect = snapshot
        .droppables
        .iter()
        .find(|d| d.id == over && !d.disabled)
        .map(|d| d.rect)?;

    Some((over, insertion_side_for_pointer(pointer, rect, axis)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Rect, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn insertion_side_respects_axis() {
        let r = rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            insertion_side_for_pointer(Point::new(Px(2.0), Px(5.0)), r, Axis::X),
            InsertionSide::Before
        );
        assert_eq!(
            insertion_side_for_pointer(Point::new(Px(8.0), Px(5.0)), r, Axis::X),
            InsertionSide::After
        );
        assert_eq!(
            insertion_side_for_pointer(Point::new(Px(5.0), Px(2.0)), r, Axis::Y),
            InsertionSide::Before
        );
        assert_eq!(
            insertion_side_for_pointer(Point::new(Px(5.0), Px(8.0)), r, Axis::Y),
            InsertionSide::After
        );
    }

    #[test]
    fn sortable_insertion_uses_collision_selection_then_half_split() {
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
                    rect: rect(0.0, 20.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
            ],
        };

        let (over, side) = sortable_insertion(
            &snapshot,
            Point::new(Px(5.0), Px(22.0)),
            Axis::Y,
            CollisionStrategy::ClosestCenter,
        )
        .expect("should pick an over item");
        assert_eq!(over, DndItemId(2));
        assert_eq!(side, InsertionSide::Before);

        let (over, side) = sortable_insertion(
            &snapshot,
            Point::new(Px(5.0), Px(28.0)),
            Axis::Y,
            CollisionStrategy::ClosestCenter,
        )
        .expect("should pick an over item");
        assert_eq!(over, DndItemId(2));
        assert_eq!(side, InsertionSide::After);
    }
}
