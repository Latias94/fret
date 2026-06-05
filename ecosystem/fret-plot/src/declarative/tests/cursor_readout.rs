use super::*;

#[test]
fn line_plot_panel_updates_output_cursor_on_pointer_move() {
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
    let output = app.models_mut().insert(PlotOutput::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pointer-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).output(output.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

    let published = output
        .read_ref(&app, |output| *output)
        .expect("plot output model should be readable");
    assert_eq!(published.revision, 1);
    let cursor = published
        .snapshot
        .cursor
        .expect("pointer inside the plot region should publish cursor data");
    assert!(
        (cursor.x - 1.0).abs() < 0.02,
        "expected pointer x to map to the middle of the data domain, got {:?}",
        cursor
    );
    assert!(
        (cursor.y - 0.5).abs() < 0.04,
        "expected pointer y to map to the middle of the data domain, got {:?}",
        cursor
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(4.0), Px(4.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    let published = output
        .read_ref(&app, |output| *output)
        .expect("plot output model should be readable");
    assert_eq!(published.revision, 2);
    assert_eq!(published.snapshot.cursor, None);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Path {
                order: DrawOrder(20),
                ..
            }
        )),
        "managed-surface pointer handling must preserve declarative line painting"
    );
}

#[test]
fn line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path() {
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

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-cursor-readout",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let cursor_guides = scene
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
    assert!(
        cursor_guides >= 2,
        "declarative line plot should paint cursor crosshair guides"
    );

    let readout_backgrounds = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(12),
                    ..
                }
            )
        })
        .count();
    assert!(
        readout_backgrounds >= 1,
        "declarative line plot should paint mouse readout overlay chrome"
    );

    let readout_text = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(13),
                    ..
                }
            )
        })
        .count();
    assert!(
        readout_text >= 1,
        "declarative line plot should paint mouse readout text"
    );

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Path {
                order: DrawOrder(20),
                ..
            }
        )),
        "cursor readout painting must preserve declarative line painting"
    );
}

#[test]
fn line_plot_panel_paints_series_readout_rows_on_declarative_cursor_overlay() {
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
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-series-readout",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut prepared_text = services.prepared_text.join("\n");
    prepared_text.make_ascii_lowercase();
    assert!(
        prepared_text.contains("alpha: y="),
        "declarative cursor readout should include per-series readout rows, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_series_readout_with_right_axis_formatter_on_declarative_path()
{
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
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        LineSeries::new(
            "RightAxis",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right),
    ]));

    let right_axis_labels =
        AxisLabelFormatter::custom(0x5279_6768_7441, |v, _span| format!("R{v:.1}"));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-series-readout",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).y2_axis_labels(right_axis_labels),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut prepared_text = services.prepared_text.join("\n");
    prepared_text.make_ascii_lowercase();
    assert!(
        prepared_text.contains("rightaxis: y2=r1.0"),
        "right-axis cursor readout should use the right-axis formatter, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path() {
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
    plot_state.linked_cursor_x = Some(1.0);
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-linked-cursor-readout",
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

    let linked_cursor_guides = scene
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
    assert_eq!(
        linked_cursor_guides, 1,
        "linked cursor should paint one vertical guide when no local cursor is active"
    );

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad {
                order: DrawOrder(12),
                ..
            }
        )),
        "linked cursor should paint readout overlay chrome"
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Text {
                order: DrawOrder(13),
                ..
            }
        )),
        "linked cursor should paint readout text"
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

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let local_cursor_guides = scene
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
    assert_eq!(
        local_cursor_guides, 2,
        "local cursor crosshair should take precedence over linked cursor"
    );
}
