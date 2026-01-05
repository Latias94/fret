use fret_core::{Edges, Point, Px, Transform2D};
use fret_ui::overlay_placement::Side;

pub(crate) const SHADCN_SLIDE_PX: Px = Px(8.0);

pub(crate) fn shadcn_slide_insets(side: Side) -> Edges {
    match side {
        Side::Top => Edges {
            bottom: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Bottom => Edges {
            top: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Left => Edges {
            right: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
        Side::Right => Edges {
            left: SHADCN_SLIDE_PX,
            ..Edges::all(Px(0.0))
        },
    }
}

pub(crate) fn shadcn_enter_slide_offset(side: Side, opacity: f32, opening: bool) -> Point {
    if !opening {
        return Point::new(Px(0.0), Px(0.0));
    }

    // shadcn/ui v4 uses `slide-in-from-*-2` (8px) keyed off `data-side`.
    // We approximate that by moving from 8px -> 0 as opacity approaches 1.
    let t = 1.0 - opacity.clamp(0.0, 1.0);
    match side {
        Side::Top => Point::new(Px(0.0), Px(SHADCN_SLIDE_PX.0 * t)),
        Side::Bottom => Point::new(Px(0.0), Px(-SHADCN_SLIDE_PX.0 * t)),
        Side::Left => Point::new(Px(SHADCN_SLIDE_PX.0 * t), Px(0.0)),
        Side::Right => Point::new(Px(-SHADCN_SLIDE_PX.0 * t), Px(0.0)),
    }
}

pub(crate) fn shadcn_enter_slide_transform(side: Side, opacity: f32, opening: bool) -> Transform2D {
    Transform2D::translation(shadcn_enter_slide_offset(side, opacity, opening))
}

pub(crate) fn shadcn_modal_slide_offset(side: Side, distance: Px, opacity: f32) -> Point {
    // Used by modal panels like `Sheet`, which slide in/out from the same side.
    // This differs from popper overlays (Tooltip/HoverCard/Popover) that slide towards the anchor.
    let t = 1.0 - opacity.clamp(0.0, 1.0);
    match side {
        Side::Top => Point::new(Px(0.0), Px(-distance.0 * t)),
        Side::Bottom => Point::new(Px(0.0), Px(distance.0 * t)),
        Side::Left => Point::new(Px(-distance.0 * t), Px(0.0)),
        Side::Right => Point::new(Px(distance.0 * t), Px(0.0)),
    }
}

pub(crate) fn shadcn_modal_slide_transform(side: Side, distance: Px, opacity: f32) -> Transform2D {
    Transform2D::translation(shadcn_modal_slide_offset(side, distance, opacity))
}

pub(crate) fn shadcn_zoom_transform(origin: Point, opacity: f32) -> Transform2D {
    // shadcn/ui v4 uses a small zoom-in (95% -> 100%) plus opacity transitions.
    // We approximate that with a fade-driven zoom transform around a popper-style transform origin
    // (Radix exposes this via `--radix-*-transform-origin`).
    let scale = 0.95 + 0.05 * opacity.clamp(0.0, 1.0);
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(Point::new(Px(-origin.x.0), Px(-origin.y.0)))
}
