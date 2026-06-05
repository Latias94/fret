use super::*;

#[test]
fn line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path() {
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
    let left = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let right2 = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 1_000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3 = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 2_000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right3);
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        left, right, right2, right3,
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-labels",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .y2_axis_labels(AxisLabelFormatter::custom(0x5231u64, |v, _span| {
                        format!("R1:{v:.0}")
                    }))
                    .y3_axis_labels(AxisLabelFormatter::custom(0x5232u64, |v, _span| {
                        format!("R2:{v:.0}")
                    }))
                    .y4_axis_labels(AxisLabelFormatter::custom(0x5233u64, |v, _span| {
                        format!("R3:{v:.0}")
                    })),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R1:")),
        "declarative line plot should use the y2 formatter for right-axis tick labels, got {:?}",
        services.prepared_text
    );
    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R2:")),
        "declarative line plot should use the y3 formatter for right2-axis tick labels, got {:?}",
        services.prepared_text
    );
    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R3:")),
        "declarative line plot should use the y4 formatter for right3-axis tick labels, got {:?}",
        services.prepared_text
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_right_axis_series_with_right_axis_bounds_on_declarative_path() {
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
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    services.prepared_paths.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let plot = line_plot_inner_rect(bounds, LinePlotStyle::default());
    let right_path = services
        .prepared_paths
        .iter()
        .find(|commands| {
            commands.iter().any(|command| match command {
                PathCommand::LineTo(point) => (point.y.0 - plot.origin.y.0).abs() < 0.5,
                _ => false,
            })
        })
        .cloned();
    assert!(
        right_path.is_some(),
        "declarative right-axis series should use right-axis y bounds and reach the plot top; paths={:?}",
        services.prepared_paths
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_right2_and_right3_axis_series_with_axis_bounds_on_declarative_path() {
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
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
            true,
        ),
    );
    let right2_series = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 200.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3_series = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 1.0, y: 3000.0 },
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

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right23-axis-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    services.prepared_paths.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let endpoint_y: Vec<f32> = services
        .prepared_paths
        .iter()
        .filter_map(|commands| {
            commands.iter().find_map(|command| match command {
                PathCommand::LineTo(point) => Some(point.y.0),
                _ => None,
            })
        })
        .collect();
    assert_eq!(
        endpoint_y.len(),
        3,
        "left, right2, and right3 series should each emit a line endpoint; paths={:?}",
        services.prepared_paths
    );
    let right2_endpoint_y = endpoint_y[1];
    assert_eq!(
        endpoint_y
            .iter()
            .skip(1)
            .filter(|y| (**y - right2_endpoint_y).abs() < 0.5)
            .count(),
        2,
        "right2 and right3 series should project their max y values to the same plot-space endpoint through their own y bounds; endpoint_y={endpoint_y:?}, paths={:?}",
        services.prepared_paths
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}
