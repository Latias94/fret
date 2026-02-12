use super::TransformGraph;
use crate::data::{Column, DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::ids::{AxisId, DatasetId, FieldId, GridId, SeriesId};
use crate::spec::{
    AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
    SeriesSpec,
};
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;
use std::collections::BTreeMap;

#[test]
fn y_percent_extents_cache_hits_and_invalidates() {
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

    let mut state = ChartState::default();
    state.axis_percent_windows.insert(y_axis, (0.0, 100.0));

    let mut view = ViewState::default();
    assert!(view.sync_inputs(&model, &datasets, &state));
    view.rebuild(&model, &datasets, &state);
    let view_series_index: BTreeMap<SeriesId, usize> = view
        .series
        .iter()
        .enumerate()
        .map(|(i, v)| (v.series, i))
        .collect();

    let mut graph = TransformGraph::default();
    let extents = graph.y_percent_extents_scoped_by_x_for_grid(
        &model,
        &datasets,
        &state,
        &view,
        &view_series_index,
        grid_id,
        &[series_id],
    );
    assert_eq!(extents.get(&y_axis).copied(), Some((0.0, 100.0)));

    // Force a cache hit by overriding cached extents under the same signature.
    let cached = graph
        .y_percent_extents_cache
        .get_mut(&grid_id)
        .expect("expected cache entry");
    cached.extents.insert(y_axis, (123.0, 456.0));

    let extents = graph.y_percent_extents_scoped_by_x_for_grid(
        &model,
        &datasets,
        &state,
        &view,
        &view_series_index,
        grid_id,
        &[series_id],
    );
    assert_eq!(extents.get(&y_axis).copied(), Some((123.0, 456.0)));

    // Changing the selection must invalidate the cached signature.
    let view_series_index = view_series_index;
    let mut view = view;
    view.series[0].selection = RowSelection::Range(RowRange {
        start: 50,
        end: 101,
    });
    let extents = graph.y_percent_extents_scoped_by_x_for_grid(
        &model,
        &datasets,
        &state,
        &view,
        &view_series_index,
        grid_id,
        &[series_id],
    );
    assert_eq!(extents.get(&y_axis).copied(), Some((50.0, 100.0)));
}
