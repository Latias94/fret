use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedHitTestSample, ObservedTree,
    ScenarioObserveError,
};
use serde::Deserialize;
use slotmap::Key as _;

const LAYOUT_PRIMITIVES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/layout_primitives_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LayoutPrimitiveScenario {
    AutoSizeShrinkWrapsText,
    FillResolvesUnderDefiniteContainer,
    FlexCrossAxisStretch,
    TransparentWrapperPreservesFill,
    ChromeContainerStretchKeepsOuterBox,
    GridFrAutoTrackNegotiation,
    VisualVsHitBoundsFollowRenderTransform,
}

#[test]
fn mechanism_harness_layout_primitives_match_oracles() {
    let suite: MechanismSuite<LayoutPrimitiveScenario> =
        MechanismSuite::from_json_str(LAYOUT_PRIMITIVES).expect("layout primitive fixture suite");

    let mut observer: fn(
        &MechanismCase<LayoutPrimitiveScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<LayoutPrimitiveScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let mut transformed: Option<crate::elements::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-layout-primitives",
        |cx| build_scenario(cx, &case.scenario, &mut transformed),
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    if matches!(
        case.scenario,
        LayoutPrimitiveScenario::VisualVsHitBoundsFollowRenderTransform
    ) {
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);

    for node in &snapshot.nodes {
        if let Some(layout) = ui.debug_node_bounds(node.id) {
            observed.set_layout_bounds_for_node_id(node.id.data().as_ffi(), layout);
        }
    }

    if let Some(element) = transformed {
        let layout = crate::elements::current_bounds_for_element(&mut app, window, element)
            .ok_or_else(|| ScenarioObserveError::new("missing transformed layout bounds"))?;
        let visual = crate::elements::current_visual_bounds_for_element(&mut app, window, element)
            .ok_or_else(|| ScenarioObserveError::new("missing transformed visual bounds"))?;
        observed.set_space_bounds_for_test_id(
            "transformed-pressable",
            fret_mechanism_harness::BoundsSpace::Layout,
            layout,
        );
        observed.set_visual_bounds_for_test_id("transformed-pressable", visual);
        observed.set_hit_bounds_for_test_id("transformed-pressable", visual);

        let layout_center = Point::new(
            Px(layout.origin.x.0 + layout.size.width.0 * 0.5),
            Px(layout.origin.y.0 + layout.size.height.0 * 0.5),
        );
        let visual_center = Point::new(
            Px(visual.origin.x.0 + visual.size.width.0 * 0.5),
            Px(visual.origin.y.0 + visual.size.height.0 * 0.5),
        );
        observed.push_hit_test_sample(hit_sample(&ui, &snapshot, "layout-center", layout_center));
        observed.push_hit_test_sample(hit_sample(&ui, &snapshot, "visual-center", visual_center));
    }

    Ok(observed)
}

fn build_scenario(
    cx: &mut ElementContext<'_, TestHost>,
    scenario: &LayoutPrimitiveScenario,
    transformed: &mut Option<crate::elements::GlobalElementId>,
) -> Vec<AnyElement> {
    match scenario {
        LayoutPrimitiveScenario::AutoSizeShrinkWrapsText => {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    vec![cx.text("x")]
                })
                .test_id("auto-box"),
            ]
        }
        LayoutPrimitiveScenario::FillResolvesUnderDefiniteContainer => {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Px(Px(120.0));
            outer.layout.size.height = Length::Px(Px(40.0));

            let mut fill = crate::element::ContainerProps::default();
            fill.layout.size.width = Length::Fill;
            fill.layout.size.height = Length::Fill;

            vec![cx.container(outer, |cx| {
                vec![cx.container(fill, |_cx| Vec::new()).test_id("fill-box")]
            })]
        }
        LayoutPrimitiveScenario::FlexCrossAxisStretch => {
            let row = crate::element::FlexProps {
                direction: fret_core::Axis::Horizontal,
                align: CrossAlign::Stretch,
                ..Default::default()
            };

            let mut addon = crate::element::ContainerProps::default();
            addon.layout.flex.align_self = Some(CrossAlign::Stretch);

            let mut control = crate::element::ContainerProps::default();
            control.layout.size.width = Length::Px(Px(120.0));
            control.layout.size.height = Length::Px(Px(36.0));

            vec![cx.flex(row, |cx| {
                vec![
                    cx.container(addon, |cx| vec![cx.text("x")])
                        .test_id("stretch-addon"),
                    cx.container(control, |_cx| Vec::new())
                        .test_id("stretch-control"),
                ]
            })]
        }
        LayoutPrimitiveScenario::TransparentWrapperPreservesFill => {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Px(Px(100.0));
            outer.layout.size.height = Length::Px(Px(40.0));

            let mut semantics = crate::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("transparent wrapper")),
                ..Default::default()
            };
            semantics.layout.size.width = Length::Fill;
            semantics.layout.size.height = Length::Fill;

            let mut fill = crate::element::ContainerProps::default();
            fill.layout.size.width = Length::Fill;
            fill.layout.size.height = Length::Fill;

            vec![cx.container(outer, |cx| {
                vec![
                    cx.semantics(semantics, |cx| {
                        vec![
                            cx.container(fill, |_cx| Vec::new())
                                .test_id("transparent-fill"),
                        ]
                    })
                    .test_id("transparent-wrapper"),
                ]
            })]
        }
        LayoutPrimitiveScenario::ChromeContainerStretchKeepsOuterBox => {
            let row = crate::element::FlexProps {
                direction: fret_core::Axis::Horizontal,
                align: CrossAlign::Stretch,
                ..Default::default()
            };
            let mut addon = crate::element::ContainerProps {
                padding: fret_core::Edges {
                    top: Px(0.0),
                    right: Px(8.0),
                    bottom: Px(0.0),
                    left: Px(8.0),
                }
                .into(),
                border: fret_core::Edges::all(Px(1.0)),
                ..Default::default()
            };
            addon.layout.flex.align_self = Some(CrossAlign::Stretch);

            let content_row = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: CrossAlign::Center,
                ..Default::default()
            };

            let mut control = crate::element::ContainerProps::default();
            control.layout.size.width = Length::Px(Px(120.0));
            control.layout.size.height = Length::Px(Px(36.0));

            vec![cx.flex(row, |cx| {
                vec![
                    cx.container(addon, |cx| {
                        vec![cx.flex(content_row, |cx| {
                            vec![
                                cx.container(crate::element::ContainerProps::default(), |cx| {
                                    vec![cx.text("https://")]
                                })
                                .test_id("chrome-inner"),
                            ]
                        })]
                    })
                    .test_id("chrome-addon"),
                    cx.container(control, |_cx| Vec::new())
                        .test_id("chrome-control"),
                ]
            })]
        }
        LayoutPrimitiveScenario::GridFrAutoTrackNegotiation => {
            let grid = crate::element::GridProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Px(Px(200.0)),
                        height: Length::Px(Px(20.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cols: 2,
                template_columns: Some(vec![
                    crate::element::GridTrackSizing::Fr(1.0),
                    crate::element::GridTrackSizing::Auto,
                ]),
                align: CrossAlign::Stretch,
                ..Default::default()
            };

            let mut fr = crate::element::ContainerProps::default();
            fr.layout.size.width = Length::Fill;
            fr.layout.size.height = Length::Fill;
            fr.layout.grid.column.start = Some(1);

            let mut auto = crate::element::ContainerProps::default();
            auto.layout.size.width = Length::Px(Px(40.0));
            auto.layout.size.height = Length::Fill;
            auto.layout.grid.column.start = Some(2);

            vec![cx.grid(grid, |cx| {
                vec![
                    cx.container(fr, |_cx| Vec::new()).test_id("grid-fr-cell"),
                    cx.container(auto, |_cx| Vec::new())
                        .test_id("grid-auto-cell"),
                ]
            })]
        }
        LayoutPrimitiveScenario::VisualVsHitBoundsFollowRenderTransform => {
            let transform = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            vec![cx.render_transform(transform, |cx| {
                let mut props = crate::element::PressableProps::default();
                props.layout.size.width = Length::Px(Px(20.0));
                props.layout.size.height = Length::Px(Px(20.0));
                vec![
                    cx.pressable_with_id(props, |_cx, _state, id| {
                        *transformed = Some(id);
                        Vec::new()
                    })
                    .test_id("transformed-pressable"),
                ]
            })]
        }
    }
}

fn hit_sample(
    ui: &UiTree<TestHost>,
    snapshot: &fret_core::SemanticsSnapshot,
    id: &str,
    point: Point,
) -> ObservedHitTestSample {
    let hit = ui.debug_hit_test(point);
    let hit_node = hit.hit;
    let hit_test_id = hit_node.and_then(|hit| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == hit)
            .and_then(|node| node.test_id.clone())
    });

    ObservedHitTestSample {
        id: id.to_string(),
        point,
        hit_node_id: hit_node.map(|node| node.data().as_ffi()),
        hit_test_id,
        barrier_root_node_id: hit.barrier_root.map(|node| node.data().as_ffi()),
        active_layer_root_node_ids: hit
            .active_layer_roots
            .iter()
            .map(|node| node.data().as_ffi())
            .collect(),
    }
}
