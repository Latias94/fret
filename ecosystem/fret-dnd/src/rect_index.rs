use fret_core::{Point, Rect};

use crate::{DndItemId, Droppable};

#[derive(Debug, Clone)]
pub struct RectDroppableIndex<T: Copy> {
    entries: Vec<(Droppable, T)>,
}

impl<T: Copy> RectDroppableIndex<T> {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn push_rect(&mut self, id: DndItemId, rect: Rect, z_index: i32, disabled: bool, value: T) {
        self.entries.push((
            Droppable {
                id,
                rect,
                disabled,
                z_index,
            },
            value,
        ));
    }

    pub fn pick_pointer_within(&self, pointer: Point) -> Option<(DndItemId, T)> {
        let mut best: Option<(DndItemId, i32, T)> = None;
        for (droppable, value) in &self.entries {
            if droppable.disabled || !droppable.rect.contains(pointer) {
                continue;
            }

            match best {
                None => {
                    best = Some((droppable.id, droppable.z_index, *value));
                }
                Some((best_id, best_z, _best_value)) => {
                    if droppable.z_index > best_z
                        || (droppable.z_index == best_z && droppable.id < best_id)
                    {
                        best = Some((droppable.id, droppable.z_index, *value));
                    }
                }
            }
        }

        best.map(|(id, _z, value)| (id, value))
    }
}

impl<T: Copy> Default for RectDroppableIndex<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
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
    fn pick_pointer_within_prefers_higher_z_index() {
        let mut idx = RectDroppableIndex::<u32>::default();
        idx.push_rect(DndItemId(1), rect(0.0, 0.0, 10.0, 10.0), 0, false, 1);
        idx.push_rect(DndItemId(2), rect(0.0, 0.0, 10.0, 10.0), 10, false, 2);

        let picked = idx.pick_pointer_within(Point::new(Px(5.0), Px(5.0)));
        assert_eq!(picked, Some((DndItemId(2), 2)));
    }

    #[test]
    fn pick_pointer_within_ties_break_by_id() {
        let mut idx = RectDroppableIndex::<u32>::default();
        idx.push_rect(DndItemId(2), rect(0.0, 0.0, 10.0, 10.0), 0, false, 2);
        idx.push_rect(DndItemId(1), rect(0.0, 0.0, 10.0, 10.0), 0, false, 1);

        let picked = idx.pick_pointer_within(Point::new(Px(5.0), Px(5.0)));
        assert_eq!(picked, Some((DndItemId(1), 1)));
    }
}
