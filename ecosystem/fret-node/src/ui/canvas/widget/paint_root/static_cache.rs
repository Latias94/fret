use crate::ui::canvas::widget::*;

pub(super) fn static_layer_cache_key(
    scope: &'static str,
    base_key: DerivedBaseKey,
    style_key: u64,
    tile_size_canvas: f32,
    cache_rect: Rect,
) -> u64 {
    let mut b = TileCacheKeyBuilder::new(scope);
    b.add_u64(base_key.graph_rev);
    b.add_u32(base_key.zoom_bits);
    b.add_u32(base_key.node_origin_x_bits);
    b.add_u32(base_key.node_origin_y_bits);
    b.add_u64(base_key.draw_order.lo);
    b.add_u64(base_key.draw_order.hi);
    b.add_u64(base_key.presenter_rev);
    b.add_u64(base_key.edge_types_rev);
    b.add_u64(base_key.overrides_rev);
    b.add_u64(style_key);
    b.add_f32_bits(tile_size_canvas);
    b.add_u32(cache_rect.origin.x.0.to_bits());
    b.add_u32(cache_rect.origin.y.0.to_bits());
    b.finish()
}

pub(super) fn try_replay_static_scene_cache<FTouch>(
    cache: &mut SceneOpTileCache<u64>,
    scene: &mut fret_core::Scene,
    paint_cache: &mut CanvasPaintCache,
    key: u64,
    replay_delta: Point,
    touch: &FTouch,
) -> bool
where
    FTouch: Fn(&mut CanvasPaintCache, &[SceneOp]),
{
    cache.try_replay_with(key, scene, replay_delta, |ops| touch(paint_cache, ops))
}

pub(super) fn store_and_replay_static_scene_cache<H: UiHost, FTouch>(
    cache: &mut SceneOpTileCache<u64>,
    cx: &mut PaintCx<'_, H>,
    paint_cache: &mut CanvasPaintCache,
    key: u64,
    replay_delta: Point,
    ops: Vec<SceneOp>,
    touch: FTouch,
) where
    FTouch: Fn(&mut CanvasPaintCache, &[SceneOp]),
{
    cache.store_ops(key, ops);
    let _ = try_replay_static_scene_cache(cache, cx.scene, paint_cache, key, replay_delta, &touch);
}
