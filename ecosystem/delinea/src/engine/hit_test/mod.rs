use fret_core::Point;

use crate::data::DatasetStore;
use crate::engine::HoverHit;
use crate::engine::model::ChartModel;
use crate::marks::{MarkKind, MarkPayloadRef, MarkTree};
use crate::spec::SeriesKind;

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
        if node.kind != MarkKind::Polyline {
            continue;
        }
        let Some(series_id) = node.source_series else {
            continue;
        };

        let Some(series) = model.series.get(&series_id) else {
            continue;
        };
        if !matches!(
            series.kind,
            SeriesKind::Line | SeriesKind::Area | SeriesKind::Band
        ) || !series.visible
        {
            continue;
        }

        let table = datasets
            .datasets
            .iter()
            .find_map(|(id, t)| (*id == series.dataset).then_some(t));
        let Some(table) = table else {
            continue;
        };

        let Some(x) = table.column_f64(series.x_col) else {
            continue;
        };
        let y_col = if series.kind == SeriesKind::Band {
            let variant = (node.id.0 & 0x7) as u8;
            if variant == 2 {
                let Some(y2) = series.y2_col else {
                    continue;
                };
                y2
            } else {
                series.y_col
            }
        } else {
            series.y_col
        };
        let Some(y) = table.column_f64(y_col) else {
            continue;
        };

        let MarkPayloadRef::Polyline(poly) = &node.payload else {
            continue;
        };

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
            let y_value = y[ia] + t64 * (y[ib] - y[ia]);
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
    use crate::ids::{AxisId, ChartId, DatasetId, GridId, LayerId, MarkId, SeriesId};
    use crate::marks::{
        MarkKind, MarkNode, MarkOrderKey, MarkPayloadRef, MarkPolylineRef, MarkTree,
    };
    use crate::spec::{
        AxisKind, AxisSpec, ChartSpec, DatasetSpec, GridSpec, SeriesKind, SeriesSpec,
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

        let spec = ChartSpec {
            id: chart_id,
            viewport: None,
            datasets: vec![DatasetSpec { id: dataset_id }],
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
                x_col: 0,
                y_col: 1,
                y2_col: None,
                x_axis,
                y_axis,
                area_baseline: None,
            }],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut store = DatasetStore::default();
        store.datasets.push((dataset_id, {
            let mut t = DataTable::default();
            t.push_column(Column::F64(vec![0.0, 10.0]));
            t.push_column(Column::F64(vec![0.0, 0.0]));
            t
        }));

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
}
