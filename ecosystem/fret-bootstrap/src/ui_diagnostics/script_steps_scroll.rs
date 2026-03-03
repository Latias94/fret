use super::*;

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

pub(super) fn handle_scroll_into_view_step(
    svc: &mut UiDiagnosticsService,
    _app: &App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
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

    let container_bounds: Option<Rect> = container_node.map(|node| {
        ui.as_deref()
            .and_then(|ui| ui.debug_node_visual_bounds(node.id))
            .unwrap_or(node.bounds)
    });
    let target_bounds: Option<Rect> = target_node.map(|node| {
        ui.as_deref()
            .and_then(|ui| ui.debug_node_visual_bounds(node.id))
            .unwrap_or(node.bounds)
    });

    // `padding_px` / `padding_insets_px` is a *scroll margin preference* (how much breathing room we
    // try to maintain while scrolling), not a hard correctness requirement.
    //
    // Treat "require fully within window/container" as a strict visibility requirement against the
    // raw bounds. This avoids pathological loops where the target is already fully visible, but
    // cannot satisfy the padded inset at a scroll boundary (leading to `stuck_no_progress`).
    let visible_ok = target_bounds.is_some_and(|bounds| {
        if require_fully_within_window {
            rect_fully_contains(window_bounds, bounds)
        } else {
            rect_intersection(bounds, window_bounds).is_some()
        }
    });
    let container_ok = if require_fully_within_container {
        container_bounds
            .zip(target_bounds)
            .is_some_and(|(container_bounds, target_bounds)| {
                rect_fully_contains(container_bounds, target_bounds)
            })
    } else {
        true
    };

    if visible_ok && container_ok {
        active.v2_step_state = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        if svc.cfg.script_auto_dump {
            *force_dump_label = Some(format!("script-step-{step_index:04}-scroll_into_view"));
        }
    } else if state.remaining_frames == 0 {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-scroll_into_view-timeout"
        ));
        *stop_script = true;
        *failure_reason = Some("scroll_into_view_timeout".to_string());
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

                if state
                    .last_target_bounds
                    .is_some_and(|prev| prev == target_bounds)
                {
                    state.no_progress_frames = state.no_progress_frames.saturating_add(1);
                } else {
                    state.no_progress_frames = 0;
                }
                state.last_target_bounds = Some(target_bounds);

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
                    let abs_dx = effective_dx.abs();
                    let target_left = target_bounds.origin.x.0;
                    let target_right = target_left + target_bounds.size.width.0.max(0.0);
                    let inner_left = inner_visible.origin.x.0;
                    let inner_right = inner_left + inner_visible.size.width.0.max(0.0);
                    if target_left < inner_left {
                        effective_dx = abs_dx;
                    } else if target_right > inner_right {
                        effective_dx = -abs_dx;
                    }
                }

                if effective_dy.abs() > 0.01 {
                    let abs_dy = effective_dy.abs();
                    let target_top = target_bounds.origin.y.0;
                    let target_bottom = target_top + target_bounds.size.height.0.max(0.0);
                    let inner_top = inner_visible.origin.y.0;
                    let inner_bottom = inner_top + inner_visible.size.height.0.max(0.0);
                    if target_top < inner_top {
                        effective_dy = abs_dy;
                    } else if target_bottom > inner_bottom {
                        effective_dy = -abs_dy;
                    }
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

            if let Some(ui) = ui.as_deref_mut() {
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
