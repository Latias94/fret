use super::*;

use fret_diag_protocol::{DiagScreenshotRequestV1, DiagScreenshotWindowRequestV1};

pub(super) fn handle_window_effect_steps(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    anchor_window: AppWindowId,
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
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
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
        UiActionStepV2::SetWindowStyle {
            window: target_window,
            style,
        } => {
            if !cfg!(target_os = "windows") {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_window_style-unsupported-platform"
                ));
                *stop_script = true;
                *failure_reason = Some("window_style_patch_unsupported_platform".to_string());
                active.wait_until = None;
                active.screenshot_wait = None;
                active.v2_step_state = None;
                output.request_redraw = true;
                return;
            }
            if let Err(fields) = validate_window_style_patch_supported_fields_windows(&style) {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_window_style-unsupported-fields-{fields}"
                ));
                *stop_script = true;
                *failure_reason = Some("window_style_patch_unsupported_fields".to_string());
                active.wait_until = None;
                active.screenshot_wait = None;
                active.v2_step_state = None;
                output.request_redraw = true;
                return;
            }
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
                let patch = window_style_request_from_patch(style);
                output
                    .effects
                    .push(Effect::Window(fret_app::WindowRequest::SetStyle {
                        window: target_window,
                        style: patch,
                    }));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-set_window_style-window-not-found"
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
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
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
                active.last_explicit_cursor_override = Some(CursorOverrideTarget::ScreenPhysical);
                active.last_explicit_cursor_override_pos = Some(ExplicitCursorOverridePos {
                    target: CursorOverrideTarget::ScreenPhysical,
                    x_px,
                    y_px,
                });
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
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
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
                    active.last_explicit_cursor_override =
                        Some(CursorOverrideTarget::WindowClientPhysical(target_window));
                    active.last_explicit_cursor_override_pos = Some(ExplicitCursorOverridePos {
                        target: CursorOverrideTarget::WindowClientPhysical(target_window),
                        x_px,
                        y_px,
                    });
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
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
                if write_cursor_override_window_client_logical(
                    &svc.cfg.out_dir,
                    target_window,
                    x_px,
                    y_px,
                )
                .is_ok()
                {
                    active.last_explicit_cursor_override =
                        Some(CursorOverrideTarget::WindowClientLogical(target_window));
                    active.last_explicit_cursor_override_pos = Some(ExplicitCursorOverridePos {
                        target: CursorOverrideTarget::WindowClientLogical(target_window),
                        x_px,
                        y_px,
                    });
                    if let Some(session) = active.pointer_session.as_mut()
                        && session.window == target_window
                    {
                        session.position = Point::new(Px(x_px), Px(y_px));
                    }
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
                svc.resolve_window_target_for_active_step(
                    window,
                    anchor_window,
                    Some(target_window),
                )
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
            if let Some(target_window) = svc.resolve_window_target_for_active_step(
                window,
                anchor_window,
                target_window.as_ref(),
            ) {
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

pub(super) fn handle_effect_only_steps(
    svc: &mut UiDiagnosticsService,
    window: AppWindowId,
    step: UiActionStepV2,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
) -> bool {
    match step {
        UiActionStepV2::SetClipboardForceUnavailable { enabled } => {
            output
                .effects
                .push(Effect::DiagClipboardForceUnavailable { window, enabled });
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        UiActionStepV2::SetClipboardText { text } => {
            output.effects.push(Effect::ClipboardSetText { text });
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        UiActionStepV2::InjectIncomingOpen { items } => {
            let items = items
                .into_iter()
                .map(|item| match item {
                    UiIncomingOpenInjectItemV1::FileUtf8 {
                        name,
                        text,
                        media_type,
                    } => fret_runtime::DiagIncomingOpenItem::File {
                        name,
                        bytes: text.into_bytes(),
                        media_type,
                    },
                    UiIncomingOpenInjectItemV1::Text { text, media_type } => {
                        fret_runtime::DiagIncomingOpenItem::Text { text, media_type }
                    }
                })
                .collect();
            output
                .effects
                .push(Effect::DiagIncomingOpenInject { window, items });
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        UiActionStepV2::WaitFrames { n, .. } => {
            active.wait_frames_remaining = n;
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        UiActionStepV2::ResetDiagnostics => {
            svc.reset_diagnostics_ring_for_window(window);
            ui_thread_cpu_time::reset();
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        _ => false,
    }
}

fn validate_window_style_patch_supported_fields_windows(
    patch: &fret_diag_protocol::UiWindowStylePatchV1,
) -> Result<(), String> {
    let mut unsupported: Vec<&'static str> = Vec::new();
    if patch.taskbar.is_some() {
        unsupported.push("taskbar");
    }
    if patch.activation.is_some() {
        unsupported.push("activation");
    }
    if patch.decorations.is_some() {
        unsupported.push("decorations");
    }
    if patch.resizable.is_some() {
        unsupported.push("resizable");
    }
    if patch.transparent.is_some() {
        unsupported.push("transparent");
    }

    // Supported (Windows-only, as of 2026-03-04):
    // - z_level
    // - background_material
    // - hit_test (including passthrough regions)
    // - opacity_alpha_u8
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(unsupported.join("-"))
    }
}

fn window_style_request_from_patch(
    patch: fret_diag_protocol::UiWindowStylePatchV1,
) -> fret_runtime::WindowStyleRequest {
    use fret_diag_protocol::{
        UiActivationPolicyV1, UiTaskbarVisibilityV1, UiWindowBackgroundMaterialRequestV1,
        UiWindowDecorationsRequestV1, UiWindowHitTestPatchV1, UiWindowHitTestRegionV1,
        UiWindowZLevelV1,
    };
    use fret_runtime::{
        ActivationPolicy, TaskbarVisibility, WindowBackgroundMaterialRequest,
        WindowDecorationsRequest, WindowHitTestRegionV1, WindowHitTestRequestV1, WindowOpacity,
        WindowStyleRequest, WindowZLevel,
    };

    fn taskbar(v: UiTaskbarVisibilityV1) -> TaskbarVisibility {
        match v {
            UiTaskbarVisibilityV1::Show => TaskbarVisibility::Show,
            UiTaskbarVisibilityV1::Hide => TaskbarVisibility::Hide,
        }
    }

    fn activation(v: UiActivationPolicyV1) -> ActivationPolicy {
        match v {
            UiActivationPolicyV1::Activates => ActivationPolicy::Activates,
            UiActivationPolicyV1::NonActivating => ActivationPolicy::NonActivating,
        }
    }

    fn z_level(v: UiWindowZLevelV1) -> WindowZLevel {
        match v {
            UiWindowZLevelV1::Normal => WindowZLevel::Normal,
            UiWindowZLevelV1::AlwaysOnTop => WindowZLevel::AlwaysOnTop,
        }
    }

    fn decorations(v: UiWindowDecorationsRequestV1) -> WindowDecorationsRequest {
        match v {
            UiWindowDecorationsRequestV1::System => WindowDecorationsRequest::System,
            UiWindowDecorationsRequestV1::None => WindowDecorationsRequest::None,
            UiWindowDecorationsRequestV1::Server => WindowDecorationsRequest::Server,
            UiWindowDecorationsRequestV1::Client => WindowDecorationsRequest::Client,
        }
    }

    fn material(v: UiWindowBackgroundMaterialRequestV1) -> WindowBackgroundMaterialRequest {
        match v {
            UiWindowBackgroundMaterialRequestV1::None => WindowBackgroundMaterialRequest::None,
            UiWindowBackgroundMaterialRequestV1::SystemDefault => {
                WindowBackgroundMaterialRequest::SystemDefault
            }
            UiWindowBackgroundMaterialRequestV1::Mica => WindowBackgroundMaterialRequest::Mica,
            UiWindowBackgroundMaterialRequestV1::Acrylic => {
                WindowBackgroundMaterialRequest::Acrylic
            }
            UiWindowBackgroundMaterialRequestV1::Vibrancy => {
                WindowBackgroundMaterialRequest::Vibrancy
            }
        }
    }

    fn hit_test_region(v: UiWindowHitTestRegionV1) -> WindowHitTestRegionV1 {
        match v {
            UiWindowHitTestRegionV1::Rect {
                x,
                y,
                width,
                height,
            } => WindowHitTestRegionV1::Rect {
                x,
                y,
                width,
                height,
            },
            UiWindowHitTestRegionV1::RRect {
                x,
                y,
                width,
                height,
                radius,
            } => WindowHitTestRegionV1::RRect {
                x,
                y,
                width,
                height,
                radius,
            },
        }
    }

    fn hit_test(v: UiWindowHitTestPatchV1) -> WindowHitTestRequestV1 {
        match v {
            UiWindowHitTestPatchV1::Normal => WindowHitTestRequestV1::Normal,
            UiWindowHitTestPatchV1::PassthroughAll => WindowHitTestRequestV1::PassthroughAll,
            UiWindowHitTestPatchV1::PassthroughRegions { regions } => {
                WindowHitTestRequestV1::PassthroughRegions {
                    regions: regions.into_iter().map(hit_test_region).collect(),
                }
            }
        }
    }

    WindowStyleRequest {
        taskbar: patch.taskbar.map(taskbar),
        activation: patch.activation.map(activation),
        z_level: patch.z_level.map(z_level),
        decorations: patch.decorations.map(decorations),
        resizable: patch.resizable,
        transparent: patch.transparent,
        background_material: patch.background_material.map(material),
        hit_test: patch.hit_test.map(hit_test),
        opacity: patch.opacity_alpha_u8.map(WindowOpacity),
    }
}

pub(super) fn handle_capture_steps(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    scale_factor: f32,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    force_dump_max_snapshots: &mut Option<usize>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    match step {
        UiActionStepV2::CaptureBundle {
            label,
            max_snapshots,
        } => {
            *force_dump_label =
                Some(label.unwrap_or_else(|| format!("script-step-{step_index:04}-capture")));
            *force_dump_max_snapshots = max_snapshots.map(|n| n as usize);
            active.wait_until = None;
            active.screenshot_wait = None;
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            true
        }
        UiActionStepV2::CaptureScreenshot {
            label,
            timeout_frames,
        } => {
            let window_ffi = window.data().as_ffi();
            active.wait_until = None;
            if !svc.cfg.screenshots_enabled {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-capture_screenshot-disabled"
                ));
                *stop_script = true;
                *failure_reason = Some("screenshots_disabled".to_string());
                active.screenshot_wait = None;
                output.request_redraw = true;
            } else {
                let mut state = match active.screenshot_wait.take() {
                    Some(mut state) if state.step_index == step_index => {
                        state.remaining_frames = state.remaining_frames.min(timeout_frames);
                        Some(state)
                    }
                    _ => None,
                };

                if state.is_none() {
                    if svc.last_dump_dir.is_none() {
                        let dump_label =
                            label.as_deref().map(sanitize_label).unwrap_or_else(|| {
                                format!("script-step-{step_index:04}-capture_screenshot")
                            });
                        svc.dump_bundle(Some(&dump_label));
                    }

                    let bundle_dir_name = svc
                        .last_dump_dir
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    if bundle_dir_name.is_empty() {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-capture_screenshot-no-last-dump"
                        ));
                        *stop_script = true;
                        *failure_reason = Some("no_last_dump_dir".to_string());
                        active.screenshot_wait = None;
                        output.request_redraw = true;
                    } else {
                        let request_id = format!(
                            "script-run-{run_id}-window-{window_ffi}-step-{step_index:04}",
                            run_id = active.run_id
                        );

                        let req = DiagScreenshotRequestV1 {
                            schema_version: 1,
                            out_dir: svc.cfg.out_dir.to_string_lossy().to_string(),
                            bundle_dir_name,
                            request_id: Some(request_id.clone()),
                            windows: vec![DiagScreenshotWindowRequestV1 {
                                window: window_ffi,
                                tick_id: app.tick_id().0,
                                frame_id: app.frame_id().0,
                                scale_factor: scale_factor as f64,
                            }],
                        };

                        let bytes = serde_json::to_vec_pretty(&req).ok();
                        if let Some(bytes) = bytes {
                            if let Some(parent) = svc.cfg.screenshot_request_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            let write_ok = std::fs::write(&svc.cfg.screenshot_request_path, bytes)
                                .is_ok()
                                && touch_file(&svc.cfg.screenshot_trigger_path).is_ok();
                            if write_ok {
                                state = Some(ScreenshotWaitState {
                                    step_index,
                                    remaining_frames: timeout_frames,
                                    request_id,
                                    window_ffi,
                                });
                            } else {
                                *force_dump_label = Some(format!(
                                    "script-step-{step_index:04}-capture_screenshot-write-failed"
                                ));
                                *stop_script = true;
                                *failure_reason =
                                    Some("screenshot_request_write_failed".to_string());
                                active.screenshot_wait = None;
                                output.request_redraw = true;
                            }
                        } else {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-capture_screenshot-serialize-failed"
                            ));
                            *stop_script = true;
                            *failure_reason =
                                Some("screenshot_request_serialize_failed".to_string());
                            active.screenshot_wait = None;
                            output.request_redraw = true;
                        }
                    }
                }

                if !*stop_script {
                    if let Some(state) = state {
                        let trigger_stamp =
                            read_touch_stamp(&svc.cfg.screenshot_result_trigger_path);
                        let completed = trigger_stamp.is_some()
                            && screenshot_request_completed(
                                &svc.cfg.screenshot_result_path,
                                &state.request_id,
                                state.window_ffi,
                            );

                        if completed {
                            active.screenshot_wait = None;
                            active.next_step = active.next_step.saturating_add(1);
                            output.request_redraw = true;
                        } else if state.remaining_frames == 0 {
                            *force_dump_label = Some(format!(
                                "script-step-{step_index:04}-capture_screenshot-timeout"
                            ));
                            *stop_script = true;
                            *failure_reason = Some("capture_screenshot_timeout".to_string());
                            active.screenshot_wait = None;
                            output.request_redraw = true;
                        } else {
                            active.screenshot_wait = Some(ScreenshotWaitState {
                                step_index: state.step_index,
                                remaining_frames: state.remaining_frames.saturating_sub(1),
                                request_id: state.request_id,
                                window_ffi: state.window_ffi,
                            });
                            output.request_redraw = true;
                        }
                    } else {
                        *force_dump_label = Some(format!(
                            "script-step-{step_index:04}-capture_screenshot-no-state"
                        ));
                        *stop_script = true;
                        *failure_reason = Some("capture_screenshot_state_missing".to_string());
                        active.screenshot_wait = None;
                        output.request_redraw = true;
                    }
                }
            }
            true
        }
        _ => false,
    }
}

pub(super) fn handle_capture_layout_sidecar_step(
    svc: &mut UiDiagnosticsService,
    app: &mut App,
    window: AppWindowId,
    window_bounds: Rect,
    step_index: usize,
    label: Option<String>,
    root_label_filter: Option<String>,
    scale_factor: f32,
    ui: &mut Option<&mut UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
) -> bool {
    let root_label_filter = root_label_filter
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    // v1 is native-only, best-effort. Scripts should still pass even if sidecars are missing.
    if cfg!(target_arch = "wasm32") {
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "layout_sidecar.skipped".to_string(),
                step_index: Some(step_index as u32),
                note: Some("wasm32_unsupported".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        return true;
    }

    let dump_label = label
        .as_deref()
        .map(sanitize_label)
        .unwrap_or_else(|| format!("script-step-{step_index:04}-layout_sidecar"));
    let dumped_dir = svc.dump_bundle(Some(&dump_label));
    let Some(dumped_dir) = dumped_dir else {
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "layout_sidecar.skipped".to_string(),
                step_index: Some(step_index as u32),
                note: Some("bundle_dump_failed".to_string()),
                bundle_dir: None,
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        return true;
    };

    let Some(ui) = ui.as_deref_mut() else {
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "layout_sidecar.skipped".to_string(),
                step_index: Some(step_index as u32),
                note: Some("ui_unavailable".to_string()),
                bundle_dir: Some(display_path(&svc.cfg.out_dir, &dumped_dir)),
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        return true;
    };

    let Some(root) = ui.base_root() else {
        push_script_event_log(
            active,
            &svc.cfg,
            UiScriptEventLogEntryV1 {
                unix_ms: unix_ms_now(),
                kind: "layout_sidecar.skipped".to_string(),
                step_index: Some(step_index as u32),
                note: Some("missing_base_root".to_string()),
                bundle_dir: Some(display_path(&svc.cfg.out_dir, &dumped_dir)),
                window: Some(window.data().as_ffi()),
                tick_id: Some(app.tick_id().0),
                frame_id: Some(app.frame_id().0),
                window_snapshot_seq: None,
            },
        );
        active.wait_until = None;
        active.screenshot_wait = None;
        active.next_step = active.next_step.saturating_add(1);
        output.request_redraw = true;
        return true;
    };

    let captured_at_unix_ms = unix_ms_now();
    let sidecar_result = ui.debug_write_layout_sidecar_taffy_v1_json(
        app,
        window,
        root,
        window_bounds,
        scale_factor,
        root_label_filter,
        &dumped_dir,
        captured_at_unix_ms,
    );

    match sidecar_result {
        Ok(path) => {
            let note = format!(
                "path={} root_label_filter={:?}",
                display_path(&svc.cfg.out_dir, &path),
                root_label_filter
            );
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: captured_at_unix_ms,
                    kind: "layout_sidecar.written".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some(note),
                    bundle_dir: Some(display_path(&svc.cfg.out_dir, &dumped_dir)),
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
        }
        Err(err) => {
            let note = format!("error={err} root_label_filter={root_label_filter:?}");
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: captured_at_unix_ms,
                    kind: "layout_sidecar.write_failed".to_string(),
                    step_index: Some(step_index as u32),
                    note: Some(note),
                    bundle_dir: Some(display_path(&svc.cfg.out_dir, &dumped_dir)),
                    window: Some(window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
        }
    }

    active.wait_until = None;
    active.screenshot_wait = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    true
}
