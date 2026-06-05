use super::*;

#[test]
fn line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path() {
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
        "plot-declarative-wheel-zoom-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let updated = state
        .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
        .expect("plot state should be readable");
    let view = updated
        .1
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        !updated.0,
        "declarative wheel zoom should switch/keep plot view in controlled mode"
    );
    assert!(
        view.x_max - view.x_min < 4.0 && view.y_max - view.y_min < 4.0,
        "positive wheel delta should zoom the declarative view in around the pointer, got {view:?}"
    );
    assert!(
        view.x_min > 0.0 && view.x_max < 4.0 && view.y_min > 0.0 && view.y_max < 4.0,
        "center wheel zoom should keep the next view inside the previous bounds, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path() {
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
        "plot-declarative-wheel-x-only-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "Shift+wheel should zoom the declarative X range, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "Shift+wheel should preserve the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path() {
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
        "plot-declarative-wheel-y-only-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "Ctrl+wheel should preserve the declarative X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "Ctrl+wheel should zoom the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path() {
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
        "plot-declarative-wheel-x-axis-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(163.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative x-axis wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "wheel over the declarative X axis should zoom the X range, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "wheel over the declarative X axis should preserve the Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path() {
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
        "plot-declarative-wheel-y-axis-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(17.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative y-axis wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "wheel over the declarative Y axis should preserve the X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "wheel over the declarative Y axis should zoom the Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path() {
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
    plot_state.axis_locks.x.zoom = true;
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
        "plot-declarative-wheel-x-lock-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "X zoom lock should preserve the declarative X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "X zoom lock should still allow declarative Y zoom, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path() {
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
    plot_state.axis_locks.y.zoom = true;
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
        "plot-declarative-wheel-y-lock-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "Y zoom lock should still allow declarative X zoom, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "Y zoom lock should preserve the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path() {
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
    plot_state.axis_locks.x.zoom = true;
    plot_state.axis_locks.y.zoom = true;
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
        "plot-declarative-wheel-both-lock-view",
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
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should preserve explicit view bounds");
    assert_eq!(
        view,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "wheel zoom should not change declarative view bounds when both axes are zoom-locked"
    );
}
