use crate::ui::NodeShadowHint;
use crate::ui::canvas::widget::*;
use fret_core::TextStyle;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_static_node(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        node: GraphNodeId,
        rect: Rect,
        is_selected: bool,
        title: &Arc<str>,
        body: Option<&Arc<str>>,
        pin_rows: usize,
        hint: crate::ui::NodeChromeHint,
        node_text_style: &TextStyle,
        zoom: f32,
        corner: Px,
        title_pad: f32,
        title_h: f32,
    ) {
        let background = hint.background.unwrap_or(self.style.paint.node_background);
        let border = if is_selected {
            hint.border_selected
                .or(hint.border)
                .unwrap_or(self.style.paint.node_border_selected)
        } else {
            hint.border.unwrap_or(self.style.paint.node_border)
        };
        let border_w = Px(1.0 / zoom);

        let paint_override = self
            .paint_overrides
            .as_ref()
            .and_then(|overrides| overrides.node_paint_override(node));

        let body_background: fret_core::scene::PaintBindingV1 = paint_override
            .as_ref()
            .and_then(|override_item| override_item.body_background)
            .unwrap_or_else(|| fret_core::Paint::Solid(background).into());

        let header_background: Option<fret_core::scene::PaintBindingV1> = paint_override
            .as_ref()
            .and_then(|override_item| override_item.header_background)
            .or_else(|| {
                hint.header_background
                    .map(|color| fret_core::Paint::Solid(color).into())
            });

        let border_paint: fret_core::scene::PaintBindingV1 = paint_override
            .as_ref()
            .and_then(|override_item| override_item.border_paint)
            .unwrap_or_else(|| fret_core::Paint::Solid(border).into());

        let shadow = hint.shadow;
        if let Some(shadow) = shadow
            && let Some((bounds, drop_shadow)) =
                shadow_to_drop_shadow_canvas_units(rect, zoom, shadow)
        {
            scene.push(SceneOp::PushEffect {
                bounds,
                mode: EffectMode::FilterContent,
                chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(drop_shadow)]),
                quality: EffectQuality::Auto,
            });
        }

        self.paint_static_node_quads(
            scene,
            rect,
            body_background,
            header_background,
            border_paint,
            border_w,
            corner,
            title_h,
        );

        if shadow.is_some() {
            scene.push(SceneOp::PopEffect);
        }

        self.paint_static_node_title(
            scene,
            services,
            scale_factor,
            rect,
            title,
            hint,
            node_text_style,
            zoom,
            title_pad,
            title_h,
        );
        self.paint_static_node_body(
            scene,
            services,
            scale_factor,
            rect,
            body,
            pin_rows,
            node_text_style,
            zoom,
            title_pad,
        );
    }

    fn paint_static_node_quads(
        &mut self,
        scene: &mut fret_core::Scene,
        rect: Rect,
        body_background: fret_core::scene::PaintBindingV1,
        header_background: Option<fret_core::scene::PaintBindingV1>,
        border_paint: fret_core::scene::PaintBindingV1,
        border_w: Px,
        corner: Px,
        title_h: f32,
    ) {
        scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: body_background,
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(corner),
        });

        if let Some(paint) = header_background {
            scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: Rect::new(
                    rect.origin,
                    Size::new(rect.size.width, Px(title_h.min(rect.size.height.0))),
                ),
                background: paint,
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

        scene.push(SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: fret_core::Paint::TRANSPARENT.into(),
            border: Edges::all(border_w),
            border_paint,
            corner_radii: Corners::all(corner),
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_static_node_title(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        rect: Rect,
        title: &Arc<str>,
        hint: crate::ui::NodeChromeHint,
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
            scale_factor: effective_scale_factor(scale_factor, zoom),
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(services, title.clone(), node_text_style, constraints);

        let text_x = Px(rect.origin.x.0 + title_pad);
        let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
        let text_y = Px(inner_y + metrics.baseline.0);
        scene.push(SceneOp::Text {
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

    #[allow(clippy::too_many_arguments)]
    fn paint_static_node_body(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
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
            scale_factor: effective_scale_factor(scale_factor, zoom),
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(services, body.clone(), node_text_style, constraints);

        let text_x = Px(rect.origin.x.0 + title_pad);
        let inner_y = body_top + metrics.baseline.0;
        scene.push(SceneOp::Text {
            order: DrawOrder(4),
            origin: Point::new(text_x, Px(inner_y)),
            text: blob,
            paint: self.style.paint.context_menu_text.into(),
            outline: None,
            shadow: None,
        });
    }
}

fn shadow_to_drop_shadow_canvas_units(
    rect: Rect,
    zoom: f32,
    shadow: NodeShadowHint,
) -> Option<(Rect, DropShadowV1)> {
    let z = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };

    if !shadow.offset_x_px.is_finite()
        || !shadow.offset_y_px.is_finite()
        || !shadow.blur_radius_px.is_finite()
    {
        return None;
    }

    let blur_canvas = (shadow.blur_radius_px / z).max(0.0);
    let ox_canvas = shadow.offset_x_px / z;
    let oy_canvas = shadow.offset_y_px / z;

    let pad_x = blur_canvas + ox_canvas.abs();
    let pad_y = blur_canvas + oy_canvas.abs();

    let bounds = Rect::new(
        Point::new(Px(rect.origin.x.0 - pad_x), Px(rect.origin.y.0 - pad_y)),
        Size::new(
            Px(rect.size.width.0 + 2.0 * pad_x),
            Px(rect.size.height.0 + 2.0 * pad_y),
        ),
    );

    Some((
        bounds,
        DropShadowV1 {
            offset_px: Point::new(Px(ox_canvas), Px(oy_canvas)),
            blur_radius_px: Px(blur_canvas),
            downsample: shadow.downsample,
            color: shadow.color,
        }
        .sanitize(),
    ))
}
