use super::super::frame::{ElementInstance, element_record_for_node, layout_style_for_node};
use super::super::prelude::*;
use super::ElementHostWidget;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use crate::widget::MeasureCx;
use fret_core::{FrameId, TextWrap};

fn available_px_or_zero(constraints: LayoutConstraints) -> Size {
    let w = constraints
        .known
        .width
        .or_else(|| constraints.available.width.definite())
        .unwrap_or(Px(0.0));
    let h = constraints
        .known
        .height
        .or_else(|| constraints.available.height.definite())
        .unwrap_or(Px(0.0));
    Size::new(w, h)
}

fn clamp_to_constraints_in_measure(
    mut size: Size,
    style: LayoutStyle,
    constraints: LayoutConstraints,
) -> Size {
    match style.size.width {
        Length::Px(px) => size.width = Px(px.0.max(0.0)),
        Length::Fill => {
            if let Some(avail) = constraints.available.width.definite() {
                size.width = avail;
            }
        }
        Length::Auto => {}
    }
    match style.size.height {
        Length::Px(px) => size.height = Px(px.0.max(0.0)),
        Length::Fill => {
            if let Some(avail) = constraints.available.height.definite() {
                size.height = avail;
            }
        }
        Length::Auto => {}
    }

    if let Some(min_w) = style.size.min_width {
        size.width = Px(size.width.0.max(min_w.0.max(0.0)));
    }
    if let Some(min_h) = style.size.min_height {
        size.height = Px(size.height.0.max(min_h.0.max(0.0)));
    }
    if let Some(max_w) = style.size.max_width {
        size.width = Px(size.width.0.min(max_w.0.max(0.0)));
    }
    if let Some(max_h) = style.size.max_height {
        size.height = Px(size.height.0.min(max_h.0.max(0.0)));
    }

    if let Some(avail_w) = constraints.available.width.definite() {
        size.width = Px(size.width.0.max(0.0).min(avail_w.0.max(0.0)));
    } else {
        size.width = Px(size.width.0.max(0.0));
    }
    if let Some(avail_h) = constraints.available.height.definite() {
        size.height = Px(size.height.0.max(0.0).min(avail_h.0.max(0.0)));
    } else {
        size.height = Px(size.height.0.max(0.0));
    }

    if let Some(known_w) = constraints.known.width {
        size.width = known_w;
    }
    if let Some(known_h) = constraints.known.height {
        size.height = known_h;
    }

    size
}

fn text_max_width_for_constraints(constraints: LayoutConstraints, wrap: TextWrap) -> Option<Px> {
    if let Some(known_w) = constraints.known.width {
        return Some(known_w);
    }

    match constraints.available.width {
        AvailableSpace::Definite(px) => Some(px),
        AvailableSpace::MaxContent => None,
        AvailableSpace::MinContent => match wrap {
            // Model `TextWrap::Word` as "break when needed" for min-content sizing. This keeps the
            // layout engine honest when it probes intrinsic sizes and later assigns a smaller
            // definite width (e.g. flex/grid shrink), avoiding multi-line paint overflows.
            TextWrap::Word | TextWrap::Grapheme => Some(Px(0.0)),
            TextWrap::None => None,
        },
    }
}

fn max_non_absolute_children<H: UiHost>(
    cx: &mut MeasureCx<'_, H>,
    window: AppWindowId,
    child_constraints: LayoutConstraints,
) -> Size {
    let mut max_child = Size::new(Px(0.0), Px(0.0));
    for &child in cx.children {
        let layout_style = layout_style_for_node(cx.app, window, child);
        if layout_style.position == crate::element::PositionStyle::Absolute {
            continue;
        }
        let child_size = cx.measure_in(child, child_constraints);
        max_child.width = Px(max_child.width.0.max(child_size.width.0));
        max_child.height = Px(max_child.height.0.max(child_size.height.0));
    }
    max_child
}

fn available_space_to_taffy(space: AvailableSpace) -> TaffyAvailableSpace {
    match space {
        AvailableSpace::Definite(px) => TaffyAvailableSpace::Definite(px.0),
        AvailableSpace::MinContent => TaffyAvailableSpace::MinContent,
        AvailableSpace::MaxContent => TaffyAvailableSpace::MaxContent,
    }
}

fn taffy_available_space_to_runtime(space: TaffyAvailableSpace) -> AvailableSpace {
    match space {
        TaffyAvailableSpace::Definite(v) => AvailableSpace::Definite(Px(v)),
        TaffyAvailableSpace::MinContent => AvailableSpace::MinContent,
        TaffyAvailableSpace::MaxContent => AvailableSpace::MaxContent,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollMeasureKey {
    avail_w: u64,
    avail_h: u64,
}

#[derive(Debug, Default, Clone)]
struct ScrollMeasureCacheState {
    frame_id: FrameId,
    entries: Vec<(ScrollMeasureKey, Size)>,
}

fn available_space_cache_key(space: AvailableSpace) -> u64 {
    match space {
        AvailableSpace::Definite(px) => px.0.to_bits() as u64,
        AvailableSpace::MinContent => 1 << 62,
        AvailableSpace::MaxContent => 2 << 62,
    }
}

impl ElementHostWidget {
    pub(super) fn measure_impl<H: UiHost>(&mut self, cx: &mut MeasureCx<'_, H>) -> Size {
        let Some(window) = cx.window else {
            return Size::new(Px(0.0), Px(0.0));
        };

        crate::elements::with_observed_models_for_element(cx.app, window, self.element, |items| {
            for &(model, invalidation) in items {
                (cx.observe_model)(model, invalidation);
            }
        });
        crate::elements::with_observed_globals_for_element(cx.app, window, self.element, |items| {
            for &(global, invalidation) in items {
                (cx.observe_global)(global, invalidation);
            }
        });

        let Some(instance) = element_record_for_node(cx.app, window, cx.node).map(|r| r.instance)
        else {
            return Size::new(Px(0.0), Px(0.0));
        };

        match instance {
            ElementInstance::InteractivityGate(props) if !props.present => {
                // When `present == false`, this subtree is treated like `display: none`. The layout
                // engine may skip calling `layout_impl`, so we must eagerly update the widget-level
                // gates here to avoid stale semantics / hit-test behavior.
                self.hit_testable = false;
                self.hit_test_children = false;
                self.focus_traversal_children = false;
                self.semantics_present = false;
                self.semantics_children = false;
                Size::new(Px(0.0), Px(0.0))
            }
            ElementInstance::Container(props) => self.measure_container(cx, window, props),
            ElementInstance::Pressable(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::Opacity(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::EffectLayer(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::VisualTransform(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::RenderTransform(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::FractionalRenderTransform(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::Semantics(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::SemanticFlex(props) => self.measure_flex(cx, window, props.flex),
            ElementInstance::ViewCache(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::FocusScope(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::InteractivityGate(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::PointerRegion(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::TextInputRegion(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::InternalDragRegion(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::HoverRegion(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::WheelRegion(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::DismissibleLayer(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::Anchored(props) => {
                self.measure_passthrough_box(cx, window, props.layout)
            }
            ElementInstance::Stack(props) => self.measure_passthrough_box(cx, window, props.layout),
            ElementInstance::Spacer(props) => clamp_to_constraints_in_measure(
                Size::new(Px(0.0), Px(0.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Spinner(props) => clamp_to_constraints_in_measure(
                Size::new(Px(16.0), Px(16.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Image(props) => clamp_to_constraints_in_measure(
                Size::new(Px(0.0), Px(0.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Canvas(props) => clamp_to_constraints_in_measure(
                available_px_or_zero(cx.constraints),
                props.layout,
                cx.constraints,
            ),
            #[cfg(feature = "unstable-retained-bridge")]
            ElementInstance::RetainedSubtree(props) => clamp_to_constraints_in_measure(
                available_px_or_zero(cx.constraints),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::ViewportSurface(props) => clamp_to_constraints_in_measure(
                Size::new(Px(0.0), Px(0.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::SvgIcon(props) => clamp_to_constraints_in_measure(
                Size::new(Px(0.0), Px(0.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Scrollbar(props) => clamp_to_constraints_in_measure(
                Size::new(Px(0.0), Px(0.0)),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Scroll(props) => self.measure_scroll(cx, window, props),
            ElementInstance::VirtualList(props) => self.measure_virtual_list(cx, window, props),
            ElementInstance::Text(props) => self.measure_text(cx, props),
            ElementInstance::StyledText(props) => self.measure_styled_text(cx, props),
            ElementInstance::SelectableText(props) => self.measure_selectable_text(cx, props),
            ElementInstance::TextInput(props) => self.measure_text_input(cx, props),
            ElementInstance::TextArea(props) => self.measure_text_area(cx, props),
            ElementInstance::ResizablePanelGroup(props) => clamp_to_constraints_in_measure(
                available_px_or_zero(cx.constraints),
                props.layout,
                cx.constraints,
            ),
            ElementInstance::Flex(props) => self.measure_flex(cx, window, props),
            ElementInstance::RovingFlex(props) => self.measure_flex(cx, window, props.flex),
            ElementInstance::Grid(props) => self.measure_grid(cx, window, props),
        }
    }

    fn measure_passthrough_box<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        layout: LayoutStyle,
    ) -> Size {
        let child_constraints =
            LayoutConstraints::new(LayoutSize::new(None, None), cx.constraints.available);
        let mut max_child = max_non_absolute_children(cx, window, child_constraints);

        // During intrinsic sizing, parents may pass `available = 0` as a placeholder for
        // "unknown". A passthrough box with only absolute-positioned children would otherwise
        // collapse to zero (because absolute children are ignored for sizing), breaking hit
        // testing for overflow-visible overlays.
        let placeholder_width = cx.constraints.known.width.is_none()
            && cx.constraints.available.width.definite() == Some(Px(0.0));
        let placeholder_height = cx.constraints.known.height.is_none()
            && cx.constraints.available.height.definite() == Some(Px(0.0));

        if (placeholder_width || placeholder_height)
            && (max_child.width.0 <= 0.0 || max_child.height.0 <= 0.0)
        {
            let has_absolute_child = cx.children.iter().copied().any(|child| {
                layout_style_for_node(cx.app, window, child).position
                    == crate::element::PositionStyle::Absolute
            });
            if has_absolute_child {
                let mut abs_constraints = child_constraints;
                if placeholder_width {
                    abs_constraints.available.width = AvailableSpace::MaxContent;
                }
                if placeholder_height {
                    abs_constraints.available.height = AvailableSpace::MaxContent;
                }

                for &child in cx.children {
                    let style = layout_style_for_node(cx.app, window, child);
                    if style.position != crate::element::PositionStyle::Absolute {
                        continue;
                    }
                    let child_size = cx.measure_in(child, abs_constraints);
                    let left = style.inset.left.map(|v| v.0);
                    let right = style.inset.right.map(|v| v.0);
                    let top = style.inset.top.map(|v| v.0);
                    let bottom = style.inset.bottom.map(|v| v.0);

                    let required_w = match (left, right) {
                        (Some(l), Some(r)) => Px(l + r + child_size.width.0),
                        (Some(l), None) => Px(l + child_size.width.0),
                        (None, Some(r)) => Px(r + child_size.width.0),
                        (None, None) => child_size.width,
                    };
                    let required_h = match (top, bottom) {
                        (Some(t), Some(b)) => Px(t + b + child_size.height.0),
                        (Some(t), None) => Px(t + child_size.height.0),
                        (None, Some(b)) => Px(b + child_size.height.0),
                        (None, None) => child_size.height,
                    };

                    if placeholder_width {
                        max_child.width = Px(max_child.width.0.max(required_w.0));
                    }
                    if placeholder_height {
                        max_child.height = Px(max_child.height.0.max(required_h.0));
                    }
                }
            }
        }

        let mut clamp_constraints = cx.constraints;
        if placeholder_width {
            clamp_constraints.available.width = AvailableSpace::MaxContent;
        }
        if placeholder_height {
            clamp_constraints.available.height = AvailableSpace::MaxContent;
        }
        clamp_to_constraints_in_measure(max_child, layout, clamp_constraints)
    }

    fn measure_container<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        props: ContainerProps,
    ) -> Size {
        // Tailwind/shadcn assume `box-sizing: border-box` by default. Model borders as part of the
        // container's layout insets so auto-sized containers match web geometry.
        let pad_left = props.padding.left.0.max(0.0) + props.border.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0) + props.border.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0) + props.border.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0) + props.border.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;

        let child_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(
                cx.constraints.available.width.shrink_by(pad_w),
                cx.constraints.available.height.shrink_by(pad_h),
            ),
        );
        let max_child = max_non_absolute_children(cx, window, child_constraints);

        let desired = Size::new(
            Px((max_child.width.0 + pad_w).max(0.0)),
            Px((max_child.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints_in_measure(desired, props.layout, cx.constraints)
    }

    fn measure_text<H: UiHost>(&mut self, cx: &mut MeasureCx<'_, H>, props: TextProps) -> Size {
        let theme = cx.theme().snapshot();
        let input = props.build_text_input(theme);
        let max_width = text_max_width_for_constraints(cx.constraints, props.wrap);
        let max_width = max_width.map(|v| crate::pixel_snap::snap_px_round(v, cx.scale_factor));
        let max_width = cx
            .tree
            .maybe_bucket_text_wrap_max_width(props.wrap, max_width);
        let constraints = TextConstraints {
            max_width,
            wrap: props.wrap,
            overflow: props.overflow,
            scale_factor: cx.scale_factor,
        };
        cx.tree
            .debug_record_text_constraints_measured(cx.node, constraints);
        let metrics = cx.services.text().measure(&input, constraints);
        clamp_to_constraints_in_measure(metrics.size, props.layout, cx.constraints)
    }

    fn measure_styled_text<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        props: crate::element::StyledTextProps,
    ) -> Size {
        let theme = cx.theme().snapshot();
        let input = props.build_text_input(theme);
        let max_width = text_max_width_for_constraints(cx.constraints, props.wrap);
        let max_width = max_width.map(|v| crate::pixel_snap::snap_px_round(v, cx.scale_factor));
        let max_width = cx
            .tree
            .maybe_bucket_text_wrap_max_width(props.wrap, max_width);
        let constraints = TextConstraints {
            max_width,
            wrap: props.wrap,
            overflow: props.overflow,
            scale_factor: cx.scale_factor,
        };
        cx.tree
            .debug_record_text_constraints_measured(cx.node, constraints);
        let metrics = cx.services.text().measure(&input, constraints);
        clamp_to_constraints_in_measure(metrics.size, props.layout, cx.constraints)
    }

    fn measure_selectable_text<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        props: crate::element::SelectableTextProps,
    ) -> Size {
        let theme = cx.theme().snapshot();
        let input = props.build_text_input(theme);
        let max_width = text_max_width_for_constraints(cx.constraints, props.wrap);
        let max_width = max_width.map(|v| crate::pixel_snap::snap_px_round(v, cx.scale_factor));
        let max_width = cx
            .tree
            .maybe_bucket_text_wrap_max_width(props.wrap, max_width);
        let constraints = TextConstraints {
            max_width,
            wrap: props.wrap,
            overflow: props.overflow,
            scale_factor: cx.scale_factor,
        };
        cx.tree
            .debug_record_text_constraints_measured(cx.node, constraints);
        let metrics = cx.services.text().measure(&input, constraints);
        clamp_to_constraints_in_measure(metrics.size, props.layout, cx.constraints)
    }

    fn measure_text_input<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        props: crate::element::TextInputProps,
    ) -> Size {
        let max_width = cx
            .constraints
            .known
            .width
            .or_else(|| cx.constraints.available.width.definite());
        let constraints = TextConstraints {
            max_width,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .text()
            .measure_str("M", &props.text_style, constraints);
        let border_h = props.chrome.border.top.0.max(0.0) + props.chrome.border.bottom.0.max(0.0);
        let pad_h = props.chrome.padding.top.0.max(0.0) + props.chrome.padding.bottom.0.max(0.0);
        let h = Px((metrics.size.height.0 + pad_h + border_h).max(0.0));

        let avail = available_px_or_zero(cx.constraints);
        let w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill | Length::Auto => avail.width,
        };

        clamp_to_constraints_in_measure(Size::new(w, h), props.layout, cx.constraints)
    }

    fn measure_text_area<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        props: crate::element::TextAreaProps,
    ) -> Size {
        let max_width = cx
            .constraints
            .known
            .width
            .or_else(|| cx.constraints.available.width.definite());
        let constraints = TextConstraints {
            max_width,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .text()
            .measure_str("M", &props.text_style, constraints);
        let border_h = props.chrome.border.top.0.max(0.0) + props.chrome.border.bottom.0.max(0.0);
        let pad_h = props.chrome.padding_y.0.max(0.0) * 2.0;
        let min_h = props.min_height.0.max(0.0);
        let h = Px((metrics.size.height.0 + pad_h + border_h).max(min_h));

        let avail = available_px_or_zero(cx.constraints);
        let w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill | Length::Auto => avail.width,
        };

        clamp_to_constraints_in_measure(Size::new(w, h), props.layout, cx.constraints)
    }

    fn measure_scroll<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        props: crate::element::ScrollProps,
    ) -> Size {
        let _span = tracing::trace_span!(
            "fret_ui.measure_scroll",
            node = ?cx.node,
            axis = ?props.axis,
            probe_unbounded = props.probe_unbounded,
            child_count = cx.children.len(),
            known_w = ?cx.constraints.known.width,
            known_h = ?cx.constraints.known.height,
            avail_w = ?cx.constraints.available.width,
            avail_h = ?cx.constraints.available.height,
        )
        .entered();

        // During intrinsic sizing, parents may pass `available.{width,height} = 0` as a
        // placeholder for "unknown". When scroll probing is enabled, treat that as non-definite
        // so the scroll node can report its measured content size and participate in
        // shrink-wrapping layouts.
        let mut constraints = cx.constraints;
        if props.probe_unbounded {
            if props.axis.scroll_x()
                && constraints.known.width.is_none()
                && constraints.available.width.definite() == Some(Px(0.0))
            {
                constraints.available.width = AvailableSpace::MaxContent;
            }
            if props.axis.scroll_y()
                && constraints.known.height.is_none()
                && constraints.available.height.definite() == Some(Px(0.0))
            {
                constraints.available.height = AvailableSpace::MaxContent;
            }
        }

        if props.intrinsic_measure_mode == crate::element::ScrollIntrinsicMeasureMode::Viewport {
            return clamp_to_constraints_in_measure(
                available_px_or_zero(constraints),
                props.layout,
                constraints,
            );
        }

        let width_determined = match props.layout.size.width {
            Length::Px(_) => true,
            Length::Fill => {
                constraints.known.width.is_some()
                    || constraints.available.width.definite().is_some()
            }
            Length::Auto => false,
        };
        let height_determined = match props.layout.size.height {
            Length::Px(_) => true,
            Length::Fill => {
                constraints.known.height.is_some()
                    || constraints.available.height.definite().is_some()
            }
            Length::Auto => false,
        };
        if width_determined && height_determined {
            return clamp_to_constraints_in_measure(
                available_px_or_zero(constraints),
                props.layout,
                constraints,
            );
        }

        let child_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(
                if props.axis.scroll_x() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    constraints.available.width
                },
                if props.axis.scroll_y() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    constraints.available.height
                },
            ),
        );
        let ignore_width_in_cache_key =
            matches!(props.axis, crate::element::ScrollAxis::Y) && props.probe_unbounded;
        let ignore_height_in_cache_key =
            matches!(props.axis, crate::element::ScrollAxis::X) && props.probe_unbounded;

        let key = ScrollMeasureKey {
            avail_w: if ignore_width_in_cache_key {
                0
            } else {
                available_space_cache_key(child_constraints.available.width)
            },
            avail_h: if ignore_height_in_cache_key {
                0
            } else {
                available_space_cache_key(child_constraints.available.height)
            },
        };
        let frame_id = cx.app.frame_id();

        let cached = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollMeasureCacheState::default,
            |state| {
                if state.frame_id != frame_id {
                    state.frame_id = frame_id;
                    state.entries.clear();
                }
                state
                    .entries
                    .iter()
                    .find_map(|(k, v)| (*k == key).then_some(*v))
            },
        );
        let max_child = if let Some(cached) = cached {
            tracing::trace!(cache_hit = true, "scroll probe cached");
            cached
        } else {
            let started = fret_core::time::Instant::now();
            let measured = max_non_absolute_children(cx, window, child_constraints);
            tracing::trace!(
                cache_hit = false,
                probe_time_us = started.elapsed().as_micros() as u64,
                "scroll probe measured"
            );
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                ScrollMeasureCacheState::default,
                |state| {
                    if state.frame_id != frame_id {
                        state.frame_id = frame_id;
                        state.entries.clear();
                    }
                    if let Some((_k, v)) = state.entries.iter_mut().find(|(k, _)| *k == key) {
                        *v = measured;
                    } else {
                        if state.entries.len() >= 8 {
                            state.entries.remove(0);
                        }
                        state.entries.push((key, measured));
                    }
                },
            );
            measured
        };

        clamp_to_constraints_in_measure(max_child, props.layout, constraints)
    }

    fn measure_virtual_list<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        props: crate::element::VirtualListProps,
    ) -> Size {
        let metrics = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::VirtualListState::default,
            |state| {
                state.metrics.ensure_with_mode(
                    props.measure_mode,
                    props.len,
                    props.estimate_row_height,
                    props.gap,
                    props.scroll_margin,
                );
                state.metrics.clone()
            },
        );
        let content_extent = metrics.total_height();

        let estimate_extent = Px(props.estimate_row_height.0.max(0.0));
        let available_w = cx
            .constraints
            .known
            .width
            .or_else(|| cx.constraints.available.width.definite())
            .unwrap_or(Px(0.0));
        let available_h = cx
            .constraints
            .known
            .height
            .or_else(|| cx.constraints.available.height.definite())
            .unwrap_or(Px(0.0));
        let measured_w = match cx.constraints.available.width {
            AvailableSpace::Definite(px) => px,
            AvailableSpace::MaxContent | AvailableSpace::MinContent => Px(0.0),
        };
        let measured_h = match cx.constraints.available.height {
            AvailableSpace::Definite(px) => px,
            AvailableSpace::MaxContent => content_extent,
            AvailableSpace::MinContent => estimate_extent,
        };
        let avail = Size::new(
            Px(available_w.0.max(measured_w.0)),
            Px(available_h.0.max(measured_h.0)),
        );
        let axis = props.axis;

        let desired_w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill => avail.width,
            Length::Auto => match axis {
                fret_core::Axis::Vertical => avail.width,
                fret_core::Axis::Horizontal => Px(content_extent.0.min(avail.width.0.max(0.0))),
            },
        };
        let desired_h = match props.layout.size.height {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill => avail.height,
            Length::Auto => match axis {
                fret_core::Axis::Vertical => Px(content_extent.0.min(avail.height.0.max(0.0))),
                fret_core::Axis::Horizontal => avail.height,
            },
        };

        clamp_to_constraints_in_measure(
            Size::new(desired_w, desired_h),
            props.layout,
            cx.constraints,
        )
    }

    fn measure_flex<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        props: FlexProps,
    ) -> Size {
        let pad_left = props.padding.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;

        let inner_available = LayoutSize::new(
            cx.constraints.available.width.shrink_by(pad_w),
            cx.constraints.available.height.shrink_by(pad_h),
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
            justify_content: Some(super::super::taffy_layout::taffy_justify(props.justify)),
            align_items: Some(super::super::taffy_layout::taffy_align_items(props.align)),
            gap: TaffySize {
                width: LengthPercentage::length(props.gap.0.max(0.0)),
                height: LengthPercentage::length(props.gap.0.max(0.0)),
            },
            size: TaffySize {
                width: match props.layout.size.width {
                    Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                    Length::Fill => match inner_available.width {
                        AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                        AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                            Dimension::auto()
                        }
                    },
                    Length::Auto => Dimension::auto(),
                },
                height: match props.layout.size.height {
                    Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                    Length::Fill => match inner_available.height {
                        AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                        AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                            Dimension::auto()
                        }
                    },
                    Length::Auto => Dimension::auto(),
                },
            },
            max_size: TaffySize {
                width: match inner_available.width {
                    AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                    AvailableSpace::MinContent | AvailableSpace::MaxContent => Dimension::auto(),
                },
                height: match inner_available.height {
                    AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                    AvailableSpace::MinContent | AvailableSpace::MaxContent => Dimension::auto(),
                },
            },
            ..Default::default()
        };

        let main_axis_definite = match props.direction {
            fret_core::Axis::Horizontal => {
                matches!(inner_available.width, AvailableSpace::Definite(_))
            }
            fret_core::Axis::Vertical => {
                matches!(inner_available.height, AvailableSpace::Definite(_))
            }
        };

        let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();
        let root = taffy.new_leaf(root_style).expect("taffy root");

        let mut child_nodes = Vec::with_capacity(cx.children.len());
        for &child in cx.children {
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

            let style = TaffyStyle {
                display: Display::Block,
                position: super::super::taffy_layout::taffy_position(layout_style.position),
                inset: super::super::taffy_layout::taffy_rect_lpa_from_inset(
                    layout_style.position,
                    layout_style.inset,
                ),
                size: TaffySize {
                    width: super::super::taffy_layout::taffy_dimension(layout_style.size.width),
                    height: super::super::taffy_layout::taffy_dimension(layout_style.size.height),
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
                margin: super::super::taffy_layout::taffy_rect_lpa_from_margin_edges(
                    layout_style.margin,
                ),
                flex_grow: if main_axis_definite {
                    layout_style.flex.grow.max(0.0)
                } else {
                    0.0
                },
                flex_shrink: layout_style.flex.shrink.max(0.0),
                flex_basis: super::super::taffy_layout::taffy_dimension(layout_style.flex.basis),
                align_self: layout_style
                    .flex
                    .align_self
                    .map(super::super::taffy_layout::taffy_align_self),
                ..Default::default()
            };

            let node = taffy
                .new_leaf_with_context(style, Some(child))
                .expect("taffy child");
            child_nodes.push(node);
        }
        taffy
            .set_children(root, &child_nodes)
            .expect("taffy children");

        let mut measure_cache: std::collections::HashMap<
            super::super::taffy_layout::TaffyMeasureKey,
            taffy::geometry::Size<f32>,
        > = std::collections::HashMap::new();
        measure_cache.reserve(cx.children.len().saturating_mul(4));

        let available = taffy::geometry::Size {
            width: available_space_to_taffy(inner_available.width),
            height: available_space_to_taffy(inner_available.height),
        };
        taffy
            .compute_layout_with_measure(root, available, |known, avail, _id, ctx, _style| {
                let Some(child) = ctx.and_then(|c| *c) else {
                    return taffy::geometry::Size::default();
                };

                let key = super::super::taffy_layout::TaffyMeasureKey {
                    child,
                    known_w: known.width.map(|v| v.to_bits()),
                    known_h: known.height.map(|v| v.to_bits()),
                    avail_w: super::super::taffy_layout::taffy_available_space_key(avail.width),
                    avail_h: super::super::taffy_layout::taffy_available_space_key(avail.height),
                };
                if let Some(size) = measure_cache.get(&key) {
                    return *size;
                }

                let constraints = LayoutConstraints::new(
                    LayoutSize::new(known.width.map(Px), known.height.map(Px)),
                    LayoutSize::new(
                        taffy_available_space_to_runtime(avail.width),
                        taffy_available_space_to_runtime(avail.height),
                    ),
                );
                let s = cx.measure_in(child, constraints);
                let out = taffy::geometry::Size {
                    width: s.width.0,
                    height: s.height.0,
                };
                measure_cache.insert(key, out);
                out
            })
            .expect("taffy compute");

        let root_layout = taffy.layout(root).expect("taffy root layout");
        let inner_size = Size::new(
            Px(root_layout.size.width.max(0.0)),
            Px(root_layout.size.height.max(0.0)),
        );
        let desired = Size::new(
            Px((inner_size.width.0 + pad_w).max(0.0)),
            Px((inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints_in_measure(desired, props.layout, cx.constraints)
    }

    fn measure_grid<H: UiHost>(
        &mut self,
        cx: &mut MeasureCx<'_, H>,
        window: AppWindowId,
        props: crate::element::GridProps,
    ) -> Size {
        let pad_left = props.padding.left.0.max(0.0);
        let pad_right = props.padding.right.0.max(0.0);
        let pad_top = props.padding.top.0.max(0.0);
        let pad_bottom = props.padding.bottom.0.max(0.0);
        let pad_w = pad_left + pad_right;
        let pad_h = pad_top + pad_bottom;

        let inner_available = LayoutSize::new(
            cx.constraints.available.width.shrink_by(pad_w),
            cx.constraints.available.height.shrink_by(pad_h),
        );

        let root_style = TaffyStyle {
            display: Display::Grid,
            justify_content: Some(super::super::taffy_layout::taffy_justify(props.justify)),
            align_items: Some(super::super::taffy_layout::taffy_align_items(props.align)),
            gap: TaffySize {
                width: LengthPercentage::length(props.gap.0.max(0.0)),
                height: LengthPercentage::length(props.gap.0.max(0.0)),
            },
            size: TaffySize {
                width: match props.layout.size.width {
                    Length::Px(px) => Dimension::length((px.0 - pad_w).max(0.0)),
                    Length::Fill => match inner_available.width {
                        AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                        AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                            Dimension::auto()
                        }
                    },
                    Length::Auto => Dimension::auto(),
                },
                height: match props.layout.size.height {
                    Length::Px(px) => Dimension::length((px.0 - pad_h).max(0.0)),
                    Length::Fill => match inner_available.height {
                        AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                        AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                            Dimension::auto()
                        }
                    },
                    Length::Auto => Dimension::auto(),
                },
            },
            max_size: TaffySize {
                width: match inner_available.width {
                    AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                    AvailableSpace::MinContent | AvailableSpace::MaxContent => Dimension::auto(),
                },
                height: match inner_available.height {
                    AvailableSpace::Definite(px) => Dimension::length(px.0.max(0.0)),
                    AvailableSpace::MinContent | AvailableSpace::MaxContent => Dimension::auto(),
                },
            },
            grid_template_columns: taffy::style_helpers::evenly_sized_tracks(props.cols),
            grid_template_rows: props
                .rows
                .map(taffy::style_helpers::evenly_sized_tracks)
                .unwrap_or_default(),
            ..Default::default()
        };

        let mut taffy: TaffyTree<Option<NodeId>> = TaffyTree::new();
        let root = taffy.new_leaf(root_style).expect("taffy root");

        let mut child_nodes = Vec::with_capacity(cx.children.len());
        for &child in cx.children {
            let layout_style = layout_style_for_node(cx.app, window, child);
            let style = TaffyStyle {
                display: Display::Block,
                position: super::super::taffy_layout::taffy_position(layout_style.position),
                inset: super::super::taffy_layout::taffy_rect_lpa_from_inset(
                    layout_style.position,
                    layout_style.inset,
                ),
                size: TaffySize {
                    width: super::super::taffy_layout::taffy_dimension(layout_style.size.width),
                    height: super::super::taffy_layout::taffy_dimension(layout_style.size.height),
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
                margin: super::super::taffy_layout::taffy_rect_lpa_from_margin_edges(
                    layout_style.margin,
                ),
                grid_column: super::super::taffy_layout::taffy_grid_line(layout_style.grid.column),
                grid_row: super::super::taffy_layout::taffy_grid_line(layout_style.grid.row),
                ..Default::default()
            };
            let node = taffy
                .new_leaf_with_context(style, Some(child))
                .expect("taffy child");
            child_nodes.push(node);
        }
        taffy
            .set_children(root, &child_nodes)
            .expect("taffy children");

        let mut measure_cache: std::collections::HashMap<
            super::super::taffy_layout::TaffyMeasureKey,
            taffy::geometry::Size<f32>,
        > = std::collections::HashMap::new();
        measure_cache.reserve(cx.children.len().saturating_mul(4));

        let available = taffy::geometry::Size {
            width: available_space_to_taffy(inner_available.width),
            height: available_space_to_taffy(inner_available.height),
        };
        taffy
            .compute_layout_with_measure(root, available, |known, avail, _id, ctx, _style| {
                let Some(child) = ctx.and_then(|c| *c) else {
                    return taffy::geometry::Size::default();
                };

                let key = super::super::taffy_layout::TaffyMeasureKey {
                    child,
                    known_w: known.width.map(|v| v.to_bits()),
                    known_h: known.height.map(|v| v.to_bits()),
                    avail_w: super::super::taffy_layout::taffy_available_space_key(avail.width),
                    avail_h: super::super::taffy_layout::taffy_available_space_key(avail.height),
                };
                if let Some(size) = measure_cache.get(&key) {
                    return *size;
                }

                let constraints = LayoutConstraints::new(
                    LayoutSize::new(known.width.map(Px), known.height.map(Px)),
                    LayoutSize::new(
                        taffy_available_space_to_runtime(avail.width),
                        taffy_available_space_to_runtime(avail.height),
                    ),
                );
                let s = cx.measure_in(child, constraints);
                let out = taffy::geometry::Size {
                    width: s.width.0,
                    height: s.height.0,
                };
                measure_cache.insert(key, out);
                out
            })
            .expect("taffy compute");

        let root_layout = taffy.layout(root).expect("taffy root layout");
        let inner_size = Size::new(
            Px(root_layout.size.width.max(0.0)),
            Px(root_layout.size.height.max(0.0)),
        );
        let desired = Size::new(
            Px((inner_size.width.0 + pad_w).max(0.0)),
            Px((inner_size.height.0 + pad_h).max(0.0)),
        );
        clamp_to_constraints_in_measure(desired, props.layout, cx.constraints)
    }
}
