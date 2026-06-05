use super::*;

#[test]
fn line_plot_panel_paints_seeded_line_on_declarative_path() {
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
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 4.0 },
                    DataPoint { x: 2.0, y: 2.0 },
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
        "plot-declarative-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let line_paths = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, fret_core::SceneOp::Path { order, .. } if order.0 >= 1))
        .count();
    assert!(
        line_paths > 0,
        "declarative line plot panel should emit at least one path"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn area_plot_panel_paints_area_fill_and_stroke_on_declarative_path() {
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
    let model = app.models_mut().insert(AreaPlotModel::from_series(vec![
        AreaSeries::new(
            "Area",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.8 },
                    DataPoint { x: 2.0, y: 0.4 },
                ],
                true,
            ),
        )
        .fill_alpha(0.25),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-area-panel",
        |cx| vec![area_plot_panel(cx, AreaPlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    let stroke_paths = scene
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
        fill_paths, 1,
        "declarative area plot should emit one filled area path"
    );
    assert_eq!(
        stroke_paths, 1,
        "declarative area plot should keep the area stroke path"
    );
    assert!(
        services
            .prepared_paths
            .iter()
            .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
        "area fill path should close back to the baseline"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn error_bars_plot_panel_paints_x_y_errors_caps_and_markers_on_declarative_path() {
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
        .insert(ErrorBarsPlotModel::from_series(vec![
            ErrorBarsSeries::new(
                "measurement",
                Series::from_points_sorted(vec![DataPoint { x: 1.0, y: 1.0 }], true),
            )
            .x_errors(std::sync::Arc::from([ErrorBar::symmetric(0.25)]))
            .y_errors(std::sync::Arc::from([ErrorBar::symmetric(0.5)]))
            .cap_size(Px(5.0))
            .marker_radius(Px(3.0)),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-error-bars-panel",
        |cx| {
            vec![error_bars_plot_panel(
                cx,
                ErrorBarsPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let error_paths = scene
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
        error_paths, 1,
        "declarative error-bars plot should emit one path for the series error bars"
    );

    let error_path = services
        .prepared_paths
        .iter()
        .find(|path| path.len() >= 16)
        .expect("error-bars path should include y-error, x-error, caps, and plus marker");
    assert!(
        !error_path
            .iter()
            .any(|command| matches!(command, PathCommand::Close)),
        "default error-bars markers and caps should be open stroke commands"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn histogram_plot_panel_paints_closed_bin_fill_paths_on_declarative_path() {
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
        .insert(HistogramPlotModel::from_series(vec![
            HistogramSeries::new("histogram", std::sync::Arc::from([0.1, 0.2, 0.8, 1.2, 1.8]))
                .bins(2)
                .range(0.0, 2.0)
                .bar_gap_fraction(0.0),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-histogram-panel",
        |cx| {
            vec![histogram_plot_panel(
                cx,
                HistogramPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        fill_paths, 1,
        "declarative histogram should emit one fill path for the series bins"
    );

    let histogram_path = services
        .prepared_paths
        .iter()
        .find(|path| {
            path.iter()
                .filter(|cmd| matches!(cmd, PathCommand::Close))
                .count()
                >= 2
        })
        .expect("histogram fill path should close each non-empty bin");
    assert_eq!(
        histogram_path
            .iter()
            .filter(|cmd| matches!(cmd, PathCommand::Close))
            .count(),
        2,
        "the fixture should produce two closed histogram bin rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn heatmap_plot_panel_paints_grid_cells_as_declarative_quads() {
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
    let model = app.models_mut().insert(HeatmapPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 0.5, 0.75, 1.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-heatmap-panel",
        |cx| {
            vec![heatmap_plot_panel(
                cx,
                HeatmapPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let heatmap_quads: Vec<_> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
        .collect();
    assert_eq!(
        heatmap_quads.len(),
        4,
        "declarative heatmap should emit one visible quad per finite grid cell"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn histogram2d_plot_panel_paints_grid_cells_and_default_colorbar_on_declarative_path() {
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
    let model = app.models_mut().insert(Histogram2DPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 2.0, 3.0, 4.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-histogram2d-panel",
        |cx| {
            vec![histogram2d_plot_panel(
                cx,
                Histogram2DPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let histogram2d_quads: Vec<_> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
        .collect();
    assert_eq!(
        histogram2d_quads.len(),
        4,
        "declarative histogram2d should emit one visible quad per finite grid cell"
    );

    let gradient_steps = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(4),
                    ..
                }
            )
        })
        .count();
    assert!(
        gradient_steps >= 8,
        "declarative histogram2d should paint a default colorbar gradient"
    );

    assert!(
        services.prepared_text.iter().any(|text| text == "4.000"),
        "declarative histogram2d colorbar should label the finite maximum value"
    );
    assert!(
        services.prepared_text.iter().any(|text| text == "0.000"),
        "declarative histogram2d colorbar should label the finite minimum value"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn heatmap_plot_panel_paints_default_colorbar_on_declarative_path() {
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
    let model = app.models_mut().insert(HeatmapPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 0.5, 0.75, 1.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-heatmap-colorbar-panel",
        |cx| {
            vec![heatmap_plot_panel(
                cx,
                HeatmapPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let gradient_steps = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(4),
                    ..
                }
            )
        })
        .count();
    assert!(
        gradient_steps >= 8,
        "declarative heatmap should paint a default colorbar gradient"
    );

    assert!(
        services.prepared_text.iter().any(|text| text == "1.000"),
        "declarative heatmap colorbar should label the finite maximum value"
    );
    assert!(
        services.prepared_text.iter().any(|text| text == "0.000"),
        "declarative heatmap colorbar should label the finite minimum value"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn candlestick_plot_panel_paints_wicks_and_up_down_bodies_on_declarative_path() {
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
        .insert(CandlestickPlotModel::from_series(vec![
            CandlestickSeries::new_sorted(
                "ohlc",
                std::sync::Arc::from([
                    OhlcPoint {
                        x: 0.0,
                        open: 1.0,
                        high: 2.0,
                        low: 0.5,
                        close: 1.5,
                    },
                    OhlcPoint {
                        x: 1.0,
                        open: 2.0,
                        high: 2.5,
                        low: 1.0,
                        close: 1.25,
                    },
                ]),
                true,
            )
            .width(0.8),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-candlestick-panel",
        |cx| {
            vec![candlestick_plot_panel(
                cx,
                CandlestickPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let wick_paths = scene
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
        wick_paths, 1,
        "declarative candlestick should emit one wick stroke path"
    );

    let body_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        body_paths, 2,
        "declarative candlestick should emit separate up and down body fill paths"
    );

    let closed_body_paths = services
        .prepared_paths
        .iter()
        .filter(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close)))
        .count();
    assert_eq!(
        closed_body_paths, 2,
        "up and down candle bodies should be closed fill rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn bars_plot_panel_paints_grouped_and_stacked_closed_fill_paths_on_declarative_path() {
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
    let grouped = BarSeries::new(
        "grouped",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
            true,
        ),
    )
    .width(0.8);
    let stacked = BarSeries::new(
        "stacked",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 2.5 }, DataPoint { x: 1.0, y: -1.5 }],
            true,
        ),
    )
    .width(0.8)
    .baseline_by_index(std::sync::Arc::from([1.0, -0.5]));
    let model = app
        .models_mut()
        .insert(BarsPlotModel::from_series(vec![grouped, stacked]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-bars-panel",
        |cx| vec![bars_plot_panel(cx, BarsPlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        fill_paths, 2,
        "declarative bars should emit one fill path per visible series"
    );

    let closed_bar_rects = services
        .prepared_paths
        .iter()
        .filter(|path| {
            path.iter()
                .filter(|cmd| matches!(cmd, PathCommand::Close))
                .count()
                >= 2
        })
        .count();
    assert_eq!(
        closed_bar_rects, 2,
        "grouped and stacked series should each close both bar rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn shaded_plot_panel_paints_band_fill_and_two_strokes_on_declarative_path() {
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
    let model = app.models_mut().insert(ShadedPlotModel::from_series(vec![
        crate::models::ShadedSeries::new(
            "Band",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.8 },
                    DataPoint { x: 1.0, y: 1.2 },
                    DataPoint { x: 2.0, y: 0.9 },
                ],
                true,
            ),
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.4 },
                    DataPoint { x: 2.0, y: 0.3 },
                ],
                true,
            ),
        )
        .fill_alpha(0.25),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-shaded-panel",
        |cx| {
            vec![shaded_plot_panel(
                cx,
                ShadedPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    let stroke_paths = scene
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
        fill_paths, 1,
        "declarative shaded plot should emit one filled band path"
    );
    assert_eq!(
        stroke_paths, 2,
        "declarative shaded plot should emit upper and lower stroke paths"
    );
    assert!(
        services
            .prepared_paths
            .iter()
            .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
        "shaded fill path should close the upper/lower band"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn stems_plot_panel_paints_stems_from_baseline_on_declarative_path() {
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
    let model = app.models_mut().insert(StemsPlotModel::from_series(vec![
        StemsSeries::new(
            "Stems",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.8 },
                    DataPoint { x: 2.0, y: 0.4 },
                ],
                true,
            ),
        )
        .baseline(0.0),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-stems-panel",
        |cx| {
            vec![stems_plot_panel(
                cx,
                StemsPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let stem_paths = scene
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
        stem_paths, 1,
        "declarative stems plot should emit one stem path"
    );

    let stem_path = services
        .prepared_paths
        .iter()
        .find(|path| {
            path.windows(2).any(|commands| {
                matches!(
                    (&commands[0], &commands[1]),
                    (PathCommand::MoveTo(_), PathCommand::LineTo(_))
                )
            })
        })
        .expect("stems panel should prepare move/line stem commands");
    assert!(
        stem_path.len() >= 6,
        "three sampled stems should produce at least six path commands; got {stem_path:?}"
    );
    assert!(
        !stem_path
            .iter()
            .any(|cmd| matches!(cmd, PathCommand::Close)),
        "stems should be strokes from the baseline, not closed fills"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}
