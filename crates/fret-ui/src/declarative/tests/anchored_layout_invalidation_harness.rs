use super::*;

use crate::element::{
    AnchoredProps, ContainerProps, InsetEdge, LayoutStyle, PositionStyle, SizeStyle,
};
use crate::overlay_placement::{Align, AnchoredPanelLayout, AnchoredPanelOptions, Side};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedNode, ObservedTree,
    ScenarioObserveError,
};
use serde::Deserialize;

const ANCHORED_LAYOUT_INVALIDATION_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/anchored_layout_invalidation_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
struct AnchoredLayoutInvalidationScenario {
    window_bounds: Rect,
    #[serde(default)]
    anchor_elements: Option<AnchorElementsSpec>,
    first: AnchoredFrameSpec,
    second: AnchoredFrameSpec,
}

#[derive(Debug, Clone, Deserialize)]
struct AnchoredFrameSpec {
    anchor: Rect,
    #[serde(default)]
    anchor_element: Option<AnchorElementRef>,
    content: Size,
    side: SideFixture,
    align: AlignFixture,
    #[serde(default)]
    side_offset_px: f32,
    #[serde(default)]
    outer_margin: EdgesFixture,
}

#[derive(Debug, Clone, Deserialize)]
struct AnchorElementsSpec {
    a: Rect,
    b: Rect,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnchorElementRef {
    A,
    B,
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

#[test]
fn mechanism_harness_anchored_layout_invalidation_matches_oracles() {
    let suite: MechanismSuite<AnchoredLayoutInvalidationScenario> =
        MechanismSuite::from_json_str(ANCHORED_LAYOUT_INVALIDATION_FIXTURE)
            .expect("anchored layout invalidation fixture suite");

    let mut observer: fn(
        &MechanismCase<AnchoredLayoutInvalidationScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_anchored_layout_invalidation_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_anchored_layout_invalidation_case(
    case: &MechanismCase<AnchoredLayoutInvalidationScenario>,
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

    let root_name = "anchored-layout-invalidation-harness";
    render_anchored_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        case.scenario.window_bounds,
        root_name,
        case.scenario.anchor_elements.as_ref(),
        &case.scenario.first,
        layout_out.clone(),
    );
    ui.layout_all(&mut app, &mut services, case.scenario.window_bounds, 1.0);
    let first = app
        .models()
        .get_copied(&layout_out)
        .ok_or_else(|| ScenarioObserveError::new("missing first anchored layout"))?;

    app.advance_frame();
    render_anchored_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        case.scenario.window_bounds,
        root_name,
        case.scenario.anchor_elements.as_ref(),
        &case.scenario.second,
        layout_out.clone(),
    );
    ui.layout_all(&mut app, &mut services, case.scenario.window_bounds, 1.0);
    let second = app
        .models()
        .get_copied(&layout_out)
        .ok_or_else(|| ScenarioObserveError::new("missing second anchored layout"))?;

    let mut observed = ObservedTree::new(case.scenario.window_bounds);
    observed.push_node(ObservedNode::new("first-panel", first.rect));
    observed.push_node(ObservedNode::new("second-panel", second.rect));
    set_side_metrics(&mut observed, "anchored.first.side", first.side);
    set_side_metrics(&mut observed, "anchored.second.side", second.side);
    set_align_metrics(&mut observed, "anchored.first.align", first.align);
    set_align_metrics(&mut observed, "anchored.second.align", second.align);
    observed.set_metric(
        "anchored.panel.delta_x_px",
        second.rect.origin.x.0 - first.rect.origin.x.0,
    );
    observed.set_metric(
        "anchored.panel.delta_y_px",
        second.rect.origin.y.0 - first.rect.origin.y.0,
    );

    Ok(observed)
}

fn render_anchored_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    anchor_elements: Option<&AnchorElementsSpec>,
    spec: &AnchoredFrameSpec,
    layout_out: fret_runtime::Model<AnchoredPanelLayout>,
) {
    let root = render_root(ui, app, services, window, bounds, root_name, |cx| {
        let layout_out = layout_out.clone();
        let (anchor_a, anchor_b, anchor_element) = match anchor_elements {
            Some(anchors) => {
                let anchor_a = Some(anchor_element_node(cx, anchors.a));
                let anchor_b = Some(anchor_element_node(cx, anchors.b));
                let anchor_element = match spec.anchor_element {
                    Some(AnchorElementRef::A) => anchor_a.as_ref().map(|element| element.id.0),
                    Some(AnchorElementRef::B) => anchor_b.as_ref().map(|element| element.id.0),
                    None => None,
                };
                (anchor_a, anchor_b, anchor_element)
            }
            None => (None, None, None),
        };

        let anchored = cx.anchored_props(
            AnchoredProps {
                anchor: spec.anchor,
                anchor_element,
                side: spec.side.into(),
                align: spec.align.into(),
                side_offset: Px(spec.side_offset_px),
                outer_margin: spec.outer_margin.into(),
                options: AnchoredPanelOptions::default(),
                layout_out: Some(layout_out),
                ..Default::default()
            },
            |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: crate::element::Length::Px(spec.content.width),
                                height: crate::element::Length::Px(spec.content.height),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )]
            },
        );

        let mut out = Vec::new();
        if let Some(anchor) = anchor_a {
            out.push(anchor);
        }
        if let Some(anchor) = anchor_b {
            out.push(anchor);
        }
        out.push(anchored);
        out
    });
    ui.set_root(root);
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
