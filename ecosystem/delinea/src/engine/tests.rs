use std::collections::BTreeMap;

use crate::action::Action;
use crate::data::{Column, DataTable};
use crate::engine::ChartEngine;
use crate::engine::window::DataWindow;
use crate::marks::MarkPayloadRef;
use crate::scheduler::WorkBudget;
use crate::spec::{
    AxisKind, AxisPointerSpec, AxisRange, AxisSpec, ChartSpec, DataZoomXSpec, DataZoomYSpec,
    DatasetSpec, FieldSpec, FilterMode, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
    VisualMapMode, VisualMapSpec,
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Item,
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: crate::spec::AxisPointerTrigger::Axis,
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
    let appended_index = (table.row_count - 1) as u32;

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
