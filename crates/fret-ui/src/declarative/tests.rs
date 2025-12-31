use std::sync::Arc;

use super::render_root;
use crate::UiHost;
use crate::action::{ActivateReason, DismissReason};
use crate::element::{AnyElement, CrossAlign, Length, MainAlign, TextInputProps};
use crate::elements::{ContinuousFrames, ElementCx};
use crate::test_host::TestHost;
use crate::tree::UiTree;
use crate::widget::Invalidation;
use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{
    AppWindowId, Color, Modifiers, MouseButton, MouseButtons, NodeId, Point, Px, Rect, Scene,
    SceneOp, Size, TextConstraints, TextMetrics, TextService, TextStyle, Transform2D,
};
use fret_runtime::{CommandId, Effect};

#[derive(Default)]
struct FakeTextService {}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _text: &str,
        _style: TextStyle,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeTextService {
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

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

#[derive(Default)]
struct FillStack;

impl<H: UiHost> Widget<H> for FillStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout(child, cx.available);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
    }
}

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

            cx.key_on_key_down_for(
                input.id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat {
                        return false;
                    }
                    if down.key != fret_core::KeyCode::ArrowDown {
                        return false;
                    }
                    let _ = host.models_mut().update(invoked, |v| *v += 1);
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
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
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

    assert_eq!(app.models().get(invoked).copied().unwrap_or_default(), 1);
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
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
            runtime.prepare_window_for_frame(window, app.frame_id());
            let st = runtime.for_window_mut(window);
            st.node_entry(ids[1].1).unwrap().node
        });

    // Remove item 2 from the render output, but it should not be swept immediately.
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
fn declarative_text_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text",
        |cx| vec![cx.text("Hello declarative")],
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    // Root is a host widget, so text is in a descendant; ensure at least one Text node carries
    // the label payload.
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some("Hello declarative")),
        "expected a Text semantics node with label"
    );
}

#[test]
fn declarative_text_input_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-label",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model);
            props.a11y_label = Some("Search".into());
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::TextField
                && n.label.as_deref() == Some("Search")),
        "expected a TextField semantics node with label"
    );
}

#[test]
fn declarative_text_area_updates_model_on_text_input() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-text-input",
        |cx| {
            let mut props = crate::element::TextAreaProps::new(model);
            props.min_height = Px(80.0);
            vec![cx.text_area(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable text area");
    ui.set_focus(Some(focusable));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("hello\nworld".to_string()),
    );
    assert_eq!(
        app.models().get(model).map(|s| s.as_str()),
        Some("hello\nworld")
    );
}

#[test]
fn declarative_pointer_region_can_capture_and_receive_move_up() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-capture-move-up",
        |cx| {
            let counter_down = counter;
            let counter_move = counter;
            let counter_up = counter;

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(counter_down, |v| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_move = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      _cx: crate::action::ActionCx,
                      _mv: crate::action::PointerMoveCx| {
                    let _ = host
                        .models_mut()
                        .update(counter_move, |v| *v = v.saturating_add(10));
                    true
                },
            );

            let on_up = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      up: crate::action::PointerUpCx| {
                    if up.button == MouseButton::Left {
                        host.release_pointer_capture();
                    }
                    let _ = host
                        .models_mut()
                        .update(counter_up, |v| *v = v.saturating_add(100));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");

    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
    );
    let outside = Point::new(Px(region_bounds.origin.x.0 + 250.0), inside.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: outside,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(counter).copied(), Some(111));
}

#[test]
fn declarative_pointer_region_hook_can_request_focus_for_other_element() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-can-request-focus-other-element",
        |cx| {
            vec![cx.semantics(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Slider,
                    label: Some(Arc::from("focus-target")),
                    ..Default::default()
                },
                |cx| {
                    let target = cx.root_id();

                    vec![cx.pointer_region(
                        crate::element::PointerRegionProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: crate::element::Length::Fill,
                                    height: crate::element::Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            enabled: true,
                        },
                        |cx| {
                            cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                host.request_focus(target);
                                true
                            }));
                            vec![]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics_node = ui.children(root)[0];
    let pointer_node = ui.children(semantics_node)[0];
    let pointer_bounds = ui.debug_node_bounds(pointer_node).expect("pointer bounds");
    let position = Point::new(
        Px(pointer_bounds.origin.x.0 + 2.0),
        Px(pointer_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(ui.focus(), Some(semantics_node));
}

#[test]
fn declarative_resizable_panel_group_updates_model_on_drag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "resizable-panel-group-drag",
        |cx| {
            let mut props =
                crate::element::ResizablePanelGroupProps::new(fret_core::Axis::Horizontal, model);
            props.min_px = vec![Px(10.0)];
            props.chrome = crate::ResizablePanelGroupStyle {
                hit_thickness: Px(10.0),
                ..Default::default()
            };
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let fractions_now = app.models().get(model).cloned().unwrap_or_default();
    let layout = crate::resizable_panel_group::compute_resizable_panel_group_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        fractions_now,
        Px(0.0),
        Px(10.0),
        &[Px(10.0)],
    );
    let down_x = layout.handle_centers.first().copied().unwrap_or(0.0);
    let down = Point::new(Px(down_x), Px(20.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(128.0), Px(20.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(128.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    let v = app.models().get(model).cloned().unwrap_or_default();
    assert!(
        v.first().copied().unwrap_or(0.0) > 0.33,
        "expected left panel to grow, got {v:?}"
    );
}

#[test]
fn pressable_on_activate_hook_runs_on_pointer_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-pointer",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Pointer);
                        let _ = host.models_mut().update(activated, |v| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get(activated).copied(), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(activated).copied(), Some(true));
}

#[test]
fn pressable_on_hover_change_hook_runs_on_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-hover-change-hook",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host.models_mut().update(hovered, |v| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get(hovered).copied(), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(hovered).copied(), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(hovered).copied(), Some(false));
}

#[test]
fn pressable_on_activate_hook_runs_on_keyboard_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Keyboard);
                        let _ = host.models_mut().update(activated, |v| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get(activated).copied(), Some(false));

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );

    assert_eq!(app.models().get(activated).copied(), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_escape() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-escape",
        |cx| {
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                assert_eq!(reason, DismissReason::Escape);
                let _ = host.models_mut().update(dismissed, |v| *v = true);
            }));

            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _| {
                    vec![cx.text("child")]
                }),
            ]
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Focus a descendant in the overlay so Escape bubbles up to the dismissible layer.
    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get(dismissed).copied(), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_outside_press_observer() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-outside-press",
        |cx| {
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, reason| {
                assert_eq!(reason, DismissReason::OutsidePress);
                let _ = host.models_mut().update(dismissed, |v| *v = true);
            }));
            Vec::new()
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer down hits the base root (overlay has no children and is hit-test transparent),
    // so outside-press observer dispatch runs for the overlay root.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(dismissed).copied(), Some(true));
}

#[test]
fn roving_flex_arrow_keys_move_focus_and_update_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(Option::<Arc<str>>::Some(Arc::from("a")));
    let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, true, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                let values = values.clone();
                cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
                    let Some(value) = values.get(idx).cloned() else {
                        return;
                    };
                    let next = Some(value);
                    let _ = host.models_mut().update(model, |v| *v = next);
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(c),
        "expected ArrowDown to skip disabled child"
    );
    assert_eq!(
        app.models().get(model).and_then(|v| v.as_deref()),
        Some("c")
    );
}

#[test]
fn roving_flex_typeahead_hook_can_choose_target_index() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex-typeahead-hook",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_typeahead(Arc::new(
                    |_host, _cx, it| {
                        if it.input == 'c' { Some(2) } else { None }
                    },
                ));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::KeyC,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(c));
}

#[test]
fn pressable_semantics_checked_is_exposed() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-checked",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    a11y: crate::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::Checkbox),
                        label: Some(Arc::from("checked")),
                        checked: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("x")],
            )]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::Checkbox && n.label.as_deref() == Some("checked")
        })
        .expect("expected checkbox semantics node");

    assert_eq!(node.flags.checked, Some(true));
    assert!(node.actions.invoke, "expected checkbox to be invokable");
}

#[track_caller]
fn build_keyed_rows(
    cx: &mut ElementCx<'_, TestHost>,
    items: &[u64],
    ids: &mut Vec<(u64, crate::elements::GlobalElementId)>,
) -> Vec<crate::element::AnyElement> {
    let mut out = Vec::new();
    for &item in items {
        let el = cx.keyed(item, |cx| cx.text("row"));
        ids.push((item, el.id));
        out.push(el);
    }
    out
}

#[test]
fn virtual_list_computes_visible_range_after_first_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    fn build_list(
        cx: &mut ElementCx<'_, TestHost>,
        list_element_id: &mut Option<crate::elements::GlobalElementId>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> crate::element::AnyElement {
        let list = cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                    .collect()
            },
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: no viewport height is known yet (it is written during layout), so the list
    // renders with an empty visible range.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let list_node = ui.children(root)[0];
    assert_eq!(ui.children(list_node).len(), 0);
    let viewport_h = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.unwrap(),
        crate::element::VirtualListState::default,
        |s| s.viewport_h,
    );
    assert_eq!(viewport_h, Px(50.0));

    // Frame 1: the list has recorded its viewport height during layout, so the authoring layer
    // can compute a visible range and mount only the visible children.
    app.advance_frame();
    let prev_list_element_id = list_element_id;
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::ElementInstance::VirtualList(props) = props.expect("list instance exists").instance
    else {
        panic!("expected VirtualList instance");
    };
    assert_eq!(
        props
            .visible_items
            .iter()
            .map(|item| item.index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

#[test]
fn virtual_list_scroll_to_item_keeps_target_visible() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-to",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect()
                },
            );
            list_element_id = Some(list.id);
            vec![list]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: request scroll-to on a row below the viewport.
    let target = 6usize; // row_top=60, viewport=30 => needs offset ~= 40..60
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest))
    );
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-to",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect()
                },
            );
            list_element_id = Some(list.id);
            vec![list]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(scroll_handle.deferred_scroll_to_item().is_none());

    let state = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.expect("list element id"),
        crate::element::VirtualListState::default,
        |s| s.clone(),
    );
    assert_eq!(state.viewport_h, Px(30.0));
    assert!((state.metrics.offset_for_index(target).0 - 60.0).abs() < 0.01);
    assert!(
        (state.offset_y.0 - 40.0).abs() < 0.01,
        "state_offset_y={:?}",
        state.offset_y
    );

    assert!(
        (scroll_handle.offset().y.0 - 40.0).abs() < 0.01,
        "offset_y={:?}",
        scroll_handle.offset().y
    );
}

#[test]
fn virtual_list_scroll_to_item_uses_measured_row_heights() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    fn row_with_height<H: UiHost>(cx: &mut ElementCx<'_, H>, height: Px) -> AnyElement {
        let mut style = crate::element::LayoutStyle::default();
        style.size.height = crate::element::Length::Px(height);
        cx.container(
            crate::element::ContainerProps {
                layout: style,
                ..Default::default()
            },
            |_| Vec::new(),
        )
    }

    fn build_list<H: UiHost>(
        cx: &mut ElementCx<'_, H>,
        list_element_id: &mut Option<crate::elements::GlobalElementId>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let list = cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.keyed(item.key, |cx| {
                            if item.index == 0 {
                                row_with_height(cx, Px(100.0))
                            } else {
                                row_with_height(cx, Px(10.0))
                            }
                        })
                    })
                    .collect()
            },
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: ensure row 0 gets mounted and measured.
    let prev_list_element_id = list_element_id;
    list_element_id = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );
    app.advance_frame();

    // Frame 2: scroll to item 1; should account for the measured height of item 0.
    scroll_handle.scroll_to_item(1, crate::scroll::ScrollStrategy::Start);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((1, crate::scroll::ScrollStrategy::Start))
    );
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        (scroll_handle.offset().y.0 - 100.0).abs() < 0.01,
        "offset_y={:?}",
        scroll_handle.offset().y
    );
}

#[test]
fn virtual_list_paint_clips_each_visible_row() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    fn build_list<H: UiHost>(
        cx: &mut ElementCx<'_, H>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                    .collect()
            },
        )
    }

    // Frame 0: record viewport height (no visible children yet).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount visible children based on the recorded viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    // One clip for the list viewport + one clip per visible row child.
    let pushes = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
        .count();
    assert_eq!(pushes, 1 + 5);
}

#[test]
fn virtual_list_keyed_reuses_node_ids_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let mut items: Vec<u64> = vec![10, 20, 30];
    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

    fn build_list<H: UiHost>(
        cx: &mut ElementCx<'_, H>,
        items: &[u64],
        mut ids: Option<&mut Vec<(u64, crate::elements::GlobalElementId)>>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let items_revision = items
            .iter()
            .fold(0u64, |acc, k| acc.wrapping_mul(1_000_003).wrapping_add(*k));
        let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        options.items_revision = items_revision;

        cx.virtual_list_keyed(
            items.len(),
            options,
            scroll_handle,
            |i| items[i],
            |cx, i| {
                let row = cx.text("row");
                if let Some(ids) = ids.as_deref_mut() {
                    ids.push((items[i], row.id));
                }
                row
            },
        )
    }

    // Frame 0: record viewport height (no visible children yet).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-keyed",
        |cx| vec![build_list(cx, &items, None, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    let mut prev: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> =
        std::collections::HashMap::new();

    for pass in 0..2 {
        ids.clear();
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-keyed",
            |cx| vec![build_list(cx, &items, Some(&mut ids), &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let cur: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> = app
            .with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
                runtime.prepare_window_for_frame(window, app.frame_id());
                let st = runtime.for_window_mut(window);
                ids.iter()
                    .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                    .collect()
            });

        if pass == 1 {
            for item in [10u64, 20u64, 30u64] {
                let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                assert_eq!(prev_id, cur_id, "element id should be stable");
                assert_eq!(prev_node, cur_node, "node id should be stable");
            }
        }

        prev = cur;
        items.reverse();
        app.advance_frame();
    }
}

#[test]
fn hover_region_reports_hovered_even_when_child_is_pressable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementCx<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.hover_region(
            crate::element::HoverRegionProps::default(),
            |cx, hovered| {
                let trigger = cx
                    .pressable(crate::element::PressableProps::default(), |cx, _state| {
                        vec![cx.text("trigger")]
                    });

                let mut children = vec![trigger];
                if hovered {
                    children.push(cx.text("hovered"));
                }
                children
            },
        )]
    }

    // Frame 0: not hovered yet, so only the trigger is present.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 1);
    let trigger_node = ui.children(hover_region_node)[0];
    let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

    let pos = fret_core::Point::new(
        Px(trigger_bounds.origin.x.0 + 2.0),
        Px(trigger_bounds.origin.y.0 + 2.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    // Frame 1: hover_region should now observe hovered=true even though the hit node is a Pressable.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 2);
}

#[test]
fn row_justify_center_and_align_end_positions_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(20.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "row-align",
        |cx| {
            let mut props = crate::element::RowProps {
                gap: Px(5.0),
                justify: MainAlign::Center,
                align: CrossAlign::End,
                ..Default::default()
            };
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            vec![cx.row(props, |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let row_node = ui.children(root)[0];
    let children = ui.children(row_node);
    assert_eq!(children.len(), 3);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
    let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");

    // Each text measures to 10x10. With gap=5 and width=100:
    // content_w = 3*10 + 2*5 = 40; remaining=60; center => start_offset=30.
    assert!((b0.origin.x.0 - 30.0).abs() < 0.01, "x0={:?}", b0.origin.x);
    assert!((b1.origin.x.0 - 45.0).abs() < 0.01, "x1={:?}", b1.origin.x);
    assert!((b2.origin.x.0 - 60.0).abs() < 0.01, "x2={:?}", b2.origin.x);

    // align-end with row height 20 => y = 0 + (20-10)=10.
    assert!((b0.origin.y.0 - 10.0).abs() < 0.01, "y0={:?}", b0.origin.y);
    assert!((b1.origin.y.0 - 10.0).abs() < 0.01, "y1={:?}", b1.origin.y);
    assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
}

#[test]
fn pressable_keyboard_activation_dispatches_click_command() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let cmd = CommandId::new("test.pressable.click");

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let cmd = cmd.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), cmd.clone());
                    }));
                    vec![cx.text("ok")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));
    assert_eq!(ui.focus(), Some(pressable_node));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );
    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command, .. } if *command == cmd)),
        "expected click command effect"
    );
}

#[test]
fn pressable_disabled_is_not_focusable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-disabled-focus",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: false,
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("disabled")],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(ui.first_focusable_descendant(root), None);
}

#[test]
fn image_paints_image_op() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let img = fret_core::ImageId::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-image",
        |cx| {
            let mut p = crate::element::ImageProps::new(img);
            p.layout.size.width = crate::element::Length::Px(Px(160.0));
            p.layout.size.height = crate::element::Length::Px(Px(80.0));
            vec![cx.image_props(p)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Image { image, .. } if *image == img)),
        "expected an Image op for the declarative image element"
    );
}

#[test]
fn overflow_clip_pushes_clip_rect_for_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let pushes = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
        .count();
    let pops = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PopClip))
        .count();

    assert!(
        pushes >= 1,
        "expected container overflow clip to push a clip rect"
    );
    assert!(
        pops >= 1,
        "expected container overflow clip to pop a clip rect"
    );
}

#[test]
fn overflow_clip_with_corner_radii_pushes_rounded_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip-rounded",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            props.corner_radii = fret_core::Corners::all(Px(8.0));
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::PushClipRRect { .. })),
        "expected container overflow clip + corner radii to push a rounded clip rect"
    );
}

#[test]
fn overflow_visible_does_not_push_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-visible",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("child")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        !scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. }
        )),
        "expected no clip ops by default"
    );
}

#[test]
fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui.debug_node_bounds(column_node).expect("column bounds");

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_bounds(column_node)
        .expect("column bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_thumb_drag_updates_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            stack_layout.size.height = crate::element::Length::Px(Px(20.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0),
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: None,
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let column_node = ui.children(scroll_node)[0];
    let before = ui.debug_node_bounds(column_node).expect("column bounds");

    // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let down_pos = fret_core::Point::new(
        Px(scrollbar_bounds.origin.x.0 + 1.0),
        Px(scrollbar_bounds.origin.y.0 + 2.0),
    );
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().y
    );

    ui.invalidate(scroll_node, Invalidation::Layout);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_bounds(column_node)
        .expect("column bounds after drag");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after thumb drag: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn fill_respects_max_width_constraint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-max-width",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            max_width: Some(Px(100.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let rect = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(rect.size.width, Px(100.0));
}

#[test]
fn flex_child_margin_affects_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-flex-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(15.0));
}

#[test]
fn flex_child_auto_margins_center_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-mx-auto",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));
                    a.layout.margin.left = crate::element::MarginEdge::Auto;
                    a.layout.margin.right = crate::element::MarginEdge::Auto;
                    vec![cx.container(a, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert_eq!(flex_bounds.size.width, Px(100.0));
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 1);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");

    assert_eq!(a_bounds.origin.x, Px(45.0));
}

#[test]
fn flex_child_negative_margin_shifts_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-negative-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(-5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(5.0));
}

#[test]
fn container_absolute_inset_positions_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-stack-absolute",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(0.0));
                    badge.layout.inset.right = Some(Px(0.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(container_bounds.size.width, Px(100.0));
    assert_eq!(container_bounds.size.height, Px(80.0));

    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(90.0));
    assert_eq!(badge_bounds.origin.y, Px(0.0));
}

#[test]
fn container_absolute_negative_inset_offsets_outside_parent() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-stack-absolute-negative-inset",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(-5.0));
                    badge.layout.inset.left = Some(Px(-6.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(-6.0));
    assert_eq!(badge_bounds.origin.y, Px(-5.0));
}

#[test]
fn grid_places_children_in_columns() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-grid",
        |cx| {
            vec![cx.grid(
                crate::element::GridProps {
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l.size.height = crate::element::Length::Fill;
                        l
                    },
                    cols: 2,
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Fill;
                    a.layout.size.height = crate::element::Length::Fill;

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Fill;
                    b.layout.size.height = crate::element::Length::Fill;

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let grid_node = ui.children(root)[0];
    let children = ui.children(grid_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(100.0));
    assert_eq!(a_bounds.size.width, Px(100.0));
    assert_eq!(b_bounds.size.width, Px(100.0));
}

#[test]
fn focus_ring_is_focus_visible_only() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(64.0), Px(32.0)),
    );
    let mut text = FakeTextService::default();

    let ring = crate::element::RingStyle {
        placement: crate::element::RingPlacement::Outset,
        width: Px(2.0),
        offset: Px(2.0),
        color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        offset_color: None,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-focus-visible",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            height: crate::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    focus_ring: Some(ring),
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let pressable_node = ui.children(root)[0];

    // Focus the pressable via pointer: should *not* show focus-visible ring.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: fret_core::Point::new(Px(4.0), Px(4.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected pressable to be focused after pointer down"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert_eq!(
        scene.ops().len(),
        0,
        "expected no ring ops for mouse-focused control"
    );

    // Enable focus-visible via keyboard navigation: ring should appear for focused control.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected focus to remain on pressable after keydown"
    );
    assert!(
        crate::focus_visible::is_focus_visible(&mut app, Some(window)),
        "expected focus-visible to be enabled after Tab keydown"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(
        !scene.ops().is_empty(),
        "expected ring ops for keyboard navigation focus-visible"
    );
}

#[test]
fn declarative_elements_can_observe_models_for_invalidation() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root_name = "mvp50-observe-model";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        root_name,
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(model, Invalidation::Layout);
                let v = cx.app.models().get(model).copied().unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let stats0 = ui.debug_stats();
    assert!(
        stats0.layout_nodes_visited > 0,
        "expected layout traversal: visited={} performed={}",
        stats0.layout_nodes_visited,
        stats0.layout_nodes_performed
    );
    let performed0 = stats0.layout_nodes_performed;
    assert!(performed0 > 0, "expected initial layout work");

    // A second layout pass with no changes and no re-render should perform no node layouts.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed1 = ui.debug_stats().layout_nodes_performed;
    assert_eq!(performed1, 0, "expected no layout work when clean");

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    ui.propagate_model_changes(&mut app, &changed);

    // The observed model change should invalidate the declarative host, enabling layout work.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed2 = ui.debug_stats().layout_nodes_performed;
    assert!(performed2 > 0, "expected model change to trigger relayout");
}

#[test]
fn model_observation_requires_rerender_after_frame_advance() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-observe-contract-frame-advance",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(model, Invalidation::Layout);
                let v = cx.app.models().get(model).copied().unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Advance the frame but intentionally skip the render pass.
    app.advance_frame();

    // The first model change still invalidates because UiTree retains the previous observation
    // index until the next layout/paint pass records observations again.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected invalidation from the last recorded observation index"
    );

    // Layout now runs on the advanced frame. Without a new render pass, the declarative layer
    // has no per-frame observation data to re-register, so the observation index is cleared.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // A second model change no longer invalidates: this encodes the ADR 0028 execution contract
    // that `render_root(...)` must be called each frame before layout/paint.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        !ui.propagate_model_changes(&mut app, &changed),
        "expected no invalidation without re-rendering after a frame advance"
    );
}

#[test]
fn container_applies_padding_and_paints_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-container",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    background: Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
    assert_eq!(text_bounds.origin.x, Px(4.0));
    assert_eq!(text_bounds.origin.y, Px(6.0));
    assert_eq!(text_bounds.size.width, Px(10.0));
    assert_eq!(text_bounds.size.height, Px(10.0));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 2);
    match scene.ops()[0] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected quad op"),
    }
}

#[test]
fn container_paints_shadow_before_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-shadow",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    shadow: Some(crate::element::ShadowStyle {
                        color: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.5,
                        },
                        offset_x: Px(2.0),
                        offset_y: Px(3.0),
                        spread: Px(1.0),
                        softness: 0,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    }),
                    corner_radii: fret_core::Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);

    let shadow_bounds = match scene.ops()[0] {
        SceneOp::Quad { rect, .. } => rect,
        _ => panic!("expected shadow quad first"),
    };
    match scene.ops()[1] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected background quad second"),
    }

    assert!(shadow_bounds.origin.x.0 > container_bounds.origin.x.0);
    assert!(shadow_bounds.origin.y.0 > container_bounds.origin.y.0);
    assert!(shadow_bounds.size.width.0 > container_bounds.size.width.0);
    assert!(shadow_bounds.size.height.0 > container_bounds.size.height.0);
}

#[test]
fn pressable_dispatches_click_command_when_released_over_self() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let command = CommandId::from("test.click");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-pressable",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    ..Default::default()
                },
                |cx, _state| {
                    let command = command.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), command.clone());
                    }));
                    vec![cx.container(
                        crate::element::ContainerProps {
                            padding: fret_core::Edges::all(Px(4.0)),
                            ..Default::default()
                        },
                        |cx| vec![cx.text("hi")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 10.0),
        Px(pressable_bounds.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command: c, .. } if c.as_str() == "test.click")),
        "expected Effect::Command(test.click), got {effects:?}"
    );

    // Sanity: move outside should clear hover state for future interactions.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(200.0), Px(200.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
        }),
    );
}

#[test]
fn flex_defaults_to_fit_content_under_constraints() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-fit",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(5.0),
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("a"), cx.text("b")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");

    // FakeTextService measures each text to 10x10. With gap=5 and padding (4,6):
    // inner_w = 10 + 5 + 10 = 25, outer_w = 25 + 8 = 33
    // inner_h = 10, outer_h = 10 + 12 = 22
    assert!(
        (flex_bounds.size.width.0 - 33.0).abs() < 0.01,
        "w={:?}",
        flex_bounds.size.width
    );
    assert!(
        (flex_bounds.size.height.0 - 22.0).abs() < 0.01,
        "h={:?}",
        flex_bounds.size.height
    );
}
