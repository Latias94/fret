use fret_core::{Edges, Point, Px, Rect, Size};

use super::*;

fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

#[test]
fn keeps_bottom_when_it_fits() {
    let outer = r(0.0, 0.0, 400.0, 400.0);
    let anchor = r(10.0, 10.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(80.0));

    let placed = anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
    assert!(placed.origin.y.0 >= anchor.origin.y.0 + anchor.size.height.0);
}

#[test]
fn flips_from_bottom_to_top_when_bottom_overflows() {
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let anchor = r(10.0, 190.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(80.0));

    let placed = anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
    assert!(placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0);
    assert!(outer.contains(placed.origin));
}

#[test]
fn inset_rect_shrinks_bounds() {
    let outer = r(0.0, 0.0, 100.0, 50.0);
    let inset = inset_rect(outer, Edges::all(Px(8.0)));
    assert_eq!(inset.origin, Point::new(Px(8.0), Px(8.0)));
    assert_eq!(inset.size, Size::new(Px(84.0), Px(34.0)));
}

#[test]
fn flips_from_right_to_left_when_right_overflows() {
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let anchor = r(190.0, 10.0, 10.0, 20.0);
    let content = Size::new(Px(120.0), Px(40.0));

    let placed = anchored_panel_bounds(outer, anchor, content, Px(6.0), Side::Right, Align::Start);
    assert!(
        placed.origin.x.0 + placed.size.width.0 <= anchor.origin.x.0,
        "expected right placement to flip left when overflowing"
    );
}

#[test]
fn chooses_side_with_less_main_axis_overflow_when_neither_fits() {
    // Both bottom and top overflow, but bottom overflows less on the main axis.
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let anchor = r(10.0, 5.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(180.0));

    let placed = anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
    // With less main-axis overflow on bottom, the clamped rect should end up below (as much as possible).
    assert!(
        placed.origin.y.0 >= anchor.origin.y.0,
        "expected placement to prefer bottom when it overflows less than top"
    );
    assert!(outer.contains(placed.origin));
}

#[test]
fn sized_variant_prefers_side_with_less_main_axis_overflow() {
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let anchor = r(10.0, 150.0, 40.0, 10.0);
    let desired = Size::new(Px(120.0), Px(180.0));

    let placed =
        anchored_panel_bounds_sized(outer, anchor, desired, Px(8.0), Side::Bottom, Align::Start);

    // Available space below = 200 - (150 + 10 + 8) = 32
    // Available space above = 150 - 8 = 142
    // Neither side fits the desired height (180), so the solver should prefer the side with
    // less main-axis overflow (top in this case) and then clamp to the available space.
    assert_eq!(placed.size.height, Px(142.0));
    assert!(
        placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0,
        "expected placement to be above the anchor"
    );
    assert!(outer.contains(placed.origin));
}

#[test]
fn sized_variant_prefers_side_with_more_available_space_for_oversized_content() {
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let anchor = r(10.0, 150.0, 40.0, 10.0);
    // Simulate a "greedy" widget measured with an unconstrained probe.
    let desired = Size::new(Px(120.0), Px(10_000.0));

    let placed =
        anchored_panel_bounds_sized(outer, anchor, desired, Px(8.0), Side::Bottom, Align::Start);

    // Available space below = 200 - (150 + 10 + 8) = 32
    // Available space above = 150 - 8 = 142
    // We should choose the side with more available space (top) and clamp to it.
    assert_eq!(placed.size.height, Px(142.0));
    assert!(
        placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0,
        "expected placement to be above the anchor"
    );
    assert!(outer.contains(placed.origin));
}

#[test]
fn offset_applies_cross_axis_skidding() {
    let outer = r(0.0, 0.0, 400.0, 400.0);
    let anchor = r(100.0, 100.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(80.0));

    let base = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions::default(),
    );

    let skidded = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            offset: Offset {
                cross_axis: Px(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    assert_eq!(skidded.rect.origin.x, Px(base.rect.origin.x.0 + 12.0));
    assert_eq!(skidded.rect.origin.y, base.rect.origin.y);
}

#[test]
fn alignment_axis_inverts_under_rtl_for_vertical_alignments() {
    let outer = r(0.0, 0.0, 400.0, 400.0);
    let anchor = r(100.0, 100.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(80.0));

    let ltr_base = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            direction: LayoutDirection::Ltr,
            offset: Offset {
                alignment_axis: None,
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let ltr = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            direction: LayoutDirection::Ltr,
            offset: Offset {
                alignment_axis: Some(Px(10.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let rtl_base = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            direction: LayoutDirection::Rtl,
            offset: Offset {
                alignment_axis: None,
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let rtl = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            direction: LayoutDirection::Rtl,
            offset: Offset {
                alignment_axis: Some(Px(10.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    // `alignment_axis` is applied as a signed delta relative to the base aligned position.
    // For vertical placements, Radix/Floating flip the sign under RTL.
    assert_eq!(Px(ltr.rect.origin.x.0 - ltr_base.rect.origin.x.0), Px(10.0));
    assert_eq!(
        Px(rtl.rect.origin.x.0 - rtl_base.rect.origin.x.0),
        Px(-10.0)
    );
}

#[test]
fn arrow_centers_when_possible() {
    let outer = r(0.0, 0.0, 800.0, 800.0);
    let anchor = r(300.0, 200.0, 100.0, 20.0);
    let content = Size::new(Px(200.0), Px(120.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(8.0),
        Side::Bottom,
        Align::Center,
        AnchoredPanelOptions {
            arrow: Some(ArrowOptions {
                size: Size::new(Px(12.0), Px(12.0)),
                padding: Edges::all(Px(8.0)),
            }),
            ..Default::default()
        },
    );

    let arrow = layout.arrow.expect("arrow layout");
    assert_eq!(arrow.side, Side::Top);
    assert!((arrow.offset.0 - 94.0).abs() < 0.1);
    assert_eq!(arrow.alignment_offset, Px(0.0));
    assert!(arrow.center_offset.0.abs() < 0.1);
}

#[test]
fn arrow_clamps_to_padding_near_edge() {
    let outer = r(0.0, 0.0, 220.0, 200.0);
    let anchor = r(0.0, 50.0, 10.0, 10.0);
    let content = Size::new(Px(200.0), Px(80.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(4.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            arrow: Some(ArrowOptions {
                size: Size::new(Px(12.0), Px(12.0)),
                padding: Edges::all(Px(16.0)),
            }),
            ..Default::default()
        },
    );

    let arrow = layout.arrow.expect("arrow layout");
    assert!(arrow.offset.0 >= 16.0 - 0.01);
    assert!(arrow.center_offset.0.abs() > 0.1);
}

#[test]
fn collision_padding_insets_outer_before_flip_decision() {
    let outer = r(0.0, 0.0, 200.0, 100.0);
    let anchor = r(10.0, 40.0, 40.0, 10.0);
    let content = Size::new(Px(120.0), Px(40.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            collision: CollisionOptions {
                padding: Edges {
                    bottom: Px(20.0),
                    ..Edges::all(Px(0.0))
                },
                boundary: None,
            },
            ..Default::default()
        },
    );

    // Without collision padding, bottom fits: y=50, y+40=90 <= 100.
    // With bottom padding=20, effective outer bottom=80, so bottom does not fit; we should flip.
    assert_eq!(layout.side, Side::Top);
}

#[test]
fn collision_boundary_intersects_outer_before_solving() {
    let outer = r(0.0, 0.0, 200.0, 200.0);
    let boundary = r(0.0, 0.0, 100.0, 100.0);
    let anchor = r(80.0, 80.0, 10.0, 10.0);
    let content = Size::new(Px(60.0), Px(40.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            collision: CollisionOptions {
                padding: Edges::all(Px(0.0)),
                boundary: Some(boundary),
            },
            ..Default::default()
        },
    );

    // The effective outer bottom is 100. Bottom would place at y=90 and overflow, so it flips.
    assert_eq!(layout.side, Side::Top);
    assert!(boundary.contains(layout.rect.origin));
}

#[test]
fn sticky_always_clamps_into_outer() {
    let outer = r(0.0, 0.0, 100.0, 100.0);
    let anchor = r(150.0, 10.0, 10.0, 10.0);
    let content = Size::new(Px(10.0), Px(10.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            sticky: StickyMode::Always,
            ..Default::default()
        },
    );

    // Clamped into `outer`: max_x = 100 - 10 = 90.
    assert_eq!(layout.rect.origin.x, Px(90.0));
}

#[test]
fn sticky_partial_limits_shift_to_keep_anchor_touching() {
    let outer = r(0.0, 0.0, 100.0, 100.0);
    let anchor = r(150.0, 10.0, 10.0, 10.0);
    let content = Size::new(Px(10.0), Px(10.0));

    let layout = anchored_panel_layout(
        outer,
        anchor,
        content,
        Px(0.0),
        Side::Bottom,
        Align::Start,
        AnchoredPanelOptions {
            sticky: StickyMode::Partial,
            ..Default::default()
        },
    );

    // `limitShift()` keeps the panel from detaching from the anchor on the alignment axis, even if
    // that overflows `outer`: min_x = anchor_x - panel_w = 150 - 10 = 140.
    assert_eq!(layout.rect.origin.x, Px(140.0));
}
