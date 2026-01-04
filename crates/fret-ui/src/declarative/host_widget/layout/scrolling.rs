use super::super::ElementHostWidget;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;

impl ElementHostWidget {
    pub(super) fn layout_virtual_list_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: crate::element::VirtualListProps,
    ) -> Size {
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

        let size =
            clamp_to_constraints(Size::new(desired_w, desired_h), props.layout, cx.available);
        let viewport_h = Px(size.height.0.max(0.0));
        let mut needs_redraw = false;

        props.scroll_handle.set_items_count(props.len);

        let prev_offset = props.scroll_handle.offset();
        let mut offset_y = metrics.clamp_offset(prev_offset.y, viewport_h);

        // Avoid consuming deferred scroll requests during "probe" layout passes that use an
        // effectively-unbounded available height (e.g. Stack/Pressable measuring with
        // `Px(1.0e9)`). Those passes are not the final viewport constraints and would
        // otherwise clear the request before the real layout happens.
        let is_probe_layout = cx.available.height.0 >= 1.0e8;

        if !is_probe_layout
            && viewport_h.0 > 0.0
            && props.len > 0
            && let Some((index, strategy)) = props.scroll_handle.deferred_scroll_to_item()
        {
            offset_y = metrics.scroll_offset_for_item(index, viewport_h, offset_y, strategy);
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

        let anchor = metrics
            .visible_range(offset_y, viewport_h, 0)
            .map(|r| r.start_index);
        let anchor_offset_in_viewport = anchor.map(|anchor| {
            let start = metrics.offset_for_index(anchor);
            Px((offset_y.0 - start.0).max(0.0))
        });

        let mut measured_updates: Vec<(fret_core::NodeId, usize, Px)> =
            Vec::with_capacity(cx.children.len());

        for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
            let idx = item.index;
            let y = cx.bounds.origin.y.0 + metrics.offset_for_index(idx).0 - offset_y.0;
            let origin = fret_core::Point::new(cx.bounds.origin.x, Px(y));

            let measure_bounds = Rect::new(origin, Size::new(size.width, Px(1.0e9)));
            let measured = cx.layout_in(child, measure_bounds);
            let measured_h = Px(measured.height.0.max(0.0));

            measured_updates.push((child, idx, measured_h));
        }

        let mut any_measured_change = false;
        for (_, idx, measured_h) in &measured_updates {
            if metrics.set_measured_height(*idx, *measured_h) {
                any_measured_change = true;
            }
        }

        if any_measured_change {
            needs_redraw = true;

            if !is_probe_layout
                && let (Some(anchor), Some(anchor_offset_in_viewport)) =
                    (anchor, anchor_offset_in_viewport)
            {
                let desired = Px(metrics.offset_for_index(anchor).0 + anchor_offset_in_viewport.0);
                offset_y = metrics.clamp_offset(desired, viewport_h);

                let prev = props.scroll_handle.offset();
                if (prev.y.0 - offset_y.0).abs() > 0.01 {
                    needs_redraw = true;
                }
                props
                    .scroll_handle
                    .set_offset(fret_core::Point::new(prev.x, offset_y));
            }
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

        for (child, idx, measured_h) in &measured_updates {
            let y = cx.bounds.origin.y.0 + metrics.offset_for_index(*idx).0 - offset_y.0;
            let origin = fret_core::Point::new(cx.bounds.origin.x, Px(y));
            let child_bounds = Rect::new(origin, Size::new(size.width, *measured_h));
            let _ = cx.layout_in(*child, child_bounds);
        }

        crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::VirtualListState::default,
            |state| {
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

    pub(super) fn layout_scroll_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: crate::element::ScrollProps,
    ) -> Size {
        let probe_w = if props.axis.scroll_x() {
            Px(1.0e9)
        } else {
            cx.available.width
        };
        let probe_h = if props.axis.scroll_y() {
            Px(1.0e9)
        } else {
            cx.available.height
        };
        let probe_bounds = Rect::new(cx.bounds.origin, Size::new(probe_w, probe_h));

        let mut max_child = Size::new(Px(0.0), Px(0.0));
        for &child in cx.children {
            let child_size = cx.layout_in(child, probe_bounds);
            max_child.width = Px(max_child.width.0.max(child_size.width.0));
            max_child.height = Px(max_child.height.0.max(child_size.height.0));
        }

        let desired = clamp_to_constraints(max_child, props.layout, cx.available);
        let content_w = if props.axis.scroll_x() {
            Px(max_child.width.0.max(0.0))
        } else {
            desired.width
        };
        let content_h = if props.axis.scroll_y() {
            Px(max_child.height.0.max(0.0))
        } else {
            desired.height
        };

        // Avoid mutating the imperative handle during "probe" layout passes that use an
        // effectively-unbounded available height (e.g. Stack/Pressable measuring with
        // `Px(1.0e9)`), otherwise scroll position can be clamped to zero prematurely.
        let is_probe_layout = cx.available.height.0 >= 1.0e8 || cx.available.width.0 >= 1.0e8;
        let external_handle = props.scroll_handle.clone();
        let offset = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::ScrollState::default,
            |state| {
                let handle = external_handle.as_ref().unwrap_or(&state.scroll_handle);
                if !is_probe_layout {
                    handle.set_viewport_size(desired);
                    handle.set_content_size(Size::new(content_w, content_h));
                    let prev = handle.offset();
                    handle.set_offset(prev);
                }
                handle.offset()
            },
        );

        let offset_x = if props.axis.scroll_x() {
            offset.x
        } else {
            Px(0.0)
        };
        let offset_y = if props.axis.scroll_y() {
            offset.y
        } else {
            Px(0.0)
        };

        let shifted = Rect::new(
            fret_core::Point::new(
                Px(cx.bounds.origin.x.0 - offset_x.0),
                Px(cx.bounds.origin.y.0 - offset_y.0),
            ),
            Size::new(content_w, content_h),
        );
        for &child in cx.children {
            let _ = cx.layout_in(child, shifted);
        }

        desired
    }

    pub(super) fn layout_scrollbar_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        props: crate::element::ScrollbarProps,
    ) -> Size {
        clamp_to_constraints(cx.available, props.layout, cx.available)
    }
}
