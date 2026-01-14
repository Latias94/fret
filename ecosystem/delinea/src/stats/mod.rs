use crate::ids::Revision;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EngineStats {
    pub revision: Revision,
    pub stage_data_runs: u64,
    pub stage_layout_runs: u64,
    pub stage_visual_runs: u64,
    pub stage_marks_runs: u64,
    pub filter_plan_runs: u64,
    pub filter_plan_grids: u64,
    pub filter_plan_steps_run: u64,
    pub filter_xy_weakfilter_applied_series: u64,
    pub filter_xy_weakfilter_pending_series: u64,
    pub filter_xy_weakfilter_skipped_view_len_cap_series: u64,
    pub filter_x_indices_applied_series: u64,
    pub filter_y_indices_applied_series: u64,
    pub filter_y_indices_skipped_view_len_cap_series: u64,
    pub filter_y_indices_skipped_indices_scan_avoid_series: u64,
    pub marks_emitted: u64,
    pub points_emitted: u64,
}
