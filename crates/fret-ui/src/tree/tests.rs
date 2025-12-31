use super::*;
use fret_core::{
    Color, Corners, DrawOrder, Edges, Px, Scene, SceneOp, TextConstraints, TextMetrics,
    TextService, TextStyle, TextWrap,
};
use fret_runtime::{BindingV1, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Model};
use slotmap::KeyData;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct TestStack;

impl<H: UiHost> Widget<H> for TestStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Default)]
struct FakeUiServices;

impl TextService for FakeUiServices {
    fn prepare(
        &mut self,
        _text: &str,
        _style: TextStyle,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                baseline: fret_core::Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeUiServices {
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

impl fret_core::SvgService for FakeUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

struct ObservingWidget {
    model: Model<u32>,
}

struct PaintObservingWidget {
    model: Model<u32>,
}

impl<H: UiHost> Widget<H> for PaintObservingWidget {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
    }
}

struct HitTestObservingWidget {
    model: Model<u32>,
}

impl<H: UiHost> Widget<H> for HitTestObservingWidget {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::HitTest);
    }
}

impl<H: UiHost> Widget<H> for ObservingWidget {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.model, Invalidation::Layout);
        let _ = cx.services.text().prepare(
            "x",
            TextStyle {
                font: fret_core::FontId::default(),
                size: fret_core::Px(12.0),
                ..Default::default()
            },
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            },
        );
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
        let _ = cx.scene;
    }
}

struct RoundedClipWidget;

impl<H: UiHost> Widget<H> for RoundedClipWidget {
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        true
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
        Some(Corners::all(Px(20.0)))
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn model_change_invalidates_observers() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(ObservingWidget { model });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    if let Some(n) = ui.nodes.get_mut(node) {
        n.invalidation.clear();
    }

    let _ = model.update(&mut app, |v, _cx| {
        *v += 1;
    });
    let changed = app.take_changed_models();
    assert!(changed.contains(&model.id()));

    ui.propagate_model_changes(&mut app, &changed);
    let n = ui.nodes.get(node).unwrap();
    assert!(n.invalidation.layout);
    assert!(n.invalidation.paint);
}

#[test]
fn model_change_invalidates_observers_across_windows() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = ui_a.create_node(ObservingWidget { model });
    ui_a.set_root(node_a);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_a = Scene::default();
    ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
    ui_a.nodes.get_mut(node_a).unwrap().invalidation.clear();

    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = ui_b.create_node(ObservingWidget { model });
    ui_b.set_root(node_b);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_b = Scene::default();
    ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
    ui_b.nodes.get_mut(node_b).unwrap().invalidation.clear();

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(changed.contains(&model.id()));

    ui_a.propagate_model_changes(&mut app, &changed);
    ui_b.propagate_model_changes(&mut app, &changed);

    let na = ui_a.nodes.get(node_a).unwrap();
    assert!(na.invalidation.layout);
    assert!(na.invalidation.paint);

    let nb = ui_b.nodes.get(node_b).unwrap();
    assert!(nb.invalidation.layout);
    assert!(nb.invalidation.paint);
}

#[test]
fn paint_observation_only_invalidates_paint() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(PaintObservingWidget { model });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.nodes.get_mut(node).unwrap().invalidation.clear();

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(ui.propagate_model_changes(&mut app, &changed));

    let n = ui.nodes.get(node).unwrap();
    assert!(!n.invalidation.layout);
    assert!(n.invalidation.paint);
    assert!(!n.invalidation.hit_test);
}

#[test]
fn hit_test_observation_escalates_to_layout_and_paint() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(HitTestObservingWidget { model });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.nodes.get_mut(node).unwrap().invalidation.clear();

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(ui.propagate_model_changes(&mut app, &changed));

    let n = ui.nodes.get(node).unwrap();
    assert!(n.invalidation.hit_test);
    assert!(n.invalidation.layout);
    assert!(n.invalidation.paint);
}

#[test]
fn model_change_requests_redraw_for_each_invalidated_window() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = ui_a.create_node(PaintObservingWidget { model });
    ui_a.set_root(node_a);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_a = Scene::default();
    ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
    ui_a.nodes.get_mut(node_a).unwrap().invalidation.clear();

    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = ui_b.create_node(PaintObservingWidget { model });
    ui_b.set_root(node_b);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_b = Scene::default();
    ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
    ui_b.nodes.get_mut(node_b).unwrap().invalidation.clear();

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();

    ui_a.propagate_model_changes(&mut app, &changed);
    ui_b.propagate_model_changes(&mut app, &changed);

    let effects = app.flush_effects();
    let redraws: std::collections::HashSet<AppWindowId> = effects
        .into_iter()
        .filter_map(|e| match e {
            Effect::Redraw(w) => Some(w),
            _ => None,
        })
        .collect();
    let expected: std::collections::HashSet<AppWindowId> =
        [window_a, window_b].into_iter().collect();

    assert_eq!(redraws, expected);
}

#[test]
fn paint_all_sets_ime_allowed_for_focused_text_input() {
    #[derive(Default)]
    struct FakeTextInput;

    impl<H: UiHost> Widget<H> for FakeTextInput {
        fn is_text_input(&self) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(FakeTextInput);
    ui.set_root(node);
    ui.set_focus(Some(node));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ImeAllow { enabled: true, .. }))
    );
}

#[test]
fn hit_test_respects_rounded_overflow_clip() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(RoundedClipWidget);
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Inside bounds, but outside the rounded corner arc.
    assert_eq!(ui.hit_test(node, Point::new(Px(1.0), Px(1.0))), None);

    // Inside the rounded rectangle.
    assert_eq!(
        ui.hit_test(node, Point::new(Px(25.0), Px(25.0))),
        Some(node)
    );
}

#[test]
fn hit_test_respects_rounded_overflow_clip_under_render_transform() {
    struct RoundedClipTranslatedWidget {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for RoundedClipTranslatedWidget {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn clips_hit_test(&self, _bounds: Rect) -> bool {
            true
        }

        fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
            Some(Corners::all(Px(20.0)))
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(RoundedClipTranslatedWidget {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Inside the visual bounds, but outside the rounded corner arc (after inverse mapping).
    assert_eq!(ui.hit_test(node, Point::new(Px(41.0), Px(1.0))), None);

    // Inside the rounded rectangle (after inverse mapping).
    assert_eq!(
        ui.hit_test(node, Point::new(Px(65.0), Px(25.0))),
        Some(node)
    );
}

struct CountingPaintWidget {
    paints: Arc<AtomicUsize>,
}

impl<H: UiHost> Widget<H> for CountingPaintWidget {
    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paints.fetch_add(1, Ordering::SeqCst);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Color::TRANSPARENT,
            border: Edges::default(),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::default(),
        });
    }
}

#[test]
fn paint_cache_replays_subtree_ops_when_clean() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 1);

    ui.invalidate(node, Invalidation::Paint);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);
    assert_eq!(scene.ops_len(), 1);

    let bounds2 = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(200.0), fret_core::Px(100.0)),
    );
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds2, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 3);
    assert_eq!(scene.ops_len(), 1);
}

struct TransparentOverlay;

impl<H: UiHost> Widget<H> for TransparentOverlay {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }
}

struct ClickCounter {
    clicks: Model<u32>,
}

impl<H: UiHost> Widget<H> for ClickCounter {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(
            event,
            Event::Pointer(fret_core::PointerEvent::Up {
                button: fret_core::MouseButton::Left,
                ..
            })
        ) {
            let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn hit_test_can_make_overlay_pointer_transparent() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter { clicks });
    ui.set_root(base);

    let overlay = ui.create_node(TransparentOverlay);
    let _ = ui.push_overlay_root_ex(overlay, false, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let value = app.models().get(clicks).copied().unwrap_or(0);
    assert_eq!(value, 1);
}

#[test]
fn layer_hit_testable_flag_can_make_overlay_pointer_transparent() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter { clicks });
    ui.set_root(base);

    let overlay = ui.create_node(ClickCounter { clicks });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_hit_testable(layer, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let value = app.models().get(clicks).copied().unwrap_or(0);
    assert_eq!(value, 1);
}

#[test]
fn overlay_render_transform_affects_hit_testing_and_event_coordinates() {
    struct TransformOverlayRoot {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for TransformOverlayRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct RecordOverlayClicks {
        clicks: Model<u32>,
        last_pos: Model<Point>,
    }

    impl<H: UiHost> Widget<H> for RecordOverlayClicks {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            match event {
                Event::Pointer(PointerEvent::Down { position, .. }) => {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(self.last_pos, |p| *p = *position);
                    cx.stop_propagation();
                }
                Event::Pointer(PointerEvent::Up { .. }) => {
                    let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
                    cx.stop_propagation();
                }
                _ => {}
            }
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let underlay_clicks = app.models_mut().insert(0u32);
    let overlay_clicks = app.models_mut().insert(0u32);
    let overlay_last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: underlay_clicks,
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(TransformOverlayRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let overlay_leaf = ui.create_node(RecordOverlayClicks {
        clicks: overlay_clicks,
        last_pos: overlay_last_pos,
    });
    ui.add_child(overlay_root, overlay_leaf);
    let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click inside the overlay leaf (after overlay transform).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(45.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(45.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(overlay_clicks).copied(), Some(1));
    assert_eq!(
        app.models().get(overlay_last_pos).copied(),
        Some(Point::new(Px(5.0), Px(5.0)))
    );
    assert_eq!(
        app.models().get(underlay_clicks).copied(),
        Some(0),
        "expected underlay to not receive clicks when overlay leaf handles them"
    );

    // Click outside the overlay leaf should reach the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(5.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(5.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(underlay_clicks).copied(), Some(1));
}

#[test]
fn render_transform_affects_hit_testing_and_pointer_event_coordinates() {
    struct TransformRoot {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for TransformRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct RecordPointerPos {
        clicks: Model<u32>,
        last_pos: Model<Point>,
    }

    impl<H: UiHost> Widget<H> for RecordPointerPos {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            match event {
                Event::Pointer(PointerEvent::Down { position, .. }) => {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(self.last_pos, |p| *p = *position);
                    cx.stop_propagation();
                }
                Event::Pointer(PointerEvent::Up { .. }) => {
                    let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
                    cx.stop_propagation();
                }
                _ => {}
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);
    let last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TransformRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let child = ui.create_node(RecordPointerPos { clicks, last_pos });
    ui.add_child(root, child);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(45.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(45.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(app.models().get(clicks).copied(), Some(1));
    assert_eq!(
        app.models().get(last_pos).copied(),
        Some(Point::new(Px(5.0), Px(5.0)))
    );
}

#[test]
fn nested_render_transforms_compose_for_pointer_event_coordinates() {
    struct TranslateRoot {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for TranslateRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, cx.bounds.size);
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, cx.bounds.size);
            cx.paint(child, child_bounds);
        }
    }

    struct ScaleRoot {
        scale: f32,
    }

    impl<H: UiHost> Widget<H> for ScaleRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::scale_uniform(self.scale))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct RecordPointerPos {
        last_pos: Model<Point>,
    }

    impl<H: UiHost> Widget<H> for RecordPointerPos {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if let Event::Pointer(PointerEvent::Down { position, .. }) = event {
                let _ = cx
                    .app
                    .models_mut()
                    .update(self.last_pos, |p| *p = *position);
                cx.stop_propagation();
            }
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TranslateRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let scale = ui.create_node(ScaleRoot { scale: 2.0 });
    let leaf = ui.create_node(RecordPointerPos { last_pos });
    ui.add_child(root, scale);
    ui.add_child(scale, leaf);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Leaf local (5,5) -> Scale(2x) -> (10,10) -> Translate(+40,0) -> (50,10).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(50.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(
        app.models().get(last_pos).copied(),
        Some(Point::new(Px(5.0), Px(5.0)))
    );
}

#[test]
fn visual_bounds_for_element_includes_ancestor_render_transform() {
    struct TransformRoot {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for TransformRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct ElementLeaf;

    impl<H: UiHost> Widget<H> for ElementLeaf {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let element = crate::elements::GlobalElementId(123);

    let root = ui.create_node(TransformRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let leaf = ui.create_node_for_element(element, ElementLeaf);
    ui.add_child(root, leaf);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // `visual_bounds_for_element` is defined as a cross-frame query: the "last frame" value is
    // made visible after `prepare_window_for_frame` advances the window element state.
    app.advance_frame();

    let visual = crate::elements::visual_bounds_for_element(&mut app, window, element)
        .expect("expected visual bounds to be recorded during paint");
    assert_eq!(visual.origin, Point::new(Px(40.0), Px(0.0)));
    assert_eq!(visual.size, Size::new(Px(10.0), Px(10.0)));
}

#[test]
fn non_invertible_render_transform_is_ignored_for_paint_and_visual_bounds() {
    struct NonInvertibleRoot {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for NonInvertibleRoot {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            let t = Transform2D::translation(self.delta);
            // A singular scale makes the transform non-invertible; ADR 0083 requires treating
            // such transforms as `None` to keep paint/hit-testing consistent.
            let s = Transform2D::scale_uniform(0.0);
            Some(t * s)
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct ElementLeaf;

    impl<H: UiHost> Widget<H> for ElementLeaf {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let element = crate::elements::GlobalElementId(456);

    let root = ui.create_node(NonInvertibleRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let leaf = ui.create_node_for_element(element, ElementLeaf);
    ui.add_child(root, leaf);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        !scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::PushTransform { .. })),
        "non-invertible render transforms must not emit scene transform ops"
    );

    // `visual_bounds_for_element` is defined as a cross-frame query: the "last frame" value is
    // made visible after `prepare_window_for_frame` advances the window element state.
    app.advance_frame();

    let visual = crate::elements::visual_bounds_for_element(&mut app, window, element)
        .expect("expected visual bounds to be recorded during paint");
    assert_eq!(visual.origin, Point::new(Px(0.0), Px(0.0)));
    assert_eq!(visual.size, Size::new(Px(10.0), Px(10.0)));
}

#[test]
fn outside_press_observer_must_not_capture_pointer_or_break_click_through() {
    struct CaptureOnPointerDownOutside;

    impl<H: UiHost> Widget<H> for CaptureOnPointerDownOutside {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter { clicks });
    ui.set_root(base);

    let overlay = ui.create_node(CaptureOnPointerDownOutside);
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let value = app.models().get(clicks).copied().unwrap_or(0);
    assert_eq!(
        value, 1,
        "expected click-through dispatch to reach underlay"
    );
    assert_eq!(
        ui.captured(),
        None,
        "observer pass must not capture pointer"
    );
}

#[test]
fn outside_press_observer_dispatch_sets_input_context_phase() {
    struct RecordObserverPhase {
        phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverPhase {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(self.phase, |v| *v = cx.input_ctx.dispatch_phase);
            }
        }
    }

    struct RecordNormalPhase {
        phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
    }

    impl<H: UiHost> Widget<H> for RecordNormalPhase {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(self.phase, |v| *v = cx.input_ctx.dispatch_phase);
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let observer_phase = app
        .models_mut()
        .insert(fret_runtime::InputDispatchPhase::Normal);
    let normal_phase = app
        .models_mut()
        .insert(fret_runtime::InputDispatchPhase::Observer);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(RecordNormalPhase {
        phase: normal_phase,
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverPhase {
        phase: observer_phase,
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    assert_eq!(
        app.models().get(observer_phase).copied(),
        Some(fret_runtime::InputDispatchPhase::Observer),
        "observer pass should tag InputContext as Observer"
    );
    assert_eq!(
        app.models().get(normal_phase).copied(),
        Some(fret_runtime::InputDispatchPhase::Normal),
        "normal hit-tested dispatch should tag InputContext as Normal"
    );
}

#[test]
fn paint_cache_replays_ops_when_node_translates() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds_a = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    let bounds_b = Rect::new(
        Point::new(fret_core::Px(20.0), fret_core::Px(15.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 3);

    match (scene.ops()[0], scene.ops()[1], scene.ops()[2]) {
        (
            SceneOp::PushTransform { transform },
            SceneOp::Quad { rect, .. },
            SceneOp::PopTransform,
        ) => {
            assert_eq!(transform.tx, bounds_b.origin.x.0 - bounds_a.origin.x.0);
            assert_eq!(transform.ty, bounds_b.origin.y.0 - bounds_a.origin.y.0);
            assert_eq!(rect, bounds_a);
        }
        _ => panic!("expected push-transform + quad + pop-transform ops"),
    }
}

#[test]
fn semantics_snapshot_includes_visible_roots_and_barrier() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let base = ui.create_node(TestStack);
    ui.set_root(base);
    let base_child = ui.create_node(TestStack);
    ui.add_child(base, base_child);

    let overlay_root = ui.create_node(TestStack);
    ui.push_overlay_root(overlay_root, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert_eq!(snap.roots.len(), 2);
    assert_eq!(snap.barrier_root, Some(overlay_root));
    assert_eq!(
        snap.nodes.iter().find(|n| n.id == base).unwrap().role,
        SemanticsRole::Window
    );
    assert_ne!(
        snap.nodes
            .iter()
            .find(|n| n.id == overlay_root)
            .unwrap()
            .role,
        SemanticsRole::Window
    );
    assert!(snap.nodes.iter().any(|n| n.id == base));
    assert!(snap.nodes.iter().any(|n| n.id == base_child));
    assert!(snap.nodes.iter().any(|n| n.id == overlay_root));
}

#[test]
fn modal_barrier_clears_focus_and_capture_in_underlay() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.request_focus(cx.node);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let underlay = ui.create_node(CaptureOnDown);
    ui.add_child(root, underlay);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );
    assert_eq!(ui.focus(), Some(underlay));
    assert_eq!(ui.captured(), Some(underlay));

    let overlay_root = ui.create_node(TestStack);
    let _layer = ui.push_overlay_root(overlay_root, true);

    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);
}

#[test]
fn focus_traversal_includes_roots_above_modal_barrier() {
    #[derive(Default)]
    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let underlay_focusable = ui.create_node(Focusable);
    ui.add_child(base_root, underlay_focusable);
    ui.set_root(base_root);

    let modal_root = ui.create_node(TestStack);
    let modal_focusable = ui.create_node(Focusable);
    ui.add_child(modal_root, modal_focusable);
    ui.push_overlay_root(modal_root, true);

    // Simulate a nested "portal" overlay that lives above the modal barrier (e.g. combobox popover
    // inside a dialog).
    let popup_root = ui.create_node(TestStack);
    let popup_focusable = ui.create_node(Focusable);
    ui.add_child(popup_root, popup_focusable);
    ui.push_overlay_root(popup_root, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Under a modal barrier, traversal must not reach underlay focusables.
    ui.set_focus(Some(modal_focusable));
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(popup_focusable));

    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(modal_focusable));

    // Reverse direction should also wrap within the active layers set.
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.previous"));
    assert_eq!(ui.focus(), Some(popup_focusable));
}

#[test]
fn focus_traversal_prefers_topmost_overlay_root() {
    #[derive(Default)]
    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let base_focusable = ui.create_node(Focusable);
    ui.add_child(base_root, base_focusable);
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_focusable = ui.create_node(Focusable);
    ui.add_child(overlay_root, overlay_focusable);
    ui.push_overlay_root(overlay_root, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.set_focus(Some(base_focusable));
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(overlay_focusable));

    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(base_focusable));
}

#[test]
fn tab_focus_next_runs_when_text_input_not_composing() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "expected focus traversal command effect"
    );
}

#[test]
fn tab_focus_next_is_suppressed_during_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "did not expect focus traversal command effect during IME composition"
    );
}

#[test]
fn reserved_shortcuts_are_suppressed_during_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("test.tab".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Enter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.numpad_enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "NumpadEnter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.space".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Space".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.escape".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Escape".into(),
                    },
                },
            ],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    for key in [
        KeyCode::Tab,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
        KeyCode::Escape,
    ] {
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
    }

    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Command { .. })),
        "did not expect any shortcut commands during IME composition"
    );
}

#[test]
fn remove_layer_uninstalls_overlay_and_removes_subtree() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_child = ui.create_node(TestStack);
    ui.add_child(overlay_root, overlay_child);
    let layer = ui.push_overlay_root(overlay_root, true);

    // Pretend an overlay widget captured focus/pointer.
    ui.focus = Some(overlay_child);
    ui.captured = Some(overlay_child);

    let mut services = FakeUiServices;
    let removed_root = ui.remove_layer(&mut services, layer);

    assert_eq!(removed_root, Some(overlay_root));
    assert!(ui.layers.get(layer).is_none());
    assert!(!ui.layer_order.contains(&layer));
    assert!(!ui.root_to_layer.contains_key(&overlay_root));

    assert!(ui.nodes.get(overlay_root).is_none());
    assert!(ui.nodes.get(overlay_child).is_none());
    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);
}

#[test]
fn event_cx_bounds_tracks_translated_nodes() {
    struct BoundsProbe {
        out: Model<Point>,
    }

    impl BoundsProbe {
        fn new(out: Model<Point>) -> Self {
            Self { out }
        }
    }

    impl<H: UiHost> Widget<H> for BoundsProbe {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if !matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                return;
            }
            let origin = cx.bounds.origin;
            let _ = cx.app.models_mut().update(self.out, |v| *v = origin);
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let out = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let probe = ui.create_node(BoundsProbe::new(out));
    ui.add_child(root, probe);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let size = Size::new(Px(120.0), Px(40.0));

    ui.layout_in(
        &mut app,
        &mut services,
        root,
        Rect::new(Point::new(Px(0.0), Px(0.0)), size),
        1.0,
    );

    // Layout again with the same size but translated origin: the tree uses a fast-path that
    // translates node bounds without re-running widget.layout for the subtree.
    ui.layout_in(
        &mut app,
        &mut services,
        root,
        Rect::new(Point::new(Px(0.0), Px(100.0)), size),
        1.0,
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(110.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let origin = app.models().get(out).copied().unwrap_or_default();
    assert_eq!(origin, Point::new(Px(0.0), Px(100.0)));
}
