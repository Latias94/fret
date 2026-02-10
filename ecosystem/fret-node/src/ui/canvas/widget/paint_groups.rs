use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
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

        let mut group_text_style = self.style.context_menu_text_style.clone();
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
                background: fret_core::Paint::Solid(self.style.group_background),

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.group_border),

                corner_radii: Corners::all(group_corner),
            });

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * group_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
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
                    color: self.style.context_menu_text,
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

        let group_corner = Px(10.0 / zoom);
        for (rect, _title, selected) in groups {
            if !*selected {
                continue;
            }
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: *rect,
                background: fret_core::Paint::Solid(self.style.group_background),

                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.node_border_selected),

                corner_radii: Corners::all(group_corner),
            });
        }
    }
}
