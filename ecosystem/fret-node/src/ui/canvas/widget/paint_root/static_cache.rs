use crate::ui::canvas::widget::*;

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
