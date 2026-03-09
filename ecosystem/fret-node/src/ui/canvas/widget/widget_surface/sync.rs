use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn reset_geometry_cache_keys(&mut self) {
        self.geometry.geom_key = None;
        self.geometry.index_key = None;
        self.geometry.drag_preview = None;
    }

    fn clear_scene_caches_and_build_states(&mut self) {
        self.grid_scene_cache.clear();
        self.groups_scene_cache.clear();
        self.nodes_scene_cache.clear();
        self.edges_scene_cache.clear();
        self.edge_labels_scene_cache.clear();
        self.edges_build_states.clear();
        self.edge_labels_build_states.clear();
        self.edge_labels_build_state = None;
    }

    pub(in super::super) fn sync_style_from_color_mode(
        &mut self,
        theme: fret_ui::ThemeSnapshot,
        services: Option<&mut dyn fret_core::UiServices>,
    ) {
        let Some(mode) = self.color_mode else {
            return;
        };

        let needs_update = match mode {
            NodeGraphColorMode::System => {
                let rev = theme.revision;
                self.color_mode_last != Some(mode) || self.color_mode_theme_rev != Some(rev)
            }
            NodeGraphColorMode::Light | NodeGraphColorMode::Dark => {
                self.color_mode_last != Some(mode)
            }
        };

        if !needs_update {
            return;
        }

        self.color_mode_last = Some(mode);
        self.color_mode_theme_rev = match mode {
            NodeGraphColorMode::System => Some(theme.revision),
            NodeGraphColorMode::Light | NodeGraphColorMode::Dark => None,
        };

        let prev_geometry_fp = self.style.geometry_fingerprint();
        let mut next_style = NodeGraphStyle::from_snapshot_with_color_mode(theme, mode);
        if let Some(background) = self.background_override {
            next_style = next_style.with_background_style(background);
        }

        let geometry_changed = prev_geometry_fp != next_style.geometry_fingerprint();
        self.style = next_style;

        if geometry_changed {
            self.reset_geometry_cache_keys();
        }

        if let Some(services) = services {
            self.paint_cache.clear(services);
        }

        self.clear_scene_caches_and_build_states();
    }

    pub(in super::super) fn sync_skin(
        &mut self,
        _services: Option<&mut dyn fret_core::UiServices>,
    ) {
        let Some(skin) = self.skin.as_ref() else {
            self.skin_last_rev = None;
            return;
        };

        let rev = skin.revision();
        if self.skin_last_rev == Some(rev) {
            return;
        }
        self.skin_last_rev = Some(rev);
        self.clear_scene_caches_and_build_states();
    }

    pub(in super::super) fn sync_paint_overrides(
        &mut self,
        _services: Option<&mut dyn fret_core::UiServices>,
    ) {
        let Some(overrides) = self.paint_overrides.as_ref() else {
            self.paint_overrides_last_rev = None;
            return;
        };

        let rev = overrides.revision();
        if self.paint_overrides_last_rev == Some(rev) {
            return;
        }
        self.paint_overrides_last_rev = Some(rev);
        self.clear_scene_caches_and_build_states();
    }
}
