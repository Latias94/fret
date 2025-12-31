use super::*;

pub(super) struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
    hit_testable: bool,
    hit_test_children: bool,
    is_focusable: bool,
    is_text_input: bool,
    clips_hit_test: bool,
    clip_hit_test_corner_radii: Option<fret_core::Corners>,
    text_input: Option<BoundTextInput>,
    text_area: Option<crate::text_area::BoundTextArea>,
    resizable_panel_group: Option<crate::resizable_panel_group::BoundResizablePanelGroup>,
    flex_cache: Option<TaffyContainerCache>,
    grid_cache: Option<TaffyContainerCache>,
}

impl ElementHostWidget {
    pub(super) fn new(element: GlobalElementId) -> Self {
        Self {
            element,
            text_cache: TextCache::default(),
            hit_testable: true,
            hit_test_children: true,
            is_focusable: false,
            is_text_input: false,
            clips_hit_test: true,
            clip_hit_test_corner_radii: None,
            text_input: None,
            text_area: None,
            resizable_panel_group: None,
            flex_cache: None,
            grid_cache: None,
        }
    }

    fn instance<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<ElementInstance> {
        element_record_for_node(app, window, node).map(|r| r.instance)
    }

    fn layout_flex_container<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
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
                .compute_layout_with_measure(root, available, |known, avail, _id, ctx, _style| {
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
                })
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
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width - left - right;
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
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
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
                        let free =
                            auto_margin_inner_size.height.0 - layout.size.height - top - bottom;
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
                        let free =
                            auto_margin_inner_size.width.0 - layout.size.width - left - right;
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
}

impl<H: UiHost> Widget<H> for ElementHostWidget {
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        self.clips_hit_test
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<fret_core::Corners> {
        self.clip_hit_test_corner_radii
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        self.hit_testable
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        if !self.hit_test_children {
            return false;
        }
        true
    }

    fn is_focusable(&self) -> bool {
        self.is_focusable
    }

    fn is_text_input(&self) -> bool {
        self.is_text_input
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        let is_text_input = matches!(
            instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
        );

        if let Event::Timer { token } = event {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::TimerActionHooks::default,
                |hooks| hooks.on_timer.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
                    },
                    *token,
                );
                if handled {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        let try_key_hook = |cx: &mut EventCx<'_, H>,
                            key: fret_core::KeyCode,
                            modifiers: fret_core::Modifiers,
                            repeat: bool| {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::KeyActionHooks::default,
                |hooks| hooks.on_key_down.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
                    },
                    KeyDownCx {
                        key,
                        modifiers,
                        repeat,
                    },
                );
                if handled {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return true;
                }
            }
            false
        };

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && cx.focus == Some(cx.node)
            && !is_text_input
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {
            return;
        }

        match instance {
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
                input.event(cx, event);
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
                area.event(cx, event);
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
                group.event(cx, event);
            }
            ElementInstance::VirtualList(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        crate::elements::with_element_state(
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
                                let viewport_h = Px(state.viewport_h.0.max(0.0));

                                let prev = props.scroll_handle.offset();
                                let offset_y = state.metrics.clamp_offset(prev.y, viewport_h);

                                let next = state
                                    .metrics
                                    .clamp_offset(Px(offset_y.0 - delta.y.0), viewport_h);
                                if (prev.y.0 - next.0).abs() > 0.01 {
                                    props
                                        .scroll_handle
                                        .set_offset(fret_core::Point::new(prev.x, next));
                                }
                            },
                        );
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    fret_core::PointerEvent::Down { button, .. } => {
                        if *button == MouseButton::Left {
                            cx.request_focus(cx.node);
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::Scroll(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                if let fret_core::PointerEvent::Wheel { delta, .. } = pe {
                    if let Some(handle) = props.scroll_handle.as_ref() {
                        let prev = handle.offset();
                        handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                    } else {
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                let prev = state.scroll_handle.offset();
                                state
                                    .scroll_handle
                                    .set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                            },
                        );
                    }
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            ElementInstance::Scrollbar(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };

                let handle = props.scroll_handle.clone();
                let scroll_target = props.scroll_target;
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        let prev = handle.offset();
                        handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));

                        if let Some(target) = scroll_target
                            && let Some(node) =
                                node_for_element_in_window_frame(&mut *cx.app, window, target)
                        {
                            cx.invalidate(node, Invalidation::Layout);
                            cx.invalidate(node, Invalidation::Paint);
                        }

                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    fret_core::PointerEvent::Move { position, .. } => {
                        let mut needs_layout = false;
                        let mut needs_paint = false;

                        let bounds = cx.bounds;
                        let position = *position;

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                                let hovered = bounds.contains(position);
                                if state.hovered != hovered && !state.dragging_thumb {
                                    state.hovered = hovered;
                                    needs_paint = true;
                                }

                                if state.dragging_thumb
                                    && max_offset.0 > 0.0
                                    && let Some(thumb) = scrollbar_thumb_rect(
                                        bounds,
                                        viewport_h,
                                        content_h,
                                        state.drag_start_offset_y,
                                    )
                                {
                                    let max_thumb_y =
                                        (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                                    if max_thumb_y > 0.0 {
                                        let delta_y = position.y.0 - state.drag_start_pointer_y.0;
                                        let scale = max_offset.0 / max_thumb_y;
                                        let next = Px((state.drag_start_offset_y.0
                                            + delta_y * scale)
                                            .max(0.0));
                                        let next = Px(next.0.min(max_offset.0));
                                        if (handle.offset().y.0 - next.0).abs() > 0.01 {
                                            let prev = handle.offset();
                                            handle.set_offset(Point::new(prev.x, next));
                                            needs_layout = true;
                                            needs_paint = true;
                                        }
                                        state.hovered = true;
                                    }
                                }
                            },
                        );

                        if needs_layout {
                            cx.invalidate_self(Invalidation::Layout);
                            if let Some(target) = scroll_target
                                && let Some(node) =
                                    node_for_element_in_window_frame(&mut *cx.app, window, target)
                            {
                                cx.invalidate(node, Invalidation::Layout);
                                cx.invalidate(node, Invalidation::Paint);
                            }
                        }
                        if needs_paint {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                    fret_core::PointerEvent::Down {
                        position, button, ..
                    } => {
                        if *button != MouseButton::Left {
                            return;
                        }

                        let bounds = cx.bounds;
                        let position = *position;

                        let mut did_handle = false;
                        let mut did_start_drag = false;
                        let mut did_change_offset = false;
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                                if max_offset.0 <= 0.0 {
                                    return;
                                }

                                let Some(thumb) = scrollbar_thumb_rect(
                                    bounds,
                                    viewport_h,
                                    content_h,
                                    handle.offset().y,
                                ) else {
                                    return;
                                };

                                did_handle = true;
                                state.hovered = true;

                                if thumb.contains(position) {
                                    state.dragging_thumb = true;
                                    state.drag_start_pointer_y = position.y;
                                    state.drag_start_offset_y = handle.offset().y;
                                    did_start_drag = true;
                                } else if bounds.contains(position) {
                                    // Page to the click position (center the thumb on the pointer).
                                    let max_thumb_y =
                                        (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                                    if max_thumb_y > 0.0 {
                                        let click_y = (position.y.0 - bounds.origin.y.0)
                                            .clamp(0.0, bounds.size.height.0);
                                        let thumb_top = (click_y - thumb.size.height.0 * 0.5)
                                            .clamp(0.0, max_thumb_y);
                                        let t = thumb_top / max_thumb_y;
                                        let next = Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                        let prev = handle.offset();
                                        handle.set_offset(Point::new(prev.x, next));
                                        did_change_offset = true;
                                    }
                                } else {
                                    did_handle = false;
                                }
                            },
                        );

                        if did_handle {
                            if did_start_drag {
                                cx.capture_pointer(cx.node);
                            }
                            if did_change_offset
                                && let Some(target) = scroll_target
                                && let Some(node) =
                                    node_for_element_in_window_frame(&mut *cx.app, window, target)
                            {
                                cx.invalidate(node, Invalidation::Layout);
                                cx.invalidate(node, Invalidation::Paint);
                            }
                            cx.invalidate_self(Invalidation::Layout);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    fret_core::PointerEvent::Up { button, .. } => {
                        if *button != MouseButton::Left {
                            return;
                        }

                        let mut did_handle = false;
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                if state.dragging_thumb {
                                    did_handle = true;
                                    state.dragging_thumb = false;
                                }
                            },
                        );
                        if did_handle {
                            cx.release_pointer_capture();
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                }
            }
            ElementInstance::DismissibleLayer(props) => {
                if !props.enabled {
                    return;
                }

                match event {
                    Event::KeyDown {
                        key: fret_core::KeyCode::Escape,
                        repeat: false,
                        ..
                    } => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::Escape,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
                        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Observer
                        {
                            return;
                        }
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::OutsidePress,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::Pressable(props) => {
                if !props.enabled {
                    return;
                }
                match event {
                    Event::Pointer(pe) => match pe {
                        fret_core::PointerEvent::Move { .. } => {
                            cx.set_cursor_icon(CursorIcon::Pointer);
                        }
                        fret_core::PointerEvent::Down { button, .. } => {
                            if *button != MouseButton::Left {
                                return;
                            }
                            if props.focusable {
                                cx.request_focus(cx.node);
                            }
                            cx.capture_pointer(cx.node);
                            crate::elements::set_pressed_pressable(
                                &mut *cx.app,
                                window,
                                Some(self.element),
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                        fret_core::PointerEvent::Up { button, .. } => {
                            if *button != MouseButton::Left {
                                return;
                            }
                            cx.release_pointer_capture();
                            crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                            let hovered = crate::elements::is_hovered_pressable(
                                &mut *cx.app,
                                window,
                                self.element,
                            );

                            if hovered {
                                let hook = crate::elements::with_element_state(
                                    &mut *cx.app,
                                    window,
                                    self.element,
                                    crate::action::PressableActionHooks::default,
                                    |hooks| hooks.on_activate.clone(),
                                );

                                if let Some(h) = hook {
                                    let mut host =
                                        action::UiActionHostAdapter { app: &mut *cx.app };
                                    h(
                                        &mut host,
                                        action::ActionCx {
                                            window,
                                            target: self.element,
                                        },
                                        ActivateReason::Pointer,
                                    );
                                }
                            }
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                        _ => {}
                    },
                    Event::KeyDown { key, repeat, .. } => {
                        if *repeat {
                            return;
                        }
                        if cx.focus != Some(cx.node) {
                            return;
                        }
                        if !matches!(
                            key,
                            fret_core::KeyCode::Enter
                                | fret_core::KeyCode::NumpadEnter
                                | fret_core::KeyCode::Space
                        ) {
                            return;
                        }
                        crate::elements::set_pressed_pressable(
                            &mut *cx.app,
                            window,
                            Some(self.element),
                        );
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    Event::KeyUp { key, .. } => {
                        if cx.focus != Some(cx.node) {
                            return;
                        }
                        if !matches!(
                            key,
                            fret_core::KeyCode::Enter
                                | fret_core::KeyCode::NumpadEnter
                                | fret_core::KeyCode::Space
                        ) {
                            return;
                        }
                        let pressed = crate::elements::is_pressed_pressable(
                            &mut *cx.app,
                            window,
                            self.element,
                        );
                        if !pressed {
                            return;
                        }
                        crate::elements::set_pressed_pressable(&mut *cx.app, window, None);
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PressableActionHooks::default,
                            |hooks| hooks.on_activate.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                ActivateReason::Keyboard,
                            );
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                };
            }
            ElementInstance::PointerRegion(props) => {
                if !props.enabled {
                    return;
                }

                struct PointerHookHost<'a, H: UiHost> {
                    app: &'a mut H,
                    window: AppWindowId,
                    node: NodeId,
                    bounds: Rect,
                    input_ctx: &'a fret_runtime::InputContext,
                    requested_focus: &'a mut Option<NodeId>,
                    requested_capture: &'a mut Option<Option<NodeId>>,
                    requested_cursor: &'a mut Option<fret_core::CursorIcon>,
                }

                impl<H: UiHost> action::UiActionHost for PointerHookHost<'_, H> {
                    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                        self.app.models_mut()
                    }

                    fn push_effect(&mut self, effect: Effect) {
                        self.app.push_effect(effect);
                    }

                    fn request_redraw(&mut self, window: AppWindowId) {
                        self.app.request_redraw(window);
                    }

                    fn next_timer_token(&mut self) -> fret_core::TimerToken {
                        self.app.next_timer_token()
                    }
                }

                impl<H: UiHost> action::UiPointerActionHost for PointerHookHost<'_, H> {
                    fn bounds(&self) -> Rect {
                        self.bounds
                    }

                    fn request_focus(&mut self, target: crate::GlobalElementId) {
                        let Some(node) = crate::elements::with_window_state(
                            &mut *self.app,
                            self.window,
                            |window_state| window_state.node_entry(target).map(|e| e.node),
                        ) else {
                            return;
                        };
                        *self.requested_focus = Some(node);
                    }

                    fn capture_pointer(&mut self) {
                        *self.requested_capture = Some(Some(self.node));
                    }

                    fn release_pointer_capture(&mut self) {
                        *self.requested_capture = Some(None);
                    }

                    fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
                        if !self.input_ctx.caps.ui.cursor_icons {
                            return;
                        }
                        *self.requested_cursor = Some(icon);
                    }
                }

                match event {
                    Event::Pointer(fret_core::PointerEvent::Down {
                        position,
                        button,
                        modifiers,
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_down.clone(),
                        );

                        let Some(h) = hook else {
                            return;
                        };

                        let down = action::PointerDownCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
                        };

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::PointerRegionState::default,
                            |state| {
                                state.last_down = Some(down);
                            },
                        );

                        let mut host = PointerHookHost {
                            app: &mut *cx.app,
                            window,
                            node: cx.node,
                            bounds: cx.bounds,
                            input_ctx: &cx.input_ctx,
                            requested_focus: &mut cx.requested_focus,
                            requested_capture: &mut cx.requested_capture,
                            requested_cursor: &mut cx.requested_cursor,
                        };
                        let handled = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            down,
                        );

                        if handled {
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Move {
                        position,
                        buttons,
                        modifiers,
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_move.clone(),
                        );

                        let Some(h) = hook else {
                            return;
                        };

                        let mv = action::PointerMoveCx {
                            position: *position,
                            buttons: *buttons,
                            modifiers: *modifiers,
                        };

                        let mut host = PointerHookHost {
                            app: &mut *cx.app,
                            window,
                            node: cx.node,
                            bounds: cx.bounds,
                            input_ctx: &cx.input_ctx,
                            requested_focus: &mut cx.requested_focus,
                            requested_capture: &mut cx.requested_capture,
                            requested_cursor: &mut cx.requested_cursor,
                        };
                        let handled = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            mv,
                        );

                        if handled {
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Up {
                        position,
                        button,
                        modifiers,
                    }) => {
                        let was_captured = cx.captured == Some(cx.node);

                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_up.clone(),
                        );

                        let up = action::PointerUpCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
                        };

                        if let Some(h) = hook {
                            let mut host = PointerHookHost {
                                app: &mut *cx.app,
                                window,
                                node: cx.node,
                                bounds: cx.bounds,
                                input_ctx: &cx.input_ctx,
                                requested_focus: &mut cx.requested_focus,
                                requested_capture: &mut cx.requested_capture,
                                requested_cursor: &mut cx.requested_cursor,
                            };
                            let handled = h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                up,
                            );

                            if handled {
                                cx.stop_propagation();
                            }
                        }

                        if was_captured {
                            cx.release_pointer_capture();
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::RovingFlex(props) => {
                if !props.roving.enabled {
                    return;
                }

                let Event::KeyDown { key, repeat, .. } = event else {
                    return;
                };
                if *repeat {
                    return;
                }

                enum Nav {
                    Prev,
                    Next,
                    Home,
                    End,
                }

                let nav = match (props.flex.direction, key) {
                    (_, fret_core::KeyCode::Home) => Some(Nav::Home),
                    (_, fret_core::KeyCode::End) => Some(Nav::End),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowUp) => Some(Nav::Prev),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowDown) => Some(Nav::Next),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowLeft) => Some(Nav::Prev),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowRight) => {
                        Some(Nav::Next)
                    }
                    _ => None,
                };
                let len = cx.children.len();
                if len == 0 {
                    return;
                }

                let current = cx
                    .focus
                    .and_then(|focus| cx.children.iter().position(|n| *n == focus));

                let is_disabled = |idx: usize| -> bool {
                    props.roving.disabled.get(idx).copied().unwrap_or(false)
                };

                let mut target: Option<usize> = None;
                match nav {
                    Some(Nav::Home) => {
                        target = (0..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::End) => {
                        target = (0..len).rev().find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Next) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + step) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Prev) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + len - (step % len)) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Next) => {
                        let Some(current) = current else {
                            return;
                        };
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Prev) => {
                        let Some(current) = current else {
                            return;
                        };
                        if current > 0 {
                            target = (0..current).rev().find(|&i| !is_disabled(i));
                        }
                    }
                    None => {}
                }

                let key_to_ascii = |key: fret_core::KeyCode| -> Option<char> {
                    use fret_core::KeyCode;
                    Some(match key {
                        KeyCode::KeyA => 'a',
                        KeyCode::KeyB => 'b',
                        KeyCode::KeyC => 'c',
                        KeyCode::KeyD => 'd',
                        KeyCode::KeyE => 'e',
                        KeyCode::KeyF => 'f',
                        KeyCode::KeyG => 'g',
                        KeyCode::KeyH => 'h',
                        KeyCode::KeyI => 'i',
                        KeyCode::KeyJ => 'j',
                        KeyCode::KeyK => 'k',
                        KeyCode::KeyL => 'l',
                        KeyCode::KeyM => 'm',
                        KeyCode::KeyN => 'n',
                        KeyCode::KeyO => 'o',
                        KeyCode::KeyP => 'p',
                        KeyCode::KeyQ => 'q',
                        KeyCode::KeyR => 'r',
                        KeyCode::KeyS => 's',
                        KeyCode::KeyT => 't',
                        KeyCode::KeyU => 'u',
                        KeyCode::KeyV => 'v',
                        KeyCode::KeyW => 'w',
                        KeyCode::KeyX => 'x',
                        KeyCode::KeyY => 'y',
                        KeyCode::KeyZ => 'z',
                        KeyCode::Digit0 => '0',
                        KeyCode::Digit1 => '1',
                        KeyCode::Digit2 => '2',
                        KeyCode::Digit3 => '3',
                        KeyCode::Digit4 => '4',
                        KeyCode::Digit5 => '5',
                        KeyCode::Digit6 => '6',
                        KeyCode::Digit7 => '7',
                        KeyCode::Digit8 => '8',
                        KeyCode::Digit9 => '9',
                        _ => return None,
                    })
                };

                if target.is_none()
                    && let Some(ch) = key_to_ascii(*key)
                {
                    let hook = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::action::RovingActionHooks::default,
                        |hooks| hooks.on_typeahead.clone(),
                    );

                    if let Some(h) = hook {
                        let tick = cx.app.tick_id().0;
                        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                        target = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            crate::action::RovingTypeaheadCx {
                                input: ch,
                                current,
                                len,
                                disabled: props.roving.disabled.clone(),
                                wrap: props.roving.wrap,
                                tick,
                            },
                        );
                    }
                }

                let Some(target) = target else {
                    return;
                };
                if current.is_some_and(|current| target == current) {
                    return;
                }

                cx.request_focus(cx.children[target]);

                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::action::RovingActionHooks::default,
                    |hooks| hooks.on_active_change.clone(),
                );

                if let Some(h) = hook {
                    let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                    h(
                        &mut host,
                        action::ActionCx {
                            window,
                            target: self.element,
                        },
                        target,
                    );
                }

                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }

        if is_text_input
            && !cx.stop_propagation
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
            && cx.focus == Some(cx.node)
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {}
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(blob) = self.text_cache.blob.take() {
            services.text().release(blob);
        }
        self.text_cache.prepared_scale_factor_bits = None;
        self.text_cache.metrics = None;
        if let Some(input) = self.text_input.as_mut() {
            input.cleanup_resources(services);
        }
        if let Some(area) = self.text_area.as_mut() {
            area.cleanup_resources(services);
        }
        if let Some(group) = self.resizable_panel_group.as_mut() {
            group.cleanup_resources(services);
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };
        match instance {
            ElementInstance::Text(props) => {
                cx.set_role(SemanticsRole::Text);
                cx.set_label(props.text.as_ref().to_string());
            }
            ElementInstance::Semantics(props) => {
                cx.set_role(props.role);
                if let Some(label) = props.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(value) = props.value.as_ref() {
                    cx.set_value(value.as_ref().to_string());
                }
                if props.disabled {
                    cx.set_disabled(true);
                }
                if props.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.expanded {
                    cx.set_expanded(expanded);
                }
                if props.checked.is_some() {
                    cx.set_checked(props.checked);
                }
                if props.active_descendant.is_some() {
                    cx.set_active_descendant(props.active_descendant);
                }
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
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                cx.set_active_descendant(props.active_descendant);
                input.semantics(cx);
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
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                area.semantics(cx);
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
                group.semantics(cx);
            }
            ElementInstance::Pressable(props) => {
                cx.set_role(props.a11y.role.unwrap_or(SemanticsRole::Button));
                if let Some(label) = props.a11y.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if props.a11y.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.a11y.expanded {
                    cx.set_expanded(expanded);
                }
                if props.a11y.checked.is_some() {
                    cx.set_checked(props.a11y.checked);
                }
                cx.set_disabled(!props.enabled);
                cx.set_focusable(props.enabled);
                cx.set_invokable(props.enabled);
                cx.set_collection_position(props.a11y.pos_in_set, props.a11y.set_size);
            }
            ElementInstance::VirtualList(_) => {
                cx.set_role(SemanticsRole::List);
            }
            ElementInstance::Flex(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::RovingFlex(_)
            | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::Scroll(_) => {
                cx.set_role(SemanticsRole::Generic);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
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

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
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
