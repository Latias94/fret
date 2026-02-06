use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn prune_paint_caches(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        snapshot: &ViewSnapshot,
    ) {
        let prune = snapshot.interaction.paint_cache_prune;

        self.groups_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.nodes_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.edges_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.edge_labels_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );

        if prune.max_entries > 0 && prune.max_age_frames > 0 {
            self.paint_cache
                .prune(services, prune.max_age_frames, prune.max_entries);
            let tile_budget = (prune.max_entries / 10).clamp(64, 2048);
            self.grid_scene_cache
                .prune(prune.max_age_frames, tile_budget);
        }
    }
}
