use super::super::frame::*;
use super::super::paint_helpers::*;
use super::super::prelude::*;
use super::ElementHostWidget;
use std::time::Instant;

impl ElementHostWidget {
    pub(super) fn paint_impl<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };

        let models_started = cx.tree.debug_enabled().then(Instant::now);
        crate::elements::with_observed_models_for_element(cx.app, window, self.element, |items| {
            for &(model, invalidation) in items {
                (cx.observe_model)(model, invalidation);
            }
            if let Some(started) = models_started.as_ref() {
                cx.tree
                    .debug_record_paint_host_widget_observed_models(started.elapsed(), items.len());
            }
        });

        let globals_started = cx.tree.debug_enabled().then(Instant::now);
        crate::elements::with_observed_globals_for_element(cx.app, window, self.element, |items| {
            for &(global, invalidation) in items {
                (cx.observe_global)(global, invalidation);
            }
            if let Some(started) = globals_started.as_ref() {
                cx.tree.debug_record_paint_host_widget_observed_globals(
                    started.elapsed(),
                    items.len(),
                );
            }
        });

        let instance_started = cx.tree.debug_enabled().then(Instant::now);
        let instance = self.instance(cx.app, window, cx.node);
        if let Some(instance_started) = instance_started {
            cx.tree
                .debug_record_paint_host_widget_instance_lookup(instance_started.elapsed());
        }
        let Some(instance) = instance else {
            return;
        };

        match instance {
            ElementInstance::Container(props) => {
                let bounds = if props.snap_to_device_pixels {
                    crate::pixel_snap::snap_rect_edges_round(cx.bounds, cx.scale_factor)
                } else {
                    cx.bounds
                };

                let should_draw = props.shadow.is_some()
                    || props.background.is_some()
                    || props.border_color.is_some()
                    || props.border != Edges::all(Px(0.0));

                if should_draw {
                    if let Some(shadow) = props.shadow {
                        crate::paint::paint_shadow(cx.scene, DrawOrder(0), bounds, shadow);
                    }
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: bounds,
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

                let focused = cx.focus.is_some_and(|focus| {
                    if props.focus_within {
                        cx.tree.is_descendant(cx.node, focus)
                    } else {
                        focus == cx.node
                    }
                });

                if focused && crate::focus_visible::is_focus_visible(cx.app, cx.window) {
                    if let Some(border_color) = props.focus_border_color {
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(1),
                            rect: bounds,
                            background: Color::TRANSPARENT,
                            border: props.border,
                            border_color,
                            corner_radii: props.corner_radii,
                        });
                    }

                    if let Some(ring) = props.focus_ring {
                        crate::paint::paint_focus_ring(cx.scene, DrawOrder(2), bounds, ring);
                    }
                }
            }
            ElementInstance::Semantics(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::SemanticFlex(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.flex.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::ViewCache(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            #[cfg(feature = "unstable-retained-bridge")]
            ElementInstance::RetainedSubtree(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::FocusScope(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::InteractivityGate(props) => {
                if !props.present {
                    return;
                }

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
            ElementInstance::EffectLayer(props) => {
                if cx.bounds.size.width.0 <= 0.0 || cx.bounds.size.height.0 <= 0.0 {
                    return;
                }

                if !props.chain.is_empty() {
                    cx.scene.push(SceneOp::PushEffect {
                        bounds: cx.bounds,
                        mode: props.mode,
                        chain: props.chain,
                        quality: props.quality,
                    });
                }

                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );

                if !props.chain.is_empty() {
                    cx.scene.push(SceneOp::PopEffect);
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
            ElementInstance::RenderTransform(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::FractionalRenderTransform(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
            }
            ElementInstance::Anchored(props) => {
                paint_children_clipped_if(
                    cx,
                    matches!(props.layout.overflow, Overflow::Clip),
                    None,
                );
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
                    let bounds = props.focus_ring_bounds.map_or(cx.bounds, |b| {
                        Rect::new(
                            Point::new(
                                cx.bounds.origin.x + b.origin.x,
                                cx.bounds.origin.y + b.origin.y,
                            ),
                            b.size,
                        )
                    });
                    crate::paint::paint_focus_ring(cx.scene, DrawOrder(0), bounds, ring);
                }
            }
            ElementInstance::Text(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let theme = cx.theme().snapshot();
                let style = props.resolved_text_style(theme);
                let input = props.build_text_input_with_style(style.clone());
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let max_width =
                    crate::pixel_snap::snap_px_round(cx.bounds.size.width, cx.scale_factor);
                let max_width = cx.tree.maybe_bucket_text_wrap_width(props.wrap, max_width);
                let constraints = TextConstraints {
                    max_width: Some(max_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };
                cx.tree
                    .debug_record_text_constraints_prepared(cx.node, constraints);

                let scale_bits = cx.scale_factor.to_bits();
                let blob_missing = self.text_cache.blob.is_none();
                let scale_changed = self.text_cache.prepared_scale_factor_bits != Some(scale_bits);
                let text_changed = self.text_cache.last_text.as_ref() != Some(&props.text);
                let style_changed = self.text_cache.last_style.as_ref() != Some(&style);
                let wrap_changed = self.text_cache.last_wrap != Some(props.wrap);
                let overflow_changed = self.text_cache.last_overflow != Some(props.overflow);
                let width_changed = self.text_cache.last_width != Some(max_width);
                let font_stack_changed =
                    self.text_cache.last_font_stack_key != Some(font_stack_key);
                let needs_prepare = blob_missing
                    || scale_changed
                    || text_changed
                    || style_changed
                    || wrap_changed
                    || overflow_changed
                    || width_changed
                    || font_stack_changed;
                let reasons_mask = (blob_missing as u16)
                    | ((scale_changed as u16) << 1)
                    | ((text_changed as u16) << 2)
                    | ((false as u16) << 3)
                    | ((style_changed as u16) << 4)
                    | ((wrap_changed as u16) << 5)
                    | ((overflow_changed as u16) << 6)
                    | ((width_changed as u16) << 7)
                    | ((font_stack_changed as u16) << 8);

                if needs_prepare && cx.tree.debug_enabled() {
                    cx.tree.debug_record_paint_text_prepare_reasons(
                        blob_missing,
                        scale_changed,
                        text_changed,
                        false,
                        style_changed,
                        wrap_changed,
                        overflow_changed,
                        width_changed,
                        font_stack_changed,
                    );
                }

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.services.text().release(blob);
                    }
                    let prepare_started = cx.tree.debug_enabled().then(Instant::now);
                    let (blob, metrics) = cx.services.text().prepare(&input, constraints);
                    if let Some(prepare_started) = prepare_started {
                        let elapsed = prepare_started.elapsed();
                        cx.tree.debug_record_paint_text_prepare(elapsed);
                        cx.tree.debug_record_paint_text_prepare_hotspot(
                            cx.node,
                            Some(self.element),
                            "Text",
                            props.text.len().min(u32::MAX as usize) as u32,
                            constraints,
                            reasons_mask,
                            elapsed,
                        );
                    }
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = Some(props.text.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_width = Some(max_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
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
            ElementInstance::StyledText(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let theme = cx.theme().snapshot();
                let style = props.resolved_text_style(theme);
                let input = props.build_text_input_with_style(style.clone());
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let max_width =
                    crate::pixel_snap::snap_px_round(cx.bounds.size.width, cx.scale_factor);
                let max_width = cx.tree.maybe_bucket_text_wrap_width(props.wrap, max_width);
                let constraints = TextConstraints {
                    max_width: Some(max_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };
                cx.tree
                    .debug_record_text_constraints_prepared(cx.node, constraints);

                let scale_bits = cx.scale_factor.to_bits();
                let blob_missing = self.text_cache.blob.is_none();
                let scale_changed = self.text_cache.prepared_scale_factor_bits != Some(scale_bits);
                let rich_changed = self.text_cache.last_rich.as_ref() != Some(&props.rich);
                let style_changed = self.text_cache.last_style.as_ref() != Some(&style);
                let wrap_changed = self.text_cache.last_wrap != Some(props.wrap);
                let overflow_changed = self.text_cache.last_overflow != Some(props.overflow);
                let width_changed = self.text_cache.last_width != Some(max_width);
                let font_stack_changed =
                    self.text_cache.last_font_stack_key != Some(font_stack_key);
                let needs_prepare = blob_missing
                    || scale_changed
                    || rich_changed
                    || style_changed
                    || wrap_changed
                    || overflow_changed
                    || width_changed
                    || font_stack_changed;
                let reasons_mask = (blob_missing as u16)
                    | ((scale_changed as u16) << 1)
                    | ((false as u16) << 2)
                    | ((rich_changed as u16) << 3)
                    | ((style_changed as u16) << 4)
                    | ((wrap_changed as u16) << 5)
                    | ((overflow_changed as u16) << 6)
                    | ((width_changed as u16) << 7)
                    | ((font_stack_changed as u16) << 8);

                if needs_prepare && cx.tree.debug_enabled() {
                    cx.tree.debug_record_paint_text_prepare_reasons(
                        blob_missing,
                        scale_changed,
                        false,
                        rich_changed,
                        style_changed,
                        wrap_changed,
                        overflow_changed,
                        width_changed,
                        font_stack_changed,
                    );
                }

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.services.text().release(blob);
                    }
                    let prepare_started = cx.tree.debug_enabled().then(Instant::now);
                    let (blob, metrics) = cx.services.text().prepare(&input, constraints);
                    if let Some(prepare_started) = prepare_started {
                        let elapsed = prepare_started.elapsed();
                        cx.tree.debug_record_paint_text_prepare(elapsed);
                        cx.tree.debug_record_paint_text_prepare_hotspot(
                            cx.node,
                            Some(self.element),
                            "StyledText",
                            props.rich.text.len().min(u32::MAX as usize) as u32,
                            constraints,
                            reasons_mask,
                            elapsed,
                        );
                    }
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = None;
                    self.text_cache.last_rich = Some(props.rich.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_width = Some(max_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
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
            ElementInstance::SelectableText(props) => {
                cx.observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
                let font_stack_key = cx
                    .app
                    .global::<fret_runtime::TextFontStackKey>()
                    .map(|k| k.0)
                    .unwrap_or(0);
                let theme = cx.theme().snapshot();
                let style = props.resolved_text_style(theme);
                let input = props.build_text_input_with_style(style.clone());
                let color = props
                    .color
                    .or_else(|| cx.theme().color_by_key("foreground"))
                    .unwrap_or(cx.theme().colors.text_primary);
                let max_width =
                    crate::pixel_snap::snap_px_round(cx.bounds.size.width, cx.scale_factor);
                let max_width = cx.tree.maybe_bucket_text_wrap_width(props.wrap, max_width);
                let constraints = TextConstraints {
                    max_width: Some(max_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };
                cx.tree
                    .debug_record_text_constraints_prepared(cx.node, constraints);

                let scale_bits = cx.scale_factor.to_bits();
                let blob_missing = self.text_cache.blob.is_none();
                let scale_changed = self.text_cache.prepared_scale_factor_bits != Some(scale_bits);
                let rich_changed = self.text_cache.last_rich.as_ref() != Some(&props.rich);
                let style_changed = self.text_cache.last_style.as_ref() != Some(&style);
                let wrap_changed = self.text_cache.last_wrap != Some(props.wrap);
                let overflow_changed = self.text_cache.last_overflow != Some(props.overflow);
                let width_changed = self.text_cache.last_width != Some(max_width);
                let font_stack_changed =
                    self.text_cache.last_font_stack_key != Some(font_stack_key);
                let needs_prepare = blob_missing
                    || scale_changed
                    || rich_changed
                    || style_changed
                    || wrap_changed
                    || overflow_changed
                    || width_changed
                    || font_stack_changed;
                let reasons_mask = (blob_missing as u16)
                    | ((scale_changed as u16) << 1)
                    | ((false as u16) << 2)
                    | ((rich_changed as u16) << 3)
                    | ((style_changed as u16) << 4)
                    | ((wrap_changed as u16) << 5)
                    | ((overflow_changed as u16) << 6)
                    | ((width_changed as u16) << 7)
                    | ((font_stack_changed as u16) << 8);

                if needs_prepare && cx.tree.debug_enabled() {
                    cx.tree.debug_record_paint_text_prepare_reasons(
                        blob_missing,
                        scale_changed,
                        false,
                        rich_changed,
                        style_changed,
                        wrap_changed,
                        overflow_changed,
                        width_changed,
                        font_stack_changed,
                    );
                }

                if needs_prepare {
                    if let Some(blob) = self.text_cache.blob.take() {
                        cx.services.text().release(blob);
                    }
                    let prepare_started = cx.tree.debug_enabled().then(Instant::now);
                    let (blob, metrics) = cx.services.text().prepare(&input, constraints);
                    if let Some(prepare_started) = prepare_started {
                        let elapsed = prepare_started.elapsed();
                        cx.tree.debug_record_paint_text_prepare(elapsed);
                        cx.tree.debug_record_paint_text_prepare_hotspot(
                            cx.node,
                            Some(self.element),
                            "SelectableText",
                            props.rich.text.len().min(u32::MAX as usize) as u32,
                            constraints,
                            reasons_mask,
                            elapsed,
                        );
                    }
                    self.text_cache.blob = Some(blob);
                    self.text_cache.metrics = Some(metrics);
                    self.text_cache.prepared_scale_factor_bits = Some(scale_bits);
                    self.text_cache.last_text = None;
                    self.text_cache.last_rich = Some(props.rich.clone());
                    self.text_cache.last_style = Some(style);
                    self.text_cache.last_wrap = Some(props.wrap);
                    self.text_cache.last_overflow = Some(props.overflow);
                    self.text_cache.last_width = Some(max_width);
                    self.text_cache.last_font_stack_key = Some(font_stack_key);
                }

                let Some(blob) = self.text_cache.blob else {
                    return;
                };
                let Some(metrics) = self.text_cache.metrics else {
                    return;
                };

                // Ensure any persisted selection state remains valid for this frame's text buffer.
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::SelectableTextState::default,
                    |state| {
                        crate::text_edit::utf8::clamp_selection_to_grapheme_boundaries(
                            &props.rich.text,
                            &mut state.selection_anchor,
                            &mut state.caret,
                        );
                    },
                );

                let focused = cx.focus == Some(cx.node);
                let (dragging, last_pointer_pos) = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::SelectableTextState::default,
                    |state| (state.dragging, state.last_pointer_pos),
                );
                if focused
                    && dragging
                    && let Some(pointer_pos) = last_pointer_pos
                {
                    let local = fret_core::Point::new(
                        fret_core::Px(pointer_pos.x.0 - cx.bounds.origin.x.0),
                        fret_core::Px(pointer_pos.y.0 - cx.bounds.origin.y.0),
                    );
                    let hit = cx.services.hit_test_point(blob, local);
                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::SelectableTextState::default,
                        |state| {
                            state.caret = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                                &props.rich.text,
                                hit.index,
                            );
                            state.affinity = hit.affinity;
                        },
                    );
                }
                let clip = fret_core::Rect::new(
                    fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                    cx.bounds.size,
                );

                let mut bg_runs: Vec<(usize, usize, Color)> = Vec::new();
                let mut rects: Vec<fret_core::Rect> = Vec::new();

                let mut offset = 0usize;
                let mut active_bg: Option<(usize, Color)> = None;

                for span in props.rich.spans.as_ref() {
                    let end = offset.saturating_add(span.len);

                    match (active_bg, span.paint.bg) {
                        (Some((start, bg)), Some(next)) if bg == next => {}
                        (Some((start, bg)), Some(next)) => {
                            if start < offset {
                                bg_runs.push((start, offset, bg));
                            }
                            active_bg = Some((offset, next));
                        }
                        (Some((start, bg)), None) => {
                            if start < offset {
                                bg_runs.push((start, offset, bg));
                            }
                            active_bg = None;
                        }
                        (None, Some(next)) => {
                            active_bg = Some((offset, next));
                        }
                        (None, None) => {}
                    }

                    offset = end;
                }

                if let Some((start, bg)) = active_bg
                    && start < offset
                {
                    bg_runs.push((start, offset, bg));
                }

                for (start, end, bg) in bg_runs {
                    if start >= end {
                        continue;
                    }
                    rects.clear();
                    cx.services
                        .selection_rects_clipped(blob, (start, end), clip, &mut rects);
                    for r in rects.iter() {
                        let rect = fret_core::Rect::new(
                            fret_core::Point::new(
                                fret_core::Px(cx.bounds.origin.x.0 + r.origin.x.0),
                                fret_core::Px(cx.bounds.origin.y.0 + r.origin.y.0),
                            ),
                            r.size,
                        );
                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(0),
                            rect,
                            background: bg,
                            border: fret_core::Edges::all(fret_core::Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: fret_core::Corners::all(fret_core::Px(0.0)),
                        });
                    }
                }

                if focused {
                    let (anchor, caret) = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::SelectableTextState::default,
                        |state| (state.selection_anchor, state.caret),
                    );
                    let start = anchor.min(caret);
                    let end = anchor.max(caret);
                    if start < end {
                        let mut rects: Vec<fret_core::Rect> = Vec::new();
                        cx.services
                            .selection_rects_clipped(blob, (start, end), clip, &mut rects);
                        let sel_color = cx.theme().color_required("selection.background");
                        for r in rects {
                            let rect = fret_core::Rect::new(
                                fret_core::Point::new(
                                    fret_core::Px(cx.bounds.origin.x.0 + r.origin.x.0),
                                    fret_core::Px(cx.bounds.origin.y.0 + r.origin.y.0),
                                ),
                                r.size,
                            );
                            cx.scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: sel_color,
                                border: fret_core::Edges::all(fret_core::Px(0.0)),
                                border_color: Color::TRANSPARENT,
                                corner_radii: fret_core::Corners::all(fret_core::Px(0.0)),
                            });
                        }
                    }
                }

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

                if dragging
                    && let Some(pointer_pos) = last_pointer_pos
                    && let Some(window) = cx.window
                {
                    const EDGE_MARGIN: Px = Px(24.0);
                    const MAX_STEP: Px = Px(16.0);

                    let mut node = cx.node;
                    while let Some(parent) = cx.tree.node_parent(node) {
                        node = parent;
                        let Some(record) = crate::declarative::frame::element_record_for_node(
                            cx.app, window, node,
                        ) else {
                            continue;
                        };

                        let (handle, handle_key, scroll_x, scroll_y) = match record.instance {
                            ElementInstance::Scroll(props) => {
                                let handle = if let Some(handle) = props.scroll_handle.as_ref() {
                                    handle.clone()
                                } else {
                                    crate::elements::with_element_state(
                                        &mut *cx.app,
                                        window,
                                        record.element,
                                        crate::element::ScrollState::default,
                                        |state| state.scroll_handle.clone(),
                                    )
                                };
                                let key = props.scroll_handle.as_ref().map(|h| h.binding_key());
                                (handle, key, props.axis.scroll_x(), props.axis.scroll_y())
                            }
                            ElementInstance::VirtualList(props) => {
                                if props.axis == fret_core::Axis::Vertical {
                                    (
                                        props.scroll_handle.base_handle().clone(),
                                        Some(props.scroll_handle.base_handle().binding_key()),
                                        false,
                                        true,
                                    )
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
                        };

                        if !scroll_x && !scroll_y {
                            continue;
                        }

                        let Some(scroll_bounds) = cx.tree.node_bounds(node) else {
                            break;
                        };
                        let left = scroll_bounds.origin.x;
                        let right =
                            fret_core::Px(scroll_bounds.origin.x.0 + scroll_bounds.size.width.0);
                        let top = scroll_bounds.origin.y;
                        let bottom =
                            fret_core::Px(scroll_bounds.origin.y.0 + scroll_bounds.size.height.0);

                        let mut step_x = Px(0.0);
                        if scroll_x {
                            if pointer_pos.x.0 < left.0 + EDGE_MARGIN.0 {
                                let t = ((left.0 + EDGE_MARGIN.0 - pointer_pos.x.0)
                                    / EDGE_MARGIN.0)
                                    .clamp(0.0, 1.0);
                                step_x = Px(-MAX_STEP.0 * t);
                            } else if pointer_pos.x.0 > right.0 - EDGE_MARGIN.0 {
                                let t = ((pointer_pos.x.0 - (right.0 - EDGE_MARGIN.0))
                                    / EDGE_MARGIN.0)
                                    .clamp(0.0, 1.0);
                                step_x = Px(MAX_STEP.0 * t);
                            }
                        }

                        let mut step_y = Px(0.0);
                        if scroll_y {
                            if pointer_pos.y.0 < top.0 + EDGE_MARGIN.0 {
                                let t = ((top.0 + EDGE_MARGIN.0 - pointer_pos.y.0) / EDGE_MARGIN.0)
                                    .clamp(0.0, 1.0);
                                step_y = Px(-MAX_STEP.0 * t);
                            } else if pointer_pos.y.0 > bottom.0 - EDGE_MARGIN.0 {
                                let t = ((pointer_pos.y.0 - (bottom.0 - EDGE_MARGIN.0))
                                    / EDGE_MARGIN.0)
                                    .clamp(0.0, 1.0);
                                step_y = Px(MAX_STEP.0 * t);
                            }
                        }

                        if step_x.0.abs() < 0.01 && step_y.0.abs() < 0.01 {
                            break;
                        }

                        let prev = handle.offset();
                        handle.set_offset(fret_core::Point::new(
                            Px(prev.x.0 + step_x.0),
                            Px(prev.y.0 + step_y.0),
                        ));
                        let next = handle.offset();
                        let did_scroll = (next.y.0 - prev.y.0).abs() > 0.01
                            || (next.x.0 - prev.x.0).abs() > 0.01;

                        if did_scroll {
                            if let Some(handle_key) = handle_key {
                                let bound =
                                    crate::declarative::frame::bound_elements_for_scroll_handle(
                                        cx.app, window, handle_key,
                                    );
                                let mut unique =
                                    std::collections::HashSet::with_capacity(bound.len());
                                for element in bound {
                                    if !unique.insert(element) {
                                        continue;
                                    }
                                    let Some(node) =
                                        crate::declarative::mount::node_for_element_in_window_frame(
                                            cx.app, window, element,
                                        )
                                    else {
                                        continue;
                                    };
                                    cx.tree.invalidate(node, Invalidation::Layout);
                                    cx.tree.invalidate(node, Invalidation::Paint);
                                }
                            }

                            cx.tree.invalidate(node, Invalidation::HitTest);
                            cx.app.request_redraw(window);
                            cx.app.push_effect(Effect::RequestAnimationFrame(window));
                        }

                        break;
                    }
                }
            }
            ElementInstance::TextInput(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(model.clone()));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != model_id {
                    input.set_model(model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_placeholder(props.placeholder);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.paint(cx);
            }
            ElementInstance::TextArea(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(model.clone()));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != model_id {
                    area.set_model(model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                area.paint(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            model.clone(),
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != model_id {
                    group.set_model(model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.paint(cx);
            }
            ElementInstance::VirtualList(props) => {
                if cx.tree.view_cache_enabled()
                    && !cx.tree.inspection_active()
                    && let Some(window) = cx.window
                {
                    let requested_refresh = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::VirtualListState::default,
                        |state| {
                            let axis = props.axis;
                            state.metrics.ensure_with_mode(
                                props.measure_mode,
                                props.len,
                                props.estimate_row_height,
                                props.gap,
                                props.scroll_margin,
                            );
                            let viewport = match axis {
                                fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                                fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                            };
                            if viewport.0 <= 0.0 || props.len == 0 {
                                return false;
                            }

                            let handle_offset = match axis {
                                fret_core::Axis::Vertical => props.scroll_handle.offset().y,
                                fret_core::Axis::Horizontal => props.scroll_handle.offset().x,
                            };
                            let offset = state.metrics.clamp_offset(handle_offset, viewport);
                            let Some(range) =
                                state
                                    .metrics
                                    .visible_range(offset, viewport, props.overscan)
                            else {
                                return false;
                            };
                            crate::virtual_list::virtual_list_needs_visible_range_refresh(
                                &props.visible_items,
                                range,
                            )
                        },
                    );
                    cx.tree
                        .debug_record_virtual_list_visible_range_check(requested_refresh);
                }

                cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });
                let children_transform = cx.children_render_transform;
                if let Some(transform) = children_transform {
                    cx.scene.push(SceneOp::PushTransform { transform });
                }
                let accumulated = children_transform
                    .map(|t| cx.accumulated_transform.compose(t))
                    .unwrap_or(cx.accumulated_transform);

                for &child in cx.children {
                    let Some(child_bounds) = cx.child_bounds(child) else {
                        continue;
                    };
                    // Clip rects live in the current transform space at push time (see
                    // `fret-render`'s affine clip conformance tests). Since we push the scroll
                    // translation before painting the children, the per-row clip rect must be
                    // specified in the same pre-transform/content coordinate space as
                    // `child_bounds`.
                    let clip_rect = child_bounds;
                    cx.scene.push(SceneOp::PushClipRect { rect: clip_rect });
                    cx.tree.paint_node(
                        cx.app,
                        cx.services,
                        child,
                        child_bounds,
                        cx.scene,
                        cx.scale_factor,
                        accumulated,
                    );
                    cx.scene.push(SceneOp::PopClip);
                }

                if children_transform.is_some() {
                    cx.scene.push(SceneOp::PopTransform);
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
            ElementInstance::Canvas(props) => {
                let on_paint = crate::elements::with_element_state(
                    cx.app,
                    window,
                    self.element,
                    crate::canvas::CanvasPaintHooks::default,
                    |hooks| hooks.on_paint.clone(),
                );

                self.canvas_cache
                    .begin_paint(cx.app.frame_id().0, props.cache_policy);
                if let Some(on_paint) = on_paint {
                    {
                        let mut host = crate::canvas::UiCanvasHostAdapter::new(cx);
                        let mut painter =
                            crate::canvas::CanvasPainter::new(&mut host, &mut self.canvas_cache);
                        (on_paint)(&mut painter);
                    }
                }
                self.canvas_cache.end_paint(cx.services);
            }
            ElementInstance::ViewportSurface(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 {
                    return;
                }
                let mapping = fret_core::ViewportMapping {
                    content_rect: cx.bounds,
                    target_px_size: props.target_px_size,
                    fit: props.fit,
                };
                cx.scene.push(SceneOp::ViewportSurface {
                    order: DrawOrder(0),
                    rect: mapping.map().draw_rect,
                    target: props.target,
                    opacity,
                });
            }
            ElementInstance::SvgIcon(props) => {
                let opacity = props.opacity.clamp(0.0, 1.0);
                if opacity <= 0.0 || props.color.a <= 0.0 {
                    return;
                }

                let svg = self.resolve_svg_for_icon(cx.services, &props.svg);
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
                    .unwrap_or_else(|| theme.color_required("muted-foreground"));

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
            ElementInstance::TextInputRegion(props) => {
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
            ElementInstance::InternalDragRegion(props) => {
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
            ElementInstance::WheelRegion(props) => {
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

                let is_horizontal = matches!(props.axis, crate::element::ScrollbarAxis::Horizontal);
                let offset = handle.offset();
                let viewport = handle.viewport_size();
                let content = handle.content_size();
                let has_overflow = if is_horizontal {
                    (content.width.0 - viewport.width.0).max(0.0) > 0.0
                } else {
                    (content.height.0 - viewport.height.0).max(0.0) > 0.0
                };

                if !has_overflow {
                    if cx.tree.debug_enabled() {
                        cx.tree.debug_record_scrollbar_telemetry(
                            crate::tree::UiDebugScrollbarTelemetry {
                                node: cx.node,
                                element: Some(self.element),
                                axis: if is_horizontal {
                                    crate::tree::UiDebugScrollAxis::X
                                } else {
                                    crate::tree::UiDebugScrollAxis::Y
                                },
                                scroll_target: props.scroll_target,
                                offset,
                                viewport,
                                content,
                                track: cx.bounds,
                                thumb: None,
                                hovered,
                                dragging,
                            },
                        );
                    }
                    return;
                }

                let thumb = if is_horizontal {
                    scrollbar_thumb_rect_horizontal(
                        cx.bounds,
                        viewport.width,
                        content.width,
                        offset.x,
                        props.style.track_padding,
                    )
                } else {
                    scrollbar_thumb_rect(
                        cx.bounds,
                        viewport.height,
                        content.height,
                        offset.y,
                        props.style.track_padding,
                    )
                };

                if cx.tree.debug_enabled() {
                    cx.tree.debug_record_scrollbar_telemetry(
                        crate::tree::UiDebugScrollbarTelemetry {
                            node: cx.node,
                            element: Some(self.element),
                            axis: if is_horizontal {
                                crate::tree::UiDebugScrollAxis::X
                            } else {
                                crate::tree::UiDebugScrollAxis::Y
                            },
                            scroll_target: props.scroll_target,
                            offset,
                            viewport,
                            content,
                            track: cx.bounds,
                            thumb,
                            hovered,
                            dragging,
                        },
                    );
                }

                let Some(thumb) = thumb else {
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

                let rect = if is_horizontal {
                    let inset = 1.0f32.min(thumb.size.height.0 * 0.25);
                    Rect::new(
                        fret_core::Point::new(thumb.origin.x, Px(thumb.origin.y.0 + inset)),
                        Size::new(
                            thumb.size.width,
                            Px((thumb.size.height.0 - inset * 2.0).max(0.0)),
                        ),
                    )
                } else {
                    let inset = 1.0f32.min(thumb.size.width.0 * 0.25);
                    Rect::new(
                        fret_core::Point::new(Px(thumb.origin.x.0 + inset), thumb.origin.y),
                        Size::new(
                            Px((thumb.size.width.0 - inset * 2.0).max(0.0)),
                            thumb.size.height,
                        ),
                    )
                };

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
