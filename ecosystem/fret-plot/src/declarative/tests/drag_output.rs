use super::*;

#[test]
fn line_plot_panel_drags_right_axis_y_line_output_on_declarative_path() {
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
        "plot-declarative-right-axis-drag-line-y-output",
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(8.0)),
            button: MouseButton::Left,
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
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable Y line should publish drag output");
    match drag {
        PlotDragOutput::LineY { id, axis, y, phase } => {
            assert_eq!(id, 50);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (y - 50.0).abs() < 0.2,
                "dragging to the plot middle should map through right-axis bounds, got {y}"
            );
        }
        other => panic!("expected right-axis LineY drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable Y line should publish drag end output");
    match drag {
        PlotDragOutput::LineY { id, axis, y, phase } => {
            assert_eq!(id, 50);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (y - 50.0).abs() < 0.2,
                "drag end should preserve the right-axis mapped value, got {y}"
            );
        }
        other => panic!("expected right-axis LineY drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_x_line_output_on_declarative_path() {
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
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
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
        .push(DragLineX::new(60, 1.0));
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
        "plot-declarative-drag-line-x-output",
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(98.0), Px(81.0)),
            button: MouseButton::Left,
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
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("draggable X line should publish drag output");
    match drag {
        PlotDragOutput::LineX { id, x, phase } => {
            assert_eq!(id, 60);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (x - 2.0).abs() < 0.03,
                "dragging to the plot middle should map through the X view bounds, got {x}"
            );
        }
        other => panic!("expected LineX drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("draggable X line should publish drag end output");
    match drag {
        PlotDragOutput::LineX { id, x, phase } => {
            assert_eq!(id, 60);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (x - 2.0).abs() < 0.03,
                "drag end should preserve the X mapped value, got {x}"
            );
        }
        other => panic!("expected LineX drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_right_axis_point_output_on_declarative_path() {
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
    plot_state.overlays.drag_points.push(DragPoint::new(
        70,
        DataPoint { x: 2.0, y: 50.0 },
        YAxis::Right,
    ));
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
        "plot-declarative-right-axis-drag-point-output",
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
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
            position: Point::new(Px(240.5), Px(117.5)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable point should publish drag output");
    match drag {
        PlotDragOutput::Point {
            id,
            axis,
            point,
            phase,
        } => {
            assert_eq!(id, 70);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (point.x - 3.0).abs() < 0.03,
                "dragging point right should map through the X view bounds, got {point:?}"
            );
            assert!(
                (point.y - 25.0).abs() < 0.3,
                "dragging point down should map through right-axis bounds, got {point:?}"
            );
        }
        other => panic!("expected right-axis Point drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(240.5), Px(117.5)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable point should publish drag end output");
    match drag {
        PlotDragOutput::Point {
            id,
            axis,
            point,
            phase,
        } => {
            assert_eq!(id, 70);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (point.x - 3.0).abs() < 0.03 && (point.y - 25.0).abs() < 0.3,
                "drag end should preserve the mapped point, got {point:?}"
            );
        }
        other => panic!("expected right-axis Point drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_right_axis_rect_output_on_declarative_path() {
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
    plot_state.overlays.drag_rects.push(DragRect::new(
        80,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 25.0,
            y_max: 75.0,
        },
        YAxis::Right,
    ));
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
        "plot-declarative-right-axis-drag-rect-output",
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
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
            position: Point::new(Px(240.5), Px(117.5)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable rect should publish drag output");
    match drag {
        PlotDragOutput::Rect {
            id,
            axis,
            rect,
            phase,
        } => {
            assert_eq!(id, 80);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (rect.x_min - 2.0).abs() < 0.03
                    && (rect.x_max - 4.0).abs() < 0.03
                    && (rect.y_min - 0.0).abs() < 0.3
                    && (rect.y_max - 50.0).abs() < 0.3,
                "dragging inside the rect should move the whole right-axis rect, got {rect:?}"
            );
        }
        other => panic!("expected right-axis Rect drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(240.5), Px(117.5)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable rect should publish drag end output");
    match drag {
        PlotDragOutput::Rect {
            id,
            axis,
            rect,
            phase,
        } => {
            assert_eq!(id, 80);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (rect.x_min - 2.0).abs() < 0.03
                    && (rect.x_max - 4.0).abs() < 0.03
                    && (rect.y_min - 0.0).abs() < 0.3
                    && (rect.y_max - 50.0).abs() < 0.3,
                "drag end should preserve the mapped rect, got {rect:?}"
            );
        }
        other => panic!("expected right-axis Rect drag end output, got {other:?}"),
    }
}
