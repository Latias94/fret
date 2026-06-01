//! Shared Material field-family overlay geometry.
//!
//! Select, Autocomplete, ExposedDropdown, and docked search surfaces all place popup content from
//! field chrome. This Module keeps width, collision, placement, and transform-origin policy local
//! so recipes do not drift while composing their own listbox bodies.

use fret_core::{Edges, LayoutDirection, Point, Px, Rect, Size, Transform2D};
use fret_ui::overlay_placement::{Align, AnchoredPanelLayout, Side};
use fret_ui_kit::primitives::popper::{
    self, PopperContentPlacement, popper_content_transform_origin,
};

pub(crate) const MATERIAL_FIELD_OVERLAY_SIDE_OFFSET: Px = Px(4.0);
pub(crate) const MATERIAL_FIELD_OVERLAY_VERTICAL_PADDING: Px = Px(8.0);
pub(crate) const MATERIAL_FIELD_OVERLAY_WIDTH_FLOOR: Px = Px(210.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialFieldOverlayAlign {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MaterialFieldOverlayWidth {
    MatchAnchor,
    Content {
        estimated_content_width: Px,
        floor: Px,
    },
}

pub(crate) fn material_field_overlay_collision_padding() -> Edges {
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(48.0),
        bottom: Px(48.0),
    }
}

pub(crate) fn material_field_overlay_placement(
    direction: LayoutDirection,
    align: MaterialFieldOverlayAlign,
) -> PopperContentPlacement {
    PopperContentPlacement::new(
        direction,
        Side::Bottom,
        material_field_overlay_popper_align(align),
        MATERIAL_FIELD_OVERLAY_SIDE_OFFSET,
    )
    .with_collision_padding(material_field_overlay_collision_padding())
}

pub(crate) fn material_field_overlay_listbox_size(
    anchor_width: Px,
    row_height: Px,
    visible_row_count: usize,
    width: MaterialFieldOverlayWidth,
) -> Size {
    let row_count = visible_row_count.max(1) as f32;
    Size::new(
        material_field_overlay_width(anchor_width, width),
        Px(row_height.0 * row_count + MATERIAL_FIELD_OVERLAY_VERTICAL_PADDING.0 * 2.0),
    )
}

pub(crate) fn material_field_overlay_width(
    anchor_width: Px,
    width: MaterialFieldOverlayWidth,
) -> Px {
    match width {
        MaterialFieldOverlayWidth::MatchAnchor => anchor_width,
        MaterialFieldOverlayWidth::Content {
            estimated_content_width,
            floor,
        } => Px(anchor_width.0.max(estimated_content_width.0).max(floor.0)),
    }
}

pub(crate) fn material_field_overlay_scale_transform(
    layout: &AnchoredPanelLayout,
    anchor: Rect,
    scale: f32,
) -> Transform2D {
    let origin = popper_content_transform_origin(layout, anchor, None);
    let origin_inv = Point::new(Px(-origin.x.0), Px(-origin.y.0));
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale.max(0.0))
        * Transform2D::translation(origin_inv)
}

fn material_field_overlay_popper_align(align: MaterialFieldOverlayAlign) -> Align {
    match align {
        MaterialFieldOverlayAlign::Start => Align::Start,
        MaterialFieldOverlayAlign::End => Align::End,
    }
}

pub(crate) fn material_field_overlay_layout(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    placement: PopperContentPlacement,
) -> AnchoredPanelLayout {
    popper::popper_content_layout_sized(outer, anchor, desired, placement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_overlay_width_strategy_matches_anchor_or_content_floor() {
        assert_eq!(
            material_field_overlay_width(Px(120.0), MaterialFieldOverlayWidth::MatchAnchor),
            Px(120.0)
        );

        assert_eq!(
            material_field_overlay_width(
                Px(120.0),
                MaterialFieldOverlayWidth::Content {
                    estimated_content_width: Px(160.0),
                    floor: MATERIAL_FIELD_OVERLAY_WIDTH_FLOOR,
                },
            ),
            Px(210.0)
        );

        assert_eq!(
            material_field_overlay_width(
                Px(320.0),
                MaterialFieldOverlayWidth::Content {
                    estimated_content_width: Px(480.0),
                    floor: MATERIAL_FIELD_OVERLAY_WIDTH_FLOOR,
                },
            ),
            Px(480.0)
        );
    }

    #[test]
    fn field_overlay_listbox_size_keeps_at_least_one_row() {
        assert_eq!(
            material_field_overlay_listbox_size(
                Px(200.0),
                Px(48.0),
                0,
                MaterialFieldOverlayWidth::MatchAnchor,
            ),
            Size::new(Px(200.0), Px(64.0))
        );
    }

    #[test]
    fn field_overlay_placement_uses_material_gap_and_collision_padding() {
        let placement =
            material_field_overlay_placement(LayoutDirection::Rtl, MaterialFieldOverlayAlign::End);
        assert_eq!(placement.direction, LayoutDirection::Rtl);
        assert_eq!(placement.side, Side::Bottom);
        assert_eq!(placement.align, Align::End);
        assert_eq!(placement.side_offset, MATERIAL_FIELD_OVERLAY_SIDE_OFFSET);
        assert_eq!(
            placement.collision_padding,
            material_field_overlay_collision_padding()
        );
    }
}
