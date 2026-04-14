#![allow(clippy::arc_with_non_send_sync)]

use super::*;
use fret_runtime::{GlobalsHost, PlatformCapabilities};

#[test]
fn keyed_elements_reuse_node_ids_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let mut items: Vec<u64> = vec![1, 2, 3];
    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

    let mut prev: std::collections::HashMap<
        u64,
        (crate::elements::GlobalElementId, fret_core::NodeId),
    > = std::collections::HashMap::new();

    let mut root: Option<fret_core::NodeId> = None;

    for pass in 0..2 {
        ids.clear();
        let r = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp49",
            |cx| build_keyed_rows(cx, &items, &mut ids),
        );
        root.get_or_insert(r);

        let cur: std::collections::HashMap<
            u64,
            (crate::elements::GlobalElementId, fret_core::NodeId),
        > = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
            runtime.prepare_window_for_frame(window, app.frame_id());
            let st = runtime.for_window_mut(window);
            ids.iter()
                .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                .collect()
        });

        if pass == 1 {
            for item in [1u64, 2u64, 3u64] {
                let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                assert_eq!(
                    prev_id, cur_id,
                    "element id should be stable for item {item}"
                );
                assert_eq!(
                    prev_node, cur_node,
                    "node id should be stable for item {item}"
                );
            }
        }

        prev = cur;
        items.reverse();
        app.advance_frame();
    }

    assert_eq!(ui.children(root.unwrap()).len(), 3);
}

#[test]
fn opacity_element_emits_opacity_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "opacity-element-emits-ops",
        |cx| {
            vec![cx.opacity(0.5, |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.background = Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                vec![cx.container(props, |_| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushOpacity { opacity } if (opacity - 0.5).abs() < 1e-6
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopOpacity));
}

#[test]
fn effect_layer_element_emits_effect_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "effect-layer-element-emits-ops",
        |cx| {
            let chain =
                fret_core::EffectChain::from_steps(&[fret_core::EffectStep::Pixelate { scale: 6 }]);
            vec![
                cx.effect_layer(fret_core::EffectMode::FilterContent, chain, |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.background = Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                    vec![cx.container(props, |_| Vec::new())]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushEffect {
            mode: fret_core::EffectMode::FilterContent,
            ..
        }
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopEffect));
}

#[test]
fn backdrop_source_group_element_emits_backdrop_source_group_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "backdrop-source-group-element-emits-ops",
        |cx| {
            let mut layout = crate::element::LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;

            let props = crate::element::BackdropSourceGroupProps {
                layout,
                pyramid: Some(fret_core::scene::CustomEffectPyramidRequestV1 {
                    max_levels: 6,
                    max_radius_px: Px(32.0),
                }),
                quality: fret_core::EffectQuality::Auto,
            };

            vec![cx.backdrop_source_group_v1_props(props, |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.background = Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                vec![cx.container(props, |_| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushBackdropSourceGroupV1 { bounds: b, .. } if b == bounds
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopBackdropSourceGroup));
}

#[test]
fn mask_layer_element_emits_mask_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "mask-layer-element-emits-ops",
        |cx| {
            let mut stops = [fret_core::scene::GradientStop::new(0.0, Color::TRANSPARENT);
                fret_core::scene::MAX_STOPS];
            stops[0] = fret_core::scene::GradientStop::new(
                0.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            );
            stops[1] = fret_core::scene::GradientStop::new(
                1.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            );

            let g = fret_core::scene::LinearGradient {
                start: fret_core::Point::new(Px(0.0), Px(0.0)),
                end: fret_core::Point::new(Px(100.0), Px(0.0)),
                tile_mode: fret_core::scene::TileMode::Clamp,
                color_space: fret_core::scene::ColorSpace::Srgb,
                stop_count: 2,
                stops,
            };
            let mask = fret_core::scene::Mask::linear_gradient(g);
            vec![cx.mask_layer(mask, |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.background = Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                vec![cx.container(props, |_| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(scene.ops()[0], SceneOp::PushMask { .. }));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopMask));
}

#[test]
fn composite_group_element_emits_composite_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "composite-group-element-emits-ops",
        |cx| {
            vec![cx.composite_group(fret_core::scene::BlendMode::Add, |cx| {
                let mut props = crate::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;
                props.background = Some(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                vec![cx.container(props, |_| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(scene.ops()[0], SceneOp::PushCompositeGroup { .. }));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopCompositeGroup));
}

#[test]
fn visual_transform_element_emits_transform_stack_ops() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "visual-transform-element-emits-ops",
        |cx| {
            vec![cx.visual_transform(
                Transform2D::translation(Point::new(Px(10.0), Px(0.0))),
                |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.background = Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                    vec![cx.container(props, |_| Vec::new())]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);
    assert!(matches!(
        scene.ops()[0],
        SceneOp::PushTransform { transform } if (transform.tx - 10.0).abs() < 1e-6
    ));
    assert!(matches!(scene.ops()[1], SceneOp::Quad { .. }));
    assert!(matches!(scene.ops()[2], SceneOp::PopTransform));
}

#[test]
fn visual_transform_does_not_affect_hit_testing() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "visual-transform-hit-test",
        |cx| {
            let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
            vec![cx.visual_transform(transform, |cx| {
                let mut props = crate::element::PressableProps::default();
                props.layout.size.width = Length::Px(Px(20.0));
                props.layout.size.height = Length::Px(Px(20.0));
                vec![cx.pressable(props, |_cx, _state| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let pressable_node = ui.children(transform_node)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");

    let original_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_eq!(
        ui.debug_hit_test(original_hit_pos).hit,
        Some(pressable_node)
    );

    let translated_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 52.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_ne!(
        ui.debug_hit_test(translated_hit_pos).hit,
        Some(pressable_node)
    );
}

#[test]
fn render_transform_affects_hit_testing() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "render-transform-hit-test",
        |cx| {
            let transform = Transform2D::translation(Point::new(Px(50.0), Px(0.0)));
            vec![cx.render_transform(transform, |cx| {
                let mut props = crate::element::PressableProps::default();
                props.layout.size.width = Length::Px(Px(20.0));
                props.layout.size.height = Length::Px(Px(20.0));
                vec![cx.pressable(props, |_cx, _state| Vec::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let pressable_node = ui.children(transform_node)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");

    let original_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_ne!(
        ui.debug_hit_test(original_hit_pos).hit,
        Some(pressable_node)
    );

    let translated_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 52.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );
    assert_eq!(
        ui.debug_hit_test(translated_hit_pos).hit,
        Some(pressable_node)
    );
}

#[test]
fn mask_layer_is_paint_only_for_hit_testing_by_default() {
    fn build_root(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        window: AppWindowId,
        bounds: Rect,
        overflow: crate::element::Overflow,
        test_id: &'static str,
    ) -> (NodeId, NodeId, Rect, Rect) {
        let mask = fret_core::scene::Mask::Image {
            image: fret_core::ImageId::default(),
            uv: fret_core::scene::UvRect::FULL,
            sampling: fret_core::scene::ImageSamplingHint::Default,
        };

        let root = render_root(ui, app, services, window, bounds, test_id, |cx| {
            let mut mask_layout = crate::element::LayoutStyle {
                position: crate::element::PositionStyle::Absolute,
                overflow,
                ..Default::default()
            };
            mask_layout.inset.left = Some(Px(0.0)).into();
            mask_layout.inset.top = Some(Px(0.0)).into();
            mask_layout.size.width = Length::Px(Px(20.0));
            mask_layout.size.height = Length::Px(Px(20.0));

            let mask_props = crate::element::MaskLayerProps {
                layout: mask_layout,
                mask,
            };

            vec![cx.mask_layer_props(mask_props, |cx| {
                let mut pressable_props = crate::element::PressableProps::default();
                pressable_props.layout.position = crate::element::PositionStyle::Absolute;
                pressable_props.layout.inset.left = Some(Px(30.0)).into();
                pressable_props.layout.inset.top = Some(Px(0.0)).into();
                pressable_props.layout.size.width = Length::Px(Px(20.0));
                pressable_props.layout.size.height = Length::Px(Px(20.0));

                vec![cx.pressable(pressable_props, |_cx, _state| Vec::new())]
            })]
        });

        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);

        let mask_node = ui.children(root)[0];
        let pressable_node = ui.children(mask_node)[0];

        let mask_bounds = ui.debug_node_bounds(mask_node).expect("mask bounds");
        let pressable_bounds = ui
            .debug_node_bounds(pressable_node)
            .expect("pressable bounds");

        (root, pressable_node, mask_bounds, pressable_bounds)
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let (_root, pressable_node, mask_bounds, pressable_bounds) = build_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        crate::element::Overflow::Visible,
        "mask-layer-hit-test-visible",
    );

    assert!(
        pressable_bounds.origin.x.0 >= mask_bounds.origin.x.0 + mask_bounds.size.width.0,
        "pressable should be outside the mask wrapper bounds for this hit-test gate"
    );

    let overflow_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );

    assert_eq!(
        ui.debug_hit_test(overflow_hit_pos).hit,
        Some(pressable_node)
    );

    app.advance_frame();

    let (_root, pressable_node, _mask_bounds, pressable_bounds) = build_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        crate::element::Overflow::Clip,
        "mask-layer-hit-test-clip",
    );

    let clipped_hit_pos = Point::new(
        Px(pressable_bounds.origin.x.0 + 2.0),
        Px(pressable_bounds.origin.y.0 + 2.0),
    );

    assert_ne!(ui.debug_hit_test(clipped_hit_pos).hit, Some(pressable_node));
}

#[test]
fn hit_test_gate_is_layout_transparent_for_intrinsic_sizing() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "hit-test-gate-layout-transparent",
        |cx| {
            let mut container_props = crate::element::ContainerProps::default();
            container_props.layout.position = crate::element::PositionStyle::Absolute;
            container_props.layout.inset.left = Some(Px(10.0)).into();
            container_props.layout.inset.top = Some(Px(30.0)).into();

            let container = cx.container(container_props, |cx| vec![cx.text("x")]);

            let gate_layout = crate::element::LayoutStyle {
                position: crate::element::PositionStyle::Absolute,
                inset: crate::element::InsetStyle {
                    left: Some(Px(10.0)).into(),
                    top: Some(Px(30.0)).into(),
                    ..Default::default()
                },
                ..Default::default()
            };

            let gate = cx.hit_test_gate_props(
                crate::element::HitTestGateProps {
                    layout: gate_layout,
                    hit_test: false,
                },
                |cx| vec![cx.text("x")],
            );

            vec![container, gate]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let children = ui.children(root);
    assert_eq!(children.len(), 2);

    let container_bounds = ui.debug_node_bounds(children[0]).expect("container bounds");
    let gate_bounds = ui.debug_node_bounds(children[1]).expect("gate bounds");

    assert_eq!(container_bounds.size, gate_bounds.size);
    assert_eq!(gate_bounds.size, Size::new(Px(10.0), Px(10.0)));
}

#[test]
fn key_hook_runs_for_focused_text_input() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let value = app.models_mut().insert(String::new());
    let invoked = app.models_mut().insert(0u32);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "key-hook-text-input",
        |cx| {
            let mut props = TextInputProps::new(value);
            props.layout.size.width = Length::Px(Px(160.0));
            props.layout.size.height = Length::Px(Px(32.0));
            let input = cx.text_input(props);

            let invoked = invoked.clone();
            cx.key_on_key_down_for(
                input.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat {
                        return false;
                    }
                    if down.key != fret_core::KeyCode::ArrowDown {
                        return false;
                    }
                    let _ = host.models_mut().update(&invoked, |v: &mut u32| *v += 1);
                    true
                }),
            );

            vec![input]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let input_node = ui.children(root)[0];
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 2.0),
        Px(input_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&invoked).unwrap_or_default(), 1);
}

#[test]
fn render_root_rebuild_refreshes_window_input_context_snapshot_before_paint() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let value = app.models_mut().insert(String::new());

    let mut input_element = None;
    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-input-ctx",
        |cx| {
            let input = cx.text_input(TextInputProps::new(value.clone()));
            input_element = Some(input.id);
            vec![input]
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let input_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        input_element.expect("text input element id"),
    )
    .expect("text input node");
    ui.set_focus(Some(input_node));

    let mut scene = Scene::default();
    paint_frame(&mut ui, &mut app, &mut text, bounds, &mut scene);

    let initial_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial window input context snapshot");
    assert!(
        initial_snapshot.focus_is_text_input,
        "baseline snapshot should reflect the focused text input before rebuild"
    );

    app.advance_frame();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-input-ctx",
        |cx| vec![cx.text("plain text")],
    );

    let rebuilt_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("rebuilt window input context snapshot");
    assert!(
        !rebuilt_snapshot.focus_is_text_input,
        "render_root should refresh the window input context snapshot once rebuild removed the focused text input"
    );
}

#[test]
fn render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let mut keyed_element = None;
    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-keyctx",
        |cx| {
            let element = cx.text("demo ctx").key_context("demo");
            keyed_element = Some(element.id);
            vec![element]
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let keyed_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        keyed_element.expect("keyed element id"),
    )
    .expect("keyed node");
    ui.set_focus(Some(keyed_node));
    ui.publish_window_command_action_availability_snapshot(
        &mut app,
        &fret_runtime::InputContext::default(),
    );

    let initial_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(initial_key_contexts, vec![Arc::<str>::from("demo")]);

    app.advance_frame();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-keyctx",
        |cx| vec![cx.text("plain text")],
    );

    let rebuilt_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert!(
        rebuilt_key_contexts.is_empty(),
        "render_root should refresh the published key-context stack after rebuild removed the focused keyed subtree"
    );
}

#[test]
fn render_root_rebuild_refreshes_command_action_availability_before_next_publish() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let command = CommandId::from("test.render_root_available");
    app.register_command(
        command.clone(),
        fret_runtime::CommandMeta::new("Render Root Available")
            .with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let mut availability_element = None;

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-action-availability",
        |cx| {
            let root = cx.container(crate::element::ContainerProps::default(), |_cx| Vec::new());
            availability_element = Some(root.id);
            cx.command_on_command_availability_for(
                root.id,
                Arc::new(|_host, _acx, requested| {
                    if requested.as_str() == "test.render_root_available" {
                        return crate::widget::CommandAvailability::Available;
                    }
                    crate::widget::CommandAvailability::NotHandled
                }),
            );
            vec![root]
        },
    );
    let availability_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        availability_element.expect("availability element id"),
    )
    .expect("availability node");
    ui.set_focus(Some(availability_node));
    ui.publish_window_command_action_availability_snapshot(
        &mut app,
        &fret_runtime::InputContext::default(),
    );

    let initial_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        initial_availability,
        Some(true),
        "baseline rebuild publish should expose the declarative command availability hook"
    );

    app.advance_frame();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-action-availability",
        |cx| vec![cx.container(crate::element::ContainerProps::default(), |_cx| Vec::new())],
    );

    let rebuilt_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        rebuilt_availability,
        Some(false),
        "render_root should republish widget command availability after rebuild removed the declarative hook"
    );
}

#[test]
fn render_root_rebuild_refreshes_focus_traversal_availability_before_layout() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    app.register_command(
        CommandId::from("focus.next"),
        fret_runtime::CommandMeta::new("Focus Next").with_scope(fret_runtime::CommandScope::Widget),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        fret_runtime::CommandMeta::new("Focus Previous")
            .with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-focus-traversal-availability",
        |cx| vec![cx.text("plain text")],
    );

    let initial_next = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.next")));
    let initial_previous = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.previous")));
    assert_eq!(initial_next, Some(false));
    assert_eq!(initial_previous, Some(false));

    app.advance_frame();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-focus-traversal-availability",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = Length::Px(Px(48.0));
                        layout.size.height = Length::Px(Px(20.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            )]
        },
    );

    let rebuilt_next = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.next")));
    let rebuilt_previous = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.previous")));
    assert_eq!(
        rebuilt_next,
        Some(true),
        "render_root should republish focus traversal availability as soon as rebuild introduces focusable descendants, even before layout"
    );
    assert_eq!(
        rebuilt_previous,
        Some(true),
        "render_root should republish reverse focus traversal availability as soon as rebuild introduces focusable descendants, even before layout"
    );
}

#[test]
fn layout_refines_focus_traversal_availability_after_structural_fallback() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    app.register_command(
        CommandId::from("focus.next"),
        fret_runtime::CommandMeta::new("Focus Next").with_scope(fret_runtime::CommandScope::Widget),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        fret_runtime::CommandMeta::new("Focus Previous")
            .with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "focus-traversal-layout-refine",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = Length::Px(Px(0.0));
                        layout.size.height = Length::Px(Px(0.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            )]
        },
    );

    let pre_layout_next = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.next")));
    let pre_layout_previous = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.previous")));
    assert_eq!(
        pre_layout_next,
        Some(true),
        "dirty rebuild should publish a structural fallback for focus traversal availability before layout resolves bounds"
    );
    assert_eq!(
        pre_layout_previous,
        Some(true),
        "dirty rebuild should publish the same structural fallback for reverse focus traversal availability before layout resolves bounds"
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let refined_next = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.next")));
    let refined_previous = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &CommandId::from("focus.previous")));
    assert_eq!(
        refined_next,
        Some(false),
        "final layout should republish authoritative focus traversal availability once structural fallback can be replaced with bounds-qualified truth"
    );
    assert_eq!(
        refined_previous,
        Some(false),
        "final layout should republish authoritative reverse focus traversal availability once structural fallback can be replaced with bounds-qualified truth"
    );
}

#[test]
fn focus_traversal_command_can_focus_rebuilt_declarative_nodes_before_layout() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let mut focusable_element = None;

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-focus-traverse-dispatch",
        |cx| vec![cx.text("plain text")],
    );

    app.advance_frame();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "rebuild-focus-traverse-dispatch",
        |cx| {
            vec![cx.pressable_with_id(
                crate::element::PressableProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = Length::Px(Px(48.0));
                        layout.size.height = Length::Px(Px(20.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    focusable_element = Some(id);
                    Vec::new()
                },
            )]
        },
    );

    let did_handle = ui.dispatch_command(&mut app, &mut text, &CommandId::from("focus.next"));
    assert!(
        did_handle,
        "focus.next should still handle when a declarative rebuild introduces focusable nodes before layout runs"
    );

    let focusable_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        focusable_element.expect("focusable element id"),
    )
    .expect("focusable node");
    assert_eq!(
        ui.focus(),
        Some(focusable_node),
        "focus.next should use the same structural declarative fallback as availability when rebuild happens before layout"
    );
}

#[test]
fn render_dismissible_root_initial_attach_commits_window_snapshot_after_root_attachment() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "dismissible-attach-base",
        |cx| vec![cx.text("base")],
    );

    let initial_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial window input context snapshot");
    assert!(
        !initial_snapshot.ui_has_modal,
        "baseline snapshot should not report a modal before overlay attachment"
    );

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "dismissible-attach-overlay",
        |cx| vec![cx.text("overlay")],
    );

    let snapshot_after_render = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("snapshot after detached dismissible rebuild");
    assert!(
        !snapshot_after_render.ui_has_modal,
        "detached dismissible rebuild must not publish a modal snapshot before the root is attached"
    );

    let _layer = ui.push_overlay_root(overlay_root, true);
    let stale_after_attach = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("stale snapshot after raw overlay attach");
    assert!(
        !stale_after_attach.ui_has_modal,
        "raw overlay attachment should remain stale until the pending declarative commit is finished"
    );

    assert!(
        ui.commit_pending_declarative_window_runtime_snapshots(&mut app, overlay_root),
        "expected detached dismissible root attachment to finish a pending window snapshot commit"
    );

    let committed_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("committed window input context snapshot");
    assert!(committed_snapshot.ui_has_modal);
    assert_eq!(
        committed_snapshot
            .window_arbitration
            .expect("window arbitration")
            .modal_barrier_root,
        Some(overlay_root),
        "the committed snapshot should reflect the attached modal barrier root"
    );
}

#[test]
fn render_dismissible_root_parent_attach_commits_window_snapshot_after_root_attachment() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let value = app.models_mut().insert(String::new());

    let base_root = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "dismissible-parent-attach-base",
        |cx| vec![cx.text("base")],
    );

    let initial_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial window input context snapshot");
    assert!(
        !initial_snapshot.focus_is_text_input,
        "baseline snapshot should not report text-input focus before the detached root is attached"
    );

    let attached_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "dismissible-parent-attach-overlay",
        |cx| vec![cx.text_input(TextInputProps::new(value.clone()))],
    );

    ui.set_children(base_root, vec![attached_root]);
    let text_input = ui.children(attached_root)[0];
    ui.set_focus(Some(text_input));

    assert!(
        ui.commit_pending_declarative_window_runtime_snapshots(&mut app, attached_root),
        "expected parent attachment to finish the pending declarative window snapshot commit"
    );

    let committed_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("committed window input context snapshot");
    assert!(
        committed_snapshot.focus_is_text_input,
        "committed snapshot should observe text-input focus once the detached root is attached as a child"
    );
}

#[test]
fn nested_render_root_during_layout_defers_window_snapshot_publish_until_post_layout() {
    #[derive(Debug, Default, Clone, Copy)]
    struct NestedRenderRootSnapshotProbe {
        saw_snapshot_during_layout: bool,
    }

    #[derive(Debug, Default)]
    struct NestedRenderRootLayoutWidget;

    impl Widget<TestHost> for NestedRenderRootLayoutWidget {
        fn layout(&mut self, cx: &mut LayoutCx<'_, TestHost>) -> Size {
            let window = cx.window.expect("window");
            let nested = render_root(
                cx.tree,
                cx.app,
                cx.services,
                window,
                cx.bounds,
                "nested-layout-root",
                |el| vec![el.text("nested")],
            );
            cx.tree.set_children(cx.node, vec![nested]);

            let saw_snapshot_during_layout = cx
                .app
                .global::<fret_runtime::WindowCommandActionAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .is_some();
            cx.app.with_global_mut_untracked(
                NestedRenderRootSnapshotProbe::default,
                |probe, _app| {
                    probe.saw_snapshot_during_layout = saw_snapshot_during_layout;
                },
            );

            let _ = cx.layout(nested, cx.available);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, TestHost>) {
            for &child in cx.children {
                cx.paint(child, cx.bounds);
            }
        }
    }

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.register_command(
        CommandId::from("test.widget"),
        fret_runtime::CommandMeta::new("Widget").with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let root = ui.create_node(NestedRenderRootLayoutWidget);
    ui.set_root(root);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut text = FakeTextService::default();
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let probe = app
        .global::<NestedRenderRootSnapshotProbe>()
        .copied()
        .unwrap_or_default();
    assert!(
        !probe.saw_snapshot_during_layout,
        "nested render_root calls during layout must defer window snapshot publication until the post-layout refine step"
    );
    assert!(
        app.global::<fret_runtime::WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some(),
        "post-layout refine should still publish the window command availability snapshot"
    );
}

#[test]
fn imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_input_context() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let value = app.models_mut().insert(String::new());

    let mut input_element = None;
    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "imperative-input-ctx-commit",
        |cx| {
            let input = cx.text_input(TextInputProps::new(value.clone()));
            input_element = Some(input.id);
            vec![input]
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let input_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        input_element.expect("text input element id"),
    )
    .expect("text input node");
    ui.set_focus(Some(input_node));
    ui.publish_window_runtime_snapshots(&mut app);

    let initial_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial window input context snapshot");
    assert!(initial_snapshot.focus_is_text_input);

    let replacement_root = ui.create_node(FillStack);
    ui.set_root(replacement_root);

    let stale_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("stale window input context snapshot");
    assert!(
        stale_snapshot.focus_is_text_input,
        "raw tree mutation should not silently republish window snapshots before the explicit commit boundary"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed_snapshot = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("committed window input context snapshot");
    assert!(
        !committed_snapshot.focus_is_text_input,
        "explicit window snapshot commit should refresh the authoritative input context after raw tree mutation"
    );
}

#[test]
fn imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_key_contexts() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let mut keyed_element = None;
    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "imperative-keyctx-commit",
        |cx| {
            let element = cx.text("demo ctx").key_context("demo");
            keyed_element = Some(element.id);
            vec![element]
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let keyed_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        keyed_element.expect("keyed element id"),
    )
    .expect("keyed node");
    ui.set_focus(Some(keyed_node));
    ui.publish_window_runtime_snapshots(&mut app);

    let initial_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(initial_key_contexts, vec![Arc::<str>::from("demo")]);

    let replacement_root = ui.create_node(FillStack);
    ui.set_root(replacement_root);

    let stale_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(
        stale_key_contexts,
        vec![Arc::<str>::from("demo")],
        "raw tree mutation should leave the previous key-context snapshot published until the explicit commit boundary"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert!(
        committed_key_contexts.is_empty(),
        "explicit window snapshot commit should refresh the authoritative key-context stack after raw tree mutation"
    );
}

#[test]
fn imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_command_availability() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let command = CommandId::from("test.imperative_commit_available");
    app.register_command(
        command.clone(),
        fret_runtime::CommandMeta::new("Imperative Commit Available")
            .with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let mut availability_element = None;

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "imperative-command-availability-commit",
        |cx| {
            let root = cx.container(crate::element::ContainerProps::default(), |_cx| Vec::new());
            availability_element = Some(root.id);
            cx.command_on_command_availability_for(
                root.id,
                Arc::new(|_host, _acx, requested| {
                    if requested.as_str() == "test.imperative_commit_available" {
                        return crate::widget::CommandAvailability::Available;
                    }
                    crate::widget::CommandAvailability::NotHandled
                }),
            );
            vec![root]
        },
    );

    let availability_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        availability_element.expect("availability element id"),
    )
    .expect("availability node");
    ui.set_focus(Some(availability_node));
    ui.publish_window_runtime_snapshots(&mut app);

    let initial_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(initial_availability, Some(true));

    let replacement_root = ui.create_node(FillStack);
    ui.set_root(replacement_root);

    let stale_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        stale_availability,
        Some(true),
        "raw tree mutation should not silently republish command availability before the explicit commit boundary"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert_eq!(
        committed_availability,
        Some(false),
        "explicit window snapshot commit should refresh widget command availability after raw tree mutation"
    );
}

#[test]
fn layout_all_after_imperative_tree_mutation_still_requires_explicit_window_snapshot_commit() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let command = CommandId::from("test.imperative_layout_commit_available");
    app.register_command(
        command.clone(),
        fret_runtime::CommandMeta::new("Imperative Layout Commit Available")
            .with_scope(fret_runtime::CommandScope::Widget),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();
    let value = app.models_mut().insert(String::new());
    let mut input_element = None;

    render_root_for_frame(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "imperative-layout-commit",
        |cx| {
            let input = cx.text_input(TextInputProps::new(value.clone()));
            input_element = Some(input.id);
            let root = cx
                .container(crate::element::ContainerProps::default(), |_cx| vec![input])
                .key_context("demo");
            cx.command_on_command_availability_for(
                root.id,
                Arc::new(|_host, _acx, requested| {
                    if requested.as_str() == "test.imperative_layout_commit_available" {
                        return crate::widget::CommandAvailability::Available;
                    }
                    crate::widget::CommandAvailability::NotHandled
                }),
            );
            vec![root]
        },
    );

    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let input_node = crate::declarative::mount::node_for_element_in_window_frame(
        &mut app,
        window,
        input_element.expect("text input element id"),
    )
    .expect("text input node");
    ui.set_focus(Some(input_node));
    ui.publish_window_runtime_snapshots(&mut app);

    let initial_input_context = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("initial window input context snapshot");
    let initial_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let initial_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert!(initial_input_context.focus_is_text_input);
    assert_eq!(initial_key_contexts, vec![Arc::<str>::from("demo")]);
    assert_eq!(initial_availability, Some(true));

    let replacement_root = ui.create_node(FillStack);
    ui.set_root(replacement_root);
    layout_frame(&mut ui, &mut app, &mut text, bounds);

    let stale_input_context = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("stale window input context snapshot");
    let stale_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let stale_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert!(
        stale_input_context.focus_is_text_input,
        "layout_all after raw tree mutation should not silently republish window input context before the explicit commit boundary"
    );
    assert_eq!(
        stale_key_contexts,
        vec![Arc::<str>::from("demo")],
        "layout_all after raw tree mutation should not silently republish key contexts before the explicit commit boundary"
    );
    assert_eq!(
        stale_availability,
        Some(true),
        "layout_all after raw tree mutation should not silently republish command availability before the explicit commit boundary"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed_input_context = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("committed window input context snapshot");
    let committed_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let committed_availability = app
        .global::<fret_runtime::WindowCommandActionAvailabilityService>()
        .and_then(|svc| svc.available(window, &command));
    assert!(
        !committed_input_context.focus_is_text_input,
        "explicit window snapshot commit should refresh window input context after layout completed the raw rebuild"
    );
    assert!(
        committed_key_contexts.is_empty(),
        "explicit window snapshot commit should refresh key contexts after layout completed the raw rebuild"
    );
    assert_eq!(
        committed_availability,
        Some(false),
        "explicit window snapshot commit should refresh command availability after layout completed the raw rebuild"
    );
}

#[test]
fn key_hook_can_request_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::elements::GlobalElementId> = None;
    let mut second_id: Option<crate::elements::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "key-hook-focus",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = true;

            let first = cx.pressable_with_id(props, |_cx, _st, id| {
                first_id = Some(id);
                Vec::new()
            });

            let mut props2 = crate::element::PressableProps::default();
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.pressable_with_id(props2, |_cx, _st, id| {
                second_id = Some(id);
                Vec::new()
            });

            let second_target = second.id;
            cx.key_on_key_down_for(
                first.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat || down.key != fret_core::KeyCode::ArrowRight {
                        return false;
                    }
                    host.request_focus(second_target);
                    true
                }),
            );

            vec![cx.column(crate::element::ColumnProps::default(), |_| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");

    let first_node = crate::elements::node_for_element(&mut app, window, first_id).expect("first");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second");

    let first_bounds = ui.debug_node_bounds(first_node).expect("first bounds");
    let pos = Point::new(
        Px(first_bounds.origin.x.0 + 2.0),
        Px(first_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(first_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(second_node));
}

#[test]
fn key_hook_focus_request_ignores_stale_detached_node_entry() {
    use crate::elements::NodeEntry;

    #[derive(Default)]
    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::elements::GlobalElementId> = None;
    let mut second_id: Option<crate::elements::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "key-hook-focus-stale-node-entry",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = true;

            let first = cx.pressable_with_id(props, |_cx, _st, id| {
                first_id = Some(id);
                Vec::new()
            });

            let mut props2 = crate::element::PressableProps::default();
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.pressable_with_id(props2, |_cx, _st, id| {
                second_id = Some(id);
                Vec::new()
            });

            let second_target = second.id;
            cx.key_on_key_down_for(
                first.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat || down.key != fret_core::KeyCode::ArrowRight {
                        return false;
                    }
                    host.request_focus(second_target);
                    true
                }),
            );

            vec![cx.column(crate::element::ColumnProps::default(), |_| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");
    let first_node = crate::elements::node_for_element(&mut app, window, first_id).expect("first");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second");
    let stale_detached = ui.create_node_for_element(second_id, DetachedDummy);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            second_id,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: second_id,
            },
        );
    });

    ui.set_focus(Some(first_node));
    assert_eq!(ui.focus(), Some(first_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(second_node));
}

#[test]
fn continuous_frames_lease_requests_animation_frames_while_held() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut services = FakeTextService::default();

    let mut lease: Option<ContinuousFrames> = None;

    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            lease = Some(cx.begin_continuous_frames());
            Vec::<AnyElement>::new()
        },
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected RequestAnimationFrame while beginning a continuous frames lease"
    );

    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |_cx| Vec::<AnyElement>::new(),
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected RequestAnimationFrame while continuous frames lease is held"
    );

    drop(lease.take());
    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |_cx| Vec::<AnyElement>::new(),
    );

    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "did not expect RequestAnimationFrame after dropping the last continuous frames lease"
    );
}

#[test]
fn stale_nodes_are_swept_after_gc_lag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64, 2u64], &mut ids),
    );

    let node_to_remove =
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
            runtime
                .for_window_mut(window)
                .node_entry(ids[1].1)
                .unwrap()
                .node
        });

    // Remove item 2 from the render output, but it should not be swept immediately.
    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    // Advance frames until the GC lag is exceeded, then render again to trigger the sweep.
    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep",
        |cx| build_keyed_rows(cx, &[1u64], &mut Vec::new()),
    );
    assert!(ui.debug_node_bounds(node_to_remove).is_none());
}

#[test]
fn stale_nodes_are_swept_after_gc_lag_under_view_cache_reuse() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep-view-cache",
        |cx| {
            let view_cache = crate::element::ViewCacheProps {
                cache_key: 1,
                ..Default::default()
            };
            let cached = cx.view_cache(view_cache, |cx| vec![cx.text("cached")]);

            let mut out = vec![cached];
            out.extend(build_keyed_rows(cx, &[1u64, 2u64], &mut ids));
            out
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let node_to_remove =
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
            runtime
                .for_window_mut(window)
                .node_entry(ids[1].1)
                .unwrap()
                .node
        });

    // Remove item 2 from the render output, but it should not be swept immediately.
    app.advance_frame();
    let _root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep-view-cache",
        |cx| {
            let view_cache = crate::element::ViewCacheProps {
                cache_key: 1,
                ..Default::default()
            };
            let cached = cx.view_cache(view_cache, |cx| vec![cx.text("cached")]);

            let mut out = vec![cached];
            out.extend(build_keyed_rows(cx, &[1u64], &mut Vec::new()));
            out
        },
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    // Advance frames until the GC lag is exceeded, then render again to trigger the sweep.
    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep-view-cache",
        |cx| {
            let view_cache = crate::element::ViewCacheProps {
                cache_key: 1,
                ..Default::default()
            };
            let cached = cx.view_cache(view_cache, |cx| vec![cx.text("cached")]);

            let mut out = vec![cached];
            out.extend(build_keyed_rows(cx, &[1u64], &mut Vec::new()));
            out
        },
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(ui.debug_node_bounds(node_to_remove).is_some());

    app.advance_frame();
    let _ = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp49-sweep-view-cache",
        |cx| {
            let view_cache = crate::element::ViewCacheProps {
                cache_key: 1,
                ..Default::default()
            };
            let cached = cx.view_cache(view_cache, |cx| vec![cx.text("cached")]);

            let mut out = vec![cached];
            out.extend(build_keyed_rows(cx, &[1u64], &mut Vec::new()));
            out
        },
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(ui.debug_node_bounds(node_to_remove).is_none());
}

#[test]
fn view_cache_subtree_membership_includes_nested_cache_roots() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    #[derive(Default)]
    struct CapturedIds {
        outer: Option<crate::elements::GlobalElementId>,
        inner: Option<crate::elements::GlobalElementId>,
        leaf: Option<crate::elements::GlobalElementId>,
    }

    let ids = Arc::new(std::sync::Mutex::new(CapturedIds::default()));
    let outer_runs = Arc::new(AtomicUsize::new(0));
    let inner_runs = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;
    for frame in 0..2 {
        let ids = ids.clone();
        let outer_runs = outer_runs.clone();
        let inner_runs = inner_runs.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "nested-view-cache-membership",
            move |cx| {
                let outer_props = crate::element::ViewCacheProps {
                    cache_key: 1,
                    ..Default::default()
                };

                let ids_for_outer = ids.clone();
                let outer = cx.view_cache(outer_props, move |cx| {
                    outer_runs.fetch_add(1, Ordering::Relaxed);

                    let inner_props = crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    };

                    let ids_for_inner = ids_for_outer.clone();
                    let inner_runs = inner_runs.clone();
                    let inner = cx.view_cache(inner_props, move |cx| {
                        inner_runs.fetch_add(1, Ordering::Relaxed);
                        let leaf = cx.named("leaf", |cx| cx.text("leaf"));
                        ids_for_inner.lock().unwrap().leaf = Some(leaf.id);
                        vec![leaf]
                    });

                    ids_for_outer.lock().unwrap().inner = Some(inner.id);
                    vec![inner]
                });

                ids.lock().unwrap().outer = Some(outer.id);
                vec![outer]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

        if frame == 0 {
            app.advance_frame();
        }
    }

    assert_eq!(
        outer_runs.load(Ordering::Relaxed),
        1,
        "expected the outer view-cache subtree closure to be skipped on cache hit"
    );
    assert_eq!(
        inner_runs.load(Ordering::Relaxed),
        1,
        "expected the inner view-cache subtree closure to be skipped on cache hit (by virtue of the outer root being reused)"
    );

    let (outer, inner, leaf) = {
        let ids = ids.lock().unwrap();
        (ids.outer, ids.inner, ids.leaf)
    };
    let outer = outer.expect("outer view-cache id");
    let inner = inner.expect("inner view-cache id");
    let leaf = leaf.expect("leaf element id");

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
        let window_state = runtime.for_window_mut(window);
        let outer_list = window_state
            .view_cache_elements_for_root(outer)
            .expect("expected outer membership list to be recorded")
            .to_vec();
        let inner_list = window_state
            .view_cache_elements_for_root(inner)
            .expect("expected inner membership list to be recorded")
            .to_vec();

        assert!(
            outer_list.contains(&inner),
            "expected outer membership list to include the nested cache root"
        );
        assert!(
            outer_list.contains(&leaf),
            "expected outer membership list to include nested descendants (even across cache-root boundaries)"
        );
        assert!(
            inner_list.contains(&leaf),
            "expected inner membership list to include its descendants"
        );
    });
}

#[test]
fn dismissible_root_recreates_nodes_after_layer_removal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(500.0)),
    );
    let mut text = FakeTextService::default();

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "base",
        |_cx| Vec::<AnyElement>::new(),
    );
    ui.set_root(base_root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "overlay-root",
        |cx| vec![cx.text("overlay")],
    );
    let layer = ui.push_overlay_root(overlay_root, true);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let _ = ui.remove_layer(&mut text, layer);
    assert!(ui.debug_node_bounds(overlay_root).is_none());

    app.advance_frame();
    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "overlay-root",
        |cx| vec![cx.text("overlay")],
    );
    let _layer = ui.push_overlay_root(overlay_root, true);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
}
