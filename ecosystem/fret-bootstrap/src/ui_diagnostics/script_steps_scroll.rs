use super::*;

const SCROLL_INTO_VIEW_VISIBILITY_EPS_PX: f32 = 0.5;
const SCROLL_INTO_VIEW_PROGRESS_EPS_PX: f32 = 0.5;

fn rect_intersection(a: Rect, b: Rect) -> Option<Rect> {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0.max(0.0);
    let ay1 = ay0 + a.size.height.0.max(0.0);

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0.max(0.0);
    let by1 = by0 + b.size.height.0.max(0.0);

    let ix0 = ax0.max(bx0);
    let iy0 = ay0.max(by0);
    let ix1 = ax1.min(bx1);
    let iy1 = ay1.min(by1);

    if ix1 <= ix0 || iy1 <= iy0 {
        return None;
    }

    Some(Rect {
        origin: Point::new(Px(ix0), Px(iy0)),
        size: fret_core::Size {
            width: Px(ix1 - ix0),
            height: Px(iy1 - iy0),
        },
    })
}

fn rect_fully_contains_with_epsilon(outer: Rect, inner: Rect, eps_px: f32) -> bool {
    let eps = eps_px.max(0.0);
    let ox0 = outer.origin.x.0;
    let oy0 = outer.origin.y.0;
    let ox1 = ox0 + outer.size.width.0.max(0.0);
    let oy1 = oy0 + outer.size.height.0.max(0.0);

    let ix0 = inner.origin.x.0;
    let iy0 = inner.origin.y.0;
    let ix1 = ix0 + inner.size.width.0.max(0.0);
    let iy1 = iy0 + inner.size.height.0.max(0.0);

    ix0 + eps >= ox0 && iy0 + eps >= oy0 && ix1 <= ox1 + eps && iy1 <= oy1 + eps
}

fn rects_close_with_epsilon(a: Rect, b: Rect, eps_px: f32) -> bool {
    let eps = eps_px.max(0.0);
    (a.origin.x.0 - b.origin.x.0).abs() <= eps
        && (a.origin.y.0 - b.origin.y.0).abs() <= eps
        && (a.size.width.0 - b.size.width.0).abs() <= eps
        && (a.size.height.0 - b.size.height.0).abs() <= eps
}

fn rect_needs_axis_scroll_to_fully_contain(
    outer: Rect,
    inner: Rect,
    horizontal: bool,
    eps_px: f32,
) -> bool {
    let eps = eps_px.max(0.0);
    if horizontal {
        let ox0 = outer.origin.x.0;
        let ox1 = ox0 + outer.size.width.0.max(0.0);
        let ix0 = inner.origin.x.0;
        let ix1 = ix0 + inner.size.width.0.max(0.0);
        ix0 + eps < ox0 || ix1 > ox1 + eps
    } else {
        let oy0 = outer.origin.y.0;
        let oy1 = oy0 + outer.size.height.0.max(0.0);
        let iy0 = inner.origin.y.0;
        let iy1 = iy0 + inner.size.height.0.max(0.0);
        iy0 + eps < oy0 || iy1 > oy1 + eps
    }
}

fn scroll_into_view_unscrollable_axis_reason(
    window_bounds: Rect,
    container_bounds: Option<Rect>,
    target_bounds: Option<Rect>,
    require_fully_within_container: bool,
    require_fully_within_window: bool,
    delta_x: f32,
    delta_y: f32,
) -> Option<&'static str> {
    let target_bounds = target_bounds?;
    let can_scroll_x = delta_x.abs() > 0.01;
    let can_scroll_y = delta_y.abs() > 0.01;

    if require_fully_within_container && let Some(container_bounds) = container_bounds {
        if !can_scroll_x
            && rect_needs_axis_scroll_to_fully_contain(
                container_bounds,
                target_bounds,
                true,
                SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
            )
        {
            return Some("scroll_into_view_impossible_unscrollable_x_for_container");
        }
        if !can_scroll_y
            && rect_needs_axis_scroll_to_fully_contain(
                container_bounds,
                target_bounds,
                false,
                SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
            )
        {
            return Some("scroll_into_view_impossible_unscrollable_y_for_container");
        }
    }

    if require_fully_within_window {
        if !can_scroll_x
            && rect_needs_axis_scroll_to_fully_contain(
                window_bounds,
                target_bounds,
                true,
                SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
            )
        {
            return Some("scroll_into_view_impossible_unscrollable_x_for_window");
        }
        if !can_scroll_y
            && rect_needs_axis_scroll_to_fully_contain(
                window_bounds,
                target_bounds,
                false,
                SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
            )
        {
            return Some("scroll_into_view_impossible_unscrollable_y_for_window");
        }
    }

    None
}

fn scroll_into_view_visibility_satisfied(
    window_bounds: Rect,
    container_bounds: Option<Rect>,
    target_bounds: Option<Rect>,
    require_fully_within_container: bool,
    require_fully_within_window: bool,
) -> bool {
    let visible_ok = target_bounds.is_some_and(|bounds| {
        if require_fully_within_window {
            rect_fully_contains_with_epsilon(
                window_bounds,
                bounds,
                SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
            )
        } else {
            rect_intersection(bounds, window_bounds).is_some()
        }
    });
    let container_ok = if require_fully_within_container {
        container_bounds
            .zip(target_bounds)
            .is_some_and(|(container_bounds, target_bounds)| {
                rect_fully_contains_with_epsilon(
                    container_bounds,
                    target_bounds,
                    SCROLL_INTO_VIEW_VISIBILITY_EPS_PX,
                )
            })
    } else {
        true
    };

    visible_ok && container_ok
}

fn bounded_scroll_delta_for_axis(
    configured_delta: f32,
    target_start: f32,
    target_end: f32,
    preferred_start: f32,
    preferred_end: f32,
    fallback_start: f32,
    fallback_end: f32,
) -> f32 {
    let max_step = configured_delta.abs();
    if max_step <= 0.01 {
        return 0.0;
    }

    let target_len = (target_end - target_start).max(0.0);
    let preferred_len = (preferred_end - preferred_start).max(0.0);
    let (visible_start, visible_end) = if target_len <= preferred_len + 0.5 {
        (preferred_start, preferred_end)
    } else {
        (fallback_start, fallback_end)
    };

    if target_start < visible_start {
        (visible_start - target_start).min(max_step)
    } else if target_end > visible_end {
        -(target_end - visible_end).min(max_step)
    } else {
        0.0
    }
}

fn ui_rect_from_rect(rect: Rect) -> UiRectV1 {
    UiRectV1 {
        x_px: rect.origin.x.0,
        y_px: rect.origin.y.0,
        w_px: rect.size.width.0,
        h_px: rect.size.height.0,
    }
}

fn sign_with_epsilon_f32(value: f32, eps: f32) -> Option<i8> {
    if value > eps {
        Some(1)
    } else if value < -eps {
        Some(-1)
    } else {
        None
    }
}

fn sign_with_epsilon_f64(value: f64, eps: f64) -> Option<i8> {
    if value > eps {
        Some(1)
    } else if value < -eps {
        Some(-1)
    } else {
        None
    }
}

fn record_scroll_motion_check(
    active: &mut ActiveScript,
    step_index: usize,
    container: &UiSelectorV1,
    target: &UiSelectorV1,
    check: &UiScrollMotionCheckV1,
    state: &mut V2ScrollIntoViewState,
    scroll_node: Option<&fret_core::SemanticsNode>,
    target_bounds: Option<Rect>,
) -> Option<String> {
    state.motion_sample_count = state.motion_sample_count.saturating_add(1);

    let scroll_value = scroll_node.and_then(|node| semantics_scroll_field_value(node, check.field));
    let scroll_delta = scroll_value
        .zip(state.motion_last_scroll_value)
        .map(|(value, last)| value - last);
    let target_delta_y = target_bounds
        .zip(state.motion_last_target_y)
        .map(|(bounds, last)| bounds.origin.y.0 - last);

    let mut note = "scroll_into_view.motion.sample".to_string();
    let mut reason = None;

    if let Some(delta) = scroll_delta.filter(|delta| delta.is_finite()) {
        if delta.abs() > 0.01 {
            state.motion_saw_scroll_progress = true;
        }
        if let Some(sign) = sign_with_epsilon_f64(delta, check.max_scroll_reverse_px) {
            if state.motion_scroll_direction.is_none() {
                state.motion_scroll_direction = Some(sign);
            } else if state.motion_scroll_direction != Some(sign) {
                note = format!(
                    "scroll_into_view.motion.scroll_reversed delta={delta:.3} last_direction={:?}",
                    state.motion_scroll_direction
                );
                reason = Some("scroll_into_view_motion_scroll_reversed".to_string());
            }
        }
    }

    if reason.is_none()
        && let Some(delta) = target_delta_y
        && let Some(sign) = sign_with_epsilon_f32(delta, check.max_target_reverse_px)
    {
        if state.motion_target_direction.is_none() {
            state.motion_target_direction = Some(sign);
        } else if state.motion_target_direction != Some(sign) {
            note = format!(
                "scroll_into_view.motion.target_reversed delta={delta:.3} last_direction={:?}",
                state.motion_target_direction
            );
            reason = Some("scroll_into_view_motion_target_reversed".to_string());
        }
    }

    push_scroll_motion_trace(
        &mut active.scroll_motion_trace,
        UiScrollMotionTraceEntryV1 {
            step_index: step_index as u32,
            container: container.clone(),
            scroll_target: check.scroll_target.clone(),
            target: target.clone(),
            field: check.field,
            sample_count: state.motion_sample_count,
            scroll_value,
            scroll_delta,
            target_bounds: target_bounds.map(ui_rect_from_rect),
            target_delta_y_px: target_delta_y,
            max_target_reverse_px: Some(check.max_target_reverse_px),
            max_scroll_reverse_px: Some(check.max_scroll_reverse_px),
            note: Some(note),
        },
    );

    if let Some(value) = scroll_value.filter(|value| value.is_finite()) {
        state.motion_last_scroll_value = Some(value);
    }
    if let Some(bounds) = target_bounds {
        state.motion_last_target_y = Some(bounds.origin.y.0);
    }

    reason
}

pub(super) fn handle_scroll_into_view_step(
    svc: &mut UiDiagnosticsService,
    _app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&mut UiTree<App>>,
    _text_font_stack_key_stable_frames: u32,
    _font_catalog_populated: bool,
    _system_font_rescan_idle: bool,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::ScrollIntoView {
        window: _,
        pointer_kind,
        container,
        target,
        delta_x,
        delta_y,
        require_fully_within_container,
        require_fully_within_window,
        padding_px,
        padding_insets_px,
        motion_check,
        timeout_frames,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    let Some(snapshot) = semantics_snapshot else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-scroll_into_view-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    };

    let insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(padding_px));

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::ScrollIntoView(mut state)) if state.step_index == step_index => {
            state.remaining_frames = state.remaining_frames.min(timeout_frames);
            state
        }
        _ => V2ScrollIntoViewState {
            step_index,
            remaining_frames: timeout_frames,
            no_progress_frames: 0,
            last_target_bounds: None,
            last_container_bounds: None,
            motion_sample_count: 0,
            motion_last_scroll_value: None,
            motion_last_target_y: None,
            motion_scroll_direction: None,
            motion_target_direction: None,
            motion_saw_scroll_progress: false,
        },
    };

    let container_node = select_semantics_node_with_trace(
        snapshot,
        window,
        element_runtime,
        &container,
        active.scope_root_for_window(window),
        step_index as u32,
        svc.cfg.redact_text,
        &mut active.selector_resolution_trace,
    );
    let scroll_target_selector = motion_check
        .as_ref()
        .and_then(|check| check.scroll_target.as_ref())
        .unwrap_or(&container);
    let scroll_node = if motion_check.is_some() {
        select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            scroll_target_selector,
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        )
    } else {
        container_node
    };
    let target_node = select_semantics_node_with_trace(
        snapshot,
        window,
        element_runtime,
        &target,
        active.scope_root_for_window(window),
        step_index as u32,
        svc.cfg.redact_text,
        &mut active.selector_resolution_trace,
    );

    let container_bounds = container_node.map(|node| node.bounds);
    let target_bounds = target_node.map(|node| node.bounds);

    if let Some(check) = motion_check.as_ref()
        && let Some(reason) = record_scroll_motion_check(
            active,
            step_index,
            &container,
            &target,
            check,
            &mut state,
            scroll_node,
            target_bounds,
        )
    {
        *force_dump_label = Some(format!("script-step-{step_index:04}-{reason}"));
        *stop_script = true;
        *failure_reason = Some(reason);
        active.v2_step_state = None;
        output.request_redraw = true;
        return true;
    }

    // `padding_px` / `padding_insets_px` is a *scroll margin preference* (how much breathing room we
    // try to maintain while scrolling), not a hard correctness requirement.
    //
    // Treat "require fully within window/container" as a strict visibility requirement against the
    // raw bounds. This avoids pathological loops where the target is already fully visible, but
    // cannot satisfy the padded inset at a scroll boundary (leading to `stuck_no_progress`).
    if scroll_into_view_visibility_satisfied(
        window_bounds,
        container_bounds,
        target_bounds,
        require_fully_within_container,
        require_fully_within_window,
    ) {
        if let Some(check) = motion_check.as_ref()
            && check.require_scroll_progress
            && !state.motion_saw_scroll_progress
        {
            let reason = "scroll_into_view_motion_no_scroll_progress".to_string();
            push_scroll_motion_trace(
                &mut active.scroll_motion_trace,
                UiScrollMotionTraceEntryV1 {
                    step_index: step_index as u32,
                    container: container.clone(),
                    scroll_target: check.scroll_target.clone(),
                    target: target.clone(),
                    field: check.field,
                    sample_count: state.motion_sample_count,
                    scroll_value: state.motion_last_scroll_value,
                    scroll_delta: None,
                    target_bounds: target_bounds.map(ui_rect_from_rect),
                    target_delta_y_px: None,
                    max_target_reverse_px: Some(check.max_target_reverse_px),
                    max_scroll_reverse_px: Some(check.max_scroll_reverse_px),
                    note: Some(reason.clone()),
                },
            );
            *force_dump_label = Some(format!("script-step-{step_index:04}-{reason}"));
            *stop_script = true;
            *failure_reason = Some(reason);
            active.v2_step_state = None;
            output.request_redraw = true;
            return true;
        }
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-scroll_into_view"));
        }
    } else if state.remaining_frames == 0 {
        let timeout_reason = match (container_node.is_some(), target_node.is_some()) {
            (false, false) => "scroll_into_view_container_and_target_not_found",
            (false, true) => "scroll_into_view_container_not_found",
            (true, false) => "scroll_into_view_target_not_found",
            (true, true) => "scroll_into_view_timeout",
        };
        let dump_suffix = match timeout_reason {
            "scroll_into_view_container_and_target_not_found" => {
                "scroll_into_view-container-and-target-not-found"
            }
            "scroll_into_view_container_not_found" => "scroll_into_view-container-not-found",
            "scroll_into_view_target_not_found" => "scroll_into_view-target-not-found",
            _ => "scroll_into_view-timeout",
        };
        *force_dump_label = Some(format!("script-step-{step_index:04}-{dump_suffix}"));
        *stop_script = true;
        *failure_reason = Some(timeout_reason.to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    } else {
        if let (Some(container_node), Some(container_bounds)) = (container_node, container_bounds) {
            let visible_container =
                rect_intersection(container_bounds, window_bounds).unwrap_or(window_bounds);
            let inner_visible = rect_inset(visible_container, insets);

            let mut effective_dx = delta_x;
            let mut effective_dy = delta_y;
            let target_bounds_opt = target_bounds;
            if let Some(target_bounds) = target_bounds_opt {
                if require_fully_within_window {
                    let target_w = target_bounds.size.width.0.max(0.0);
                    let target_h = target_bounds.size.height.0.max(0.0);
                    let window_w = window_bounds.size.width.0.max(0.0);
                    let window_h = window_bounds.size.height.0.max(0.0);
                    if window_w > 1.0
                        && window_h > 1.0
                        && (target_w > window_w + 0.5 || target_h > window_h + 0.5)
                    {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-scroll_into_view-impossible-oversized"
                        ));
                        *stop_script = true;
                        *failure_reason =
                            Some("scroll_into_view_impossible_oversized_target".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                        return true;
                    }
                }

                if require_fully_within_container {
                    let target_w = target_bounds.size.width.0.max(0.0);
                    let target_h = target_bounds.size.height.0.max(0.0);
                    let container_w = container_bounds.size.width.0.max(0.0);
                    let container_h = container_bounds.size.height.0.max(0.0);
                    if container_w > 1.0
                        && container_h > 1.0
                        && (target_w > container_w + 0.5 || target_h > container_h + 0.5)
                    {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-scroll_into_view-impossible-oversized"
                        ));
                        *stop_script = true;
                        *failure_reason = Some(
                            "scroll_into_view_impossible_oversized_target_for_container"
                                .to_string(),
                        );
                        active.v2_step_state = None;
                        output.request_redraw = true;
                        return true;
                    }
                }

                if let Some(reason) = scroll_into_view_unscrollable_axis_reason(
                    window_bounds,
                    Some(container_bounds),
                    Some(target_bounds),
                    require_fully_within_container,
                    require_fully_within_window,
                    effective_dx,
                    effective_dy,
                ) {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-scroll_into_view-impossible-unscrollable-axis"
                    ));
                    *stop_script = true;
                    *failure_reason = Some(reason.to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                    return true;
                }

                let target_stable = state.last_target_bounds.is_some_and(|prev| {
                    rects_close_with_epsilon(prev, target_bounds, SCROLL_INTO_VIEW_PROGRESS_EPS_PX)
                });
                let container_stable = state.last_container_bounds.is_some_and(|prev| {
                    rects_close_with_epsilon(
                        prev,
                        container_bounds,
                        SCROLL_INTO_VIEW_PROGRESS_EPS_PX,
                    )
                });
                if target_stable && container_stable {
                    state.no_progress_frames = state.no_progress_frames.saturating_add(1);
                } else {
                    state.no_progress_frames = 0;
                }
                state.last_target_bounds = Some(target_bounds);
                state.last_container_bounds = Some(container_bounds);

                if state.no_progress_frames >= 20 {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-scroll_into_view-stuck-no-progress"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("scroll_into_view_stuck_no_progress".to_string());
                    active.v2_step_state = None;
                    output.request_redraw = true;
                    return true;
                }

                if effective_dx.abs() > 0.01 {
                    let target_left = target_bounds.origin.x.0;
                    let target_right = target_left + target_bounds.size.width.0.max(0.0);
                    let inner_left = inner_visible.origin.x.0;
                    let inner_right = inner_left + inner_visible.size.width.0.max(0.0);
                    let visible_left = visible_container.origin.x.0;
                    let visible_right = visible_left + visible_container.size.width.0.max(0.0);
                    effective_dx = bounded_scroll_delta_for_axis(
                        effective_dx,
                        target_left,
                        target_right,
                        inner_left,
                        inner_right,
                        visible_left,
                        visible_right,
                    );
                }

                if effective_dy.abs() > 0.01 {
                    let target_top = target_bounds.origin.y.0;
                    let target_bottom = target_top + target_bounds.size.height.0.max(0.0);
                    let inner_top = inner_visible.origin.y.0;
                    let inner_bottom = inner_top + inner_visible.size.height.0.max(0.0);
                    let visible_top = visible_container.origin.y.0;
                    let visible_bottom = visible_top + visible_container.size.height.0.max(0.0);
                    effective_dy = bounded_scroll_delta_for_axis(
                        effective_dy,
                        target_top,
                        target_bottom,
                        inner_top,
                        inner_bottom,
                        visible_top,
                        visible_bottom,
                    );
                }
            }

            let ix0 = inner_visible.origin.x.0;
            let iy0 = inner_visible.origin.y.0;
            let ix1 = ix0 + inner_visible.size.width.0.max(0.0);
            let iy1 = iy0 + inner_visible.size.height.0.max(0.0);

            let pad_x = 8.0f32.min((ix1 - ix0).max(0.0) * 0.5);
            let pad_y = 8.0f32.min((iy1 - iy0).max(0.0) * 0.5);

            let x_mid = (ix0 + ix1) * 0.5;
            let y_mid = (iy0 + iy1) * 0.5;
            let y_top = (iy0 + pad_y).clamp(iy0, iy1);
            let y_bottom = (iy1 - pad_y).clamp(iy0, iy1);

            let vx0 = visible_container.origin.x.0;
            let vx1 = vx0 + visible_container.size.width.0.max(0.0);
            let edge_pad_x = 2.0f32.min((vx1 - vx0).max(0.0) * 0.5);
            let x_edge_left = (vx0 + edge_pad_x).clamp(vx0, vx1);
            let x_edge_right = (vx1 - edge_pad_x).clamp(vx0, vx1);

            let x_pref = target_bounds_opt
                .map(|bounds| bounds.origin.x.0 + bounds.size.width.0.max(0.0) * 0.5)
                .unwrap_or(x_mid)
                .clamp(ix0 + pad_x, ix1 - pad_x);

            let candidates = [
                Point::new(Px(x_edge_left), Px(y_mid)),
                Point::new(Px(x_edge_left), Px(y_top)),
                Point::new(Px(x_edge_left), Px(y_bottom)),
                Point::new(Px(x_edge_right), Px(y_mid)),
                Point::new(Px(x_edge_right), Px(y_top)),
                Point::new(Px(x_edge_right), Px(y_bottom)),
                Point::new(Px(x_mid.clamp(ix0 + pad_x, ix1 - pad_x)), Px(y_mid)),
                Point::new(Px(x_pref), Px(y_mid.clamp(iy0 + pad_y, iy1 - pad_y))),
                Point::new(Px(x_pref), Px(y_top)),
                Point::new(Px(x_pref), Px(y_bottom)),
            ];

            let intended_id = container_node.id.data().as_ffi();
            let pos = if let Some(ui) = ui.as_deref() {
                let index = SemanticsIndex::new(snapshot);

                let nearest_scrollable_ancestor_id = |mut id: u64| -> Option<u64> {
                    while let Some(node) = index.by_id.get(&id).copied() {
                        if node.actions.scroll_by {
                            return Some(id);
                        }
                        let parent = node.parent?;
                        id = parent.data().as_ffi();
                    }
                    None
                };

                let mut best: Option<(i32, Point)> = None;
                for pos in candidates {
                    let Some(hit) = pick_semantics_node_at(snapshot, ui, pos) else {
                        continue;
                    };
                    let hit_id = hit.id.data().as_ffi();
                    let controls_intended = hit
                        .controls
                        .iter()
                        .any(|id| id.data().as_ffi() == intended_id);
                    let descendant_intended = index.is_descendant_of_or_self(hit_id, intended_id);
                    if !descendant_intended && !controls_intended {
                        continue;
                    }
                    // If the intended viewport is fully covered by nested scrollables (e.g. a code
                    // block), allow targeting the viewport's scrollbar (via `controls`).
                    if hit.role == fret_core::SemanticsRole::ScrollBar && !controls_intended {
                        continue;
                    }

                    let scroll_owner = nearest_scrollable_ancestor_id(hit_id);

                    let mut score: i32 = 0;
                    if scroll_owner == Some(intended_id) {
                        score += 100;
                    } else if controls_intended {
                        score += 95;
                    } else if scroll_owner.is_some() {
                        score += 0;
                    } else {
                        score -= 50;
                    }
                    if hit.role != fret_core::SemanticsRole::Text
                        && hit.role != fret_core::SemanticsRole::TextField
                    {
                        score += 10;
                    }
                    if hit.actions.invoke {
                        score -= 3;
                    }
                    if hit.actions.focus {
                        score -= 1;
                    }

                    if best
                        .as_ref()
                        .is_none_or(|(best_score, _)| score > *best_score)
                    {
                        best = Some((score, pos));
                    }
                }
                best.map(|(_, pos)| pos).unwrap_or(candidates[0])
            } else {
                candidates[0]
            };

            if let Some(ui) = ui {
                let note = format!(
                    "scroll_into_view.wheel dx={delta_x} dy={delta_y} -> dx={effective_dx} dy={effective_dy}"
                );
                record_hit_test_trace_for_selector(
                    &mut active.hit_test_trace,
                    ui,
                    element_runtime,
                    window,
                    semantics_snapshot,
                    &container,
                    step_index as u32,
                    pos,
                    Some(container_node),
                    Some(note.as_str()),
                    svc.cfg.max_debug_string_bytes,
                );
            }

            output.events.push(move_pointer_event(pos, pointer_type));
            output
                .events
                .push(wheel_event(pos, effective_dx, effective_dy, pointer_type));
        }

        state.remaining_frames = state.remaining_frames.saturating_sub(1);
        active.v2_step_state = Some(V2StepState::ScrollIntoView(state));
        output.request_redraw = true;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(width), Px(height)))
    }

    #[test]
    fn scroll_visibility_tolerates_subpixel_full_containment_edges() {
        let outer = rect(0.0, 0.0, 100.0, 100.0);

        assert!(rect_fully_contains_with_epsilon(
            outer,
            rect(10.0, 10.0, 90.25, 90.25),
            0.5
        ));
        assert!(!rect_fully_contains_with_epsilon(
            outer,
            rect(10.0, 10.0, 90.75, 90.75),
            0.5,
        ));
    }

    #[test]
    fn scroll_progress_treats_subpixel_bounds_jitter_as_stable() {
        assert!(rects_close_with_epsilon(
            rect(10.0, 20.0, 30.0, 40.0),
            rect(10.25, 19.75, 30.25, 39.75),
            0.5,
        ));
        assert!(!rects_close_with_epsilon(
            rect(10.0, 20.0, 30.0, 40.0),
            rect(10.75, 20.0, 30.0, 40.0),
            0.5,
        ));
    }

    #[test]
    fn scroll_unscrollable_axis_reports_impossible_container_containment() {
        let window = rect(0.0, 0.0, 480.0, 840.0);
        let container = rect(208.66667, 176.0, 247.33333, 640.0);
        let target = rect(234.0, 764.0, 240.0, 36.0);

        assert_eq!(
            scroll_into_view_unscrollable_axis_reason(
                window,
                Some(container),
                Some(target),
                true,
                false,
                0.0,
                -420.0,
            ),
            Some("scroll_into_view_impossible_unscrollable_x_for_container"),
        );
    }

    #[test]
    fn scroll_unscrollable_axis_allows_configured_horizontal_scroll() {
        let window = rect(0.0, 0.0, 480.0, 840.0);
        let container = rect(208.66667, 176.0, 247.33333, 640.0);
        let target = rect(234.0, 764.0, 240.0, 36.0);

        assert_eq!(
            scroll_into_view_unscrollable_axis_reason(
                window,
                Some(container),
                Some(target),
                true,
                false,
                32.0,
                -420.0,
            ),
            None,
        );
    }

    #[test]
    fn scroll_visibility_rejects_target_outside_container_even_if_visual_bounds_would_fit() {
        let window = rect(0.0, 0.0, 1080.0, 720.0);
        let container = rect(296.0, 546.0, 178.0, 164.0);
        let raw_target = rect(300.0, 1224.0, 170.0, 32.0);

        assert!(!scroll_into_view_visibility_satisfied(
            window,
            Some(container),
            Some(raw_target),
            true,
            true,
        ));
    }

    #[test]
    fn scroll_delta_is_capped_to_axis_visibility_gap() {
        assert_eq!(
            bounded_scroll_delta_for_axis(-40.0, 170.0, 206.0, 178.0, 234.0, 176.0, 236.0),
            8.0,
        );
        assert_eq!(
            bounded_scroll_delta_for_axis(-40.0, 204.0, 242.0, 178.0, 234.0, 176.0, 236.0),
            -8.0,
        );
        assert_eq!(
            bounded_scroll_delta_for_axis(-40.0, 180.0, 220.0, 178.0, 234.0, 176.0, 236.0),
            0.0,
        );
    }

    #[test]
    fn scroll_delta_falls_back_when_padding_makes_fit_impossible() {
        assert_eq!(
            bounded_scroll_delta_for_axis(-40.0, 170.0, 232.0, 178.0, 234.0, 176.0, 236.0),
            6.0,
        );
    }
}
