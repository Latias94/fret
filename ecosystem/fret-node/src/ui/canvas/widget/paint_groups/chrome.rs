use super::super::*;

fn group_corner_radius(zoom: f32) -> Px {
    Px(10.0 / zoom)
}

fn group_border_width(zoom: f32) -> Px {
    Px(1.0 / zoom)
}

fn group_padding(zoom: f32) -> f32 {
    10.0 / zoom
}

fn group_title_max_width(rect: Rect, padding: f32) -> f32 {
    (rect.size.width.0 - 2.0 * padding).max(0.0)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_groups_static(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) {
        if groups.is_empty() {
            return;
        }

        let mut group_text_style = self.style.geometry.context_menu_text_style.clone();
        group_text_style.size = Px(group_text_style.size.0 / zoom);
        if let Some(line_height) = group_text_style.line_height.as_mut() {
            line_height.0 /= zoom;
        }

        let padding = group_padding(zoom);
        let corner_radius = group_corner_radius(zoom);
        let border_width = group_border_width(zoom);
        for (rect, title, _selected) in groups {
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: *rect,
                background: fret_core::Paint::Solid(self.style.paint.group_background).into(),
                border: Edges::all(border_width),
                border_paint: fret_core::Paint::Solid(self.style.paint.group_border).into(),
                corner_radii: Corners::all(corner_radius),
            });

            if !title.is_empty() {
                let constraints = TextConstraints {
                    max_width: Some(Px(group_title_max_width(*rect, padding))),
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

                let text_x = Px(rect.origin.x.0 + padding);
                let text_y = Px(rect.origin.y.0 + padding + metrics.baseline.0);
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
}

#[cfg(test)]
mod tests;
