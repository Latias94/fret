use crate::stats::BundleStatsSnapshotRow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TopCodeEditorRowSceneFields {
    pub(super) rows_painted: u64,
    pub(super) rows_scene_replayed: u64,
    pub(super) rows_scene_stored: u64,
    pub(super) row_scene_ops_stored: u64,
    pub(super) row_scene_replay_hit_rate_pct: u64,
    pub(super) row_scene_prepaint_plan_us: u64,
    pub(super) row_scene_prepaint_probe_us: u64,
    pub(super) row_scene_prepaint_key_compare_us: u64,
    pub(super) windowed_surface_paint_callback_us: u64,
    pub(super) windowed_surface_non_row_us: u64,
    pub(super) windowed_surface_row_callback_gap_us: u64,
    pub(super) torture_autoscroll_us: u64,
    pub(super) torture_overlay_us: u64,
}

impl TopCodeEditorRowSceneFields {
    pub(super) fn from_top(top: Option<&BundleStatsSnapshotRow>) -> Self {
        let Some(perf) = top.and_then(|r| r.code_editor_paint_perf.as_ref()) else {
            return Self::default();
        };

        Self {
            rows_painted: perf.rows_painted,
            rows_scene_replayed: perf.rows_scene_replayed,
            rows_scene_stored: perf.rows_scene_stored,
            row_scene_ops_stored: perf.row_scene_ops_stored,
            row_scene_replay_hit_rate_pct: replay_hit_rate_pct(
                perf.rows_scene_replayed,
                perf.rows_painted,
            ),
            row_scene_prepaint_plan_us: perf.us_row_scene_prepaint_plan,
            row_scene_prepaint_probe_us: perf.us_row_scene_prepaint_probe,
            row_scene_prepaint_key_compare_us: perf.us_row_scene_prepaint_key_compare,
            windowed_surface_paint_callback_us: perf.us_windowed_surface_paint_callback,
            windowed_surface_non_row_us: perf.us_windowed_surface_non_row,
            windowed_surface_row_callback_gap_us: perf.us_windowed_surface_row_callback_gap,
            torture_autoscroll_us: perf.us_torture_autoscroll,
            torture_overlay_us: perf.us_torture_overlay,
        }
    }
}

fn replay_hit_rate_pct(rows_scene_replayed: u64, rows_painted: u64) -> u64 {
    if rows_painted == 0 {
        0
    } else {
        rows_scene_replayed.saturating_mul(100) / rows_painted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::BundleStatsCodeEditorPaintPerf;

    #[test]
    fn top_code_editor_row_scene_fields_default_without_paint_perf() {
        assert_eq!(
            TopCodeEditorRowSceneFields::from_top(None),
            TopCodeEditorRowSceneFields::default()
        );
        assert_eq!(replay_hit_rate_pct(5, 0), 0);
    }

    #[test]
    fn top_code_editor_row_scene_fields_compute_replay_rate() {
        let row = BundleStatsSnapshotRow {
            code_editor_paint_perf: Some(BundleStatsCodeEditorPaintPerf {
                rows_painted: 12,
                rows_scene_replayed: 9,
                rows_scene_stored: 3,
                row_scene_ops_stored: 42,
                us_row_scene_prepaint_plan: 31,
                us_row_scene_prepaint_probe: 19,
                us_row_scene_prepaint_key_compare: 11,
                us_windowed_surface_paint_callback: 120,
                us_windowed_surface_non_row: 30,
                us_windowed_surface_row_callback_gap: 7,
                us_torture_autoscroll: 4,
                us_torture_overlay: 21,
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            TopCodeEditorRowSceneFields::from_top(Some(&row)),
            TopCodeEditorRowSceneFields {
                rows_painted: 12,
                rows_scene_replayed: 9,
                rows_scene_stored: 3,
                row_scene_ops_stored: 42,
                row_scene_replay_hit_rate_pct: 75,
                row_scene_prepaint_plan_us: 31,
                row_scene_prepaint_probe_us: 19,
                row_scene_prepaint_key_compare_us: 11,
                windowed_surface_paint_callback_us: 120,
                windowed_surface_non_row_us: 30,
                windowed_surface_row_callback_gap_us: 7,
                torture_autoscroll_us: 4,
                torture_overlay_us: 21,
            }
        );
    }
}
