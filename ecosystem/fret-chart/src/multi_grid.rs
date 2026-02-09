use std::collections::{BTreeMap, BTreeSet};

use delinea::engine::model::ModelError;
use delinea::ids::{AxisId, ChartId, GridId, SeriesId};
use delinea::spec::{ChartSpec, GridSpec, VisualMapSpec};

#[derive(Debug, Clone)]
pub struct SplitByGrid {
    pub grid: GridId,
    pub spec: ChartSpec,
}

fn collect_grids(spec: &ChartSpec) -> Vec<GridId> {
    if !spec.grids.is_empty() {
        return spec.grids.iter().map(|g| g.id).collect();
    }

    let mut ids: BTreeSet<GridId> = spec.axes.iter().map(|a| a.grid).collect();
    if ids.is_empty() {
        ids.insert(GridId::new(1));
    }
    ids.into_iter().collect()
}

fn filter_visual_maps_for_series(
    maps: &[VisualMapSpec],
    series: &[delinea::spec::SeriesSpec],
    series_ids: &BTreeSet<SeriesId>,
) -> Vec<VisualMapSpec> {
    maps.iter()
        .filter_map(|map| {
            if let Some(dataset) = map.dataset {
                let any = series.iter().any(|s| s.dataset == dataset);
                return any.then(|| map.clone());
            }

            if map.series.is_empty() {
                return None;
            }

            let mut next = map.clone();
            next.series.retain(|id| series_ids.contains(id));
            (!next.series.is_empty()).then(|| next)
        })
        .collect()
}

/// Splits a multi-grid `ChartSpec` into multiple single-grid `ChartSpec`s.
///
/// This is a pragmatic UI adapter helper for the current v1 rendering stack, which assumes a
/// single viewport per chart.
///
/// Current (v1): `fret-chart` hosts one `ChartEngine` per grid and lays multiple canvases out in
/// the UI (see `crate::retained::multi_grid` and `crate::declarative::multi_grid`).
///
/// Target: a single engine instance owns per-grid viewports/layout and emits deterministic per-grid
/// outputs (workstream milestone M1: `docs/workstreams/delinea-engine-contract-closure-v1.md`).
///
/// Each returned `ChartSpec`:
/// - keeps all datasets (shared),
/// - keeps exactly one grid,
/// - filters axes/series/dataZoom/visualMap to what is reachable from that grid.
pub fn split_chart_spec_by_grid(spec: &ChartSpec) -> Result<Vec<SplitByGrid>, ModelError> {
    let grids = collect_grids(spec);
    if grids.len() == 1 {
        let grid_id = grids[0];
        let mut next = spec.clone();
        if next.grids.is_empty() {
            next.grids = vec![GridSpec { id: grid_id }];
        }
        return Ok(vec![SplitByGrid {
            grid: grid_id,
            spec: next,
        }]);
    }

    let grid_specs: BTreeMap<GridId, GridSpec> =
        spec.grids.iter().cloned().map(|g| (g.id, g)).collect();

    let mut out = Vec::with_capacity(grids.len());
    for (i, grid_id) in grids.iter().copied().enumerate() {
        let axes: Vec<_> = spec
            .axes
            .iter()
            .filter(|a| a.grid == grid_id)
            .cloned()
            .collect();
        let axis_ids: BTreeSet<AxisId> = axes.iter().map(|a| a.id).collect();

        let series: Vec<_> = spec
            .series
            .iter()
            .filter(|s| axis_ids.contains(&s.x_axis) && axis_ids.contains(&s.y_axis))
            .cloned()
            .collect();
        let series_ids: BTreeSet<SeriesId> = series.iter().map(|s| s.id).collect();

        let data_zoom_x: Vec<_> = spec
            .data_zoom_x
            .iter()
            .filter(|z| axis_ids.contains(&z.axis))
            .cloned()
            .collect();
        let data_zoom_y: Vec<_> = spec
            .data_zoom_y
            .iter()
            .filter(|z| axis_ids.contains(&z.axis))
            .cloned()
            .collect();

        let visual_maps = filter_visual_maps_for_series(&spec.visual_maps, &series, &series_ids);

        let grid_spec = grid_specs
            .get(&grid_id)
            .cloned()
            .unwrap_or(GridSpec { id: grid_id });

        let child_id = ChartId::new(spec.id.0.wrapping_mul(1024).wrapping_add(i as u64 + 1));

        out.push(SplitByGrid {
            grid: grid_id,
            spec: ChartSpec {
                id: child_id,
                viewport: None,
                datasets: spec.datasets.clone(),
                grids: vec![grid_spec],
                axes,
                data_zoom_x,
                data_zoom_y,
                tooltip: spec.tooltip.clone(),
                axis_pointer: spec.axis_pointer.clone(),
                visual_maps,
                series,
            },
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use delinea::ids::{AxisId, DatasetId, FieldId, VisualMapId};
    use delinea::scale::AxisScale;
    use delinea::spec::{
        AxisKind, DatasetSpec, FieldSpec, SeriesEncode, SeriesKind, SeriesSpec, VisualMapMode,
    };

    #[test]
    fn split_filters_visual_maps_by_grid_series() {
        let dataset = DatasetId::new(1);
        let field_x = FieldId::new(1);
        let field_y = FieldId::new(2);

        let grid_a = GridId::new(1);
        let grid_b = GridId::new(2);

        let x_a = AxisId::new(1);
        let y_a = AxisId::new(2);
        let x_b = AxisId::new(3);
        let y_b = AxisId::new(4);

        let s_a = SeriesId::new(1);
        let s_b = SeriesId::new(2);

        let spec = ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset,
                fields: vec![
                    FieldSpec {
                        id: field_x,
                        column: 0,
                    },
                    FieldSpec {
                        id: field_y,
                        column: 1,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_a }, GridSpec { id: grid_b }],
            axes: vec![
                delinea::spec::AxisSpec {
                    id: x_a,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_a,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::spec::AxisSpec {
                    id: y_a,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_a,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::spec::AxisSpec {
                    id: x_b,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_b,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::spec::AxisSpec {
                    id: y_b,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_b,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![VisualMapSpec {
                id: VisualMapId::new(1),
                mode: VisualMapMode::Continuous,
                dataset: None,
                series: vec![s_a],
                field: field_y,
                domain: (0.0, 1.0),
                initial_range: None,
                initial_piece_mask: None,
                point_radius_mul_range: None,
                stroke_width_range: None,
                opacity_mul_range: None,
                buckets: 8,
                out_of_range_opacity: 0.25,
            }],
            series: vec![
                SeriesSpec {
                    id: s_a,
                    name: None,
                    kind: SeriesKind::Scatter,
                    dataset,
                    encode: SeriesEncode {
                        x: field_x,
                        y: field_y,
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
                    dataset,
                    encode: SeriesEncode {
                        x: field_x,
                        y: field_y,
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

        let split = split_chart_spec_by_grid(&spec).expect("split ok");
        assert_eq!(split.len(), 2);

        let a = split.iter().find(|s| s.grid == grid_a).unwrap();
        let b = split.iter().find(|s| s.grid == grid_b).unwrap();

        assert_eq!(a.spec.visual_maps.len(), 1);
        assert_eq!(b.spec.visual_maps.len(), 0);
    }
}
