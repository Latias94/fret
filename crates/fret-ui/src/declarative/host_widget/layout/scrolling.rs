use super::super::ElementHostWidget;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;

use crate::cache_key::CacheKeyBuilder;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use crate::tree::{UiDebugInvalidationDetail, UiDebugInvalidationSource};

impl ElementHostWidget {
    pub(super) fn layout_virtual_list_impl<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        window: AppWindowId,
        props: crate::element::VirtualListProps,
    ) -> Size {
        let axis = props.axis;
        let (
            mut metrics,
            prev_items_revision,
            render_window_range,
            prev_window_range,
            prev_offset_x,
            prev_offset_y,
            prev_viewport_w,
            prev_viewport_h,
        ) = crate::elements::with_element_state(
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
                (
                    state.metrics.clone(),
                    state.items_revision,
                    state.render_window_range,
                    state.window_range,
                    state.offset_x,
                    state.offset_y,
                    state.viewport_w,
                    state.viewport_h,
                )
            },
        );
        let content_extent = metrics.total_height();
        let should_remeasure_visible_items = props.items_revision != prev_items_revision;

        let desired_w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill => cx.available.width,
            Length::Auto => match axis {
                fret_core::Axis::Vertical => cx.available.width,
                fret_core::Axis::Horizontal => {
                    Px(content_extent.0.min(cx.available.width.0.max(0.0)))
                }
            },
        };
        let desired_h = match props.layout.size.height {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill => cx.available.height,
            Length::Auto => match axis {
                fret_core::Axis::Vertical => {
                    Px(content_extent.0.min(cx.available.height.0.max(0.0)))
                }
                fret_core::Axis::Horizontal => cx.available.height,
            },
        };

        let size =
            clamp_to_constraints(Size::new(desired_w, desired_h), props.layout, cx.available);
        let viewport = match axis {
            fret_core::Axis::Vertical => Px(size.height.0.max(0.0)),
            fret_core::Axis::Horizontal => Px(size.width.0.max(0.0)),
        };
        let mut needs_redraw = false;

        let cross_extent = match axis {
            fret_core::Axis::Vertical => size.width,
            fret_core::Axis::Horizontal => size.height,
        };
        if metrics.reset_measured_cache_if_cross_extent_changed(cross_extent) {
            needs_redraw = true;
        }

        props.scroll_handle.set_items_count(props.len);
        self.scroll_child_transform = Some(super::super::ScrollChildTransform {
            handle: props.scroll_handle.base_handle().clone(),
            axis: match axis {
                fret_core::Axis::Vertical => crate::element::ScrollAxis::Y,
                fret_core::Axis::Horizontal => crate::element::ScrollAxis::X,
            },
        });

        let prev_offset = props.scroll_handle.offset();
        let prev_offset_axis = match axis {
            fret_core::Axis::Vertical => prev_offset.y,
            fret_core::Axis::Horizontal => prev_offset.x,
        };
        let mut offset = metrics.clamp_offset(prev_offset_axis, viewport);
        let deferred_scroll_to_item = props.scroll_handle.deferred_scroll_to_item().is_some();
        let mut deferred_scroll_consumed = false;

        // Avoid consuming deferred scroll requests during "probe" layout passes that use an
        // effectively-unbounded available space. Those passes are not the final viewport
        // constraints and would
        // otherwise clear the request before the real layout happens.
        let is_probe_layout = cx.pass_kind == crate::layout_pass::LayoutPassKind::Probe;

        if !is_probe_layout
            && viewport.0 > 0.0
            && props.len > 0
            && let Some((index, strategy)) = props.scroll_handle.deferred_scroll_to_item()
        {
            deferred_scroll_consumed = true;
            offset = metrics.scroll_offset_for_item(index, viewport, offset, strategy);
            props.scroll_handle.clear_deferred_scroll_to_item();
        }

        offset = metrics.clamp_offset(offset, viewport);

        if (prev_offset_axis.0 - offset.0).abs() > 0.01 {
            needs_redraw = true;
        }

        let visible_range = metrics.visible_range(offset, viewport, 0);
        let anchor = visible_range.map(|r| r.start_index);
        let anchor_offset_in_viewport = anchor.map(|anchor| {
            let start = metrics.offset_for_index(anchor);
            Px((offset.0 - start.0).max(0.0))
        });

        let mut measured_updates: Vec<(fret_core::NodeId, usize, Px)> =
            Vec::with_capacity(cx.children.len());

        match props.measure_mode {
            crate::element::VirtualListMeasureMode::Measured => {
                let item_constraints = LayoutConstraints::new(
                    LayoutSize::new(
                        match axis {
                            fret_core::Axis::Vertical => Some(size.width),
                            fret_core::Axis::Horizontal => None,
                        },
                        match axis {
                            fret_core::Axis::Vertical => None,
                            fret_core::Axis::Horizontal => Some(size.height),
                        },
                    ),
                    LayoutSize::new(
                        match axis {
                            fret_core::Axis::Vertical => AvailableSpace::Definite(size.width),
                            fret_core::Axis::Horizontal => AvailableSpace::MaxContent,
                        },
                        match axis {
                            fret_core::Axis::Vertical => AvailableSpace::MaxContent,
                            fret_core::Axis::Horizontal => AvailableSpace::Definite(size.height),
                        },
                    ),
                );

                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let should_measure = !metrics.is_measured(idx)
                        || (cx.pass_kind == crate::layout_pass::LayoutPassKind::Final
                            && cx.tree.node_needs_layout(child));
                    let measured_extent = if should_measure {
                        #[cfg(test)]
                        crate::virtual_list::debug_record_virtual_list_item_measure();
                        let measured = cx.measure_in(child, item_constraints);
                        match axis {
                            fret_core::Axis::Vertical => Px(measured.height.0.max(0.0)),
                            fret_core::Axis::Horizontal => Px(measured.width.0.max(0.0)),
                        }
                    } else {
                        metrics.height_at(idx)
                    };

                    measured_updates.push((child, idx, measured_extent));
                }

                let mut any_measured_change = false;
                for (_, idx, measured_extent) in &measured_updates {
                    if metrics.set_measured_height(*idx, *measured_extent) {
                        any_measured_change = true;
                    }
                }

                if any_measured_change || should_remeasure_visible_items {
                    needs_redraw = true;

                    if !is_probe_layout
                        && let (Some(anchor), Some(anchor_offset_in_viewport)) =
                            (anchor, anchor_offset_in_viewport)
                    {
                        let prev_offset = offset;
                        let desired =
                            Px(metrics.offset_for_index(anchor).0 + anchor_offset_in_viewport.0);
                        offset = metrics.clamp_offset(desired, viewport);
                        if (prev_offset.0 - offset.0).abs() > 0.01 {
                            needs_redraw = true;
                        }
                    }
                }
            }
            crate::element::VirtualListMeasureMode::Fixed => {
                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let estimated_extent = metrics.height_at(idx);
                    measured_updates.push((child, idx, estimated_extent));
                }
            }
            crate::element::VirtualListMeasureMode::Known => {
                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let known_extent = metrics.height_at(idx);
                    measured_updates.push((child, idx, known_extent));
                }
            }
        }

        let content_extent = metrics.total_height();
        props
            .scroll_handle
            .set_viewport_size_internal(Size::new(size.width, size.height));
        let content_size = match axis {
            fret_core::Axis::Vertical => Size::new(size.width, content_extent),
            fret_core::Axis::Horizontal => Size::new(content_extent, size.height),
        };
        props.scroll_handle.set_content_size_internal(content_size);

        let prev_offset = props.scroll_handle.offset();
        let clamped = metrics.clamp_offset(offset, viewport);
        if (clamped.0 - offset.0).abs() > 0.01 {
            needs_redraw = true;
        }
        match axis {
            fret_core::Axis::Vertical => {
                props
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(prev_offset.x, clamped));
            }
            fret_core::Axis::Horizontal => {
                props
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(clamped, prev_offset.y));
            }
        }
        offset = clamped;

        // Layout children in stable "content space" and apply the scroll offset via
        // `children_render_transform` (same pattern as `Scroll`).
        //
        // This avoids the "translation-only layout" O(N) subtree bound updates that happen when
        // we bake the scroll offset into each child's layout rect.
        self.scroll_child_transform = Some(super::super::ScrollChildTransform {
            handle: props.scroll_handle.base_handle().clone(),
            axis: match axis {
                fret_core::Axis::Vertical => crate::element::ScrollAxis::Y,
                fret_core::Axis::Horizontal => crate::element::ScrollAxis::X,
            },
        });

        let mut child_rects: Vec<(NodeId, Rect)> = Vec::with_capacity(measured_updates.len());
        for (child, idx, measured_extent) in &measured_updates {
            let start = metrics.offset_for_index(*idx);
            let origin = match axis {
                fret_core::Axis::Vertical => {
                    let y = cx.bounds.origin.y.0 + start.0;
                    fret_core::Point::new(cx.bounds.origin.x, Px(y))
                }
                fret_core::Axis::Horizontal => {
                    let x = cx.bounds.origin.x.0 + start.0;
                    fret_core::Point::new(Px(x), cx.bounds.origin.y)
                }
            };
            let child_bounds = match axis {
                fret_core::Axis::Vertical => {
                    Rect::new(origin, Size::new(size.width, *measured_extent))
                }
                fret_core::Axis::Horizontal => {
                    Rect::new(origin, Size::new(*measured_extent, size.height))
                }
            };
            child_rects.push((*child, child_bounds));
        }

        if !is_probe_layout {
            cx.solve_barrier_child_roots_if_needed(&child_rects);
        }

        for (child, child_bounds) in &child_rects {
            let _ = cx.layout_in(*child, *child_bounds);
        }

        let window_range = if !is_probe_layout {
            metrics.visible_range(offset, viewport, props.overscan)
        } else {
            None
        };

        crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::VirtualListState::default,
            |state| {
                match axis {
                    fret_core::Axis::Vertical => {
                        state.offset_y = offset;
                        if state.viewport_h != viewport {
                            state.viewport_h = viewport;
                            needs_redraw = true;
                        }
                    }
                    fret_core::Axis::Horizontal => {
                        state.offset_x = offset;
                        if state.viewport_w != viewport {
                            state.viewport_w = viewport;
                            needs_redraw = true;
                        }
                    }
                }
                if !is_probe_layout && viewport.0 > 0.0 {
                    state.has_final_viewport = true;
                }
                if !is_probe_layout {
                    state.window_range = window_range;
                    state.deferred_scroll_offset_hint = None;
                }
                state.items_revision = props.items_revision;
                state.metrics = metrics;
            },
        );

        let window_mismatch = {
            // `render_window_range` is the window that was used during declarative render to build
            // `props.visible_items` (typically an overscanned window).
            //
            // `visible_range` is the true visible window (no overscan).
            //
            // We only need to force a cache-root rerender when the current visible window falls
            // outside the previously rendered window. A mere mismatch between the “ideal” window
            // for the current scroll offset and the rendered window is expected and should not
            // trigger rerender while we're still within overscan.
            if is_probe_layout || viewport.0 <= 0.0 {
                false
            } else if let Some(visible) = visible_range {
                match render_window_range {
                    None => visible.count > 0,
                    Some(rendered) => {
                        if rendered.count == 0 {
                            // If declarative render couldn't produce a window (typically because
                            // the viewport size was unknown), the next non-probe layout pass will
                            // compute a real visible range. Ensure we schedule a rerender so the
                            // view-cache root can build the initial visible items.
                            visible.count > 0
                        } else {
                            let rendered_start =
                                rendered.start_index.saturating_sub(rendered.overscan);
                            let rendered_end = (rendered.end_index + rendered.overscan)
                                .min(rendered.count.saturating_sub(1));
                            visible.start_index < rendered_start || visible.end_index > rendered_end
                        }
                    }
                }
            } else {
                false
            }
        };

        if cx.tree.debug_enabled() {
            let policy_key = {
                let mut b = CacheKeyBuilder::new();
                b.write_u32(axis as u32);
                b.write_u32(props.measure_mode as u32);
                b.write_u64(props.overscan as u64);
                b.write_px(props.estimate_row_height);
                b.write_px(props.gap);
                b.write_px(props.scroll_margin);
                b.finish()
            };
            let inputs_key = {
                let mut b = CacheKeyBuilder::new();
                b.write_u64(policy_key);
                b.write_u64(props.len as u64);
                b.write_u64(props.items_revision);
                b.write_px(viewport);
                b.write_px(offset);
                b.write_px(content_extent);
                b.finish()
            };
            let prev_offset_state = match axis {
                fret_core::Axis::Vertical => prev_offset_y,
                fret_core::Axis::Horizontal => prev_offset_x,
            };
            let prev_viewport_state = match axis {
                fret_core::Axis::Vertical => prev_viewport_h,
                fret_core::Axis::Horizontal => prev_viewport_w,
            };

            cx.tree
                .debug_record_virtual_list_window(crate::tree::UiDebugVirtualListWindow {
                    source: crate::tree::UiDebugVirtualListWindowSource::Layout,
                    node: cx.node,
                    element: self.element,
                    axis,
                    is_probe_layout,
                    items_len: props.len,
                    items_revision: props.items_revision,
                    prev_items_revision,
                    measure_mode: props.measure_mode,
                    overscan: props.overscan,
                    estimate_row_height: props.estimate_row_height,
                    gap: props.gap,
                    scroll_margin: props.scroll_margin,
                    viewport,
                    prev_viewport: prev_viewport_state,
                    offset,
                    prev_offset: prev_offset_state,
                    content_extent,
                    policy_key,
                    inputs_key,
                    window_range,
                    prev_window_range,
                    render_window_range,
                    deferred_scroll_to_item,
                    deferred_scroll_consumed,
                    window_mismatch,
                    window_shift_kind: if window_mismatch {
                        crate::tree::UiDebugVirtualListWindowShiftKind::Escape
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftKind::None
                    },
                });
        }

        if !is_probe_layout && cx.tree.view_cache_enabled() && window_mismatch {
            let retained_host =
                crate::elements::with_window_state(&mut *cx.app, window, |window_state| {
                    let retained = window_state
                        .has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                        self.element,
                    );
                    if retained {
                        window_state.mark_retained_virtual_list_needs_reconcile(
                            self.element,
                            crate::tree::UiDebugRetainedVirtualListReconcileKind::Escape,
                        );
                    }
                    retained
                });

            if !retained_host {
                // Virtual list visible-item sets are computed during the declarative render pass.
                // If the current visible window is outside the previously rendered overscan
                // window, ensure the nearest view-cache root re-renders on the next frame so it
                // can rebuild the visible items.
                //
                // Important: avoid forcing an additional "contained relayout" pass in the current
                // frame. We only need to mark the view-cache reuse gate as dirty and schedule a
                // redraw; the rerender frame will rebuild children and propagate structural
                // invalidations normally.
                cx.tree.mark_nearest_view_cache_root_needs_rerender(
                    cx.node,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                );
            }
            needs_redraw = true;
        }

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
        let mut max_child = Size::new(Px(0.0), Px(0.0));
        let child_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(
                if props.axis.scroll_x() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(cx.available.width)
                },
                if props.axis.scroll_y() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(cx.available.height)
                },
            ),
        );
        for &child in cx.children {
            let child_size = cx.measure_in(child, child_constraints);
            max_child.width = Px(max_child.width.0.max(child_size.width.0));
            max_child.height = Px(max_child.height.0.max(child_size.height.0));
        }

        let desired = clamp_to_constraints(max_child, props.layout, cx.available);
        // Scroll containers should not under-report their scrollable extent due to fractional
        // layout rounding. Match DOM behavior by rounding the scrollable axis up to the next
        // whole pixel (tolerating tiny floating point noise).
        const ROUND_EPSILON: f32 = 0.001;
        let content_w = if props.axis.scroll_x() {
            Px((max_child.width.0.max(0.0) - ROUND_EPSILON).ceil().max(0.0))
        } else {
            desired.width
        };
        let content_h = if props.axis.scroll_y() {
            Px((max_child.height.0.max(0.0) - ROUND_EPSILON)
                .ceil()
                .max(0.0))
        } else {
            desired.height
        };
        // Ensure the scroll content bounds never underflow the viewport bounds.
        //
        // This matches DOM behavior (the scrollable content box is at least the viewport size),
        // and prevents `Length::Fill` descendants from collapsing when we probe with
        // `AvailableSpace::MaxContent` on the scroll axis.
        let content_w = Px(content_w.0.max(desired.width.0.max(0.0)));
        let content_h = Px(content_h.0.max(desired.height.0.max(0.0)));

        // Avoid mutating the imperative handle during "probe" layout passes that use an
        // effectively-unbounded available space, otherwise scroll position can be clamped to zero
        // prematurely.
        let is_probe_layout = cx.pass_kind == crate::layout_pass::LayoutPassKind::Probe;
        let external_handle = props.scroll_handle.clone();
        let handle = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::ScrollState::default,
            |state| {
                let handle = external_handle
                    .as_ref()
                    .unwrap_or(&state.scroll_handle)
                    .clone();
                if !is_probe_layout {
                    handle.set_viewport_size_internal(desired);
                    handle.set_content_size_internal(Size::new(content_w, content_h));
                    let prev = handle.offset();
                    handle.set_offset_internal(prev);
                }
                handle
            },
        );

        self.scroll_child_transform = Some(super::super::ScrollChildTransform {
            handle: handle.clone(),
            axis: props.axis,
        });

        let content_bounds = Rect::new(cx.bounds.origin, Size::new(content_w, content_h));

        if !is_probe_layout {
            let roots: Vec<(NodeId, Rect)> =
                cx.children.iter().map(|&c| (c, content_bounds)).collect();
            cx.solve_barrier_child_roots_if_needed(&roots);
        }

        for &child in cx.children {
            let _ = cx.layout_in(child, content_bounds);
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
