mod geometry;
mod paint;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn static_scene_style_key(&self, scale_factor: f32) -> u64 {
        let mut builder = TileCacheKeyBuilder::new("fret-node.canvas.static_scene_style.v1");
        let paint_overrides_rev = self
            .paint_overrides
            .as_ref()
            .map(|overrides| overrides.revision())
            .unwrap_or(0);

        builder.add_u64(paint_overrides_rev);
        paint::append_static_scene_paint_style_key(&mut builder, &self.style);
        geometry::append_static_scene_geometry_style_key(&mut builder, &self.style, scale_factor);
        builder.finish()
    }
}
