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
    PercentSizeResolvesUnderDefiniteContainer,
    MinMaxClampsFillChild,
    PercentMinMaxBehavesLikeAutoUnderIndefiniteMeasure,
    TextWrapHeightContributesToRow,
    TextMeasurePaintWrapWidthColumn,
    TextMeasurePaintWrapWidthMaxWidthRow,
    TextMeasurePaintOverflowScale,
    ScrollRootPreservesChildLayoutBounds,
    AbsoluteInsetFractionResolvesAgainstContainingBlock,
    FlexCrossAxisStretch,
    FlexPercentBasisKeepsAutoHeightFixedSibling,
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
    let mut services = LayoutPrimitiveServices::default();
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

    let pre_layout_scalar_metrics =
        observe_pre_layout_scalar_metrics(&mut ui, &mut app, &mut services, root, &case.scenario)?;
    ui.request_semantics_snapshot();
    let scale_factor = scale_factor_for_scenario(&case.scenario);
    ui.layout_all(&mut app, &mut services, bounds, scale_factor);

    let mut scene = Scene::default();
    if scenario_needs_paint(&case.scenario) {
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);
    }

    let post_layout_scalar_metrics =
        observe_post_layout_scalar_metrics(&ui, &services, root, &case.scenario)?;
    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    for (id, value) in pre_layout_scalar_metrics
        .into_iter()
        .chain(post_layout_scalar_metrics)
    {
        observed.set_metric(id, value);
    }

    for node in snapshot.nodes.iter() {
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

fn scale_factor_for_scenario(scenario: &LayoutPrimitiveScenario) -> f32 {
    match scenario {
        LayoutPrimitiveScenario::TextMeasurePaintOverflowScale => 1.25,
        _ => 1.0,
    }
}

fn scenario_needs_paint(scenario: &LayoutPrimitiveScenario) -> bool {
    matches!(
        scenario,
        LayoutPrimitiveScenario::VisualVsHitBoundsFollowRenderTransform
            | LayoutPrimitiveScenario::TextMeasurePaintWrapWidthColumn
            | LayoutPrimitiveScenario::TextMeasurePaintWrapWidthMaxWidthRow
            | LayoutPrimitiveScenario::TextMeasurePaintOverflowScale
    )
}

fn observe_pre_layout_scalar_metrics(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut LayoutPrimitiveServices,
    root: NodeId,
    scenario: &LayoutPrimitiveScenario,
) -> Result<Vec<(&'static str, f32)>, ScenarioObserveError> {
    if !matches!(
        scenario,
        LayoutPrimitiveScenario::PercentMinMaxBehavesLikeAutoUnderIndefiniteMeasure
    ) {
        return Ok(Vec::new());
    }

    let children = ui.children(root);
    let max_node = *children
        .first()
        .ok_or_else(|| ScenarioObserveError::new("missing max constraint node"))?;
    let min_node = *children
        .get(1)
        .ok_or_else(|| ScenarioObserveError::new("missing min constraint node"))?;

    let min_constraints = crate::layout_constraints::LayoutConstraints::new(
        crate::layout_constraints::LayoutSize::new(None, None),
        crate::layout_constraints::LayoutSize::new(
            crate::layout_constraints::AvailableSpace::MinContent,
            crate::layout_constraints::AvailableSpace::MinContent,
        ),
    );
    let max_constraints = crate::layout_constraints::LayoutConstraints::new(
        crate::layout_constraints::LayoutSize::new(None, None),
        crate::layout_constraints::LayoutSize::new(
            crate::layout_constraints::AvailableSpace::MaxContent,
            crate::layout_constraints::AvailableSpace::MaxContent,
        ),
    );
    let definite_constraints = crate::layout_constraints::LayoutConstraints::new(
        crate::layout_constraints::LayoutSize::new(None, None),
        crate::layout_constraints::LayoutSize::new(
            crate::layout_constraints::AvailableSpace::Definite(Px(200.0)),
            crate::layout_constraints::AvailableSpace::Definite(Px(80.0)),
        ),
    );

    let max_min_content = ui.measure_in(app, services, max_node, min_constraints, 1.0);
    let min_min_content = ui.measure_in(app, services, min_node, min_constraints, 1.0);
    let max_max_content = ui.measure_in(app, services, max_node, max_constraints, 1.0);
    let min_max_content = ui.measure_in(app, services, min_node, max_constraints, 1.0);
    let max_definite = ui.measure_in(app, services, max_node, definite_constraints, 1.0);
    let min_definite = ui.measure_in(app, services, min_node, definite_constraints, 1.0);

    Ok(vec![
        (
            "percent_min_max.max.min_content_width",
            max_min_content.width.0,
        ),
        (
            "percent_min_max.min.min_content_width",
            min_min_content.width.0,
        ),
        (
            "percent_min_max.max.max_content_width",
            max_max_content.width.0,
        ),
        (
            "percent_min_max.min.max_content_width",
            min_max_content.width.0,
        ),
        ("percent_min_max.max.definite_width", max_definite.width.0),
        ("percent_min_max.min.definite_width", min_definite.width.0),
    ])
}

fn observe_post_layout_scalar_metrics(
    ui: &UiTree<TestHost>,
    services: &LayoutPrimitiveServices,
    root: NodeId,
    scenario: &LayoutPrimitiveScenario,
) -> Result<Vec<(&'static str, f32)>, ScenarioObserveError> {
    match scenario {
        LayoutPrimitiveScenario::TextMeasurePaintWrapWidthColumn => {
            let col = child_at(ui, root, 0, "column root")?;
            let text = child_at(ui, col, 0, "column text")?;
            let sibling = child_at(ui, col, 1, "column sibling")?;
            let text_bounds = ui
                .debug_node_bounds(text)
                .ok_or_else(|| ScenarioObserveError::new("missing column text bounds"))?;
            let sibling_bounds = ui
                .debug_node_bounds(sibling)
                .ok_or_else(|| ScenarioObserveError::new("missing column sibling bounds"))?;
            let (measured, prepared, prepared_metrics) =
                text_constraint_pair_for_width(services, Px(40.0), fret_core::TextWrap::Word)?;

            Ok(vec![
                (
                    "text_measure_paint.column.wrap_width_delta",
                    max_width_delta(measured, prepared),
                ),
                (
                    "text_measure_paint.column.prepared_height",
                    prepared_metrics.size.height.0,
                ),
                (
                    "text_measure_paint.column.sibling_after_painted_height",
                    sibling_bounds.origin.y.0
                        - (text_bounds.origin.y.0 + prepared_metrics.size.height.0),
                ),
            ])
        }
        LayoutPrimitiveScenario::TextMeasurePaintWrapWidthMaxWidthRow => {
            let container = child_at(ui, root, 0, "max-width container")?;
            let col = child_at(ui, container, 0, "max-width column")?;
            let row = child_at(ui, col, 0, "max-width row")?;
            let sibling = child_at(ui, col, 1, "max-width sibling")?;
            let wrapped = child_at(ui, row, 1, "max-width wrapped text")?;
            let wrapped_bounds = ui
                .debug_node_bounds(wrapped)
                .ok_or_else(|| ScenarioObserveError::new("missing max-width wrapped bounds"))?;
            let sibling_bounds = ui
                .debug_node_bounds(sibling)
                .ok_or_else(|| ScenarioObserveError::new("missing max-width sibling bounds"))?;
            let (measured, prepared, prepared_metrics) =
                widest_text_constraint_pair_for_wrap(services, fret_core::TextWrap::WordBreak)?;

            Ok(vec![
                (
                    "text_measure_paint.max_width_row.wrap_width_delta",
                    max_width_delta(measured, prepared),
                ),
                (
                    "text_measure_paint.max_width_row.prepared_height",
                    prepared_metrics.size.height.0,
                ),
                (
                    "text_measure_paint.max_width_row.sibling_after_painted_height",
                    sibling_bounds.origin.y.0
                        - (wrapped_bounds.origin.y.0 + prepared_metrics.size.height.0),
                ),
            ])
        }
        LayoutPrimitiveScenario::TextMeasurePaintOverflowScale => {
            let (measured, prepared, _prepared_metrics) =
                text_constraint_pair_for_width(services, Px(40.0), fret_core::TextWrap::None)?;

            Ok(vec![
                (
                    "text_measure_paint.overflow_scale.wrap_width_delta",
                    max_width_delta(measured, prepared),
                ),
                (
                    "text_measure_paint.overflow_scale.scale_delta",
                    measured.scale_factor - prepared.scale_factor,
                ),
                (
                    "text_measure_paint.overflow_scale.wrap_same",
                    bool_metric(measured.wrap == prepared.wrap),
                ),
                (
                    "text_measure_paint.overflow_scale.overflow_same",
                    bool_metric(measured.overflow == prepared.overflow),
                ),
            ])
        }
        _ => Ok(Vec::new()),
    }
}

fn child_at(
    ui: &UiTree<TestHost>,
    node: NodeId,
    index: usize,
    label: &str,
) -> Result<NodeId, ScenarioObserveError> {
    ui.children(node)
        .get(index)
        .copied()
        .ok_or_else(|| ScenarioObserveError::new(format!("missing {label} at index {index}")))
}

fn text_constraint_pair_for_width(
    services: &LayoutPrimitiveServices,
    width: Px,
    wrap: fret_core::TextWrap,
) -> Result<(TextConstraints, TextConstraints, TextMetrics), ScenarioObserveError> {
    let measured = services
        .measured
        .iter()
        .copied()
        .find(|constraints| {
            constraints.wrap == wrap
                && constraints
                    .max_width
                    .is_some_and(|max| (max.0 - width.0).abs() < 0.01)
        })
        .ok_or_else(|| ScenarioObserveError::new("missing measured text constraints"))?;
    let (prepared, metrics) = services
        .prepared
        .iter()
        .copied()
        .zip(services.prepared_metrics.iter().copied())
        .find(|(constraints, _metrics)| {
            constraints.wrap == wrap
                && constraints
                    .max_width
                    .is_some_and(|max| (max.0 - width.0).abs() < 0.01)
        })
        .ok_or_else(|| ScenarioObserveError::new("missing prepared text constraints"))?;

    Ok((measured, prepared, metrics))
}

fn widest_text_constraint_pair_for_wrap(
    services: &LayoutPrimitiveServices,
    wrap: fret_core::TextWrap,
) -> Result<(TextConstraints, TextConstraints, TextMetrics), ScenarioObserveError> {
    let measured = services
        .measured
        .iter()
        .copied()
        .filter(|constraints| constraints.wrap == wrap)
        .max_by(|a, b| {
            a.max_width
                .unwrap_or(Px(0.0))
                .0
                .total_cmp(&b.max_width.unwrap_or(Px(0.0)).0)
        })
        .ok_or_else(|| ScenarioObserveError::new("missing measured wrapped text constraints"))?;
    let (prepared, metrics) = services
        .prepared
        .iter()
        .copied()
        .zip(services.prepared_metrics.iter().copied())
        .filter(|(constraints, _metrics)| constraints.wrap == wrap)
        .max_by(|(a, _), (b, _)| {
            a.max_width
                .unwrap_or(Px(0.0))
                .0
                .total_cmp(&b.max_width.unwrap_or(Px(0.0)).0)
        })
        .ok_or_else(|| ScenarioObserveError::new("missing prepared wrapped text constraints"))?;

    Ok((measured, prepared, metrics))
}

fn max_width_delta(measured: TextConstraints, prepared: TextConstraints) -> f32 {
    measured.max_width.unwrap_or(Px(0.0)).0 - prepared.max_width.unwrap_or(Px(0.0)).0
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
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
        LayoutPrimitiveScenario::PercentSizeResolvesUnderDefiniteContainer => {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Px(Px(200.0));
            outer.layout.size.height = Length::Px(Px(80.0));

            let mut percent = crate::element::ContainerProps::default();
            percent.layout.size.width = Length::Fraction(0.5);
            percent.layout.size.height = Length::Fraction(0.25);

            vec![cx.container(outer, |cx| {
                vec![
                    cx.container(percent, |_cx| Vec::new())
                        .test_id("percent-box"),
                ]
            })]
        }
        LayoutPrimitiveScenario::MinMaxClampsFillChild => {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Px(Px(200.0));
            outer.layout.size.height = Length::Px(Px(80.0));

            let mut clamped = crate::element::ContainerProps::default();
            clamped.layout.size.width = Length::Fill;
            clamped.layout.size.height = Length::Fill;
            clamped.layout.size.min_width = Some(Length::Px(Px(80.0)));
            clamped.layout.size.max_width = Some(Length::Px(Px(120.0)));
            clamped.layout.size.min_height = Some(Length::Px(Px(16.0)));
            clamped.layout.size.max_height = Some(Length::Px(Px(48.0)));

            vec![cx.container(outer, |cx| {
                vec![
                    cx.container(clamped, |_cx| Vec::new())
                        .test_id("min-max-clamped"),
                ]
            })]
        }
        LayoutPrimitiveScenario::PercentMinMaxBehavesLikeAutoUnderIndefiniteMeasure => {
            let mut max_props = crate::element::ContainerProps::default();
            max_props.layout.size.width = Length::Px(Px(150.0));
            max_props.layout.size.max_width = Some(Length::Fraction(0.5));

            let mut min_props = crate::element::ContainerProps::default();
            min_props.layout.size.width = Length::Px(Px(10.0));
            min_props.layout.size.min_width = Some(Length::Fraction(0.5));

            vec![
                cx.container(max_props, |cx| vec![cx.text("x")])
                    .test_id("percent-max-measure"),
                cx.container(min_props, |cx| vec![cx.text("x")])
                    .test_id("percent-min-measure"),
            ]
        }
        LayoutPrimitiveScenario::TextWrapHeightContributesToRow => {
            let row_layout = crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Px(Px(100.0)),
                    ..Default::default()
                },
                ..Default::default()
            };
            let row = crate::element::FlexProps {
                layout: row_layout,
                direction: fret_core::Axis::Horizontal,
                gap: Px(4.0).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
                ..Default::default()
            };
            let mut wrapped_layout = crate::element::LayoutStyle::default();
            wrapped_layout.flex.grow = 1.0;
            wrapped_layout.flex.shrink = 1.0;
            wrapped_layout.size.min_width = Some(Length::Px(Px(0.0)));

            vec![
                cx.flex(row, |cx| {
                    vec![
                        cx.text("x").test_id("wrap-bullet"),
                        cx.text_props(crate::element::TextProps {
                            layout: wrapped_layout,
                            text: Arc::from("wrap-wrap-wrap-wrap"),
                            style: None,
                            color: None,
                            wrap: fret_core::TextWrap::WordBreak,
                            overflow: fret_core::TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: crate::element::TextInkOverflow::None,
                        })
                        .test_id("wrapped-text"),
                    ]
                })
                .test_id("wrap-row"),
            ]
        }
        LayoutPrimitiveScenario::TextMeasurePaintWrapWidthColumn => {
            let mut text_props = crate::element::TextProps::new("wrap me");
            text_props.layout.size.width = Length::Px(Px(40.0));
            text_props.wrap = fret_core::TextWrap::Word;

            let mut sibling = crate::element::ContainerProps::default();
            sibling.layout.size.width = Length::Fill;
            sibling.layout.size.height = Length::Px(Px(10.0));

            vec![
                cx.column(crate::element::ColumnProps::default(), |cx| {
                    vec![
                        cx.text_props(text_props).test_id("measure-paint-text"),
                        cx.container(sibling, |_cx| Vec::new())
                            .test_id("measure-paint-sibling"),
                    ]
                })
                .test_id("measure-paint-column"),
            ]
        }
        LayoutPrimitiveScenario::TextMeasurePaintWrapWidthMaxWidthRow => {
            let mut container_layout = crate::element::LayoutStyle::default();
            container_layout.size.width = Length::Fill;
            container_layout.size.max_width = Some(Length::Px(Px(90.0)));

            let mut row_layout = crate::element::LayoutStyle::default();
            row_layout.size.width = Length::Fill;

            let mut wrapped_layout = crate::element::LayoutStyle::default();
            wrapped_layout.flex.grow = 1.0;
            wrapped_layout.flex.shrink = 1.0;
            wrapped_layout.size.min_width = Some(Length::Px(Px(0.0)));

            let mut sibling = crate::element::ContainerProps::default();
            sibling.layout.size.width = Length::Fill;
            sibling.layout.size.height = Length::Px(Px(10.0));

            vec![
                cx.container(
                    crate::element::ContainerProps {
                        layout: container_layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.column(crate::element::ColumnProps::default(), |cx| {
                                vec![
                                    cx.flex(
                                        crate::element::FlexProps {
                                            layout: row_layout,
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(4.0).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Start,
                                            wrap: false,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![
                                                cx.text("x").test_id("measure-paint-row-bullet"),
                                                cx.text_props(crate::element::TextProps {
                                                    layout: wrapped_layout,
                                                    text: Arc::from("wrapwrapwrap"),
                                                    style: None,
                                                    color: None,
                                                    wrap: fret_core::TextWrap::WordBreak,
                                                    overflow: fret_core::TextOverflow::Clip,
                                                    align: fret_core::TextAlign::Start,
                                                    ink_overflow:
                                                        crate::element::TextInkOverflow::None,
                                                })
                                                .test_id("measure-paint-row-text"),
                                            ]
                                        },
                                    )
                                    .test_id("measure-paint-row"),
                                    cx.container(sibling, |_cx| Vec::new())
                                        .test_id("measure-paint-row-sibling"),
                                ]
                            })
                            .test_id("measure-paint-row-column"),
                        ]
                    },
                )
                .test_id("measure-paint-row-container"),
            ]
        }
        LayoutPrimitiveScenario::TextMeasurePaintOverflowScale => {
            let mut text_props = crate::element::TextProps::new("ellipsis me");
            text_props.layout.size.width = Length::Px(Px(40.0));
            text_props.wrap = fret_core::TextWrap::None;
            text_props.overflow = fret_core::TextOverflow::Ellipsis;

            vec![
                cx.text_props(text_props)
                    .test_id("measure-paint-overflow-scale-text"),
            ]
        }
        LayoutPrimitiveScenario::ScrollRootPreservesChildLayoutBounds => {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Px(Px(120.0));
            scroll.layout.size.height = Length::Px(Px(40.0));
            scroll.axis = crate::element::ScrollAxis::Y;
            scroll.probe_unbounded = true;
            scroll.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;

            let mut child = crate::element::ContainerProps::default();
            child.layout.size.width = Length::Px(Px(120.0));
            child.layout.size.height = Length::Px(Px(80.0));

            vec![
                cx.scroll(scroll, |cx| {
                    vec![
                        cx.container(child, |_cx| Vec::new())
                            .test_id("scroll-content"),
                    ]
                })
                .test_id("scroll-root"),
            ]
        }
        LayoutPrimitiveScenario::AbsoluteInsetFractionResolvesAgainstContainingBlock => {
            let mut outer = crate::element::ContainerProps::default();
            outer.layout.size.width = Length::Px(Px(200.0));
            outer.layout.size.height = Length::Px(Px(100.0));
            outer.layout.position = crate::element::PositionStyle::Relative;

            let mut child = crate::element::ContainerProps::default();
            child.layout.position = crate::element::PositionStyle::Absolute;
            child.layout.inset.left = crate::element::InsetEdge::Fraction(0.25);
            child.layout.inset.top = crate::element::InsetEdge::Fraction(0.1);
            child.layout.size.width = Length::Px(Px(20.0));
            child.layout.size.height = Length::Px(Px(10.0));

            vec![cx.container(outer, |cx| {
                vec![
                    cx.container(child, |_cx| Vec::new())
                        .test_id("absolute-percent-child"),
                ]
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
        LayoutPrimitiveScenario::FlexPercentBasisKeepsAutoHeightFixedSibling => {
            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = Length::Px(Px(120.0));
            let stack = crate::element::FlexProps {
                layout: stack_layout,
                direction: fret_core::Axis::Vertical,
                gap: Px(6.0).into(),
                align: CrossAlign::Start,
                ..Default::default()
            };

            let mut content = crate::element::ContainerProps::default();
            content.layout.flex.grow = 1.0;
            content.layout.flex.shrink = 1.0;
            content.layout.flex.basis = Length::Fraction(0.0);
            content.layout.size.min_width = Some(Length::Px(Px(0.0)));

            let mut control = crate::element::ContainerProps::default();
            control.layout.size.width = Length::Px(Px(80.0));
            control.layout.size.height = Length::Px(Px(36.0));

            vec![
                cx.flex(stack, |cx| {
                    vec![
                        cx.container(content, |cx| vec![cx.text("x")])
                            .test_id("flex-percent-content"),
                        cx.container(control, |_cx| Vec::new())
                            .test_id("flex-percent-fixed-sibling"),
                    ]
                })
                .test_id("flex-percent-stack"),
            ]
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

#[derive(Default)]
struct LayoutPrimitiveServices {
    measured: Vec<TextConstraints>,
    prepared: Vec<TextConstraints>,
    prepared_metrics: Vec<TextMetrics>,
}

impl TextService for LayoutPrimitiveServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let metrics = self.fake_metrics(input, constraints);
        self.prepared.push(constraints);
        self.prepared_metrics.push(metrics);
        (fret_core::TextBlobId::default(), metrics)
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}

    fn measure(
        &mut self,
        input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> TextMetrics {
        self.measured.push(constraints);
        self.fake_metrics(input, constraints)
    }
}

impl LayoutPrimitiveServices {
    fn fake_metrics(
        &self,
        input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let char_w = 10.0f32;
        let line_h = 10.0f32;
        let char_count = input.text().chars().count().max(1) as f32;
        let natural_w = char_count * char_w;
        let max_w = constraints.max_width.map(|width| width.0.max(0.0));

        let lines = match (constraints.wrap, max_w) {
            (fret_core::TextWrap::None, _) | (_, None) => 1.0,
            (fret_core::TextWrap::Word, Some(width)) if width <= 0.01 => {
                let longest = input
                    .text()
                    .split_whitespace()
                    .map(|segment| segment.chars().count() as f32)
                    .fold(1.0f32, f32::max);
                (natural_w / (longest * char_w).max(char_w)).ceil().max(1.0)
            }
            (_, Some(width)) => {
                let wrap_w = width.max(char_w);
                (natural_w / wrap_w).ceil().max(1.0)
            }
        };
        let width = match (constraints.wrap, max_w) {
            (fret_core::TextWrap::None, _) | (_, None) => natural_w,
            (_, Some(width)) => natural_w.min(width.max(char_w)),
        };

        TextMetrics {
            size: Size::new(Px(width), Px(line_h * lines)),
            baseline: Px(8.0),
        }
    }
}

impl fret_core::PathService for LayoutPrimitiveServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for LayoutPrimitiveServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for LayoutPrimitiveServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
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
