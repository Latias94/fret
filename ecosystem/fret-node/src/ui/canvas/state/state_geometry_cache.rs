use super::*;

impl GeometryCache {
    pub(crate) fn clear_drag_preview(&mut self) {
        self.drag_preview = None;
    }

    pub(crate) fn ensure_geom_key(&mut self, geom_key: GeometryCacheKey) -> bool {
        if self.geom_key == Some(geom_key) {
            return false;
        }
        self.geom_key = Some(geom_key);
        self.index_key = None;
        self.clear_drag_preview();
        self.counters.geom_rebuilds = self.counters.geom_rebuilds.saturating_add(1);
        true
    }

    pub(crate) fn ensure_index_key(&mut self, index_key: SpatialIndexCacheKey) -> bool {
        if self.index_key == Some(index_key) {
            return false;
        }
        self.index_key = Some(index_key);
        self.clear_drag_preview();
        self.counters.index_rebuilds = self.counters.index_rebuilds.saturating_add(1);
        true
    }

    pub(crate) fn drag_preview_rebuild_needed(
        &self,
        kind: DragPreviewKind,
        base_index_key: SpatialIndexCacheKey,
    ) -> bool {
        self.drag_preview
            .as_ref()
            .is_none_or(|cache| cache.kind != kind || cache.base_index_key != base_index_key)
    }

    pub(crate) fn set_drag_preview(&mut self, cache: DragPreviewCache) {
        self.drag_preview = Some(cache);
    }

    pub(crate) fn drag_preview_outputs_for_rev(
        &mut self,
        preview_rev: u64,
        update: impl FnOnce(
            &mut DragPreviewCacheMetaMut<'_>,
            &mut super::super::geometry::CanvasGeometry,
            &mut super::super::spatial::CanvasSpatialDerived,
        ),
    ) -> Option<(
        Arc<super::super::geometry::CanvasGeometry>,
        Arc<super::super::spatial::CanvasSpatialDerived>,
    )> {
        let cache = self.drag_preview.as_mut()?;
        if cache.preview_rev != preview_rev {
            let node_positions = &mut cache.node_positions;
            let node_rects = &mut cache.node_rects;
            let node_ports = &cache.node_ports;
            let geom_mut = Arc::make_mut(&mut cache.geom);
            let index_mut = Arc::make_mut(&mut cache.index);
            let mut meta = DragPreviewCacheMetaMut {
                node_positions,
                node_rects,
                node_ports,
            };
            update(&mut meta, geom_mut, index_mut);
            cache.preview_rev = preview_rev;
        }
        Some((cache.geom.clone(), cache.index.clone()))
    }
}
