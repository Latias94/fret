use std::collections::HashMap;

use crate::cache::{CacheStats, SceneOpTileCacheStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasCacheKey {
    pub window: u64,
    pub node: u64,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheKindSnapshot {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: CacheStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneOpTileCacheSnapshot {
    pub entries: usize,
    pub requested_tiles: usize,
    pub budget_limit: u32,
    pub budget_used: u32,
    pub skipped_tiles: u32,
    pub stats: SceneOpTileCacheStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkBudgetSnapshot {
    pub requested_units: u32,
    pub limit: u32,
    pub used: u32,
    pub skipped_units: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CanvasCacheSnapshot {
    pub last_frame_id: u64,
    pub path: Option<CacheKindSnapshot>,
    pub svg: Option<CacheKindSnapshot>,
    pub text: Option<CacheKindSnapshot>,
    pub scene_op_tiles: Option<SceneOpTileCacheSnapshot>,
    pub work_budget: Option<WorkBudgetSnapshot>,
}

#[derive(Debug, Default)]
pub struct CanvasCacheStatsRegistry {
    entries: HashMap<CanvasCacheKey, CanvasCacheSnapshot>,
}

impl CanvasCacheStatsRegistry {
    pub fn iter(&self) -> impl Iterator<Item = (CanvasCacheKey, &CanvasCacheSnapshot)> + '_ {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    pub fn record_path_cache(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        entries: usize,
        stats: CacheStats,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.path = Some(CacheKindSnapshot {
            entries,
            bytes_ready: 0,
            stats,
        });
    }

    pub fn record_svg_cache(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        entries: usize,
        bytes_ready: u64,
        stats: CacheStats,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.svg = Some(CacheKindSnapshot {
            entries,
            bytes_ready,
            stats,
        });
    }

    pub fn record_text_cache(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        entries: usize,
        stats: CacheStats,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.text = Some(CacheKindSnapshot {
            entries,
            bytes_ready: 0,
            stats,
        });
    }

    pub fn record_scene_op_tile_cache(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        entries: usize,
        stats: SceneOpTileCacheStats,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.scene_op_tiles = Some(SceneOpTileCacheSnapshot {
            entries,
            requested_tiles: 0,
            budget_limit: 0,
            budget_used: 0,
            skipped_tiles: 0,
            stats,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_scene_op_tile_cache_with_budget(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        entries: usize,
        requested_tiles: usize,
        budget_limit: u32,
        budget_used: u32,
        skipped_tiles: u32,
        stats: SceneOpTileCacheStats,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.scene_op_tiles = Some(SceneOpTileCacheSnapshot {
            entries,
            requested_tiles,
            budget_limit,
            budget_used,
            skipped_tiles,
            stats,
        });
    }

    pub fn record_work_budget(
        &mut self,
        key: CanvasCacheKey,
        frame_id: u64,
        requested_units: u32,
        limit: u32,
        used: u32,
        skipped_units: u32,
    ) {
        let snap = self.entries.entry(key).or_default();
        snap.last_frame_id = frame_id;
        snap.work_budget = Some(WorkBudgetSnapshot {
            requested_units,
            limit,
            used,
            skipped_units,
        });
    }
}
