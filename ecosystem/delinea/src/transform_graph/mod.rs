//! Incremental transform graph scaffolding.
//!
//! This is a v1 stepping stone towards an ECharts-class processor pipeline:
//! - cached node outputs keyed by dataset/model/state signatures,
//! - derived columns and domain transforms as first-class nodes,
//! - and a single source of truth for domain/selection outputs.
//!
//! For now, we keep the surface intentionally small and migrate behavior incrementally.

use crate::data::DatasetStore;
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window::DataWindow;
use crate::ids::{AxisId, Revision};
use crate::spec::AxisRange;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct TransformGraph {
    x_extent_cache: BTreeMap<AxisId, CachedExtent>,
}

#[derive(Debug, Default, Clone)]
struct CachedExtent {
    signature: u64,
    extent: Option<(f64, f64)>,
}

impl TransformGraph {
    pub fn clear(&mut self) {
        self.x_extent_cache.clear();
    }

    /// Returns a finite `(min, max)` data extent for the X axis based on visible series and the
    /// effective dataset row ranges.
    ///
    /// The result is cached using a metadata signature (dataset revisions + row ranges + bindings),
    /// so changing any of those inputs invalidates the cached extent.
    pub fn x_data_extent(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
        axis: AxisId,
    ) -> Option<(f64, f64)> {
        let signature = x_extent_signature(model, datasets, state, axis);
        if let Some(cached) = self.x_extent_cache.get(&axis)
            && cached.signature == signature
        {
            return cached.extent;
        }

        let extent = scan_x_extent(model, datasets, state, axis);
        self.x_extent_cache
            .insert(axis, CachedExtent { signature, extent });
        extent
    }

    /// Maps an ECharts-style percent range (0..=100) into a value-space window, clamping to a valid
    /// non-degenerate window and applying axis range constraints (lock min/max).
    pub fn percent_range_to_value_window(
        extent: (f64, f64),
        axis_range: AxisRange,
        start: f64,
        end: f64,
    ) -> Option<DataWindow> {
        let (mut dmin, mut dmax) = extent;
        if dmin > dmax {
            core::mem::swap(&mut dmin, &mut dmax);
        }

        match axis_range {
            AxisRange::Fixed { min, max } => {
                dmin = min;
                dmax = max;
            }
            AxisRange::Auto | AxisRange::LockMin { .. } | AxisRange::LockMax { .. } => {
                if let Some(min) = axis_range.locked_min() {
                    dmin = min;
                }
                if let Some(max) = axis_range.locked_max() {
                    dmax = max;
                }
            }
        }
        if !dmin.is_finite() || !dmax.is_finite() {
            return None;
        }

        let span = dmax - dmin;
        if !span.is_finite() || span <= 0.0 {
            return None;
        }

        if !start.is_finite() || !end.is_finite() {
            return None;
        }

        let mut a = (start / 100.0).clamp(0.0, 1.0);
        let mut b = (end / 100.0).clamp(0.0, 1.0);
        if a > b {
            core::mem::swap(&mut a, &mut b);
        }

        let mut window = DataWindow {
            min: dmin + span * a,
            max: dmin + span * b,
        };
        window.clamp_non_degenerate();
        Some(window.apply_constraints(axis_range.locked_min(), axis_range.locked_max()))
    }
}

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x00000100000001B3;

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV1A_PRIME)
}

fn rev_u64(rev: Revision) -> u64 {
    rev.0 as u64
}

fn x_extent_signature(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    axis: AxisId,
) -> u64 {
    let mut h = FNV1A_OFFSET;
    h = fnv1a_step(h, axis.0);

    for series_id in &model.series_order {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        if !series.visible || series.x_axis != axis {
            continue;
        }

        h = fnv1a_step(h, series_id.0);
        h = fnv1a_step(h, series.dataset.0);

        let table_rev = datasets
            .dataset(series.dataset)
            .map(|t| rev_u64(t.revision))
            .unwrap_or(0);
        h = fnv1a_step(h, table_rev);

        let x_col = model
            .datasets
            .get(&series.dataset)
            .and_then(|ds| ds.fields.get(&series.encode.x).copied())
            .unwrap_or(usize::MAX);
        h = fnv1a_step(h, x_col as u64);

        let range = state.dataset_row_ranges.get(&series.dataset).copied();
        match range {
            Some(r) => {
                h = fnv1a_step(h, r.start as u64);
                h = fnv1a_step(h, r.end as u64);
            }
            None => {
                h = fnv1a_step(h, 0);
                h = fnv1a_step(h, u64::MAX);
            }
        }
    }

    h
}

fn scan_x_extent(
    model: &ChartModel,
    datasets: &DatasetStore,
    state: &ChartState,
    axis: AxisId,
) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found = false;

    for series_id in &model.series_order {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        if !series.visible || series.x_axis != axis {
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
        let Some(x_values) = table.column_f64(x_col) else {
            continue;
        };

        let mut range = state
            .dataset_row_ranges
            .get(&series.dataset)
            .copied()
            .unwrap_or(crate::transform::RowRange {
                start: 0,
                end: table.row_count,
            });
        range.clamp_to_len(table.row_count);

        for i in range.start..range.end {
            let v = x_values.get(i).copied().unwrap_or(f64::NAN);
            if !v.is_finite() {
                continue;
            }
            min = min.min(v);
            max = max.max(v);
            found = true;
        }
    }

    if found && min.is_finite() && max.is_finite() {
        Some((min, max))
    } else {
        None
    }
}
