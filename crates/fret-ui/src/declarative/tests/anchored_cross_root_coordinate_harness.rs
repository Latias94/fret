use super::*;

use std::cell::Cell;

use crate::element::{
    AnchoredProps, ContainerProps, InsetEdge, LayoutStyle, PositionStyle, SizeStyle,
};
use crate::elements::GlobalElementId;
use crate::overlay_placement::{Align, AnchoredPanelLayout, AnchoredPanelOptions, Side};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedNode, ObservedTree,
    ScenarioObserveError,
};
use serde::Deserialize;

const ANCHORED_CROSS_ROOT_COORDINATE_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/anchored_cross_root_coordinate_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
struct AnchoredCrossRootCoordinateScenario {
    window_bounds: Rect,
    anchor_root_bounds: Rect,
    overlay_root_bounds: Rect,
    anchor: Rect,
    fallback_anchor: Rect,
    content: Size,
    side: SideFixture,
    align: AlignFixture,
    #[serde(default)]
    side_offset_px: f32,
    #[serde(default)]
    outer_margin: EdgesFixture,
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

impl From<EdgesFixture> for fret_core::Edges {
    fn from(value: EdgesFixture) -> Self {
        Self {
            top: Px(value.top),
            right: Px(value.right),
            bottom: Px(value.bottom),
            left: Px(value.left),
        }
    }
}

struct CrossRootLayout {
    anchor_root_bounds: Rect,
    overlay_root_bounds: Rect,
}

impl<H: UiHost> Widget<H> for CrossRootLayout {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if let Some(&anchor_root) = cx.children.first() {
            let _ = cx.layout_viewport_root(anchor_root, self.anchor_root_bounds);
        }
        if let Some(&overlay_root) = cx.children.get(1) {
            let _ = cx.layout_viewport_root(overlay_root, self.overlay_root_bounds);
        }
        cx.available
    }
}

#[test]
fn mechanism_harness_anchored_cross_root_coordinate_matches_oracles() {
    let suite: MechanismSuite<AnchoredCrossRootCoordinateScenario> =
        MechanismSuite::from_json_str(ANCHORED_CROSS_ROOT_COORDINATE_FIXTURE)
            .expect("anchored cross-root coordinate fixture suite");

    let mut observer: fn(
        &MechanismCase<AnchoredCrossRootCoordinateScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_anchored_cross_root_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_anchored_cross_root_case(
    case: &MechanismCase<AnchoredCrossRootCoordinateScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let layout_out = app.models_mut().insert(AnchoredPanelLayout {
        rect: Rect::default(),
        side: Side::Bottom,
        align: Align::Start,
        arrow: None,
    });

    let anchor_id_cell: Cell<Option<GlobalElementId>> = Cell::new(None);
    let anchor_root = render_anchor_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        case.scenario.window_bounds,
        &case.scenario,
        &anchor_id_cell,
    );

    let anchor_element = anchor_id_cell
        .get()
        .ok_or_else(|| ScenarioObserveError::new("missing anchor element id"))?;

    let overlay_root = render_overlay_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        case.scenario.window_bounds,
        &case.scenario,
        anchor_element,
        layout_out.clone(),
    );

    let parent = ui.create_node(CrossRootLayout {
        anchor_root_bounds: case.scenario.anchor_root_bounds,
        overlay_root_bounds: case.scenario.overlay_root_bounds,
    });
    ui.set_children(parent, vec![anchor_root, overlay_root]);
    ui.set_root(parent);
    ui.layout_all(&mut app, &mut services, case.scenario.window_bounds, 1.0);

    let layout = app
        .models()
        .get_copied(&layout_out)
        .ok_or_else(|| ScenarioObserveError::new("missing anchored layout output"))?;

    let anchor_node = ui
        .resolve_live_attached_node_for_element(&mut app, Some(window), anchor_element)
        .ok_or_else(|| ScenarioObserveError::new("missing live anchor node"))?;
    let anchor_bounds = ui
        .debug_node_visual_bounds(anchor_node)
        .or_else(|| ui.debug_node_bounds(anchor_node))
        .ok_or_else(|| ScenarioObserveError::new("missing anchor node bounds"))?;

    let side = case.scenario.side.into();
    let align = case.scenario.align.into();
    let outer = crate::overlay_placement::inset_rect(
        case.scenario.overlay_root_bounds,
        case.scenario.outer_margin.into(),
    );
    let fallback_layout = crate::overlay_placement::anchored_panel_layout_sized(
        outer,
        case.scenario.fallback_anchor,
        case.scenario.content,
        Px(case.scenario.side_offset_px),
        side,
        align,
        AnchoredPanelOptions::default(),
    );

    let mut observed = ObservedTree::new(case.scenario.window_bounds);
    observed.push_node(ObservedNode::new(
        "anchor-root",
        case.scenario.anchor_root_bounds,
    ));
    observed.push_node(ObservedNode::new(
        "overlay-root",
        case.scenario.overlay_root_bounds,
    ));
    observed.push_node(ObservedNode::new("anchor", anchor_bounds));
    observed.push_node(ObservedNode::new("panel", layout.rect));
    observed.push_node(ObservedNode::new("fallback-panel", fallback_layout.rect));
    set_side_metrics(&mut observed, "anchored.side", layout.side);
    set_align_metrics(&mut observed, "anchored.align", layout.align);
    observed.set_metric(
        "anchored.used_anchor_element",
        bool_metric(!rects_approx_eq(layout.rect, fallback_layout.rect)),
    );
    observed.set_metric(
        "anchored.panel_matches_fallback",
        bool_metric(rects_approx_eq(layout.rect, fallback_layout.rect)),
    );
    observed.set_metric(
        "anchored.shift_delta_x_px",
        layout.rect.origin.x.0
            - case.scenario.anchor_root_bounds.origin.x.0
            - case.scenario.anchor.origin.x.0,
    );
    observed.set_metric(
        "anchored.shift_delta_y_px",
        layout.rect.origin.y.0
            - case.scenario.anchor_root_bounds.origin.y.0
            - case.scenario.anchor.origin.y.0,
    );

    Ok(observed)
}

fn render_anchor_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    scenario: &AnchoredCrossRootCoordinateScenario,
    anchor_id_cell: &Cell<Option<GlobalElementId>>,
) -> NodeId {
    render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "anchored-cross-root-coordinate-anchor-root",
        |cx| {
            let anchor = anchor_element_node(cx, scenario.anchor);
            anchor_id_cell.set(Some(anchor.id));
            vec![anchor]
        },
    )
}

fn render_overlay_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    scenario: &AnchoredCrossRootCoordinateScenario,
    anchor_element: GlobalElementId,
    layout_out: fret_runtime::Model<AnchoredPanelLayout>,
) -> NodeId {
    render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "anchored-cross-root-coordinate-overlay-root",
        |cx| {
            let layout_out = layout_out.clone();
            vec![cx.anchored_props(
                AnchoredProps {
                    anchor: scenario.fallback_anchor,
                    anchor_element: Some(anchor_element.0),
                    side: scenario.side.into(),
                    align: scenario.align.into(),
                    side_offset: Px(scenario.side_offset_px),
                    outer_margin: scenario.outer_margin.into(),
                    options: AnchoredPanelOptions::default(),
                    layout_out: Some(layout_out),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: crate::element::Length::Px(scenario.content.width),
                                    height: crate::element::Length::Px(scenario.content.height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                },
            )]
        },
    )
}

fn anchor_element_node(cx: &mut ElementContext<'_, TestHost>, bounds: Rect) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.left = InsetEdge::Px(bounds.origin.x);
    props.layout.inset.top = InsetEdge::Px(bounds.origin.y);
    props.layout.size.width = crate::element::Length::Px(bounds.size.width);
    props.layout.size.height = crate::element::Length::Px(bounds.size.height);
    cx.container(props, |_cx| Vec::new())
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

fn set_align_metrics(observed: &mut ObservedTree, prefix: &str, align: Align) {
    observed.set_metric(
        format!("{prefix}.start"),
        bool_metric(matches!(align, Align::Start)),
    );
    observed.set_metric(
        format!("{prefix}.center"),
        bool_metric(matches!(align, Align::Center)),
    );
    observed.set_metric(
        format!("{prefix}.end"),
        bool_metric(matches!(align, Align::End)),
    );
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn rects_approx_eq(a: Rect, b: Rect) -> bool {
    const EPS: f32 = 0.01;
    (a.origin.x.0 - b.origin.x.0).abs() <= EPS
        && (a.origin.y.0 - b.origin.y.0).abs() <= EPS
        && (a.size.width.0 - b.size.width.0).abs() <= EPS
        && (a.size.height.0 - b.size.height.0).abs() <= EPS
}
