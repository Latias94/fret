use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_overlays<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        if self.close_command.is_some() {
            let rect = Self::close_button_rect(snapshot.pan, zoom);
            let hovered = self
                .interaction
                .last_pos
                .is_some_and(|p| Self::rect_contains(rect, p));

            let background = if hovered {
                self.style.context_menu_hover_background
            } else {
                self.style.context_menu_background
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(60),
                rect,
                background: fret_core::Paint::Solid(background),
                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.context_menu_border),

                corner_radii: Corners::all(Px(6.0 / zoom)),
            });

            let mut text_style = self.style.context_menu_text_style.clone();
            text_style.size = Px(text_style.size.0 / zoom);
            if let Some(lh) = text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }
            let pad = 10.0 / zoom;
            let constraints = TextConstraints {
                max_width: Some(Px((rect.size.width.0 - 2.0 * pad).max(0.0))),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: effective_scale_factor(cx.scale_factor, zoom),
            };
            let (blob, metrics) =
                self.paint_cache
                    .text_blob(cx.services, "Close", &text_style, constraints);

            let text_x = Px(rect.origin.x.0 + pad);
            let inner_y = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(61),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        if let Some(wire_drag) = self.interaction.wire_drag.clone() {
            self.paint_wire_drag_hint(cx, snapshot, &wire_drag, zoom);
        }

        if let Some(marquee) = self.interaction.marquee.clone() {
            self.paint_marquee(cx, &marquee, zoom);
        }

        if let Some(guides) = self.interaction.snap_guides {
            self.paint_snap_guides(
                cx,
                &guides,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_w,
                viewport_h,
            );
        }

        if let Some(searcher) = self.interaction.searcher.clone() {
            self.paint_searcher(cx, &searcher, zoom);
        }

        if let Some(menu) = self.interaction.context_menu.clone() {
            self.paint_context_menu(cx, &menu, zoom);
        }

        if let Some(toast) = self.interaction.toast.clone() {
            self.paint_toast(
                cx,
                &toast,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_h,
            );
        }
    }
}
