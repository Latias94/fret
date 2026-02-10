#![allow(clippy::all)]

use std::collections::BTreeMap;
use std::ops::Range;

use crate::action::Action;
use crate::data::{Column, DataTable};
use crate::engine::ChartEngine;
use crate::engine::window::DataWindow;
use crate::marks::MarkKind;
use crate::marks::MarkPayloadRef;
use crate::scheduler::WorkBudget;
use crate::spec::{
    AxisKind, AxisPointerSpec, AxisPointerType, AxisRange, AxisSpec, ChartSpec, DataZoomXSpec,
    DataZoomYSpec, DatasetSpec, FieldSpec, FilterMode, GridSpec, SeriesEncode, SeriesKind,
    SeriesSpec, VisualMapMode, VisualMapSpec,
};
use crate::text::{TextMeasurer, TextMetrics};
use crate::transform::{RowRange, RowSelection};
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    }
}

#[test]
fn multi_grid_plot_viewports_route_mark_mapping_by_grid() {
    use crate::engine::model::{ChartPatch, PatchMode};
    use crate::text::{TextMeasurer, TextMetrics};

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

    fn rect_contains_point(rect: Rect, point: Point) -> bool {
        let x0 = rect.origin.x.0;
        let y0 = rect.origin.y.0;
        let x1 = x0 + rect.size.width.0;
        let y1 = y0 + rect.size.height.0;
        point.x.0 >= x0 && point.x.0 <= x1 && point.y.0 >= y0 && point.y.0 <= y1
    }

    let dataset_id = crate::ids::DatasetId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let grid_a = crate::ids::GridId::new(1);
    let grid_b = crate::ids::GridId::new(2);

    let x_a = crate::ids::AxisId::new(1);
    let y_a = crate::ids::AxisId::new(2);
    let x_b = crate::ids::AxisId::new(3);
    let y_b = crate::ids::AxisId::new(4);

    let s_a = crate::ids::SeriesId::new(1);
    let s_b = crate::ids::SeriesId::new(2);

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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_a }, GridSpec { id: grid_b }],
        axes: vec![
            AxisSpec {
                id: x_a,
                name: None,
                kind: AxisKind::X,
                grid: grid_a,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_a,
                name: None,
                kind: AxisKind::Y,
                grid: grid_a,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: x_b,
                name: None,
                kind: AxisKind::X,
                grid: grid_b,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_b,
                name: None,
                kind: AxisKind::Y,
                grid: grid_b,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: s_a,
                name: None,
                kind: SeriesKind::Scatter,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x_a,
                y_axis: y_a,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: s_b,
                name: None,
                kind: SeriesKind::Scatter,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x_b,
                y_axis: y_b,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let viewport_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let viewport_b = Rect::new(
        Point::new(Px(200.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    let mut patch = ChartPatch::default();
    patch
        .plot_viewports_by_grid
        .insert(grid_a, Some(viewport_a));
    patch
        .plot_viewports_by_grid
        .insert(grid_b, Some(viewport_b));
    engine.apply_patch(patch, PatchMode::Merge).unwrap();

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..32 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let output = engine.output();
    assert_eq!(
        output.plot_viewports_by_grid.get(&grid_a).copied(),
        Some(viewport_a)
    );
    assert_eq!(
        output.plot_viewports_by_grid.get(&grid_b).copied(),
        Some(viewport_b)
    );

    let mut a_points: Vec<Point> = Vec::new();
    let mut b_points: Vec<Point> = Vec::new();
    for node in &output.marks.nodes {
        let series = match node.source_series {
            Some(s) => s,
            None => continue,
        };
        let out = if series == s_a {
            &mut a_points
        } else if series == s_b {
            &mut b_points
        } else {
            continue;
        };

        match &node.payload {
            MarkPayloadRef::Points(p) => {
                out.extend(p.points.clone().map(|i| output.marks.arena.points[i]));
            }
            MarkPayloadRef::Polyline(p) => {
                out.extend(p.points.clone().map(|i| output.marks.arena.points[i]));
            }
            MarkPayloadRef::Rect(r) => {
                out.extend(
                    r.rects
                        .clone()
                        .map(|i| output.marks.arena.rects[i])
                        .map(|rect| rect.origin),
                );
            }
            MarkPayloadRef::Text(t) => {
                out.push(t.rect.origin);
            }
            MarkPayloadRef::Group(_) => {}
        }
    }

    assert!(!a_points.is_empty());
    assert!(!b_points.is_empty());
    for p in a_points {
        assert!(rect_contains_point(viewport_a, p));
    }
    for p in b_points {
        assert!(rect_contains_point(viewport_b, p));
    }

    let before_windows = output.axis_windows.clone();

    engine.apply_action(Action::SetAxisWindowPercent {
        axis: x_a,
        range: Some((25.0, 75.0)),
    });
    for _ in 0..32 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let after_windows = engine.output().axis_windows.clone();
    assert_ne!(before_windows.get(&x_a), after_windows.get(&x_a));
    assert_eq!(before_windows.get(&x_b), after_windows.get(&x_b));
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn bar_filter_mode_none_culls_categories_outside_x_window() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::None,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    table.push_column(Column::F64(vec![1.0, -2.0, 3.0, 0.5]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: -0.5,
            max: 0.5,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(16_384, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    assert_eq!(
        marks.arena.rects.len(),
        1,
        "expected out-of-window categories to be culled for FilterMode::None"
    );
}

#[test]
fn horizontal_bar_emits_rect_batch() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let value_field = crate::ids::FieldId::new(1);
    let category_field = crate::ids::FieldId::new(2);

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
                    id: value_field,
                    column: 0,
                },
                FieldSpec {
                    id: category_field,
                    column: 1,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                }),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: value_field,
                y: category_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, -2.0, 3.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
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
fn stacked_bar_uses_stack_base() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let stack = crate::ids::StackId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into()],
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![1.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 1_024))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;

    fn rect_top_bottom(rect: Rect) -> (f32, f32) {
        let top = rect.origin.y.0;
        let bottom = top + rect.size.height.0;
        (top, bottom)
    }

    let find_rects_for_series =
        |series_id: crate::ids::SeriesId| -> Vec<(u32, Rect)> {
            let Some(node) = marks.nodes.iter().find(|n| {
                n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id)
            }) else {
                return Vec::new();
            };

            let MarkPayloadRef::Rect(rect_ref) = &node.payload else {
                return Vec::new();
            };

            rect_ref
                .rects
                .clone()
                .map(|i| (marks.arena.rect_data_indices[i], marks.arena.rects[i]))
                .collect()
        };

    let rects_a = find_rects_for_series(series_a);
    let rects_b = find_rects_for_series(series_b);
    assert_eq!(rects_a.len(), 2);
    assert_eq!(rects_b.len(), 2);

    let a0 = rects_a.iter().find(|(idx, _)| *idx == 0).map(|(_, r)| *r);
    let b0 = rects_b.iter().find(|(idx, _)| *idx == 0).map(|(_, r)| *r);
    let a0 = a0.unwrap();
    let b0 = b0.unwrap();
    let (a_top, _) = rect_top_bottom(a0);
    let (_, b_bottom) = rect_top_bottom(b0);
    assert!((b_bottom - a_top).abs() < 1.0);
}

#[test]
fn grouped_bars_have_distinct_x_offsets() {
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
            Size::new(Px(320.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into()],
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![1.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 1_024))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;

    let rects_for_series =
        |series_id: crate::ids::SeriesId| -> Vec<(u32, Rect)> {
            let Some(node) = marks.nodes.iter().find(|n| {
                n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id)
            }) else {
                return Vec::new();
            };
            let MarkPayloadRef::Rect(rect_ref) = &node.payload else {
                return Vec::new();
            };
            rect_ref
                .rects
                .clone()
                .map(|i| (marks.arena.rect_data_indices[i], marks.arena.rects[i]))
                .collect()
        };

    let rects_a = rects_for_series(series_a);
    let rects_b = rects_for_series(series_b);
    let a0 = rects_a.iter().find(|(idx, _)| *idx == 0).map(|(_, r)| r);
    let b0 = rects_b.iter().find(|(idx, _)| *idx == 0).map(|(_, r)| r);
    let a0 = a0.unwrap();
    let b0 = b0.unwrap();

    let ax = a0.origin.x.0;
    let bx = b0.origin.x.0;
    assert!((ax - bx).abs() > 5.0);
    assert!(ax < bx);
}

#[test]
fn stacked_and_grouped_bars_share_and_separate_slots() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let stack = crate::ids::StackId::new(1);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let series_c = crate::ids::SeriesId::new(3);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);
    let y_c_field = crate::ids::FieldId::new(4);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
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
                FieldSpec {
                    id: y_c_field,
                    column: 3,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into()],
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_c,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_c_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![1.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 2.0]));
    table.push_column(Column::F64(vec![3.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 1_024))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;

    let center_x_for_series_at_index_0 = |series_id: crate::ids::SeriesId| -> f32 {
        let node = marks
            .nodes
            .iter()
            .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
            .unwrap();
        let MarkPayloadRef::Rect(rect_ref) = &node.payload else {
            panic!("expected rect payload");
        };

        let mut center: Option<f32> = None;
        for i in rect_ref.rects.clone() {
            if marks.arena.rect_data_indices[i] == 0 {
                let rect = marks.arena.rects[i];
                center = Some(rect.origin.x.0 + 0.5 * rect.size.width.0);
                break;
            }
        }
        center.unwrap()
    };

    let ax = center_x_for_series_at_index_0(series_a);
    let bx = center_x_for_series_at_index_0(series_b);
    let cx = center_x_for_series_at_index_0(series_c);

    assert!((ax - bx).abs() < 1.0);
    assert!((ax - cx).abs() > 5.0);
}

#[test]
fn grouped_bars_order_slots_by_first_occurrence_across_stacks() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let stack_1 = crate::ids::StackId::new(1);
    let stack_2 = crate::ids::StackId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let series_c = crate::ids::SeriesId::new(3);
    let series_d = crate::ids::SeriesId::new(4);
    let series_e = crate::ids::SeriesId::new(5);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);
    let y_c_field = crate::ids::FieldId::new(4);
    let y_d_field = crate::ids::FieldId::new(5);
    let y_e_field = crate::ids::FieldId::new(6);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(600.0), Px(240.0)),
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
                FieldSpec {
                    id: y_c_field,
                    column: 3,
                },
                FieldSpec {
                    id: y_d_field,
                    column: 4,
                },
                FieldSpec {
                    id: y_e_field,
                    column: 5,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into()],
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_a_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack_1),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_b_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack_1),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_c,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_c_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack_2),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_d,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_d_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: Some(stack_2),
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_e,
                name: None,
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_e_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![1.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 2.0]));
    table.push_column(Column::F64(vec![3.0, 3.0]));
    table.push_column(Column::F64(vec![4.0, 4.0]));
    table.push_column(Column::F64(vec![5.0, 5.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;

    let center_x_for_series_at_index_0 = |series_id: crate::ids::SeriesId| -> f32 {
        let node = marks
            .nodes
            .iter()
            .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
            .unwrap();
        let MarkPayloadRef::Rect(rect_ref) = &node.payload else {
            panic!("expected rect payload");
        };

        rect_ref
            .rects
            .clone()
            .filter(|&i| marks.arena.rect_data_indices[i] == 0)
            .map(|i| {
                let rect = marks.arena.rects[i];
                rect.origin.x.0 + 0.5 * rect.size.width.0
            })
            .next()
            .unwrap()
    };

    let ax = center_x_for_series_at_index_0(series_a);
    let bx = center_x_for_series_at_index_0(series_b);
    let cx = center_x_for_series_at_index_0(series_c);
    let dx = center_x_for_series_at_index_0(series_d);
    let ex = center_x_for_series_at_index_0(series_e);

    assert!((ax - bx).abs() < 1.0);
    assert!((cx - dx).abs() < 1.0);

    assert!(ax < cx);
    assert!(cx < ex);
    assert!((ax - cx).abs() > 5.0);
    assert!((cx - ex).abs() > 5.0);
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
fn brush_selection_updates_state_without_bumping_view_revision_and_is_exposed_in_output() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    ));

    let mut engine = ChartEngine::new(spec).unwrap();
    let rev = engine.state().revision;

    engine.apply_action(Action::SetBrushSelection2D {
        x_axis,
        y_axis,
        x: DataWindow {
            min: 10.0,
            max: 20.0,
        },
        y: DataWindow {
            min: -5.0,
            max: 5.0,
        },
    });

    assert_eq!(engine.state().revision, rev);
    assert_eq!(
        engine.state().brush_selection_2d,
        Some(crate::engine::BrushSelection2D {
            grid: Some(crate::ids::GridId::new(1)),
            x_axis,
            y_axis,
            x: DataWindow {
                min: 10.0,
                max: 20.0,
            },
            y: DataWindow {
                min: -5.0,
                max: 5.0,
            },
        })
    );

    #[derive(Debug, Default)]
    struct NoopMeasurer;

    impl TextMeasurer for NoopMeasurer {
        fn measure(
            &mut self,
            _text: crate::ids::StringId,
            _style: crate::text::TextStyleId,
        ) -> TextMetrics {
            TextMetrics::default()
        }
    }

    let mut measurer = NoopMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert_eq!(
        engine.output().brush_selection_2d,
        engine.state().brush_selection_2d
    );

    engine.apply_action(Action::ClearBrushSelection);
    assert_eq!(engine.state().revision, rev);
    assert_eq!(engine.state().brush_selection_2d, None);
}

#[test]
fn brush_x_row_range_is_derived_for_matching_series_axes() {
    let series_id = crate::ids::SeriesId::new(1);
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    ));

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| (v * 10) as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetBrushSelection2D {
        x_axis,
        y_axis,
        x: DataWindow { min: 2.0, max: 5.0 },
        y: DataWindow {
            min: -100.0,
            max: 100.0,
        },
    });

    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let range = engine
        .output()
        .brush_x_row_ranges_by_series
        .get(&series_id)
        .copied()
        .expect("expected row range for the brushed series");
    assert_eq!(range, RowRange { start: 2, end: 6 });
}

#[test]
fn brush_selection_is_scoped_to_grid_and_filters_series_ranges() {
    use crate::ids::{AxisId, DatasetId, FieldId, GridId, SeriesId};

    let dataset_id = DatasetId::new(1);
    let x_field = FieldId::new(1);
    let y_field = FieldId::new(2);

    let grid_a = GridId::new(1);
    let grid_b = GridId::new(2);

    let x_a = AxisId::new(1);
    let y_a = AxisId::new(2);
    let x_b = AxisId::new(3);
    let y_b = AxisId::new(4);

    let series_a = SeriesId::new(1);
    let series_b = SeriesId::new(2);

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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_a }, GridSpec { id: grid_b }],
        axes: vec![
            AxisSpec {
                id: x_a,
                name: None,
                kind: AxisKind::X,
                grid: grid_a,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_a,
                name: None,
                kind: AxisKind::Y,
                grid: grid_a,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: x_b,
                name: None,
                kind: AxisKind::X,
                grid: grid_b,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_b,
                name: None,
                kind: AxisKind::Y,
                grid: grid_b,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x_a,
                y_axis: y_a,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x_b,
                y_axis: y_b,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| (v * 10) as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetBrushSelection2D {
        x_axis: x_a,
        y_axis: y_a,
        x: DataWindow { min: 2.0, max: 5.0 },
        y: DataWindow {
            min: -100.0,
            max: 100.0,
        },
    });

    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let selection = engine
        .output()
        .brush_selection_2d
        .expect("expected brush selection in output");
    assert_eq!(selection.grid, Some(grid_a));

    assert!(
        engine
            .output()
            .brush_x_row_ranges_by_series
            .contains_key(&series_a)
    );
    assert!(
        !engine
            .output()
            .brush_x_row_ranges_by_series
            .contains_key(&series_b)
    );

    // Opt-in: allow cross-grid X linking by matching `(dataset, encode.x)` across series.
    engine.apply_action(Action::SetLinkBrushXExportPolicy {
        policy: crate::link::BrushXExportPolicy::SameDatasetXField,
    });
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(
        engine
            .output()
            .brush_x_row_ranges_by_series
            .contains_key(&series_a)
    );
    assert!(
        engine
            .output()
            .brush_x_row_ranges_by_series
            .contains_key(&series_b)
    );

    // Mismatched grids should clear the selection defensively.
    engine.apply_action(Action::SetBrushSelection2D {
        x_axis: x_a,
        y_axis: y_b,
        x: DataWindow { min: 2.0, max: 5.0 },
        y: DataWindow {
            min: -100.0,
            max: 100.0,
        },
    });
    assert_eq!(engine.state().brush_selection_2d, None);
}

#[test]
fn brush_selection_emits_link_event_when_link_group_is_set() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    ));

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| (v * 10) as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetLinkGroup {
        group: Some(crate::ids::LinkGroupId::new(1)),
    });

    engine.apply_action(Action::SetBrushSelection2D {
        x_axis,
        y_axis,
        x: DataWindow { min: 2.0, max: 5.0 },
        y: DataWindow {
            min: -100.0,
            max: 100.0,
        },
    });

    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let events = engine.drain_link_events();
    assert!(events.iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::BrushSelectionChanged { selection }
                if *selection == engine.state().brush_selection_2d
        )
    }));

    // Same selection again should not re-emit the event.
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().is_empty());

    // Clearing selection should emit exactly once.
    engine.apply_action(Action::ClearBrushSelection);
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    let events = engine.drain_link_events();
    assert!(events.iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::BrushSelectionChanged { selection: None }
        )
    }));
}

#[test]
fn axis_pointer_emits_link_event_when_link_group_is_set() {
    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    ));
    spec.axis_pointer = Some(AxisPointerSpec::default());

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| (v * 10) as f64).collect()));
    engine
        .datasets_mut()
        .insert(crate::ids::DatasetId::new(1), table);

    engine.apply_action(Action::SetLinkGroup {
        group: Some(crate::ids::LinkGroupId::new(1)),
    });

    // Hover inside the plot viewport should emit an anchor once.
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(50.0)),
    });
    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let events = engine.drain_link_events();
    assert!(events.iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::AxisPointerChanged {
                anchor: Some(anchor)
            } if anchor.axis == crate::ids::AxisId::new(1)
                && anchor.axis_kind == AxisKind::X
                && anchor.value.is_finite()
        )
    }));

    // Same hover again should not re-emit.
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().is_empty());

    // Moving the hover should emit a new anchor.
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(150.0), Px(50.0)),
    });
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::AxisPointerChanged {
                anchor: Some(anchor)
            } if anchor.axis == crate::ids::AxisId::new(1)
                && anchor.axis_kind == AxisKind::X
                && anchor.value.is_finite()
        )
    }));

    // Leaving the viewport should emit a clear once.
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(10_000.0), Px(10_000.0)),
    });
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::AxisPointerChanged { anchor: None }
        )
    }));
}

#[test]
fn domain_window_emits_link_event_when_link_group_is_set() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    ));

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| (v * 10) as f64).collect()));
    engine
        .datasets_mut()
        .insert(crate::ids::DatasetId::new(1), table);

    engine.apply_action(Action::SetLinkGroup {
        group: Some(crate::ids::LinkGroupId::new(1)),
    });

    let window = DataWindow { min: 2.0, max: 5.0 };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(window),
    });

    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let events = engine.drain_link_events();
    assert!(events.iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::DomainWindowChanged { axis, window: Some(w) }
                if *axis == x_axis && *w == window
        )
    }));

    // Same window again should not re-emit.
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().is_empty());

    // Clearing should emit once.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: None,
    });
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();
    assert!(engine.drain_link_events().iter().any(|e| {
        matches!(
            e,
            crate::link::LinkEvent::DomainWindowChanged { axis, window: None } if *axis == x_axis
        )
    }));
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
fn min_value_span_clamps_interactive_zoom_in() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: Some(5.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base,
        center_px: 50.0,
        log2_scale: 4.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(window.span(), 5.0);
}

#[test]
fn min_value_span_does_not_expand_when_base_is_already_below_min() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: Some(10.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow { min: 0.0, max: 1.0 };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base,
        center_px: 50.0,
        log2_scale: 1.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(window, base);
}

#[test]
fn min_value_span_clamps_slider_handle_updates_without_moving_the_opposite_edge() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: Some(5.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::SetDataWindowXFromZoom {
        axis: x_axis,
        base,
        window: DataWindow {
            min: 19.0,
            max: 20.0,
        },
        anchor: crate::engine::window::WindowSpanAnchor::LockMax,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(
        window,
        DataWindow {
            min: 15.0,
            max: 20.0
        }
    );
}

#[test]
fn min_value_span_is_applied_for_box_zoom_writes() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: Some(5.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base_x = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    let base_y = DataWindow {
        min: -10.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetViewWindow2D {
        x_axis,
        y_axis,
        x: Some(base_x),
        y: Some(base_y),
    });

    engine.apply_action(Action::SetViewWindow2DFromZoom {
        x_axis,
        y_axis,
        base_x,
        base_y,
        x: Some(DataWindow {
            min: 9.0,
            max: 10.0,
        }),
        y: Some(DataWindow {
            min: -1.0,
            max: 1.0,
        }),
    });

    let x = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected x window");
    assert_eq!(x.span(), 5.0);
}

#[test]
fn max_value_span_clamps_interactive_zoom_out() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: Some(50.0),
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base,
        center_px: 50.0,
        log2_scale: -4.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(window.span(), 50.0);
}

#[test]
fn max_value_span_does_not_shrink_when_base_is_already_above_max() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: Some(50.0),
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 100.0,
    };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowXFromBase {
        axis: x_axis,
        base,
        center_px: 50.0,
        log2_scale: -1.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(window, base);
}

#[test]
fn max_value_span_clamps_slider_handle_updates_without_moving_the_opposite_edge() {
    let x_axis = crate::ids::AxisId::new(1);

    let mut spec = basic_spec();
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: Some(10.0),
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow { min: 0.0, max: 8.0 };
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(base),
    });

    engine.apply_action(Action::SetDataWindowXFromZoom {
        axis: x_axis,
        base,
        window: DataWindow {
            min: -20.0,
            max: 8.0,
        },
        anchor: crate::engine::window::WindowSpanAnchor::LockMax,
    });

    let window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected window");
    assert_eq!(
        window,
        DataWindow {
            min: -2.0,
            max: 8.0
        }
    );
}

#[test]
fn min_value_span_clamps_interactive_zoom_in_y() {
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: Some(2.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: -10.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowYFromBase {
        axis: y_axis,
        base,
        center_px: 50.0,
        log2_scale: 6.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert_eq!(window.span(), 2.0);
}

#[test]
fn max_value_span_clamps_interactive_zoom_out_y() {
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: None,
        max_value_span: Some(50.0),
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(base),
    });

    engine.apply_action(Action::ZoomDataWindowYFromBase {
        axis: y_axis,
        base,
        center_px: 50.0,
        log2_scale: -6.0,
        viewport_span_px: 100.0,
    });

    let window = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert_eq!(window.span(), 50.0);
}

#[test]
fn min_value_span_clamps_slider_handle_update_y() {
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: Some(5.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(base),
    });

    engine.apply_action(Action::SetDataWindowYFromZoom {
        axis: y_axis,
        base,
        window: DataWindow {
            min: 19.0,
            max: 20.0,
        },
        anchor: crate::engine::window::WindowSpanAnchor::LockMax,
    });

    let window = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert_eq!(
        window,
        DataWindow {
            min: 15.0,
            max: 20.0
        }
    );
}

#[test]
fn min_value_span_is_applied_for_box_zoom_y_writes() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: Some(5.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base_x = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    let base_y = DataWindow {
        min: -10.0,
        max: 10.0,
    };
    engine.apply_action(Action::SetViewWindow2D {
        x_axis,
        y_axis,
        x: Some(base_x),
        y: Some(base_y),
    });

    engine.apply_action(Action::SetViewWindow2DFromZoom {
        x_axis,
        y_axis,
        base_x,
        base_y,
        x: Some(DataWindow {
            min: 9.0,
            max: 10.0,
        }),
        y: Some(DataWindow { min: 0.0, max: 1.0 }),
    });

    let y = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert_eq!(y.span(), 5.0);
}

#[test]
fn min_value_span_does_not_expand_box_zoom_y_when_base_is_below_min() {
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: Some(10.0),
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let base_x = DataWindow {
        min: 0.0,
        max: 20.0,
    };
    let base_y = DataWindow { min: 0.0, max: 1.0 };
    engine.apply_action(Action::SetViewWindow2D {
        x_axis,
        y_axis,
        x: Some(base_x),
        y: Some(base_y),
    });

    engine.apply_action(Action::SetViewWindow2DFromZoom {
        x_axis,
        y_axis,
        base_x,
        base_y,
        x: Some(DataWindow {
            min: 9.0,
            max: 10.0,
        }),
        y: Some(DataWindow { min: 0.0, max: 0.5 }),
    });

    let y = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert_eq!(y, base_y);
}

#[test]
fn data_zoom_y_filter_mode_filters_scatter_selection_by_y_window_after_x_selection() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let series_id = crate::ids::SeriesId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    ));
    spec.series[0].kind = SeriesKind::Scatter;
    spec.series[0].stack = None;
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((-5..=4).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 2.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let _step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
        .unwrap();

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices for y-filtered scatter selection");
    };
    assert_eq!(&indices[..], &[5, 6, 7]);
}

#[test]
fn engine_stats_track_y_filter_indices_materialization() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let series_id = crate::ids::SeriesId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    ));
    spec.series[0].kind = SeriesKind::Scatter;
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 2.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(1_000_000, 0, 256))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert_eq!(&indices[..], &[2, 3, 4]);

    let stats = engine.stats();
    assert_eq!(stats.filter_plan_runs, 1);
    assert_eq!(stats.filter_plan_grids, 1);
    assert_eq!(stats.filter_plan_steps_run, 5);
    assert_eq!(stats.filter_y_indices_applied_series, 1);
    assert_eq!(stats.filter_x_indices_applied_series, 0);
    assert_eq!(stats.filter_xy_weakfilter_applied_series, 0);
}

#[test]
fn data_zoom_y_filter_mode_filter_ignores_x_window_when_x_filter_mode_none() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::None,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X in [5,9], Y in [0,4].
    // Under x.filterMode=none, Y filtering must ignore the X window (stable raw row space).
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert_eq!(&indices[..], &[0, 1, 2, 3, 4]);

    let stats = engine.stats();
    assert_eq!(stats.filter_x_indices_applied_series, 0);
    assert_eq!(stats.filter_y_indices_applied_series, 1);
}

#[test]
fn data_zoom_y_filter_mode_filter_ignores_x_window_when_x_filter_mode_empty() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X in [5,9], Y in [0,4].
    // Under x.filterMode=empty, X must not cull the row selection space (it is represented as a mask).
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert_eq!(
        &indices[..],
        &[0, 1, 2, 3, 4],
        "expected y filter indices to ignore x window under x=empty"
    );
    assert!(
        participation.empty_mask.x_active,
        "expected x=empty to be represented as an active empty mask"
    );

    let stats = engine.stats();
    assert_eq!(stats.filter_x_indices_applied_series, 0);
    assert_eq!(stats.filter_y_indices_applied_series, 1);
}

#[test]
fn data_zoom_x_filter_mode_empty_masks_scatter_marks_without_culling_row_selection() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X in [5,9], Y in [0,4].
    // Under x.filterMode=empty, X must not cull the row selection space; it is represented as an
    // empty mask. This means we may still select Y rows while emitting zero marks due to X.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert_eq!(&indices[..], &[0, 1, 2, 3, 4]);
    assert!(participation.empty_mask.x_active);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert_eq!(
        points.points.end - points.points.start,
        0,
        "expected x=empty to mask all points for disjoint windows without culling selection"
    );
}

#[test]
fn data_zoom_y_filter_mode_filter_respects_x_window_when_x_filter_mode_filter() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X in [5,9], Y in [0,4].
    // Under x.filterMode=filter, Y filtering must observe the X selection first (x-before-y).
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert!(indices.is_empty(), "expected x-before-y culling to win");
}

#[test]
fn data_zoom_y_filter_mode_filter_respects_x_window_when_x_filter_mode_weakfilter() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X in [5,9], Y in [0,4].
    // v1 policy: X=weakFilter behaves like X=filter here (no XY weakFilter subset),
    // so Y filtering must still observe the X selection first (x-before-y).
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected RowSelection::Indices after y filter materialization");
    };
    assert!(indices.is_empty(), "expected x-before-y culling to win");
}

#[test]
fn data_zoom_xy_filter_mode_filter_applies_x_indices_before_y_indices_in_same_frame() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    let n = 250_001usize;
    let period = 1_000usize;
    let denom = 1_000.0f64;
    let xs: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(xs.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 0.1 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let mut steps = 0u64;
    while steps < 64 {
        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();
        steps += 1;

        let stats = engine.stats();
        if stats.filter_x_indices_applied_series == 0 {
            continue;
        }
        assert_eq!(
            stats.filter_x_indices_applied_series, 1,
            "expected x indices to be applied once"
        );
        assert_eq!(
            stats.filter_y_indices_applied_series, 1,
            "expected y indices to be materialized in the same frame as x indices"
        );
        assert_eq!(
            stats.filter_y_indices_skipped_indices_scan_avoid_series, 0,
            "expected y indices scan to run when x indices were applied in the same frame"
        );

        let Some(participation) = engine.participation().series_participation(series_id) else {
            panic!("expected series participation");
        };
        let RowSelection::Indices(indices) = &participation.selection else {
            panic!("expected RowSelection::Indices after x/y filter materialization");
        };
        assert!(
            indices.is_empty(),
            "expected x-before-y composition to cull disjoint windows"
        );

        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();
        let stats = engine.stats();
        assert_eq!(
            stats.filter_y_indices_applied_series, 1,
            "expected y indices not to be re-materialized for stable indices selections"
        );
        assert_eq!(
            stats.filter_y_indices_skipped_indices_scan_avoid_series, 1,
            "expected y indices scan to be skipped when base selection is indices and x indices were not applied this frame"
        );
        return;
    }

    panic!("expected x indices selection to be materialized within the step loop");
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
fn visual_map_can_emit_stroke_width_for_scatter_buckets() {
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
            Size::new(Px(400.0), Px(300.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![VisualMapSpec {
            id: crate::ids::VisualMapId::new(1),
            mode: VisualMapMode::Continuous,
            dataset: None,
            series: vec![series_id],
            field: y_field,
            domain: (-1.0, 1.0),
            initial_range: None,
            initial_piece_mask: None,
            point_radius_mul_range: Some((0.5, 2.0)),
            stroke_width_range: Some((0.5, 3.0)),
            opacity_mul_range: Some((0.2, 1.0)),
            buckets: 8,
            out_of_range_opacity: 0.25,
        }],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).expect("spec should be valid");

    let n = 4096usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        xs.push(t);
        ys.push((t * std::f64::consts::TAU).sin());
    }

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let mut unfinished = true;
    while unfinished {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .expect("step should succeed");
        unfinished = step.unfinished;
    }

    let marks = &engine.output().marks;
    let has_stroked_bucket = marks.nodes.iter().any(|node| {
        if node.kind != crate::marks::MarkKind::Points {
            return false;
        }
        let MarkPayloadRef::Points(points) = &node.payload else {
            return false;
        };
        points.fill.is_some() && points.stroke.is_some()
    });
    assert!(has_stroked_bucket);
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
        .map(|n| crate::ids::mark_variant(n.id) as u8)
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn category_x_window_updates_axis_window_and_rounds_axis_pointer_value() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..10).map(|i| (i as f64).sin()).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 4.0 }),
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

    let window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((window.min - 1.5).abs() < 1e-6);
    assert!((window.max - 4.5).abs() < 1e-6);

    let indices = &engine.output().marks.arena.data_indices;
    assert!(!indices.is_empty(), "expected marks to contain indices");
    assert!(
        indices.iter().all(|&i| (2..=4).contains(&(i as usize))),
        "expected all indices to be within the configured x window"
    );

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 3.0);
}

#[test]
fn axis_pointer_tooltip_respects_y_empty_mask_for_line_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(1),
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
    table.push_column(Column::F64(vec![5.0, 5.0, 5.0]));
    table.push_column(Column::F64(vec![100.0, 100.0, 100.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };

    let entry_a = axis
        .series
        .iter()
        .find(|e| e.series == series_a)
        .expect("missing series A tooltip entry");
    let entry_b = axis
        .series
        .iter()
        .find(|e| e.series == series_b)
        .expect("missing series B tooltip entry");

    assert!(!matches!(entry_a.value, crate::TooltipSeriesValue::Missing));
    assert!(matches!(entry_b.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn axis_pointer_tooltip_respects_y_empty_mask_for_scatter_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(1),
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Scatter,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Scatter,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
    table.push_column(Column::F64(vec![5.0, 5.0, 5.0]));
    table.push_column(Column::F64(vec![100.0, 100.0, 100.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };

    let entry_a = axis
        .series
        .iter()
        .find(|e| e.series == series_a)
        .expect("missing series A tooltip entry");
    let entry_b = axis
        .series
        .iter()
        .find(|e| e.series == series_b)
        .expect("missing series B tooltip entry");

    assert!(!matches!(entry_a.value, crate::TooltipSeriesValue::Missing));
    assert!(matches!(entry_b.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn axis_pointer_tooltip_respects_y_empty_mask_for_band_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y0_a_field = crate::ids::FieldId::new(2);
    let y1_a_field = crate::ids::FieldId::new(3);
    let y0_b_field = crate::ids::FieldId::new(4);
    let y1_b_field = crate::ids::FieldId::new(5);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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
                    id: y0_a_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_a_field,
                    column: 2,
                },
                FieldSpec {
                    id: y0_b_field,
                    column: 3,
                },
                FieldSpec {
                    id: y1_b_field,
                    column: 4,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(1),
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Band,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y0_a_field,
                    y2: Some(y1_a_field),
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Band,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y0_b_field,
                    y2: Some(y1_b_field),
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs = vec![0.0, 1.0, 2.0];
    let y0_a = vec![-1.0, -2.0, -1.0];
    let y1_a = vec![0.5, 0.8, 0.6];
    let y0_b = vec![-3.0, -3.0, -3.0];
    let y1_b = vec![-2.0, -2.0, -2.0];

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(y0_a));
    table.push_column(Column::F64(y1_a));
    table.push_column(Column::F64(y0_b));
    table.push_column(Column::F64(y1_b));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 1.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 1.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };

    let entry_a = axis
        .series
        .iter()
        .find(|e| e.series == series_a)
        .expect("missing series A tooltip entry");
    let entry_b = axis
        .series
        .iter()
        .find(|e| e.series == series_b)
        .expect("missing series B tooltip entry");

    assert!(
        matches!(
            entry_a.value,
            crate::TooltipSeriesValue::Range { min, max }
                if (min - (-2.0)).abs() < 1e-6 && (max - 0.8).abs() < 1e-6
        ),
        "expected intersecting band interval to be sampled as a range"
    );
    assert!(matches!(entry_b.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn axis_pointer_tooltip_respects_y_empty_mask_under_x_weakfilter_for_scatter_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..=9).map(|v| v as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 4.0, max: 6.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    // Sample at x=3.0: selection contains it (x=weakFilter), but the y-empty mask must treat it as
    // missing (y=3 is outside [4,6]).
    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 3.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 3.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(matches!(entry.value, crate::TooltipSeriesValue::Missing));

    // Sample at x=5.0: inside the y-empty window, so it must be present.
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 5.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 5.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(matches!(entry.value, crate::TooltipSeriesValue::Scalar(v) if (v - 5.0).abs() < 1e-6));
}

#[test]
fn axis_pointer_tooltip_respects_x_empty_mask_when_marks_are_empty_but_selection_is_not() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    // Result: selection is non-empty (y indices), while marks are empty due to the X empty mask.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let participation = engine
        .participation()
        .series_participation(series_id)
        .expect("expected series participation");
    assert!(matches!(participation.selection, RowSelection::Indices(_)));
    assert!(
        participation.empty_mask.x_active,
        "expected x=empty to be represented as an active empty mask"
    );

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert_eq!(
        points.points.end - points.points.start,
        0,
        "expected x=empty to mask scatter marks for disjoint windows"
    );

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 7.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(
        matches!(entry.value, crate::TooltipSeriesValue::Missing),
        "expected axis tooltip sampling to respect the X empty mask"
    );
}

#[test]
fn axis_pointer_tooltip_respects_x_empty_mask_under_y_filtered_selection_for_line_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    // Result: selection is non-empty (y indices), while marks are empty due to the X empty mask.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let participation = engine
        .participation()
        .series_participation(series_id)
        .expect("expected series participation");
    assert!(matches!(participation.selection, RowSelection::Indices(_)));
    assert!(
        participation.empty_mask.x_active,
        "expected x=empty to be represented as an active empty mask"
    );

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 7.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(
        matches!(entry.value, crate::TooltipSeriesValue::Missing),
        "expected axis tooltip sampling to respect the X empty mask"
    );
}

#[test]
fn axis_pointer_tooltip_respects_x_empty_mask_under_y_filtered_selection_for_band_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((1..=10).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    // Result: selection is non-empty (y indices), while marks are empty due to the X empty mask.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let participation = engine
        .participation()
        .series_participation(series_id)
        .expect("expected series participation");
    assert!(matches!(participation.selection, RowSelection::Indices(_)));
    assert!(
        participation.empty_mask.x_active,
        "expected x=empty to be represented as an active empty mask"
    );

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 7.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(
        matches!(entry.value, crate::TooltipSeriesValue::Missing),
        "expected axis tooltip sampling to respect the X empty mask"
    );
}

#[test]
fn axis_pointer_item_trigger_returns_none_when_marks_are_empty_under_x_empty_mask() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 4.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().hover.is_none(),
        "expected hover hit-test to return none when marks are empty"
    );
    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when no hit is possible"
    );
}

#[test]
fn axis_pointer_item_trigger_returns_none_when_line_marks_are_empty_under_x_empty_mask() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 4.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().hover.is_none(),
        "expected hover hit-test to return none when marks are empty"
    );
    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when no hit is possible"
    );
}

#[test]
fn axis_pointer_item_trigger_returns_none_when_band_marks_are_empty_under_x_empty_mask() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 4.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|v| v as f64).collect()));
    table.push_column(Column::F64((1..=10).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows:
    // - X is `Empty` and represented as a mask (x window [5,9])
    // - Y is `Filter` and materializes indices (y window [0,4])
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().hover.is_none(),
        "expected hover hit-test to return none when marks are empty"
    );
    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when no hit is possible"
    );
}

#[test]
fn axis_pointer_item_trigger_is_suppressed_for_y_empty_masked_line_samples() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..=9).map(|v| v as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 4.0, max: 6.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();

    // Sanity: an in-window sample is hittable.
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 5.0, viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 5.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);
    assert!(
        engine.output().axis_pointer.is_some(),
        "expected item-trigger axis pointer output to be present for an in-window hit"
    );

    // Masked sample (y=3 is outside the y-empty window [4,6]) must not leak into hit-test via
    // clamping-to-bounds or line interpolation.
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 3.0, viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 4.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected masked samples under y.filterMode=Empty to be non-hittable for item-trigger axis pointer"
    );
}

#[test]
fn axis_pointer_item_trigger_does_not_hit_clamped_y_empty_gap_for_line_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    max: 1000.0,
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![500.0, 501.0, 502.0, 503.0, 504.0, 505.0]));
    table.push_column(Column::F64(vec![5.0, 5.0, 5.0, 3.0, 5.0, 5.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 4.0, max: 6.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();

    // Sanity: in-window points are hittable.
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 501.0, viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 5.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);
    assert!(
        engine.output().axis_pointer.is_some(),
        "expected item-trigger axis pointer output to be present for an in-window hit"
    );
    assert!(
        engine.output().hover.is_some(),
        "expected output.hover to be present for an in-window hit"
    );

    // The out-of-window sample at x=503.0 (y=3.0) is masked under `y.filterMode=Empty`, and must
    // behave as a gap. If it were erroneously clamped to y=4.0 and left in the polyline, we'd be
    // able to hit a segment on the y=4 boundary here.
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 503.0, viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 4.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when hovering over a y-empty gap"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be none when hovering over a y-empty gap"
    );
}

#[test]
fn axis_pointer_tooltip_respects_y_empty_mask_for_bar_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into(), "C".into()],
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
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(1),
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
    table.push_column(Column::F64(vec![5.0, 5.0, 5.0]));
    table.push_column(Column::F64(vec![100.0, 100.0, 100.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };

    let entry_a = axis
        .series
        .iter()
        .find(|e| e.series == series_a)
        .expect("missing series A tooltip entry");
    let entry_b = axis
        .series
        .iter()
        .find(|e| e.series == series_b)
        .expect("missing series B tooltip entry");

    assert!(!matches!(entry_a.value, crate::TooltipSeriesValue::Missing));
    assert!(matches!(entry_b.value, crate::TooltipSeriesValue::Missing));
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn set_series_visibility_batch_bumps_visual_revision_once() {
    let mut spec = basic_spec();
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);

    spec.series.push(SeriesSpec {
        id: series_b,
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
        bar_layout: Default::default(),
        area_baseline: None,
        lod: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();
    let before = engine.model().revs.visual.0;

    engine.apply_action(Action::SetSeriesVisibility {
        updates: vec![(series_a, false), (series_b, false)],
    });

    assert!(!engine.model().series.get(&series_a).unwrap().visible);
    assert!(!engine.model().series.get(&series_b).unwrap().visible);
    assert_eq!(engine.model().revs.visual.0, before + 1);
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn data_zoom_x_filter_mode_empty_preserves_base_row_selection_for_monotonic_x() {
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..100).map(|i| i as f64).collect()));
    table.push_column(Column::F64((0..100).map(|i| i as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 20.0,
            max: 40.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    assert_eq!(
        view.selection,
        RowSelection::Range(RowRange { start: 0, end: 100 }),
        "expected FilterMode::Empty to preserve the base row selection"
    );
    assert_eq!(view.x_policy.filter.min, Some(20.0));
    assert_eq!(view.x_policy.filter.max, Some(40.0));
}

#[test]
fn filter_mode_empty_line_marks_respect_indices_selection_from_y_filter() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(2),
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|i| i as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|i| (i % 2) as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // X window culls points outside [2,8] (but does not change the base selection under Empty).
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    // Y window filters out the odd points (materializing a sparse selection in the view layer).
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 0.1 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut emitted_indices: Vec<u32> = Vec::new();
    for node in &marks.nodes {
        if node.kind != MarkKind::Polyline || node.source_series != Some(series_id) {
            continue;
        }
        let MarkPayloadRef::Polyline(p) = &node.payload else {
            continue;
        };
        emitted_indices.extend_from_slice(&marks.arena.data_indices[p.points.clone()]);
    }

    assert!(!emitted_indices.is_empty(), "expected some polyline points");
    for raw in emitted_indices {
        let x = raw as usize;
        assert!(
            (2..=8).contains(&x),
            "expected X window to cull outside points"
        );
        assert_eq!((x % 2) as u32, 0, "expected Y filter to remove odd points");
    }
}

#[test]
fn filter_mode_empty_does_not_cull_y_filtered_row_selection_by_x_window() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(2),
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=9).map(|i| i as f64).collect()));
    table.push_column(Column::F64((0..=9).map(|i| (i % 2) as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 0.1 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let participation = engine
        .participation()
        .series_participation(series_id)
        .expect("expected series participation");
    assert_eq!(
        participation.x_filter_mode,
        FilterMode::Empty,
        "expected X to remain in Empty mode"
    );
    assert!(
        participation.x_policy.filter.min.is_some() || participation.x_policy.filter.max.is_some(),
        "expected X empty mask predicate to remain active"
    );

    let RowSelection::Indices(indices) = &participation.selection else {
        panic!("expected indices-backed selection for Y filter");
    };
    assert_eq!(
        &indices[..],
        &[0, 2, 4, 6, 8],
        "expected Y filter indices to preserve X-empty row space"
    );
}

#[test]
fn data_zoom_x_filter_mode_weakfilter_matches_filter_for_monotonic_x() {
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64((0..100).map(|i| i as f64).collect()));
    table.push_column(Column::F64((0..100).map(|i| i as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 20.0,
            max: 40.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    assert_eq!(
        view.selection,
        RowSelection::Range(RowRange { start: 20, end: 41 }),
        "expected WeakFilter to match Filter semantics in v1"
    );
    assert_eq!(view.x_policy.filter.min, Some(20.0));
    assert_eq!(view.x_policy.filter.max, Some(40.0));
}

#[test]
fn data_zoom_xy_filter_mode_weakfilter_drops_only_same_side_outliers() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(2),
            axis: y_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    // X is monotonic and spans outside the window on both sides. Y also spans outside.
    //
    // Window: x in [0,10], y in [0,10]
    // Points:
    // - idx0: x below, y inside -> keep (weakFilter keeps mixed/outside points)
    // - idx1: x below, y below  -> drop (same-side below)
    // - idx2: x inside, y inside -> keep
    // - idx3: x inside, y above  -> keep
    // - idx4: x inside, y inside -> keep
    // - idx5: x above, y above   -> drop (same-side above)
    table.push_column(Column::F64(vec![-100.0, -50.0, 0.0, 5.0, 10.0, 20.0]));
    table.push_column(Column::F64(vec![5.0, -100.0, 5.0, 20.0, 5.0, 20.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    let RowSelection::Indices(indices) = &view.selection else {
        panic!("expected indices selection for multi-dim weakFilter");
    };
    assert_eq!(indices.as_ref(), &[0, 2, 3, 4]);
}

#[test]
fn data_zoom_xy_filter_mode_weakfilter_prefers_xy_indices_over_x_only_indices_for_large_non_monotonic_views()
 {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    // Non-monotonic X (sawtooth) ensures an X-only indices view is eligible. With XY weakFilter we
    // must prefer the XY indices selection instead, because X-only slicing cannot represent the
    // semantics.
    //
    // Window: x in [0.2, 0.8], y in [0.2, 0.8]
    // - y=0 is always below, y=1 is always above
    // - weakFilter drops only same-side outliers (Below/Below and Above/Above)
    let n = 60_001usize;
    let period = 1000usize;
    let denom = 1000.0f64;
    let xs: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    let ys: Vec<f64> = (0..n).map(|i| if i % 2 == 0 { 0.0 } else { 1.0 }).collect();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..64 {
        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();

        let Some(participation) = engine.participation().series_participation(series_id) else {
            continue;
        };
        let RowSelection::Indices(indices) = &participation.selection else {
            continue;
        };

        assert_eq!(
            participation.x_policy.filter,
            Default::default(),
            "expected x filter predicate to be cleared for XY weakFilter"
        );

        assert_eq!(indices.len(), 48_000);
        assert_eq!(indices[0], 1);
        assert_eq!(indices[1], 3);
        assert_eq!(indices[2], 5);
        assert_eq!(indices[399], 499);
        assert_eq!(indices[400], 500);
        assert_eq!(indices[401], 501);
        assert_eq!(indices[indices.len() - 1], 59_998);

        let stats = engine.stats();
        assert_eq!(stats.filter_x_indices_applied_series, 0);
        assert_eq!(stats.filter_y_indices_applied_series, 0);
        assert_eq!(stats.filter_xy_weakfilter_applied_series, 1);
        return;
    }

    panic!("expected xy weakFilter indices selection to be materialized within the step loop");
}

#[test]
fn data_zoom_xy_filter_mode_weakfilter_keeps_mixed_side_outliers_for_scatter() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    // Window: x in [0,1], y in [0,1]
    // - idx0: x below, y below -> drop (Below/Below)
    // - idx1: x inside, y below -> keep (mixed)
    // - idx2: x above, y above -> drop (Above/Above)
    // - idx3: x below, y above -> keep (mixed)
    // - idx4: x above, y below -> keep (mixed)
    // - idx5: x inside, y inside -> keep
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![-1.0, 0.5, 1.5, -1.0, 1.5, 0.5]));
    table.push_column(Column::F64(vec![-1.0, -1.0, 1.5, 1.5, -1.0, 0.5]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 0.0, max: 1.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 1.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        assert!(!step.unfinished);

        let Some(participation) = engine.participation().series_participation(series_id) else {
            continue;
        };
        let RowSelection::Indices(indices) = &participation.selection else {
            continue;
        };

        assert!(!participation.empty_mask.x_active);
        assert!(!participation.empty_mask.y_active);
        assert_eq!(
            participation.x_policy.filter,
            Default::default(),
            "expected x filter predicate to be cleared for XY weakFilter"
        );
        assert_eq!(
            &indices[..],
            &[1, 3, 4, 5],
            "expected weakFilter to drop only same-side outliers (Below/Below, Above/Above)"
        );

        let stats = engine.stats();
        assert_eq!(stats.filter_xy_weakfilter_applied_series, 1);
        return;
    }

    panic!("expected indices selection for multi-dim weakFilter");
}

#[test]
fn data_zoom_xy_filter_mode_weakfilter_drops_only_same_side_outliers_for_band() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id_x = crate::ids::DataZoomId::new(1);
    let zoom_id_y = crate::ids::DataZoomId::new(2);
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_id_x,
            axis: x_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id_y,
            axis: y_axis,
            filter_mode: FilterMode::WeakFilter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    // Window: x in [0,10], y in [0,10].
    // Same-side rule drops:
    // - idx1: x below, y interval below
    // - idx5: x above, y interval above
    table.push_column(Column::F64(vec![-100.0, -50.0, 0.0, 5.0, 10.0, 20.0]));
    table.push_column(Column::F64(vec![5.0, -100.0, -100.0, 20.0, 5.0, 20.0]));
    table.push_column(Column::F64(vec![6.0, -50.0, -50.0, 25.0, 6.0, 25.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    let RowSelection::Indices(indices) = &view.selection else {
        panic!("expected indices selection for multi-dim weakFilter");
    };
    assert_eq!(indices.as_ref(), &[0, 2, 3, 4]);
}

#[test]
fn data_zoom_x_filter_mode_none_vs_filter_vs_empty_y_axis_window_semantics() {
    fn y_window_for_filter_mode(mode: FilterMode) -> DataWindow {
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
                Size::new(Px(320.0), Px(200.0)),
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

                from: None,
                transforms: Vec::new(),
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
                id: crate::ids::DataZoomId::new(1),
                axis: x_axis,
                filter_mode: mode,
                min_value_span: None,
                max_value_span: None,
            }],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        };

        let mut engine = ChartEngine::new(spec).unwrap();
        let mut table = DataTable::default();

        let n = 100usize;
        let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let ys: Vec<f64> = (0..n)
            .map(|i| {
                if (20..=40).contains(&i) {
                    i as f64
                } else {
                    1000.0
                }
            })
            .collect();
        table.push_column(Column::F64(xs));
        table.push_column(Column::F64(ys));
        engine.datasets_mut().insert(dataset_id, table);

        engine.apply_action(Action::SetDataWindowX {
            axis: x_axis,
            window: Some(DataWindow {
                min: 20.0,
                max: 40.0,
            }),
        });

        let mut measurer = NullTextMeasurer::default();
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        assert!(!step.unfinished);

        engine
            .output()
            .axis_windows
            .get(&y_axis)
            .copied()
            .expect("expected y axis window")
    }

    // ECharts-class semantics:
    // - `none`: do not scope the Y axis extent to the X window (global bounds).
    // - `filter`: scope Y bounds to the X window.
    // - `empty`: also scope Y bounds to the X window (bounds ignore missing), while keeping a
    //   stable row space.
    let y_none = y_window_for_filter_mode(FilterMode::None);
    assert!(
        y_none.max > 900.0,
        "expected global Y bounds under none: {y_none:?}"
    );

    let y_filter = y_window_for_filter_mode(FilterMode::Filter);
    assert!(
        y_filter.max < 200.0,
        "expected Y bounds scoped to X window under filter: {y_filter:?}"
    );

    let y_empty = y_window_for_filter_mode(FilterMode::Empty);
    assert!(
        y_empty.max < 200.0,
        "expected Y bounds scoped to X window under empty: {y_empty:?}"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_breaks_line_into_segments_for_interleaved_out_of_window_points() {
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![
        1.0, 2.0, 100.0, 3.0, 4.0, 200.0, 5.0, 6.0,
    ]));
    table.push_column(Column::F64(vec![1.0, 2.0, 1.0, 3.0, 4.0, 1.0, 5.0, 6.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut segments: BTreeMap<u64, Range<usize>> = BTreeMap::new();
    for node in &marks.nodes {
        if node.kind != crate::marks::MarkKind::Polyline || node.source_series != Some(series_id) {
            continue;
        }
        let MarkPayloadRef::Polyline(poly) = &node.payload else {
            continue;
        };
        segments.insert(crate::ids::mark_variant(node.id), poly.points.clone());
    }

    assert_eq!(segments.len(), 3, "expected three polyline segments");
    let expected = [
        (0u64, vec![0u32, 1u32]),
        (1u64, vec![3u32, 4u32]),
        (2u64, vec![6u32, 7u32]),
    ];
    for (variant, indices) in expected {
        let range = segments.get(&variant).expect("missing segment variant");
        let actual = &marks.arena.data_indices[range.clone()];
        assert_eq!(actual, &indices[..]);
    }
}

#[test]
fn data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_line_marks_are_empty() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    // Keep selection non-empty but make the x-empty window fully disjoint so marks emit nothing.
    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    // Provide a stable Y axis window so bounds/axis windows do not fall back to empty-data defaults.
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert!(participation.empty_mask.x_active);
    assert_eq!(participation.selection.view_len(10), 5);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 5.0).abs() < 1e-6);
    assert!((x_window.max - 9.0).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 0.0).abs() < 1e-6);
    assert!((y_window.max - 10.0).abs() < 1e-6);

    assert!(
        !engine
            .output()
            .marks
            .nodes
            .iter()
            .any(|n| n.source_series == Some(series_id)),
        "expected no marks to be emitted under x-empty disjoint mask"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_with_y_filter_keeps_axis_windows_stable_when_line_marks_are_empty()
{
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 1.0, max: 3.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert!(participation.empty_mask.x_active);
    assert_eq!(
        participation.selection,
        RowSelection::Indices(vec![1, 2, 3].into()),
        "expected y=filter to cull selection even when x is represented as an empty mask"
    );

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 5.0).abs() < 1e-6);
    assert!((x_window.max - 9.0).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 1.0).abs() < 1e-6);
    assert!((y_window.max - 3.0).abs() < 1e-6);

    assert!(
        !engine
            .output()
            .marks
            .nodes
            .iter()
            .any(|n| n.source_series == Some(series_id)),
        "expected no marks to be emitted when x-empty mask is fully disjoint"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_band_marks_are_empty() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y0: Vec<f64> = xs.iter().map(|_| 0.0).collect();
    let y1: Vec<f64> = xs.iter().map(|_| 1.0).collect();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(y0));
    table.push_column(Column::F64(y1));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert!(participation.empty_mask.x_active);
    assert_eq!(participation.selection.view_len(10), 5);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 5.0).abs() < 1e-6);
    assert!((x_window.max - 9.0).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 0.0).abs() < 1e-6);
    assert!((y_window.max - 10.0).abs() < 1e-6);

    assert!(
        !engine
            .output()
            .marks
            .nodes
            .iter()
            .any(|n| n.source_series == Some(series_id)),
        "expected no marks to be emitted under x-empty disjoint mask"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_with_y_filter_keeps_axis_windows_stable_when_band_marks_are_empty()
{
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let y0: Vec<f64> = xs.clone();
    let y1: Vec<f64> = xs.iter().map(|v| v + 1.0).collect();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(y0));
    table.push_column(Column::F64(y1));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 1.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert!(participation.empty_mask.x_active);
    assert_eq!(
        participation.selection,
        RowSelection::Indices(vec![0, 1].into()),
        "expected y=filter to cull band rows by interval intersection"
    );

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 5.0).abs() < 1e-6);
    assert!((x_window.max - 9.0).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 0.0).abs() < 1e-6);
    assert!((y_window.max - 1.0).abs() < 1e-6);

    assert!(
        !engine
            .output()
            .marks
            .nodes
            .iter()
            .any(|n| n.source_series == Some(series_id)),
        "expected no marks to be emitted when x-empty mask is fully disjoint"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_scatter_lod_marks_are_empty() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let n = 50_000usize;
    let xs: Vec<f64> = (0..n).map(|i| i as f64 / (n as f64 - 1.0)).collect();
    let ys: Vec<f64> = (0..n).map(|_| 0.5).collect();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: n }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert!(participation.empty_mask.x_active);
    assert_eq!(participation.selection.view_len(n), n);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 5.0).abs() < 1e-6);
    assert!((x_window.max - 9.0).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 0.0).abs() < 1e-6);
    assert!((y_window.max - 10.0).abs() < 1e-6);

    assert!(
        !engine
            .output()
            .marks
            .nodes
            .iter()
            .any(|n| n.source_series == Some(series_id)),
        "expected no marks to be emitted under x-empty disjoint window"
    );
}

#[test]
fn data_zoom_y_filter_mode_empty_breaks_line_into_segments_for_interleaved_out_of_window_points() {
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]));
    table.push_column(Column::F64(vec![
        1.0, 2.0, 100.0, 3.0, 4.0, 200.0, 5.0, 6.0,
    ]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut segments: BTreeMap<u64, Range<usize>> = BTreeMap::new();
    for node in &marks.nodes {
        if node.kind != crate::marks::MarkKind::Polyline || node.source_series != Some(series_id) {
            continue;
        }
        let MarkPayloadRef::Polyline(poly) = &node.payload else {
            continue;
        };
        segments.insert(crate::ids::mark_variant(node.id), poly.points.clone());
    }

    assert_eq!(segments.len(), 3, "expected three polyline segments");
    let expected = [
        (0u64, vec![0u32, 1u32]),
        (1u64, vec![3u32, 4u32]),
        (2u64, vec![6u32, 7u32]),
    ];
    for (variant, indices) in expected {
        let range = segments.get(&variant).expect("missing segment variant");
        let actual = &marks.arena.data_indices[range.clone()];
        assert_eq!(actual, &indices[..]);
    }
}

#[test]
fn data_zoom_y_filter_mode_empty_sets_empty_mask_without_culling_row_selection() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    ));
    spec.series[0].kind = SeriesKind::Line;
    spec.series[0].stack = None;
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::Empty,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..=7).map(|v| v as f64).collect();
    let ys: Vec<f64> = vec![1.0, 2.0, 100.0, 3.0, 4.0, 200.0, 5.0, 6.0];

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(ys.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 1.0, max: 5.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert!(
        matches!(participation.selection, RowSelection::Range(_)),
        "expected Y Empty to preserve the row selection space (no indices materialization)"
    );
    assert_eq!(participation.x_filter_mode, FilterMode::Filter);
    assert_eq!(participation.y_filter_mode, FilterMode::Empty);

    let mask = participation.empty_mask;
    assert!(
        !mask.x_active,
        "expected X Filter not to activate empty masking"
    );
    assert!(mask.y_active, "expected Y Empty to activate empty masking");
    assert!(mask.allows_raw_index(1, &xs, &ys, None));
    assert!(!mask.allows_raw_index(2, &xs, &ys, None));
    assert!(mask.allows_raw_index(3, &xs, &ys, None));
    assert!(!mask.allows_raw_index(5, &xs, &ys, None));
}

#[test]
fn data_zoom_y_filter_mode_empty_masks_scatter_marks_without_culling_x_selection_under_x_weakfilter()
 {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    ));
    spec.series[0].kind = SeriesKind::Scatter;
    spec.series[0].stack = None;
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::WeakFilter,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::Empty,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..=9).map(|v| v as f64).collect();
    let ys: Vec<f64> = xs.clone();

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(ys.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    // Disjoint windows: X selects [5,9], Y masks [0,4].
    // Under y.filterMode=empty, Y must not cull the row selection space; it is represented as an
    // empty mask. Under x.filterMode=weakFilter (without the XY subset), X behaves like Filter.
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::WeakFilter);
    assert_eq!(participation.y_filter_mode, FilterMode::Empty);
    assert_eq!(
        participation.selection,
        RowSelection::Range(RowRange { start: 5, end: 10 }),
        "expected Y Empty to preserve the X-filtered row selection space"
    );

    let mask = participation.empty_mask;
    assert!(!mask.x_active);
    assert!(mask.y_active);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert_eq!(
        points.points.end - points.points.start,
        0,
        "expected y=empty to mask all points for disjoint windows without culling selection"
    );
}

#[test]
fn data_zoom_y_filter_mode_empty_masks_scatter_marks_within_window_under_x_weakfilter() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    ));
    spec.series[0].kind = SeriesKind::Scatter;
    spec.series[0].stack = None;
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::WeakFilter,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::Empty,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..=9).map(|v| v as f64).collect();
    let ys: Vec<f64> = xs.clone();

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(ys.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    // Partially overlapping windows:
    // - X selects [2,8]
    // - Y empty masks [4,6]
    // Expected:
    // - selection is the X-filtered row space
    // - marks are emitted only for the Y-window subset
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 4.0, max: 6.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(
        participation.selection,
        RowSelection::Range(RowRange { start: 2, end: 9 }),
        "expected Y Empty to preserve the X-filtered row selection space"
    );

    let mask = participation.empty_mask;
    assert!(!mask.x_active);
    assert!(mask.y_active);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert_eq!(points.points.end - points.points.start, 3);

    let indices = &marks.arena.data_indices[points.points.clone()];
    assert_eq!(
        indices,
        &[4, 5, 6],
        "expected y=empty to mask points outside the y window while keeping selection intact"
    );
}

#[test]
fn data_zoom_y_filter_mode_empty_keeps_band_visible_when_interval_intersects_window() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs = vec![0.0, 1.0, 2.0];
    let y0 = vec![-1.0, -2.0, -1.0];
    let y1 = vec![0.5, 0.8, 0.6];

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(y0.clone()));
    table.push_column(Column::F64(y1.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 1.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    let mask = participation.empty_mask;
    assert!(mask.y_active, "expected Y Empty to activate empty masking");
    for i in 0..xs.len() {
        assert!(
            mask.allows_raw_index(i, &xs, &y0, Some(&y1)),
            "expected band interval to be visible when it intersects the window"
        );
    }

    let marks = &engine.output().marks;
    assert!(
        marks
            .nodes
            .iter()
            .any(|n| n.kind == crate::marks::MarkKind::Polyline
                && n.source_series == Some(series_id)),
        "expected band marks to be emitted when intervals intersect the Y window"
    );
}

#[test]
fn data_zoom_y_filter_mode_filter_culls_band_rows_by_interval_intersection() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..=9).map(|v| v as f64).collect();
    let y0: Vec<f64> = xs.clone();
    let y1: Vec<f64> = xs.iter().map(|v| v + 1.0).collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(y0));
    table.push_column(Column::F64(y1));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 4.0, max: 6.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };

    assert_eq!(
        participation.selection,
        RowSelection::Indices(vec![3, 4, 5, 6].into()),
        "expected y=filter to cull the row participation space for bands by interval intersection"
    );
    assert!(!participation.empty_mask.y_active);
}

#[test]
fn data_zoom_y_filter_mode_empty_masks_bar_marks_outside_window() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let value_field = crate::ids::FieldId::new(1);
    let category_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: value_field,
                    column: 0,
                },
                FieldSpec {
                    id: category_field,
                    column: 1,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: category_field,
                y: value_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, 100.0, 3.0, 4.0]));
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected rect marks for bar series");
    let MarkPayloadRef::Rect(r) = &node.payload else {
        panic!("expected rect payload");
    };
    let indices = &marks.arena.rect_data_indices[r.rects.clone()];
    assert!(
        !indices.contains(&1),
        "expected y-empty to mask the out-of-window bar sample"
    );
}

#[test]
fn data_zoom_x_filter_mode_empty_breaks_band_into_segments_for_interleaved_out_of_window_points() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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
            Size::new(Px(240.0), Px(160.0)),
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![
        1.0, 2.0, 100.0, 3.0, 4.0, 200.0, 5.0, 6.0,
    ]));
    table.push_column(Column::F64(vec![1.0, 2.0, 1.0, 3.0, 4.0, 1.0, 5.0, 6.0]));
    table.push_column(Column::F64(vec![2.0, 3.0, 2.0, 4.0, 5.0, 2.0, 6.0, 7.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut variants: Vec<u64> = Vec::new();
    let mut lower: BTreeMap<u64, Range<usize>> = BTreeMap::new();
    let mut upper: BTreeMap<u64, Range<usize>> = BTreeMap::new();

    for node in &marks.nodes {
        if node.kind != crate::marks::MarkKind::Polyline || node.source_series != Some(series_id) {
            continue;
        }
        let MarkPayloadRef::Polyline(poly) = &node.payload else {
            continue;
        };
        let v = crate::ids::mark_variant(node.id);
        variants.push(v);
        if v % 2 == 1 {
            lower.insert(v, poly.points.clone());
        } else {
            upper.insert(v, poly.points.clone());
        }
    }

    variants.sort_unstable();
    assert_eq!(variants, vec![1, 2, 3, 4, 5, 6]);

    let expected = [
        (1u64, vec![0u32, 1u32]),
        (3u64, vec![3u32, 4u32]),
        (5u64, vec![6u32, 7u32]),
    ];
    for (variant, indices) in expected {
        let range = lower.get(&variant).expect("missing lower segment variant");
        let actual = &marks.arena.data_indices[range.clone()];
        assert_eq!(actual, &indices[..]);

        let range = upper
            .get(&(variant + 1))
            .expect("missing upper segment variant");
        let actual = &marks.arena.data_indices[range.clone()];
        assert_eq!(actual, &indices[..]);
    }
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

            from: None,
            transforms: Vec::new(),
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
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec::default()),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 50.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
    let crate::TooltipOutput::Item(item) = &axis_pointer.tooltip else {
        panic!("expected item-trigger tooltip payload");
    };
    assert_eq!(item.series, series_id);
    assert_eq!(item.x_axis, x_axis);
    assert_eq!(item.y_axis, y_axis);
    assert!(item.data_index < 2);
    assert!(item.x_value.is_finite());
    assert!(item.y_value.is_finite());
}

#[test]
fn axis_pointer_item_trigger_is_suppressed_when_far_from_series() {
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
            Size::new(Px(1000.0), Px(1000.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 1.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
        point: Point::new(Px(500.0), Px(100.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    assert!(engine.output().axis_pointer.is_none());
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be gated by item-trigger distance"
    );
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 0.5);
    assert_eq!(axis.series.len(), 2);
    assert_eq!(axis.series[0].series, series_a);
    assert_eq!(axis.series[1].series, series_b);

    let crate::TooltipSeriesValue::Scalar(a) = axis.series[0].value else {
        panic!("expected scalar tooltip value for series A");
    };
    assert_eq!(a, 0.5);
    let crate::TooltipSeriesValue::Scalar(b) = axis.series[1].value else {
        panic!("expected scalar tooltip value for series B");
    };
    assert_eq!(b, 1.0);
}

#[test]
fn output_hover_is_gated_by_axis_trigger_marker_distance() {
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
            Size::new(Px(1000.0), Px(1000.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 1.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
        point: Point::new(Px(500.0), Px(100.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_some(),
        "expected axis-trigger axis pointer output to remain active"
    );
    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert!(
        axis_pointer.hit.is_none(),
        "expected axis-trigger marker hit to be gated by trigger distance"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to match the gated marker hit for axis-trigger"
    );
}

#[test]
fn axis_pointer_axis_trigger_respects_x_filter_for_non_monotonic_x() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    // Non-monotonic X: the in-window candidate (x=5) is farther from axis_value=9.9 than the
    // out-of-window point (x=10.1). `FilterMode::Filter` must exclude the out-of-window point.
    table.push_column(Column::F64(vec![0.0, 10.1, 5.0]));
    table.push_column(Column::F64(vec![0.0, 10.1, 5.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(99.0), Px(50.0)),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };

    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert!((axis.axis_value - 9.9).abs() < 1e-6);
    assert_eq!(axis.series.len(), 1);

    let crate::TooltipSeriesValue::Scalar(v) = axis.series[0].value else {
        panic!("expected scalar tooltip value");
    };
    assert_eq!(v, 5.0);
}

#[test]
fn axis_pointer_axis_trigger_emits_range_for_band_series() {
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![2.0, 3.0]));
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
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 0.5);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);

    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Range { min: 0.5, max: 2.5 }
    );
}

#[test]
fn axis_pointer_axis_trigger_samples_scatter_by_nearest_point() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 10.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(75.0), Px(50.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 0.75);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);
    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Scalar(10.0)
    );
}

#[test]
fn axis_pointer_axis_trigger_handles_indices_selection_from_y_filter() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: crate::ids::DataZoomId::new(2),
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Scatter,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Scatter,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64((0..=10).map(|v| v as f64).collect()));
    table.push_column(Column::F64(
        (0..=10)
            .map(|i| if i % 2 == 0 { 0.0 } else { 1.0 })
            .collect(),
    ));
    table.push_column(Column::F64((0..=10).map(|i| 100.0 + i as f64).collect()));
    engine.datasets_mut().insert(dataset_id, table);

    // X slices to [2,8] first, then Y filter materializes an indices selection for scatter A:
    // x=[2..=8], y alternates 0/1 -> indices = [2,4,6,8].
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 8.0 }),
    });
    // Keep only the y=0 samples for series A; series B has no in-window values and becomes empty.
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 0.1 }),
    });

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(50.0)),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert_eq!(axis_pointer.crosshair_px, Point::new(Px(50.0), Px(50.0)));

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert!((axis.axis_value - 5.0).abs() < 1e-6);

    assert_eq!(axis.series.len(), 2);
    assert_eq!(axis.series[0].series, series_a);
    assert_eq!(axis.series[1].series, series_b);

    assert!(matches!(axis.series[0].value, crate::TooltipSeriesValue::Scalar(v) if v == 0.0));
    assert!(matches!(
        axis.series[1].value,
        crate::TooltipSeriesValue::Missing
    ));
}

#[test]
fn axis_pointer_axis_trigger_uses_nearest_x_index_for_large_non_monotonic_views() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 200_001usize;
    let period = 1001usize;
    let denom = 1000.0f64;
    let xs: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    let ys: Vec<f64> = (0..n).map(|i| i as f64).collect();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(50.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(
        step.unfinished,
        "expected the nearest-x index to build over multiple steps"
    );

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].value, crate::TooltipSeriesValue::Missing);

    let mut value = crate::TooltipSeriesValue::Missing;
    for _ in 0..64 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
            .unwrap();
        let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
        let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
            panic!("expected axis-trigger tooltip payload");
        };
        value = axis.series[0].value.clone();
        if !step.unfinished {
            break;
        }
    }

    assert_eq!(value, crate::TooltipSeriesValue::Scalar(500.0));
}

#[test]
fn data_zoom_x_filter_mode_materializes_indices_selection_for_large_non_monotonic_views() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_id = crate::ids::DataZoomId::new(1);
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

            from: None,
            transforms: Vec::new(),
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
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 60_001usize;
    let period = 1000usize;
    let denom = 1000.0f64;
    let xs: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    let ys: Vec<f64> = (0..n).map(|i| i as f64).collect();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..64 {
        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();

        let Some(participation) = engine.participation().series_participation(series_id) else {
            continue;
        };
        let RowSelection::Indices(indices) = &participation.selection else {
            continue;
        };

        assert_eq!(
            participation.x_policy.filter,
            Default::default(),
            "expected x filter predicate to be cleared after indices materialization"
        );

        assert!(
            indices.len() < n,
            "expected x filtering to drop some points"
        );
        assert!(indices.len() > 1000, "expected a non-trivial indices view");
        assert_eq!(indices[0], 200);
        assert_eq!(indices[600], 800);
        assert_eq!(indices[601], 1200);
        return;
    }

    panic!("expected x filter indices view to be materialized within the step loop");
}

#[test]
fn data_zoom_x_then_y_filter_materializes_indices_in_order_for_large_non_monotonic_views() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let zoom_y_id = crate::ids::DataZoomId::new(2);
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

            from: None,
            transforms: Vec::new(),
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
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_y_id,
            axis: y_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 60_001usize;
    let period = 1000usize;
    let denom = 1000.0f64;
    let xs: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    let ys: Vec<f64> = (0..n).map(|i| if i % 2 == 0 { 0.0 } else { 1.0 }).collect();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow { min: 0.0, max: 0.1 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..64 {
        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();

        let Some(participation) = engine.participation().series_participation(series_id) else {
            continue;
        };
        let RowSelection::Indices(indices) = &participation.selection else {
            continue;
        };

        assert_eq!(
            participation.x_policy.filter,
            Default::default(),
            "expected x filter predicate to be cleared after indices materialization"
        );

        assert_eq!(indices.len(), 60 * 301);
        assert_eq!(indices[0], 200);
        assert_eq!(indices[1], 202);
        assert_eq!(indices[300], 800);
        assert_eq!(indices[301], 1200);
        assert_eq!(indices[indices.len() - 1], 59_800);
        return;
    }

    panic!("expected x->y indices selection to be materialized within the step loop");
}

#[test]
fn filter_plan_isolated_per_grid_for_x_indices_materialization() {
    let dataset1_id = crate::ids::DatasetId::new(1);
    let dataset2_id = crate::ids::DatasetId::new(2);
    let grid1_id = crate::ids::GridId::new(1);
    let grid2_id = crate::ids::GridId::new(2);
    let x1_axis = crate::ids::AxisId::new(1);
    let y1_axis = crate::ids::AxisId::new(2);
    let x2_axis = crate::ids::AxisId::new(3);
    let y2_axis = crate::ids::AxisId::new(4);
    let zoom1_id = crate::ids::DataZoomId::new(1);
    let series1_id = crate::ids::SeriesId::new(1);
    let series2_id = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        )),
        datasets: vec![
            DatasetSpec {
                id: dataset1_id,
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

                from: None,
                transforms: Vec::new(),
            },
            DatasetSpec {
                id: dataset2_id,
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

                from: None,
                transforms: Vec::new(),
            },
        ],
        grids: vec![GridSpec { id: grid1_id }, GridSpec { id: grid2_id }],
        axes: vec![
            AxisSpec {
                id: x1_axis,
                name: None,
                kind: AxisKind::X,
                grid: grid1_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y1_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid1_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: x2_axis,
                name: None,
                kind: AxisKind::X,
                grid: grid2_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y2_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid2_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom1_id,
            axis: x1_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series1_id,
                name: None,
                kind: SeriesKind::Scatter,
                dataset: dataset1_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x1_axis,
                y_axis: y1_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series2_id,
                name: None,
                kind: SeriesKind::Scatter,
                dataset: dataset2_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis: x2_axis,
                y_axis: y2_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table1 = DataTable::default();
    let n = 60_001usize;
    let period = 1000usize;
    let denom = 1000.0f64;
    let xs1: Vec<f64> = (0..n).map(|i| (i % period) as f64 / denom).collect();
    let ys1: Vec<f64> = (0..n).map(|i| i as f64).collect();
    table1.push_column(Column::F64(xs1));
    table1.push_column(Column::F64(ys1));
    engine.datasets_mut().insert(dataset1_id, table1);

    let mut table2 = DataTable::default();
    table2.push_column(Column::F64((0..=7).map(|v| v as f64).collect()));
    table2.push_column(Column::F64((0..=7).map(|v| v as f64).collect()));
    engine.datasets_mut().insert(dataset2_id, table2);

    engine.apply_action(Action::SetDataWindowX {
        axis: x1_axis,
        window: Some(DataWindow { min: 0.2, max: 0.8 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let mut steps = 0u64;
    for _ in 0..64 {
        let _ = engine
            .step(&mut measurer, WorkBudget::new(1_000_000, 0, 2_048))
            .unwrap();
        steps += 1;

        let Some(p1) = engine.participation().series_participation(series1_id) else {
            continue;
        };
        let RowSelection::Indices(_indices1) = &p1.selection else {
            continue;
        };

        let Some(p2) = engine.participation().series_participation(series2_id) else {
            panic!("expected series2 participation");
        };
        assert!(
            matches!(p2.selection, RowSelection::Range(_)),
            "expected grid2 series selection to remain a range (unaffected by grid1 indices)"
        );

        let stats = engine.stats();
        assert_eq!(stats.filter_plan_runs, steps);
        assert_eq!(stats.filter_plan_grids, steps * 2);
        assert_eq!(stats.filter_plan_steps_run, steps * 10);
        assert_eq!(stats.filter_x_indices_applied_series, 1);
        assert_eq!(stats.filter_y_indices_applied_series, 0);
        return;
    }

    panic!("expected x filter indices view to be materialized within the step loop");
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.series.len(), 2);
    assert_eq!(axis.series[0].series, series_a);
    assert_eq!(axis.series[1].series, series_b);

    let crate::TooltipSeriesValue::Scalar(a) = axis.series[0].value else {
        panic!("expected scalar tooltip value for series A");
    };
    assert_eq!(a, 10.0);
    let crate::TooltipSeriesValue::Scalar(b) = axis.series[1].value else {
        panic!("expected scalar tooltip value for series B");
    };
    assert_eq!(b, 20.0);
}

#[test]
fn axis_pointer_axis_trigger_includes_placeholders_for_missing_series_values() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    // Non-monotonic X: [0, 1, 0.5]. Hover at x=50px will prefer the nearest sample,
    // which is the last row (x=0.5). Series A has a missing value there.
    table.push_column(Column::F64(vec![0.0, 1.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, f64::NAN]));
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
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.series.len(), 2);
    assert_eq!(axis.series[0].series, series_a);
    assert_eq!(axis.series[1].series, series_b);
    assert_eq!(axis.series[0].value, crate::TooltipSeriesValue::Missing);
    let crate::TooltipSeriesValue::Scalar(b) = axis.series[1].value else {
        panic!("expected scalar tooltip value for series B");
    };
    assert_eq!(b, 20.0);
}

#[test]
fn axis_pointer_item_trigger_snaps_to_hit_point_when_enabled() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn axis_pointer_axis_trigger_snaps_axis_value_to_nearest_sample_when_enabled() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

    // Hover is closer to the x=0 sample than x=1. In snap mode the axis pointer aligns to x=0.
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(25.0), Px(10.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert_eq!(axis_pointer.crosshair_px.x, Px(0.0));
    assert_eq!(axis_pointer.crosshair_px.y, Px(10.0));
    assert!(axis_pointer.hit.is_none());

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 0.0);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);
    assert_eq!(axis.series[0].value, crate::TooltipSeriesValue::Scalar(0.0));
}

#[test]
fn axis_pointer_axis_trigger_uses_first_visible_series_as_primary() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis_a = crate::ids::AxisId::new(1);
    let x_axis_b = crate::ids::AxisId::new(2);
    let y_axis = crate::ids::AxisId::new(3);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);

    let x_a = crate::ids::FieldId::new(1);
    let y_a = crate::ids::FieldId::new(2);
    let x_b = crate::ids::FieldId::new(3);
    let y_b = crate::ids::FieldId::new(4);

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        )),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec { id: x_a, column: 0 },
                FieldSpec { id: y_a, column: 1 },
                FieldSpec { id: x_b, column: 2 },
                FieldSpec { id: y_b, column: 3 },
            ],

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis_a,
                name: None,
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: x_axis_b,
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: Some("A".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_a,
                    y: y_a,
                    y2: None,
                },
                x_axis: x_axis_a,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: Some("B".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_b,
                    y: y_b,
                    y2: None,
                },
                x_axis: x_axis_b,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![0.0, 1.0]));
    table.push_column(Column::F64(vec![10.0, 11.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::SetSeriesVisible {
        series: series_a,
        visible: false,
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(25.0), Px(10.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis_b);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_b);
    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Scalar(10.0)
    );
}

#[test]
fn axis_pointer_axis_trigger_snaps_category_y_to_band_center_when_enabled() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let value_field = crate::ids::FieldId::new(1);
    let category_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: value_field,
                    column: 0,
                },
                FieldSpec {
                    id: category_field,
                    column: 1,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                }),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: value_field,
                y: category_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, -2.0, 3.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(50.0), Px(150.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    assert!(axis_pointer.hit.is_none());
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, y_axis);
    assert_eq!(axis.axis_kind, AxisKind::Y);
    assert_eq!(axis.axis_value, 1.0);

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    let expected_y = crate::engine::axis::y_px_at_data_in_rect(trigger_window, 1.0, viewport);
    assert!((axis_pointer.crosshair_px.y.0 - expected_y).abs() < 1e-4);

    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);
    assert_eq!(axis.series[0].value_axis, x_axis);
    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Scalar(-2.0)
    );
}

#[test]
fn axis_pointer_axis_trigger_emits_shadow_rect_for_category_trigger_axis() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let value_field = crate::ids::FieldId::new(1);
    let category_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: value_field,
                    column: 0,
                },
                FieldSpec {
                    id: category_field,
                    column: 1,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: category_field,
                y: value_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, 2.0, 3.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(120.0), Px(100.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let shadow = axis_pointer.shadow_rect_px.expect("expected a shadow band");

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let x0 = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 0.5, viewport);
    let x1 = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 1.5, viewport);

    let expected_left = x0.min(x1);
    let expected_right = x0.max(x1);
    assert!((shadow.origin.x.0 - expected_left).abs() < 1e-4);
    assert!(((shadow.origin.x.0 + shadow.size.width.0) - expected_right).abs() < 1e-4);
    assert_eq!(shadow.origin.y, viewport.origin.y);
    assert_eq!(shadow.size.height, viewport.size.height);
}

#[test]
fn axis_pointer_shadow_rect_respects_category_band_edges_under_x_window() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let value_field = crate::ids::FieldId::new(1);
    let category_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let spec = ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: Some(viewport),
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: value_field,
                    column: 0,
                },
                FieldSpec {
                    id: category_field,
                    column: 1,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: true,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![SeriesSpec {
            id: series_id,
            name: None,
            kind: SeriesKind::Bar,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: category_field,
                y: value_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, 2.0, 3.0, 0.5]));
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 1.0, max: 2.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let trigger_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((trigger_window.min - 0.5).abs() < 1e-6);
    assert!((trigger_window.max - 2.5).abs() < 1e-6);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(75.0), Px(100.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let shadow = axis_pointer.shadow_rect_px.expect("expected a shadow band");

    let x0 = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 0.5, viewport);
    let x1 = crate::engine::axis::x_px_at_data_in_rect(trigger_window, 1.5, viewport);
    let expected_left = x0.min(x1);
    let expected_right = x0.max(x1);
    assert!((shadow.origin.x.0 - expected_left).abs() < 1e-4);
    assert!(((shadow.origin.x.0 + shadow.size.width.0) - expected_right).abs() < 1e-4);
    assert_eq!(shadow.origin.y, viewport.origin.y);
    assert_eq!(shadow.size.height, viewport.size.height);
}

#[test]
fn category_x_filter_culls_marks_for_non_monotonic_line_and_samples_first_duplicate() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..6).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    // Non-monotonic category indices, with duplicates at x=3.
    let xs = vec![0.0, 3.0, 2.0, 3.0, 5.0, 4.0];
    let ys = vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    assert_eq!(
        view.selection,
        RowSelection::Indices(vec![1, 2, 3, 5].into()),
        "expected non-monotonic category axis to materialize indices selection under Filter mode"
    );
    assert_eq!(
        view.x_policy.filter,
        Default::default(),
        "expected x filter predicate to be cleared after indices materialization"
    );

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 1.5).abs() < 1e-6);
    assert!((x_window.max - 4.5).abs() < 1e-6);

    let indices = &engine.output().marks.arena.data_indices;
    assert!(!indices.is_empty(), "expected marks to contain indices");
    assert!(
        indices.iter().all(|&i| matches!(i, 1 | 2 | 3 | 5)),
        "expected mark indices to be culled by the x window"
    );

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 3.0);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);
    assert_eq!(axis.series[0].value_axis, y_axis);
    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Scalar(10.0),
        "expected axis-pointer sampling to pick the first raw index for duplicated category values"
    );
}

#[test]
fn category_x_filter_materializes_indices_for_scatter_and_respects_base_row_range() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..6).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    // Non-monotonic category indices, with duplicates at x=3. The base row range excludes the
    // earlier duplicate so we can assert axis-pointer sampling respects the filtered selection.
    let xs = vec![0.0, 3.0, 2.0, 3.0, 5.0, 4.0];
    let ys = vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 2, end: 6 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 2.0, max: 4.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..16 {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    let view = engine
        .view()
        .series_view(series_id)
        .expect("expected series view");
    assert_eq!(
        view.selection,
        RowSelection::Indices(vec![2, 3, 5].into()),
        "expected non-monotonic category axis to materialize indices selection under Filter mode"
    );
    assert_eq!(
        view.x_policy.filter,
        Default::default(),
        "expected x filter predicate to be cleared after indices materialization"
    );

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert_eq!(axis.axis_value, 3.0);
    assert_eq!(axis.series.len(), 1);
    assert_eq!(axis.series[0].series, series_id);
    assert_eq!(axis.series[0].value_axis, y_axis);
    assert_eq!(
        axis.series[0].value,
        crate::TooltipSeriesValue::Scalar(30.0),
        "expected axis-pointer sampling to respect the indices selection / base row range"
    );
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
fn scatter_filter_mode_none_culls_points_outside_x_window() {
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

            from: None,
            transforms: Vec::new(),
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
            id: crate::ids::DataZoomId::new(1),
            axis: x_axis,
            filter_mode: FilterMode::None,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();

    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![1.0, 100.0]));
    table.push_column(Column::F64(vec![1.0, 1.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };

    assert_eq!(
        points.points.end - points.points.start,
        1,
        "expected out-of-window points to be culled for FilterMode::None"
    );
    assert_eq!(
        marks.arena.data_indices[points.points.start], 0,
        "expected the in-window raw index to be emitted"
    );
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

#[test]
fn scatter_large_threshold_can_force_large_mode() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(20.0), Px(100.0)),
    );

    let mut spec = ChartSpec {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let n = 1_000usize;
    let mut table = DataTable::default();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        xs.push(i as f64 / (n as f64 - 1.0));
        ys.push(((i as f64) * 0.01).sin());
    }
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));

    let mut engine = ChartEngine::new(spec.clone()).unwrap();
    engine.datasets_mut().insert(dataset_id, table.clone());

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    let emitted_default = points.points.end - points.points.start;
    assert_eq!(emitted_default, n, "expected all points to be emitted");

    spec.series[0].lod = Some(crate::spec::SeriesLodSpecV1 {
        large: Some(true),
        large_threshold: Some(1),
        ..Default::default()
    });

    let mut engine = ChartEngine::new(spec).unwrap();
    engine.datasets_mut().insert(dataset_id, table);

    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    let emitted_forced = points.points.end - points.points.start;

    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    assert!(emitted_forced <= width_px * 4, "emitted={emitted_forced}");
    assert!(
        emitted_forced < emitted_default,
        "forced={emitted_forced} default={emitted_default}"
    );
}

#[test]
fn scatter_progressive_can_force_multiple_steps() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: Some(crate::spec::SeriesLodSpecV1 {
                large: Some(true),
                large_threshold: Some(1),
                progressive: Some(512),
                progressive_threshold: Some(1),
            }),
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 5_000usize;
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

    assert!(steps > 1, "expected progressive to require multiple steps");
    assert!(steps <= 256);
}

#[test]
fn scatter_large_mode_respects_y_empty_mask() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 50_000usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        xs.push(i as f64 / (n as f64 - 1.0));
        ys.push(5.0);
    }

    let outliers = [
        5_000usize,
        15_000usize,
        25_000usize,
        35_000usize,
        45_000usize,
    ];
    for (j, &i) in outliers.iter().enumerate() {
        ys[i] = if j % 2 == 0 { 1_000.0 } else { -1_000.0 };
    }

    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(ys.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

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

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert!(participation.empty_mask.y_active);
    for &i in &outliers {
        assert!(
            !participation.empty_mask.allows_raw_index(i, &xs, &ys, None),
            "expected outlier to be masked under Y empty"
        );
    }

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

    let indices = &marks.arena.data_indices[points.points.clone()];
    for &i in &outliers {
        assert!(
            !indices.contains(&(i as u32)),
            "expected masked outlier raw index {i} not to be emitted in LOD points"
        );
    }
}

#[test]
fn scatter_large_mode_does_not_hit_y_empty_masked_outlier() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 2.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 50_000usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        xs.push(i as f64 / (n as f64 - 1.0));
        ys.push(5.0);
    }

    let outlier = 25_000usize;
    ys[outlier] = 1_000.0;

    table.push_column(Column::F64(xs.clone()));
    table.push_column(Column::F64(ys.clone()));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

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

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert!(participation.empty_mask.y_active);
    assert!(
        !participation
            .empty_mask
            .allows_raw_index(outlier, &xs, &ys, None),
        "expected outlier to be masked under Y empty"
    );

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Points && n.source_series == Some(series_id))
        .expect("expected a points mark node");
    let MarkPayloadRef::Points(points) = &node.payload else {
        panic!("expected points payload");
    };
    assert!(points.points.end > points.points.start);

    // Sanity: hovering on an emitted point hits.
    let first_point_px = marks.arena.points[points.points.start];
    engine.apply_action(Action::HoverAt {
        point: first_point_px,
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);
    assert!(
        engine.output().axis_pointer.is_some(),
        "expected item-trigger axis pointer output to be present when hovering on a mark"
    );
    assert!(
        engine.output().hover.is_some(),
        "expected output.hover to be present when hovering on a mark"
    );

    // Hover exactly where an unmasked outlier would be clamped to the Y window max in LOD mode.
    // If `y.filterMode=Empty` masking were not respected, this would produce a hittable point on
    // the y=10 boundary at `x=xs[outlier]`.
    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, xs[outlier], viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 10.0, viewport);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected masked outlier not to be hittable under y.filterMode=Empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be none when hovering the masked outlier's would-be clamped location"
    );
}

#[test]
fn append_only_marks_rebuild_updates_lod_polyline_without_clearing_nodes() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
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

            from: None,
            transforms: Vec::new(),
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
                range: Some(AxisRange::Fixed { min: 0.0, max: 1.0 }),
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::Fixed {
                    min: -1.5,
                    max: 1.5,
                }),
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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

    let before_nodes = engine.output().marks.nodes.len();
    let before_marks_rev = engine.output().marks.revision;

    let before_points_range = engine
        .output()
        .marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id))
        .expect("expected a polyline node");
    let MarkPayloadRef::Polyline(before_poly) = &before_points_range.payload else {
        panic!("expected polyline payload");
    };
    let before_points_range = before_poly.points.clone();

    let table = engine.datasets_mut().dataset_mut(dataset_id).unwrap();
    table.append_row_f64(&[1.0, 0.0]).unwrap();
    let appended_index = (table.row_count() - 1) as u32;

    let mut steps = 0;
    let mut result = crate::scheduler::StepResult::default();
    while steps < 32 {
        result = engine
            .step(&mut measurer, WorkBudget::new(8192, 0, 8))
            .unwrap();
        steps += 1;
        if !result.unfinished {
            break;
        }
    }
    assert!(
        !result.unfinished,
        "expected append-only rebuild to finish quickly (steps={steps})"
    );

    assert_eq!(engine.output().marks.nodes.len(), before_nodes);
    assert!(engine.output().marks.revision.0 > before_marks_rev.0);

    let after_node = engine
        .output()
        .marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id))
        .expect("expected a polyline node");
    let MarkPayloadRef::Polyline(after_poly) = &after_node.payload else {
        panic!("expected polyline payload");
    };

    assert_ne!(before_points_range, after_poly.points);
    assert!(
        engine.output().marks.arena.data_indices[after_poly.points.clone()]
            .iter()
            .any(|&i| i == appended_index),
        "expected the appended row index to be represented in the LOD output"
    );
}

#[test]
fn append_only_marks_rebuild_preserves_geometry_while_unfinished_multi_series() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let x_field = crate::ids::FieldId::new(1);
    let y1_field = crate::ids::FieldId::new(2);
    let y2_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(100.0)),
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
                    id: y1_field,
                    column: 1,
                },
                FieldSpec {
                    id: y2_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
                range: Some(AxisRange::Fixed { min: 0.0, max: 1.0 }),
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::Fixed {
                    min: -1.5,
                    max: 1.5,
                }),
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y1_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y2_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 120_000usize;
    let mut xs = Vec::with_capacity(n);
    let mut y1s = Vec::with_capacity(n);
    let mut y2s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n as f64 - 1.0);
        xs.push(t);
        y1s.push((t * 20.0).sin());
        y2s.push((t * 17.0).cos());
    }
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(y1s));
    table.push_column(Column::F64(y2s));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let mut steps = 0usize;
    loop {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        steps += 1;
        if !step.unfinished || steps > 512 {
            break;
        }
    }
    assert!(steps <= 512);

    let before_marks_rev = engine.output().marks.revision;
    let before_nodes_len = engine.output().marks.nodes.len();
    assert!(before_nodes_len > 0);

    for series_id in [series_a, series_b] {
        assert!(
            engine
                .output()
                .marks
                .nodes
                .iter()
                .any(|n| n.kind == crate::marks::MarkKind::Polyline
                    && n.source_series == Some(series_id)),
            "expected a polyline node for series {series_id:?} before append"
        );
    }

    let table = engine.datasets_mut().dataset_mut(dataset_id).unwrap();
    table.append_row_f64(&[1.0, 0.0, 1.0]).unwrap();
    let appended_index = (table.row_count() - 1) as u32;

    let mut saw_unfinished = false;
    let mut last_rev = before_marks_rev;
    let mut result = crate::scheduler::StepResult::default();

    for _ in 0..512 {
        result = engine
            .step(&mut measurer, WorkBudget::new(2048, 0, 8))
            .unwrap();

        let marks = &engine.output().marks;
        assert!(marks.nodes.len() >= before_nodes_len);
        assert!(marks.revision.0 >= last_rev.0);
        last_rev = marks.revision;

        for series_id in [series_a, series_b] {
            assert!(
                marks.nodes.iter().any(|n| {
                    n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id)
                }),
                "expected a polyline node for series {series_id:?} during append-only rebuild"
            );
        }

        if result.unfinished {
            saw_unfinished = true;
        } else {
            break;
        }
    }

    assert!(saw_unfinished, "expected at least one unfinished step");
    assert!(!result.unfinished, "expected append-only rebuild to finish");

    for series_id in [series_a, series_b] {
        let has_appended_index = engine
            .output()
            .marks
            .nodes
            .iter()
            .filter(|n| {
                n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id)
            })
            .any(|node| {
                let crate::marks::MarkPayloadRef::Polyline(poly) = &node.payload else {
                    return false;
                };
                engine.output().marks.arena.data_indices[poly.points.clone()]
                    .iter()
                    .any(|&i| i == appended_index)
            });

        assert!(
            has_appended_index,
            "expected appended raw index to be represented in the LOD output for series {series_id:?}"
        );
    }
}

#[test]
fn update_mutation_clears_marks_and_forces_rebuild() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(100.0)),
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

            from: None,
            transforms: Vec::new(),
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
                range: Some(AxisRange::Fixed { min: 0.0, max: 1.0 }),
            },
            AxisSpec {
                id: y_axis,
                name: None,
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: Some(AxisRange::Fixed {
                    min: -1.5,
                    max: 1.5,
                }),
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();

    let n = 20_000usize;
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / (n as f64 - 1.0);
        xs.push(t);
        ys.push((t * 20.0).sin());
    }
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    loop {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }

    assert!(!engine.output().marks.nodes.is_empty());
    let marks_rev_before = engine.output().marks.revision;

    // In-place update (no row_count growth) must invalidate caches deterministically and must not be
    // mistaken for append-only rebuild.
    {
        let table = engine.datasets_mut().dataset_mut(dataset_id).unwrap();
        let mid = table.row_count() / 2;
        let x_mid = table.column_f64(0).unwrap()[mid];
        table.update_row_f64(mid, &[x_mid, 0.0]).unwrap();
    }

    // With a zero budget, marks cannot be rebuilt; the engine should have cleared marks due to the
    // update revision change.
    let step = engine
        .step(&mut measurer, WorkBudget::new(0, 0, 0))
        .unwrap();
    assert!(step.unfinished);
    assert!(engine.output().marks.nodes.is_empty());
    assert!(engine.output().marks.revision.0 > marks_rev_before.0);

    // Rebuild should be possible under a normal budget again.
    loop {
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
            .unwrap();
        if !step.unfinished {
            break;
        }
    }
    assert!(!engine.output().marks.nodes.is_empty());
}

#[test]
fn bar_item_trigger_does_not_hit_y_empty_masked_outlier() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    categories: vec!["A".into(), "B".into(), "C".into()],
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 4.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
    table.push_column(Column::F64(vec![5.0, 1_000.0, 5.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert!(participation.empty_mask.y_active);
    assert!(
        !participation
            .empty_mask
            .allows_raw_index(1, &[0.0, 1.0, 2.0], &[5.0, 1_000.0, 5.0], None),
        "expected outlier to be masked under Y empty"
    );

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected a rect mark node");
    let MarkPayloadRef::Rect(rects) = &node.payload else {
        panic!("expected rect payload");
    };
    assert!(rects.rects.end > rects.rects.start);
    let indices = &marks.arena.rect_data_indices[rects.rects.clone()];
    assert!(
        !indices.contains(&1u32),
        "expected y-empty masked outlier raw index not to be emitted as a bar rect"
    );

    // Sanity: hovering inside an emitted bar hits.
    let i0 = rects.rects.start;
    let r0 = marks.arena.rects[i0];
    let c0 = Point::new(
        Px(r0.origin.x.0 + 0.5 * r0.size.width.0),
        Px(r0.origin.y.0 + 0.5 * r0.size.height.0),
    );
    engine.apply_action(Action::HoverAt { point: c0 });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);
    assert!(
        engine.output().axis_pointer.is_some(),
        "expected item-trigger axis pointer output to be present when hovering on a bar"
    );
    assert!(
        engine.output().hover.is_some(),
        "expected output.hover to be present when hovering on a bar"
    );

    // Hover where the masked outlier would be drawn if it leaked via clamping (x=1 category, y≈max).
    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 1.0, viewport);
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 9.9, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected y-empty masked outlier bar to be non-hittable"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be none over the masked outlier's would-be location"
    );
}

#[test]
fn axis_pointer_shadow_rect_is_emitted_for_category_axis_when_bar_is_y_empty_masked() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let zoom_id = crate::ids::DataZoomId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: vec!["A".into(), "B".into(), "C".into()],
                }),
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
        data_zoom_y: vec![DataZoomYSpec {
            id: zoom_id,
            axis: y_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
    table.push_column(Column::F64(vec![5.0, 1_000.0, 5.0]));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 1.0, viewport);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine
        .output()
        .axis_pointer
        .as_ref()
        .expect("expected axis pointer output");
    assert!(
        axis_pointer.shadow_rect_px.is_some(),
        "expected category-axis shadow rect to be emitted even when the bar is y-empty masked"
    );
    assert!(
        axis_pointer.hit.is_none(),
        "expected no marker hit for the y-empty masked bar"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to remain none when no marker hit is possible"
    );

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 1.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(matches!(entry.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn data_zoom_x_filter_mode_empty_masks_bar_marks_without_culling_row_selection() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
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
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let Some(participation) = engine.participation().series_participation(series_id) else {
        panic!("expected series participation");
    };
    assert_eq!(participation.x_filter_mode, FilterMode::Empty);
    assert_eq!(participation.y_filter_mode, FilterMode::None);
    assert!(participation.empty_mask.x_active);
    assert!(!participation.empty_mask.y_active);
    assert_eq!(participation.selection.view_len(10), 5);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    assert!((x_window.min - 4.5).abs() < 1e-6);
    assert!((x_window.max - 9.5).abs() < 1e-6);

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    assert!((y_window.min - 0.0).abs() < 1e-6);
    assert!((y_window.max - 10.0).abs() < 1e-6);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected rect marks for bar series");
    let MarkPayloadRef::Rect(r) = &node.payload else {
        panic!("expected rect payload");
    };
    assert_eq!(
        r.rects.start, r.rects.end,
        "expected x-empty mask to suppress mark emission while keeping y-filter selection"
    );
}

#[test]
fn axis_pointer_tooltip_respects_x_empty_mask_for_bar_when_marks_are_empty_but_selection_is_not() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });

    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 7.0, viewport);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine
        .output()
        .axis_pointer
        .as_ref()
        .expect("expected axis pointer output");
    assert!(
        axis_pointer.shadow_rect_px.is_some(),
        "expected category-axis shadow rect even when the series cannot be sampled"
    );
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to remain none when marks are empty"
    );

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(matches!(entry.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn axis_pointer_item_trigger_returns_none_for_bar_under_x_empty_mask_when_marks_are_empty() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    // Keep a stable (non-empty) selection, but disjoint the x-empty mask window so marks are empty.
    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });
    engine.apply_action(Action::SetDataWindowY {
        axis: y_axis,
        window: Some(DataWindow {
            min: 0.0,
            max: 10.0,
        }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected rect marks for bar series");
    let MarkPayloadRef::Rect(r) = &node.payload else {
        panic!("expected rect payload");
    };
    assert_eq!(
        r.rects.start, r.rects.end,
        "expected bar marks to be empty under x.filterMode=Empty disjoint mask"
    );

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 7.0, viewport);

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be absent when marks are empty"
    );
}

#[test]
fn axis_pointer_item_trigger_returns_none_for_stacked_bar_under_x_empty_mask_when_marks_are_empty()
{
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let stack = crate::ids::StackId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys.clone()));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    for series_id in [series_a, series_b] {
        let marks = &engine.output().marks;
        let node = marks
            .nodes
            .iter()
            .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
            .expect("expected rect marks for stacked bar series");
        let MarkPayloadRef::Rect(r) = &node.payload else {
            panic!("expected rect payload");
        };
        assert_eq!(
            r.rects.start, r.rects.end,
            "expected stacked bar marks to be empty under x.filterMode=Empty disjoint mask"
        );
    }

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be absent when marks are empty"
    );
}

#[test]
fn axis_pointer_item_trigger_returns_none_for_horizontal_bar_under_x_empty_mask_when_marks_are_empty()
 {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
                range: None,
            },
        ],
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 10_000.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected rect marks for horizontal bar series");
    let MarkPayloadRef::Rect(r) = &node.payload else {
        panic!("expected rect payload");
    };
    assert_eq!(
        r.rects.start, r.rects.end,
        "expected horizontal bar marks to be empty under x.filterMode=Empty disjoint mask"
    );

    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    assert!(
        engine.output().axis_pointer.is_none(),
        "expected item-trigger axis pointer output to be absent when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to be absent when marks are empty"
    );
}

#[test]
fn axis_pointer_axis_trigger_emits_shadow_and_missing_tooltip_for_stacked_bar_under_x_empty_mask() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_a = crate::ids::SeriesId::new(1);
    let series_b = crate::ids::SeriesId::new(2);
    let stack = crate::ids::StackId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_a_field = crate::ids::FieldId::new(2);
    let y_b_field = crate::ids::FieldId::new(3);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
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
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
        series: vec![
            SeriesSpec {
                id: series_a,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
            SeriesSpec {
                id: series_b,
                name: None,
                kind: SeriesKind::Bar,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            },
        ],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs.clone();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys.clone()));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    for series_id in [series_a, series_b] {
        let marks = &engine.output().marks;
        let node = marks
            .nodes
            .iter()
            .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
            .expect("expected rect marks for stacked bar series");
        let MarkPayloadRef::Rect(r) = &node.payload else {
            panic!("expected rect payload");
        };
        assert_eq!(
            r.rects.start, r.rects.end,
            "expected stacked bar marks to be empty under x.filterMode=Empty disjoint mask"
        );
    }

    let x_window = engine
        .output()
        .axis_windows
        .get(&x_axis)
        .copied()
        .unwrap_or_default();
    let hover_x = crate::engine::axis::x_px_at_data_in_rect(x_window, 7.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(hover_x), Px(120.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine
        .output()
        .axis_pointer
        .as_ref()
        .expect("expected axis pointer output");
    assert!(
        axis_pointer.shadow_rect_px.is_some(),
        "expected category-axis shadow rect even when stacked bar marks are empty"
    );
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to remain none when marks are empty"
    );

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, x_axis);
    assert_eq!(axis.axis_kind, AxisKind::X);
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    for id in [series_a, series_b] {
        let entry = axis
            .series
            .iter()
            .find(|e| e.series == id)
            .expect("missing tooltip entry");
        assert!(matches!(entry.value, crate::TooltipSeriesValue::Missing));
    }
}

#[test]
fn axis_pointer_axis_trigger_emits_shadow_and_missing_tooltip_for_horizontal_bar_under_x_empty_mask()
 {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let zoom_x_id = crate::ids::DataZoomId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Value".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: y_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10).map(|i| format!("C{i:02}")).collect(),
                }),
                range: None,
            },
        ],
        data_zoom_x: vec![DataZoomXSpec {
            id: zoom_x_id,
            axis: x_axis,
            filter_mode: FilterMode::Empty,
            min_value_span: None,
            max_value_span: None,
        }],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Shadow,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 0.0,
            throttle_px: 0.0,
        }),
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = ChartEngine::new(spec).unwrap();
    let xs: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let ys: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetDatasetRowRange {
        dataset: dataset_id,
        range: Some(RowRange { start: 0, end: 5 }),
    });
    engine.apply_action(Action::SetDataWindowX {
        axis: x_axis,
        window: Some(DataWindow { min: 5.0, max: 9.0 }),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let node = marks
        .nodes
        .iter()
        .find(|n| n.kind == crate::marks::MarkKind::Rect && n.source_series == Some(series_id))
        .expect("expected rect marks for horizontal bar series");
    let MarkPayloadRef::Rect(r) = &node.payload else {
        panic!("expected rect payload");
    };
    assert_eq!(
        r.rects.start, r.rects.end,
        "expected horizontal bar marks to be empty under x.filterMode=Empty disjoint mask"
    );

    let y_window = engine
        .output()
        .axis_windows
        .get(&y_axis)
        .copied()
        .unwrap_or_default();
    let hover_y = crate::engine::axis::y_px_at_data_in_rect(y_window, 7.0, viewport);
    engine.apply_action(Action::HoverAt {
        point: Point::new(Px(200.0), Px(hover_y)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(32_768, 0, 16))
        .unwrap();
    assert!(!step.unfinished);

    let axis_pointer = engine
        .output()
        .axis_pointer
        .as_ref()
        .expect("expected axis pointer output");
    assert!(
        axis_pointer.shadow_rect_px.is_some(),
        "expected category-axis shadow rect even when horizontal bar marks are empty"
    );
    assert!(
        axis_pointer.hit.is_none(),
        "expected no hover hit when marks are empty"
    );
    assert!(
        engine.output().hover.is_none(),
        "expected output.hover to remain none when marks are empty"
    );

    let crate::TooltipOutput::Axis(axis) = &axis_pointer.tooltip else {
        panic!("expected axis-trigger tooltip payload");
    };
    assert_eq!(axis.axis, y_axis);
    assert_eq!(axis.axis_kind, AxisKind::Y);
    assert!((axis.axis_value - 7.0).abs() < 1e-4);
    let entry = axis
        .series
        .iter()
        .find(|e| e.series == series_id)
        .expect("missing tooltip entry");
    assert!(matches!(entry.value, crate::TooltipSeriesValue::Missing));
}

#[test]
fn line_large_mode_is_pixel_bounded() {
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
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
        .find(|n| n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id))
        .expect("expected a polyline mark node");
    let MarkPayloadRef::Polyline(poly) = &node.payload else {
        panic!("expected polyline payload");
    };
    let emitted = poly.points.end - poly.points.start;

    let width_px = viewport.size.width.0.max(1.0).ceil() as usize;
    assert!(
        emitted <= width_px * 4,
        "emitted={emitted} width={width_px}"
    );
    assert!(emitted > 0);
}

#[test]
fn lod_scatter_large_mode_is_budget_invariant() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let n = 60_000usize;
    let xs: Vec<f64> = (0..n).map(|i| i as f64 / (n as f64 - 1.0)).collect();
    let ys: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.01).sin()).collect();

    let build_engine = || {
        let mut engine = ChartEngine::new(spec.clone()).unwrap();
        let mut table = DataTable::default();
        table.push_column(Column::F64(xs.clone()));
        table.push_column(Column::F64(ys.clone()));
        engine.datasets_mut().insert(dataset_id, table);
        engine
    };

    let mut engine_a = build_engine();
    let mut measurer_a = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_a,
        &mut measurer_a,
        WorkBudget::new(16_384, 0, 16),
        512,
    );

    let mut engine_b = build_engine();
    let mut measurer_b = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_b,
        &mut measurer_b,
        WorkBudget::new(1_000_000, 0, 1_024),
        64,
    );

    assert_eq!(
        engine_a.output().axis_windows,
        engine_b.output().axis_windows,
        "expected axis windows to be budget-invariant"
    );
    assert_eq!(
        marks_signature(&engine_a.output().marks),
        marks_signature(&engine_b.output().marks),
        "expected marks output to be budget-invariant"
    );
}

#[test]
fn lod_line_large_mode_is_budget_invariant() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let n = 50_000usize;
    let xs: Vec<f64> = (0..n).map(|i| i as f64 / (n as f64 - 1.0)).collect();
    let ys: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.01).sin()).collect();

    let build_engine = || {
        let mut engine = ChartEngine::new(spec.clone()).unwrap();
        let mut table = DataTable::default();
        table.push_column(Column::F64(xs.clone()));
        table.push_column(Column::F64(ys.clone()));
        engine.datasets_mut().insert(dataset_id, table);
        engine
    };

    let mut engine_a = build_engine();
    let mut measurer_a = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_a,
        &mut measurer_a,
        WorkBudget::new(16_384, 0, 16),
        512,
    );

    let mut engine_b = build_engine();
    let mut measurer_b = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_b,
        &mut measurer_b,
        WorkBudget::new(1_000_000, 0, 1_024),
        64,
    );

    assert_eq!(
        engine_a.output().axis_windows,
        engine_b.output().axis_windows,
        "expected axis windows to be budget-invariant"
    );
    assert_eq!(
        marks_signature(&engine_a.output().marks),
        marks_signature(&engine_b.output().marks),
        "expected marks output to be budget-invariant"
    );
}

#[test]
fn lod_bar_mode_is_budget_invariant() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let viewport = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
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

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis,
                name: Some("Category".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                    categories: (0..10_000).map(|i| format!("C{i:05}")).collect(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let n = 10_000usize;
    let xs: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let ys: Vec<f64> = (0..n).map(|i| ((i as f64) * 0.01).sin()).collect();

    let build_engine = || {
        let mut engine = ChartEngine::new(spec.clone()).unwrap();
        let mut table = DataTable::default();
        table.push_column(Column::F64(xs.clone()));
        table.push_column(Column::F64(ys.clone()));
        engine.datasets_mut().insert(dataset_id, table);
        engine
    };

    let mut engine_a = build_engine();
    let mut measurer_a = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_a,
        &mut measurer_a,
        WorkBudget::new(8_192, 0, 8),
        4096,
    );

    let mut engine_b = build_engine();
    let mut measurer_b = NullTextMeasurer::default();
    run_engine_to_completion(
        &mut engine_b,
        &mut measurer_b,
        WorkBudget::new(1_000_000, 0, 1_024),
        256,
    );

    assert_eq!(
        engine_a.output().axis_windows,
        engine_b.output().axis_windows,
        "expected axis windows to be budget-invariant"
    );
    assert_eq!(
        marks_signature(&engine_a.output().marks),
        marks_signature(&engine_b.output().marks),
        "expected marks output to be budget-invariant"
    );
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
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

#[test]
fn line_missing_values_break_into_segments() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let series_id = crate::ids::SeriesId::new(1);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    ));

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0, 4.0]));
    table.push_column(Column::F64(vec![1.0, 2.0, f64::NAN, 3.0, 4.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut segments: Vec<(u64, Vec<u32>)> = marks
        .nodes
        .iter()
        .filter(|n| {
            n.kind == crate::marks::MarkKind::Polyline && n.source_series == Some(series_id)
        })
        .filter_map(|n| {
            let MarkPayloadRef::Polyline(poly) = &n.payload else {
                return None;
            };
            Some((
                crate::ids::mark_variant(n.id),
                marks.arena.data_indices[poly.points.clone()].to_vec(),
            ))
        })
        .collect();

    segments.sort_by_key(|(variant, _)| *variant);
    assert_eq!(segments.len(), 2, "expected two polyline segments");
    assert_eq!(segments[0].0, 0);
    assert_eq!(segments[1].0, 1);
    assert_eq!(segments[0].1, vec![0, 1]);
    assert_eq!(segments[1].1, vec![3, 4]);
}

#[derive(Debug, Clone, PartialEq)]
struct MarkTreeSignature {
    pub arena_points: Vec<fret_core::Point>,
    pub arena_data_indices: Vec<u32>,
    pub arena_rects: Vec<fret_core::Rect>,
    pub arena_rect_data_indices: Vec<u32>,
    pub nodes: Vec<MarkNodeSignature>,
}

#[derive(Debug, Clone, PartialEq)]
struct MarkNodeSignature {
    pub id: crate::ids::MarkId,
    pub parent: Option<crate::ids::MarkId>,
    pub layer: crate::ids::LayerId,
    pub order: crate::marks::MarkOrderKey,
    pub kind: crate::marks::MarkKind,
    pub source_series: Option<crate::ids::SeriesId>,
    pub payload: MarkPayloadSignature,
}

#[derive(Debug, Clone, PartialEq)]
enum MarkPayloadSignature {
    Group {
        clip: Option<fret_core::Rect>,
    },
    Polyline {
        points: core::ops::Range<usize>,
        stroke: Option<(crate::ids::PaintId, crate::paint::StrokeStyleV2)>,
    },
    Points {
        points: core::ops::Range<usize>,
        fill: Option<crate::ids::PaintId>,
        opacity_mul: Option<u32>,
        radius_mul: Option<u32>,
        stroke: Option<(crate::ids::PaintId, crate::paint::StrokeStyleV2)>,
    },
    Rect {
        rects: core::ops::Range<usize>,
        fill: Option<crate::ids::PaintId>,
        opacity_mul: Option<u32>,
        stroke: Option<(crate::ids::PaintId, crate::paint::StrokeStyleV2)>,
    },
    Text {
        rect: fret_core::Rect,
        text: crate::ids::StringId,
        style: crate::text::TextStyleId,
        fill: Option<crate::ids::PaintId>,
    },
}

fn f32_to_bits(v: Option<f32>) -> Option<u32> {
    v.map(|v| v.to_bits())
}

fn marks_signature(marks: &crate::marks::MarkTree) -> MarkTreeSignature {
    let mut nodes: Vec<MarkNodeSignature> = marks
        .nodes
        .iter()
        .map(|n| MarkNodeSignature {
            id: n.id,
            parent: n.parent,
            layer: n.layer,
            order: n.order,
            kind: n.kind,
            source_series: n.source_series,
            payload: match &n.payload {
                crate::marks::MarkPayloadRef::Group(g) => {
                    MarkPayloadSignature::Group { clip: g.clip }
                }
                crate::marks::MarkPayloadRef::Polyline(p) => MarkPayloadSignature::Polyline {
                    points: p.points.clone(),
                    stroke: p.stroke.clone(),
                },
                crate::marks::MarkPayloadRef::Points(p) => MarkPayloadSignature::Points {
                    points: p.points.clone(),
                    fill: p.fill,
                    opacity_mul: f32_to_bits(p.opacity_mul),
                    radius_mul: f32_to_bits(p.radius_mul),
                    stroke: p.stroke.clone(),
                },
                crate::marks::MarkPayloadRef::Rect(r) => MarkPayloadSignature::Rect {
                    rects: r.rects.clone(),
                    fill: r.fill,
                    opacity_mul: f32_to_bits(r.opacity_mul),
                    stroke: r.stroke.clone(),
                },
                crate::marks::MarkPayloadRef::Text(t) => MarkPayloadSignature::Text {
                    rect: t.rect,
                    text: t.text,
                    style: t.style,
                    fill: t.fill,
                },
            },
        })
        .collect();
    nodes.sort_by_key(|n| (n.layer.0, n.order.0, n.id.0));

    MarkTreeSignature {
        arena_points: marks.arena.points.clone(),
        arena_data_indices: marks.arena.data_indices.clone(),
        arena_rects: marks.arena.rects.clone(),
        arena_rect_data_indices: marks.arena.rect_data_indices.clone(),
        nodes,
    }
}

fn run_engine_to_completion(
    engine: &mut ChartEngine,
    measurer: &mut NullTextMeasurer,
    budget: WorkBudget,
    max_steps: usize,
) {
    for _ in 0..max_steps {
        let step = engine.step(measurer, budget).unwrap();
        if !step.unfinished {
            return;
        }
    }
    panic!("engine did not finish within max_steps={max_steps}");
}

#[test]
fn band_missing_upper_breaks_and_preserves_pairing() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let grid_id = crate::ids::GridId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);
    let series_id = crate::ids::SeriesId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y0_field = crate::ids::FieldId::new(2);
    let y1_field = crate::ids::FieldId::new(3);

    let mut spec = ChartSpec {
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
                    id: y0_field,
                    column: 1,
                },
                FieldSpec {
                    id: y1_field,
                    column: 2,
                },
            ],

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    // No dataZoom config: the only break is missing values.
    spec.data_zoom_x = vec![];

    let mut engine = ChartEngine::new(spec).unwrap();
    let mut table = DataTable::default();
    table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0, 4.0]));
    table.push_column(Column::F64(vec![1.0, 2.0, 3.0, 4.0, 5.0]));
    table.push_column(Column::F64(vec![2.0, 3.0, f64::NAN, 5.0, 6.0]));
    engine.datasets_mut().insert(dataset_id, table);

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let marks = &engine.output().marks;
    let mut by_variant: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
    for node in &marks.nodes {
        if node.kind != crate::marks::MarkKind::Polyline || node.source_series != Some(series_id) {
            continue;
        }
        let MarkPayloadRef::Polyline(poly) = &node.payload else {
            continue;
        };
        by_variant.insert(
            crate::ids::mark_variant(node.id),
            marks.arena.data_indices[poly.points.clone()].to_vec(),
        );
    }

    // Segment 0: variants 1/2. Segment 1: variants 3/4.
    assert_eq!(by_variant.get(&1).cloned(), Some(vec![0, 1]));
    assert_eq!(by_variant.get(&2).cloned(), Some(vec![0, 1]));
    assert_eq!(by_variant.get(&3).cloned(), Some(vec![3, 4]));
    assert_eq!(by_variant.get(&4).cloned(), Some(vec![3, 4]));
}

#[test]
fn percent_y_extent_is_scoped_by_x_percent_window() {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_axis = crate::ids::AxisId::new(1);
    let y_axis = crate::ids::AxisId::new(2);

    let mut spec = basic_spec();
    spec.viewport = Some(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(240.0)),
    ));
    spec.data_zoom_x.push(DataZoomXSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: x_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::Filter,
        min_value_span: None,
        max_value_span: None,
    });

    let mut engine = ChartEngine::new(spec).unwrap();

    let xs: Vec<f64> = (0..=100).map(|i| i as f64).collect();
    let ys: Vec<f64> = xs
        .iter()
        .copied()
        .map(|x| if x < 30.0 || x > 70.0 { 1000.0 } else { x })
        .collect();

    let mut table = DataTable::default();
    table.push_column(Column::F64(xs));
    table.push_column(Column::F64(ys));
    engine.datasets_mut().insert(dataset_id, table);

    engine.apply_action(Action::SetAxisWindowPercent {
        axis: x_axis,
        range: Some((30.0, 70.0)),
    });
    engine.apply_action(Action::SetAxisWindowPercent {
        axis: y_axis,
        range: Some((0.0, 100.0)),
    });

    let mut measurer = NullTextMeasurer::default();
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let x_window = engine
        .state()
        .data_zoom_x
        .get(&x_axis)
        .and_then(|s| s.window)
        .expect("expected x window");
    assert!((x_window.min - 30.0).abs() < 1e-9);
    assert!((x_window.max - 70.0).abs() < 1e-9);

    let y_window = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert!(
        (y_window.max - 70.0).abs() < 1e-9,
        "expected Y percent extent to ignore out-of-window outliers (got {y_window:?})"
    );

    let plan = engine.filter_plan_output();
    let grid = plan
        .grids
        .iter()
        .find(|g| g.series.iter().any(|s| *s == crate::ids::SeriesId::new(1)))
        .expect("expected grid output");
    let y_extent = grid
        .y_percent_extents
        .get(&y_axis)
        .copied()
        .expect("expected y percent extent");
    assert!((y_extent.0 - 30.0).abs() < 1e-9);
    assert!((y_extent.1 - 70.0).abs() < 1e-9);

    engine.apply_action(Action::SetAxisWindowPercent {
        axis: x_axis,
        range: Some((40.0, 60.0)),
    });
    let step = engine
        .step(&mut measurer, WorkBudget::new(262_144, 0, 64))
        .unwrap();
    assert!(!step.unfinished);

    let y_window = engine
        .state()
        .data_window_y
        .get(&y_axis)
        .copied()
        .expect("expected y window");
    assert!((y_window.min - 40.0).abs() < 1e-9);
    assert!((y_window.max - 60.0).abs() < 1e-9);

    let plan = engine.filter_plan_output();
    let grid = plan
        .grids
        .iter()
        .find(|g| g.series.iter().any(|s| *s == crate::ids::SeriesId::new(1)))
        .expect("expected grid output");
    let y_extent = grid
        .y_percent_extents
        .get(&y_axis)
        .copied()
        .expect("expected y percent extent");
    assert!((y_extent.0 - 40.0).abs() < 1e-9);
    assert!((y_extent.1 - 60.0).abs() < 1e-9);
}
