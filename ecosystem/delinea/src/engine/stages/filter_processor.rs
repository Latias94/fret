use super::DataViewStage;
use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::transform::RowSelection;
use crate::view::ViewState;

#[derive(Debug, Default, Clone)]
pub struct FilterProcessorStage;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FilterProcessorResult {
    pub xy_weak_filter_pending: bool,
}

impl FilterProcessorStage {
    pub fn request_data_views(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &ViewState,
        data_view: &mut DataViewStage,
    ) {
        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };

            let Some(zoom_state) = state.data_zoom_x.get(&series.x_axis) else {
                continue;
            };
            if zoom_state.window.is_none() {
                continue;
            }
            let filter_mode = zoom_state.filter_mode;

            // Multi-dimensional `weakFilter` (v1 subset): request an indices view when both X and Y
            // dataZoom are `WeakFilter` and the Y window is active. This is correctness-critical
            // for the subset because X-only range slicing cannot represent weakFilter semantics.
            if filter_mode == crate::spec::FilterMode::WeakFilter {
                let y_filter_mode = model
                    .data_zoom_y_by_axis
                    .get(&series.y_axis)
                    .and_then(|id| model.data_zoom_y.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or(crate::spec::FilterMode::None);

                const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;
                if y_filter_mode == crate::spec::FilterMode::WeakFilter
                    && state.data_window_y.get(&series.y_axis).is_some()
                    && series.stack.is_none()
                    && matches!(
                        series.kind,
                        crate::spec::SeriesKind::Scatter
                            | crate::spec::SeriesKind::Line
                            | crate::spec::SeriesKind::Area
                            | crate::spec::SeriesKind::Band
                    )
                {
                    let Some(series_view) = view.series_view(*series_id) else {
                        continue;
                    };
                    let Some(dataset_view) = view.dataset_view(series.dataset) else {
                        continue;
                    };

                    let base_range = dataset_view.row_range;
                    let base_len = base_range.end.saturating_sub(base_range.start);
                    if base_len <= MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN
                        && matches!(
                            series_view.selection,
                            RowSelection::All | RowSelection::Range(_)
                        )
                    {
                        let x_axis_range = model
                            .axes
                            .get(&series.x_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();
                        let y_axis_range = model
                            .axes
                            .get(&series.y_axis)
                            .map(|a| a.range)
                            .unwrap_or_default();

                        let x_filter = crate::engine::window_policy::axis_filter_1d(
                            x_axis_range,
                            zoom_state.window,
                            crate::spec::FilterMode::WeakFilter,
                        );
                        let y_filter = crate::engine::window_policy::axis_filter_1d(
                            y_axis_range,
                            state.data_window_y.get(&series.y_axis).copied(),
                            crate::spec::FilterMode::WeakFilter,
                        );

                        match series.kind {
                            crate::spec::SeriesKind::Band => {
                                let _ = data_view.request_xy_weak_filter_band_for_series(
                                    model, datasets, view, *series_id, base_range, x_filter,
                                    y_filter,
                                );
                            }
                            _ => {
                                let _ = data_view.request_xy_weak_filter_for_series(
                                    model, datasets, view, *series_id, base_range, x_filter,
                                    y_filter,
                                );
                            }
                        }
                    }
                }
            }

            // ADR 1150:
            // - `Filter` / `WeakFilter` may use indices views as an optimization carrier.
            // - `Empty` must preserve a stable row/index space (avoid indices-backed selections).
            if !matches!(
                filter_mode,
                crate::spec::FilterMode::Filter | crate::spec::FilterMode::WeakFilter
            ) {
                continue;
            }

            data_view.request_x_filter_for_series(model, datasets, view, *series_id);
        }
    }

    pub fn apply(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        view: &mut ViewState,
        data_views: &DataViewStage,
    ) -> FilterProcessorResult {
        let mut xy_weak_filter_pending = false;
        let mut xy_weak_filter_applied = false;

        const MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN: usize = 200_000;

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            if !series.visible || series.stack.is_some() {
                continue;
            }
            if !matches!(
                series.kind,
                crate::spec::SeriesKind::Scatter
                    | crate::spec::SeriesKind::Line
                    | crate::spec::SeriesKind::Area
                    | crate::spec::SeriesKind::Band
            ) {
                continue;
            }

            let Some(zoom_state) = state.data_zoom_x.get(&series.x_axis) else {
                continue;
            };
            if zoom_state.filter_mode != crate::spec::FilterMode::WeakFilter
                || zoom_state.window.is_none()
            {
                continue;
            }

            let y_filter_mode = model
                .data_zoom_y_by_axis
                .get(&series.y_axis)
                .and_then(|id| model.data_zoom_y.get(id))
                .map(|z| z.filter_mode)
                .unwrap_or(crate::spec::FilterMode::None);
            if y_filter_mode != crate::spec::FilterMode::WeakFilter {
                continue;
            }

            let Some(y_window) = state.data_window_y.get(&series.y_axis).copied() else {
                continue;
            };

            let Some(series_view_index) = view.series.iter().position(|v| v.series == *series_id)
            else {
                continue;
            };
            if !matches!(
                view.series[series_view_index].selection,
                RowSelection::All | RowSelection::Range(_) | RowSelection::Indices(_)
            ) {
                continue;
            }

            let Some(table) = datasets.dataset(series.dataset) else {
                continue;
            };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };
            let Some(y0_col) = dataset.fields.get(&series.encode.y).copied() else {
                continue;
            };
            let y1_col = if series.kind == crate::spec::SeriesKind::Band {
                let Some(y1_field) = series.encode.y2 else {
                    continue;
                };
                let Some(y1_col) = dataset.fields.get(&y1_field).copied() else {
                    continue;
                };
                Some(y1_col)
            } else {
                None
            };

            let base_range = view
                .dataset_view(series.dataset)
                .map(|v| v.row_range)
                .unwrap_or(crate::transform::RowRange {
                    start: 0,
                    end: table.row_count,
                });
            let base_len = base_range.end.saturating_sub(base_range.start);
            if base_len > MAX_MULTI_DIM_WEAKFILTER_VIEW_LEN {
                continue;
            }

            let x_axis_range = model
                .axes
                .get(&series.x_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            let y_axis_range = model
                .axes
                .get(&series.y_axis)
                .map(|a| a.range)
                .unwrap_or_default();

            let x_filter = crate::engine::window_policy::axis_filter_1d(
                x_axis_range,
                zoom_state.window,
                crate::spec::FilterMode::WeakFilter,
            );
            let y_filter = crate::engine::window_policy::axis_filter_1d(
                y_axis_range,
                Some(y_window),
                crate::spec::FilterMode::WeakFilter,
            );
            let multi_active = (x_filter.min.is_some() || x_filter.max.is_some())
                && (y_filter.min.is_some() || y_filter.max.is_some());
            if !multi_active {
                continue;
            }

            let sel = match y1_col {
                Some(y1_col) => data_views.selection_for_xy_weak_filter_band(
                    series.dataset,
                    x_col,
                    y0_col,
                    y1_col,
                    base_range,
                    x_filter,
                    y_filter,
                    table.revision,
                ),
                None => data_views.selection_for_xy_weak_filter(
                    series.dataset,
                    x_col,
                    y0_col,
                    base_range,
                    x_filter,
                    y_filter,
                    table.revision,
                ),
            };

            if let Some(sel) = sel {
                let series_view = &mut view.series[series_view_index];
                if series_view.selection != sel {
                    series_view.selection = sel;
                    xy_weak_filter_applied = true;
                }
                if series_view.x_policy.filter != Default::default() {
                    series_view.x_policy.filter = Default::default();
                    xy_weak_filter_applied = true;
                }
            } else {
                xy_weak_filter_pending = true;
            }
        }

        if xy_weak_filter_applied {
            view.revision.bump();
            for series_view in &mut view.series {
                series_view.revision = view.revision;
            }
        }

        FilterProcessorResult {
            xy_weak_filter_pending,
        }
    }
}
