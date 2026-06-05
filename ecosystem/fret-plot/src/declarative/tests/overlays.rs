use super::*;

#[test]
fn line_plot_panel_paints_reference_lines_on_declarative_path() {
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
    plot_state.overlays.inf_lines_x.push(InfLineX::new(2.0));
    plot_state
        .overlays
        .inf_lines_y
        .push(InfLineY::new(1.0, YAxis::Left));
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
        "plot-declarative-reference-lines",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let reference_lines = line_plot_reference_line_rects(&scene);
    assert!(
        reference_lines.iter().any(|rect| {
            (rect.origin.x.0 - 169.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 1.0).abs() < 0.01
                && (rect.size.height.0 - 146.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned X reference line, got {reference_lines:?}"
    );
    assert!(
        reference_lines.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned Y reference line, got {reference_lines:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_lines_on_declarative_path() {
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
    plot_state
        .overlays
        .drag_lines_x
        .push(DragLineX::new(10, 2.0));
    plot_state
        .overlays
        .drag_lines_y
        .push(DragLineY::new(11, 1.0, YAxis::Left));
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
        "plot-declarative-draggable-lines",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_lines = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_lines.iter().any(|rect| {
            (rect.origin.x.0 - 169.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 1.0).abs() < 0.01
                && (rect.size.height.0 - 146.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable X line, got {draggable_lines:?}"
    );
    assert!(
        draggable_lines.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable Y line, got {draggable_lines:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_point_and_rect_on_declarative_path() {
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
    plot_state.overlays.drag_points.push(DragPoint::new(
        20,
        DataPoint { x: 2.0, y: 1.0 },
        YAxis::Left,
    ));
    plot_state.overlays.drag_rects.push(DragRect::new(
        21,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 1.0,
            y_max: 3.0,
        },
        YAxis::Left,
    ));
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
        "plot-declarative-draggable-point-rect",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_shapes = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 165.0).abs() < 0.01
                && (rect.origin.y.0 - 114.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable point, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 98.0).abs() < 0.01
                && (rect.origin.y.0 - 45.0).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable rect, got {draggable_shapes:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path() {
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
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state
        .overlays
        .drag_lines_y
        .push(DragLineY::new(50, 100.0, YAxis::Right));
    plot_state.overlays.drag_points.push(DragPoint::new(
        51,
        DataPoint { x: 2.0, y: 50.0 },
        YAxis::Right,
    ));
    plot_state.overlays.drag_rects.push(DragRect::new(
        52,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 25.0,
            y_max: 75.0,
        },
        YAxis::Right,
    ));
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
        "plot-declarative-right-axis-draggable-shapes",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_shapes = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable Y line, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 165.0).abs() < 0.01
                && (rect.origin.y.0 - 77.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable point, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 98.0).abs() < 0.01
                && (rect.origin.y.0 - 45.0).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable rect, got {draggable_shapes:?}"
    );
}

#[test]
fn line_plot_panel_paints_plot_text_overlay_on_declarative_path() {
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
    plot_state.overlays.text.push(
        PlotText::new(2.0, 1.0, YAxis::Left, "threshold note")
            .background(Color::from_srgb_hex_rgb(0x19_33_4c))
            .padding(Px(4.0))
            .offset(Point::new(Px(4.0), Px(-6.0))),
    );
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
        "plot-declarative-plot-text-overlay",
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
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("threshold note"),
        "declarative line plot should prepare caller-owned PlotText overlay text, got {prepared_text:?}"
    );

    let text_backgrounds = line_plot_reference_line_rects(&scene);
    assert!(
        text_backgrounds.iter().any(|rect| {
            (rect.origin.x.0 - 173.0).abs() < 0.01
                && (rect.origin.y.0 - 112.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible PlotText background, got {text_backgrounds:?}"
    );
}

#[test]
fn line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path() {
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
    plot_state
        .overlays
        .tags_x
        .push(TagX::new(2.0).label("X Gate").show_value(false));
    plot_state.overlays.tags_y.push(
        TagY::new(1.0, YAxis::Left)
            .label("Y Gate")
            .show_value(false),
    );
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
        "plot-declarative-tags",
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
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("X Gate"),
        "declarative line plot should prepare caller-owned TagX text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Y Gate"),
        "declarative line plot should prepare caller-owned TagY text, got {prepared_text:?}"
    );

    let tag_rects = line_plot_reference_line_rects(&scene);
    assert!(
        tag_rects.iter().any(|rect| {
            (rect.origin.x.0 - 168.0).abs() < 0.01
                && (rect.origin.y.0 - 146.0).abs() < 0.01
                && (rect.size.width.0 - 2.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible TagX marker, got {tag_rects:?}"
    );
    assert!(
        tag_rects.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible left-axis TagY marker, got {tag_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_overlay_labels_on_declarative_path() {
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
    plot_state
        .overlays
        .drag_lines_x
        .push(DragLineX::new(30, 2.0).label("X Drag").show_value(false));
    plot_state.overlays.drag_lines_y.push(
        DragLineY::new(31, 1.0, YAxis::Left)
            .label("Y Drag")
            .show_value(false),
    );
    plot_state
        .overlays
        .drag_points
        .push(DragPoint::new(32, DataPoint { x: 2.0, y: 1.0 }, YAxis::Left).label("Point Drag"));
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
        "plot-declarative-drag-labels",
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
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("X Drag"),
        "declarative line plot should prepare draggable X-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Y Drag"),
        "declarative line plot should prepare draggable Y-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Point Drag"),
        "declarative line plot should prepare draggable point label text, got {prepared_text:?}"
    );

    let label_rects = line_plot_reference_line_rects(&scene);
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 168.0).abs() < 0.01
                && (rect.origin.y.0 - 146.0).abs() < 0.01
                && (rect.size.width.0 - 2.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible draggable X-line label marker, got {label_rects:?}"
    );
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible draggable Y-line label marker, got {label_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path() {
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
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.drag_lines_y.push(
        DragLineY::new(40, 100.0, YAxis::Right)
            .label("Right Y Drag")
            .show_value(false),
    );
    plot_state.overlays.drag_points.push(
        DragPoint::new(41, DataPoint { x: 2.0, y: 50.0 }, YAxis::Right).label("Right Point Drag"),
    );
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
        "plot-declarative-right-axis-drag-labels",
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
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Right Y Drag"),
        "declarative line plot should prepare right-axis draggable Y-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Right Point Drag"),
        "declarative line plot should prepare right-axis draggable point label text, got {prepared_text:?}"
    );

    let label_rects = line_plot_reference_line_rects(&scene);
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 304.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible right-axis draggable Y-line label marker, got {label_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_plot_image_overlay_on_declarative_path() {
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
    let uv = UvRect {
        u0: 0.25,
        v0: 0.10,
        u1: 0.75,
        v1: 0.90,
    };
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 1.0,
                y_max: 3.0,
            },
            YAxis::Left,
        )
        .uv(uv)
        .opacity(0.5),
    );
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
        "plot-declarative-image-overlay",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let image_regions = line_plot_image_regions(&scene);
    assert!(
        image_regions.iter().any(|(rect, found_uv, opacity)| {
            (rect.origin.x.0 - 97.5).abs() < 0.01
                && (rect.origin.y.0 - 44.5).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
                && *found_uv == uv
                && (*opacity - 0.5).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned PlotImage overlay, got {image_regions:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_plot_image_overlays_on_declarative_path() {
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
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right2_series = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 200.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3_series = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 3000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right3);
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        left_series,
        right2_series,
        right3_series,
    ]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 200.0,
            },
            YAxis::Right2,
        )
        .opacity(0.42),
    );
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 3000.0,
            },
            YAxis::Right3,
        )
        .opacity(0.43),
    );
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
        "plot-declarative-right-axis-image-overlays",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let image_regions = line_plot_image_regions(&scene);
    for expected_opacity in [0.42, 0.43] {
        assert!(
            image_regions.iter().any(|(rect, _uv, opacity)| {
                (rect.origin.x.0 - 97.5).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 143.0).abs() < 0.01
                    && (rect.size.height.0 - 146.0).abs() < 0.01
                    && (*opacity - expected_opacity).abs() < 0.01
            }),
            "declarative line plot should paint right-axis PlotImage overlay with opacity {expected_opacity}, got {image_regions:?}"
        );
    }
}

#[test]
fn line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path() {
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
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.tags_y.push(
        TagY::new(100.0, YAxis::Right)
            .label("threshold")
            .show_value(true),
    );
    plot_state.overlays.text.push(
        PlotText::new(2.0, 50.0, YAxis::Right, "right-axis note")
            .background(Color::from_srgb_hex_rgb(0x0A141E)),
    );
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
        "plot-declarative-right-axis-tagy-text",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let tag_y_quads = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    let tag_y_texts = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    assert!(
        tag_y_quads >= 2 && tag_y_texts >= 2,
        "declarative line plot should paint right-axis TagY and PlotText overlays"
    );
}
