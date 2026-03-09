use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn try_replay_cached_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        key: u64,
        replay_delta: Point,
    ) -> bool {
        self.edge_labels_scene_cache
            .try_replay_with(key, cx.scene, replay_delta, |ops| {
                self.paint_cache.touch_text_blobs_in_scene_ops(ops);
            })
    }

    pub(super) fn build_single_rect_edge_labels_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        labels_key: u64,
        edges_cache_rect: Rect,
        zoom: f32,
        view_interacting: bool,
    ) {
        if self.edge_labels_scene_cache.contains_key(labels_key) {
            self.edge_labels_build_state = None;
            return;
        }

        let mut state = self
            .edge_labels_build_state
            .take()
            .filter(|state| state.key == labels_key)
            .unwrap_or_else(|| {
                self.init_edge_labels_build_state(
                    &*cx.app,
                    snapshot,
                    geom,
                    index,
                    labels_key,
                    edges_cache_rect,
                    edges_cache_rect,
                    zoom,
                )
            });

        let budget_limit = Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut budget = WorkBudget::new(budget_limit);
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

        let mut tmp = fret_core::Scene::default();
        if self.paint_edge_labels_build_state_step(
            &mut tmp,
            &*cx.app,
            cx.services,
            cx.scale_factor,
            zoom,
            bezier_steps,
            &mut state,
            &mut budget,
        ) {
            super::super::redraw_request::request_paint_redraw(cx);
        }

        if state.next_edge >= state.edges.len() {
            self.edge_labels_scene_cache
                .store_ops(labels_key, state.ops.clone());
            self.edge_labels_build_state = None;
        } else {
            self.edge_labels_build_state = Some(state);
        }
    }

    pub(super) fn replay_single_rect_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        labels_key: u64,
        replay_delta: Point,
    ) {
        if self.try_replay_cached_edge_labels(cx, labels_key, replay_delta) {
            return;
        }
        if let Some(state) = self
            .edge_labels_build_state
            .as_ref()
            .filter(|state| state.key == labels_key)
        {
            cx.scene.replay_ops_translated(&state.ops, replay_delta);
            self.paint_cache.touch_text_blobs_in_scene_ops(&state.ops);
        }
    }
}
