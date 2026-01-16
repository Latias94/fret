use std::collections::HashMap;

use crate::cache::CacheStats;

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CanvasCacheSnapshot {
    pub last_frame_id: u64,
    pub path: Option<CacheKindSnapshot>,
    pub svg: Option<CacheKindSnapshot>,
    pub text: Option<CacheKindSnapshot>,
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
}
