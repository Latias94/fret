use std::path::PathBuf;
use std::sync::Arc;

use fret_app::App;
use fret_diag::transport::DiagTransportKind;
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionListV1, DevtoolsSessionRemovedV1,
    DiagTransportMessageV1, UiHitTestExplainAckV1, UiScriptResultV1, UiScriptStageV1, UiSelectorV1,
    UiSemanticsNodeGetAckV1,
};

use crate::{
    State, clear_regression_artifacts, is_abs_path, pack, push_log, refresh_regression_artifacts,
};

pub(crate) fn require_session_selected(app: &mut App, st: &State) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    if selected.is_some() {
        return true;
    }
    push_log(
        app,
        &st.log_lines,
        "no session selected (connect an app or pick a session)",
    );
    false
}

pub(crate) fn drain_ws_messages(app: &mut App, st: &mut State) {
    while let Some(msg) = st.devtools.try_recv() {
        let ty = msg.r#type.clone();
        let compact = match msg.session_id.as_deref() {
            Some(s) => format!("type={ty} session_id={s}"),
            None => format!("type={ty}"),
        };
        push_log(app, &st.log_lines, &compact);

        match ty.as_str() {
            "session.list" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionListV1>(msg.payload) {
                    let sessions = parsed.sessions;
                    let _ = app.models_mut().update(&st.sessions, |v| *v = sessions);
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.added" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionAddedV1>(msg.payload) {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        if let Some(pos) = v
                            .iter()
                            .position(|s| s.session_id == parsed.session.session_id)
                        {
                            v[pos] = parsed.session;
                        } else {
                            v.push(parsed.session);
                        }
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.removed" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionRemovedV1>(msg.payload)
                {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        v.retain(|s| s.session_id != parsed.session_id);
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "pick.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_pick_json, |v| *v = text);
                }
            }
            "inspect.hover" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_inspect_hover_json, |v| *v = text);
                }
            }
            "inspect.focus" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_inspect_focus_json, |v| *v = text);
                }
            }
            "overlay.summary" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_overlay_summary_json, |v| *v = text);
                }
            }
            "script.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload.clone())
                {
                    let _ = app
                        .models_mut()
                        .update(&st.script_last_stage, |v| *v = Some(parsed.stage.clone()));
                    let _ = app
                        .models_mut()
                        .update(&st.script_last_step_index, |v| *v = parsed.step_index);
                    let _ = app.models_mut().update(&st.script_last_reason, |v| {
                        *v = parsed.reason.map(Into::into);
                    });
                    let _ = app.models_mut().update(&st.script_last_bundle_dir, |v| {
                        *v = parsed.last_bundle_dir.clone().map(Into::into);
                    });

                    if let Some(out_dir) = app
                        .models()
                        .read(&st.target_out_dir, |v| v.clone())
                        .ok()
                        .flatten()
                        .map(|s| s.to_string())
                    {
                        if let Some(rel) = parsed.last_bundle_dir.as_deref() {
                            if let Some(abs) = resolve_bundle_dir_abs(&out_dir, rel) {
                                let _ = app.models_mut().update(&st.last_bundle_dir_abs, |v| {
                                    *v = Some(abs.into());
                                });
                            }
                        }
                    }
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_script_result_json, |v| *v = text);
                }
                maybe_start_pack_after_run(app, st);
            }
            "bundle.dumped" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Some(ts) = msg.payload.get("exported_unix_ms").and_then(|v| v.as_u64()) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_bundle_dump_exported_unix_ms, |v| *v = Some(ts));
                }
                if let Some(bundle) = msg.payload.get("bundle") {
                    if let Ok(text) = serde_json::to_string_pretty(bundle) {
                        let _ = app
                            .models_mut()
                            .update(&st.last_bundle_dump_bundle_json, |v| {
                                *v = Some(Arc::<str>::from(text));
                            });
                    }
                }
                if let Some(out_dir) = msg.payload.get("out_dir").and_then(|v| v.as_str()) {
                    let _ = app.models_mut().update(&st.target_out_dir, |v| {
                        *v = Some(Arc::<str>::from(out_dir.to_string()));
                    });
                }
                if let (Some(out_dir), Some(dir)) = (
                    msg.payload.get("out_dir").and_then(|v| v.as_str()),
                    msg.payload.get("dir").and_then(|v| v.as_str()),
                ) {
                    if let Some(abs) = resolve_bundle_dir_abs(out_dir, dir) {
                        let _ = app.models_mut().update(&st.last_bundle_dir_abs, |v| {
                            *v = Some(Arc::<str>::from(abs));
                        });
                    }
                }
                if msg.payload.get("bundle").is_none() {
                    let loaded = msg
                        .payload
                        .get("out_dir")
                        .and_then(|v| v.as_str())
                        .zip(msg.payload.get("dir").and_then(|v| v.as_str()))
                        .and_then(|(out_dir, dir)| resolve_bundle_dir_abs(out_dir, dir))
                        .and_then(|abs_dir| {
                            let path = PathBuf::from(abs_dir).join("bundle.json");
                            std::fs::read_to_string(path).ok()
                        })
                        .map(Arc::<str>::from);
                    let _ = app
                        .models_mut()
                        .update(&st.last_bundle_dump_bundle_json, |v| *v = loaded);
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_bundle_json, |v| *v = text);
                }
                refresh_regression_artifacts(app, st);
                maybe_start_pack_after_run(app, st);
            }
            "screenshot.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_screenshot_json, |v| *v = text);
                }
            }
            "semantics.node.get_ack" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                let payload = msg.payload.clone();
                if let Ok(parsed) =
                    serde_json::from_value::<UiSemanticsNodeGetAckV1>(payload.clone())
                {
                    let expected_node_id = app
                        .models()
                        .read(&st.semantics_selected_id, |v| *v)
                        .ok()
                        .flatten();
                    let expected_window_ffi = app
                        .models()
                        .read(&st.semantics_cache, |v| v.as_ref().map(|i| i.window))
                        .ok()
                        .flatten();
                    if expected_node_id != Some(parsed.node_id)
                        || expected_window_ffi != Some(parsed.window)
                    {
                        continue;
                    }

                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_node_live_status, |v| {
                            *v = Some(Arc::<str>::from(parsed.status));
                        });
                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_node_live_updated_unix_ms, |v| {
                            *v = parsed.captured_unix_ms
                        });
                    let _ =
                        app.models_mut()
                            .update(&st.semantics_selected_node_live_children, |v| {
                                *v = parsed.children;
                            });

                    if let Some(node) = parsed.node {
                        if let Ok(text) = serde_json::to_string_pretty(&node) {
                            let _ = app
                                .models_mut()
                                .update(&st.semantics_selected_node_live_json, |v| *v = text);
                        }
                    } else {
                        let _ = app
                            .models_mut()
                            .update(&st.semantics_selected_node_live_json, |v| v.clear());
                    }
                } else if let Ok(text) = serde_json::to_string_pretty(&payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_node_live_json, |v| *v = text);
                }
            }
            "hit_test.explain_ack" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                let payload = msg.payload.clone();
                if let Ok(parsed) = serde_json::from_value::<UiHitTestExplainAckV1>(payload.clone())
                {
                    let expected_node_id = app
                        .models()
                        .read(&st.semantics_selected_id, |v| *v)
                        .ok()
                        .flatten();
                    let expected_window_ffi = app
                        .models()
                        .read(&st.semantics_cache, |v| v.as_ref().map(|i| i.window))
                        .ok()
                        .flatten();
                    let target_matches = matches!(
                        &parsed.target,
                        UiSelectorV1::NodeId { node, .. } if Some(*node) == expected_node_id
                    );
                    if !target_matches || expected_window_ffi != Some(parsed.window) {
                        continue;
                    }

                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_hit_test_explain_status, |v| {
                            *v = Some(Arc::<str>::from(parsed.status.clone()))
                        });
                    let _ = app.models_mut().update(
                        &st.semantics_selected_hit_test_explain_updated_unix_ms,
                        |v| *v = parsed.captured_unix_ms,
                    );
                    let summary = summarize_hit_test_explain(&parsed);
                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_hit_test_explain_summary, |v| {
                            *v = summary
                        });
                    if let Ok(text) = serde_json::to_string_pretty(&parsed) {
                        let _ = app
                            .models_mut()
                            .update(&st.semantics_selected_hit_test_explain_json, |v| *v = text);
                    }
                } else if let Ok(text) = serde_json::to_string_pretty(&payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_hit_test_explain_json, |v| *v = text);
                    let _ = app
                        .models_mut()
                        .update(&st.semantics_selected_hit_test_explain_summary, |v| {
                            v.clear()
                        });
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn sync_selected_session_to_client(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();

    if selected.as_deref() == st.applied_session_id.as_deref() {
        return;
    }

    st.devtools
        .set_default_session_id(selected.as_ref().map(|s| s.to_string()));
    st.applied_session_id = selected;

    st.live_semantics_last_target = None;
    st.live_semantics_last_sent_unix_ms = None;
    st.live_semantics_last_force_nonce = 0;
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_node_live_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_node_live_status, |v| *v = None);
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_node_live_updated_unix_ms, |v| {
            *v = None
        });
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_node_live_children, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_hit_test_explain_json, |v| v.clear());
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_hit_test_explain_summary, |v| {
            v.clear()
        });
    let _ = app
        .models_mut()
        .update(&st.semantics_selected_hit_test_explain_status, |v| {
            *v = None
        });
    let _ = app.models_mut().update(
        &st.semantics_selected_hit_test_explain_updated_unix_ms,
        |v| *v = None,
    );
    if st.cfg.transport == DiagTransportKind::WebSocket {
        let _ = app.models_mut().update(&st.target_out_dir, |v| *v = None);
        clear_regression_artifacts(app, st);
    }
}

pub(crate) fn maybe_request_semantics_node_details(app: &mut App, st: &mut State) {
    if st.devtools.client().kind() != DiagTransportKind::WebSocket {
        return;
    }

    let live_enabled = app
        .models()
        .read(&st.semantics_live_enabled, |v| *v)
        .unwrap_or(true);
    if !live_enabled {
        st.live_semantics_last_target = None;
        st.live_semantics_last_sent_unix_ms = None;
        st.live_semantics_last_force_nonce = 0;
        return;
    }

    let selected_session_id = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    if selected_session_id.is_none() {
        st.live_semantics_last_target = None;
        st.live_semantics_last_sent_unix_ms = None;
        return;
    }

    let selected_node_id = app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let Some(selected_node_id) = selected_node_id else {
        st.live_semantics_last_target = None;
        st.live_semantics_last_sent_unix_ms = None;
        return;
    };

    let window_ffi = app
        .models()
        .read(&st.semantics_cache, |v| v.as_ref().map(|i| i.window))
        .ok()
        .flatten();
    let Some(window_ffi) = window_ffi else {
        return;
    };

    let now = unix_ms_now();
    let force_nonce = app
        .models()
        .read(&st.semantics_live_force_nonce, |v| *v)
        .unwrap_or(0);
    let target = (window_ffi, selected_node_id);
    let decision = live_semantics_request_decision(
        st.live_semantics_last_target,
        st.live_semantics_last_sent_unix_ms,
        st.live_semantics_last_force_nonce,
        target,
        force_nonce,
        now,
    );
    if !decision.should_request {
        return;
    }

    st.live_semantics_last_target = Some(target);
    st.live_semantics_last_sent_unix_ms = Some(now);
    st.live_semantics_last_force_nonce = force_nonce;

    let _ = st
        .devtools
        .semantics_node_get(None, window_ffi, selected_node_id);
    let _ = st.devtools.hit_test_explain(
        None,
        window_ffi,
        UiSelectorV1::NodeId {
            node: selected_node_id,
            root_z_index: None,
        },
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LiveSemanticsRequestDecision {
    should_request: bool,
    selection_changed: bool,
    force_refresh: bool,
    due: bool,
}

fn live_semantics_request_decision(
    last_target: Option<(u64, u64)>,
    last_sent_unix_ms: Option<u64>,
    last_force_nonce: u64,
    target: (u64, u64),
    force_nonce: u64,
    now_unix_ms: u64,
) -> LiveSemanticsRequestDecision {
    let selection_changed = last_target != Some(target);
    let force_refresh = force_nonce != last_force_nonce;
    let due = match last_sent_unix_ms {
        None => true,
        Some(prev) => now_unix_ms.saturating_sub(prev) >= 1000,
    };
    LiveSemanticsRequestDecision {
        should_request: selection_changed || force_refresh || due,
        selection_changed,
        force_refresh,
        due,
    }
}

fn unix_ms_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn ensure_session_selection_is_valid(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let sessions = app
        .models()
        .read(&st.sessions, |v| v.clone())
        .unwrap_or_default();

    let new_selected = selected_session_after_session_list_refresh(
        selected.as_deref(),
        sessions.iter().map(|s| s.session_id.as_str()),
    )
    .map(Arc::<str>::from);
    let _ = app
        .models_mut()
        .update(&st.selected_session_id, |v| *v = new_selected);
}

fn selected_session_after_session_list_refresh<'a>(
    selected: Option<&str>,
    sessions: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let mut first = None::<String>;
    for session in sessions {
        if first.is_none() {
            first = Some(session.to_string());
        }
        if selected.is_some_and(|selected| selected == session) {
            return selected.map(str::to_string);
        }
    }
    first
}

fn message_matches_selected_session(
    app: &mut App,
    st: &State,
    msg: &DiagTransportMessageV1,
) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    message_session_matches_selected(selected.as_deref(), msg.session_id.as_deref())
}

fn message_session_matches_selected(selected: Option<&str>, message_session: Option<&str>) -> bool {
    match selected {
        Some(selected) => message_session == Some(selected),
        None => true,
    }
}

fn maybe_start_pack_after_run(app: &mut App, st: &mut State) {
    let pack_after = app
        .models()
        .read(&st.script_pack_after_run, |v| *v)
        .unwrap_or(false);
    if !pack_after {
        return;
    }

    let stage = app
        .models()
        .read(&st.script_last_stage, |v| v.clone())
        .ok()
        .flatten();
    if !matches!(
        stage,
        Some(UiScriptStageV1::Passed) | Some(UiScriptStageV1::Failed)
    ) {
        return;
    }

    let has_bundle_dir = app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.is_some())
        .unwrap_or(false);
    let has_bundle_payload = app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.is_some())
        .unwrap_or(false);
    if !(has_bundle_dir || has_bundle_payload) {
        return;
    }

    if let Err(err) = pack::start_pack_last_bundle(app, st) {
        push_log(app, &st.log_lines, &format!("pack refused: {err}"));
    }
    let _ = app
        .models_mut()
        .update(&st.script_pack_after_run, |v| *v = false);
}

fn resolve_bundle_dir_abs(out_dir: &str, dir: &str) -> Option<String> {
    let dir = dir.trim();
    if dir.is_empty() {
        return None;
    }
    if is_abs_path(dir) {
        return Some(dir.to_string());
    }

    let out_dir = out_dir.trim();
    if out_dir.is_empty() {
        return Some(dir.to_string());
    }
    let base = PathBuf::from(out_dir);
    Some(base.join(dir).to_string_lossy().to_string())
}

fn summarize_hit_test_explain(ack: &UiHitTestExplainAckV1) -> String {
    let mut lines = Vec::new();
    lines.push(format!("hittable={}", option_bool_text(ack.hittable)));
    if let Some(reason) = ack.reason.as_deref() {
        lines.push(format!("reason={reason}"));
    }
    if let Some(hit_test) = ack.hit_test.as_ref() {
        lines.push(format!(
            "includes_intended={} hit_path_contains_intended={}",
            option_bool_text(hit_test.includes_intended),
            option_bool_text(hit_test.hit_path_contains_intended)
        ));
        if let Some(reason) = hit_test.blocking_reason.as_deref() {
            lines.push(format!("blocking_reason={reason}"));
        }
        if let Some(root) = hit_test.blocking_root {
            lines.push(format!("blocking_root={root}"));
        }
        if let Some(layer_id) = hit_test.blocking_layer_id {
            lines.push(format!("blocking_layer_id={layer_id}"));
        }
        if let Some(explain) = hit_test.routing_explain.as_deref() {
            lines.push(format!("routing_explain={explain}"));
        }
        lines.push(format!(
            "intended_node_id={} hit_node_id={} hit_semantics_node_id={}",
            option_u64_text(hit_test.intended_node_id),
            option_u64_text(hit_test.hit_node_id),
            option_u64_text(hit_test.hit_semantics_node_id)
        ));
        if let Some(test_id) = hit_test.hit_semantics_test_id.as_deref() {
            lines.push(format!("hit_semantics_test_id={test_id}"));
        }
    }
    lines.join(
        "
",
    )
}

fn option_bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn option_u64_text(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_session_refresh_keeps_valid_selection_or_falls_back_to_first_session() {
        let sessions = ["session-a", "session-b"];

        assert_eq!(
            selected_session_after_session_list_refresh(Some("session-b"), sessions),
            Some("session-b".to_string())
        );
        assert_eq!(
            selected_session_after_session_list_refresh(Some("stale-session"), sessions),
            Some("session-a".to_string())
        );
        assert_eq!(
            selected_session_after_session_list_refresh(None, sessions),
            Some("session-a".to_string())
        );
        assert_eq!(
            selected_session_after_session_list_refresh(Some("stale-session"), []),
            None
        );
    }

    #[test]
    fn message_session_matching_uses_selected_session_when_present() {
        assert!(message_session_matches_selected(None, None));
        assert!(message_session_matches_selected(None, Some("session-b")));
        assert!(message_session_matches_selected(
            Some("session-b"),
            Some("session-b")
        ));
        assert!(!message_session_matches_selected(
            Some("session-b"),
            Some("session-a")
        ));
        assert!(!message_session_matches_selected(Some("session-b"), None));
    }

    #[test]
    fn live_semantics_request_decision_throttles_unchanged_selection_to_one_hz() {
        let target = (7, 42);
        let decision =
            live_semantics_request_decision(Some(target), Some(10_000), 3, target, 3, 10_999);

        assert_eq!(
            decision,
            LiveSemanticsRequestDecision {
                should_request: false,
                selection_changed: false,
                force_refresh: false,
                due: false,
            }
        );

        let decision =
            live_semantics_request_decision(Some(target), Some(10_000), 3, target, 3, 11_000);
        assert!(decision.should_request);
        assert!(decision.due);
        assert!(!decision.selection_changed);
        assert!(!decision.force_refresh);
    }

    #[test]
    fn live_semantics_request_decision_allows_selection_change_and_manual_refresh() {
        let previous = (7, 42);
        let next = (7, 43);

        let selection = live_semantics_request_decision(
            Some(previous),
            Some(10_500),
            3,
            next,
            3,
            10_600,
        );
        assert!(selection.should_request);
        assert!(selection.selection_changed);
        assert!(!selection.force_refresh);
        assert!(!selection.due);

        let manual = live_semantics_request_decision(
            Some(previous),
            Some(10_500),
            3,
            previous,
            4,
            10_600,
        );
        assert!(manual.should_request);
        assert!(!manual.selection_changed);
        assert!(manual.force_refresh);
        assert!(!manual.due);
    }
}
