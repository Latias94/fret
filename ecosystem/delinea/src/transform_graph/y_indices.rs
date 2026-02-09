use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{Revision, SeriesId};
use crate::transform::RowSelection;
use crate::view::ViewState;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct YIndicesNodeOutput {
    pub result: YIndicesNodeResult,
    pub x_filter_should_cull_selection: bool,
}

#[derive(Debug, Clone)]
pub enum YIndicesNodeResult {
    NoChange,
    Indices(Arc<[u32]>),
    SkippedViewLenCap,
    SkippedIndicesScanAvoid,
}

#[derive(Debug, Clone)]
pub(super) struct CachedYIndicesNode {
    pub signature: u64,
    pub result: YIndicesNodeResult,
    pub x_filter_should_cull_selection: bool,
}

impl super::TransformGraph {
    pub fn y_indices_node_for_series(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        view: &ViewState,
        series_id: SeriesId,
        series_view_index: usize,
        max_view_len: usize,
        x_indices_applied: bool,
    ) -> YIndicesNodeOutput {
        let Some(series_model) = model.series.get(&series_id) else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let series_view = &view.series[series_view_index];

        let y_filter = series_view.y_filter;
        if y_filter.min.is_none() && y_filter.max.is_none() {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        }

        let base_selection = &series_view.selection;
        if matches!(base_selection, RowSelection::Indices(_)) && !x_indices_applied {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::SkippedIndicesScanAvoid,
                x_filter_should_cull_selection: false,
            };
        }

        let root = model.root_dataset_id(series_model.dataset);
        let Some(table) = datasets.dataset(root) else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let Some(dataset) = model.datasets.get(&series_model.dataset) else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let Some(x_col) = dataset.fields.get(&series_model.encode.x).copied() else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let Some(y0_col) = dataset.fields.get(&series_model.encode.y).copied() else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let y1_col = if series_model.kind == crate::spec::SeriesKind::Band {
            series_model
                .encode
                .y2
                .and_then(|f| dataset.fields.get(&f).copied())
        } else {
            None
        };

        let Some(x) = table.column_f64(x_col) else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let Some(y0) = table.column_f64(y0_col) else {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        };
        let y1 = y1_col.and_then(|c| table.column_f64(c));
        if y1_col.is_some() && y1.is_none() {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        }

        let len = match y1 {
            Some(y1) => x.len().min(y0.len()).min(y1.len()),
            None => x.len().min(y0.len()),
        };
        let view_len = base_selection.view_len(len);
        if view_len == 0 {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::NoChange,
                x_filter_should_cull_selection: false,
            };
        }
        if view_len > max_view_len {
            return YIndicesNodeOutput {
                result: YIndicesNodeResult::SkippedViewLenCap,
                x_filter_should_cull_selection: false,
            };
        }

        let x_filter_mode = series_view.x_filter_mode;
        let x_filter = series_view.x_policy.filter;
        let x_filter_active = x_filter.min.is_some() || x_filter.max.is_some();
        let x_filter_should_cull_selection = matches!(
            x_filter_mode,
            crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
        );

        let signature = y_indices_signature(
            model.revs.spec,
            view.revision,
            table.revision,
            series_id,
            series_model.kind,
            x_col,
            y0_col,
            y1_col,
            base_selection,
            y_filter,
            x_filter_mode,
            x_filter,
            max_view_len,
        );
        if let Some(cached) = self.y_indices_cache.get(&series_id)
            && cached.signature == signature
        {
            return YIndicesNodeOutput {
                result: cached.result.clone(),
                x_filter_should_cull_selection: cached.x_filter_should_cull_selection,
            };
        }

        let mut indices: Vec<u32> = Vec::new();
        indices.reserve(view_len.min(4096));

        let mut kept = 0usize;
        for view_index in 0..view_len {
            let Some(raw_index) = base_selection.get_raw_index(len, view_index) else {
                continue;
            };

            let xi = x.get(raw_index).copied().unwrap_or(f64::NAN);
            if !xi.is_finite() {
                continue;
            }
            if x_filter_should_cull_selection && x_filter_active && !x_filter.contains(xi) {
                continue;
            }

            let y_ok = if let Some(y1) = y1 {
                let y0i = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                let y1i = y1.get(raw_index).copied().unwrap_or(f64::NAN);
                y0i.is_finite() && y1i.is_finite() && y_filter.intersects_interval(y0i, y1i)
            } else {
                let yi = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                yi.is_finite() && y_filter.contains(yi)
            };
            if !y_ok {
                continue;
            }

            indices.push(raw_index.min(u32::MAX as usize) as u32);
            kept += 1;
        }

        let result = if kept == view_len {
            YIndicesNodeResult::NoChange
        } else {
            YIndicesNodeResult::Indices(indices.into())
        };

        self.y_indices_cache.insert(
            series_id,
            CachedYIndicesNode {
                signature,
                result: result.clone(),
                x_filter_should_cull_selection,
            },
        );

        YIndicesNodeOutput {
            result,
            x_filter_should_cull_selection,
        }
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

fn hash_selection(mut h: u64, sel: &RowSelection) -> u64 {
    match sel {
        RowSelection::All => fnv1a_step(h, 1),
        RowSelection::Range(r) => {
            h = fnv1a_step(h, 2);
            h = fnv1a_step(h, r.start as u64);
            fnv1a_step(h, r.end as u64)
        }
        RowSelection::Indices(indices) => {
            h = fnv1a_step(h, 3);
            h = fnv1a_step(h, indices.len() as u64);
            let ptr = indices.as_ref().as_ptr() as usize as u64;
            fnv1a_step(h, ptr)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn y_indices_signature(
    model_rev: Revision,
    view_rev: Revision,
    data_rev: Revision,
    series_id: SeriesId,
    kind: crate::spec::SeriesKind,
    x_col: usize,
    y0_col: usize,
    y1_col: Option<usize>,
    selection: &RowSelection,
    y_filter: AxisFilter1D,
    x_filter_mode: crate::spec::FilterMode,
    x_filter: AxisFilter1D,
    max_view_len: usize,
) -> u64 {
    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, model_rev.0 as u64);
    h = fnv1a_step(h, view_rev.0 as u64);
    h = fnv1a_step(h, data_rev.0 as u64);
    h = fnv1a_step(h, series_id.0);
    h = fnv1a_step(h, kind as u64);
    h = fnv1a_step(h, x_col as u64);
    h = fnv1a_step(h, y0_col as u64);
    h = fnv1a_step(h, y1_col.map(|c| c as u64).unwrap_or(u64::MAX));
    h = fnv1a_step(h, max_view_len as u64);
    h = hash_selection(h, selection);
    h = hash_filter(h, y_filter);
    h = fnv1a_step(h, x_filter_mode as u64);
    hash_filter(h, x_filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, DataTable};
    use crate::engine::ChartState;
    use crate::engine::model::ChartModel;
    use crate::ids::{AxisId, DatasetId, FieldId, GridId};
    use crate::spec::{
        AxisKind, AxisSpec, ChartSpec, DataZoomXSpec, DataZoomYSpec, DatasetSpec, FieldSpec,
        FilterMode, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
    };
    use crate::transform::{RowRange, RowSelection};
    use crate::view::ViewState;

    #[test]
    fn y_indices_node_cache_hits() {
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

        let mut state = ChartState::default();
        state.data_zoom_x.insert(
            x_axis,
            crate::engine::DataZoomXState {
                window: Some(crate::engine::window::DataWindowX {
                    min: 0.0,
                    max: 100.0,
                }),
                filter_mode: FilterMode::Filter,
            },
        );
        state.data_window_y.insert(
            y_axis,
            crate::engine::window::DataWindowY {
                min: 0.0,
                max: 100.0,
            },
        );

        let mut view = ViewState::default();
        assert!(view.sync_inputs(&model, &datasets, &state));
        view.rebuild(&model, &datasets, &state);

        // Force a stable base selection for signature.
        view.series[0].selection = RowSelection::Range(RowRange { start: 0, end: 101 });

        let mut graph = super::super::TransformGraph::default();

        let out =
            graph.y_indices_node_for_series(&model, &datasets, &view, series_id, 0, 200_000, false);
        assert!(matches!(out.result, YIndicesNodeResult::NoChange));

        // Force a cache hit by overriding the cached result under the same signature.
        let cached = graph
            .y_indices_cache
            .get_mut(&series_id)
            .expect("expected cache entry");
        cached.result = YIndicesNodeResult::Indices(Arc::from([42u32, 99u32]));

        let out =
            graph.y_indices_node_for_series(&model, &datasets, &view, series_id, 0, 200_000, false);
        match out.result {
            YIndicesNodeResult::Indices(indices) => {
                assert_eq!(indices.as_ref(), &[42u32, 99u32]);
            }
            _ => panic!("expected indices from cache"),
        }
    }
}
