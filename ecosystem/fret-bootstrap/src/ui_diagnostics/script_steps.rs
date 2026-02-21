use super::*;

pub(super) fn handle_window_effect_steps(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) {
    match step {
        UiActionStepV2::SetWindowInnerSize {
            window: target_window,
            width_px,
            height_px,
        } => {
            if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
                let size = fret_core::Size::new(fret_core::Px(width_px), fret_core::Px(height_px));
                output
                    .effects
                    .push(Effect::Window(fret_app::WindowRequest::SetInnerSize {
                        window: target_window,
                        size,
                    }));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_window_inner_size-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                active.wait_until = None;
                active.screenshot_wait = None;
                active.v2_step_state = None;
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetWindowOuterPosition {
            window: target_window,
            x_px,
            y_px,
        } => {
            if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
                output
                    .effects
                    .push(Effect::Window(fret_app::WindowRequest::SetOuterPosition {
                        window: target_window,
                        position: fret_core::WindowLogicalPosition {
                            x: x_px.round() as i32,
                            y: y_px.round() as i32,
                        },
                    }));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_window_outer_position-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                active.wait_until = None;
                active.screenshot_wait = None;
                active.v2_step_state = None;
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetCursorScreenPos { x_px, y_px } => {
            let payload =
                format!("schema_version=1\nkind=screen_physical\nx_px={x_px}\ny_px={y_px}\n");
            let text_path = svc.cfg.out_dir.join("cursor_screen_pos.override.txt");
            let trigger_path = svc.cfg.out_dir.join("cursor_screen_pos.touch");
            let _ = std::fs::create_dir_all(&svc.cfg.out_dir);
            if std::fs::write(text_path, payload).is_ok() && touch_file(&trigger_path).is_ok() {
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_cursor_screen_pos-write-failed"
                ));
                *stop_script = true;
                *failure_reason = Some("cursor_override_write_failed".to_string());
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetCursorInWindow {
            window: target_window,
            x_px,
            y_px,
        } => {
            if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
                let payload = format!(
                    "schema_version=1\nkind=window_client_physical\nwindow={}\nx_px={}\ny_px={}\n",
                    target_window.data().as_ffi(),
                    x_px,
                    y_px
                );
                let text_path = svc.cfg.out_dir.join("cursor_screen_pos.override.txt");
                let trigger_path = svc.cfg.out_dir.join("cursor_screen_pos.touch");
                let _ = std::fs::create_dir_all(&svc.cfg.out_dir);
                if std::fs::write(text_path, payload).is_ok() && touch_file(&trigger_path).is_ok() {
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                } else {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-set_cursor_in_window-write-failed"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("cursor_override_write_failed".to_string());
                    output.request_redraw = true;
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_cursor_in_window-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetCursorInWindowLogical {
            window: target_window,
            x_px,
            y_px,
        } => {
            if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
                if write_cursor_override_window_client_logical(
                    &svc.cfg.out_dir,
                    target_window,
                    x_px,
                    y_px,
                )
                .is_ok()
                {
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                } else {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-set_cursor_in_window_logical-write-failed"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("cursor_override_write_failed".to_string());
                    output.request_redraw = true;
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_cursor_in_window_logical-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetMouseButtons {
            window: target_window,
            left,
            right,
            middle,
        } => {
            let resolved_window = if let Some(target_window) = target_window.as_ref() {
                svc.resolve_window_target(window, Some(target_window))
            } else {
                None
            };
            if target_window.is_some() && resolved_window.is_none() {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_mouse_buttons-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                output.request_redraw = true;
            }

            if !*stop_script {
                let mut payload = String::from("schema_version=1\n");
                if let Some(window) = resolved_window {
                    payload.push_str(&format!("window={}\n", window.data().as_ffi()));
                }
                if let Some(left) = left {
                    payload.push_str(&format!("left={left}\n"));
                }
                if let Some(right) = right {
                    payload.push_str(&format!("right={right}\n"));
                }
                if let Some(middle) = middle {
                    payload.push_str(&format!("middle={middle}\n"));
                }

                let text_path = svc.cfg.out_dir.join("mouse_buttons.override.txt");
                let trigger_path = svc.cfg.out_dir.join("mouse_buttons.touch");
                let _ = std::fs::create_dir_all(&svc.cfg.out_dir);
                if std::fs::write(text_path, payload).is_ok() && touch_file(&trigger_path).is_ok() {
                    active.wait_until = None;
                    active.screenshot_wait = None;
                    active.next_step = active.next_step.saturating_add(1);
                    output.request_redraw = true;
                } else {
                    *force_dump_label = Some(format!(
                        "script-step-{step_index:04}-set_mouse_buttons-write-failed"
                    ));
                    *stop_script = true;
                    *failure_reason = Some("mouse_buttons_override_write_failed".to_string());
                    output.request_redraw = true;
                }
            }
        }
        UiActionStepV2::RaiseWindow {
            window: target_window,
        } => {
            if let Some(target_window) = svc.resolve_window_target(window, target_window.as_ref()) {
                output
                    .effects
                    .push(Effect::Window(fret_app::WindowRequest::Raise {
                        window: target_window,
                        sender: Some(window),
                    }));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-raise_window-window-not-found"
                ));
                *stop_script = true;
                *failure_reason = Some("window_target_unresolved".to_string());
                output.request_redraw = true;
            }
        }
        UiActionStepV2::SetWindowInsets {
            safe_area_insets,
            occlusion_insets,
        } => {
            let edges_from_insets = |insets: UiPaddingInsetsV1| fret_core::Edges {
                left: fret_core::Px(insets.left_px),
                top: fret_core::Px(insets.top_px),
                right: fret_core::Px(insets.right_px),
                bottom: fret_core::Px(insets.bottom_px),
            };

            let to_override = |ovr: fret_diag_protocol::UiInsetsOverrideV1| match ovr {
                fret_diag_protocol::UiInsetsOverrideV1::NoChange => None,
                fret_diag_protocol::UiInsetsOverrideV1::Clear => Some(None),
                fret_diag_protocol::UiInsetsOverrideV1::Set { insets_px } => {
                    Some(Some(edges_from_insets(insets_px)))
                }
            };

            output.effects.push(Effect::WindowMetricsSetInsets {
                window,
                safe_area_insets: to_override(safe_area_insets),
                occlusion_insets: to_override(occlusion_insets),
            });
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
        }
        _ => {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-internal-window-step-unexpected"
            ));
            *stop_script = true;
            *failure_reason = Some("script_internal_unexpected_step".to_string());
            active.v2_step_state = None;
            output.request_redraw = true;
        }
    }
}
