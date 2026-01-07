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
        if series.kind != SeriesKind::Line || !series.visible {
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
        let Some(y) = table.column_f64(series.y_col) else {
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

        for (local_i, p) in points[start..end].iter().enumerate() {
            let dx = p.x.0 - hover_px.x.0;
            let dy = p.y.0 - hover_px.y.0;
            let dist2 = dx * dx + dy * dy;

            let global_i = start + local_i;
            let data_i = indices[global_i] as usize;
            if data_i >= x.len() || data_i >= y.len() {
                continue;
            }

            let hit = HoverHit {
                series: series_id,
                data_index: indices[global_i],
                point_px: *p,
                dist2_px: dist2,
            };

            if best.is_none_or(|b| hit.dist2_px < b.dist2_px) {
                best = Some(hit);
            }
        }
    }

    best
}
