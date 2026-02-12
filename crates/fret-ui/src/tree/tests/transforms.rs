use super::*;

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
    let child = ui.create_node(RecordPointerPos {
        clicks: clicks.clone(),
        last_pos: last_pos.clone(),
    });
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

    assert_eq!(app.models().get_copied(&clicks), Some(1));
    assert_eq!(
        app.models().get_copied(&last_pos),
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
                    .update(&self.last_pos, |p: &mut Point| *p = *position);
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
    let leaf = ui.create_node(RecordPointerPos {
        last_pos: last_pos.clone(),
    });
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&last_pos),
        Some(Point::new(Px(5.0), Px(5.0)))
    );
}

#[test]
fn event_cx_pointer_helpers_expose_window_and_local_coordinates() {
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
            let child_bounds = Rect::new(
                Point::new(Px(10.0), Px(20.0)),
                Size::new(Px(10.0), Px(10.0)),
            );
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(
                Point::new(Px(10.0), Px(20.0)),
                Size::new(Px(10.0), Px(10.0)),
            );
            cx.paint(child, child_bounds);
        }
    }

    struct RecordPositions {
        mapped: Model<Point>,
        local: Model<Point>,
        window: Model<Point>,
    }

    impl<H: UiHost> Widget<H> for RecordPositions {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if let Event::Pointer(PointerEvent::Down { position, .. }) = event {
                let local = cx
                    .pointer_position_local(event)
                    .expect("expected local pointer position");
                let window = cx
                    .pointer_position_window(event)
                    .expect("expected window pointer position");

                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.mapped, |p: &mut Point| *p = *position);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.local, |p: &mut Point| *p = local);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.window, |p: &mut Point| *p = window);
                cx.stop_propagation();
            }
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mapped = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));
    let local = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));
    let window_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TransformRoot {
        delta: Point::new(Px(40.0), Px(0.0)),
    });
    let leaf = ui.create_node(RecordPositions {
        mapped: mapped.clone(),
        local: local.clone(),
        window: window_pos.clone(),
    });
    ui.add_child(root, leaf);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Leaf local (5, 6) -> root child bounds origin (10, 20) -> (15, 26) -> root translate(+40, 0)
    // -> window (55, 26).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(55.0), Px(26.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&mapped),
        Some(Point::new(Px(15.0), Px(26.0)))
    );
    assert_eq!(
        app.models().get_copied(&local),
        Some(Point::new(Px(5.0), Px(6.0)))
    );
    assert_eq!(
        app.models().get_copied(&window_pos),
        Some(Point::new(Px(55.0), Px(26.0)))
    );
}

#[test]
fn event_cx_wheel_delta_helpers_expose_window_and_local_deltas() {
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

    struct RecordWheelDeltas {
        local: Model<Point>,
        window: Model<Point>,
    }

    impl<H: UiHost> Widget<H> for RecordWheelDeltas {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Wheel { .. })) {
                let local = cx
                    .pointer_delta_local(event)
                    .expect("expected local wheel delta");
                let window = cx
                    .pointer_delta_window(event)
                    .expect("expected window wheel delta");
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.local, |p: &mut Point| *p = local);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.window, |p: &mut Point| *p = window);
                cx.stop_propagation();
            }
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let local = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));
    let window_delta = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(ScaleRoot { scale: 2.0 });
    let leaf = ui.create_node(RecordWheelDeltas {
        local: local.clone(),
        window: window_delta.clone(),
    });
    ui.add_child(root, leaf);
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
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(10.0), Px(0.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&local),
        Some(Point::new(Px(5.0), Px(0.0)))
    );
    assert_eq!(
        app.models().get_copied(&window_delta),
        Some(Point::new(Px(10.0), Px(0.0)))
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
            // A singular scale makes the transform non-invertible; ADR 0082 requires treating
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
