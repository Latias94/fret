use super::super::ElementHostWidget;
use crate::declarative::layout_helpers::clamp_to_constraints;
use crate::declarative::prelude::*;

use crate::cache_key::CacheKeyBuilder;
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use crate::tree::{
    UiDebugInvalidationDetail, UiDebugInvalidationSource, UiDebugScrollAxis,
    UiDebugScrollNodeTelemetry,
};
use fret_core::FrameId;
use fret_core::time::{Duration, Instant};
use std::sync::OnceLock;

#[derive(Debug, Clone)]
struct ScrollLayoutProfileConfig {
    min_elapsed: Duration,
    min_self_measure: Duration,
}

impl ScrollLayoutProfileConfig {
    fn from_env() -> Option<Self> {
        let cfg = crate::runtime_config::ui_runtime_config().scroll_layout_profile?;
        Some(Self {
            min_elapsed: cfg.min_elapsed,
            min_self_measure: cfg.min_self_measure,
        })
    }
}

fn scroll_layout_profile_config() -> Option<&'static ScrollLayoutProfileConfig> {
    static CONFIG: OnceLock<Option<ScrollLayoutProfileConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(ScrollLayoutProfileConfig::from_env)
        .as_ref()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollLayoutProbeKey {
    avail_w: u64,
    avail_h: u64,
}

#[derive(Debug, Default, Clone)]
struct ScrollLayoutProbeCacheState {
    frame_id: FrameId,
    entries: Vec<(ScrollLayoutProbeKey, Size)>,
    last_max_child: Size,
}

fn available_space_cache_key(space: AvailableSpace) -> u64 {
    match space {
        AvailableSpace::Definite(px) => px.0.to_bits() as u64,
        AvailableSpace::MinContent => 1 << 62,
        AvailableSpace::MaxContent => 2 << 62,
    }
}

fn scroll_defer_unbounded_probe_on_resize_enabled() -> bool {
    crate::runtime_config::ui_runtime_config().scroll_defer_unbounded_probe_on_resize
}

fn scroll_defer_unbounded_probe_on_invalidation_enabled() -> bool {
    crate::runtime_config::ui_runtime_config().scroll_defer_unbounded_probe_on_invalidation
}

fn scroll_defer_unbounded_probe_stable_frames() -> u8 {
    crate::runtime_config::ui_runtime_config().scroll_defer_unbounded_probe_stable_frames
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ScrollDeferredUnboundedProbeKind {
    #[default]
    None,
    Invalidation,
    Resize,
}

#[derive(Debug, Default, Clone, Copy)]
struct ScrollDeferredUnboundedProbeState {
    kind: ScrollDeferredUnboundedProbeKind,
    stable_frames: u8,
    pending_invalidation_probe: bool,
}

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
            mut layout_scratch,
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
                    std::mem::take(&mut state.metrics),
                    state.items_revision,
                    state.render_window_range,
                    state.window_range,
                    state.offset_x,
                    state.offset_y,
                    state.viewport_w,
                    state.viewport_h,
                    std::mem::take(&mut state.layout_scratch),
                )
            },
        );
        let content_extent = metrics.total_height();
        let should_remeasure_visible_items = props.items_revision != prev_items_revision;

        let desired_w = match props.layout.size.width {
            Length::Px(px) => Px(px.0.max(0.0)),
            Length::Fill => cx.available.width,
            Length::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                Px((cx.available.width.0 * f).max(0.0))
            }
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
            Length::Fraction(f) => {
                let f = if f.is_finite() { f.max(0.0) } else { 0.0 };
                Px((cx.available.height.0 * f).max(0.0))
            }
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

        let handle_offset = props.scroll_handle.offset();
        let handle_offset_axis = match axis {
            fret_core::Axis::Vertical => handle_offset.y,
            fret_core::Axis::Horizontal => handle_offset.x,
        };
        let mut offset = metrics.clamp_offset(handle_offset_axis, viewport);
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
            props
                .scroll_handle
                .clear_deferred_scroll_to_item(cx.app.frame_id());
        }

        offset = metrics.clamp_offset(offset, viewport);

        if (handle_offset_axis.0 - offset.0).abs() > 0.01 {
            needs_redraw = true;
        }

        let visible_range = metrics.visible_range(offset, viewport, 0);
        let anchor = visible_range.map(|r| r.start_index);
        let anchor_offset_in_viewport = anchor.map(|anchor| {
            let start = metrics.offset_for_index(anchor);
            Px((offset.0 - start.0).max(0.0))
        });

        layout_scratch.measured_updates.clear();
        layout_scratch.measured_updates.reserve(cx.children.len());

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
                    // Treat `items_revision` as the mechanism-level contract for "size-affecting
                    // content changed". Avoid forcing re-measure just because the widget subtree
                    // was (re)mounted or otherwise marked layout-invalidated: the virtualizer can
                    // legitimately reuse a cached extent for a previously measured index.
                    let should_measure =
                        should_remeasure_visible_items || !metrics.is_measured(idx);
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

                    layout_scratch
                        .measured_updates
                        .push((child, idx, measured_extent));
                }

                let mut any_measured_change = false;
                for (_, idx, measured_extent) in &layout_scratch.measured_updates {
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
                    layout_scratch
                        .measured_updates
                        .push((child, idx, estimated_extent));
                }
            }
            crate::element::VirtualListMeasureMode::Known => {
                for (&child, item) in cx.children.iter().zip(props.visible_items.iter()) {
                    let idx = item.index;
                    let known_extent = metrics.height_at(idx);
                    layout_scratch
                        .measured_updates
                        .push((child, idx, known_extent));
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

        layout_scratch.barrier_roots.clear();
        layout_scratch
            .barrier_roots
            .reserve(layout_scratch.measured_updates.len());
        let mut should_defer_overscan_layout = false;
        if !is_probe_layout && props.overscan > 0 && viewport.0 > 0.0 {
            if deferred_scroll_consumed {
                // On large scroll-to-item jumps, laying out the full overscan window in a single
                // frame can create tail spikes. Prioritize the true visible window and let
                // overscanned rows catch up on subsequent frames.
                should_defer_overscan_layout = true;
            } else {
                // `scroll_to_bottom()` / `scroll_to_item()` may update the handle immediately
                // (without a deferred-scroll marker). Detect large jumps by comparing against the
                // last committed offset from the element state.
                let prev_state_viewport_axis = match axis {
                    fret_core::Axis::Vertical => prev_viewport_h,
                    fret_core::Axis::Horizontal => prev_viewport_w,
                };
                let prev_state_offset_axis = match axis {
                    fret_core::Axis::Vertical => prev_offset_y,
                    fret_core::Axis::Horizontal => prev_offset_x,
                };

                let viewport_unchanged = (prev_state_viewport_axis.0 - viewport.0).abs() <= 0.01
                    && prev_state_viewport_axis.0 > 0.0;

                if viewport_unchanged {
                    let prev_clamped = metrics.clamp_offset(prev_state_offset_axis, viewport);
                    let prev_visible = metrics.visible_range(prev_clamped, viewport, 0);

                    let large_index_jump = match (prev_visible, visible_range) {
                        (Some(prev), Some(now)) => {
                            let prev_len = prev
                                .end_index
                                .saturating_sub(prev.start_index)
                                .saturating_add(1);
                            let threshold = prev_len
                                .saturating_mul(4)
                                .max(props.overscan.saturating_mul(8));
                            now.start_index.abs_diff(prev.start_index) > threshold
                        }
                        _ => {
                            let delta_px = (offset.0 - prev_clamped.0).abs();
                            delta_px > (viewport.0 * 3.0)
                        }
                    };

                    if large_index_jump {
                        should_defer_overscan_layout = true;
                    }
                }
            }
        }

        let bounds_for_start_and_extent = |start: Px, extent: Px| -> Rect {
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
            match axis {
                fret_core::Axis::Vertical => Rect::new(origin, Size::new(size.width, extent)),
                fret_core::Axis::Horizontal => Rect::new(origin, Size::new(extent, size.height)),
            }
        };

        let use_visible_item_starts = props.measure_mode
            != crate::element::VirtualListMeasureMode::Measured
            && layout_scratch.measured_updates.len() == props.visible_items.len();

        let mut prev_idx: Option<usize> = None;
        let mut prev_start: Px = Px(0.0);
        let mut prev_extent: Px = Px(0.0);
        let gap = metrics.gap();

        for (pos, (child, idx, measured_extent)) in
            layout_scratch.measured_updates.iter().enumerate()
        {
            let start = if use_visible_item_starts {
                props
                    .visible_items
                    .get(pos)
                    .map(|item| item.start)
                    .unwrap_or_else(|| metrics.offset_for_index(*idx))
            } else {
                let start = if let Some(prev) = prev_idx
                    && *idx == prev.saturating_add(1)
                {
                    Px(prev_start.0 + prev_extent.0 + gap.0)
                } else {
                    metrics.offset_for_index(*idx)
                };
                prev_idx = Some(*idx);
                prev_start = start;
                prev_extent = *measured_extent;
                start
            };

            if should_defer_overscan_layout {
                let Some(visible) = visible_range else {
                    continue;
                };
                if *idx < visible.start_index || *idx > visible.end_index {
                    continue;
                }
            }

            let child_bounds = bounds_for_start_and_extent(start, *measured_extent);
            layout_scratch.barrier_roots.push((*child, child_bounds));
        }

        if !is_probe_layout {
            cx.solve_barrier_child_roots_if_needed(&layout_scratch.barrier_roots);
        }

        for (child, child_bounds) in &layout_scratch.barrier_roots {
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
                if !is_probe_layout {
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
                    if viewport.0 > 0.0 {
                        state.has_final_viewport = true;
                    }

                    state.window_range = window_range;
                    state.deferred_scroll_offset_hint = None;
                }
                state.items_revision = props.items_revision;
                state.metrics = metrics;
                state.layout_scratch = layout_scratch;
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
            let scroll_to_item_consumed_in_frame = props
                .scroll_handle
                .scroll_to_item_consumed_in_frame(cx.app.frame_id());
            let scroll_to_item_in_frame =
                deferred_scroll_to_item || scroll_to_item_consumed_in_frame;
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
            let (window_shift_reason, window_shift_apply_mode, window_shift_invalidation_detail) =
                if window_mismatch {
                    let reason = if scroll_to_item_in_frame {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem
                    } else if props.items_revision != prev_items_revision {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision
                    } else if (viewport.0 - prev_viewport_state.0).abs() > 0.01 {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize
                    } else if (offset.0 - prev_offset_state.0).abs() > 0.01 {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset
                    } else if prev_window_range.map(|r| (r.count, r.overscan))
                        != window_range.map(|r| (r.count, r.overscan))
                    {
                        crate::tree::UiDebugVirtualListWindowShiftReason::InputsChange
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftReason::Unknown
                    };
                    let retained_host = crate::elements::with_window_state(
                        &mut *cx.app,
                        window,
                        |window_state| {
                            window_state.has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(self.element)
                        },
                    );
                    let mode = if retained_host {
                        crate::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender
                    };
                    let invalidation_detail = if cx.tree.view_cache_enabled() && !retained_host {
                        Some(match reason {
                            crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => {
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate
                            }
                            crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => {
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate
                            }
                            crate::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => {
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleItemsRevisionWindowUpdate
                            }
                            _ => crate::tree::UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                        })
                    } else {
                        None
                    };
                    (Some(reason), Some(mode), invalidation_detail)
                } else {
                    (None, None, None)
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
                    deferred_scroll_to_item: scroll_to_item_in_frame,
                    deferred_scroll_consumed: deferred_scroll_consumed
                        || scroll_to_item_consumed_in_frame,
                    window_mismatch,
                    window_shift_kind: if window_mismatch {
                        crate::tree::UiDebugVirtualListWindowShiftKind::Escape
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftKind::None
                    },
                    window_shift_reason,
                    window_shift_apply_mode,
                    window_shift_invalidation_detail,
                });
        }

        // Window-boundary invalidation under view-cache is prepaint-driven (ADR 0175):
        // - retained hosts reconcile during prepaint,
        // - non-retained lists schedule a one-shot rerender during prepaint.
        //
        // Layout still records window telemetry and updates `VirtualListState`, but should not
        // duplicate the scheduling side effects.
        if !is_probe_layout && cx.tree.view_cache_enabled() && window_mismatch {
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
        let profile_cfg = scroll_layout_profile_config();
        let profile_started = profile_cfg.is_some().then(Instant::now);
        let mut t_measure_children: Duration = Duration::default();
        let mut t_solve_barrier: Duration = Duration::default();
        let mut t_layout_children: Duration = Duration::default();

        let is_probe_layout = cx.pass_kind == crate::layout_pass::LayoutPassKind::Probe;

        // Acquire the imperative handle early so probe layout passes can use the last known
        // viewport size instead of the probe pass' effectively-unbounded available size.
        //
        // This keeps scroll probing stable across probe/final passes and avoids accidental
        // "infinite window" layouts (e.g. text reflowing as a single long line) during probes.
        let external_handle = props.scroll_handle.clone();
        let (handle, intrinsic_measure_cache) = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::ScrollState::default,
            |state| {
                (
                    external_handle
                        .as_ref()
                        .unwrap_or(&state.scroll_handle)
                        .clone(),
                    state.intrinsic_measure_cache,
                )
            },
        );

        let available = if is_probe_layout {
            let last = handle.viewport_size();
            if last.width.0 > 0.0 && last.height.0 > 0.0 {
                last
            } else {
                cx.available
            }
        } else {
            cx.available
        };

        // If the user is already at the current scroll extent edge, avoid relying on prior-frame
        // caches for the content extent. Otherwise the scroll container can temporarily "pin" its
        // content size to the previous frame, making it impossible to scroll further when content
        // grows (e.g. expanding a collapsible near the bottom of a scroll view).
        let prev_offset = handle.offset();
        let prev_max_offset = handle.max_offset();
        let at_scroll_extent_edge = match props.axis {
            crate::element::ScrollAxis::X => prev_offset.x.0 + 0.5 >= prev_max_offset.x.0,
            crate::element::ScrollAxis::Y => prev_offset.y.0 + 0.5 >= prev_max_offset.y.0,
            crate::element::ScrollAxis::Both => {
                prev_offset.x.0 + 0.5 >= prev_max_offset.x.0
                    || prev_offset.y.0 + 0.5 >= prev_max_offset.y.0
            }
        };

        let child_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(
                if props.axis.scroll_x() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(available.width)
                },
                if props.axis.scroll_y() && props.probe_unbounded {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(available.height)
                },
            ),
        );

        let children_layout_invalidated = cx
            .children
            .iter()
            .copied()
            .any(|child| cx.tree.node_layout_invalidated(child));
        let at_end_with_invalidated_child = at_scroll_extent_edge && children_layout_invalidated;

        let mut intrinsic_cached_max_child: Option<Size> = None;
        let mut cached_max_child: Option<Size> = None;
        if !is_probe_layout && cx.children.len() == 1 {
            let child = cx.children[0];
            let cache_key = crate::element::ScrollIntrinsicMeasureCacheKey {
                avail_w: available_space_cache_key(child_constraints.available.width),
                avail_h: available_space_cache_key(child_constraints.available.height),
                axis: match props.axis {
                    crate::element::ScrollAxis::X => 0,
                    crate::element::ScrollAxis::Y => 1,
                    crate::element::ScrollAxis::Both => 2,
                },
                probe_unbounded: props.probe_unbounded,
                scale_bits: cx.scale_factor.to_bits(),
            };

            intrinsic_cached_max_child = intrinsic_measure_cache
                .and_then(|cache| (cache.key == cache_key).then_some(cache.max_child));
            // Safe fast path: only use intrinsic size caching as a substitute for measuring the
            // child when the child subtree does not need layout this frame.
            if !at_end_with_invalidated_child && !cx.tree.node_needs_layout(child) {
                cached_max_child = intrinsic_cached_max_child;
            }
        }

        let wants_unbounded_probe = props.probe_unbounded
            && (props.axis.scroll_x() || props.axis.scroll_y())
            && !is_probe_layout;
        let defer_probe_on_resize = scroll_defer_unbounded_probe_on_resize_enabled();
        let defer_probe_on_invalidation = scroll_defer_unbounded_probe_on_invalidation_enabled();
        let prev_viewport = handle.viewport_size();
        let viewport_known = prev_viewport.width.0 > 0.0 && prev_viewport.height.0 > 0.0;
        let viewport_changed = viewport_known
            && (prev_viewport.width.0.to_bits() != available.width.0.to_bits()
                || prev_viewport.height.0.to_bits() != available.height.0.to_bits());

        let can_defer_probe_with_cached_max_child = intrinsic_cached_max_child
            .or(cached_max_child)
            .is_some_and(|size| size != Size::default());
        let can_defer_probe_with_cached_children = can_defer_probe_with_cached_max_child
            || cx.children.iter().copied().any(|child| {
                cx.tree
                    .node_measured_size(child)
                    .is_some_and(|size| size != Size::default())
            });
        let viewport_became_known_during_resize = !viewport_known
            && cx.tree.interactive_resize_active()
            && available.width.0 > 0.0
            && available.height.0 > 0.0
            && can_defer_probe_with_cached_children;

        let should_defer_unbounded_probe_on_resize = wants_unbounded_probe
            && defer_probe_on_resize
            && (viewport_changed || viewport_became_known_during_resize);
        let should_defer_unbounded_probe_on_invalidation = wants_unbounded_probe
            && defer_probe_on_invalidation
            && can_defer_probe_with_cached_children
            && children_layout_invalidated
            && !at_scroll_extent_edge;

        let stable_frames_required = scroll_defer_unbounded_probe_stable_frames();
        let (defer_state, defer_this_frame) = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollDeferredUnboundedProbeState::default,
            |state| {
                let mut defer_this_frame = false;
                if should_defer_unbounded_probe_on_resize {
                    defer_this_frame = true;
                    state.kind = ScrollDeferredUnboundedProbeKind::Resize;
                    state.stable_frames = 0;
                } else {
                    match state.kind {
                        ScrollDeferredUnboundedProbeKind::Resize => {
                            if stable_frames_required == 0 {
                                state.kind = ScrollDeferredUnboundedProbeKind::None;
                                state.stable_frames = 0;
                            } else {
                                state.stable_frames = state.stable_frames.saturating_add(1);
                                if state.stable_frames < stable_frames_required {
                                    defer_this_frame = true;
                                } else {
                                    state.kind = ScrollDeferredUnboundedProbeKind::None;
                                    state.stable_frames = 0;
                                }
                            }
                        }
                        ScrollDeferredUnboundedProbeKind::Invalidation => {
                            // Under view-cache reconciliation, descendants can remain layout-invalidated
                            // for multiple frames. Keep deferring while invalidated, and only allow the
                            // expensive unbounded probe once the subtree stabilizes for a few frames.
                            if at_scroll_extent_edge {
                                state.kind = ScrollDeferredUnboundedProbeKind::None;
                                state.stable_frames = 0;
                            } else if !wants_unbounded_probe || !defer_probe_on_invalidation {
                                state.kind = ScrollDeferredUnboundedProbeKind::None;
                                state.stable_frames = 0;
                            } else if children_layout_invalidated {
                                defer_this_frame = true;
                                state.pending_invalidation_probe = true;
                                state.stable_frames = 0;
                            } else if stable_frames_required == 0 {
                                state.kind = ScrollDeferredUnboundedProbeKind::None;
                                state.stable_frames = 0;
                            } else {
                                state.stable_frames = state.stable_frames.saturating_add(1);
                                if state.stable_frames < stable_frames_required {
                                    defer_this_frame = true;
                                } else {
                                    state.kind = ScrollDeferredUnboundedProbeKind::None;
                                    state.stable_frames = 0;
                                }
                            }
                        }
                        ScrollDeferredUnboundedProbeKind::None => {
                            if should_defer_unbounded_probe_on_invalidation {
                                defer_this_frame = true;
                                state.kind = ScrollDeferredUnboundedProbeKind::Invalidation;
                                state.pending_invalidation_probe = true;
                                state.stable_frames = 0;
                            }
                        }
                    }
                }
                (*state, defer_this_frame)
            },
        );

        if defer_this_frame {
            let schedule_follow_up = match defer_state.kind {
                ScrollDeferredUnboundedProbeKind::Invalidation => true,
                ScrollDeferredUnboundedProbeKind::Resize => !viewport_changed,
                ScrollDeferredUnboundedProbeKind::None => false,
            };
            if schedule_follow_up {
                cx.tree.schedule_barrier_relayout_with_source_and_detail(
                    cx.node,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ScrollDeferredProbe,
                );
                cx.request_redraw();
            }
        }

        let must_probe_for_growing_extent = at_scroll_extent_edge
            && (children_layout_invalidated || defer_state.pending_invalidation_probe);
        if must_probe_for_growing_extent {
            cached_max_child = None;
        }

        // Avoid recomputing the unbounded scroll probe twice in a single frame when the runtime
        // performs probe+final layout passes (e.g. view-cache reconciliation).
        let key = ScrollLayoutProbeKey {
            avail_w: available_space_cache_key(child_constraints.available.width),
            avail_h: available_space_cache_key(child_constraints.available.height),
        };
        let frame_id = cx.app.frame_id();
        let (cached, last_max_child) = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollLayoutProbeCacheState::default,
            |state| {
                if state.frame_id != frame_id {
                    state.frame_id = frame_id;
                    state.entries.clear();
                }
                let cached = state
                    .entries
                    .iter()
                    .find_map(|(k, v)| (*k == key).then_some(*v));
                if let Some(cached) = cached {
                    state.last_max_child = cached;
                }
                (cached, state.last_max_child)
            },
        );

        let max_child = if must_probe_for_growing_extent {
            let measure_started = profile_cfg.is_some().then(Instant::now);
            let mut max_child = Size::new(Px(0.0), Px(0.0));
            for &child in cx.children {
                let child_size = cx.measure_in(child, child_constraints);
                max_child.width = Px(max_child.width.0.max(child_size.width.0));
                max_child.height = Px(max_child.height.0.max(child_size.height.0));
            }
            if let Some(started) = measure_started {
                t_measure_children = started.elapsed();
            }

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                ScrollLayoutProbeCacheState::default,
                |state| {
                    if state.frame_id != frame_id {
                        state.frame_id = frame_id;
                        state.entries.clear();
                    }
                    state.entries.push((key, max_child));
                    state.last_max_child = max_child;
                },
            );
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                ScrollDeferredUnboundedProbeState::default,
                |state| state.pending_invalidation_probe = false,
            );

            max_child
        } else if let Some(cached) = cached_max_child {
            cached
        } else if let Some(cached) = cached {
            cached
        } else if defer_this_frame {
            if last_max_child != Size::default() {
                // Best-effort: reuse the last measured max-child size while deferring the expensive
                // unbounded probe during interactive resize/unstable frames.
                //
                // Correctness note:
                //
                // When content shrinks (e.g. filtering a nav list) we must avoid pinning the scroll
                // extent to the previous frame's larger probe result, otherwise users can scroll
                // into blank space until the unbounded probe runs again.
                //
                // The layout pass below opportunistically observes the post-layout child bounds
                // and clamps the cached extent downward (when it can be proven smaller) without
                // performing an additional deep measure walk.
                last_max_child
            } else {
                // Fallback: if we have no cached max-child size yet, scan the last measured child
                // sizes and avoid a deep measure walk on this frame. Persist the result so future
                // deferred frames can reuse it without scanning.
                let mut max_child = Size::new(Px(0.0), Px(0.0));
                for &child in cx.children {
                    if let Some(child_size) = cx.tree.node_measured_size(child) {
                        max_child.width = Px(max_child.width.0.max(child_size.width.0));
                        max_child.height = Px(max_child.height.0.max(child_size.height.0));
                    }
                }
                if max_child != Size::default() {
                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        ScrollLayoutProbeCacheState::default,
                        |state| state.last_max_child = max_child,
                    );
                }
                max_child
            }
        } else if cx.tree.view_cache_enabled()
            && !must_probe_for_growing_extent
            && let Some(cached) = intrinsic_cached_max_child
            && cached != Size::default()
        {
            // Best-effort: reuse intrinsic sizing caches even when the child subtree is currently
            // marked `needs_layout`. This avoids deep unbounded probe walks on transient
            // invalidation frames (common under view-cache reconciliation).
            cached
        } else {
            let measure_started = profile_cfg.is_some().then(Instant::now);
            let mut max_child = Size::new(Px(0.0), Px(0.0));
            for &child in cx.children {
                let child_size = cx.measure_in(child, child_constraints);
                max_child.width = Px(max_child.width.0.max(child_size.width.0));
                max_child.height = Px(max_child.height.0.max(child_size.height.0));
            }
            if let Some(started) = measure_started {
                t_measure_children = started.elapsed();
            }

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                ScrollLayoutProbeCacheState::default,
                |state| {
                    if state.frame_id != frame_id {
                        state.frame_id = frame_id;
                        state.entries.clear();
                    }
                    state.entries.push((key, max_child));
                    state.last_max_child = max_child;
                },
            );
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                ScrollDeferredUnboundedProbeState::default,
                |state| state.pending_invalidation_probe = false,
            );

            max_child
        };

        // In unbounded probe flows, scroll surfaces frequently sit under auto-sized containers
        // (e.g. `max-height` shells). During intrinsic sizing, parents may pass
        // `available.{width,height} = 0` as a placeholder for "unknown".
        //
        // `clamp_to_constraints()` treats `available` as a hard upper bound even for `Auto`, so we
        // must avoid feeding a zero "unknown" available size into it. Use the measured content
        // size as an upper bound in that case so the scroll node can participate in intrinsic
        // sizing (similar to how percentage heights behave under `auto` in CSS).
        let mut clamp_available = available;
        if props.probe_unbounded {
            if clamp_available.width.0 <= 0.0 {
                clamp_available.width = Px(max_child.width.0.max(0.0));
            }
            if clamp_available.height.0 <= 0.0 {
                clamp_available.height = Px(max_child.height.0.max(0.0));
            }
        }
        let desired = clamp_to_constraints(max_child, props.layout, clamp_available);
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
        let mut content_w = Px(content_w.0.max(desired.width.0.max(0.0)));
        let mut content_h = Px(content_h.0.max(desired.height.0.max(0.0)));

        // Avoid mutating the imperative handle during "probe" layout passes that use an
        // effectively-unbounded available space, otherwise scroll position can be clamped to zero
        // prematurely.
        if !is_probe_layout {
            handle.set_viewport_size_internal(desired);
            handle.set_content_size_internal(Size::new(content_w, content_h));
            let prev = handle.offset();
            handle.set_offset_internal(prev);

            cx.tree
                .debug_record_scroll_node_telemetry(UiDebugScrollNodeTelemetry {
                    node: cx.node,
                    element: Some(self.element),
                    axis: match props.axis {
                        crate::element::ScrollAxis::X => UiDebugScrollAxis::X,
                        crate::element::ScrollAxis::Y => UiDebugScrollAxis::Y,
                        crate::element::ScrollAxis::Both => UiDebugScrollAxis::Both,
                    },
                    offset: handle.offset(),
                    viewport: handle.viewport_size(),
                    content: handle.content_size(),
                });

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::element::ScrollState::default,
                |state| {
                    if cx.children.len() == 1 {
                        state.intrinsic_measure_cache =
                            Some(crate::element::ScrollIntrinsicMeasureCache {
                                key: crate::element::ScrollIntrinsicMeasureCacheKey {
                                    avail_w: available_space_cache_key(
                                        child_constraints.available.width,
                                    ),
                                    avail_h: available_space_cache_key(
                                        child_constraints.available.height,
                                    ),
                                    axis: match props.axis {
                                        crate::element::ScrollAxis::X => 0,
                                        crate::element::ScrollAxis::Y => 1,
                                        crate::element::ScrollAxis::Both => 2,
                                    },
                                    probe_unbounded: props.probe_unbounded,
                                    scale_bits: cx.scale_factor.to_bits(),
                                },
                                max_child,
                            });
                    } else {
                        state.intrinsic_measure_cache = None;
                    }
                },
            );
        }

        self.scroll_child_transform = Some(super::super::ScrollChildTransform {
            handle: handle.clone(),
            axis: props.axis,
        });

        let content_bounds = Rect::new(cx.bounds.origin, Size::new(content_w, content_h));

        if !is_probe_layout {
            let solve_started = profile_cfg.is_some().then(Instant::now);
            match cx.children {
                [child] => {
                    cx.solve_barrier_child_root_if_needed(*child, content_bounds);
                }
                children => {
                    let roots: Vec<(NodeId, Rect)> =
                        children.iter().map(|&c| (c, content_bounds)).collect();
                    cx.solve_barrier_child_roots_if_needed(&roots);
                }
            }
            if let Some(started) = solve_started {
                t_solve_barrier = started.elapsed();
            }
        }

        let layout_started = profile_cfg.is_some().then(Instant::now);
        for &child in cx.children {
            let _ = cx.layout_in(child, content_bounds);
        }
        if let Some(started) = layout_started {
            t_layout_children = started.elapsed();
        }

        if !is_probe_layout {
            let offset = handle.offset();
            let offset_x = if props.axis.scroll_x() {
                offset.x.0.max(0.0)
            } else {
                0.0
            };
            let offset_y = if props.axis.scroll_y() {
                offset.y.0.max(0.0)
            } else {
                0.0
            };

            let mut observed = Size::new(Px(0.0), Px(0.0));
            for &child in cx.children {
                let Some(bounds) = cx.tree.node_bounds(child) else {
                    continue;
                };
                // `node_bounds` are expressed in the same coordinate space as hit-testing and
                // painting (including any scroll offset transforms). Convert back into
                // content-space extents by adding the current scroll offset along scroll axes.
                let right = (bounds.origin.x.0 + bounds.size.width.0 - content_bounds.origin.x.0)
                    .max(0.0)
                    + offset_x;
                let bottom = (bounds.origin.y.0 + bounds.size.height.0 - content_bounds.origin.y.0)
                    .max(0.0)
                    + offset_y;
                observed.width = Px(observed.width.0.max(right));
                observed.height = Px(observed.height.0.max(bottom));
            }

            // Best-effort: if post-layout child bounds exceed the probed extent (cached/deferral
            // cases), expand the scroll handle immediately so users can reach the new content.
            let mut changed_grow = false;
            if props.axis.scroll_x()
                && observed.width.0 > 0.0
                && observed.width.0 > content_w.0 + 0.5
            {
                content_w = Px(observed.width.0.max(desired.width.0.max(0.0)));
                changed_grow = true;
            }
            if props.axis.scroll_y()
                && observed.height.0 > 0.0
                && observed.height.0 > content_h.0 + 0.5
            {
                content_h = Px(observed.height.0.max(desired.height.0.max(0.0)));
                changed_grow = true;
            }
            if changed_grow {
                handle.set_content_size_internal(Size::new(content_w, content_h));
                let prev = handle.offset();
                handle.set_offset_internal(prev);

                cx.tree
                    .debug_record_scroll_node_telemetry(UiDebugScrollNodeTelemetry {
                        node: cx.node,
                        element: Some(self.element),
                        axis: match props.axis {
                            crate::element::ScrollAxis::X => UiDebugScrollAxis::X,
                            crate::element::ScrollAxis::Y => UiDebugScrollAxis::Y,
                            crate::element::ScrollAxis::Both => UiDebugScrollAxis::Both,
                        },
                        offset: handle.offset(),
                        viewport: handle.viewport_size(),
                        content: handle.content_size(),
                    });

                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    ScrollLayoutProbeCacheState::default,
                    |state| state.last_max_child = Size::new(content_w, content_h),
                );

                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        if cx.children.len() == 1 {
                            state.intrinsic_measure_cache =
                                Some(crate::element::ScrollIntrinsicMeasureCache {
                                    key: crate::element::ScrollIntrinsicMeasureCacheKey {
                                        avail_w: available_space_cache_key(
                                            child_constraints.available.width,
                                        ),
                                        avail_h: available_space_cache_key(
                                            child_constraints.available.height,
                                        ),
                                        axis: match props.axis {
                                            crate::element::ScrollAxis::X => 0,
                                            crate::element::ScrollAxis::Y => 1,
                                            crate::element::ScrollAxis::Both => 2,
                                        },
                                        probe_unbounded: props.probe_unbounded,
                                        scale_bits: cx.scale_factor.to_bits(),
                                    },
                                    max_child: Size::new(content_w, content_h),
                                });
                        }
                    },
                );
            }

            if defer_this_frame {
                // In deferred-probe mode, the cached `last_max_child` can temporarily overestimate
                // the true scroll extent after content shrinks. Clamp down when possible without
                // triggering an extra deep measurement walk.
                let mut changed = false;
                if props.axis.scroll_x()
                    && observed.width.0 > 0.0
                    && observed.width.0 + 0.5 < content_w.0
                {
                    content_w = Px(observed.width.0.max(desired.width.0.max(0.0)));
                    changed = true;
                }
                if props.axis.scroll_y()
                    && observed.height.0 > 0.0
                    && observed.height.0 + 0.5 < content_h.0
                {
                    content_h = Px(observed.height.0.max(desired.height.0.max(0.0)));
                    changed = true;
                }

                if changed {
                    handle.set_content_size_internal(Size::new(content_w, content_h));
                    let prev = handle.offset();
                    handle.set_offset_internal(prev);

                    cx.tree
                        .debug_record_scroll_node_telemetry(UiDebugScrollNodeTelemetry {
                            node: cx.node,
                            element: Some(self.element),
                            axis: match props.axis {
                                crate::element::ScrollAxis::X => UiDebugScrollAxis::X,
                                crate::element::ScrollAxis::Y => UiDebugScrollAxis::Y,
                                crate::element::ScrollAxis::Both => UiDebugScrollAxis::Both,
                            },
                            offset: handle.offset(),
                            viewport: handle.viewport_size(),
                            content: handle.content_size(),
                        });

                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        ScrollLayoutProbeCacheState::default,
                        |state| state.last_max_child = Size::new(content_w, content_h),
                    );

                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::ScrollState::default,
                        |state| {
                            if cx.children.len() == 1 {
                                state.intrinsic_measure_cache =
                                    Some(crate::element::ScrollIntrinsicMeasureCache {
                                        key: crate::element::ScrollIntrinsicMeasureCacheKey {
                                            avail_w: available_space_cache_key(
                                                child_constraints.available.width,
                                            ),
                                            avail_h: available_space_cache_key(
                                                child_constraints.available.height,
                                            ),
                                            axis: match props.axis {
                                                crate::element::ScrollAxis::X => 0,
                                                crate::element::ScrollAxis::Y => 1,
                                                crate::element::ScrollAxis::Both => 2,
                                            },
                                            probe_unbounded: props.probe_unbounded,
                                            scale_bits: cx.scale_factor.to_bits(),
                                        },
                                        max_child: Size::new(content_w, content_h),
                                    });
                            }
                        },
                    );
                }
            }
        }

        if let Some(cfg) = profile_cfg
            && let Some(started) = profile_started
        {
            let total = started.elapsed();
            if total >= cfg.min_elapsed && t_measure_children >= cfg.min_self_measure {
                let element_path: Option<String> = {
                    #[cfg(feature = "diagnostics")]
                    {
                        Some(crate::elements::with_window_state(
                            &mut *cx.app,
                            window,
                            |st| {
                                st.debug_path_for_element(self.element)
                                    .unwrap_or_else(|| "<unknown>".to_string())
                            },
                        ))
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    {
                        None
                    }
                };

                tracing::info!(
                    window = ?cx.window,
                    node = ?cx.node,
                    element = self.element.0,
                    pass = ?cx.pass_kind,
                    axis = ?props.axis,
                    probe_unbounded = props.probe_unbounded,
                    children = cx.children.len(),
                    available_w = cx.available.width.0,
                    available_h = cx.available.height.0,
                    desired_w = desired.width.0,
                    desired_h = desired.height.0,
                    content_w = content_w.0,
                    content_h = content_h.0,
                    measure_children_us = t_measure_children.as_micros() as u64,
                    solve_barrier_us = t_solve_barrier.as_micros() as u64,
                    layout_children_us = t_layout_children.as_micros() as u64,
                    total_us = total.as_micros() as u64,
                    element_path = element_path.as_deref().unwrap_or("<unknown>"),
                    "scroll layout profile"
                );
            }
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
