use super::*;

pub(super) fn handle_keyboard_text_steps(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&UiTree<App>>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    match step {
        UiActionStepV2::PressKey {
            key,
            modifiers,
            repeat,
        } => {
            if let Some(key) = parse_key_code(&key) {
                let note = format!("press_key key={key:?} mods={modifiers:?} repeat={repeat}");
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    semantics_snapshot,
                    ui,
                    step_index as u32,
                    None,
                    None,
                    note.as_str(),
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    note.as_str(),
                );
                record_overlay_placement_trace(
                    &mut active.overlay_placement_trace,
                    element_runtime,
                    semantics_snapshot,
                    window,
                    step_index as u32,
                    note.as_str(),
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(press_key_events(key, modifiers, repeat));
                active.wait_until = None;
                active.screenshot_wait = None;
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-press_key"));
                }
            } else {
                *force_dump_label =
                    Some(format!("script-step-{step_index:04}-press_key-unknown-key"));
                *stop_script = true;
                *failure_reason = Some(format!("unknown_key: {key}"));
                output.request_redraw = true;
            }
            true
        }
        UiActionStepV2::PressShortcut { shortcut, repeat } => {
            active.wait_until = None;
            active.screenshot_wait = None;

            if let Some((key, modifiers)) = parse_shortcut(&shortcut) {
                let note = format!("press_shortcut key={key:?} mods={modifiers:?} repeat={repeat}");
                record_focus_trace(
                    &mut active.focus_trace,
                    app,
                    window,
                    element_runtime,
                    semantics_snapshot,
                    ui,
                    step_index as u32,
                    None,
                    None,
                    note.as_str(),
                );
                record_web_ime_trace(
                    &mut active.web_ime_trace,
                    app,
                    step_index as u32,
                    note.as_str(),
                );
                record_overlay_placement_trace(
                    &mut active.overlay_placement_trace,
                    element_runtime,
                    semantics_snapshot,
                    window,
                    step_index as u32,
                    note.as_str(),
                );
                active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
                output
                    .events
                    .extend(press_key_events(key, modifiers, repeat));
                active.next_step = active.next_step.saturating_add(1);
                output.request_redraw = true;
                if svc.cfg.script_auto_dump {
                    *force_dump_label = Some(format!("script-step-{step_index:04}-press_shortcut"));
                }
            } else {
                *force_dump_label = Some(format!(
                    "script-step-{step_index:04}-press_shortcut-parse-failed"
                ));
                *stop_script = true;
                *failure_reason = Some(format!("invalid_shortcut: {shortcut}"));
                output.request_redraw = true;
            }
            true
        }
        UiActionStepV2::TypeText { text } => {
            output.events.push(Event::TextInput(text));
            active.wait_until = None;
            active.screenshot_wait = None;
            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-type_text"));
            }
            true
        }
        UiActionStepV2::Ime { event } => {
            active.wait_until = None;
            active.screenshot_wait = None;

            let note = format!("ime_event kind={}", ime_event_kind_name(&event));
            record_focus_trace(
                &mut active.focus_trace,
                app,
                window,
                element_runtime,
                semantics_snapshot,
                ui,
                step_index as u32,
                None,
                None,
                note.as_str(),
            );
            record_web_ime_trace(
                &mut active.web_ime_trace,
                app,
                step_index as u32,
                note.as_str(),
            );
            record_overlay_placement_trace(
                &mut active.overlay_placement_trace,
                element_runtime,
                semantics_snapshot,
                window,
                step_index as u32,
                note.as_str(),
            );

            active.last_injected_step = Some(step_index.min(u32::MAX as usize) as u32);
            output.events.push(Event::Ime(ime_event_from_v1(&event)));
            active.next_step = active.next_step.saturating_add(1);
            output.request_redraw = true;
            if svc.cfg.script_auto_dump {
                *force_dump_label = Some(format!("script-step-{step_index:04}-ime"));
            }
            true
        }
        _ => false,
    }
}
