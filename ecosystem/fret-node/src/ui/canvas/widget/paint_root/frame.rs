use crate::ui::canvas::widget::*;

pub(super) struct PaintRootFrameViewport {
    pub(super) viewport_rect: Rect,
    pub(super) viewport_w: f32,
    pub(super) viewport_h: f32,
    pub(super) viewport_origin_x: f32,
    pub(super) viewport_origin_y: f32,
    pub(super) render_cull_rect: Option<Rect>,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn record_path_cache_stats<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };
        let (entries, stats) = self.paint_cache.diagnostics_path_cache_snapshot();
        let frame_id = cx.app.frame_id().0;
        let key = CanvasCacheKey {
            window: window.data().as_ffi(),
            node: cx.node.data().as_ffi(),
            name: "fret-node.canvas.paths",
        };
        cx.app
            .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                registry.record_path_cache(key, frame_id, entries, stats);
            });
    }

    fn resolve_canvas_chrome_hint<H: UiHost>(
        &self,
        cx: &mut PaintCx<'_, H>,
    ) -> crate::ui::CanvasChromeHint {
        if self.skin.is_some() {
            self.graph
                .read_ref(cx.app, |graph| {
                    self.skin
                        .as_ref()
                        .map(|skin| skin.canvas_chrome_hint(graph, &self.style))
                        .unwrap_or_default()
                })
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::CanvasChromeHint::default()
        }
    }

    pub(super) fn prepare_paint_root_frame<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        view_interacting: bool,
    ) -> PaintRootFrameViewport {
        self.paint_cache.begin_frame();
        self.groups_scene_cache.begin_frame();
        self.nodes_scene_cache.begin_frame();
        self.edges_scene_cache.begin_frame();
        self.edge_labels_scene_cache.begin_frame();
        self.record_path_cache_stats(cx);

        let viewport = Self::viewport_from_pan_zoom(cx.bounds, snapshot.pan, snapshot.zoom);
        let viewport_rect = viewport.visible_canvas_rect();
        let viewport_w = viewport_rect.size.width.0;
        let viewport_h = viewport_rect.size.height.0;
        let viewport_origin_x = viewport_rect.origin.x.0;
        let viewport_origin_y = viewport_rect.origin.y.0;
        let render_cull_rect = self.compute_render_cull_rect(snapshot, cx.bounds);

        cx.scene.push(SceneOp::PushClipRect {
            rect: viewport_rect,
        });

        let canvas_hint = self.resolve_canvas_chrome_hint(cx);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: viewport_rect,
            background: fret_core::Paint::Solid(
                canvas_hint
                    .background
                    .unwrap_or(self.style.paint.background),
            )
            .into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        self.paint_grid(
            cx,
            viewport_rect,
            render_cull_rect,
            snapshot.zoom,
            view_interacting,
        );

        PaintRootFrameViewport {
            viewport_rect,
            viewport_w,
            viewport_h,
            viewport_origin_x,
            viewport_origin_y,
            render_cull_rect,
        }
    }
}
