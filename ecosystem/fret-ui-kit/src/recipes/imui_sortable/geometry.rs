use fret_core::{Point, Rect};

use crate::imui::{DropTargetResponse, ResponseExt};

use super::SortableInsertionSide;

/// Compute the vertical insertion side for a row trigger from the current drop geometry.
pub fn vertical_insertion_side<T: 'static>(
    trigger: ResponseExt,
    drop: &DropTargetResponse<T>,
) -> Option<SortableInsertionSide> {
    let rect = trigger.rect()?;
    let position = drop
        .delivered_position()
        .or_else(|| drop.preview_position())?;
    Some(insertion_side_for_rect_position(rect, position))
}

fn insertion_side_for_rect_position(rect: Rect, position: Point) -> SortableInsertionSide {
    let split_y = rect.origin.y.0 + rect.size.height.0 * 0.5;
    if position.y.0 < split_y {
        SortableInsertionSide::Before
    } else {
        SortableInsertionSide::After
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::{Px, Size};

    #[test]
    fn insertion_side_uses_upper_half_as_before() {
        let rect = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(120.0), Px(40.0)),
        );
        let upper = Point::new(Px(40.0), Px(29.0));
        let lower = Point::new(Px(40.0), Px(51.0));

        assert_eq!(
            insertion_side_for_rect_position(rect, upper),
            SortableInsertionSide::Before
        );
        assert_eq!(
            insertion_side_for_rect_position(rect, lower),
            SortableInsertionSide::After
        );
    }
}
