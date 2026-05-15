use fret_core::{Edges, Point, Px, Rect, Size};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedNode, ObservedTree,
    ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const ANCHORED_PANEL_PLACEMENT_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/overlay_placement/fixtures/anchored_panel_placement_v1.json"
));

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

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PlacementSolverVariant {
    #[default]
    Unsized,
    Sized,
}

#[derive(Debug, Clone, Deserialize)]
struct AnchoredPanelPlacementScenario {
    #[serde(default)]
    variant: PlacementSolverVariant,
    outer: Rect,
    anchor: Rect,
    content: Size,
    side_offset_px: f32,
    preferred_side: SideFixture,
    align: AlignFixture,
    #[serde(default)]
    options: AnchoredPanelOptionsFixture,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SideFixture {
    Top,
    Bottom,
    Left,
    Right,
}

impl From<SideFixture> for Side {
    fn from(value: SideFixture) -> Self {
        match value {
            SideFixture::Top => Self::Top,
            SideFixture::Bottom => Self::Bottom,
            SideFixture::Left => Self::Left,
            SideFixture::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AlignFixture {
    Start,
    Center,
    End,
}

impl From<AlignFixture> for Align {
    fn from(value: AlignFixture) -> Self {
        match value {
            AlignFixture::Start => Self::Start,
            AlignFixture::Center => Self::Center,
            AlignFixture::End => Self::End,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutDirectionFixture {
    #[default]
    Ltr,
    Rtl,
}

impl From<LayoutDirectionFixture> for LayoutDirection {
    fn from(value: LayoutDirectionFixture) -> Self {
        match value {
            LayoutDirectionFixture::Ltr => Self::Ltr,
            LayoutDirectionFixture::Rtl => Self::Rtl,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StickyModeFixture {
    #[default]
    Always,
    Partial,
}

impl From<StickyModeFixture> for StickyMode {
    fn from(value: StickyModeFixture) -> Self {
        match value {
            StickyModeFixture::Always => Self::Always,
            StickyModeFixture::Partial => Self::Partial,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct AnchoredPanelOptionsFixture {
    #[serde(default)]
    direction: LayoutDirectionFixture,
    #[serde(default)]
    offset: OffsetFixture,
    #[serde(default)]
    shift: ShiftOptionsFixture,
    #[serde(default)]
    arrow: Option<ArrowOptionsFixture>,
    #[serde(default)]
    collision: CollisionOptionsFixture,
    #[serde(default)]
    sticky: StickyModeFixture,
}

impl From<AnchoredPanelOptionsFixture> for AnchoredPanelOptions {
    fn from(value: AnchoredPanelOptionsFixture) -> Self {
        Self {
            direction: value.direction.into(),
            offset: value.offset.into(),
            shift: value.shift.into(),
            arrow: value.arrow.map(Into::into),
            collision: value.collision.into(),
            sticky: value.sticky.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct OffsetFixture {
    #[serde(default)]
    main_axis_px: f32,
    #[serde(default)]
    cross_axis_px: f32,
    #[serde(default)]
    alignment_axis_px: Option<f32>,
}

impl From<OffsetFixture> for Offset {
    fn from(value: OffsetFixture) -> Self {
        Self {
            main_axis: Px(value.main_axis_px),
            cross_axis: Px(value.cross_axis_px),
            alignment_axis: value.alignment_axis_px.map(Px),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct ShiftOptionsFixture {
    #[serde(default = "default_true")]
    main_axis: bool,
    #[serde(default = "default_true")]
    cross_axis: bool,
}

impl Default for ShiftOptionsFixture {
    fn default() -> Self {
        Self {
            main_axis: true,
            cross_axis: true,
        }
    }
}

impl From<ShiftOptionsFixture> for ShiftOptions {
    fn from(value: ShiftOptionsFixture) -> Self {
        Self {
            main_axis: value.main_axis,
            cross_axis: value.cross_axis,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct CollisionOptionsFixture {
    #[serde(default)]
    padding: EdgesFixture,
    #[serde(default)]
    boundary: Option<Rect>,
}

impl From<CollisionOptionsFixture> for CollisionOptions {
    fn from(value: CollisionOptionsFixture) -> Self {
        Self {
            padding: value.padding.into(),
            boundary: value.boundary,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct ArrowOptionsFixture {
    size: Size,
    #[serde(default)]
    padding: EdgesFixture,
}

impl From<ArrowOptionsFixture> for ArrowOptions {
    fn from(value: ArrowOptionsFixture) -> Self {
        Self {
            size: value.size,
            padding: value.padding.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
struct EdgesFixture {
    #[serde(default)]
    top: f32,
    #[serde(default)]
    right: f32,
    #[serde(default)]
    bottom: f32,
    #[serde(default)]
    left: f32,
}

impl From<EdgesFixture> for Edges {
    fn from(value: EdgesFixture) -> Self {
        Self {
            top: Px(value.top),
            right: Px(value.right),
            bottom: Px(value.bottom),
            left: Px(value.left),
        }
    }
}

#[test]
fn mechanism_harness_anchored_panel_placement_matches_oracles() {
    let suite: MechanismSuite<AnchoredPanelPlacementScenario> =
        MechanismSuite::from_json_str(ANCHORED_PANEL_PLACEMENT_FIXTURE)
            .expect("anchored panel placement fixture suite");

    let mut observer: fn(
        &MechanismCase<AnchoredPanelPlacementScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_anchored_panel_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_anchored_panel_case(
    case: &MechanismCase<AnchoredPanelPlacementScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    let scenario = &case.scenario;
    let preferred_side = scenario.preferred_side.into();
    let align = scenario.align.into();
    let options = scenario.options.into();

    let (layout, trace) = match scenario.variant {
        PlacementSolverVariant::Unsized => anchored_panel_layout_with_trace(
            scenario.outer,
            scenario.anchor,
            scenario.content,
            Px(scenario.side_offset_px),
            preferred_side,
            align,
            options,
        ),
        PlacementSolverVariant::Sized => anchored_panel_layout_sized_with_trace(
            scenario.outer,
            scenario.anchor,
            scenario.content,
            Px(scenario.side_offset_px),
            preferred_side,
            align,
            options,
        ),
    };

    let mut observed = ObservedTree::new(trace.outer_input);
    observed.push_node(ObservedNode::new("outer-input", trace.outer_input));
    observed.push_node(ObservedNode::new("outer-collision", trace.outer_collision));
    observed.push_node(ObservedNode::new("anchor", trace.anchor));
    observed.push_node(ObservedNode::new("preferred", trace.preferred_rect));
    observed.push_node(ObservedNode::new("flipped", trace.flipped_rect));
    observed.push_node(ObservedNode::new("chosen", trace.chosen_rect));
    observed.push_node(ObservedNode::new(
        "panel-after-shift",
        trace.rect_after_shift,
    ));
    observed.push_node(ObservedNode::new("panel", layout.rect));

    observed.set_metric(
        "anchored_panel.preferred_fits_without_main_clamp",
        bool_metric(trace.preferred_fits_without_main_clamp),
    );
    observed.set_metric(
        "anchored_panel.flipped_fits_without_main_clamp",
        bool_metric(trace.flipped_fits_without_main_clamp),
    );
    observed.set_metric(
        "anchored_panel.preferred_available_main_px",
        trace.preferred_available_main_px,
    );
    observed.set_metric(
        "anchored_panel.flipped_available_main_px",
        trace.flipped_available_main_px,
    );
    observed.set_metric("anchored_panel.gap_px", trace.gap.0);
    observed.set_metric("anchored_panel.shift_delta_x_px", trace.shift_delta.x.0);
    observed.set_metric("anchored_panel.shift_delta_y_px", trace.shift_delta.y.0);
    set_side_metrics(
        &mut observed,
        "anchored_panel.preferred_side",
        trace.preferred_side,
    );
    set_side_metrics(
        &mut observed,
        "anchored_panel.chosen_side",
        trace.chosen_side,
    );
    set_side_metrics(&mut observed, "anchored_panel.layout_side", layout.side);

    if let Some(arrow) = layout.arrow {
        observed.set_metric("anchored_panel.arrow_exists", 1.0);
        observed.set_metric("anchored_panel.arrow_offset_px", arrow.offset.0);
        observed.set_metric(
            "anchored_panel.arrow_alignment_offset_px",
            arrow.alignment_offset.0,
        );
        observed.set_metric(
            "anchored_panel.arrow_center_offset_px",
            arrow.center_offset.0,
        );
        observed.set_metric(
            "anchored_panel.arrow_center_offset_abs_px",
            arrow.center_offset.0.abs(),
        );
        set_side_metrics(&mut observed, "anchored_panel.arrow_side", arrow.side);
    } else {
        observed.set_metric("anchored_panel.arrow_exists", 0.0);
        clear_side_metrics(&mut observed, "anchored_panel.arrow_side");
    }

    Ok(observed)
}

fn set_side_metrics(observed: &mut ObservedTree, prefix: &str, side: Side) {
    observed.set_metric(
        format!("{prefix}.top"),
        bool_metric(matches!(side, Side::Top)),
    );
    observed.set_metric(
        format!("{prefix}.bottom"),
        bool_metric(matches!(side, Side::Bottom)),
    );
    observed.set_metric(
        format!("{prefix}.left"),
        bool_metric(matches!(side, Side::Left)),
    );
    observed.set_metric(
        format!("{prefix}.right"),
        bool_metric(matches!(side, Side::Right)),
    );
}

fn clear_side_metrics(observed: &mut ObservedTree, prefix: &str) {
    for side in ["top", "bottom", "left", "right"] {
        observed.set_metric(format!("{prefix}.{side}"), 0.0);
    }
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn default_true() -> bool {
    true
}
