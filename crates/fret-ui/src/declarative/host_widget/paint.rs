use super::super::frame::*;
use super::super::paint_helpers::*;
use super::super::prelude::*;
use super::ElementHostWidget;

impl ElementHostWidget {
    pub(super) fn paint_impl<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };

        for (model, invalidation) in
            crate::elements::observed_models_for_element(cx.app, window, self.element)
        {
            (cx.observe_model)(model, invalidation);
        }

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        match instance {
            ElementInstance::Container(props) => {
                let should_draw = props.shadow.is_some()
                    || props.background.is_some()
                    || props.border_color.is_some()
                    || props.border != Edges::all(Px(0.0));

                if should_draw {
                    if let Some(shadow) = props.shadow {
                        crate::paint::paint_shadow(cx.scene, DrawOrder(0), cx.bounds, shadow);
                    }
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        background: props.background.unwrap_or(Color::TRANSPARENT),
                        border: props.border,
                        border_color: props.border_color.unwrap_or(Color::TRANSPARENT),
                        corner_radii: props.corner_radii,
                    });
                }

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    Some(props.corner_radii),
                );
            }
            ElementInstance::Semantics(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Opacity(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 {
                    return;
                }

                if opacity < 1.0 {
                    cx.scene.push(SceneOp::PushOpacity { opacity });
                }

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

                if opacity < 1.0 {
                    cx.scene.push(SceneOp::PopOpacity);
                }
            }
            ElementInstance::VisualTransform(props) => {
                let local = props.transform;
                let is_finite = local.a.is_finite()
                    && local.b.is_finite()
                    && local.c.is_finite()
                    && local.d.is_finite()
                    && local.tx.is_finite()
                    && local.ty.is_finite();

                let needs_push = is_finite && local != Transform2D::IDENTITY;
                if needs_push {
                    let origin = cx.bounds.origin;
                    let to_origin = Transform2D::translation(origin);
                    let from_origin =
                        Transform2D::translation(Point::new(Px(-origin.x.0), Px(-origin.y.0)));
                    let transform = to_origin * local * from_origin;
                    cx.scene.push(SceneOp::PushTransform { transform });
                }

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

                if needs_push {
                    cx.scene.push(SceneOp::PopTransform);
                }
            }
            ElementInstance::DismissibleLayer(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Stack(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Flex(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::RovingFlex(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.flex.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Grid(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Spacer(_props) => {}
            ElementInstance::Pressable(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

                if props.enabled
                    && cx.focus == Some(cx.node)
                    && crate::focus_visible::is_focus_visible(cx.app, cx.window)
                    && let Some(ring) = props.focus_ring
                {
                    crate::paint::paint_focus_ring(cx.scene, DrawOrder(0), cx.bounds, ring);
                }
            }
            ElementInstance::Text(props) => {
                let theme_revision = cx.theme().revision();
                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = props.style.unwrap_or(TextStyle {
                    font: FontId::default(),
                    size: font_size,
                    line_height: Some(
                        cx.theme()
                            .metric_by_key("font.line_height")
                            .unwrap_or(cx.theme().metrics.font_line_height),
                    ),
                    ..Default::default()
                });
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let constraints = TextConstraints {
                    max_width: Some(cx.bounds.size.width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };

                let scale_bits = cx.scale_factor.to_bits();
                let needs_prepare = self.text_cache.blob.is_none()
                    || self.text_cache.prepared_scale_factor_bits != Some(scale_bits)
                    || self.text_cache.last_text.as_ref() != Some(&props.text)
                    || self.text_cache.last_style.as_ref() != Some(&style)
                    || self.text_cache.last_wrap != Some(props.wrap)
                    || self.text_cache.last_overflow != Some(props.overflow)
                    || self.text_cache.last_width != Some(cx.bounds.size.width)
                    || self.text_cache.last_theme_revision != Some(theme_revision);

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.services.text().release(blob);
                    }
                    let (blob, metrics) =
                        cx.services.text().prepare(&props.text, style, constraints);
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = Some(props.text.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_width = Some(cx.bounds.size.width);
                    self.text_cache.last_theme_revision = Some(theme_revision);
                }

                let Some(blob) = self.text_cache.blob else {
                    return;
                };
                let Some(metrics) = self.text_cache.metrics else {
                    return;
                };

                let origin = fret_core::Point::new(
                    cx.bounds.origin.x,
                    cx.bounds.origin.y + metrics.baseline,
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(0),
                    origin,
                    text: blob,
                    color,
                });
            }
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.paint(cx);
            }
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                area.paint(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.paint(cx);
            }
            ElementInstance::VirtualList(props) => {
                cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

                let offset_y = props.scroll_handle.offset().y;
                let metrics = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        state.metrics.ensure(
                            props.len,
                            props.estimate_row_height,
                            props.gap,
                            props.scroll_margin,
                        );
                        state.metrics.clone()
                    },
                );

                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let y = cx.bounds.origin.y.0 + metrics.offset_for_index(idx).0 - offset_y.0;
                    let row_h = metrics.height_at(idx);
                    let child_bounds = Rect::new(
                        fret_core::Point::new(cx.bounds.origin.x, Px(y)),
                        Size::new(cx.bounds.size.width, row_h),
                    );

                    cx.scene.push(SceneOp::PushClipRect { rect: child_bounds });
                    cx.paint(child, child_bounds);
                    cx.scene.push(SceneOp::PopClip);
                }

                cx.scene.push(SceneOp::PopClip);
            }
            ElementInstance::Image(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if let Some(uv) = props.uv {
                    cx.scene.push(SceneOp::ImageRegion {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        image: props.image,
                        uv,
                        opacity,
                    });
                } else {
                    cx.scene.push(SceneOp::Image {
                        order: DrawOrder(0),
                        rect: cx.bounds,
                        image: props.image,
                        opacity,
                    });
                }
            }
            ElementInstance::SvgIcon(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 || props.color.a <= 0.0 {
                    return;
                }

                let svg = props.svg.resolve(cx.services);
                cx.scene.push(SceneOp::SvgMaskIcon {
                    order: DrawOrder(0),
                    rect: cx.bounds,
                    svg,
                    fit: props.fit,
                    color: props.color,
                    opacity,
                });
            }
            ElementInstance::Spinner(props) => {
                let theme = cx.theme();
                let base = props
                    .color
                    .or_else(|| theme.color_by_key("muted-foreground"))
                    .unwrap_or(theme.colors.text_muted);

                let n = props.dot_count.clamp(1, 32) as usize;

                let w = cx.bounds.size.width.0.max(0.0);
                let h = cx.bounds.size.height.0.max(0.0);
                let min_dim = w.min(h);
                if min_dim <= 0.0 {
                    return;
                }

                let dot = (min_dim * 0.18).clamp(2.0, (min_dim * 0.25).max(2.0));
                let radius = (min_dim * 0.5 - dot * 0.5).max(0.0);

                let cx0 = cx.bounds.origin.x.0 + w * 0.5;
                let cy0 = cx.bounds.origin.y.0 + h * 0.5;

                let speed = props.speed.max(0.0);
                if speed > 0.0 {
                    cx.app.push_effect(Effect::RequestAnimationFrame(window));
                }

                let phase = cx.app.frame_id().0 as f32 * speed;
                let active = (phase.floor() as i32).rem_euclid(n as i32) as usize;
                let tail_len = (n.min(5)).saturating_sub(1);

                for i in 0..n {
                    let dist = ((i + n) - active) % n;
                    let t = if tail_len == 0 {
                        if dist == 0 { 1.0 } else { 0.25 }
                    } else if dist == 0 {
                        1.0
                    } else if dist <= tail_len {
                        1.0 - dist as f32 / (tail_len as f32 + 1.0)
                    } else {
                        0.25
                    };

                    let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
                    let x = cx0 + radius * angle.cos() - dot * 0.5;
                    let y = cy0 + radius * angle.sin() - dot * 0.5;

                    let mut color = base;
                    color.a = (color.a * t).clamp(0.0, 1.0);

                    let rect = Rect::new(
                        fret_core::Point::new(Px(x), Px(y)),
                        Size::new(Px(dot), Px(dot)),
                    );
                    let r = Px(dot * 0.5);
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: color,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(r),
                    });
                }
            }
            ElementInstance::HoverRegion(props) => {
                let clip = matches!(props.layout.overflow, Overflow::Clip);
                if clip {
                    cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                }

                for &child in cx.children {
                    let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
                    cx.paint(child, bounds);
                }

                if clip {
                    cx.scene.push(SceneOp::PopClip);
                }
            }
            ElementInstance::PointerRegion(props) => {
                let clip = matches!(props.layout.overflow, Overflow::Clip);
                if clip {
                    cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                }

                for &child in cx.children {
                    let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
                    cx.paint(child, bounds);
                }

                if clip {
                    cx.scene.push(SceneOp::PopClip);
                }
            }
            ElementInstance::Scroll(props) => {
                let clip = matches!(props.layout.overflow, Overflow::Clip);
                if clip {
                    cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                }

                for &child in cx.children {
                    let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
                    cx.paint(child, bounds);
                }

                if clip {
                    cx.scene.push(SceneOp::PopClip);
                }
            }
            ElementInstance::Scrollbar(props) => {
                let handle = props.scroll_handle.clone();
                let (hovered, dragging) = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollbarState::default,
                    |state| (state.hovered, state.dragging_thumb),
                );

                let offset_y = handle.offset().y;
                let viewport_h = handle.viewport_size().height;
                let content_h = handle.content_size().height;
                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                if max_offset.0 <= 0.0 {
                    return;
                }

                let Some(thumb) = scrollbar_thumb_rect(cx.bounds, viewport_h, content_h, offset_y)
                else {
                    return;
                };

                let mut bg = if hovered || dragging {
                    props.style.thumb_hover
                } else {
                    props.style.thumb
                };
                if !(hovered || dragging) {
                    bg.a *= props.style.thumb_idle_alpha.clamp(0.0, 1.0);
                }

                let inset = 1.0f32.min(thumb.size.width.0 * 0.25);
                let rect = Rect::new(
                    fret_core::Point::new(Px(thumb.origin.x.0 + inset), thumb.origin.y),
                    Size::new(
                        Px((thumb.size.width.0 - inset * 2.0).max(0.0)),
                        thumb.size.height,
                    ),
                );

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(20_000),
                    rect,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(999.0)),
                });
            }
        }
    }
}
