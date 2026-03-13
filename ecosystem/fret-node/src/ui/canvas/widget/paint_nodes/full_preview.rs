use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::canvas::state::InsertNodeDragPreview;
use crate::ui::canvas::widget::*;
use fret_core::TextStyle;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_insert_node_drag_preview<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        preview: &InsertNodeDragPreview,
        node_text_style: &TextStyle,
        zoom: f32,
        corner: Px,
        title_pad: f32,
        title_h: f32,
    ) {
        let z = zoom.max(1.0e-6);
        let (w_px, h_px) = node_size_default_px(1, 1, &self.style);
        let w = w_px / z;
        let h = h_px / z;
        let rect = Rect::new(
            Point::new(Px(preview.pos.x.0 - 0.5 * w), Px(preview.pos.y.0 - 0.5 * h)),
            Size::new(Px(w), Px(h)),
        );

        let mut bg = self.style.paint.node_background;
        bg.a *= 0.55;
        let mut border_color = self.style.paint.node_border_selected;
        border_color.a *= 0.85;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: fret_core::Paint::Solid(bg).into(),
            border: Edges::all(Px(1.0 / z)),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(corner),
        });

        if preview.label.is_empty() {
            return;
        }

        let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
        let constraints = TextConstraints {
            max_width: Some(Px(max_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor * zoom,
        };
        let (blob, metrics) = self.paint_cache.text_blob(
            cx.services,
            preview.label.clone(),
            node_text_style,
            constraints,
        );
        let text_x = Px(rect.origin.x.0 + title_pad);
        let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
        let text_y = Px(inner_y + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(4),
            origin: Point::new(text_x, text_y),
            text: blob,
            paint: self.style.paint.context_menu_text.into(),
            outline: None,
            shadow: None,
        });
    }
}
