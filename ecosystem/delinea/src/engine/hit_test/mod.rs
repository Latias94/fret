use fret_core::Point;
use std::collections::HashMap;

use crate::data::DatasetStore;
use crate::engine::HoverHit;
use crate::engine::model::ChartModel;
use crate::engine::stages::StackDimsStage;
use crate::marks::{MarkKind, MarkPayloadRef, MarkTree};
use crate::spec::SeriesKind;
use crate::transform::stack_base_at_index;

pub fn hover_hit_test(
    model: &ChartModel,
    datasets: &DatasetStore,
    marks: &MarkTree,
    hover_px: Point,
    stack_dims: &StackDimsStage,
) -> Option<HoverHit> {
    let mut best: Option<HoverHit> = None;
    let mut best_rank: Option<u32> = None;

    let mut rank_by_series = HashMap::new();
    for (i, series_id) in model.series_order.iter().enumerate() {
        rank_by_series.insert(*series_id, i as u32);
    }

    let hover_x = hover_px.x.0;
    let hover_y = hover_px.y.0;

    for node in &marks.nodes {
        if node.kind != MarkKind::Polyline
            && node.kind != MarkKind::Points
            && node.kind != MarkKind::Rect
        {
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
            let variant = crate::ids::mark_variant(node.id);
            if variant == 1 {
                continue;
            }
        }

        let table = datasets.dataset(model.root_dataset_id(series.dataset));
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
            let variant = crate::ids::mark_variant(node.id);
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

                    let y_eff_a = if let Some(stack) = series.stack {
                        stack_dims
                            .stacked_y(stack, series_id, ia, model.revs.marks, table.revision())
                            .unwrap_or_else(|| {
                                stack_base_at_index(model, datasets, series_id, ia, y0a)
                                    .map(|b| y0a + b.base)
                                    .unwrap_or(y0a)
                            })
                    } else {
                        y0a
                    };
                    let y_eff_b = if let Some(stack) = series.stack {
                        stack_dims
                            .stacked_y(stack, series_id, ib, model.revs.marks, table.revision())
                            .unwrap_or_else(|| {
                                stack_base_at_index(model, datasets, series_id, ib, y0b)
                                    .map(|b| y0b + b.base)
                                    .unwrap_or(y0b)
                            })
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

                    let rank = rank_by_series.get(&series_id).copied().unwrap_or(u32::MAX);
                    if best.is_none() {
                        best = Some(hit);
                        best_rank = Some(rank);
                        continue;
                    }

                    let Some(current) = best else {
                        continue;
                    };
                    let current_rank = best_rank.unwrap_or(u32::MAX);

                    let eps = 1e-6_f32;
                    if hit.dist2_px + eps < current.dist2_px {
                        best = Some(hit);
                        best_rank = Some(rank);
                    } else if (hit.dist2_px - current.dist2_px).abs() <= eps {
                        // Deterministic tie-break: prefer earlier `series_order` so hover/tooltip
                        // does not flicker across refactors that may reorder marks.
                        if rank < current_rank {
                            best = Some(hit);
                            best_rank = Some(rank);
                        } else if rank == current_rank && hit.data_index < current.data_index {
                            best = Some(hit);
                            best_rank = Some(rank);
                        }
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
                    let y_value = if let Some(stack) = series.stack {
                        stack_dims
                            .stacked_y(stack, series_id, idx, model.revs.marks, table.revision())
                            .unwrap_or_else(|| {
                                stack_base_at_index(model, datasets, series_id, idx, y0)
                                    .map(|b| y0 + b.base)
                                    .unwrap_or(y0)
                            })
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

                    let rank = rank_by_series.get(&series_id).copied().unwrap_or(u32::MAX);
                    if best.is_none() {
                        best = Some(hit);
                        best_rank = Some(rank);
                        continue;
                    }

                    let Some(current) = best else {
                        continue;
                    };
                    let current_rank = best_rank.unwrap_or(u32::MAX);

                    let eps = 1e-6_f32;
                    if hit.dist2_px + eps < current.dist2_px {
                        best = Some(hit);
                        best_rank = Some(rank);
                    } else if (hit.dist2_px - current.dist2_px).abs() <= eps {
                        if rank < current_rank {
                            best = Some(hit);
                            best_rank = Some(rank);
                        } else if rank == current_rank && hit.data_index < current.data_index {
                            best = Some(hit);
                            best_rank = Some(rank);
                        }
                    }
                }
            }
            MarkPayloadRef::Rect(rects_ref) => {
                let rects = &marks.arena.rects;
                let indices = &marks.arena.rect_data_indices;

                let is_horizontal_bar = series.kind == SeriesKind::Bar
                    && crate::engine::bar::bar_mapping_for_series(model, series_id).is_some_and(
                        |m| m.orientation == crate::engine::bar::BarOrientation::Horizontal,
                    );

                let start = rects_ref.rects.start;
                let end = rects_ref.rects.end;
                if end <= start || end > rects.len() || end > indices.len() {
                    continue;
                }

                for global_i in start..end {
                    let rect = rects[global_i];
                    let idx = indices[global_i] as usize;
                    if idx >= x.len() || idx >= y.len() {
                        continue;
                    }

                    let (point_px, dist2_px) = closest_point_in_rect(hover_x, hover_y, rect);

                    let cat_value = if is_horizontal_bar { y[idx] } else { x[idx] };
                    let value0 = if is_horizontal_bar { x[idx] } else { y[idx] };

                    let value = if let Some(stack) = series.stack {
                        stack_dims
                            .stacked_value(
                                stack,
                                series_id,
                                idx,
                                model.revs.marks,
                                table.revision(),
                            )
                            .unwrap_or_else(|| {
                                if is_horizontal_bar {
                                    value0
                                } else {
                                    stack_base_at_index(model, datasets, series_id, idx, value0)
                                        .map(|b| value0 + b.base)
                                        .unwrap_or(value0)
                                }
                            })
                    } else {
                        value0
                    };

                    let (x_value, y_value) = if is_horizontal_bar {
                        (value, cat_value)
                    } else {
                        (cat_value, value)
                    };
                    if !x_value.is_finite() || !y_value.is_finite() {
                        continue;
                    }

                    let hit = HoverHit {
                        series: series_id,
                        data_index: indices[global_i],
                        point_px,
                        dist2_px,
                        x_value,
                        y_value,
                    };

                    let rank = rank_by_series.get(&series_id).copied().unwrap_or(u32::MAX);
                    if best.is_none() {
                        best = Some(hit);
                        best_rank = Some(rank);
                        continue;
                    }

                    let Some(current) = best else {
                        continue;
                    };
                    let current_rank = best_rank.unwrap_or(u32::MAX);

                    let eps = 1e-6_f32;
                    if hit.dist2_px + eps < current.dist2_px {
                        best = Some(hit);
                        best_rank = Some(rank);
                    } else if (hit.dist2_px - current.dist2_px).abs() <= eps {
                        if rank < current_rank {
                            best = Some(hit);
                            best_rank = Some(rank);
                        } else if rank == current_rank && hit.data_index < current.data_index {
                            best = Some(hit);
                            best_rank = Some(rank);
                        }
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

fn closest_point_in_rect(hover_x: f32, hover_y: f32, rect: fret_core::Rect) -> (Point, f32) {
    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = left + rect.size.width.0;
    let bottom = top + rect.size.height.0;

    let px = hover_x.clamp(left, right);
    let py = hover_y.clamp(top, bottom);

    let dx = px - hover_x;
    let dy = py - hover_y;
    let dist2 = dx * dx + dy * dy;

    (
        Point {
            x: fret_core::Px(px),
            y: fret_core::Px(py),
        },
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
        MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPointsRef, MarkPolylineRef, MarkTree,
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
            &StackDimsStage::default(),
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
                    stack_strategy: StackStrategy::All,
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
                    stack_strategy: StackStrategy::All,
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
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
            &StackDimsStage::default(),
        )
        .expect("expected a hit");

        // Interpolated at x=5 -> base=1, y=2 => stacked y=3
        assert!((hit.y_value - 3.0).abs() < 1e-9);
        assert!((hit.x_value - 5.0).abs() < 1e-9);
    }

    #[test]
    fn hover_hit_returns_bar_hit_for_rect_marks() {
        let chart_id = ChartId::new(1);
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let series_id = SeriesId::new(1);
        let x_field = crate::ids::FieldId::new(1);
        let y_field = crate::ids::FieldId::new(2);

        let spec = crate::spec::ChartSpec {
            id: chart_id,
            viewport: None,
            datasets: vec![crate::spec::DatasetSpec {
                id: dataset_id,
                fields: vec![
                    crate::spec::FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    crate::spec::FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],

                from: None,
                transforms: Vec::new(),
            }],
            grids: vec![crate::spec::GridSpec { id: grid_id }],
            axes: vec![
                crate::spec::AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: crate::spec::AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                        categories: vec!["A".into()],
                    }),
                    range: None,
                },
                crate::spec::AxisSpec {
                    id: y_axis,
                    name: None,
                    kind: crate::spec::AxisKind::Y,
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
            series: vec![crate::spec::SeriesSpec {
                id: series_id,
                name: None,
                kind: crate::spec::SeriesKind::Bar,
                dataset: dataset_id,
                encode: crate::spec::SeriesEncode {
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

        let model = ChartModel::from_spec(spec).unwrap();

        let mut store = DatasetStore::default();
        store.insert(dataset_id, {
            let mut t = DataTable::default();
            t.push_column(Column::F64(vec![0.0]));
            t.push_column(Column::F64(vec![2.0]));
            t
        });

        let mut marks = MarkTree::default();
        marks.arena.rects.push(fret_core::Rect::new(
            Point {
                x: fret_core::Px(10.0),
                y: fret_core::Px(20.0),
            },
            fret_core::Size::new(fret_core::Px(40.0), fret_core::Px(50.0)),
        ));
        marks.arena.rect_data_indices.push(0);
        marks.nodes.push(crate::marks::MarkNode {
            id: MarkId::new(1),
            parent: None,
            layer: LayerId::new(1),
            order: crate::marks::MarkOrderKey(0),
            kind: MarkKind::Rect,
            source_series: Some(series_id),
            payload: MarkPayloadRef::Rect(crate::marks::MarkRectRef {
                rects: 0..1,
                fill: None,
                opacity_mul: None,
                stroke: None,
            }),
        });

        let hit = hover_hit_test(
            &model,
            &store,
            &marks,
            Point {
                x: fret_core::Px(15.0),
                y: fret_core::Px(25.0),
            },
            &StackDimsStage::default(),
        )
        .unwrap();

        assert_eq!(hit.series, series_id);
        assert_eq!(hit.data_index, 0);
        assert_eq!(hit.x_value, 0.0);
        assert_eq!(hit.y_value, 2.0);
        assert!((hit.dist2_px - 0.0).abs() < 1e-6);
        assert!((hit.point_px.x.0 - 15.0).abs() < 1e-6);
        assert!((hit.point_px.y.0 - 25.0).abs() < 1e-6);
    }

    #[test]
    fn hover_hit_ties_prefer_series_order_over_mark_order() {
        let chart_id = ChartId::new(1);
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let series_a = SeriesId::new(1);
        let series_b = SeriesId::new(2);
        let x_field = crate::ids::FieldId::new(1);
        let y_field = crate::ids::FieldId::new(2);

        // `series_a` is earlier in `series_order`, but we will insert `series_b` marks first
        // to ensure hit selection is deterministic and independent of mark iteration order.
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
                    name: Some("A".to_string()),
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
                    stack_strategy: StackStrategy::SameSign,
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: series_b,
                    name: Some("B".to_string()),
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
                    stack_strategy: StackStrategy::SameSign,
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut store = DatasetStore::default();
        store.insert(dataset_id, {
            let mut t = DataTable::default();
            t.push_column(Column::F64(vec![1.0]));
            t.push_column(Column::F64(vec![2.0]));
            t
        });

        let mut marks = MarkTree::default();

        let point = Point {
            x: fret_core::Px(10.0),
            y: fret_core::Px(20.0),
        };

        let range_b = marks.arena.extend_points_with_indices([point], [0u32]);
        marks.nodes.push(MarkNode {
            id: MarkId::new(1),
            parent: None,
            layer: LayerId::new(1),
            order: MarkOrderKey(0),
            kind: MarkKind::Points,
            source_series: Some(series_b),
            payload: MarkPayloadRef::Points(MarkPointsRef {
                points: range_b,
                fill: None,
                opacity_mul: None,
                radius_mul: None,
                stroke: None,
            }),
        });

        let range_a = marks.arena.extend_points_with_indices([point], [0u32]);
        marks.nodes.push(MarkNode {
            id: MarkId::new(2),
            parent: None,
            layer: LayerId::new(1),
            order: MarkOrderKey(0),
            kind: MarkKind::Points,
            source_series: Some(series_a),
            payload: MarkPayloadRef::Points(MarkPointsRef {
                points: range_a,
                fill: None,
                opacity_mul: None,
                radius_mul: None,
                stroke: None,
            }),
        });

        let hit = hover_hit_test(&model, &store, &marks, point, &StackDimsStage::default())
            .expect("expected a hit");
        assert_eq!(hit.dist2_px, 0.0);
        assert_eq!(hit.series, series_a);
    }
}
