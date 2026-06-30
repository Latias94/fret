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

    let base = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(TransparentOverlay);
    let _ = ui.push_overlay_root(overlay, false);

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
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let value = app.models().get_copied(&clicks).unwrap_or(0);
    assert_eq!(value, 1);
}

#[test]
fn layer_hit_testable_flag_can_make_overlay_pointer_transparent() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    let layer = ui.push_overlay_root(overlay, false);
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
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let value = app.models().get_copied(&clicks).unwrap_or(0);
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
                        .update(&self.last_pos, |p: &mut Point| *p = *position);
                    cx.stop_propagation();
                }
                Event::Pointer(PointerEvent::Up { .. }) => {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&self.clicks, |v: &mut u32| *v += 1);
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
        clicks: underlay_clicks.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(TransformOverlayRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let overlay_leaf = ui.create_node(RecordOverlayClicks {
        clicks: overlay_clicks.clone(),
        last_pos: overlay_last_pos.clone(),
    });
    ui.add_child(overlay_root, overlay_leaf);
    let _layer = ui.push_overlay_root(overlay_root, false);

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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(45.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&overlay_clicks), Some(1));
    assert_eq!(
        app.models().get_copied(&overlay_last_pos),
        Some(Point::new(Px(5.0), Px(5.0)))
    );
    assert_eq!(
        app.models().get_copied(&underlay_clicks),
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(5.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicks), Some(1));
}

#[test]
fn modal_barrier_blocks_underlay_pointer_events_even_when_modal_root_is_pointer_transparent() {
    struct CountDown {
        down: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.down, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    struct PointerTransparentModalRoot {
        down: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for PointerTransparentModalRoot {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.down, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let modal_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CountDown {
        down: underlay_down.clone(),
    });
    ui.set_root(base);

    let modal_root = ui.create_node(PointerTransparentModalRoot {
        down: modal_down.clone(),
    });
    ui.push_overlay_root(modal_root, true);

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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&underlay_down),
        Some(0),
        "expected modal barrier to keep underlay inert"
    );
    assert_eq!(
        app.models().get_copied(&modal_down),
        Some(1),
        "expected modal root to receive the event even when hit-test is pointer transparent"
    );
}

#[test]
fn hit_test_respects_nested_rounded_overflow_clips_under_rotation() {
    struct RootLayout;

    impl<H: UiHost> Widget<H> for RootLayout {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds =
                Rect::new(Point::new(Px(40.0), Px(0.0)), Size::new(Px(40.0), Px(40.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }
    }

    struct OuterClip {
        center: Point,
        transform: Transform2D,
    }

    impl<H: UiHost> Widget<H> for OuterClip {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(self.transform)
        }

        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn clips_hit_test(&self, _bounds: Rect) -> bool {
            true
        }

        fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
            Some(Corners::all(Px(18.0)))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(
                Point::new(Px(self.center.x.0 - 10.0), Px(self.center.y.0 - 10.0)),
                Size::new(Px(20.0), Px(20.0)),
            );
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }
    }

    struct InnerClip;

    impl<H: UiHost> Widget<H> for InnerClip {
        fn clips_hit_test(&self, _bounds: Rect) -> bool {
            true
        }

        fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
            Some(Corners::all(Px(10.0)))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(RootLayout);
    ui.set_root(root);

    let center = Point::new(Px(60.0), Px(20.0));
    let outer_t = Transform2D::rotation_about_degrees(45.0, center);

    let outer = ui.create_node(OuterClip {
        center,
        transform: outer_t,
    });
    ui.add_child(root, outer);

    let inner = ui.create_node(InnerClip);
    ui.add_child(outer, inner);

    let leaf = ui.create_node(TestStack);
    ui.add_child(inner, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let inner_origin = Point::new(Px(50.0), Px(10.0));
    let outside_local = Point::new(Px(inner_origin.x.0 + 1.0), Px(inner_origin.y.0 + 1.0));
    let outside_window = outer_t.apply_point(outside_local);

    assert_eq!(
        ui.hit_test(root, outside_window),
        None,
        "expected nested rounded clip to reject corner point under rotation"
    );

    let inside_local = Point::new(Px(inner_origin.x.0 + 15.0), Px(inner_origin.y.0 + 15.0));
    let inside_window = outer_t.apply_point(inside_local);
    assert_eq!(ui.hit_test(root, inside_window), Some(leaf));
}

#[test]
fn modal_barrier_fallback_delivers_transformed_event_coordinates() {
    struct RecordDownPos {
        down: Model<u32>,
        last_pos: Model<Point>,
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for RecordDownPos {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.down, |v: &mut u32| *v += 1);
                if let Event::Pointer(PointerEvent::Down { position, .. }) = event {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&self.last_pos, |p: &mut Point| *p = *position);
                }
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    struct CountDown {
        down: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.down, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let modal_down = app.models_mut().insert(0u32);
    let modal_last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CountDown {
        down: underlay_down.clone(),
    });
    ui.set_root(base);

    let modal_root = ui.create_node(RecordDownPos {
        down: modal_down.clone(),
        last_pos: modal_last_pos.clone(),
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    ui.push_overlay_root(modal_root, true);

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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_down), Some(0));
    assert_eq!(app.models().get_copied(&modal_down), Some(1));
    assert_eq!(
        app.models().get_copied(&modal_last_pos),
        Some(Point::new(Px(5.0), Px(5.0))),
        "expected barrier fallback to map pointer coordinates through render_transform"
    );
}

#[test]
fn hit_test_layers_cached_reuses_path_and_respects_layer_order() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();

    let base_root = ui.create_node(TestStack);
    let base_leaf = ui.create_node(TestStack);
    ui.set_root(base_root);
    ui.set_children(base_root, vec![base_leaf]);

    let overlay_root = ui.create_node(TestStack);
    let overlay_leaf = ui.create_node(TestStack);
    ui.set_children(overlay_root, vec![overlay_leaf]);

    let base_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let overlay_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(50.0), Px(50.0)));
    for id in [base_root, base_leaf] {
        ui.nodes[id].bounds = base_bounds;
    }
    for id in [overlay_root, overlay_leaf] {
        ui.nodes[id].bounds = overlay_bounds;
    }

    let layers = [overlay_root, base_root];

    assert_eq!(
        ui.hit_test_layers_cached(&layers, Point::new(Px(75.0), Px(75.0))),
        Some(base_leaf),
    );
    assert!(
        ui.test_hit_test_path_cache_has_entry_for_layer(base_root),
        "cached routing path should be owned by the window hit-test routing cache state"
    );
    assert_eq!(
        ui.hit_test_layers_cached(&layers, Point::new(Px(76.0), Px(76.0))),
        Some(base_leaf),
        "expected cached-path hit-test to stay stable within the same subtree"
    );
    assert_eq!(
        ui.hit_test_layers_cached(&layers, Point::new(Px(25.0), Px(25.0))),
        Some(overlay_leaf),
        "overlay root should win when it hits before the base root"
    );
}

#[test]
fn hit_test_path_routing_cache_clear_api_drops_entry() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![leaf]);

    for id in [root, leaf] {
        ui.nodes[id].bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
    }

    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(10.0), Px(10.0))),
        Some(leaf)
    );
    assert!(ui.test_hit_test_path_cache_has_entry_for_layer(root));

    ui.clear_hit_test_path_cache();
    assert!(!ui.test_hit_test_path_cache_has_entry_for_layer(root));
}

#[test]
fn suspended_hit_test_path_routing_cache_query_does_not_replace_primary_entry() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();

    let base_root = ui.create_node(TestStack);
    let base_leaf = ui.create_node(TestStack);
    ui.set_root(base_root);
    ui.set_children(base_root, vec![base_leaf]);

    let overlay_root = ui.create_node(TestStack);
    let overlay_leaf = ui.create_node(TestStack);
    ui.set_children(overlay_root, vec![overlay_leaf]);

    for id in [base_root, base_leaf] {
        ui.nodes[id].bounds =
            Rect::new(Point::new(Px(60.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    }
    for id in [overlay_root, overlay_leaf] {
        ui.nodes[id].bounds =
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    }

    assert_eq!(
        ui.hit_test_layers_cached(&[base_root], Point::new(Px(65.0), Px(5.0))),
        Some(base_leaf)
    );
    assert!(ui.test_hit_test_path_cache_has_entry_for_layer(base_root));

    ui.begin_suspended_hit_test_path_cache_query();
    assert_eq!(
        ui.hit_test_layers_cached(&[overlay_root], Point::new(Px(5.0), Px(5.0))),
        Some(overlay_leaf)
    );
    ui.end_suspended_hit_test_path_cache_query();

    assert!(
        ui.test_hit_test_path_cache_has_entry_for_layer(base_root),
        "secondary/suspended hit-test queries must not replace the primary routing cache entry"
    );
    assert!(
        !ui.test_hit_test_path_cache_has_entry_for_layer(overlay_root),
        "secondary/suspended hit-test queries must not publish their own path cache entry"
    );
}

#[test]
fn hit_test_layers_cached_rejects_stale_path_when_higher_z_sibling_moves_under_pointer() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let lower = ui.create_node(TestStack);
    let higher = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![lower, higher]);

    ui.nodes[root].bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.nodes[lower].bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[higher].bounds =
        Rect::new(Point::new(Px(60.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));

    let pointer = Point::new(Px(10.0), Px(10.0));
    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(lower),
        "prime a cached path to the lower-z child"
    );

    ui.nodes[higher].bounds = ui.nodes[lower].bounds;
    let stats_before_rehit = ui.debug_stats();
    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(higher),
        "a higher-z sibling that moved under the pointer must invalidate the stale lower-child path"
    );
    let stats_after_rehit = ui.debug_stats();
    assert!(
        stats_after_rehit.hit_test_path_cache_misses
            > stats_before_rehit.hit_test_path_cache_misses,
        "expected the stale child path to miss before fallback hit testing accepts the moved sibling"
    );

    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(higher),
        "the fallback result should refresh the cached path to the moved higher-z sibling"
    );
    let stats_after_cached_reuse = ui.debug_stats();
    assert!(
        stats_after_cached_reuse.hit_test_path_cache_hits
            > stats_after_rehit.hit_test_path_cache_hits,
        "expected the refreshed higher-z sibling path to be cache-reusable"
    );
}

#[test]
fn hit_test_layers_cached_ignores_non_hit_testable_overlapping_higher_z_siblings() {
    struct NonHitTestableOverlay;

    impl<H: UiHost> Widget<H> for NonHitTestableOverlay {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let target = ui.create_node(TestStack);
    let overlay = ui.create_node(NonHitTestableOverlay);
    ui.set_root(root);
    ui.set_children(root, vec![target, overlay]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.nodes[root].bounds = bounds;
    ui.nodes[target].bounds = bounds;
    ui.nodes[overlay].bounds = bounds;

    let pointer = Point::new(Px(10.0), Px(10.0));
    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(target),
        "prime a cached path to the hit-testable lower-z target"
    );

    let stats_before_reuse = ui.debug_stats();
    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(11.0), Px(11.0))),
        Some(target),
        "a higher-z sibling whose widget rejects hit testing must not force the path cache to miss"
    );
    let stats_after_reuse = ui.debug_stats();
    assert!(
        stats_after_reuse.hit_test_path_cache_hits > stats_before_reuse.hit_test_path_cache_hits,
        "expected the cached path to be reusable when overlapping higher-z siblings are not hit-testable"
    );
}

#[test]
fn hit_test_layers_cached_checks_transformed_higher_z_siblings_before_reuse() {
    struct TranslatedSibling {
        delta: Point,
    }

    impl<H: UiHost> Widget<H> for TranslatedSibling {
        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            Some(Transform2D::translation(self.delta))
        }
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let target = ui.create_node(TestStack);
    let sibling = ui.create_node(TranslatedSibling {
        delta: Point::new(Px(0.0), Px(0.0)),
    });
    ui.set_root(root);
    ui.set_children(root, vec![target, sibling]);

    ui.nodes[root].bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.nodes[target].bounds =
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[sibling].bounds =
        Rect::new(Point::new(Px(60.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));

    let pointer = Point::new(Px(10.0), Px(10.0));
    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(target),
        "prime a cached path to the lower-z target"
    );

    if let Some(widget) = ui.nodes[sibling].widget.as_mut() {
        *widget = Box::new(TranslatedSibling {
            delta: Point::new(Px(-60.0), Px(0.0)),
        });
    }
    let stats_before_rehit = ui.debug_stats();
    assert_eq!(
        ui.hit_test_layers_cached(&[root], pointer),
        Some(sibling),
        "a transformed higher-z sibling that visually covers the pointer must still win"
    );
    let stats_after_rehit = ui.debug_stats();
    assert!(
        stats_after_rehit.hit_test_path_cache_misses
            > stats_before_rehit.hit_test_path_cache_misses,
        "expected cached-path reuse to miss before fallback accepts the transformed sibling"
    );
}

#[test]
fn pointer_move_dispatch_rejects_stale_path_when_higher_z_sibling_moves_under_pointer() {
    struct CountPointerMove {
        moves: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for CountPointerMove {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                self.moves.fetch_add(1, Ordering::SeqCst);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let lower_moves = Arc::new(AtomicUsize::new(0));
    let higher_moves = Arc::new(AtomicUsize::new(0));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let lower = ui.create_node(CountPointerMove {
        moves: lower_moves.clone(),
    });
    let higher = ui.create_node(CountPointerMove {
        moves: higher_moves.clone(),
    });
    ui.set_root(root);
    ui.set_children(root, vec![lower, higher]);

    ui.nodes[root].bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.nodes[lower].bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[higher].bounds =
        Rect::new(Point::new(Px(60.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));

    let pointer = Point::new(Px(10.0), Px(10.0));
    let mut services = FakeUiServices;
    let first_move = Event::Pointer(PointerEvent::Move {
        position: pointer,
        buttons: fret_core::MouseButtons::default(),
        modifiers: fret_core::Modifiers::default(),
        pointer_id: fret_core::PointerId(0),
        pointer_type: fret_core::PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &first_move);
    assert_eq!(
        lower_moves.load(Ordering::SeqCst),
        1,
        "prime real pointer-move dispatch through the lower-z child"
    );
    assert_eq!(higher_moves.load(Ordering::SeqCst), 0);

    ui.nodes[higher].bounds = ui.nodes[lower].bounds;
    let stats_before_rehit = ui.debug_stats();
    ui.dispatch_event(&mut app, &mut services, &first_move);

    assert_eq!(
        lower_moves.load(Ordering::SeqCst),
        1,
        "the stale lower-child cached path must not receive the second pointer move"
    );
    assert_eq!(
        higher_moves.load(Ordering::SeqCst),
        1,
        "fallback hit testing should route the second pointer move to the moved higher-z sibling"
    );
    let stats_after_rehit = ui.debug_stats();
    assert!(
        stats_after_rehit.hit_test_path_cache_misses
            > stats_before_rehit.hit_test_path_cache_misses,
        "expected dispatch to reject the stale lower-child path before routing to the moved sibling"
    );

    ui.dispatch_event(&mut app, &mut services, &first_move);
    assert_eq!(higher_moves.load(Ordering::SeqCst), 2);
    let stats_after_cached_reuse = ui.debug_stats();
    assert!(
        stats_after_cached_reuse.hit_test_path_cache_hits
            > stats_after_rehit.hit_test_path_cache_hits,
        "expected the refreshed higher-z dispatch path to be cache-reusable"
    );
}

#[test]
fn hit_test_works_with_view_cache_root_and_prepaint_reuse_under_render_transform() {
    struct TransformedOverlayRoot;

    impl<H: UiHost> Widget<H> for TransformedOverlayRoot {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            let center = Point::new(Px(5.0), Px(5.0));
            let rotate = Transform2D::rotation_about_degrees(90.0, center);
            let translate = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            Some(translate * rotate)
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

    struct CountNormalDown {
        normal: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountNormalDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.normal, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let overlay_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let underlay = ui.create_node(CountNormalDown {
        normal: underlay_down.clone(),
    });
    ui.set_root(underlay);

    let overlay_root = ui.create_node(TransformedOverlayRoot);
    ui.set_node_view_cache_flags(overlay_root, true, false, false);
    let overlay_leaf = ui.create_node(CountNormalDown {
        normal: overlay_down.clone(),
    });
    ui.add_child(overlay_root, overlay_leaf);
    ui.push_overlay_root(overlay_root, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    // Frame 0: establish prepaint interaction recording.
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 1: reuse the cached interaction range for the overlay cache root.
    app.advance_frame();
    // Force a non-stable layout pass so the test exercises interaction-cache replay.
    //
    // The layout engine can legitimately skip work on a completely stable frame, which would
    // bypass prepaint recording/replay and make `interaction_cache_hits` remain 0.
    ui.invalidate(underlay, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let stats = ui.debug_stats();
    assert!(
        stats.interaction_cache_hits >= 1,
        "expected prepaint interaction cache to hit for clean view-cache roots (stats={stats:?})"
    );

    // This window-space point maps to local (6, 5) inside the overlay leaf after rotation+translation.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(45.0), Px(6.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&overlay_down), Some(1));
    assert_eq!(app.models().get_copied(&underlay_down), Some(0));

    // Clicking outside the overlay leaf should hit the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&overlay_down), Some(1));
    assert_eq!(app.models().get_copied(&underlay_down), Some(1));
}
