use crate::ui::NodeChromeHint;
use crate::ui::canvas::widget::*;
use crate::ui::presenter::NodeResizeHandleSet;
use fret_core::TextStyle;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_full_node<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        node: GraphNodeId,
        rect: Rect,
        is_selected: bool,
        title: &Arc<str>,
        body: Option<&Arc<str>>,
        pin_rows: usize,
        resize_handles: &NodeResizeHandleSet,
        hint: NodeChromeHint,
        node_text_style: &TextStyle,
        zoom: f32,
        corner: Px,
        title_pad: f32,
        title_h: f32,
    ) {
        let bg = hint.background.unwrap_or(self.style.paint.node_background);
        let border_color = if is_selected {
            hint.border_selected
                .or(hint.border)
                .unwrap_or(self.style.paint.node_border_selected)
        } else {
            hint.border.unwrap_or(self.style.paint.node_border)
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: fret_core::Paint::Solid(bg).into(),
            border: Edges::all(Px(1.0 / zoom)),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(corner),
        });

        self.paint_full_node_header(cx, rect, hint, corner, title_h);
        self.paint_full_node_resize_handles(cx, node, rect, is_selected, resize_handles, zoom);
        self.paint_full_node_title(
            cx,
            rect,
            title,
            hint,
            node_text_style,
            zoom,
            title_pad,
            title_h,
        );
        self.paint_full_node_body(cx, rect, body, pin_rows, node_text_style, zoom, title_pad);
    }

    fn paint_full_node_header<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        rect: Rect,
        hint: NodeChromeHint,
        corner: Px,
        title_h: f32,
    ) {
        if let Some(header_bg) = hint.header_background
            && title_h.is_finite()
            && title_h > 0.0
        {
            let h = title_h.min(rect.size.height.0);
            let header_rect = Rect::new(rect.origin, Size::new(rect.size.width, Px(h)));
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: header_rect,
                background: fret_core::Paint::Solid(header_bg).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: Corners {
                    top_left: corner,
                    top_right: corner,
                    bottom_right: Px(0.0),
                    bottom_left: Px(0.0),
                },
            });
        }
    }

    fn paint_full_node_resize_handles<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        node: GraphNodeId,
        rect: Rect,
        is_selected: bool,
        resize_handles: &NodeResizeHandleSet,
        zoom: f32,
    ) {
        let show_resize_handle = is_selected
            && (self
                .interaction
                .node_resize
                .as_ref()
                .is_some_and(|resize| resize.node == node)
                || self
                    .interaction
                    .last_pos
                    .is_some_and(|pos| Self::rect_contains(rect, pos)));
        if !show_resize_handle {
            return;
        }

        for handle in NodeResizeHandle::ALL {
            if !resize_handles.contains(handle) {
                continue;
            }
            let rect = self.node_resize_handle_rect(rect, handle, zoom);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(5),
                rect,
                background: fret_core::Paint::Solid(self.style.paint.resize_handle_background)
                    .into(),
                border: Edges::all(Px(1.0 / zoom)),
                border_paint: fret_core::Paint::Solid(self.style.paint.resize_handle_border).into(),
                corner_radii: Corners::all(Px(2.0 / zoom)),
            });
        }
    }

    fn paint_full_node_title<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        rect: Rect,
        title: &Arc<str>,
        hint: NodeChromeHint,
        node_text_style: &TextStyle,
        zoom: f32,
        title_pad: f32,
        title_h: f32,
    ) {
        if title.is_empty() {
            return;
        }

        let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
        let constraints = TextConstraints {
            max_width: Some(Px(max_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, title.clone(), node_text_style, constraints);

        let text_x = Px(rect.origin.x.0 + title_pad);
        let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
        let text_y = Px(inner_y + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(4),
            origin: Point::new(text_x, text_y),
            text: blob,
            paint: hint
                .title_text
                .unwrap_or(self.style.paint.context_menu_text)
                .into(),
            outline: None,
            shadow: None,
        });
    }

    fn paint_full_node_body<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        rect: Rect,
        body: Option<&Arc<str>>,
        pin_rows: usize,
        node_text_style: &TextStyle,
        zoom: f32,
        title_pad: f32,
    ) {
        let Some(body) = body else {
            return;
        };
        if body.is_empty() {
            return;
        }

        let pin_rows = pin_rows as f32;
        let body_top = rect.origin.y.0
            + (self.style.geometry.node_header_height
                + self.style.geometry.node_padding
                + pin_rows * self.style.geometry.pin_row_height
                + self.style.geometry.node_padding)
                / zoom;

        let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
        let constraints = TextConstraints {
            max_width: Some(Px(max_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, body.clone(), node_text_style, constraints);

        let text_x = Px(rect.origin.x.0 + title_pad);
        let inner_y = body_top + metrics.baseline.0;
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(4),
            origin: Point::new(text_x, Px(inner_y)),
            text: blob,
            paint: self.style.paint.context_menu_text.into(),
            outline: None,
            shadow: None,
        });
    }
}
