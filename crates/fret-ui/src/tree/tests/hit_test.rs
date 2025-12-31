use super::*;

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
