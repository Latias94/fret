use std::collections::BTreeMap;

use crate::action::Action;
use crate::data::{Column, DataTable};
use crate::engine::ChartEngine;
use crate::engine::window::DataWindow;
use crate::marks::MarkPayloadRef;
use crate::scheduler::WorkBudget;
use crate::spec::{
    AxisKind, AxisPointerSpec, AxisRange, AxisSpec, ChartSpec, DataZoomXSpec, DatasetSpec,
    FieldSpec, FilterMode, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
};
use crate::text::{TextMeasurer, TextMetrics};
use crate::transform::RowRange;
use fret_core::{Point, Px, Rect, Size};

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
                name: None,
                kind: AxisKind::X,
                grid: crate::ids::GridId::new(1),
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: crate::ids::AxisId::new(2),
                name: None,
                kind: AxisKind::Y,
                grid: crate::ids::GridId::new(1),
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: crate::ids::SeriesId::new(1),
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis: crate::ids::AxisId::new(1),
            y_axis: crate::ids::AxisId::new(2),
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    }
}

#[test]
fn bar_emits_rect_batch() {
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
            Size::new(Px(300.0), Px(200.0)),
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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                }),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    table.push_column(Column::F64(vec![1.0, -2.0, 3.0, 0.5]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(16_384, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    assert!(!marks.arena.rects.is_empty());
    assert_eq!(marks.arena.rects.len(), 4);
    assert!(
        marks
            .nodes
            .iter()
            .any(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
    );
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

#[test]
fn set_view_window_2d_respects_zoom_lock() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let mut engine = ChartEngine::new(basic_spec()).unwrap();

    engine.apply_action(Action::ToggleAxisZoomLock { axis: x_axis });
    let rev = engine.state().revision;

    engine.apply_action(Action::SetViewWindow2D {
        x_axis,
        y_axis,
        x: Some(DataWindow {
            min: 10.0,
            max: 20.0,
        }),
        y: Some(DataWindow {
            min: -5.0,
            max: 5.0,
        }),
    });

    let actual_x = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .window;
    assert_eq!(actual_x, None);

    assert_eq!(
        engine.state().data_window_y.get(&y_axis).copied(),
        Some(DataWindow {
            min: -5.0,
            max: 5.0
        })
    );
    assert_ne!(engine.state().revision, rev);
}

#[test]
fn pan_lock_prevents_pan_window_update() {
    let x_axis = crate::ids::AxisId::new(1);
    let mut engine = ChartEngine::new(basic_spec()).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ToggleAxisPanLock { axis: x_axis });
    let rev = engine.state().revision;

    engine.apply_action(Action::PanDataWindowXFromBase {
        axis: x_axis,
        base,
        delta_px: 10.0,
        viewport_span_px: 100.0,
    });

    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .window;
    assert_eq!(actual, Some(base));
    assert_eq!(engine.state().revision, rev);
}

#[test]
fn zoom_lock_prevents_zoom_window_update() {
    let x_axis = crate::ids::AxisId::new(1);
    let mut engine = ChartEngine::new(basic_spec()).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ToggleAxisZoomLock { axis: x_axis });
    let rev = engine.state().revision;

    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base,
        center_px: 50.0,
        log2_scale: 1.0,
        viewport_span_px: 100.0,
    });

    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .window;
    assert_eq!(actual, Some(base));
    assert_eq!(engine.state().revision, rev);
}

#[test]
fn set_data_window_applies_axis_range_lock_min() {
    let mut spec = basic_spec();
    let x_axis = spec.axes[0].id;
    spec.axes[0].range = Some(AxisRange::LockMin { min: 200.0 });

    let mut engine = ChartEngine::new(spec).unwrap();

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let stored = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected x window to be stored");
    assert_eq!(
        stored,
        DataWindow {
            min: 200.0,
            max: 210.0
        }
    );
}

#[test]
fn set_view_window_2d_applies_axis_range_lock_max() {
    let mut spec = basic_spec();
    let x_axis = spec.axes[0].id;
    let y_axis = spec.axes[1].id;
    spec.axes[1].range = Some(AxisRange::LockMax { max: -100.0 });

    let mut engine = ChartEngine::new(spec).unwrap();

    engine.apply_action(Action::SetViewWindow2D {
        x_axis,
        y_axis,
        x: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
        y: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let y = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window to be stored");
    assert_eq!(
        y,
        DataWindow {
            min: -110.0,
            max: -100.0
        }
    );
}

#[test]
fn fixed_axis_prevents_pan_and_zoom_actions() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.axes[0].range = Some(AxisRange::Fixed {
        min: 0.0,
        max: 10.0,
    });
    spec.axes[1].range = Some(AxisRange::Fixed {
        min: -5.0,
        max: 5.0,
    });

    let mut engine = ChartEngine::new(spec).unwrap();
    let rev = engine.state().revision;

    let base_x = DataWindow {
        min: 0.0,
        max: 10.0,
    };
    engine.apply_action(Action::PanDataWindowXFromBase {
        axis: x_axis,
        base: base_x,
        delta_px: 10.0,
        viewport_span_px: 100.0,
    });
    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base: base_x,
        center_px: 50.0,
        log2_scale: 1.0,
        viewport_span_px: 100.0,
    });

    let actual_x = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .window;
    assert_eq!(actual_x, None);

    let base_y = DataWindow {
        min: -5.0,
        max: 5.0,
    };
    engine.apply_action(Action::PanDataWindowYFromBase {
        axis: y_axis,
        base: base_y,
        delta_px: 10.0,
        viewport_span_px: 100.0,
    });
    engine.apply_action(Action::ZoomDataWindowYFromBase {
        axis: y_axis,
        base: base_y,
        center_px: 50.0,
        log2_scale: 1.0,
        viewport_span_px: 100.0,
    });

    assert!(engine.state().data_window_y.get(&y_axis).is_none());
    assert_eq!(engine.state().revision, rev);
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
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Band,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y0_field,
                y2: Some(y1_field),
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
fn stacked_area_emits_two_polylines() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    let stack = crate::ids::StackId::new(1);

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
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Area,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: Some(stack),
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 4096usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        xs.push(t * 10.0);
        ys.push((t * core::f64::consts::TAU).sin());
    }

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..64 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(16_384, 0, 4))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let out = engine.output();
    let variants: Vec<u8> = out
        .marks
        .nodes
        .iter()
        .filter(|n| {
            n.source_series == Some(series_id) && n.kind == crate::marks::MarkKind::Polyline
        })
        .map(|n| (n.id.0 & 0x7) as u8)
        .collect();

    assert!(
        variants.contains(&1) && variants.contains(&2),
        "expected stacked area to emit base/top polyline variants, got: {variants:?}"
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
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::Fixed {
                    min: 0.0,
                    max: 100.0,
                }),
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::LockMin { min: 200.0 }),
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
    engine.datasets_mut().insert(dataset_id, table);

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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
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
        engine.datasets_mut().insert(dataset_id, table);
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
                mode: Some(crate::spec::FilterMode::None),
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

#[test]
fn data_window_filter_mode_resets_to_spec_default() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_id,
            axis: x_axis,
            filter_mode: FilterMode::None,
        }],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .filter_mode;
    assert_eq!(actual, FilterMode::None);

    engine.apply_action(Action::SetDataWindowXFilterMode {
        axis: x_axis,
        mode: Some(FilterMode::Filter),
    });
    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .filter_mode;
    assert_eq!(actual, FilterMode::Filter);

    engine.apply_action(Action::SetDataWindowXFilterMode {
        axis: x_axis,
        mode: None,
    });
    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .filter_mode;
    assert_eq!(actual, FilterMode::None);
}

#[test]
fn set_data_window_x_inserts_state_with_spec_default_filter_mode() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
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
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_id,
            axis: x_axis,
            filter_mode: FilterMode::None,
        }],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    engine.state_mut().data_zoom_x.remove(&x_axis);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 10.0,
            max: 20.0,
        }),
    });

    let actual = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .copied()
        .unwrap_or_default()
        .filter_mode;
    assert_eq!(actual, FilterMode::None);
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

#[test]
fn hover_does_not_rebuild_marks() {
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
            Size::new(Px(100.0), Px(100.0)),
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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: Some(AxisPointerSpec::default()),
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);
    let marks_rev = engine.output().marks.revision;

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(10.0), Px(10.0)),
    });

    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);
    assert_eq!(engine.output().marks.revision, marks_rev);
}

#[test]
fn axis_pointer_is_emitted_when_hit_is_close_enough() {
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
            Size::new(Px(100.0), Px(100.0)),
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
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            snap: false,
            trigger_distance_px: 50.0,
            throttle_px: 0.0,
        }),
        series: vec![SeriesSpec {
            id: series_id,
            name: Some("MySeries".to_string()),
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(0.0), Px(100.0)),
    });

    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref();
    assert!(axis_pointer.is_some());
    let axis_pointer = axis_pointer.unwrap();
    assert_eq!(axis_pointer.crosshair_px, Point::new(Px(0.0), Px(100.0)));
    assert!(!axis_pointer.tooltip.lines.is_empty());
    assert_eq!(axis_pointer.tooltip.lines[0].value, "MySeries");
    let x_label = axis_pointer.tooltip.lines[1].label.as_str();
    assert!(x_label.contains("Time"), "unexpected x label: {x_label}");
}

#[test]
fn axis_pointer_axis_trigger_emits_multi_series_tooltip() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_a_field,
                    column: 1,
                },
                FieldSpec {
                    id: y_b_field,
                    column: 2,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        series: vec![
            SeriesSpec {
                id: series_a,
                name: Some("A".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                area_baseline: None,
            },
            SeriesSpec {
                id: series_b,
                name: Some("B".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                area_baseline: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 2.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(50.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert_eq!(axis_pointer.crosshair_px, Point::new(Px(50.0), Px(50.0)));
    assert_eq!(axis_pointer.tooltip.lines[0].label, "x (Time)");
    assert_eq!(axis_pointer.tooltip.lines[1].label, "A");
    assert_eq!(axis_pointer.tooltip.lines[1].value, "0.5");
    assert_eq!(axis_pointer.tooltip.lines[2].label, "B");
    assert_eq!(axis_pointer.tooltip.lines[2].value, "1");
}

#[test]
fn axis_pointer_axis_trigger_handles_non_monotonic_x_by_nearest_sample() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_a_field,
                    column: 1,
                },
                FieldSpec {
                    id: y_b_field,
                    column: 2,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        series: vec![
            SeriesSpec {
                id: series_a,
                name: Some("A".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                area_baseline: None,
            },
            SeriesSpec {
                id: series_b,
                name: Some("B".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                area_baseline: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    // Non-monotonic X: [0, 1, 0.5].
    table.push_column(Column::F64(vec![0.0, 1.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, 10.0]));
    table.push_column(Column::F64(vec![0.0, 2.0, 20.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(50.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert_eq!(axis_pointer.tooltip.lines[0].label, "x (Time)");
    assert_eq!(axis_pointer.tooltip.lines[1].label, "A");
    assert_eq!(axis_pointer.tooltip.lines[1].value, "10");
    assert_eq!(axis_pointer.tooltip.lines[2].label, "B");
    assert_eq!(axis_pointer.tooltip.lines[2].value, "20");
}

#[test]
fn axis_pointer_axis_trigger_snaps_to_hit_point_when_enabled() {
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
            Size::new(Px(100.0), Px(100.0)),
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
                name: Some("Time".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            snap: true,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        series: vec![SeriesSpec {
            id: series_id,
            name: Some("A".to_string()),
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(25.0), Px(75.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let hit = axis_pointer.hit.expect("expected a hit for snapping");
    assert_eq!(axis_pointer.crosshair_px, hit.point_px);
}

#[test]
fn scatter_emits_point_marks() {
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
            Size::new(Px(100.0), Px(100.0)),
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
                name: Some("X".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Y".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: Some("S".to_string()),
            kind: SeriesKind::Scatter,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    let n = 256usize;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.5).sin()).collect();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let mut steps = 0;
    loop {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        steps += 1;
        if !step.unfinished || steps > 64 {
            break;
        }
    }
    assert!(steps <= 64);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert_eq!(points.points.end - points.points.start, n);
}

#[test]
fn scatter_large_mode_is_pixel_bounded() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
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
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Scatter,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            area_baseline: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 50_000usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        xs.push(i as f64 / (n as f64 - 1.0));
        ys.push(((i as f64) * 0.01).sin());
    }
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let mut steps = 0;
    loop {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        steps += 1;
        if !step.unfinished || steps > 256 {
            break;
        }
    }
    assert!(steps <= 256);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    let emitted = points.points.end - points.points.start;

    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    assert!(
        emitted <= width_px * 4,
        "emitted={emitted} width={width_px}"
    );
    assert!(emitted > 0);
}

fn find_polyline_point_by_data_index(
    marks: &crate::marks::MarkTree,
    series: crate::ids::SeriesId,
    data_index: u32,
) -> Option<fret_core::Point> {
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series))?;
    let crate::marks::MarkPayloadRef::Polyline(poly) = &node.payload else {
        return None;
    };
    for (p, &i) in marks.arena.points[poly.points.clone()]
        .iter()
        .zip(marks.arena.data_indices[poly.points.clone()].iter())
    {
        if i == data_index {
            return Some(*p);
        }
    }
    None
}

fn data_to_px_y(y: f64, y_min: f64, y_max: f64, viewport: Rect) -> f32 {
    let span = (y_max - y_min).max(1e-12);
    let t = ((y - y_min) / span).clamp(0.0, 1.0);
    viewport.origin.y.0 + (1.0 - (t as f32)) * viewport.size.height.0
}

#[test]
fn stacked_line_series_offsets_y() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let stack = crate::ids::StackId::new(1);

    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);

    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_a_field,
                    column: 1,
                },
                FieldSpec {
                    id: y_b_field,
                    column: 2,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack),
                stack_strategy: Default::default(),
                area_baseline: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack),
                stack_strategy: Default::default(),
                area_baseline: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    table.push_column(Column::F64(vec![1.0, 1.0, 1.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 2.0, 2.0, 2.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!(y_window.max > y_window.min);

    let marks = &engine.output().marks;
    let p_a = find_polyline_point_by_data_index(marks, series_a, 0).expect("point a");
    let p_b = find_polyline_point_by_data_index(marks, series_b, 0).expect("point b");

    let expected_a = data_to_px_y(1.0, y_window.min, y_window.max, viewport);
    let expected_b = data_to_px_y(3.0, y_window.min, y_window.max, viewport);
    assert!((p_a.y.0 - expected_a).abs() < 0.5);
    assert!((p_b.y.0 - expected_b).abs() < 0.5);
    assert!(p_b.y.0 < p_a.y.0);
}

#[test]
fn stack_strategy_samesign_separates_positive_and_negative() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let stack = crate::ids::StackId::new(1);

    let series_pos = crate::ids::SeriesId::new(1);
    let series_neg = crate::ids::SeriesId::new(2);

    let x_field = crate::ids::FieldId::new(1);
    let y_pos_field = crate::ids::FieldId::new(2);
    let y_neg_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_pos_field,
                    column: 1,
                },
                FieldSpec {
                    id: y_neg_field,
                    column: 2,
                },
            ],
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![
            SeriesSpec {
                id: series_pos,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_pos_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack),
                stack_strategy: crate::spec::StackStrategy::SameSign,
                area_baseline: None,
            },
            SeriesSpec {
                id: series_neg,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_neg_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack),
                stack_strategy: crate::spec::StackStrategy::SameSign,
                area_baseline: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    table.push_column(Column::F64(vec![1.0, 1.0, 1.0, 1.0]));
    table.push_column(Column::F64(vec![-2.0, -2.0, -2.0, -2.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!(y_window.max > y_window.min);

    let marks = &engine.output().marks;
    let p_neg = find_polyline_point_by_data_index(marks, series_neg, 0).expect("point neg");

    let expected = data_to_px_y(-2.0, y_window.min, y_window.max, viewport);
    assert!((p_neg.y.0 - expected).abs() < 0.5);
}
