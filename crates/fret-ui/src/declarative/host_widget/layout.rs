use super::*;

impl ElementHostWidget {
    pub(super) fn layout_impl<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return Size::new(Px(0.0), Px(0.0));
        };

        crate::elements::record_bounds_for_element(&mut *cx.app, window, self.element, cx.bounds);

        for (model, invalidation) in
            crate::elements::observed_models_for_element(cx.app, window, self.element)
        {
            (cx.observe_model)(model, invalidation);
        }

        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return Size::new(Px(0.0), Px(0.0));
        };

        self.hit_testable = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::PointerRegion(p) => p.enabled,
            ElementInstance::Semantics(_) => false,
            ElementInstance::DismissibleLayer(_) => false,
            ElementInstance::Opacity(_) => false,
            ElementInstance::VisualTransform(_) => false,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.hit_test_children = match &instance {
            ElementInstance::Pressable(p) => p.enabled,
            ElementInstance::PointerRegion(_) => true,
            ElementInstance::Semantics(_) => true,
            ElementInstance::DismissibleLayer(_) => true,
            ElementInstance::VisualTransform(_) => true,
            ElementInstance::Spinner(_) => false,
            _ => true,
        };
        self.is_text_input = matches!(
            &instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
        );
        self.is_focusable = match &instance {
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_) => true,
            ElementInstance::Pressable(p) => p.enabled && p.focusable,
            _ => false,
        };
        self.clips_hit_test = match &instance {
            ElementInstance::Container(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Semantics(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Opacity(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::VisualTransform(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Pressable(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::PointerRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::DismissibleLayer(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Stack(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Flex(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::RovingFlex(p) => matches!(p.flex.layout.overflow, Overflow::Clip),
            ElementInstance::Grid(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextInput(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::TextArea(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::ResizablePanelGroup(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::Scroll(p) => matches!(p.layout.overflow, Overflow::Clip),
            ElementInstance::HoverRegion(p) => matches!(p.layout.overflow, Overflow::Clip),
            // These primitives are always hit-test clipped by their own bounds (they are not
            // intended as overflow-visible containers).
            ElementInstance::VirtualList(_)
            | ElementInstance::Scrollbar(_)
            | ElementInstance::Image(_)
            | ElementInstance::SvgIcon(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Text(_) => true,
            ElementInstance::Spacer(_) => true,
        };
        self.clip_hit_test_corner_radii = match &instance {
            ElementInstance::Container(p) if matches!(p.layout.overflow, Overflow::Clip) => {
                if p.corner_radii.top_left.0 > 0.0
                    || p.corner_radii.top_right.0 > 0.0
                    || p.corner_radii.bottom_right.0 > 0.0
                    || p.corner_radii.bottom_left.0 > 0.0
                {
                    Some(p.corner_radii)
                } else {
                    None
                }
            }
            _ => None,
        };

        let is_flex = matches!(&instance, ElementInstance::Flex(_));
        let is_roving_flex = matches!(&instance, ElementInstance::RovingFlex(_));
        let is_grid = matches!(&instance, ElementInstance::Grid(_));
        if !is_flex && !is_roving_flex {
            self.flex_cache = None;
        }
        if !is_grid {
            self.grid_cache = None;
        }

        match instance {
            ElementInstance::Container(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;

                let inner_avail = Size::new(
                    Px((cx.available.width.0 - pad_w).max(0.0)),
                    Px((cx.available.height.0 - pad_h).max(0.0)),
                );

                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, inner_avail);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = Size::new(
                    Px((max_child.width.0 + pad_w).max(0.0)),
                    Px((max_child.height.0 + pad_h).max(0.0)),
                );
                let desired = clamp_to_constraints(desired, props.layout, cx.available);

                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );
                let inner_size = Size::new(
                    Px((desired.width.0 - pad_w).max(0.0)),
                    Px((desired.height.0 - pad_h).max(0.0)),
                );
                let inner_bounds = Rect::new(inner_origin, inner_size);

                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(
                        cx,
                        child,
                        inner_bounds,
                        positioned_layout_style(layout_style),
                    );
                }

                desired
            }
            ElementInstance::Pressable(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Semantics(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Opacity(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::VisualTransform(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::DismissibleLayer(props) => {
                let desired = clamp_to_constraints(cx.available, props.layout, cx.available);
                let base = cx.bounds;
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Stack(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_available =
                    clamp_to_constraints(cx.available, props.layout, cx.available);
                let probe_bounds = Rect::new(cx.bounds.origin, probe_available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Spacer(props) => {
                clamp_to_constraints(Size::new(Px(0.0), Px(0.0)), props.layout, cx.available)
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
                let mut measure_width = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                if let Some(max_w) = props.layout.size.max_width {
                    measure_width = Px(measure_width.0.min(max_w.0.max(0.0)));
                }
                measure_width = Px(measure_width.0.max(0.0).min(cx.available.width.0.max(0.0)));
                let constraints = TextConstraints {
                    max_width: Some(measure_width),
                    wrap: props.wrap,
                    overflow: props.overflow,
                    scale_factor: cx.scale_factor,
                };
                let metrics = cx.services.text().measure(&props.text, style, constraints);

                self.text_cache.metrics = Some(metrics);
                self.text_cache.last_text = Some(props.text.clone());
                self.text_cache.last_style = Some(style);
                self.text_cache.last_wrap = Some(props.wrap);
                self.text_cache.last_overflow = Some(props.overflow);
                self.text_cache.last_width = Some(measure_width);
                self.text_cache.last_theme_revision = Some(theme_revision);

                clamp_to_constraints(metrics.size, props.layout, cx.available)
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

                let desired = input.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
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

                let desired = area.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
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

                let desired = group.layout(cx);
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::VirtualList(props) => {
                let mut metrics = crate::elements::with_element_state(
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
                let content_h = metrics.total_height();

                let desired_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let desired_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.max(0.0)),
                    Length::Fill => cx.available.height,
                    Length::Auto => Px(content_h.0.min(cx.available.height.0.max(0.0))),
                };

                let size = clamp_to_constraints(
                    Size::new(desired_w, desired_h),
                    props.layout,
                    cx.available,
                );
                let viewport_h = Px(size.height.0.max(0.0));
                let mut needs_redraw = false;

                props.scroll_handle.set_items_count(props.len);

                let prev_offset = props.scroll_handle.offset();
                let mut offset_y = metrics.clamp_offset(prev_offset.y, viewport_h);

                // Avoid consuming deferred scroll requests during "probe" layout passes that use
                // an effectively-unbounded available height (e.g. Stack/Pressable measuring with
                // `Px(1.0e9)`). Those passes are not the final viewport constraints and would
                // otherwise clear the request before the real layout happens.
                let is_probe_layout = cx.available.height.0 >= 1.0e8;

                if !is_probe_layout
                    && viewport_h.0 > 0.0
                    && props.len > 0
                    && let Some((index, strategy)) = props.scroll_handle.deferred_scroll_to_item()
                {
                    offset_y =
                        metrics.scroll_offset_for_item(index, viewport_h, offset_y, strategy);
                    props.scroll_handle.clear_deferred_scroll_to_item();
                }

                offset_y = metrics.clamp_offset(offset_y, viewport_h);

                if (prev_offset.y.0 - offset_y.0).abs() > 0.01 {
                    needs_redraw = true;
                }
                props
                    .scroll_handle
                    .set_offset(fret_core::Point::new(prev_offset.x, offset_y));

                props
                    .scroll_handle
                    .set_viewport_size(Size::new(size.width, size.height));
                props
                    .scroll_handle
                    .set_content_size(Size::new(size.width, content_h));

                let mut measured_updates: Vec<(usize, crate::ItemKey, Px)> =
                    Vec::with_capacity(cx.children.len());

                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let key = item.key;
                    let y = cx.bounds.origin.y.0 + metrics.offset_for_index(idx).0 - offset_y.0;
                    let origin = fret_core::Point::new(cx.bounds.origin.x, Px(y));

                    let measure_bounds = Rect::new(origin, Size::new(size.width, Px(1.0e9)));
                    let measured = cx.layout_in(child, measure_bounds);
                    let measured_h = Px(measured.height.0.max(0.0));

                    measured_updates.push((idx, key, measured_h));
                    if metrics.set_measured_height(idx, measured_h) {
                        needs_redraw = true;
                    }

                    let child_bounds = Rect::new(origin, Size::new(size.width, measured_h));
                    let _ = cx.layout_in(child, child_bounds);
                }

                let content_h = metrics.total_height();
                props
                    .scroll_handle
                    .set_viewport_size(Size::new(size.width, viewport_h));
                props
                    .scroll_handle
                    .set_content_size(Size::new(size.width, content_h));

                let prev_offset = props.scroll_handle.offset();
                let clamped = metrics.clamp_offset(prev_offset.y, viewport_h);
                if (clamped.0 - prev_offset.y.0).abs() > 0.01 {
                    needs_redraw = true;
                }
                props
                    .scroll_handle
                    .set_offset(fret_core::Point::new(prev_offset.x, clamped));
                offset_y = clamped;

                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::VirtualListState::default,
                    |state| {
                        for (idx, key, h) in &measured_updates {
                            state.size_cache.insert(*key, *h);
                            if let Some(slot) = state.keys.get_mut(*idx) {
                                *slot = *key;
                            }
                        }
                        state.offset_y = offset_y;
                        if state.viewport_h != viewport_h {
                            state.viewport_h = viewport_h;
                            needs_redraw = true;
                        }
                        state.items_revision = props.items_revision;
                        state.metrics = metrics;
                    },
                );

                if needs_redraw && let Some(window) = cx.window {
                    cx.app.request_redraw(window);
                }

                size
            }
            ElementInstance::Flex(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );
                let outer_avail_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let outer_avail_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.height,
                };

                let inner_avail = Size::new(
                    Px((outer_avail_w.0 - pad_w).max(0.0)),
                    Px((outer_avail_h.0 - pad_h).max(0.0)),
                );

                let root_style = TaffyStyle {
                    display: Display::Flex,
                    flex_direction: match props.direction {
                        fret_core::Axis::Horizontal => FlexDirection::Row,
                        fret_core::Axis::Vertical => FlexDirection::Column,
                    },
                    flex_wrap: if props.wrap {
                        FlexWrap::Wrap
                    } else {
                        FlexWrap::NoWrap
                    },
                    justify_content: Some(taffy_justify(props.justify)),
                    align_items: Some(taffy_align_items(props.align)),
                    gap: TaffySize {
                        width: LengthPercentage::length(props.gap.0.max(0.0)),
                        height: LengthPercentage::length(props.gap.0.max(0.0)),
                    },
                    size: TaffySize {
                        width: match props.layout.size.width {
                            Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                    },
                    max_size: TaffySize {
                        width: Dimension::length(inner_avail.width.0.max(0.0)),
                        height: Dimension::length(inner_avail.height.0.max(0.0)),
                    },
                    ..Default::default()
                };

                let cache = self
                    .flex_cache
                    .get_or_insert_with(TaffyContainerCache::default);

                cache.sync_root_style(root_style);
                cache.sync_children(cx.children, |child| {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    let spacer_min = element_record_for_node(cx.app, window, child).and_then(|r| {
                        if let ElementInstance::Spacer(p) = r.instance {
                            Some(p.min)
                        } else {
                            None
                        }
                    });

                    let mut min_w = layout_style.size.min_width.map(|p| p.0);
                    let mut min_h = layout_style.size.min_height.map(|p| p.0);
                    if let Some(min) = spacer_min {
                        let min = min.0.max(0.0);
                        match props.direction {
                            fret_core::Axis::Horizontal => {
                                min_w = Some(min_w.unwrap_or(0.0).max(min));
                            }
                            fret_core::Axis::Vertical => {
                                min_h = Some(min_h.unwrap_or(0.0).max(min));
                            }
                        }
                    }

                    TaffyStyle {
                        display: Display::Block,
                        position: taffy_position(layout_style.position),
                        inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
                        size: TaffySize {
                            width: taffy_dimension(layout_style.size.width),
                            height: taffy_dimension(layout_style.size.height),
                        },
                        aspect_ratio: layout_style.aspect_ratio,
                        min_size: TaffySize {
                            width: min_w.map(Dimension::length).unwrap_or_else(Dimension::auto),
                            height: min_h.map(Dimension::length).unwrap_or_else(Dimension::auto),
                        },
                        max_size: TaffySize {
                            width: layout_style
                                .size
                                .max_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .max_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                        flex_grow: layout_style.flex.grow.max(0.0),
                        flex_shrink: layout_style.flex.shrink.max(0.0),
                        flex_basis: taffy_dimension(layout_style.flex.basis),
                        align_self: layout_style.flex.align_self.map(taffy_align_self),
                        ..Default::default()
                    }
                });

                cache
                    .taffy
                    .mark_dirty(cache.root)
                    .expect("taffy mark dirty");

                cache.measure_cache.clear();
                let root = cache.root;

                {
                    let measure_cache = &mut cache.measure_cache;
                    let taffy = &mut cache.taffy;

                    let available = taffy::geometry::Size {
                        width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                        height: TaffyAvailableSpace::Definite(inner_avail.height.0),
                    };

                    taffy
                        .compute_layout_with_measure(
                            root,
                            available,
                            |known, avail, _id, ctx, _style| {
                                let Some(child) = ctx.and_then(|c| *c) else {
                                    return taffy::geometry::Size::default();
                                };

                                let key = TaffyMeasureKey {
                                    child,
                                    known_w: known.width.map(|v| v.to_bits()),
                                    known_h: known.height.map(|v| v.to_bits()),
                                    avail_w: taffy_available_space_key(avail.width),
                                    avail_h: taffy_available_space_key(avail.height),
                                };
                                if let Some(size) = measure_cache.get(&key) {
                                    return *size;
                                }

                                let max_w = match avail.width {
                                    TaffyAvailableSpace::Definite(w) => Px(w),
                                    _ => Px(1.0e9),
                                };
                                let max_h = match avail.height {
                                    TaffyAvailableSpace::Definite(h) => Px(h),
                                    _ => Px(1.0e9),
                                };

                                let known_w = known.width.map(Px);
                                let known_h = known.height.map(Px);

                                let w = known_w.unwrap_or(max_w);
                                let h = known_h.unwrap_or(max_h);

                                let probe = Rect::new(inner_origin, Size::new(w, h));
                                let s = cx.layout_in(child, probe);
                                let out = taffy::geometry::Size {
                                    width: s.width.0,
                                    height: s.height.0,
                                };
                                measure_cache.insert(key, out);
                                out
                            },
                        )
                        .expect("taffy compute");
                }

                let taffy = &cache.taffy;
                let root_layout = taffy.layout(root).expect("taffy root layout");
                let container_inner_size = Size::new(
                    Px(root_layout.size.width.max(0.0)),
                    Px(root_layout.size.height.max(0.0)),
                );
                let auto_margin_inner_size = Size::new(
                    match props.layout.size.width {
                        Length::Fill => inner_avail.width,
                        _ => container_inner_size.width,
                    },
                    match props.layout.size.height {
                        Length::Fill => inner_avail.height,
                        _ => container_inner_size.height,
                    },
                );

                for &child_node in &cache.child_nodes {
                    let layout = taffy.layout(child_node).expect("taffy layout");
                    let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                        continue;
                    };
                    let child_style = layout_style_for_node(cx.app, window, child);
                    let single_child = cx.children.len() == 1;

                    let mut x = layout.location.x;
                    let mut y = layout.location.y;

                    let margin_left_auto =
                        matches!(child_style.margin.left, crate::element::MarginEdge::Auto);
                    let margin_right_auto =
                        matches!(child_style.margin.right, crate::element::MarginEdge::Auto);
                    let margin_top_auto =
                        matches!(child_style.margin.top, crate::element::MarginEdge::Auto);
                    let margin_bottom_auto =
                        matches!(child_style.margin.bottom, crate::element::MarginEdge::Auto);

                    let margin_px = |edge: crate::element::MarginEdge| match edge {
                        crate::element::MarginEdge::Px(px) => px.0,
                        crate::element::MarginEdge::Auto => 0.0,
                    };

                    match props.direction {
                        fret_core::Axis::Horizontal => {
                            if single_child && (margin_left_auto || margin_right_auto) {
                                let left = if margin_left_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.left)
                                };
                                let right = if margin_right_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.right)
                                };
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width
                                    - left
                                    - right;
                                if margin_left_auto && margin_right_auto {
                                    x = (left + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_left_auto {
                                    x = (left + free.max(0.0)).max(0.0);
                                } else if margin_right_auto {
                                    x = left.max(0.0);
                                }
                            }

                            if margin_top_auto || margin_bottom_auto {
                                let top = if margin_top_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.top)
                                };
                                let bottom = if margin_bottom_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.bottom)
                                };
                                let free = auto_margin_inner_size.height.0
                                    - layout.size.height
                                    - top
                                    - bottom;
                                if margin_top_auto && margin_bottom_auto {
                                    y = (top + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_top_auto {
                                    y = (top + free.max(0.0)).max(0.0);
                                } else if margin_bottom_auto {
                                    y = top.max(0.0);
                                }
                            }
                        }
                        fret_core::Axis::Vertical => {
                            if single_child && (margin_top_auto || margin_bottom_auto) {
                                let top = if margin_top_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.top)
                                };
                                let bottom = if margin_bottom_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.bottom)
                                };
                                let free = auto_margin_inner_size.height.0
                                    - layout.size.height
                                    - top
                                    - bottom;
                                if margin_top_auto && margin_bottom_auto {
                                    y = (top + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_top_auto {
                                    y = (top + free.max(0.0)).max(0.0);
                                } else if margin_bottom_auto {
                                    y = top.max(0.0);
                                }
                            }

                            if margin_left_auto || margin_right_auto {
                                let left = if margin_left_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.left)
                                };
                                let right = if margin_right_auto {
                                    0.0
                                } else {
                                    margin_px(child_style.margin.right)
                                };
                                let free = auto_margin_inner_size.width.0
                                    - layout.size.width
                                    - left
                                    - right;
                                if margin_left_auto && margin_right_auto {
                                    x = (left + (free.max(0.0) / 2.0)).max(0.0);
                                } else if margin_left_auto {
                                    x = (left + free.max(0.0)).max(0.0);
                                } else if margin_right_auto {
                                    x = left.max(0.0);
                                }
                            }
                        }
                    }
                    let rect = Rect::new(
                        fret_core::Point::new(Px(inner_origin.x.0 + x), Px(inner_origin.y.0 + y)),
                        Size::new(Px(layout.size.width), Px(layout.size.height)),
                    );
                    let _ = cx.layout_in(child, rect);
                }

                let desired = Size::new(
                    Px((container_inner_size.width.0 + pad_w).max(0.0)),
                    Px((container_inner_size.height.0 + pad_h).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::RovingFlex(props) => {
                self.layout_flex_container(cx, window, props.flex)
            }
            ElementInstance::Grid(props) => {
                let pad_left = props.padding.left.0.max(0.0);
                let pad_right = props.padding.right.0.max(0.0);
                let pad_top = props.padding.top.0.max(0.0);
                let pad_bottom = props.padding.bottom.0.max(0.0);
                let pad_w = pad_left + pad_right;
                let pad_h = pad_top + pad_bottom;
                let inner_origin = fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + pad_left),
                    Px(cx.bounds.origin.y.0 + pad_top),
                );

                let outer_avail_w = match props.layout.size.width {
                    Length::Px(px) => Px(px.0.min(cx.available.width.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.width,
                };
                let outer_avail_h = match props.layout.size.height {
                    Length::Px(px) => Px(px.0.min(cx.available.height.0.max(0.0))),
                    Length::Fill | Length::Auto => cx.available.height,
                };

                let inner_avail = Size::new(
                    Px((outer_avail_w.0 - pad_w).max(0.0)),
                    Px((outer_avail_h.0 - pad_h).max(0.0)),
                );

                let root_style = TaffyStyle {
                    display: Display::Grid,
                    justify_content: Some(taffy_justify(props.justify)),
                    align_items: Some(taffy_align_items(props.align)),
                    gap: TaffySize {
                        width: LengthPercentage::length(props.gap.0.max(0.0)),
                        height: LengthPercentage::length(props.gap.0.max(0.0)),
                    },
                    size: TaffySize {
                        width: match props.layout.size.width {
                            Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.width.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                        height: match props.layout.size.height {
                            Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                            Length::Fill => Dimension::length(inner_avail.height.0.max(0.0)),
                            Length::Auto => Dimension::auto(),
                        },
                    },
                    max_size: TaffySize {
                        width: Dimension::length(inner_avail.width.0.max(0.0)),
                        height: Dimension::length(inner_avail.height.0.max(0.0)),
                    },
                    grid_template_columns: taffy::style_helpers::evenly_sized_tracks(props.cols),
                    grid_template_rows: props
                        .rows
                        .map(taffy::style_helpers::evenly_sized_tracks)
                        .unwrap_or_default(),
                    ..Default::default()
                };

                let cache = self
                    .grid_cache
                    .get_or_insert_with(TaffyContainerCache::default);

                cache.sync_root_style(root_style);
                cache.sync_children(cx.children, |child| {
                    let layout_style = layout_style_for_node(cx.app, window, child);

                    TaffyStyle {
                        display: Display::Block,
                        position: taffy_position(layout_style.position),
                        inset: taffy_rect_lpa_from_inset(layout_style.position, layout_style.inset),
                        size: TaffySize {
                            width: taffy_dimension(layout_style.size.width),
                            height: taffy_dimension(layout_style.size.height),
                        },
                        aspect_ratio: layout_style.aspect_ratio,
                        min_size: TaffySize {
                            width: layout_style
                                .size
                                .min_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .min_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        max_size: TaffySize {
                            width: layout_style
                                .size
                                .max_width
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                            height: layout_style
                                .size
                                .max_height
                                .map(|p| Dimension::length(p.0))
                                .unwrap_or_else(Dimension::auto),
                        },
                        margin: taffy_rect_lpa_from_margin_edges(layout_style.margin),
                        grid_column: taffy_grid_line(layout_style.grid.column),
                        grid_row: taffy_grid_line(layout_style.grid.row),
                        ..Default::default()
                    }
                });

                cache
                    .taffy
                    .mark_dirty(cache.root)
                    .expect("taffy mark dirty");

                cache.measure_cache.clear();
                let root = cache.root;

                {
                    let measure_cache = &mut cache.measure_cache;
                    let taffy = &mut cache.taffy;

                    let available = taffy::geometry::Size {
                        width: TaffyAvailableSpace::Definite(inner_avail.width.0),
                        height: TaffyAvailableSpace::Definite(inner_avail.height.0),
                    };

                    taffy
                        .compute_layout_with_measure(
                            root,
                            available,
                            |known, avail, _id, ctx, _style| {
                                let Some(child) = ctx.and_then(|c| *c) else {
                                    return taffy::geometry::Size::default();
                                };

                                let key = TaffyMeasureKey {
                                    child,
                                    known_w: known.width.map(|v| v.to_bits()),
                                    known_h: known.height.map(|v| v.to_bits()),
                                    avail_w: taffy_available_space_key(avail.width),
                                    avail_h: taffy_available_space_key(avail.height),
                                };
                                if let Some(size) = measure_cache.get(&key) {
                                    return *size;
                                }

                                let max_w = match avail.width {
                                    TaffyAvailableSpace::Definite(w) => Px(w),
                                    _ => Px(1.0e9),
                                };
                                let max_h = match avail.height {
                                    TaffyAvailableSpace::Definite(h) => Px(h),
                                    _ => Px(1.0e9),
                                };

                                let known_w = known.width.map(Px);
                                let known_h = known.height.map(Px);

                                let w = known_w.unwrap_or(max_w);
                                let h = known_h.unwrap_or(max_h);

                                let probe = Rect::new(inner_origin, Size::new(w, h));
                                let s = cx.layout_in(child, probe);
                                let out = taffy::geometry::Size {
                                    width: s.width.0,
                                    height: s.height.0,
                                };
                                measure_cache.insert(key, out);
                                out
                            },
                        )
                        .expect("taffy compute");
                }

                let taffy = &cache.taffy;

                for &child_node in &cache.child_nodes {
                    let layout = taffy.layout(child_node).expect("taffy layout");
                    let Some(child) = taffy.get_node_context(child_node).and_then(|c| *c) else {
                        continue;
                    };
                    let rect = Rect::new(
                        fret_core::Point::new(
                            Px(inner_origin.x.0 + layout.location.x),
                            Px(inner_origin.y.0 + layout.location.y),
                        ),
                        Size::new(Px(layout.size.width), Px(layout.size.height)),
                    );
                    let _ = cx.layout_in(child, rect);
                }

                let layout = taffy.layout(root).expect("taffy root layout");
                let inner_size = Size::new(
                    Px(layout.size.width.max(0.0)),
                    Px(layout.size.height.max(0.0)),
                );

                let desired = Size::new(
                    Px((inner_size.width.0 + pad_w).max(0.0)),
                    Px((inner_size.height.0 + pad_h).max(0.0)),
                );
                clamp_to_constraints(desired, props.layout, cx.available)
            }
            ElementInstance::Image(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::SvgIcon(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
            ElementInstance::Spinner(props) => {
                clamp_to_constraints(Size::new(Px(16.0), Px(16.0)), props.layout, cx.available)
            }
            ElementInstance::PointerRegion(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::HoverRegion(props) => {
                // Probe within the available height budget so measurement passes do not observe an
                // artificially "infinite" viewport (important for scroll/virtualized children).
                let probe_bounds = Rect::new(cx.bounds.origin, cx.available);
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    if layout_style.position == crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let base = Rect::new(cx.bounds.origin, desired);
                for &child in cx.children {
                    let layout_style = layout_style_for_node(cx.app, window, child);
                    layout_positioned_child(cx, child, base, positioned_layout_style(layout_style));
                }
                desired
            }
            ElementInstance::Scroll(props) => {
                let probe_bounds =
                    Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9)));

                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    let child_size = cx.layout_in(child, probe_bounds);
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }

                let desired = clamp_to_constraints(max_child, props.layout, cx.available);
                let content_h = Px(max_child.height.0.max(0.0));

                // Avoid mutating the imperative handle during "probe" layout passes that use an
                // effectively-unbounded available height (e.g. Stack/Pressable measuring with
                // `Px(1.0e9)`), otherwise scroll position can be clamped to zero prematurely.
                let is_probe_layout = cx.available.height.0 >= 1.0e8;
                let external_handle = props.scroll_handle.clone();
                let offset_y = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        let handle = external_handle.as_ref().unwrap_or(&state.scroll_handle);
                        if !is_probe_layout {
                            handle.set_viewport_size(desired);
                            handle.set_content_size(Size::new(max_child.width, content_h));
                            let prev = handle.offset();
                            handle.set_offset(prev);
                        }
                        handle.offset().y
                    },
                );

                let shifted = Rect::new(
                    fret_core::Point::new(
                        cx.bounds.origin.x,
                        Px(cx.bounds.origin.y.0 - offset_y.0),
                    ),
                    Size::new(desired.width, content_h),
                );
                for &child in cx.children {
                    let _ = cx.layout_in(child, shifted);
                }

                desired
            }
            ElementInstance::Scrollbar(props) => {
                clamp_to_constraints(cx.available, props.layout, cx.available)
            }
        }
    }
}
