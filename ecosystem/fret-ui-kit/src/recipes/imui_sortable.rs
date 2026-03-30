//! Immediate sortable/reorder helpers built on top of the typed `imui` drag/drop seam.
//!
//! This module intentionally lives in `recipes`, not in `imui` itself:
//! - `fret-ui-kit::imui` owns the typed drag/drop mechanism/helper boundary,
//! - this module owns reusable reorder packaging for immediate rows/lists/outliners,
//! - and app code still owns rendering plus the final domain mutation.

use std::any::Any;
use std::rc::Rc;

use fret_core::{Point, Rect};
use fret_ui::UiHost;

use crate::imui::{
    DragSourceOptions, DragSourceResponse, DropTargetOptions, DropTargetResponse, ResponseExt,
    UiWriterImUiFacadeExt,
};

/// Insertion side for a sortable target row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortableInsertionSide {
    Before,
    After,
}

impl SortableInsertionSide {
    pub fn label(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

/// Recipe-level options for a sortable immediate row.
#[derive(Debug, Clone, Copy, Default)]
pub struct SortableRowOptions {
    pub drag_source: DragSourceOptions,
    pub drop_target: DropTargetOptions,
}

/// Typed preview/delivery signal for one sortable row.
#[derive(Debug, Clone)]
pub struct SortableRowSignal<T: 'static> {
    payload: Rc<T>,
    side: SortableInsertionSide,
}

impl<T: 'static> SortableRowSignal<T> {
    pub fn payload(&self) -> Rc<T> {
        self.payload.clone()
    }

    pub fn side(&self) -> SortableInsertionSide {
        self.side
    }
}

/// Combined source/target readout for an immediate sortable row.
pub struct SortableRowResponse<T: 'static> {
    source: DragSourceResponse,
    target: DropTargetResponse<T>,
    side: Option<SortableInsertionSide>,
}

impl<T: 'static> SortableRowResponse<T> {
    pub fn source(&self) -> DragSourceResponse {
        self.source
    }

    pub fn target(&self) -> &DropTargetResponse<T> {
        &self.target
    }

    pub fn side(&self) -> Option<SortableInsertionSide> {
        self.side
    }

    pub fn preview_reorder(&self) -> Option<SortableRowSignal<T>> {
        Some(SortableRowSignal {
            payload: self.target.preview_payload()?,
            side: self.side?,
        })
    }

    pub fn delivered_reorder(&self) -> Option<SortableRowSignal<T>> {
        Some(SortableRowSignal {
            payload: self.target.delivered_payload()?,
            side: self.side?,
        })
    }
}

/// Compute the vertical insertion side for a row trigger from the current drop geometry.
pub fn vertical_insertion_side<T: 'static>(
    trigger: ResponseExt,
    drop: &DropTargetResponse<T>,
) -> Option<SortableInsertionSide> {
    let rect = trigger.core.rect?;
    let position = drop
        .delivered_position()
        .or_else(|| drop.preview_position())?;
    Some(insertion_side_for_rect_position(rect, position))
}

/// Attach sortable drag/drop behavior to a single immediate row.
///
/// The row still owns rendering and identity. This helper only packages:
/// - publishing the row payload,
/// - resolving compatible target preview/delivery,
/// - and deriving `Before` / `After` from the row rect midpoint.
pub fn sortable_row<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
) -> SortableRowResponse<T> {
    sortable_row_with_options(ui, trigger, payload, SortableRowOptions::default())
}

pub fn sortable_row_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: SortableRowOptions,
) -> SortableRowResponse<T> {
    let source = ui.drag_source_with_options(trigger, payload, options.drag_source);
    let target = ui.drop_target_with_options::<T>(trigger, options.drop_target);
    let side = vertical_insertion_side(trigger, &target);

    SortableRowResponse {
        source,
        target,
        side,
    }
}

/// Reorder a vector by stable keys while keeping the actual mutation app-owned.
pub fn reorder_vec_by_key<T, K: ?Sized + PartialEq>(
    items: &mut Vec<T>,
    active_key: &K,
    over_key: &K,
    side: SortableInsertionSide,
    mut key_of: impl for<'a> FnMut(&'a T) -> &'a K,
) -> bool {
    if active_key == over_key {
        return false;
    }

    let Some(from_index) = items.iter().position(|item| key_of(item) == active_key) else {
        return false;
    };
    let Some(over_index_before_remove) = items.iter().position(|item| key_of(item) == over_key)
    else {
        return false;
    };

    let moving = items.remove(from_index);
    let mut insert_index = items
        .iter()
        .position(|item| key_of(item) == over_key)
        .unwrap_or(over_index_before_remove.min(items.len()));
    if side == SortableInsertionSide::After {
        insert_index = insert_index.saturating_add(1).min(items.len());
    }

    items.insert(insert_index, moving);
    true
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        id: &'static str,
        label: &'static str,
    }

    #[test]
    fn reorder_vec_by_key_moves_item_after_target() {
        let mut items = vec![
            TestItem {
                id: "camera",
                label: "Camera",
            },
            TestItem {
                id: "cube",
                label: "Cube",
            },
            TestItem {
                id: "light",
                label: "Light",
            },
        ];

        assert!(reorder_vec_by_key(
            &mut items,
            "camera",
            "cube",
            SortableInsertionSide::After,
            |item| item.id,
        ));
        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec!["cube", "camera", "light"]
        );
    }

    #[test]
    fn reorder_vec_by_key_moves_item_before_target() {
        let mut items = vec![
            TestItem {
                id: "camera",
                label: "Camera",
            },
            TestItem {
                id: "cube",
                label: "Cube",
            },
            TestItem {
                id: "light",
                label: "Light",
            },
        ];

        assert!(reorder_vec_by_key(
            &mut items,
            "light",
            "cube",
            SortableInsertionSide::Before,
            |item| item.id,
        ));
        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec!["camera", "light", "cube"]
        );
    }

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
