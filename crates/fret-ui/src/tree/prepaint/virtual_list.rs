use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn prepaint_virtual_list_window_from_interaction_record(
        &mut self,
        app: &mut H,
        record: &InteractionRecord,
    ) {
        let Some(window) = self.window else {
            return;
        };
        if !record.can_scroll_descendant_into_view {
            return;
        }

        let Some(inputs) =
            crate::declarative::frame::element_record_for_node(&mut *app, window, record.node)
                .and_then(|element_record| match &element_record.instance {
                    crate::declarative::frame::ElementInstance::VirtualList(props) => {
                        Some(VirtualListPrepaintInputs {
                            element: element_record.element,
                            axis: props.axis,
                            len: props.len,
                            items_revision: props.items_revision,
                            measure_mode: props.measure_mode,
                            overscan: props.overscan,
                            estimate_row_height: props.estimate_row_height,
                            gap: props.gap,
                            scroll_margin: props.scroll_margin,
                            scroll_handle: props.scroll_handle.clone(),
                        })
                    }
                    _ => None,
                })
        else {
            return;
        };

        let viewport = match inputs.axis {
            fret_core::Axis::Vertical => Px(record.bounds.size.height.0.max(0.0)),
            fret_core::Axis::Horizontal => Px(record.bounds.size.width.0.max(0.0)),
        };
        if viewport.0 <= 0.0 || inputs.len == 0 {
            return;
        }

        let offset_point = inputs.scroll_handle.offset();
        let offset_axis = match inputs.axis {
            fret_core::Axis::Vertical => offset_point.y,
            fret_core::Axis::Horizontal => offset_point.x,
        };
        let deferred_scroll_to_item = inputs.scroll_handle.deferred_scroll_to_item().is_some()
            || inputs
                .scroll_handle
                .scroll_to_item_consumed_in_frame(app.frame_id());
        let retained_host = crate::elements::with_window_state(&mut *app, window, |window_state| {
            window_state.has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                inputs.element,
            )
        });

        let view_cache_active = self.view_cache_active();
        let update = crate::elements::with_element_state(
            &mut *app,
            window,
            inputs.element,
            crate::element::VirtualListState::default,
            |state| {
                let prev_items_revision = state.items_revision;
                let prev_viewport = match inputs.axis {
                    fret_core::Axis::Vertical => state.viewport_h,
                    fret_core::Axis::Horizontal => state.viewport_w,
                };
                let prev_offset = match inputs.axis {
                    fret_core::Axis::Vertical => state.offset_y,
                    fret_core::Axis::Horizontal => state.offset_x,
                };
                let prev_window_range = state.window_range;
                let render_window_range = state.render_window_range;

                state.metrics.ensure_with_mode(
                    inputs.measure_mode,
                    inputs.len,
                    inputs.estimate_row_height,
                    inputs.gap,
                    inputs.scroll_margin,
                );
                state.items_revision = inputs.items_revision;

                let content_extent = state.metrics.total_height();
                let offset_axis = state.metrics.clamp_offset(offset_axis, viewport);
                match inputs.axis {
                    fret_core::Axis::Vertical => {
                        state.offset_y = offset_axis;
                        state.viewport_h = viewport;
                    }
                    fret_core::Axis::Horizontal => {
                        state.offset_x = offset_axis;
                        state.viewport_w = viewport;
                    }
                }
                if viewport.0 > 0.0 {
                    state.has_final_viewport = true;
                }

                let visible_range = state.metrics.visible_range(offset_axis, viewport, 0);
                let ideal_window_range =
                    state
                        .metrics
                        .visible_range(offset_axis, viewport, inputs.overscan);

                let window_mismatch = if let Some(visible) = visible_range {
                    match render_window_range.filter(|r| {
                        r.count == inputs.len
                            && r.overscan == inputs.overscan
                            && r.start_index <= r.end_index
                            && r.end_index < r.count
                    }) {
                        None => {
                            // Without a render-derived window, the current declarative subtree may
                            // not reflect the post-layout visible range. Treat this as a mismatch
                            // so prepaint can schedule the initial one-shot rerender (Track B) or
                            // retained-host reconcile (Track A).
                            true
                        }
                        Some(rendered) => {
                            let rendered_start =
                                rendered.start_index.saturating_sub(rendered.overscan);
                            let rendered_end = (rendered.end_index + rendered.overscan)
                                .min(rendered.count.saturating_sub(1));
                            visible.start_index < rendered_start || visible.end_index > rendered_end
                        }
                    }
                } else {
                    false
                };

                let mut window_shift_kind = crate::tree::UiDebugVirtualListWindowShiftKind::None;
                let allow_preemptive_prefetch = retained_host || !view_cache_active;
                let window_range = if window_mismatch {
                    window_shift_kind = crate::tree::UiDebugVirtualListWindowShiftKind::Escape;
                    if retained_host {
                        match (render_window_range, visible_range) {
                            (Some(rendered), Some(visible))
                                if rendered.count == inputs.len
                                    && rendered.overscan == inputs.overscan
                                    && rendered.start_index <= rendered.end_index
                                    && rendered.end_index < rendered.count =>
                            {
                                Some(crate::virtual_list::shift_virtual_range_minimally(
                                    rendered, visible,
                                ))
                            }
                            _ => ideal_window_range,
                        }
                    } else {
                        ideal_window_range
                    }
                } else if inputs.overscan > 0 && !deferred_scroll_to_item {
                    match (render_window_range, visible_range) {
                        (Some(rendered), Some(visible))
                            if rendered.count == inputs.len
                                && rendered.overscan == inputs.overscan
                                && rendered.start_index <= rendered.end_index
                                && rendered.end_index < rendered.count =>
                        {
                            let forced_prefetch = if let Some(desired) = ideal_window_range {
                                let rendered_visible_len = rendered
                                    .end_index
                                    .saturating_sub(rendered.start_index)
                                    .saturating_add(1);
                                let visible_len = visible
                                    .end_index
                                    .saturating_sub(visible.start_index)
                                    .saturating_add(1);

                                // If the render-derived window was computed under a smaller
                                // viewport (e.g. during intrinsic probes), its visible span may
                                // be shorter than the current visible span. While we can still
                                // be within the rendered overscan envelope, we should stage a
                                // one-shot prefetch to the ideal window so the next frame can
                                // rebuild the correct visible-items set.
                                if visible_len > rendered_visible_len {
                                    window_shift_kind =
                                        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch;
                                    // For the non-retained VirtualList path, we will schedule a cache
                                    // root rerender on prefetch so the next frame can rebuild
                                    // `visible_items` against the updated window. Clear the
                                    // render-derived window to ensure the rerender consumes the
                                    // prepaint-derived window.
                                    if !retained_host && view_cache_active {
                                        state.render_window_range = None;
                                    }
                                    Some(desired)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(prefetch) = forced_prefetch {
                                Some(prefetch)
                            } else if !allow_preemptive_prefetch {
                                // For non-retained + view-cache hosts, prefetch shifts translate
                                // directly into cache-root rerenders. Keep the current rendered
                                // window while it still covers the visible range, and rely on
                                // escape-driven shifts when we truly leave overscan.
                                ideal_window_range
                            } else {
                                let prefetch_margin = (inputs.overscan / 6).max(1);
                                // Shift by a slightly larger step than the “near-edge” margin so we
                                // don’t prefetch on every frame during slow scroll, while still keeping
                                // each prefetch rerender bounded.
                                let prefetch_step = if retained_host {
                                    // Retained hosts (ADR 0177) can apply window shifts via reconcile
                                    // without rerendering a parent cache root, so prefer smaller, more
                                    // frequent prefetches. This bounds single-frame attach/detach bursts.
                                    //
                                    // Note: shifting the window by `step` can detach ~`step` items and
                                    // attach ~`step` items, so the attach+detach delta scales like
                                    // `2*step` (see `--check-retained-vlist-attach-detach-max`).
                                    (inputs.overscan / 2).clamp(1, RETAINED_HOST_PREFETCH_STEP_MAX)
                                } else {
                                    // Non-retained VirtualList pays a full cache-root rerender per shift,
                                    // so prefer fewer, larger shifts while we still have overscan coverage.
                                    //
                                    // This is a deliberate trade-off:
                                    // - larger steps reduce how often we schedule a rerender during smooth scroll,
                                    // - but each rerender may rebuild more rows.
                                    //
                                    // For v1 (non-retained), reducing rerender frequency generally wins because
                                    // we cannot attach/detach rows on cache-hit frames without a retained host
                                    // boundary (ADR 0177).
                                    inputs.overscan.saturating_mul(8)
                                }
                                .max(prefetch_margin)
                                .max(1);
                                let prefer_forward = if offset_axis.0 > prev_offset.0 + 0.01 {
                                    state.last_scroll_direction_forward = Some(true);
                                    Some(true)
                                } else if offset_axis.0 + 0.01 < prev_offset.0 {
                                    state.last_scroll_direction_forward = Some(false);
                                    Some(false)
                                } else {
                                    state.last_scroll_direction_forward
                                };
                                if let Some(prefetch) =
                                    crate::virtual_list::prefetch_virtual_range_step(
                                        rendered,
                                        visible,
                                        prefetch_margin,
                                        prefetch_step,
                                        prefer_forward,
                                    )
                                {
                                    window_shift_kind =
                                        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch;
                                    // For the non-retained VirtualList path, we will schedule a cache
                                    // root rerender on prefetch so the next frame can rebuild
                                    // `visible_items` against the prefetched window. Clear the
                                    // render-derived window to ensure the rerender consumes the
                                    // prepaint-derived window.
                                    if !retained_host && view_cache_active {
                                        state.render_window_range = None;
                                    }
                                    Some(prefetch)
                                } else {
                                    ideal_window_range
                                }
                            }
                        }
                        _ => ideal_window_range,
                    }
                } else {
                    ideal_window_range
                };
                state.window_range = window_range;

                VirtualListPrepaintWindowUpdate {
                    prev_items_revision,
                    prev_viewport,
                    prev_offset,
                    prev_window_range,
                    render_window_range,
                    window_range,
                    viewport,
                    offset: offset_axis,
                    deferred_scroll_to_item,
                    window_mismatch,
                    window_shift_kind,
                    content_extent,
                }
            },
        );

        inputs
            .scroll_handle
            .set_viewport_size_internal(record.bounds.size);
        let content_size = match inputs.axis {
            fret_core::Axis::Vertical => Size::new(record.bounds.size.width, update.content_extent),
            fret_core::Axis::Horizontal => {
                Size::new(update.content_extent, record.bounds.size.height)
            }
        };
        inputs.scroll_handle.set_content_size_internal(content_size);
        let prev_offset_point = inputs.scroll_handle.offset();
        match inputs.axis {
            fret_core::Axis::Vertical => {
                inputs
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(prev_offset_point.x, update.offset));
            }
            fret_core::Axis::Horizontal => {
                inputs
                    .scroll_handle
                    .set_offset_internal(fret_core::Point::new(update.offset, prev_offset_point.y));
            }
        }

        if self.debug_enabled {
            let policy_key = {
                let mut b = CacheKeyBuilder::new();
                b.write_u32(inputs.axis as u32);
                b.write_u32(inputs.measure_mode as u32);
                b.write_u64(inputs.overscan as u64);
                b.write_px(inputs.estimate_row_height);
                b.write_px(inputs.gap);
                b.write_px(inputs.scroll_margin);
                b.finish()
            };
            let inputs_key = {
                let mut b = CacheKeyBuilder::new();
                b.write_u64(policy_key);
                b.write_u64(inputs.len as u64);
                b.write_u64(inputs.items_revision);
                b.write_px(update.viewport);
                b.write_px(update.offset);
                b.write_px(update.content_extent);
                b.finish()
            };
            let (window_shift_reason, window_shift_apply_mode, window_shift_invalidation_detail) =
                if update.window_shift_kind != crate::tree::UiDebugVirtualListWindowShiftKind::None
                {
                    let reason = if update.deferred_scroll_to_item {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem
                    } else if inputs.items_revision != update.prev_items_revision {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision
                    } else if (update.viewport.0 - update.prev_viewport.0).abs() > 0.01 {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize
                    } else if (update.offset.0 - update.prev_offset.0).abs() > 0.01 {
                        crate::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset
                    } else if update.prev_window_range.map(|r| (r.count, r.overscan))
                        != update.window_range.map(|r| (r.count, r.overscan))
                    {
                        crate::tree::UiDebugVirtualListWindowShiftReason::InputsChange
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftReason::Unknown
                    };
                    let mode = if retained_host {
                        crate::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile
                    } else {
                        crate::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender
                    };
                    let invalidation_detail = if self.view_cache_active() && !retained_host {
                        match reason {
                            crate::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => Some(
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate,
                            ),
                            crate::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => Some(
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate,
                            ),
                            crate::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => Some(
                                crate::tree::UiDebugInvalidationDetail::ScrollHandleItemsRevisionWindowUpdate,
                            ),
                            _ => match update.window_shift_kind {
                                crate::tree::UiDebugVirtualListWindowShiftKind::None => None,
                                crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch => Some(
                                    crate::tree::UiDebugInvalidationDetail::ScrollHandlePrefetchWindowUpdate,
                                ),
                                crate::tree::UiDebugVirtualListWindowShiftKind::Escape => Some(
                                    crate::tree::UiDebugInvalidationDetail::ScrollHandleWindowUpdate,
                                ),
                            },
                        }
                    } else {
                        None
                    };
                    (Some(reason), Some(mode), invalidation_detail)
                } else {
                    (None, None, None)
                };

            if update.window_shift_kind != crate::tree::UiDebugVirtualListWindowShiftKind::None {
                self.debug_record_prepaint_action(crate::tree::UiDebugPrepaintAction {
                    node: record.node,
                    target: None,
                    kind: crate::tree::UiDebugPrepaintActionKind::VirtualListWindowShift,
                    invalidation: None,
                    element: Some(inputs.element),
                    virtual_list_window_shift_kind: Some(update.window_shift_kind),
                    virtual_list_window_shift_reason: window_shift_reason,
                    chart_sampling_window_key: None,
                    node_graph_cull_window_key: None,
                    frame_id: app.frame_id(),
                });
            }
            self.debug_record_virtual_list_window(crate::tree::UiDebugVirtualListWindow {
                source: crate::tree::UiDebugVirtualListWindowSource::Prepaint,
                node: record.node,
                element: inputs.element,
                axis: inputs.axis,
                is_probe_layout: false,
                items_len: inputs.len,
                items_revision: inputs.items_revision,
                prev_items_revision: update.prev_items_revision,
                measure_mode: inputs.measure_mode,
                overscan: inputs.overscan,
                estimate_row_height: inputs.estimate_row_height,
                gap: inputs.gap,
                scroll_margin: inputs.scroll_margin,
                viewport: update.viewport,
                prev_viewport: update.prev_viewport,
                offset: update.offset,
                prev_offset: update.prev_offset,
                content_extent: update.content_extent,
                policy_key,
                inputs_key,
                window_range: update.window_range,
                prev_window_range: update.prev_window_range,
                render_window_range: update.render_window_range,
                deferred_scroll_to_item: update.deferred_scroll_to_item,
                deferred_scroll_consumed: false,
                window_mismatch: update.window_mismatch,
                window_shift_kind: update.window_shift_kind,
                window_shift_reason,
                window_shift_apply_mode,
                window_shift_invalidation_detail,
            });
        }

        if self.view_cache_active()
            && update.window_shift_kind != crate::tree::UiDebugVirtualListWindowShiftKind::None
        {
            if retained_host {
                let kind = match update.window_shift_kind {
                    crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch => {
                        crate::tree::UiDebugRetainedVirtualListReconcileKind::Prefetch
                    }
                    crate::tree::UiDebugVirtualListWindowShiftKind::Escape => {
                        crate::tree::UiDebugRetainedVirtualListReconcileKind::Escape
                    }
                    crate::tree::UiDebugVirtualListWindowShiftKind::None => {
                        unreachable!("window_shift_kind checked above")
                    }
                };
                crate::elements::with_window_state(&mut *app, window, |window_state| {
                    window_state.mark_retained_virtual_list_needs_reconcile(inputs.element, kind);
                });
                self.request_redraw_coalesced(app);
            } else {
                let detail = match update.window_shift_kind {
                    crate::tree::UiDebugVirtualListWindowShiftKind::None => {
                        unreachable!("window_shift_kind checked above")
                    }
                    crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch => {
                        UiDebugInvalidationDetail::ScrollHandlePrefetchWindowUpdate
                    }
                    crate::tree::UiDebugVirtualListWindowShiftKind::Escape => {
                        UiDebugInvalidationDetail::ScrollHandleWindowUpdate
                    }
                };
                self.mark_nearest_view_cache_root_needs_rerender(
                    record.node,
                    UiDebugInvalidationSource::Other,
                    if update.deferred_scroll_to_item {
                        UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate
                    } else {
                        detail
                    },
                );
                self.request_redraw_coalesced(app);
            }
        }
    }
}
