use crate::stats::BundleStatsSnapshotRow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TopCodeEditorRowSceneFields {
    pub(super) row_text_get_calls: u64,
    pub(super) row_text_hits: u64,
    pub(super) row_text_misses: u64,
    pub(super) row_text_resets: u64,
    pub(super) row_text_hit_rate_pct: u64,
    pub(super) row_text_us: u64,
    pub(super) rows_painted: u64,
    pub(super) rows_scene_replayed: u64,
    pub(super) rows_scene_stored: u64,
    pub(super) row_scene_ops_stored: u64,
    pub(super) row_scene_replay_hit_rate_pct: u64,
    pub(super) row_scene_cache_get_calls: u64,
    pub(super) row_scene_cache_hits: u64,
    pub(super) row_scene_cache_misses: u64,
    pub(super) row_scene_cache_resets: u64,
    pub(super) row_scene_cache_hit_rate_pct: u64,
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
        let perf = top.and_then(|r| r.code_editor_paint_perf.as_ref());
        let cache = top.and_then(|r| r.code_editor_cache_stats.as_ref());
        if perf.is_none() && cache.is_none() {
            return Self::default();
        }

        Self {
            row_text_get_calls: cache.map(|c| c.row_text_get_calls).unwrap_or(0),
            row_text_hits: cache.map(|c| c.row_text_hits).unwrap_or(0),
            row_text_misses: cache.map(|c| c.row_text_misses).unwrap_or(0),
            row_text_resets: cache.map(|c| c.row_text_resets).unwrap_or(0),
            row_text_hit_rate_pct: cache_hit_rate_pct(
                cache.map(|c| c.row_text_hits).unwrap_or(0),
                cache.map(|c| c.row_text_get_calls).unwrap_or(0),
            ),
            row_text_us: perf.map(|p| p.us_row_text).unwrap_or(0),
            rows_painted: perf.map(|p| p.rows_painted).unwrap_or(0),
            rows_scene_replayed: perf.map(|p| p.rows_scene_replayed).unwrap_or(0),
            rows_scene_stored: perf.map(|p| p.rows_scene_stored).unwrap_or(0),
            row_scene_ops_stored: perf.map(|p| p.row_scene_ops_stored).unwrap_or(0),
            row_scene_replay_hit_rate_pct: replay_hit_rate_pct(
                perf.map(|p| p.rows_scene_replayed).unwrap_or(0),
                perf.map(|p| p.rows_painted).unwrap_or(0),
            ),
            row_scene_cache_get_calls: cache.map(|c| c.row_scene_get_calls).unwrap_or(0),
            row_scene_cache_hits: cache.map(|c| c.row_scene_hits).unwrap_or(0),
            row_scene_cache_misses: cache.map(|c| c.row_scene_misses).unwrap_or(0),
            row_scene_cache_resets: cache.map(|c| c.row_scene_resets).unwrap_or(0),
            row_scene_cache_hit_rate_pct: cache_hit_rate_pct(
                cache.map(|c| c.row_scene_hits).unwrap_or(0),
                cache.map(|c| c.row_scene_get_calls).unwrap_or(0),
            ),
            row_scene_prepaint_plan_us: perf.map(|p| p.us_row_scene_prepaint_plan).unwrap_or(0),
            row_scene_prepaint_probe_us: perf.map(|p| p.us_row_scene_prepaint_probe).unwrap_or(0),
            row_scene_prepaint_key_compare_us: perf
                .map(|p| p.us_row_scene_prepaint_key_compare)
                .unwrap_or(0),
            windowed_surface_paint_callback_us: perf
                .map(|p| p.us_windowed_surface_paint_callback)
                .unwrap_or(0),
            windowed_surface_non_row_us: perf.map(|p| p.us_windowed_surface_non_row).unwrap_or(0),
            windowed_surface_row_callback_gap_us: perf
                .map(|p| p.us_windowed_surface_row_callback_gap)
                .unwrap_or(0),
            torture_autoscroll_us: perf.map(|p| p.us_torture_autoscroll).unwrap_or(0),
            torture_overlay_us: perf.map(|p| p.us_torture_overlay).unwrap_or(0),
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

fn cache_hit_rate_pct(hits: u64, get_calls: u64) -> u64 {
    if get_calls == 0 {
        0
    } else {
        hits.saturating_mul(100) / get_calls
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
                row_text_get_calls: 0,
                row_text_hits: 0,
                row_text_misses: 0,
                row_text_resets: 0,
                row_text_hit_rate_pct: 0,
                row_text_us: 0,
                rows_painted: 12,
                rows_scene_replayed: 9,
                rows_scene_stored: 3,
                row_scene_ops_stored: 42,
                row_scene_replay_hit_rate_pct: 75,
                row_scene_cache_get_calls: 0,
                row_scene_cache_hits: 0,
                row_scene_cache_misses: 0,
                row_scene_cache_resets: 0,
                row_scene_cache_hit_rate_pct: 0,
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

    #[test]
    fn top_code_editor_row_scene_fields_export_cache_stats() {
        let row = BundleStatsSnapshotRow {
            code_editor_paint_perf: Some(BundleStatsCodeEditorPaintPerf {
                us_row_text: 17,
                ..Default::default()
            }),
            code_editor_cache_stats: Some(crate::stats::BundleStatsCodeEditorCacheStats {
                row_text_get_calls: 20,
                row_text_hits: 18,
                row_text_misses: 2,
                row_text_resets: 1,
                row_scene_get_calls: 10,
                row_scene_hits: 7,
                row_scene_misses: 3,
                row_scene_resets: 0,
                ..Default::default()
            }),
            ..Default::default()
        };

        let fields = TopCodeEditorRowSceneFields::from_top(Some(&row));
        assert_eq!(fields.row_text_get_calls, 20);
        assert_eq!(fields.row_text_hits, 18);
        assert_eq!(fields.row_text_misses, 2);
        assert_eq!(fields.row_text_resets, 1);
        assert_eq!(fields.row_text_hit_rate_pct, 90);
        assert_eq!(fields.row_text_us, 17);
        assert_eq!(fields.row_scene_cache_get_calls, 10);
        assert_eq!(fields.row_scene_cache_hits, 7);
        assert_eq!(fields.row_scene_cache_misses, 3);
        assert_eq!(fields.row_scene_cache_resets, 0);
        assert_eq!(fields.row_scene_cache_hit_rate_pct, 70);
    }
}
