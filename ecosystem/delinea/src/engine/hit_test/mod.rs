use fret_core::Point;

use crate::data::DatasetStore;
use crate::engine::HoverHit;
use crate::engine::model::ChartModel;
use crate::marks::{MarkKind, MarkPayloadRef, MarkTree};
use crate::spec::SeriesKind;
use crate::transform::stack_base_at_index;

pub fn hover_hit_test(
    model: &ChartModel,
    datasets: &DatasetStore,
    marks: &MarkTree,
    hover_px: Point,
) -> Option<HoverHit> {
    let mut best: Option<HoverHit> = None;

    let hover_x = hover_px.x.0;
    let hover_y = hover_px.y.0;

    for node in &marks.nodes {
        if node.kind != MarkKind::Polyline && node.kind != MarkKind::Points {
            continue;
        }
        let Some(series_id) = node.source_series else {
            continue;
        };

        let Some(series) = model.series.get(&series_id) else {
            continue;
        };
        if !series.visible {
            continue;
        }
        if series.kind == SeriesKind::Area && series.stack.is_some() {
            let variant = (node.id.0 & 0x7) as u8;
            if variant == 1 {
                continue;
            }
        }

        let table = datasets.dataset(series.dataset);
        let Some(table) = table else {
            continue;
        };

        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };

        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            continue;
        };
        let Some(x) = table.column_f64(x_col) else {
            continue;
        };

        let y_field = if series.kind == SeriesKind::Band {
            let variant = (node.id.0 & 0x7) as u8;
            if variant == 2 {
                let Some(y2) = series.encode.y2 else {
                    continue;
                };
                y2
            } else {
                series.encode.y
            }
        } else {
            series.encode.y
        };
        let Some(y_col) = dataset.fields.get(&y_field).copied() else {
            continue;
        };
        let Some(y) = table.column_f64(y_col) else {
            continue;
        };

        match &node.payload {
            MarkPayloadRef::Polyline(poly) => {
                let points = &marks.arena.points;
                let indices = &marks.arena.data_indices;

                let start = poly.points.start;
                let end = poly.points.end;
                if end > points.len() || end > indices.len() {
                    continue;
                }

                if end.saturating_sub(start) < 2 {
                    continue;
                }

                for global_i in start..(end - 1) {
                    let a = points[global_i];
                    let b = points[global_i + 1];
                    let idx_a = indices[global_i];
                    let idx_b = indices[global_i + 1];

                    let (point_px, t, dist2_px) = closest_point_on_segment(hover_x, hover_y, a, b);

                    let ia = idx_a as usize;
                    let ib = idx_b as usize;
                    if ia >= x.len() || ib >= x.len() || ia >= y.len() || ib >= y.len() {
                        continue;
                    }

                    let t64 = t as f64;
                    let x_value = x[ia] + t64 * (x[ib] - x[ia]);
                    let y0a = y[ia];
                    let y0b = y[ib];
                    if !y0a.is_finite() || !y0b.is_finite() {
                        continue;
                    }

                    let y_eff_a = if series.stack.is_some() {
                        let Some(base) = stack_base_at_index(model, datasets, series_id, ia, y0a)
                        else {
                            continue;
                        };
                        y0a + base.base
                    } else {
                        y0a
                    };
                    let y_eff_b = if series.stack.is_some() {
                        let Some(base) = stack_base_at_index(model, datasets, series_id, ib, y0b)
                        else {
                            continue;
                        };
                        y0b + base.base
                    } else {
                        y0b
                    };

                    let y_value = y_eff_a + t64 * (y_eff_b - y_eff_a);
                    if !x_value.is_finite() || !y_value.is_finite() {
                        continue;
                    }

                    let data_index = if t < 0.5 { idx_a } else { idx_b };
                    let hit = HoverHit {
                        series: series_id,
                        data_index,
                        point_px,
                        dist2_px,
                        x_value,
                        y_value,
                    };

                    if best.is_none_or(|b| hit.dist2_px < b.dist2_px) {
                        best = Some(hit);
                    }
                }
            }
            MarkPayloadRef::Points(points_ref) => {
                let points = &marks.arena.points;
                let indices = &marks.arena.data_indices;

                let start = points_ref.points.start;
                let end = points_ref.points.end;
                if end <= start || end > points.len() || end > indices.len() {
                    continue;
                }

                for global_i in start..end {
                    let p = points[global_i];
                    let idx = indices[global_i] as usize;
                    if idx >= x.len() || idx >= y.len() {
                        continue;
                    }

                    let dx = p.x.0 - hover_x;
                    let dy = p.y.0 - hover_y;
                    let dist2_px = dx * dx + dy * dy;

                    let x_value = x[idx];
                    let y0 = y[idx];
                    let y_value = if series.stack.is_some() {
                        let Some(base) = stack_base_at_index(model, datasets, series_id, idx, y0)
                        else {
                            continue;
                        };
                        y0 + base.base
                    } else {
                        y0
                    };
                    if !x_value.is_finite() || !y_value.is_finite() {
                        continue;
                    }

                    let hit = HoverHit {
                        series: series_id,
                        data_index: indices[global_i],
                        point_px: p,
                        dist2_px,
                        x_value,
                        y_value,
                    };

                    if best.is_none_or(|b| hit.dist2_px < b.dist2_px) {
                        best = Some(hit);
                    }
                }
            }
            _ => {}
        }
    }

    best
}

fn closest_point_on_segment(hover_x: f32, hover_y: f32, a: Point, b: Point) -> (Point, f32, f32) {
    let ax = a.x.0;
    let ay = a.y.0;
    let bx = b.x.0;
    let by = b.y.0;

    let abx = bx - ax;
    let aby = by - ay;
    let apx = hover_x - ax;
    let apy = hover_y - ay;

    let denom = abx * abx + aby * aby;
    let t = if denom > 0.0 && denom.is_finite() {
        (apx * abx + apy * aby) / denom
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    let px = ax + t * abx;
    let py = ay + t * aby;

    let dx = px - hover_x;
    let dy = py - hover_y;
    let dist2 = dx * dx + dy * dy;

    (
        Point {
            x: fret_core::Px(px),
            y: fret_core::Px(py),
        },
        t,
        dist2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::data::{Column, DataTable, DatasetStore};
    use crate::engine::model::ChartModel;
    use crate::ids::{AxisId, ChartId, DatasetId, GridId, LayerId, MarkId, SeriesId, StackId};
    use crate::marks::{
        MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPolylineRef, MarkTree,
    };
    use crate::spec::{
        AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
        SeriesSpec, StackStrategy,
    };

    #[test]
    fn closest_point_projects_onto_segment() {
        let a = Point {
            x: fret_core::Px(0.0),
            y: fret_core::Px(0.0),
        };
        let b = Point {
            x: fret_core::Px(100.0),
            y: fret_core::Px(0.0),
        };

        let (p, t, dist2) = closest_point_on_segment(50.0, 10.0, a, b);
        assert!((t - 0.5).abs() < 1e-6);
        assert!((p.x.0 - 50.0).abs() < 1e-6);
        assert!((p.y.0 - 0.0).abs() < 1e-6);
        assert!((dist2 - 100.0).abs() < 1e-6);
    }

    #[test]
    fn hover_hit_returns_interpolated_values() {
        let chart_id = ChartId::new(1);
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let series_id = SeriesId::new(1);
        let x_field = crate::ids::FieldId::new(1);
        let y_field = crate::ids::FieldId::new(2);

        let spec = ChartSpec {
            id: chart_id,
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

        let model = ChartModel::from_spec(spec).unwrap();

        let mut store = DatasetStore::default();
        store.insert(dataset_id, {
            let mut t = DataTable::default();
            t.push_column(Column::F64(vec![0.0, 10.0]));
            t.push_column(Column::F64(vec![0.0, 0.0]));
            t
        });

        let mut marks = MarkTree::default();
        let range = marks.arena.extend_points_with_indices(
            [
                Point {
                    x: fret_core::Px(0.0),
                    y: fret_core::Px(0.0),
                },
                Point {
                    x: fret_core::Px(100.0),
                    y: fret_core::Px(0.0),
                },
            ],
            [0u32, 1u32],
        );
        marks.nodes.push(MarkNode {
            id: MarkId::new(1),
            parent: None,
            layer: LayerId::new(1),
            order: MarkOrderKey(0),
            kind: MarkKind::Polyline,
            source_series: Some(series_id),
            payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                points: range,
                stroke: None,
            }),
        });

        let hit = hover_hit_test(
            &model,
            &store,
            &marks,
            Point {
                x: fret_core::Px(50.0),
                y: fret_core::Px(10.0),
            },
        )
        .unwrap();

        assert_eq!(hit.series, series_id);
        assert!((hit.x_value - 5.0).abs() < 1e-9);
        assert!((hit.y_value - 0.0).abs() < 1e-9);
        assert!((hit.point_px.x.0 - 50.0).abs() < 1e-6);
        assert!((hit.point_px.y.0 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn hover_hit_returns_stacked_y_for_stacked_series() {
        let chart_id = ChartId::new(1);
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let stack = StackId::new(1);
        let series_a = SeriesId::new(1);
        let series_b = SeriesId::new(2);
        let x_field = crate::ids::FieldId::new(1);
        let y_a_field = crate::ids::FieldId::new(2);
        let y_b_field = crate::ids::FieldId::new(3);

        let spec = ChartSpec {
            id: chart_id,
            viewport: None,
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
                    stack_strategy: StackStrategy::All,
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
                    stack_strategy: StackStrategy::All,
                    area_baseline: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut store = DatasetStore::default();
        store.insert(dataset_id, {
            let mut t = DataTable::default();
            t.push_column(Column::F64(vec![0.0, 10.0]));
            t.push_column(Column::F64(vec![1.0, 1.0]));
            t.push_column(Column::F64(vec![2.0, 2.0]));
            t
        });

        let mut marks = MarkTree::default();
        let range = marks.arena.extend_points_with_indices(
            [
                Point {
                    x: fret_core::Px(0.0),
                    y: fret_core::Px(0.0),
                },
                Point {
                    x: fret_core::Px(100.0),
                    y: fret_core::Px(0.0),
                },
            ],
            [0u32, 1u32],
        );

        marks.nodes.push(MarkNode {
            id: MarkId::new(1),
            parent: None,
            layer: LayerId::new(1),
            order: MarkOrderKey(0),
            kind: MarkKind::Polyline,
            source_series: Some(series_b),
            payload: MarkPayloadRef::Polyline(MarkPolylineRef {
                points: range,
                stroke: None,
            }),
        });

        let hit = hover_hit_test(
            &model,
            &store,
            &marks,
            Point::new(fret_core::Px(50.0), fret_core::Px(0.0)),
        )
        .expect("expected a hit");

        // Interpolated at x=5 -> base=1, y=2 => stacked y=3
        assert!((hit.y_value - 3.0).abs() < 1e-9);
        assert!((hit.x_value - 5.0).abs() < 1e-9);
    }
}
