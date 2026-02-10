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
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ScrollLayoutProfileConfig {
    min_elapsed: Duration,
    min_self_measure: Duration,
}

impl ScrollLayoutProfileConfig {
    fn from_env() -> Option<Self> {
        let enabled = std::env::var("FRET_SCROLL_LAYOUT_PROFILE")
            .ok()
            .is_some_and(|v| v == "1");
        if !enabled {
            return None;
        }

        let min_us = std::env::var("FRET_SCROLL_LAYOUT_PROFILE_MIN_US")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(2_000);
        let min_measure_us = std::env::var("FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1_000);

        Some(Self {
            min_elapsed: Duration::from_micros(min_us),
            min_self_measure: Duration::from_micros(min_measure_us),
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
}

fn available_space_cache_key(space: AvailableSpace) -> u64 {
    match space {
        AvailableSpace::Definite(px) => px.0.to_bits() as u64,
        AvailableSpace::MinContent => 1 << 62,
        AvailableSpace::MaxContent => 2 << 62,
    }
}

fn scroll_defer_unbounded_probe_on_resize_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        // Default-on for interactive resize/viewport churn. Set to "0" to disable.
        std::env::var("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_RESIZE")
            .ok()
            .is_none_or(|v| v != "0")
    })
}

fn scroll_defer_unbounded_probe_on_invalidation_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION")
            .is_some_and(|v| !v.is_empty())
    })
}

fn scroll_defer_unbounded_probe_stable_frames() -> u8 {
    static STABLE_FRAMES: OnceLock<u8> = OnceLock::new();
    *STABLE_FRAMES.get_or_init(|| {
        std::env::var("FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(2)
            .min(60)
    })
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
            props
                .scroll_handle
                .clear_deferred_scroll_to_item(cx.app.frame_id());
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

        // Window-boundary invalidation under view-cache is prepaint-driven (ADR 0190):
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
        let handle = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::element::ScrollState::default,
            |state| {
                external_handle
                    .as_ref()
                    .unwrap_or(&state.scroll_handle)
                    .clone()
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

        let mut cached_max_child: Option<Size> = None;
        if !is_probe_layout && cx.children.len() == 1 {
            let child = cx.children[0];
            if !cx.tree.node_needs_layout(child) {
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

                cached_max_child = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        state
                            .intrinsic_measure_cache
                            .and_then(|cache| (cache.key == cache_key).then_some(cache.max_child))
                    },
                );
            }
        }

        let wants_unbounded_probe = props.probe_unbounded
            && (props.axis.scroll_x() || props.axis.scroll_y())
            && !is_probe_layout;
        let defer_probe_on_resize = scroll_defer_unbounded_probe_on_resize_enabled();
        let defer_probe_on_invalidation = scroll_defer_unbounded_probe_on_invalidation_enabled();
        let prev_viewport = handle.viewport_size();
        let viewport_changed = prev_viewport.width.0 > 0.0
            && prev_viewport.height.0 > 0.0
            && (prev_viewport.width.0.to_bits() != available.width.0.to_bits()
                || prev_viewport.height.0.to_bits() != available.height.0.to_bits());

        let should_defer_unbounded_probe_on_resize =
            wants_unbounded_probe && defer_probe_on_resize && viewport_changed;
        let should_defer_unbounded_probe_on_invalidation = wants_unbounded_probe
            && defer_probe_on_invalidation
            && cx
                .children
                .iter()
                .copied()
                .any(|child| cx.tree.node_layout_invalidated(child));

        let mut defer_state = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollDeferredUnboundedProbeState::default,
            |state| *state,
        );

        let stable_frames_required = scroll_defer_unbounded_probe_stable_frames();
        let mut defer_this_frame = false;
        if should_defer_unbounded_probe_on_resize {
            defer_this_frame = true;
            defer_state.kind = ScrollDeferredUnboundedProbeKind::Resize;
            defer_state.stable_frames = 0;
        } else {
            match defer_state.kind {
                ScrollDeferredUnboundedProbeKind::Resize => {
                    if stable_frames_required == 0 {
                        defer_state.kind = ScrollDeferredUnboundedProbeKind::None;
                        defer_state.stable_frames = 0;
                    } else {
                        defer_state.stable_frames = defer_state.stable_frames.saturating_add(1);
                        if defer_state.stable_frames < stable_frames_required {
                            defer_this_frame = true;
                        } else {
                            defer_state.kind = ScrollDeferredUnboundedProbeKind::None;
                            defer_state.stable_frames = 0;
                        }
                    }
                }
                ScrollDeferredUnboundedProbeKind::Invalidation => {
                    // Consume the pending deferral by running the unbounded probe on this frame.
                    defer_state.kind = ScrollDeferredUnboundedProbeKind::None;
                    defer_state.stable_frames = 0;
                }
                ScrollDeferredUnboundedProbeKind::None => {
                    if should_defer_unbounded_probe_on_invalidation {
                        defer_this_frame = true;
                        defer_state.kind = ScrollDeferredUnboundedProbeKind::Invalidation;
                        defer_state.stable_frames = 0;
                    }
                }
            }
        }

        crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollDeferredUnboundedProbeState::default,
            |state| *state = defer_state,
        );

        if defer_this_frame {
            let schedule_follow_up = match defer_state.kind {
                ScrollDeferredUnboundedProbeKind::Invalidation => true,
                ScrollDeferredUnboundedProbeKind::Resize => !viewport_changed,
                ScrollDeferredUnboundedProbeKind::None => false,
            };
            if schedule_follow_up {
                cx.tree.invalidate_with_source_and_detail(
                    cx.node,
                    Invalidation::Layout,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ScrollDeferredProbe,
                );
                cx.request_redraw();
            }
        }

        // Avoid recomputing the unbounded scroll probe twice in a single frame when the runtime
        // performs probe+final layout passes (e.g. view-cache reconciliation).
        let key = ScrollLayoutProbeKey {
            avail_w: available_space_cache_key(child_constraints.available.width),
            avail_h: available_space_cache_key(child_constraints.available.height),
        };
        let frame_id = cx.app.frame_id();
        let cached = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            ScrollLayoutProbeCacheState::default,
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

        let max_child = if let Some(cached) = cached_max_child {
            cached
        } else if let Some(cached) = cached {
            cached
        } else if defer_this_frame {
            // Use the previous measured size as a best-effort estimate and avoid a deep measure
            // walk on this frame.
            let mut max_child = Size::new(Px(0.0), Px(0.0));
            for &child in cx.children {
                if let Some(child_size) = cx.tree.node_measured_size(child) {
                    max_child.width = Px(max_child.width.0.max(child_size.width.0));
                    max_child.height = Px(max_child.height.0.max(child_size.height.0));
                }
            }
            max_child
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
                },
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
        let content_w = Px(content_w.0.max(desired.width.0.max(0.0)));
        let content_h = Px(content_h.0.max(desired.height.0.max(0.0)));

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
            let roots: Vec<(NodeId, Rect)> =
                cx.children.iter().map(|&c| (c, content_bounds)).collect();
            cx.solve_barrier_child_roots_if_needed(&roots);
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
