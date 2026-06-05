use super::*;

#[test]
fn line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-box-zoom-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let updated = state
        .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
        .expect("plot state should be readable");
    let view = updated
        .1
        .expect("declarative box zoom should leave an explicit view bounds");
    assert!(
        !updated.0,
        "declarative box zoom should switch/keep plot view in controlled mode"
    );
    assert!(
        view.x_min > 0.9 && view.x_max < 2.6,
        "right-button box zoom should narrow the declarative X range, got {view:?}"
    );
    assert!(
        view.y_min > 0.8 && view.y_max < 3.0,
        "right-button box zoom should narrow the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_query_drag_updates_query_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-query-drag",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let query = state
        .read_ref(&app, |state| state.query)
        .expect("plot state should be readable")
        .expect("declarative query drag should write a query rect");
    assert!(
        query.x_min > 0.9 && query.x_max < 2.6,
        "Alt+left query drag should map the selected X range into data space, got {query:?}"
    );
    assert!(
        query.y_min > 0.8 && query.y_max < 3.1,
        "Alt+left query drag should map the selected Y range into data space, got {query:?}"
    );
}

#[test]
fn line_plot_panel_query_drag_updates_output_query_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-query-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let query = output_snapshot
        .query
        .expect("declarative query drag should publish query output");
    assert!(
        query.x_min > 0.9 && query.x_max < 2.6,
        "query output should include the selected X data range, got {query:?}"
    );
    assert!(
        query.y_min > 0.8 && query.y_max < 3.1,
        "query output should include the selected Y data range, got {query:?}"
    );
    assert_eq!(
        output_snapshot.view_bounds,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "query output should keep reporting the current declarative view bounds"
    );
}

#[test]
fn line_plot_panel_paints_query_selection_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-query-selection",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut active_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
    let active_rects = line_plot_selection_rects(&active_scene);
    assert_eq!(
        active_rects.len(),
        1,
        "active declarative query drag should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut persisted_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut persisted_scene, 1.0);
    let persisted_rects = line_plot_selection_rects(&persisted_scene);
    assert_eq!(
        persisted_rects.len(),
        1,
        "persisted declarative query state should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(persisted_rects[0], 100.0, 50.0, 100.0, 70.0);
}

#[test]
fn line_plot_panel_paints_box_zoom_selection_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-box-selection",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut active_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
    let active_rects = line_plot_selection_rects(&active_scene);
    assert_eq!(
        active_rects.len(),
        1,
        "active declarative box zoom should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut released_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut released_scene, 1.0);
    assert!(
        line_plot_selection_rects(&released_scene).is_empty(),
        "box zoom selection rectangle should clear after applying the view change"
    );
}

#[test]
fn line_plot_panel_paints_query_selection_tooltip_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-query-tooltip",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    services.prepared_text.clear();

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("query\nx=["),
        "declarative query drag should paint a query selection tooltip, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("y=["),
        "declarative query selection tooltip should include y-range text, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    let state = app.models_mut().insert(plot_state);
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-box-tooltip",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    services.prepared_text.clear();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("zoom\nx=["),
        "declarative box zoom should paint a zoom selection tooltip, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("y=["),
        "declarative box zoom tooltip should include y-range text, got {prepared_text:?}"
    );
}
