use super::selector::SemanticsIndex;
use super::*;

fn normalize_help_query(mut query: String) -> String {
    query.make_ascii_lowercase();
    truncate_string_bytes(&mut query, 64);
    query
}

fn match_rank(haystack: &str, needle: &str) -> Option<u8> {
    if haystack == needle {
        Some(0)
    } else if haystack.starts_with(needle) {
        Some(1)
    } else if haystack.contains(needle) {
        Some(2)
    } else {
        None
    }
}

fn find_inspect_help_matches(
    snapshot: &fret_core::SemanticsSnapshot,
    index: &SemanticsIndex<'_>,
    query: &str,
    redact_text: bool,
) -> Vec<u64> {
    const MAX_MATCHES: usize = 10;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct MatchKey<'a> {
        surface_rank: u8,
        match_rank: u8,
        value: &'a str,
        node_id: u64,
    }

    let needle = query.trim();
    if needle.is_empty() {
        return Vec::new();
    }

    let mut matches: Vec<(MatchKey<'_>, u64)> = Vec::new();

    for node in &snapshot.nodes {
        let node_id = node.id.data().as_ffi();
        if !index.is_selectable(node_id) {
            continue;
        }

        let mut best_key: Option<MatchKey<'_>> = node.test_id.as_deref().and_then(|test_id| {
            match_rank(test_id, needle).map(|match_rank| MatchKey {
                surface_rank: 0,
                match_rank,
                value: test_id,
                node_id,
            })
        });

        if !redact_text {
            if let Some(label) = node.label.as_deref() {
                if let Some(match_rank) = match_rank(label, needle) {
                    let label_key = MatchKey {
                        surface_rank: 1,
                        match_rank,
                        value: label,
                        node_id,
                    };
                    if best_key.map_or(true, |k| label_key < k) {
                        best_key = Some(label_key);
                    }
                }
            }
        }

        let Some(key) = best_key else {
            continue;
        };

        matches.push((key, node_id));
        matches.sort_by(|(a, _), (b, _)| a.cmp(b));
        matches.truncate(MAX_MATCHES);
    }

    matches.into_iter().map(|(_, id)| id).collect()
}

pub(super) fn handle_inspect_help_lock_best_match_and_copy_selector_step(
    svc: &mut UiDiagnosticsService,
    app: &App,
    window: AppWindowId,
    anchor_window: AppWindowId,
    step_index: usize,
    step: UiActionStepV2,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    active: &mut ActiveScript,
    output: &mut UiScriptFrameOutput,
    force_dump_label: &mut Option<String>,
    handoff_to: &mut Option<AppWindowId>,
    stop_script: &mut bool,
    failure_reason: &mut Option<String>,
) -> bool {
    let UiActionStepV2::InspectHelpLockBestMatchAndCopySelector {
        window: target_window,
        query,
        timeout_frames,
    } = step
    else {
        return false;
    };

    active.wait_until = None;
    active.screenshot_wait = None;

    let resolved_window = if let Some(target_window) =
        svc.resolve_window_target_for_active_step(window, anchor_window, target_window.as_ref())
    {
        if target_window != window {
            *handoff_to = Some(target_window);
            output.effects.push(Effect::Redraw(target_window));
            output
                .effects
                .push(Effect::RequestAnimationFrame(target_window));
            output.request_redraw = true;
            return true;
        }
        window
    } else {
        if target_window.is_some() {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-window-not-found"
            ));
            *stop_script = true;
            *failure_reason = Some("window_target_unresolved".to_string());
            output.request_redraw = true;
            return true;
        }
        window
    };

    let mut state = match active.v2_step_state.take() {
        Some(V2StepState::InspectHelpLockBestMatchAndCopySelector(state))
            if state.step_index == step_index =>
        {
            state
        }
        _ => V2InspectHelpLockBestMatchAndCopySelectorState::new(
            step_index,
            resolved_window,
            normalize_help_query(query),
            timeout_frames,
        ),
    };
    let resolved_window = state.window;

    let q = state.query.trim();
    if q.is_empty() {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-empty-query"
        ));
        *stop_script = true;
        *failure_reason = Some("inspect_help_query_empty".to_string());
        output.request_redraw = true;
        return true;
    }

    // Ensure inspection is active and help is visible so users can correlate script failures
    // with on-screen inspector state.
    if !svc.inspect_is_enabled() {
        svc.set_inspect_enabled(true, svc.inspect_consume_clicks());
    }
    svc.inspect_help_open_windows.insert(resolved_window);
    svc.inspect_help_search_query
        .insert(resolved_window, state.query.clone());
    svc.inspect_help_selected_match_index
        .insert(resolved_window, 0);

    let Some(snapshot) = semantics_snapshot else {
        state.remaining_frames = state.remaining_frames.saturating_sub(1);
        if state.remaining_frames == 0 {
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-no-semantics"
            ));
            *stop_script = true;
            *failure_reason = Some("no_semantics_snapshot".to_string());
            output.request_redraw = true;
            return true;
        }

        active.v2_step_state = Some(V2StepState::InspectHelpLockBestMatchAndCopySelector(state));
        output.request_redraw = true;
        output
            .effects
            .push(Effect::RequestAnimationFrame(resolved_window));
        return true;
    };

    let index = SemanticsIndex::new(snapshot);
    let matches = find_inspect_help_matches(snapshot, &index, q, svc.cfg.redact_text);
    svc.set_inspect_help_matches(resolved_window, matches.clone());
    if matches.is_empty() {
        state.remaining_frames = state.remaining_frames.saturating_sub(1);
        if state.remaining_frames == 0 {
            let step_index_u32 = step_index.min(u32::MAX as usize) as u32;
            push_script_event_log(
                active,
                &svc.cfg,
                UiScriptEventLogEntryV1 {
                    unix_ms: unix_ms_now(),
                    kind: "inspect_help_no_match".to_string(),
                    step_index: Some(step_index_u32),
                    note: Some(format!("query={q:?} redact_text={}", svc.cfg.redact_text)),
                    bundle_dir: None,
                    window: Some(resolved_window.data().as_ffi()),
                    tick_id: Some(app.tick_id().0),
                    frame_id: Some(app.frame_id().0),
                    window_snapshot_seq: None,
                },
            );
            *force_dump_label = Some(format!(
                "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-no-match"
            ));
            *stop_script = true;
            *failure_reason = Some("inspect_help_no_match".to_string());
            output.request_redraw = true;
            return true;
        }

        active.v2_step_state = Some(V2StepState::InspectHelpLockBestMatchAndCopySelector(state));
        output.request_redraw = true;
        output
            .effects
            .push(Effect::RequestAnimationFrame(resolved_window));
        return true;
    }

    let node_id = matches[0];
    svc.inspect_focus_down_stack
        .insert(resolved_window, Vec::new());
    svc.inspect_locked_windows.insert(resolved_window);

    let Some(node) = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == node_id)
    else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-node-missing"
        ));
        *stop_script = true;
        *failure_reason = Some("inspect_help_selected_node_missing".to_string());
        output.request_redraw = true;
        return true;
    };

    let selector =
        best_selector_for_node_validated(snapshot, resolved_window, None, node, None, &svc.cfg)
            .or_else(|| best_selector_for_node(snapshot, node, None, &svc.cfg));
    let Some(selector) = selector else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-no-selector"
        ));
        *stop_script = true;
        *failure_reason = Some("inspect_help_no_selector".to_string());
        output.request_redraw = true;
        return true;
    };

    let Ok(selector_json) = serde_json::to_string(&selector) else {
        *force_dump_label = Some(format!(
            "script-step-{step_index:04}-inspect_help_lock_best_match_and_copy_selector-serialize-failed"
        ));
        *stop_script = true;
        *failure_reason = Some("inspect_help_selector_serialize_failed".to_string());
        output.request_redraw = true;
        return true;
    };

    // Mirror the in-app inspector focus/lock semantics so overlay code can copy the selector and
    // subsequent steps can observe a stable `inspect_best_selector_json`.
    svc.inspect_focus_node_id.insert(resolved_window, node_id);
    svc.inspect_focus_selector_json
        .insert(resolved_window, selector_json.clone());
    svc.last_picked_node_id.insert(resolved_window, node_id);
    svc.last_picked_selector_json
        .insert(resolved_window, selector_json.clone());

    output.effects.push(Effect::ClipboardSetText {
        text: selector_json,
    });
    svc.inspect_toast.insert(
        resolved_window,
        inspect::InspectToast {
            message: "inspect: locked match and copied selector".to_string(),
            remaining_frames: 90,
        },
    );

    active.v2_step_state = None;
    active.next_step = active.next_step.saturating_add(1);
    output.request_redraw = true;
    true
}
