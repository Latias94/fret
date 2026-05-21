use std::collections::BTreeMap;

use delinea::marks::{MarkPayloadRef, MarkTree};
use fret_core::{Point, Px};

#[derive(Debug, Default, Clone)]
pub(crate) struct ChartA11yIndex {
    pub(crate) point_by_series_and_index: BTreeMap<(delinea::SeriesId, u32), (u32, Point)>,
    pub(crate) indices_by_series: BTreeMap<delinea::SeriesId, Vec<u32>>,
    pub(crate) series_by_index: BTreeMap<u32, Vec<delinea::SeriesId>>,
}

impl ChartA11yIndex {
    pub(crate) fn clear(&mut self) {
        self.point_by_series_and_index.clear();
        self.indices_by_series.clear();
        self.series_by_index.clear();
    }

    pub(crate) fn point(&self, series: delinea::SeriesId, data_index: u32) -> Option<Point> {
        self.point_by_series_and_index
            .get(&(series, data_index))
            .map(|(_, point)| *point)
    }

    pub(crate) fn rebuild(
        &mut self,
        marks: &MarkTree,
        series_rank_by_id: &BTreeMap<delinea::SeriesId, usize>,
    ) {
        self.clear();

        let rect_indices_available = marks.arena.rect_data_indices.len() == marks.arena.rects.len();
        let point_indices_available = marks.arena.data_indices.len() == marks.arena.points.len();

        for node in &marks.nodes {
            let series = node
                .source_series
                .or_else(|| {
                    let from_layer = delinea::SeriesId::new(node.layer.0);
                    (from_layer.0 != 0).then_some(from_layer)
                })
                .or_else(|| {
                    let inferred =
                        delinea::SeriesId::new(node.id.0 >> delinea::ids::MARK_VARIANT_BITS);
                    (inferred.0 != 0).then_some(inferred)
                })
                .unwrap_or_else(|| delinea::SeriesId::new(1));

            match &node.payload {
                MarkPayloadRef::Polyline(polyline) => {
                    let start = polyline.points.start;
                    let end = polyline.points.end.min(marks.arena.points.len());
                    for i in start..end {
                        let point = marks.arena.points[i];
                        let data_index = if point_indices_available {
                            marks.arena.data_indices[i]
                        } else {
                            u32::try_from(i.saturating_sub(start)).unwrap_or(0)
                        };
                        self.insert_point(series, data_index, node.order.0, point);
                    }
                }
                MarkPayloadRef::Rect(rects) => {
                    let start = rects.rects.start;
                    let end = rects.rects.end.min(marks.arena.rects.len());
                    for i in start..end {
                        let rect = marks.arena.rects[i];
                        let data_index = if rect_indices_available {
                            marks.arena.rect_data_indices[i]
                        } else {
                            u32::try_from(i.saturating_sub(start)).unwrap_or(0)
                        };
                        let center = Point::new(
                            Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                            Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
                        );
                        self.insert_point(series, data_index, node.order.0, center);
                    }
                }
                MarkPayloadRef::Points(points) => {
                    let start = points.points.start;
                    let end = points.points.end.min(marks.arena.points.len());
                    for i in start..end {
                        let point = marks.arena.points[i];
                        let data_index = if point_indices_available {
                            marks.arena.data_indices[i]
                        } else {
                            u32::try_from(i.saturating_sub(start)).unwrap_or(0)
                        };
                        self.insert_point(series, data_index, node.order.0, point);
                    }
                }
                _ => {}
            }
        }

        for (series, data_index) in self.point_by_series_and_index.keys() {
            self.indices_by_series
                .entry(*series)
                .or_default()
                .push(*data_index);
            self.series_by_index
                .entry(*data_index)
                .or_default()
                .push(*series);
        }

        for indices in self.indices_by_series.values_mut() {
            indices.sort_unstable();
            indices.dedup();
        }

        for series in self.series_by_index.values_mut() {
            series.sort_by_key(|id| series_rank_by_id.get(id).copied().unwrap_or(usize::MAX));
            series.dedup();
        }
    }

    fn insert_point(
        &mut self,
        series: delinea::SeriesId,
        data_index: u32,
        order: u32,
        point: Point,
    ) {
        let entry = self
            .point_by_series_and_index
            .entry((series, data_index))
            .or_insert((order, point));
        if order > entry.0 {
            *entry = (order, point);
        }
    }
}
