use super::*;

#[test]
fn line_plot_panel_paints_series_legend_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let model = app.models_mut().insert(LinePlotModel::from_series(series));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let legend_swatches = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(30),
                    ..
                }
            )
        })
        .count();
    assert!(
        legend_swatches >= 2,
        "declarative line plot should paint one legend swatch per series"
    );

    let legend_labels = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(31),
                    ..
                }
            )
        })
        .count();
    assert!(
        legend_labels >= 2,
        "declarative line plot should paint one legend label per series"
    );

    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        series_paths, 2,
        "legend painting should not replace seeded series paths"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let alpha_id = series[0].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-toggle",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert_eq!(series_paths, 2);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(32.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.contains(&alpha_id),
        "clicking a declarative legend swatch should hide that series"
    );

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        series_paths, 1,
        "hidden declarative legend series should be omitted from line painting"
    );
}

#[test]
fn line_plot_panel_legend_label_click_pins_and_unpins_series_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let beta_id = series[1].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-pin",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(64.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let pinned = state
        .read_ref(&app, |state| state.pinned_series)
        .expect("plot state should be readable");
    assert_eq!(
        pinned,
        Some(beta_id),
        "clicking a declarative legend label should pin that series"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Beta: y="),
        "pinned declarative legend series should be kept in cursor readout rows: {prepared_text:?}"
    );
    assert!(
        !prepared_text.contains("Alpha: y="),
        "pinning Beta should filter other declarative cursor readout rows: {prepared_text:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(64.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let pinned = state
        .read_ref(&app, |state| state.pinned_series)
        .expect("plot state should be readable");
    assert_eq!(
        pinned, None,
        "clicking a pinned declarative legend label should unpin it"
    );

    services.prepared_text.clear();
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Alpha: y=") && prepared_text.contains("Beta: y="),
        "unpinning should restore all visible declarative cursor readout rows: {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Gamma",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.5 },
                    DataPoint { x: 1.0, y: 1.25 },
                    DataPoint { x: 2.0, y: 0.75 },
                ],
                true,
            ),
        ),
    ];
    let alpha_id = series[0].id;
    let beta_id = series[1].id;
    let gamma_id = series[2].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-solo",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.contains(&alpha_id) && hidden.contains(&gamma_id) && !hidden.contains(&beta_id),
        "shift-clicking a declarative legend row should solo that series"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        series_paths, 1,
        "soloed declarative legend series should be the only painted line"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.is_empty(),
        "shift-clicking an already-solo declarative legend row should restore all series"
    );

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        series_paths, 3,
        "restoring declarative legend solo mode should paint every line series again"
    );
}

#[test]
fn line_plot_panel_legend_hover_emphasizes_series_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let alpha_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let beta_color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        )
        .color(alpha_color),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        )
        .color(beta_color),
    ]));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-hover",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(64.0), Px(32.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let legend_highlights = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(29),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        legend_highlights, 1,
        "hovering a declarative legend row should paint a legend highlight"
    );

    let mut alpha_path_alpha = None;
    let mut beta_path_alpha = None;
    for op in scene.ops() {
        let fret_core::SceneOp::Path {
            order: DrawOrder(20),
            paint,
            ..
        } = op
        else {
            continue;
        };
        if let Paint::Solid(color) = paint.paint {
            if (color.r - alpha_color.r).abs() < 0.001
                && (color.g - alpha_color.g).abs() < 0.001
                && (color.b - alpha_color.b).abs() < 0.001
            {
                alpha_path_alpha = Some(color.a);
            } else if (color.g - beta_color.g).abs() < 0.001 {
                beta_path_alpha = Some(color.a);
            }
        }
    }

    assert_eq!(
        alpha_path_alpha,
        Some(1.0),
        "hovered declarative legend series should keep full opacity"
    );
    assert!(
        beta_path_alpha.is_some_and(|alpha| alpha < 0.5),
        "non-hovered declarative line series should be dimmed while a legend row is hovered"
    );
}
