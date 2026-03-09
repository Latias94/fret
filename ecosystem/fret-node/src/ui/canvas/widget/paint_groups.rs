use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn paint_selected_group_overlay_rects<I>(
        &mut self,
        scene: &mut fret_core::Scene,
        rects: I,
        zoom: f32,
    ) where
        I: IntoIterator<Item = Rect>,
    {
        let group_corner = Px(10.0 / zoom);
        for rect in rects {
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: fret_core::Paint::Solid(self.style.paint.group_background).into(),

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.paint.node_border_selected).into(),

                corner_radii: Corners::all(group_corner),
            });
        }
    }

    pub(super) fn paint_groups_static(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) {
        // Groups render under edges and nodes (container frames).
        if groups.is_empty() {
            return;
        }

        let mut group_text_style = self.style.geometry.context_menu_text_style.clone();
        group_text_style.size = Px(group_text_style.size.0 / zoom);
        if let Some(lh) = group_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let group_pad = 10.0 / zoom;
        let group_corner = Px(10.0 / zoom);
        for (rect, title, _selected) in groups {
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: *rect,
                background: fret_core::Paint::Solid(self.style.paint.group_background).into(),

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.paint.group_border).into(),

                corner_radii: Corners::all(group_corner),
            });

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * group_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    scale_factor: effective_scale_factor(scale_factor, zoom),
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    services,
                    title.clone(),
                    &group_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + group_pad);
                let text_y = Px(rect.origin.y.0 + group_pad + metrics.baseline.0);
                scene.push(SceneOp::Text {
                    order: DrawOrder(1),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    paint: (self.style.paint.context_menu_text).into(),
                    outline: None,
                    shadow: None,
                });
            }
        }
    }

    pub(super) fn paint_groups_selected_overlay(
        &mut self,
        scene: &mut fret_core::Scene,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) {
        if groups.is_empty() {
            return;
        }

        self.paint_selected_group_overlay_rects(
            scene,
            groups
                .iter()
                .filter_map(|(rect, _title, selected)| selected.then_some(*rect)),
            zoom,
        );
    }

    pub(super) fn paint_selected_groups_overlay_from_snapshot<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render_cull_rect: Option<Rect>,
        zoom: f32,
    ) {
        let selected_groups = snapshot.selected_groups.clone();
        let mut overlay_rects = Vec::with_capacity(selected_groups.len());
        let _ = self.graph.read_ref(cx.app, |g| {
            for group_id in &selected_groups {
                let Some(group) = g.groups.get(group_id) else {
                    continue;
                };
                let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                let rect = Rect::new(
                    Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                    Size::new(Px(rect0.size.width), Px(rect0.size.height)),
                );
                if render_cull_rect.is_some_and(|c| !rects_intersect(rect, c)) {
                    continue;
                }
                overlay_rects.push(rect);
            }
        });

        self.paint_selected_group_overlay_rects(cx.scene, overlay_rects, zoom);
    }
}
