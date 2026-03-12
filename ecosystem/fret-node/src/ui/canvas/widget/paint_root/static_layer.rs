use crate::ui::canvas::widget::*;

#[derive(Clone, Copy)]
pub(super) enum StaticSceneLayerTarget {
    Groups,
    Nodes,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn try_replay_static_scene_layer<H: UiHost, FTouch>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        target: StaticSceneLayerTarget,
        key: u64,
        replay_delta: Point,
        touch: &FTouch,
    ) -> bool
    where
        FTouch: Fn(&mut CanvasPaintCache, &[SceneOp]),
    {
        match target {
            StaticSceneLayerTarget::Groups => super::static_cache::try_replay_static_scene_cache(
                &mut self.groups_scene_cache,
                cx.scene,
                &mut self.paint_cache,
                key,
                replay_delta,
                touch,
            ),
            StaticSceneLayerTarget::Nodes => super::static_cache::try_replay_static_scene_cache(
                &mut self.nodes_scene_cache,
                cx.scene,
                &mut self.paint_cache,
                key,
                replay_delta,
                touch,
            ),
        }
    }

    fn store_and_replay_static_scene_layer<H: UiHost, FTouch>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        target: StaticSceneLayerTarget,
        key: u64,
        replay_delta: Point,
        ops: Vec<SceneOp>,
        touch: FTouch,
    ) where
        FTouch: Fn(&mut CanvasPaintCache, &[SceneOp]),
    {
        match target {
            StaticSceneLayerTarget::Groups => {
                super::static_cache::store_and_replay_static_scene_cache(
                    &mut self.groups_scene_cache,
                    cx,
                    &mut self.paint_cache,
                    key,
                    replay_delta,
                    ops,
                    touch,
                );
            }
            StaticSceneLayerTarget::Nodes => {
                super::static_cache::store_and_replay_static_scene_cache(
                    &mut self.nodes_scene_cache,
                    cx,
                    &mut self.paint_cache,
                    key,
                    replay_delta,
                    ops,
                    touch,
                );
            }
        }
    }

    pub(super) fn paint_root_static_layer_cached<H: UiHost, FTouch, FBuild>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        target: StaticSceneLayerTarget,
        scope: &'static str,
        base_key: DerivedBaseKey,
        style_key: u64,
        tile_size_canvas: f32,
        cache_rect: Rect,
        touch: FTouch,
        build_ops: FBuild,
    ) where
        FTouch: Fn(&mut CanvasPaintCache, &[SceneOp]) + Copy,
        FBuild: FnOnce(&mut Self, &mut PaintCx<'_, H>) -> Vec<SceneOp>,
    {
        let key = super::static_cache::static_layer_cache_key(
            scope,
            base_key,
            style_key,
            tile_size_canvas,
            cache_rect,
        );
        let replay_delta = Point::new(Px(0.0), Px(0.0));
        if self.try_replay_static_scene_layer(cx, target, key, replay_delta, &touch) {
            return;
        }

        let ops = build_ops(self, cx);
        self.store_and_replay_static_scene_layer(cx, target, key, replay_delta, ops, touch);
    }
}
