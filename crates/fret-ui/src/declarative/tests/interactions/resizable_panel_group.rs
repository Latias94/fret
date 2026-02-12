use super::*;

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
            let mut props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
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

    let fractions_now = app.models().get_cloned(&model).unwrap_or_default();
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(128.0), Px(20.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(128.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models().get_cloned(&model).unwrap_or_default();
    assert!(
        v.first().copied().unwrap_or(0.0) > 0.33,
        "expected left panel to grow, got {v:?}"
    );
}

#[test]
fn resizable_panel_group_registers_viewport_roots_for_panels() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "resizable-panel-group-flow-islands",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.hover_region(
                                crate::element::HoverRegionProps::default(),
                                |cx, _hovered| vec![cx.text("left")],
                            )]
                        },
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("right")],
                    ),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let group = ui.children(root)[0];
    let panel_a = ui.children(group)[0];
    let panel_b = ui.children(group)[1];
    let hover = ui.children(panel_a)[0];
    let hover_text = ui.children(hover)[0];
    let panel_b_text = ui.children(panel_b)[0];

    let viewport_root_nodes: Vec<_> = ui
        .viewport_roots()
        .iter()
        .map(|(node, _bounds)| *node)
        .collect();
    assert_eq!(viewport_root_nodes.len(), 2);
    assert!(viewport_root_nodes.contains(&panel_a));
    assert!(viewport_root_nodes.contains(&panel_b));

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(panel_a).is_some());
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(hover_text).is_some());
    assert!(engine.layout_id_for_node(panel_b).is_some());
    assert!(engine.layout_id_for_node(panel_b_text).is_some());
    ui.put_layout_engine(engine);
}
