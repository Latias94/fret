use super::*;

use fret_mechanism_harness::{
    BoundsSpace, MechanismCase, MechanismHarness, MechanismSuite, ObservedHitTestSample,
    ObservedNode, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;
use slotmap::Key as _;

const HIT_TEST_ROUTING: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/hit_test_routing_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum HitTestRoutingScenario {
    VisualTransformKeepsLayoutHitBounds,
    RenderTransformMovesHitBounds,
    OverflowClipSuppressesEscapedChildHit,
    TransparentWrapperPreservesChildHit,
    NonHitTestableGateSuppressesChildHit,
    MaskLayerBoundsDoNotClipHitTestingByDefault,
    MaskLayerOverflowClipSuppressesEscapedChildHit,
    EffectLayerBoundsDoNotClipHitTestingByDefault,
    EffectLayerOverflowClipSuppressesEscapedChildHit,
    CompositeGroupBoundsDoNotClipHitTestingByDefault,
    CompositeGroupOverflowClipSuppressesEscapedChildHit,
    OverlayRootZOrderWins,
    ModalBarrierRootSuppressesUnderlay,
}

#[derive(Default)]
struct Capture {
    transformed: Option<crate::elements::GlobalElementId>,
}

struct HitTestTransparent;

impl<H: UiHost> Widget<H> for HitTestTransparent {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn mechanism_harness_hit_test_routing_matches_oracles() {
    let suite: MechanismSuite<HitTestRoutingScenario> =
        MechanismSuite::from_json_str(HIT_TEST_ROUTING).expect("hit-test routing fixture suite");

    let mut observer: fn(
        &MechanismCase<HitTestRoutingScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<HitTestRoutingScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match case.scenario {
        HitTestRoutingScenario::OverlayRootZOrderWins => observe_overlay_root_z_order_case(),
        HitTestRoutingScenario::ModalBarrierRootSuppressesUnderlay => {
            observe_modal_barrier_root_case()
        }
        _ => observe_declarative_case(case),
    }
}

fn observe_declarative_case(
    case: &MechanismCase<HitTestRoutingScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = window_bounds();
    let mut services = FakeTextService::default();
    let mut capture = Capture::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-hit-test-routing",
        |cx| build_declarative_scenario(cx, &case.scenario, &mut capture),
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    if matches!(
        case.scenario,
        HitTestRoutingScenario::VisualTransformKeepsLayoutHitBounds
            | HitTestRoutingScenario::RenderTransformMovesHitBounds
    ) {
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = observe_tree(&ui, &snapshot, bounds);

    match case.scenario {
        HitTestRoutingScenario::VisualTransformKeepsLayoutHitBounds => {
            observe_transform_bounds(
                &mut app,
                &mut observed,
                window,
                capture.transformed,
                "visual-transform-target",
                false,
            )?;
            push_transform_samples(&ui, &snapshot, &mut observed, "visual-transform-target")?;
        }
        HitTestRoutingScenario::RenderTransformMovesHitBounds => {
            observe_transform_bounds(
                &mut app,
                &mut observed,
                window,
                capture.transformed,
                "render-transform-target",
                true,
            )?;
            push_transform_samples(&ui, &snapshot, &mut observed, "render-transform-target")?;
        }
        HitTestRoutingScenario::OverflowClipSuppressesEscapedChildHit => {
            let wrapper = bounds_for_test_id(&observed, "clip-wrapper", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "visible-child-area",
                Point::new(Px(wrapper.origin.x.0 + 10.0), Px(wrapper.origin.y.0 + 10.0)),
            ));
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "escaped-child-area",
                Point::new(Px(wrapper.origin.x.0 + 30.0), Px(wrapper.origin.y.0 + 10.0)),
            ));
        }
        HitTestRoutingScenario::TransparentWrapperPreservesChildHit => {
            let wrapper =
                bounds_for_test_id(&observed, "transparent-wrapper", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "wrapper-center",
                rect_center(wrapper),
            ));
        }
        HitTestRoutingScenario::NonHitTestableGateSuppressesChildHit => {
            let gate = bounds_for_test_id(&observed, "hit-test-gate", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "gate-center",
                rect_center(gate),
            ));
        }
        HitTestRoutingScenario::MaskLayerBoundsDoNotClipHitTestingByDefault
        | HitTestRoutingScenario::MaskLayerOverflowClipSuppressesEscapedChildHit => {
            let wrapper = bounds_for_test_id(&observed, "mask-wrapper", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "mask-escaped-child-area",
                Point::new(Px(wrapper.origin.x.0 + 34.0), Px(wrapper.origin.y.0 + 10.0)),
            ));
        }
        HitTestRoutingScenario::EffectLayerBoundsDoNotClipHitTestingByDefault
        | HitTestRoutingScenario::EffectLayerOverflowClipSuppressesEscapedChildHit => {
            let wrapper = bounds_for_test_id(&observed, "effect-wrapper", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "effect-escaped-child-area",
                Point::new(Px(wrapper.origin.x.0 + 34.0), Px(wrapper.origin.y.0 + 10.0)),
            ));
        }
        HitTestRoutingScenario::CompositeGroupBoundsDoNotClipHitTestingByDefault
        | HitTestRoutingScenario::CompositeGroupOverflowClipSuppressesEscapedChildHit => {
            let wrapper = bounds_for_test_id(&observed, "composite-wrapper", BoundsSpace::Layout)?;
            observed.push_hit_test_sample(hit_sample(
                &ui,
                &snapshot,
                "composite-escaped-child-area",
                Point::new(Px(wrapper.origin.x.0 + 34.0), Px(wrapper.origin.y.0 + 10.0)),
            ));
        }
        HitTestRoutingScenario::OverlayRootZOrderWins
        | HitTestRoutingScenario::ModalBarrierRootSuppressesUnderlay => unreachable!(),
    }

    Ok(observed)
}

fn observe_overlay_root_z_order_case() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = window_bounds();
    let mut services = FakeTextService::default();

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-hit-test-base",
        |cx| {
            vec![absolute_pressable(
                cx,
                "underlay-target",
                10.0,
                10.0,
                60.0,
                40.0,
            )]
        },
    );
    ui.set_root(base_root);

    let overlay_root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-hit-test-overlay",
        |cx| {
            vec![absolute_pressable(
                cx,
                "overlay-target",
                10.0,
                10.0,
                60.0,
                40.0,
            )]
        },
    );
    ui.push_overlay_root(overlay_root, false);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = observe_tree(&ui, &snapshot, bounds);
    ensure_test_id_for_node(&ui, &mut observed, bounds, base_root, "base-root");
    ensure_test_id_for_node(&ui, &mut observed, bounds, overlay_root, "overlay-root");

    let target = bounds_for_test_id(&observed, "overlay-target", BoundsSpace::Layout)?;
    observed.push_hit_test_sample(hit_sample(
        &ui,
        &snapshot,
        "overlap-center",
        rect_center(target),
    ));

    Ok(observed)
}

fn observe_modal_barrier_root_case() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = window_bounds();
    let mut services = FakeTextService::default();

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mechanism-harness-hit-test-modal-base",
        |cx| {
            vec![absolute_pressable(
                cx,
                "modal-underlay-target",
                10.0,
                10.0,
                60.0,
                40.0,
            )]
        },
    );
    ui.set_root(base_root);

    let barrier_root = ui.create_node(HitTestTransparent);
    ui.push_overlay_root_with_options(
        barrier_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: true,
            hit_testable: false,
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing semantics snapshot"))?;
    let mut observed = observe_tree(&ui, &snapshot, bounds);
    ensure_test_id_for_node(&ui, &mut observed, bounds, base_root, "modal-base-root");
    ensure_test_id_for_node(
        &ui,
        &mut observed,
        bounds,
        barrier_root,
        "modal-barrier-root",
    );

    let target = bounds_for_test_id(&observed, "modal-underlay-target", BoundsSpace::Layout)?;
    observed.push_hit_test_sample(hit_sample(
        &ui,
        &snapshot,
        "underlay-center",
        rect_center(target),
    ));

    Ok(observed)
}

fn build_declarative_scenario(
    cx: &mut ElementContext<'_, TestHost>,
    scenario: &HitTestRoutingScenario,
    capture: &mut Capture,
) -> Vec<AnyElement> {
    match scenario {
        HitTestRoutingScenario::VisualTransformKeepsLayoutHitBounds => {
            let transform = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            vec![cx.visual_transform(transform, |cx| {
                vec![tracked_pressable(cx, "visual-transform-target", capture)]
            })]
        }
        HitTestRoutingScenario::RenderTransformMovesHitBounds => {
            let transform = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            vec![cx.render_transform(transform, |cx| {
                vec![tracked_pressable(cx, "render-transform-target", capture)]
            })]
        }
        HitTestRoutingScenario::OverflowClipSuppressesEscapedChildHit => {
            let mut wrapper = crate::element::ContainerProps::default();
            wrapper.layout = absolute_layout(0.0, 0.0, 20.0, 20.0);
            wrapper.layout.overflow = crate::element::Overflow::Clip;

            vec![
                cx.container(wrapper, |cx| {
                    vec![absolute_pressable(
                        cx,
                        "clipped-child",
                        0.0,
                        0.0,
                        40.0,
                        20.0,
                    )]
                })
                .test_id("clip-wrapper"),
            ]
        }
        HitTestRoutingScenario::TransparentWrapperPreservesChildHit => {
            let mut semantics = crate::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: Some(Arc::from("transparent hit-test wrapper")),
                ..Default::default()
            };
            semantics.layout = absolute_layout(10.0, 10.0, 40.0, 20.0);

            vec![
                cx.semantics(semantics, |cx| {
                    vec![fill_pressable(cx, "transparent-child")]
                })
                .test_id("transparent-wrapper"),
            ]
        }
        HitTestRoutingScenario::NonHitTestableGateSuppressesChildHit => {
            let gate = crate::element::HitTestGateProps {
                layout: absolute_layout(10.0, 10.0, 40.0, 20.0),
                hit_test: false,
            };

            vec![
                cx.hit_test_gate_props(gate, |cx| vec![fill_pressable(cx, "gated-child")])
                    .test_id("hit-test-gate"),
            ]
        }
        HitTestRoutingScenario::MaskLayerBoundsDoNotClipHitTestingByDefault => {
            vec![mask_layer_with_escaped_pressable(
                cx,
                crate::element::Overflow::Visible,
            )]
        }
        HitTestRoutingScenario::MaskLayerOverflowClipSuppressesEscapedChildHit => {
            vec![mask_layer_with_escaped_pressable(
                cx,
                crate::element::Overflow::Clip,
            )]
        }
        HitTestRoutingScenario::EffectLayerBoundsDoNotClipHitTestingByDefault => {
            vec![effect_layer_with_escaped_pressable(
                cx,
                crate::element::Overflow::Visible,
            )]
        }
        HitTestRoutingScenario::EffectLayerOverflowClipSuppressesEscapedChildHit => {
            vec![effect_layer_with_escaped_pressable(
                cx,
                crate::element::Overflow::Clip,
            )]
        }
        HitTestRoutingScenario::CompositeGroupBoundsDoNotClipHitTestingByDefault => {
            vec![composite_group_with_escaped_pressable(
                cx,
                crate::element::Overflow::Visible,
            )]
        }
        HitTestRoutingScenario::CompositeGroupOverflowClipSuppressesEscapedChildHit => {
            vec![composite_group_with_escaped_pressable(
                cx,
                crate::element::Overflow::Clip,
            )]
        }
        HitTestRoutingScenario::OverlayRootZOrderWins
        | HitTestRoutingScenario::ModalBarrierRootSuppressesUnderlay => unreachable!(),
    }
}

fn observe_tree(
    ui: &UiTree<TestHost>,
    snapshot: &fret_core::SemanticsSnapshot,
    bounds: Rect,
) -> ObservedTree {
    let mut observed = ObservedTree::from_semantics_snapshot(snapshot, bounds);
    for node in snapshot.nodes.iter() {
        if let Some(layout) = ui.debug_node_bounds(node.id) {
            observed.set_layout_bounds_for_node_id(node.id.data().as_ffi(), layout);
        }
    }
    observed
}

fn observe_transform_bounds(
    app: &mut TestHost,
    observed: &mut ObservedTree,
    window: AppWindowId,
    element: Option<crate::elements::GlobalElementId>,
    test_id: &str,
    hit_follows_visual: bool,
) -> Result<(), ScenarioObserveError> {
    let element =
        element.ok_or_else(|| ScenarioObserveError::new("missing transformed element"))?;
    let layout = crate::elements::current_bounds_for_element(app, window, element)
        .ok_or_else(|| ScenarioObserveError::new("missing transformed layout bounds"))?;
    let visual = if hit_follows_visual {
        crate::elements::current_visual_bounds_for_element(app, window, element)
            .ok_or_else(|| ScenarioObserveError::new("missing transformed visual bounds"))?
    } else {
        translate_rect(layout, 40.0, 0.0)
    };

    observed.set_space_bounds_for_test_id(test_id, BoundsSpace::Layout, layout);
    observed.set_visual_bounds_for_test_id(test_id, visual);
    observed.set_hit_bounds_for_test_id(test_id, if hit_follows_visual { visual } else { layout });
    Ok(())
}

fn push_transform_samples(
    ui: &UiTree<TestHost>,
    snapshot: &fret_core::SemanticsSnapshot,
    observed: &mut ObservedTree,
    test_id: &str,
) -> Result<(), ScenarioObserveError> {
    let layout = bounds_for_test_id(observed, test_id, BoundsSpace::Layout)?;
    let visual = bounds_for_test_id(observed, test_id, BoundsSpace::Visual)?;
    observed.push_hit_test_sample(hit_sample(
        ui,
        snapshot,
        "layout-center",
        rect_center(layout),
    ));
    observed.push_hit_test_sample(hit_sample(
        ui,
        snapshot,
        "visual-center",
        rect_center(visual),
    ));
    Ok(())
}

fn tracked_pressable(
    cx: &mut ElementContext<'_, TestHost>,
    test_id: &'static str,
    capture: &mut Capture,
) -> AnyElement {
    let mut props = crate::element::PressableProps::default();
    props.layout.size.width = Length::Px(Px(20.0));
    props.layout.size.height = Length::Px(Px(20.0));
    cx.pressable_with_id(props, |_cx, _state, id| {
        capture.transformed = Some(id);
        Vec::new()
    })
    .test_id(test_id)
}

fn absolute_pressable(
    cx: &mut ElementContext<'_, TestHost>,
    test_id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> AnyElement {
    let mut props = crate::element::PressableProps::default();
    props.layout = absolute_layout(x, y, w, h);
    cx.pressable(props, |_cx, _state| Vec::new())
        .test_id(test_id)
}

fn fill_pressable(cx: &mut ElementContext<'_, TestHost>, test_id: &'static str) -> AnyElement {
    let mut props = crate::element::PressableProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    cx.pressable(props, |_cx, _state| Vec::new())
        .test_id(test_id)
}

fn mask_layer_with_escaped_pressable(
    cx: &mut ElementContext<'_, TestHost>,
    overflow: crate::element::Overflow,
) -> AnyElement {
    let mut layout = absolute_layout(0.0, 0.0, 20.0, 20.0);
    layout.overflow = overflow;
    let props = crate::element::MaskLayerProps {
        layout,
        mask: fret_core::scene::Mask::Image {
            image: fret_core::ImageId::default(),
            uv: fret_core::scene::UvRect::FULL,
            sampling: fret_core::scene::ImageSamplingHint::Default,
        },
    };

    cx.mask_layer_props(props, |cx| {
        vec![absolute_pressable(
            cx,
            "mask-escaped-child",
            30.0,
            0.0,
            20.0,
            20.0,
        )]
    })
    .test_id("mask-wrapper")
}

fn effect_layer_with_escaped_pressable(
    cx: &mut ElementContext<'_, TestHost>,
    overflow: crate::element::Overflow,
) -> AnyElement {
    let mut layout = absolute_layout(0.0, 0.0, 20.0, 20.0);
    layout.overflow = overflow;
    let props = crate::element::EffectLayerProps {
        layout,
        mode: fret_core::EffectMode::FilterContent,
        chain: fret_core::EffectChain::from_steps(&[fret_core::EffectStep::Pixelate { scale: 2 }]),
        quality: fret_core::EffectQuality::Auto,
    };

    cx.effect_layer_props(props, |cx| {
        vec![absolute_pressable(
            cx,
            "effect-escaped-child",
            30.0,
            0.0,
            20.0,
            20.0,
        )]
    })
    .test_id("effect-wrapper")
}

fn composite_group_with_escaped_pressable(
    cx: &mut ElementContext<'_, TestHost>,
    overflow: crate::element::Overflow,
) -> AnyElement {
    let mut layout = absolute_layout(0.0, 0.0, 20.0, 20.0);
    layout.overflow = overflow;
    let props = crate::element::CompositeGroupProps {
        layout,
        mode: fret_core::scene::BlendMode::Add,
        quality: fret_core::EffectQuality::Auto,
    };

    cx.composite_group_props(props, |cx| {
        vec![absolute_pressable(
            cx,
            "composite-escaped-child",
            30.0,
            0.0,
            20.0,
            20.0,
        )]
    })
    .test_id("composite-wrapper")
}

fn absolute_layout(x: f32, y: f32, w: f32, h: f32) -> crate::element::LayoutStyle {
    let mut layout = crate::element::LayoutStyle {
        position: crate::element::PositionStyle::Absolute,
        ..Default::default()
    };
    layout.inset.left = Some(Px(x)).into();
    layout.inset.top = Some(Px(y)).into();
    layout.size.width = Length::Px(Px(w));
    layout.size.height = Length::Px(Px(h));
    layout
}

fn bounds_for_test_id(
    observed: &ObservedTree,
    test_id: &str,
    space: BoundsSpace,
) -> Result<Rect, ScenarioObserveError> {
    observed
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .map(|node| node.bounds_in(space))
        .ok_or_else(|| ScenarioObserveError::new(format!("missing observed test_id {test_id:?}")))
}

fn ensure_test_id_for_node(
    ui: &UiTree<TestHost>,
    observed: &mut ObservedTree,
    window_bounds: Rect,
    node: NodeId,
    test_id: &'static str,
) {
    let node_id = node.data().as_ffi();
    if observed.set_test_id_for_node_id(node_id, test_id) {
        return;
    }

    let bounds = ui.debug_node_bounds(node).unwrap_or(window_bounds);
    let mut observed_node = ObservedNode::new(test_id, bounds);
    observed_node.node_id = Some(node_id);
    observed_node.visible = true;
    observed.push_node(observed_node);
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

fn rect_center(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn translate_rect(rect: Rect, dx: f32, dy: f32) -> Rect {
    Rect::new(
        Point::new(Px(rect.origin.x.0 + dx), Px(rect.origin.y.0 + dy)),
        rect.size,
    )
}

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(100.0)),
    )
}
