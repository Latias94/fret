use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{Revision, SeriesId};
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;

#[derive(Debug, Clone)]
pub(super) struct CachedXRangeNode {
    pub signature: u64,
    pub selection: RowSelection,
}

impl super::TransformGraph {
    pub fn x_range_selection_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: SeriesId,
        series_view_index: usize,
    ) -> Option<RowSelection> {
        let Some(series_model) = model.series.get(&series_id) else {
            return None;
        };

        let series_view = &view.series[series_view_index];

        if !matches!(
            series_view.x_filter_mode,
            crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
        ) {
            return None;
        }
        if !matches!(
            series_view.selection,
            RowSelection::All | RowSelection::Range(_)
        ) {
            return None;
        }

        let x_filter = series_view.x_policy.filter;
        if x_filter.min.is_none() && x_filter.max.is_none() {
            return None;
        }

        let Some(dataset_view) = view.dataset_view(series_model.dataset) else {
            return None;
        };
        let base_range = dataset_view.row_range;

        let root = model.root_dataset_id(series_model.dataset);
        let Some(table) = datasets.dataset(root) else {
            return None;
        };
        let Some(dataset) = model.datasets.get(&series_model.dataset) else {
            return None;
        };
        let Some(x_col) = dataset.fields.get(&series_model.encode.x).copied() else {
            return None;
        };
        let Some(x_values) = table.column_f64(x_col) else {
            return None;
        };

        let signature = x_range_signature(
            model.revs.spec,
            table.revision(),
            series_id,
            base_range,
            x_col,
            series_view.x_filter_mode,
            x_filter,
        );
        if let Some(cached) = self.x_range_cache.get(&series_id)
            && cached.signature == signature
        {
            return Some(cached.selection.clone());
        }

        let selection =
            crate::transform::row_selection_for_x_filter(x_values, base_range, x_filter);
        self.x_range_cache.insert(
            series_id,
            CachedXRangeNode {
                signature,
                selection: selection.clone(),
            },
        );
        Some(selection)
    }
}

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x00000100000001B3;

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV1A_PRIME)
}

fn hash_opt_f64(mut h: u64, v: Option<f64>) -> u64 {
    match v {
        Some(v) => {
            h = fnv1a_step(h, 1);
            fnv1a_step(h, v.to_bits())
        }
        None => fnv1a_step(h, 0),
    }
}

fn hash_filter(mut h: u64, f: AxisFilter1D) -> u64 {
    h = hash_opt_f64(h, f.min);
    hash_opt_f64(h, f.max)
}

fn x_range_signature(
    model_rev: Revision,
    data_rev: Revision,
    series_id: SeriesId,
    base_range: RowRange,
    x_col: usize,
    x_filter_mode: crate::spec::FilterMode,
    x_filter: AxisFilter1D,
) -> u64 {
    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, model_rev.0 as u64);
    h = fnv1a_step(h, data_rev.0 as u64);
    h = fnv1a_step(h, series_id.0);
    h = fnv1a_step(h, x_col as u64);
    h = fnv1a_step(h, base_range.start as u64);
    h = fnv1a_step(h, base_range.end as u64);
    h = fnv1a_step(h, x_filter_mode as u64);
    hash_filter(h, x_filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, DataTable};
    use crate::engine::model::ChartModel;
    use crate::ids::{AxisId, DatasetId, FieldId, GridId};
    use crate::spec::{
        AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, FilterMode, GridSpec, SeriesEncode,
        SeriesKind, SeriesSpec,
    };
    use crate::transform::RowSelection;
    use crate::view::ViewState;

    #[test]
    fn x_range_node_cache_hits_and_invalidates() {
        let dataset_id = DatasetId::new(1);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let series_id = SeriesId::new(1);

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

        let model = ChartModel::from_spec(spec).unwrap();
        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64((0..=100).map(|v| v as f64).collect()));
        table.push_column(Column::F64((0..=100).map(|v| v as f64).collect()));
        datasets.insert(dataset_id, table);

        let mut view = ViewState::default();
        view.rebuild(&model, &datasets, &Default::default());
        view.series[0].x_filter_mode = FilterMode::Filter;
        view.series[0].x_policy.filter = AxisFilter1D {
            min: Some(20.0),
            max: Some(40.0),
        };

        let mut graph = super::super::TransformGraph::default();
        let sel = graph
            .x_range_selection_for_series(&model, &datasets, &view, series_id, 0)
            .expect("expected selection");
        assert_eq!(sel, RowSelection::Range(RowRange { start: 20, end: 41 }));

        let cached = graph
            .x_range_cache
            .get_mut(&series_id)
            .expect("expected cache entry");
        cached.selection = RowSelection::Range(RowRange { start: 1, end: 2 });

        let sel = graph
            .x_range_selection_for_series(&model, &datasets, &view, series_id, 0)
            .expect("expected selection");
        assert_eq!(sel, RowSelection::Range(RowRange { start: 1, end: 2 }));

        // Changing the filter invalidates the cache.
        view.series[0].x_policy.filter = AxisFilter1D {
            min: Some(10.0),
            max: Some(15.0),
        };
        let sel = graph
            .x_range_selection_for_series(&model, &datasets, &view, series_id, 0)
            .expect("expected selection");
        assert_eq!(sel, RowSelection::Range(RowRange { start: 10, end: 16 }));
    }
}
