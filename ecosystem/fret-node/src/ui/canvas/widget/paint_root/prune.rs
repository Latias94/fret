use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn prune_static_scene_tile_caches(&mut self) {
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
    }

    fn prune_dynamic_paint_caches_if_needed(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        if max_entries == 0 || max_age_frames == 0 {
            return;
        }

        self.paint_cache
            .prune(services, max_age_frames, max_entries);
        let tile_budget = (max_entries / 10).clamp(64, 2048);
        self.grid_scene_cache.prune(max_age_frames, tile_budget);
    }

    pub(in super::super) fn prune_paint_caches(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        snapshot: &ViewSnapshot,
    ) {
        let prune = snapshot.interaction.paint_cache_prune;

        self.prune_static_scene_tile_caches();
        self.prune_dynamic_paint_caches_if_needed(
            services,
            prune.max_age_frames,
            prune.max_entries,
        );
    }
}
