use std::collections::BTreeMap;

use crate::action::Action;
use crate::data::{Column, DataTable};
use crate::engine::ChartEngine;
use crate::engine::window::DataWindow;
use crate::scheduler::WorkBudget;
use crate::spec::{
    AxisKind, AxisRange, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode,
    SeriesKind, SeriesSpec,
};
use crate::text::{TextMeasurer, TextMetrics};
use crate::view::RowRange;
use fret_core::{Px, Rect, Size};

fn basic_spec() -> ChartSpec {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: None,
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec {
            id: crate::ids::GridId::new(1),
        }],
        axes: vec![
            AxisSpec {
                id: crate::ids::AxisId::new(1),
                kind: AxisKind::X,
                grid: crate::ids::GridId::new(1),
                range: None,
            },
            AxisSpec {
                id: crate::ids::AxisId::new(2),
                kind: AxisKind::Y,
                grid: crate::ids::GridId::new(1),
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: crate::ids::SeriesId::new(1),
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis: crate::ids::AxisId::new(1),
            y_axis: crate::ids::AxisId::new(2),
            area_baseline: None,
        }],
    }
}

#[test]
fn set_view_window_2d_updates_both_axes() {
    let mut engine = ChartEngine::new(basic_spec()).unwrap();
    engine.apply_action(Action::SetViewWindow2D {
        x_axis: crate::ids::AxisId::new(1),
        y_axis: crate::ids::AxisId::new(2),
        x: Some(DataWindow {
            min: 10.0,
            max: 20.0,
        }),
        y: Some(DataWindow {
            min: -5.0,
            max: 5.0,
        }),
    });

    let mut expected_x = BTreeMap::new();
    expected_x.insert(
        crate::ids::AxisId::new(1),
        DataWindow {
            min: 10.0,
            max: 20.0,
        },
    );
    let x_axis = crate::ids::AxisId::new(1);
    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .window;
    assert_eq!(actual, expected_x.get(&x_axis).copied());

    let mut expected_y = BTreeMap::new();
    expected_y.insert(
        crate::ids::AxisId::new(2),
        DataWindow {
            min: -5.0,
            max: 5.0,
        },
    );
    assert_eq!(engine.state().data_window_y, expected_y);
}

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl TextMeasurer for NullTextMeasurer {
    fn measure(
        &mut self,
        _text: crate::ids::StringId,
        _style: crate::text::TextStyleId,
    ) -> TextMetrics {
        TextMetrics::default()
    }
}

#[test]
fn band_emits_two_polylines() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(400.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: None,
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Band,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y0_field,
                y2: Some(y1_field),
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    // Intentionally larger than the per-step bounds budget (4096) so that the engine must
    // make progress across multiple `step()` calls.
    let n = 32_768usize;
    let mut xs = Vec::with_capacity(n);
    let mut lo = Vec::with_capacity(n);
    let mut hi = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        xs.push(t * 10.0);
        let y = (t * core::f64::consts::TAU).sin();
        lo.push(y);
        hi.push(y + 0.5);
    }

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(lo));
    table.push_column(Column::F64(hi));
    engine.datasets_mut().datasets.push((dataset_id, table));

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..512 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(2_048, 0, 2))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let out = engine.output();
    assert!(!out.marks.nodes.is_empty(), "expected marks to be emitted");
    assert!(
        out.marks.nodes.len() >= 2,
        "expected band to emit two polyline nodes"
    );
    assert!(
        out.marks.arena.points.len() >= 2,
        "expected band marks to have points"
    );
}

#[test]
fn row_range_limits_mark_indices() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: None,
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 512usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64).sin()).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().datasets.push((dataset_id, table));

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 10, end: 20 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let indices = &engine.output().marks.arena.data_indices;
    assert!(!indices.is_empty(), "expected marks to contain indices");
    assert!(
        indices.iter().all(|&i| (10..20).contains(&(i as usize))),
        "expected all indices to be within the configured row range"
    );
}

#[test]
fn x_window_limits_mark_indices() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: None,
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 1_024usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64).cos()).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().datasets.push((dataset_id, table));

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 200.0,
            max: 400.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let indices = &engine.output().marks.arena.data_indices;
    assert!(!indices.is_empty(), "expected marks to contain indices");
    assert!(
        indices.iter().all(|&i| (200..=400).contains(&(i as usize))),
        "expected all indices to be within the configured x window"
    );
}

#[test]
fn axis_fixed_overrides_data_window_for_marks() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: Some(AxisRange::Fixed {
                    min: 0.0,
                    max: 100.0,
                }),
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 1_024usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64).cos()).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().datasets.push((dataset_id, table));

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 200.0,
            max: 400.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let indices = &engine.output().marks.arena.data_indices;
    assert!(!indices.is_empty(), "expected marks to contain indices");
    assert!(
        indices.iter().all(|&i| (0..=100).contains(&(i as usize))),
        "expected all indices to be within the fixed axis window"
    );
}

#[test]
fn set_series_visible_hides_marks() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: None,
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 512usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64).sin()).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().datasets.push((dataset_id, table));

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    assert!(
        !engine.output().marks.arena.data_indices.is_empty(),
        "expected marks to be present before hiding the series"
    );

    engine.apply_action(Action::SetSeriesVisible {
        series: series_id,
        visible: false,
    });

    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    assert!(
        engine.output().marks.nodes.is_empty(),
        "expected no mark nodes after hiding the series"
    );
    assert!(
        engine.output().marks.arena.data_indices.is_empty(),
        "expected no mark indices after hiding the series"
    );
}

#[test]
fn axis_lock_min_filters_bounds_to_prevent_y_compression() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: Some(AxisRange::LockMin { min: 200.0 }),
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 512usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| {
            if i < 200 {
                1000.0
            } else if i % 2 == 0 {
                1.0
            } else {
                -1.0
            }
        })
        .collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().datasets.push((dataset_id, table));

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let points = &engine.output().marks.arena.points;
    assert!(!points.is_empty(), "expected marks to contain points");

    let min_y = points
        .iter()
        .map(|p| p.y.0)
        .fold(f32::INFINITY, |a, b| a.min(b));
    let max_y = points
        .iter()
        .map(|p| p.y.0)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));

    assert!(
        min_y < 40.0,
        "expected some points to reach near the top when y-bounds are filtered by the x lock"
    );
    assert!(
        max_y > 200.0,
        "expected some points to reach near the bottom when y-bounds are filtered by the x lock"
    );
}

#[test]
fn data_window_filter_mode_none_keeps_y_bounds_global() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                kind: AxisKind::X,
                grid: grid_id,
                range: None,
            },
            AxisSpec {
                id: y_axis,
                kind: AxisKind::Y,
                grid: grid_id,
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: series_id,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            area_baseline: None,
        }],
    };

    let n = 512usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| {
            if i < 200 {
                1000.0
            } else if i % 2 == 0 {
                1.0
            } else {
                -1.0
            }
        })
        .collect();

    let make_engine = |filter_mode_none: bool| {
        let mut engine = ChartEngine::new(spec.clone()).unwrap();
        let mut table = DataTable::default();
        table.push_column(Column::F64(x.clone()));
        table.push_column(Column::F64(y.clone()));
        engine.datasets_mut().datasets.push((dataset_id, table));
        engine.apply_action(Action::SetDataWindowX {
            axis: x_axis,
            window: Some(DataWindow {
                min: 200.0,
                max: 400.0,
            }),
        });
        if filter_mode_none {
            engine.apply_action(Action::SetDataWindowXFilterMode {
                axis: x_axis,
                mode: Some(crate::engine::window_policy::FilterMode::None),
            });
        }
        engine
    };

    let mut measurer = NullTextMeasurer::default();

    let mut engine_filter = make_engine(false);
    for _ in 0..16 {
        let step = engine_filter
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let mut engine_none = make_engine(true);
    for _ in 0..16 {
        let step = engine_none
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let y_span_filter = span_px_y(&engine_filter.output().marks.arena.points);
    let y_span_none = span_px_y(&engine_none.output().marks.arena.points);

    assert!(
        y_span_filter > 80.0,
        "expected filtered mode to use a large vertical span in the zoom window"
    );
    assert!(
        y_span_none < 30.0,
        "expected FilterMode::None to compress visible y variation due to global y bounds"
    );
}

fn span_px_y(points: &[fret_core::Point]) -> f32 {
    if points.is_empty() {
        return 0.0;
    }

    let min = points
        .iter()
        .map(|p| p.y.0)
        .fold(f32::INFINITY, |a, b| a.min(b));
    let max = points
        .iter()
        .map(|p| p.y.0)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));

    (max - min).abs()
}
