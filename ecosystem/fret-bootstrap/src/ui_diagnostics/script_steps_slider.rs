use super::*;

fn supports_numeric_set_value(node: &fret_core::SemanticsNode) -> bool {
    node.actions.set_value
        && matches!(
            node.role,
            fret_core::SemanticsRole::Slider
                | fret_core::SemanticsRole::SpinButton
                | fret_core::SemanticsRole::Splitter
        )
}

fn is_descendant_of(
    snapshot: &fret_core::SemanticsSnapshot,
    descendant: fret_core::NodeId,
    ancestor: fret_core::NodeId,
) -> bool {
    let mut current = Some(descendant);
    while let Some(node_id) = current {
        if node_id == ancestor {
            return true;
        }
        current = snapshot
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| node.parent);
    }
    false
}

fn numeric_set_value_target(
    snapshot: &fret_core::SemanticsSnapshot,
    selected: &fret_core::SemanticsNode,
) -> Option<fret_core::NodeId> {
    if supports_numeric_set_value(selected) {
        return Some(selected.id);
    }

    snapshot
        .nodes
        .iter()
        .find(|candidate| {
            supports_numeric_set_value(candidate)
                && is_descendant_of(snapshot, candidate.id, selected.id)
        })
        .map(|candidate| candidate.id)
}

pub(super) fn handle_set_slider_value_step(
    svc: &mut UiDiagnosticsService,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    mut ui: Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::SetSliderValue {
        window: _,
        pointer_kind,
        target,
        value,
        min,
        max,
        epsilon,
        timeout_frames,
        drag_steps,
    } = step
    else {
        return false;
    };

    let pointer_type = pointer_type_from_kind(pointer_kind);
    active.wait_until = None;
    active.screenshot_wait = None;

    if let Some(snapshot) = semantics_snapshot {
        let mut state = match active.v2_step_state.take() {
            Some(V2StepState::SetSliderValue(mut state)) if state.step_index == step_index => {
                state.remaining_frames = state.remaining_frames.min(timeout_frames);
                state
            }
            _ => V2SetSliderValueState {
                step_index,
                remaining_frames: timeout_frames,
                phase: 0,
                last_drag_x: None,
            },
        };

        let node = select_semantics_node_with_trace(
            snapshot,
            window,
            element_runtime,
            &target,
            active.scope_root_for_window(window),
            step_index as u32,
            svc.cfg.redact_text,
            &mut active.selector_resolution_trace,
        );
        if let Some(node) = node {
            if node.flags.disabled {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_slider_value-disabled"
                ));
                *stop_script = true;
                *failure_reason = Some("set_slider_value_disabled".to_string());
                active.v2_step_state = None;
                output.request_redraw = true;
            } else {
                let bounds = node.bounds;
                let left = bounds.origin.x.0;
                let width = bounds.size.width.0.max(0.0);
                let right = left + width;
                let span = (max - min).abs().max(0.0001);

                let clamp_x = |x: f32| {
                    let pad = 2.0_f32;
                    x.clamp(left + pad, right - pad)
                };
                let target_t = ((value - min) / span).clamp(0.0, 1.0);

                if state.phase == 0 {
                    if let Some(ui) = ui.as_deref_mut()
                        && let Some(numeric_target) = numeric_set_value_target(snapshot, node)
                    {
                        active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                        fret_ui_app::accessibility_actions::set_value_numeric(
                            ui,
                            app,
                            services,
                            numeric_target,
                            value as f64,
                        );
                        state.phase = 1;
                        active.v2_step_state = Some(V2StepState::SetSliderValue(state));
                        output.request_redraw = true;
                        return true;
                    }

                    let x = clamp_x(left + width * target_t);
                    let start = center_of_rect_clamped_to_rect(bounds, window_bounds);
                    let start_x = state.last_drag_x.unwrap_or(start.x.0);
                    let start = Point::new(fret_core::Px(start_x), start.y);
                    let end = Point::new(fret_core::Px(x), start.y);
                    if let Some(ui) = ui.as_deref_mut() {
                        record_hit_test_trace_for_selector(
                            &mut active.hit_test_trace,
                            ui,
                            element_runtime,
                            window,
                            Some(snapshot),
                            &target,
                            step_index as u32,
                            start,
                            Some(node),
                            Some("set_slider_value.drag_start"),
                            svc.cfg.max_debug_string_bytes,
                        );
                        record_hit_test_trace_for_selector(
                            &mut active.hit_test_trace,
                            ui,
                            element_runtime,
                            window,
                            Some(snapshot),
                            &target,
                            step_index as u32,
                            end,
                            Some(node),
                            Some("set_slider_value.drag_end"),
                            svc.cfg.max_debug_string_bytes,
                        );
                    }
                    output.events.extend(drag_events(
                        start,
                        end,
                        UiMouseButtonV1::Left,
                        drag_steps.max(1),
                        pointer_type,
                    ));
                    state.phase = 1;
                    state.last_drag_x = Some(x);
                    active.v2_step_state = Some(V2StepState::SetSliderValue(state));
                    output.request_redraw = true;
                } else {
                    let observed = node
                        .extra
                        .numeric
                        .value
                        .and_then(|v| v.is_finite().then_some(v as f32))
                        .or_else(|| {
                            node.value
                                .as_deref()
                                .and_then(parse_semantics_numeric_value)
                        });
                    if let Some(observed) = observed {
                        if (observed - value).abs() <= epsilon.max(0.0) {
                            active.v2_step_state = None;
                            active.next_step = active.next_step.saturating_add(1);
                            output.request_redraw = true;
                            if svc.cfg.script_auto_dump {
                                *force_dump_label =
                                    Some(format!("script-step-{step_index:04}-set_slider_value"));
                            }
                        } else if state.remaining_frames == 0 {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-set_slider_value-timeout"
                            ));
                            *stop_script = true;
                            *failure_reason = Some("set_slider_value_timeout".to_string());
                            active.v2_step_state = None;
                            output.request_redraw = true;
                        } else {
                            let error = value - observed;
                            let dx = (error / span) * width;
                            let start = center_of_rect_clamped_to_rect(bounds, window_bounds);
                            let start_x = state.last_drag_x.unwrap_or(start.x.0);
                            let end_x = clamp_x(start_x + dx);
                            let start = Point::new(fret_core::Px(start_x), start.y);
                            let end = Point::new(fret_core::Px(end_x), start.y);
                            if let Some(ui) = ui.as_deref_mut() {
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    element_runtime,
                                    window,
                                    Some(snapshot),
                                    &target,
                                    step_index as u32,
                                    start,
                                    Some(node),
                                    Some("set_slider_value.adjust_drag_start"),
                                    svc.cfg.max_debug_string_bytes,
                                );
                                record_hit_test_trace_for_selector(
                                    &mut active.hit_test_trace,
                                    ui,
                                    element_runtime,
                                    window,
                                    Some(snapshot),
                                    &target,
                                    step_index as u32,
                                    end,
                                    Some(node),
                                    Some("set_slider_value.adjust_drag_end"),
                                    svc.cfg.max_debug_string_bytes,
                                );
                            }
                            output.events.extend(drag_events(
                                start,
                                end,
                                UiMouseButtonV1::Left,
                                drag_steps.max(1),
                                pointer_type,
                            ));
                            state.last_drag_x = Some(end_x);
                            state.remaining_frames = state.remaining_frames.saturating_sub(1);
                            active.v2_step_state = Some(V2StepState::SetSliderValue(state));
                            output.request_redraw = true;
                        }
                    } else {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-set_slider_value-unparseable"
                        ));
                        *stop_script = true;
                        *failure_reason = Some("set_slider_value_unparseable".to_string());
                        active.v2_step_state = None;
                        output.request_redraw = true;
                    }
                }
            }
        } else if state.remaining_frames == 0 {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-set_slider_value-timeout"
            ));
            *stop_script = true;
            *failure_reason = Some("set_slider_value_timeout".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        } else {
            state.remaining_frames = state.remaining_frames.saturating_sub(1);
            active.v2_step_state = Some(V2StepState::SetSliderValue(state));
            output.request_redraw = true;
        }
    } else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-set_slider_value-no-semantics"
        ));
        *stop_script = true;
        *failure_reason = Some("no_semantics_snapshot".to_string());
        active.v2_step_state = None;
        output.request_redraw = true;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        NodeId, Point, Px, Rect, SemanticsActions, SemanticsNode, SemanticsNodeExtra,
        SemanticsRole, SemanticsSnapshot, Size,
    };
    use slotmap::KeyData;

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn rect() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)))
    }

    fn semantics_node(
        id: NodeId,
        parent: Option<NodeId>,
        role: SemanticsRole,
        set_value: bool,
    ) -> SemanticsNode {
        SemanticsNode {
            id,
            parent,
            role,
            bounds: rect(),
            flags: Default::default(),
            test_id: None,
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: SemanticsNodeExtra::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions {
                set_value,
                ..Default::default()
            },
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    #[test]
    fn numeric_set_value_target_keeps_direct_slider_selection() {
        let slider = semantics_node(node_id(2), None, SemanticsRole::Slider, true);
        let snapshot = SemanticsSnapshot {
            nodes: vec![slider.clone()],
            ..Default::default()
        };

        assert_eq!(
            numeric_set_value_target(&snapshot, &slider),
            Some(slider.id)
        );
    }

    #[test]
    fn numeric_set_value_target_finds_slider_descendant_from_generic_selection() {
        let root = semantics_node(node_id(1), None, SemanticsRole::Generic, false);
        let slider = semantics_node(node_id(2), Some(root.id), SemanticsRole::Slider, true);
        let thumb = semantics_node(node_id(3), Some(slider.id), SemanticsRole::Generic, false);
        let snapshot = SemanticsSnapshot {
            nodes: vec![root.clone(), slider.clone(), thumb],
            ..Default::default()
        };

        assert_eq!(numeric_set_value_target(&snapshot, &root), Some(slider.id));
    }

    #[test]
    fn numeric_set_value_target_ignores_descendants_without_numeric_set_value() {
        let root = semantics_node(node_id(1), None, SemanticsRole::Generic, false);
        let generic_child = semantics_node(node_id(2), Some(root.id), SemanticsRole::Generic, true);
        let snapshot = SemanticsSnapshot {
            nodes: vec![root.clone(), generic_child],
            ..Default::default()
        };

        assert_eq!(numeric_set_value_target(&snapshot, &root), None);
    }
}
