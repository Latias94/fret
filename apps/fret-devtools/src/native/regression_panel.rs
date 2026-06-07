use std::path::PathBuf;
use std::sync::Arc;

use fret_app::{App, Effect};
use fret_core::Px;
use fret_diag::regression_summary::{
    regression_bundle_followup_command_lines, regression_bundle_followup_commands,
};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::command_catalog::{
    CMD_COPY_FOLLOWUP_RESULT_COMMAND, CMD_COPY_FOLLOWUP_RESULT_JSON, CMD_COPY_FOLLOWUP_RESULT_PATH,
    CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH, CMD_OPEN_FOLLOWUP_RESULT_JSON,
    CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT, CMD_REGRESSION_PACK_SELECTED_BUNDLE,
    CMD_REGRESSION_REFRESH, CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS,
    CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF, CMD_REGRESSION_RUN_FOLLOWUP_MEMORY,
    CMD_REGRESSION_RUN_FOLLOWUP_STATS, CMD_REGRESSION_RUN_FOLLOWUP_TRACE,
    CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE, CMD_REGRESSION_RUN_FOOTPRINT_COMPARE,
    CMD_REGRESSION_RUN_VISUAL_COMPARE, CMD_REGRESSION_SUMMARIZE,
};
use super::followup_panel::{
    followup_history_list, runnable_followup_command_actions, selected_followup_readiness_lines,
};
use super::ui_primitives::{diag_card, diag_section, text_blob_sized};
use super::{
    State, followup, load_regression_summary_drilldown, regression_failing_summary_rows,
    repo_root_from_script_paths, resolve_repo_or_abs_path,
    selected_followup_history_filter_dirs_from_bundle_dirs,
};

pub(super) fn regression_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let theme = cx.theme_snapshot();
    let loaded_dir = cx
        .app
        .models()
        .read(&st.regression_loaded_dir, |v| v.clone())
        .ok()
        .flatten();
    let error = cx
        .app
        .models()
        .read(&st.regression_last_error, |v| v.clone())
        .ok()
        .flatten();
    let dashboard = cx
        .app
        .models()
        .read(&st.regression_dashboard_human, |v| v.clone())
        .unwrap_or_default();
    let index_json = cx
        .app
        .models()
        .read(&st.regression_index_json, |v| v.clone())
        .unwrap_or_default();
    let summary_json = cx
        .app
        .models()
        .read(&st.regression_summary_json, |v| v.clone())
        .unwrap_or_default();
    let selected_summary_path = cx
        .app
        .models()
        .read(&st.regression_selected_summary_path, |v| v.clone())
        .ok()
        .flatten();
    let selected_summary_json = cx
        .app
        .models()
        .read(&st.regression_selected_summary_json, |v| v.clone())
        .unwrap_or_default();
    let selected_bundle_dirs = cx
        .app
        .models()
        .read(&st.regression_selected_bundle_dirs, |v| v.clone())
        .unwrap_or_default();
    let selected_capability_sources = cx
        .app
        .models()
        .read(&st.regression_selected_capability_sources, |v| v.clone())
        .unwrap_or_default();
    let selected_capabilities_checks = cx
        .app
        .models()
        .read(&st.regression_selected_capabilities_checks, |v| v.clone())
        .unwrap_or_default();
    let selected_perf_evidence = cx
        .app
        .models()
        .read(&st.regression_selected_perf_evidence, |v| v.clone())
        .unwrap_or_default();
    let selected_first_open_evidence = cx
        .app
        .models()
        .read(&st.regression_selected_first_open_evidence, |v| v.clone())
        .unwrap_or_default();
    let selected_share_artifacts = cx
        .app
        .models()
        .read(&st.regression_selected_share_artifacts, |v| v.clone())
        .unwrap_or_default();
    let selected_error = cx
        .app
        .models()
        .read(&st.regression_selected_error, |v| v.clone())
        .ok()
        .flatten();
    let can_refresh = cx
        .app
        .models()
        .read(&st.target_out_dir, |v| v.is_some())
        .unwrap_or(false);
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let can_pack_selected_bundle = !selected_bundle_dirs.is_empty();
    let summarize_in_flight = cx
        .app
        .models()
        .read(&st.summarize_in_flight, |v| *v)
        .unwrap_or(false);
    let summarize_last_error = cx
        .app
        .models()
        .read(&st.summarize_last_error, |v| v.clone())
        .ok()
        .flatten();
    let followup_in_flight = cx
        .app
        .models()
        .read(&st.followup_in_flight, |v| *v)
        .unwrap_or(false);
    let followup_last_command_line = cx
        .app
        .models()
        .read(&st.followup_last_command_line, |v| v.clone())
        .ok()
        .flatten();
    let followup_last_result_path = cx
        .app
        .models()
        .read(&st.followup_last_result_path, |v| v.clone())
        .ok()
        .flatten();
    let followup_result_history = cx
        .app
        .models()
        .read(&st.followup_result_history, |v| v.clone())
        .unwrap_or_default();
    let followup_selected_result_path = cx
        .app
        .models()
        .read(&st.followup_selected_result_path, |v| v.clone())
        .ok()
        .flatten();
    let followup_last_error = cx
        .app
        .models()
        .read(&st.followup_last_error, |v| v.clone())
        .ok()
        .flatten();
    let followup_baseline_bundle_or_dir = cx
        .app
        .models()
        .read(&st.followup_baseline_bundle_or_dir, |v| v.clone())
        .unwrap_or_default();
    let followup_baseline_session = cx
        .app
        .models()
        .read(&st.followup_baseline_session, |v| v.clone())
        .unwrap_or_default();
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let selected_followup_history_filter_dirs =
        selected_followup_history_filter_dirs_from_bundle_dirs(
            &st.script_paths,
            &selected_bundle_dirs,
        );
    let selected_followup_history_entries =
        followup::followup_result_history_entries_for_selected_bundle(
            &followup_result_history,
            selected_followup_history_filter_dirs
                .iter()
                .map(|value| value.as_str()),
        );
    let selected_followup_result_entry = followup::followup_result_history_selected_or_latest_entry(
        &followup_result_history,
        selected_followup_history_filter_dirs
            .iter()
            .map(|value| value.as_str()),
        followup_selected_result_path.as_deref(),
    );
    let selected_followup_result_path = selected_followup_result_entry
        .as_ref()
        .map(|entry| entry.result_path.clone());
    let selected_followup_result_json = selected_followup_result_entry
        .as_ref()
        .map(|entry| entry.result_json.clone())
        .unwrap_or_default();
    let selected_followup_trace_artifact_path =
        followup::followup_trace_artifact_path_from_result_json(&selected_followup_result_json)
            .map(|path| resolve_repo_or_abs_path(&repo_root, &path).to_string_lossy().to_string());
    let failing_rows = regression_failing_summary_rows(&index_json, 10);
    let failing_count = failing_rows.len();
    let selected_bundle_count = selected_bundle_dirs.len();
    let selected_capability_source_count = selected_capability_sources.len();
    let selected_capabilities_check_count = selected_capabilities_checks.len();
    let selected_perf_evidence_count = selected_perf_evidence.len();
    let selected_first_open_evidence_count = selected_first_open_evidence.len();
    let selected_share_artifact_count = selected_share_artifacts.len();
    let summarize_status_line = {
        let err = summarize_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "summarize_in_flight={} summarize_last_error={err}",
            if summarize_in_flight { "true" } else { "false" }
        )
    };
    let followup_status_line = {
        let command = followup_last_command_line
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let result = followup_last_result_path
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        let err = followup_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "followup_in_flight={} last_followup_command={command} last_followup_result={result} followup_last_error={err}",
            if followup_in_flight { "true" } else { "false" }
        )
    };
    let loaded_dir_line = loaded_dir
        .as_deref()
        .map(|v| format!("Artifacts root: {v}"))
        .unwrap_or_else(|| "Artifacts root: <not loaded>".to_string());
    let aggregate_preview = if !dashboard.trim().is_empty() {
        dashboard.clone()
    } else if let Some(err) = error.as_deref() {
        format!("Regression load error: {err}")
    } else {
        "No aggregate dashboard loaded yet. Use Refresh or Summarize against the current artifacts root.".to_string()
    };

    let _aggregate_content = {
        let mut parts: Vec<String> = Vec::new();
        if let Some(dir) = loaded_dir.as_deref() {
            parts.push(format!("loaded_dir: {dir}"));
        }
        if let Some(err) = error.as_deref() {
            parts.push(format!("error: {err}"));
        }
        if !dashboard.trim().is_empty() {
            parts.push("dashboard:".to_string());
            parts.push(dashboard);
        }
        if !index_json.trim().is_empty() {
            parts.push("regression.index.json:".to_string());
            parts.push(index_json.clone());
        }
        if !summary_json.trim().is_empty() {
            parts.push("regression.summary.json:".to_string());
            parts.push(summary_json.clone());
        }
        if parts.is_empty() {
            "<empty>".to_string()
        } else {
            parts.join(
                "

",
            )
        }
    };

    let failing_list = if failing_rows.is_empty() {
        shadcn::ScrollArea::new([
            cx.text("No non-passing summaries in the current regression index.")
        ])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(Px(220.0)),
        )
        .into_element(cx)
    } else {
        let mut rows: Vec<AnyElement> = Vec::new();
        for row in failing_rows {
            let resolved_summary_path = resolve_repo_or_abs_path(&repo_root, &row.path);
            let resolved_summary_path_str = resolved_summary_path.to_string_lossy().to_string();
            let is_selected = selected_summary_path
                .as_deref()
                .is_some_and(|selected| selected == resolved_summary_path_str);
            let title = row.path.clone();
            let lane_label = format!("lane {}", row.lane);
            let failures_label = format!("failures {}", row.failures);
            let items_label = format!("items {}", row.items_total);
            let resolved_path_label = format!("Resolved path: {}", resolved_summary_path_str);
            let selected_summary_path_model = st.regression_selected_summary_path.clone();
            let selected_summary_json_model = st.regression_selected_summary_json.clone();
            let selected_bundle_dirs_model = st.regression_selected_bundle_dirs.clone();
            let selected_capability_sources_model =
                st.regression_selected_capability_sources.clone();
            let selected_capabilities_checks_model =
                st.regression_selected_capabilities_checks.clone();
            let selected_perf_evidence_model = st.regression_selected_perf_evidence.clone();
            let selected_first_open_evidence_model =
                st.regression_selected_first_open_evidence.clone();
            let selected_share_artifacts_model = st.regression_selected_share_artifacts.clone();
            let selected_error_model = st.regression_selected_error.clone();
            let log_lines_model = st.log_lines.clone();
            let copy_path = resolved_summary_path_str.clone();
            let select_path = resolved_summary_path_str.clone();
            let on_select: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let path = PathBuf::from(&select_path);
                    match load_regression_summary_drilldown(&path) {
                        Ok(data) => {
                            let _ = host.models_mut().update(&selected_summary_path_model, |v| {
                                *v = Some(Arc::<str>::from(select_path.clone()));
                            });
                            let _ = host.models_mut().update(&selected_summary_json_model, |v| {
                                *v = data.summary_json;
                            });
                            let _ = host.models_mut().update(&selected_bundle_dirs_model, |v| {
                                *v = data.bundle_dirs.into_iter().map(Arc::<str>::from).collect();
                            });
                            let _ =
                                host.models_mut()
                                    .update(&selected_capability_sources_model, |v| {
                                        *v = data
                                            .capability_sources
                                            .into_iter()
                                            .map(Arc::<str>::from)
                                            .collect();
                                    });
                            let _ = host.models_mut().update(
                                &selected_capabilities_checks_model,
                                |v| {
                                    *v = data
                                        .capabilities_check_paths
                                        .into_iter()
                                        .map(Arc::<str>::from)
                                        .collect();
                                },
                            );
                            let _ = host.models_mut().update(&selected_perf_evidence_model, |v| {
                                *v = data
                                    .perf_evidence_lines
                                    .into_iter()
                                    .map(Arc::<str>::from)
                                    .collect();
                            });
                            let _ = host.models_mut().update(
                                &selected_first_open_evidence_model,
                                |v| {
                                    *v = data
                                        .first_open_evidence_lines
                                        .into_iter()
                                        .map(Arc::<str>::from)
                                        .collect();
                                },
                            );
                            let _ =
                                host.models_mut()
                                    .update(&selected_share_artifacts_model, |v| {
                                        *v = data
                                            .share_artifacts
                                            .into_iter()
                                            .map(Arc::<str>::from)
                                            .collect();
                                    });
                            let _ = host
                                .models_mut()
                                .update(&selected_error_model, |v| *v = None);
                        }
                        Err(err) => {
                            let _ = host.models_mut().update(&selected_summary_path_model, |v| {
                                *v = Some(Arc::<str>::from(select_path.clone()));
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_summary_json_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_bundle_dirs_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_capability_sources_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_capabilities_checks_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_perf_evidence_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_first_open_evidence_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_share_artifacts_model, |v| v.clear());
                            let _ = host.models_mut().update(&selected_error_model, |v| {
                                *v = Some(Arc::<str>::from(format!(
                                    "failed to load selected regression summary {}: {err}",
                                    path.display()
                                )))
                            });
                            let _ = host.models_mut().update(&log_lines_model, |v| {
                                v.push(Arc::<str>::from(format!(
                                    "regression summary drill-down load failed: {}",
                                    path.display()
                                )));
                                if v.len() > 2000 {
                                    let drain = v.len().saturating_sub(2000);
                                    v.drain(0..drain);
                                }
                            });
                        }
                    }
                    host.request_redraw(action_cx.window);
                });
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: copy_path.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            let title_text = cx.text(title);
            let resolved_path_text = cx.text(resolved_path_label);
            let badges = ui::h_row(|cx| {
                [
                    shadcn::Badge::new(if is_selected {
                        "Selected"
                    } else {
                        "Non-passing"
                    })
                    .variant(if is_selected {
                        shadcn::BadgeVariant::Secondary
                    } else {
                        shadcn::BadgeVariant::Destructive
                    })
                    .into_element(cx),
                    shadcn::Badge::new(lane_label)
                        .variant(shadcn::BadgeVariant::Outline)
                        .into_element(cx),
                    shadcn::Badge::new(failures_label)
                        .variant(shadcn::BadgeVariant::Destructive)
                        .into_element(cx),
                    shadcn::Badge::new(items_label)
                        .variant(shadcn::BadgeVariant::Outline)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
            let actions = ui::h_row(|cx| {
                [
                    shadcn::Button::new(if is_selected {
                        "Opened"
                    } else {
                        "Open details"
                    })
                    .variant(if is_selected {
                        shadcn::ButtonVariant::Secondary
                    } else {
                        shadcn::ButtonVariant::Ghost
                    })
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_select)
                    .into_element(cx),
                    shadcn::Button::new("Copy path")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
            rows.push(
                shadcn::Card::new([shadcn::CardContent::new([
                    badges,
                    title_text,
                    resolved_path_text,
                    actions,
                ])
                .into_element(cx)])
                .into_element(cx),
            );
        }
        shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
            .gap(fret_ui_kit::Space::N2)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .min_h(Px(260.0)),
        )
        .into_element(cx)
    };

    let selected_bundle_dirs_text = selected_bundle_dirs
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_capability_sources_text = selected_capability_sources
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_capabilities_checks_text = selected_capabilities_checks
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_perf_evidence_text = selected_perf_evidence
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_first_open_evidence_text = selected_first_open_evidence
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_share_artifacts_text = selected_share_artifacts
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    let selected_followup_commands =
        regression_bundle_followup_commands(selected_bundle_dirs.iter().map(|v| v.as_ref()));
    let selected_runnable_followup_command_lines = selected_followup_commands
        .iter()
        .filter(|command| !command.requires_baseline)
        .map(|command| command.display_line())
        .collect::<Vec<_>>();
    let selected_manual_followup_command_lines = selected_followup_commands
        .iter()
        .filter(|command| command.requires_baseline)
        .map(|command| command.display_line())
        .collect::<Vec<_>>();
    let selected_runnable_followup_count = selected_runnable_followup_command_lines.len();
    let selected_manual_followup_count = selected_manual_followup_command_lines.len();
    let selected_followup_readiness_lines = selected_followup_readiness_lines(
        selected_bundle_count,
        &selected_followup_commands,
        &followup_baseline_bundle_or_dir,
        &followup_baseline_session,
    );
    let selected_followup_command_lines =
        regression_bundle_followup_command_lines(selected_bundle_dirs.iter().map(|v| v.as_ref()));
    let selected_followup_commands_text = selected_followup_command_lines.join("\r\n");
    let selected_followup_commands_display = if selected_followup_command_lines.is_empty() {
        "Select a non-passing summary with bundle_dir evidence to generate concrete follow-up commands.".to_string()
    } else {
        selected_followup_commands_text.clone()
    };
    let selected_runnable_followup_commands_text =
        selected_runnable_followup_command_lines.join("\r\n");
    let selected_runnable_followup_commands_display =
        if selected_runnable_followup_command_lines.is_empty() {
            "No bundle-local follow-up command is runnable from this selection yet.".to_string()
        } else {
            selected_runnable_followup_commands_text.clone()
        };
    let selected_manual_followup_commands_text = selected_manual_followup_command_lines.join("\r\n");
    let selected_manual_followup_commands_display =
        if selected_manual_followup_command_lines.is_empty() {
            "No baseline-required compare follow-up command for this selection.".to_string()
        } else {
            selected_manual_followup_commands_text.clone()
        };
    let selected_summary_overview = {
        let mut parts: Vec<String> = Vec::new();
        match selected_summary_path.as_deref() {
            Some(path) => parts.push(format!("Selected summary: {path}")),
            None => parts.push("Selected summary: <none>".to_string()),
        }
        parts.push(format!("Selected bundle dirs: {selected_bundle_count}"));
        if let Some(first) = selected_bundle_dirs.first() {
            parts.push(format!("First bundle dir: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected capability sources: {selected_capability_source_count}"
        ));
        if let Some(first) = selected_capability_sources.first() {
            parts.push(format!("First capability source: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected capability checks: {selected_capabilities_check_count}"
        ));
        if let Some(first) = selected_capabilities_checks.first() {
            parts.push(format!("First capability check: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected perf evidence lines: {selected_perf_evidence_count}"
        ));
        if let Some(first) = selected_perf_evidence.first() {
            parts.push(format!("First perf evidence: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected first-open evidence lines: {selected_first_open_evidence_count}"
        ));
        if let Some(first) = selected_first_open_evidence.first() {
            parts.push(format!("First first-open evidence: {}", first.as_ref()));
        }
        parts.push(format!(
            "Selected share artifacts: {selected_share_artifact_count}"
        ));
        if let Some(first) = selected_share_artifacts.first() {
            parts.push(format!("First share artifact: {}", first.as_ref()));
        }
        parts.push(format!(
            "Runnable follow-up commands: {selected_runnable_followup_count}"
        ));
        parts.push(format!(
            "Manual compare follow-up commands: {selected_manual_followup_count}"
        ));
        if let Some(err) = selected_error.as_deref() {
            parts.push(format!("Selected error: {err}"));
        }
        parts.join("\r\n")
    };
    let selected_detail_content = {
        let mut parts: Vec<String> = Vec::new();
        if let Some(path) = selected_summary_path.as_deref() {
            parts.push(format!("selected_summary_path: {path}"));
        }
        if !selected_bundle_dirs_text.trim().is_empty() {
            parts.push("bundle_dirs:".to_string());
            parts.push(selected_bundle_dirs_text.clone());
        }
        if !selected_capability_sources_text.trim().is_empty() {
            parts.push("capability_sources:".to_string());
            parts.push(selected_capability_sources_text.clone());
        }
        if !selected_capabilities_checks_text.trim().is_empty() {
            parts.push("capabilities_check_paths:".to_string());
            parts.push(selected_capabilities_checks_text.clone());
        }
        if !selected_perf_evidence_text.trim().is_empty() {
            parts.push("perf_evidence:".to_string());
            parts.push(selected_perf_evidence_text.clone());
        }
        if !selected_first_open_evidence_text.trim().is_empty() {
            parts.push("first_open_evidence:".to_string());
            parts.push(selected_first_open_evidence_text.clone());
        }
        if !selected_share_artifacts_text.trim().is_empty() {
            parts.push("share_artifacts:".to_string());
            parts.push(selected_share_artifacts_text.clone());
        }
        if let Some(err) = selected_error.as_deref() {
            parts.push(format!("error: {err}"));
        }
        if !selected_summary_json.trim().is_empty() {
            parts.push("selected regression.summary.json:".to_string());
            parts.push(selected_summary_json);
        }
        if parts.is_empty() {
            "<empty>".to_string()
        } else {
            parts.join(
                "

",
            )
        }
    };
    let selected_actions = ui::h_row(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if let Some(path) = selected_summary_path.as_ref().map(|v| v.to_string()) {
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: path.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy selected path")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_followup_commands_text.trim().is_empty() {
            let followup_commands = selected_followup_commands_text.clone();
            let on_copy: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: followup_commands.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy follow-up commands")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "stats")
        {
            out.push(
                shadcn::Button::new("Run stats")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_STATS)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "layout-perf-summary")
        {
            out.push(
                shadcn::Button::new("Run layout perf")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "memory-summary")
        {
            out.push(
                shadcn::Button::new("Run memory")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_MEMORY)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "triage")
        {
            out.push(
                shadcn::Button::new("Run triage")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "hotspots")
        {
            out.push(
                shadcn::Button::new("Run hotspots")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS)
                    .into_element(cx),
            );
        }
        if selected_followup_commands
            .iter()
            .any(|command| command.id == "trace")
        {
            out.push(
                shadcn::Button::new("Run trace")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(followup_in_flight)
                    .on_click(CMD_REGRESSION_RUN_FOLLOWUP_TRACE)
                    .into_element(cx),
            );
        }
        if selected_followup_result_path.is_some() {
            out.push(
                shadcn::Button::new("Copy selected follow-up result")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open selected follow-up JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_FOLLOWUP_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_followup_result_entry.is_some() {
            out.push(
                shadcn::Button::new("Copy selected follow-up command")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_COMMAND)
                    .into_element(cx),
            );
        }
        if !selected_followup_result_json.trim().is_empty() {
            out.push(
                shadcn::Button::new("Copy selected follow-up JSON")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_RESULT_JSON)
                    .into_element(cx),
            );
        }
        if selected_followup_trace_artifact_path.is_some() {
            out.push(
                shadcn::Button::new("Copy selected trace artifact")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_COPY_FOLLOWUP_TRACE_ARTIFACT_PATH)
                    .into_element(cx),
            );
            out.push(
                shadcn::Button::new("Open selected trace artifact")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_OPEN_FOLLOWUP_TRACE_ARTIFACT)
                    .into_element(cx),
            );
        }
        if let Some(first_bundle_dir) = selected_bundle_dirs.first().map(|v| v.to_string()) {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_bundle_dir.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first bundle dir")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        if let Some(first_capability_check) =
            selected_capabilities_checks.first().map(|v| v.to_string())
        {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_capability_check.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first capability check")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        if let Some(first_capability_source) =
            selected_capability_sources.first().map(|v| v.to_string())
        {
            let on_copy_first: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let token = host.next_clipboard_token();
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token,
                        text: first_capability_source.clone(),
                    });
                    host.request_redraw(action_cx.window);
                });
            out.push(
                shadcn::Button::new("Copy first capability source")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy_first)
                    .into_element(cx),
            );
        }
        out.push(
            shadcn::Button::new("Pack selected evidence")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_pack_selected_bundle || pack_in_flight)
                .on_click(CMD_REGRESSION_PACK_SELECTED_BUNDLE)
                .into_element(cx),
        );
        if !selected_bundle_dirs_text.trim().is_empty() {
            let bundle_dirs = selected_bundle_dirs_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: bundle_dirs.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy bundle dirs")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_capabilities_checks_text.trim().is_empty() {
            let capability_checks = selected_capabilities_checks_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: capability_checks.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy capability checks")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_capability_sources_text.trim().is_empty() {
            let capability_sources = selected_capability_sources_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: capability_sources.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy capability sources")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_perf_evidence_text.trim().is_empty() {
            let perf_evidence = selected_perf_evidence_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: perf_evidence.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy perf evidence")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                .into_element(cx),
            );
        }
        if !selected_first_open_evidence_text.trim().is_empty() {
            let first_open_evidence = selected_first_open_evidence_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: first_open_evidence.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy first-open evidence")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        if !selected_share_artifacts_text.trim().is_empty() {
            let share_artifacts = selected_share_artifacts_text.clone();
            let on_copy: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let token = host.next_clipboard_token();
                host.push_effect(Effect::ClipboardWriteText {
                    window: action_cx.window,
                    token,
                    text: share_artifacts.clone(),
                });
                host.request_redraw(action_cx.window);
            });
            out.push(
                shadcn::Button::new("Copy share artifacts")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_copy)
                    .into_element(cx),
            );
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let selected_summary_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if selected_summary_path.is_some() {
                "Summary selected"
            } else {
                "No selection"
            })
            .variant(if selected_summary_path.is_some() {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("bundle dirs {selected_bundle_count}"))
                .variant(if selected_bundle_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(format!(
                "capability sources {selected_capability_source_count}"
            ))
            .variant(if selected_capability_source_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!(
                "capability checks {selected_capabilities_check_count}"
            ))
            .variant(if selected_capabilities_check_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("perf evidence {selected_perf_evidence_count}"))
                .variant(if selected_perf_evidence_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(format!(
                "first-open evidence {selected_first_open_evidence_count}"
            ))
            .variant(if selected_first_open_evidence_count > 0 {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("share artifacts {selected_share_artifact_count}"))
                .variant(if selected_share_artifact_count > 0 {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Outline
                })
                .into_element(cx),
            shadcn::Badge::new(if selected_error.is_some() {
                "Selection error"
            } else {
                "Selection ok"
            })
            .variant(if selected_error.is_some() {
                shadcn::BadgeVariant::Destructive
            } else {
                shadcn::BadgeVariant::Secondary
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let status_row = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if can_refresh {
                "Artifacts root ready"
            } else {
                "No artifacts root"
            })
            .variant(if can_refresh {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("non-passing {failing_count}"))
                .variant(if failing_count > 0 {
                    shadcn::BadgeVariant::Destructive
                } else {
                    shadcn::BadgeVariant::Secondary
                })
                .into_element(cx),
            shadcn::Badge::new(format!("selected bundles {selected_bundle_count}"))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
            shadcn::Badge::new(format!(
                "selected capability sources {selected_capability_source_count}"
            ))
            .variant(shadcn::BadgeVariant::Outline)
            .into_element(cx),
            shadcn::Badge::new(format!(
                "selected capability checks {selected_capabilities_check_count}"
            ))
            .variant(shadcn::BadgeVariant::Outline)
            .into_element(cx),
            shadcn::Badge::new(if summarize_in_flight {
                "Summarizing"
            } else {
                "Summarize idle"
            })
            .variant(if summarize_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx);

    let top_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Refresh")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_refresh)
                .on_click(CMD_REGRESSION_REFRESH)
                .into_element(cx),
            shadcn::Button::new("Summarize")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_refresh || summarize_in_flight)
                .on_click(CMD_REGRESSION_SUMMARIZE)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let loaded_dir_text = cx.text(loaded_dir_line);
    let summarize_status_text = cx.text(summarize_status_line);
    let aggregate_preview_blob = text_blob_sized(cx, aggregate_preview.clone(), Px(96.0));
    let overview_status_section = diag_section(
        cx,
        "Aggregate Status",
        "Keep artifacts-root readiness and current failure counters visible at the top of the workspace.",
        vec![status_row, loaded_dir_text, summarize_status_text],
    );
    let overview_actions_section = diag_section(
        cx,
        "Primary Actions",
        "Refresh aggregate artifacts or run summarize without losing sight of the current counters.",
        vec![top_actions],
    );
    let overview_preview_section = diag_section(
        cx,
        "Dashboard Preview",
        "A compact aggregate preview stays available here, while full debug payloads live lower in the tab.",
        vec![aggregate_preview_blob],
    );

    let overview_card = diag_card(
        cx,
        "Regression Workspace",
        "Summary-first view over aggregate artifacts, non-passing summaries, and evidence actions.",
        vec![
            overview_status_section,
            overview_actions_section,
            overview_preview_section,
        ],
    );

    let left_card = diag_card(
        cx,
        "Non-passing Summaries",
        "Select one non-passing summary to open its evidence-focused drill-down.",
        vec![failing_list],
    );

    let selected_summary_overview_text = cx.text(selected_summary_overview);
    let selected_followup_status_text = cx.text(followup_status_line);
    let selected_followup_result_detail_blob = text_blob_sized(
        cx,
        followup::followup_result_history_entry_detail_lines(
            selected_followup_result_entry.as_ref(),
        )
        .join("\r\n"),
        Px(120.0),
    );
    let selected_followup_result_summary_blob = text_blob_sized(
        cx,
        followup::followup_result_summary_lines(&selected_followup_result_json).join("\r\n"),
        Px(96.0),
    );
    let selected_followup_result_history_blob = text_blob_sized(
        cx,
        followup::followup_result_history_summary_lines(
            &followup_result_history,
            selected_followup_history_filter_dirs
                .iter()
                .map(|value| value.as_str()),
        )
        .join("\r\n"),
        Px(120.0),
    );
    let selected_followup_result_history_list = followup_history_list(
        cx,
        &st.followup_selected_result_path,
        &selected_followup_history_entries,
        selected_followup_result_path.as_deref(),
    );
    let selected_runnable_followup_actions =
        runnable_followup_command_actions(
            cx,
            &st.followup_pending_command_id,
            &selected_followup_commands,
            followup_in_flight,
        );
    let baseline_bundle_input = shadcn::Input::new(st.followup_baseline_bundle_or_dir.clone())
        .a11y_label("Baseline bundle or directory")
        .placeholder("baseline bundle or dir")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);
    let baseline_session_input = shadcn::Input::new(st.followup_baseline_session.clone())
        .a11y_label("Baseline footprint session")
        .placeholder("baseline session")
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(260.0)))
        .into_element(cx);
    let has_visual_compare = selected_followup_commands
        .iter()
        .any(|command| command.id == "visual-compare");
    let has_footprint_compare = selected_followup_commands
        .iter()
        .any(|command| command.id == "footprint-compare");
    let visual_compare_ready =
        has_visual_compare && !followup_baseline_bundle_or_dir.trim().is_empty();
    let footprint_compare_ready =
        has_footprint_compare && !followup_baseline_session.trim().is_empty();
    let visual_compare_button = shadcn::Button::new("Run visual compare")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!visual_compare_ready || followup_in_flight)
        .on_click(CMD_REGRESSION_RUN_VISUAL_COMPARE)
        .into_element(cx);
    let footprint_compare_button = shadcn::Button::new("Run footprint compare")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .disabled(!footprint_compare_ready || followup_in_flight)
        .on_click(CMD_REGRESSION_RUN_FOOTPRINT_COMPARE)
        .into_element(cx);
    let visual_compare_row = ui::h_row(|_cx| [baseline_bundle_input, visual_compare_button])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .into_element(cx);
    let footprint_compare_row =
        ui::h_row(|_cx| [baseline_session_input, footprint_compare_button])
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx);
    let baseline_compare_controls = ui::v_stack(|_cx| [visual_compare_row, footprint_compare_row])
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);
    let selected_followup_result_json_blob = text_blob_sized(
        cx,
        if selected_followup_result_json.trim().is_empty() {
            "<no selected-bundle follow-up result yet>".to_string()
        } else {
            selected_followup_result_json
        },
        Px(140.0),
    );
    let selected_followup_readiness_blob = text_blob_sized(
        cx,
        selected_followup_readiness_lines.join("\r\n"),
        Px(84.0),
    );
    let selected_bundle_dirs_blob =
        text_blob_sized(cx, selected_bundle_dirs_text.clone(), Px(96.0));
    let selected_capability_sources_blob =
        text_blob_sized(cx, selected_capability_sources_text.clone(), Px(96.0));
    let selected_capabilities_blob =
        text_blob_sized(cx, selected_capabilities_checks_text.clone(), Px(96.0));
    let selected_perf_evidence_blob =
        text_blob_sized(cx, selected_perf_evidence_text.clone(), Px(120.0));
    let selected_first_open_evidence_blob =
        text_blob_sized(cx, selected_first_open_evidence_text.clone(), Px(120.0));
    let selected_share_artifacts_blob =
        text_blob_sized(cx, selected_share_artifacts_text.clone(), Px(96.0));
    let selected_followup_commands_blob =
        text_blob_sized(cx, selected_followup_commands_display, Px(116.0));
    let selected_runnable_followup_commands_blob =
        text_blob_sized(cx, selected_runnable_followup_commands_display, Px(96.0));
    let selected_manual_followup_commands_blob =
        text_blob_sized(cx, selected_manual_followup_commands_display, Px(96.0));
    let selected_raw_summary_blob = text_blob_sized(cx, selected_detail_content, Px(220.0));
    let selected_overview_section = diag_section(
        cx,
        "Selection Overview",
        "Keep the current non-passing state visible before diving into raw JSON.",
        vec![selected_summary_badges, selected_summary_overview_text],
    );
    let selected_actions_section = diag_section(
        cx,
        "Evidence Actions",
        "Copy paths or pack the currently selected evidence without leaving this inspector.",
        vec![selected_actions],
    );
    let selected_followup_run_status_section = diag_section(
        cx,
        "Follow-up Run Status",
        "Runnable follow-up commands execute through the shared diagnostics engine and report status here.",
        vec![selected_followup_status_text],
    );
    let selected_followup_readiness_section = diag_section(
        cx,
        "Follow-up Readiness",
        "A compact readiness summary links selected summary evidence to the next runnable command.",
        vec![selected_followup_readiness_blob],
    );
    let selected_followup_result_detail_section = diag_section(
        cx,
        "Follow-up Result Details",
        "Selected result status, path, command, bundle, and error preview for reproduction.",
        vec![selected_followup_result_detail_blob],
    );
    let selected_followup_result_summary_section = diag_section(
        cx,
        "Follow-up Result Summary",
        "Status, command, duration, and error preview from the latest selected-bundle follow-up result.",
        vec![selected_followup_result_summary_blob],
    );
    let selected_followup_result_history_section = diag_section(
        cx,
        "Follow-up Result History",
        "Select a GUI-launched follow-up result for the selected bundle, newest first.",
        vec![
            selected_followup_result_history_blob,
            selected_followup_result_history_list,
        ],
    );
    let selected_runnable_followup_actions_section = diag_section(
        cx,
        "Runnable Follow-up Actions",
        "Run any bundle-local follow-up command generated for the selected summary.",
        vec![selected_runnable_followup_actions],
    );
    let selected_baseline_compare_actions_section = diag_section(
        cx,
        "Baseline Compare Actions",
        "Provide a baseline to turn manual compare templates into runnable diagnostics follow-ups.",
        vec![baseline_compare_controls],
    );
    let selected_followup_result_json_section = diag_section(
        cx,
        "Follow-up Result JSON",
        "The latest selected-bundle follow-up result artifact is mirrored here for quick triage.",
        vec![selected_followup_result_json_blob],
    );
    let selected_followup_commands_section = diag_section(
        cx,
        "Follow-up Commands",
        "Concrete stats, triage, hotspot, trace, visual-compare, and footprint commands are generated from the selected bundle directory.",
        vec![selected_followup_commands_blob],
    );
    let selected_runnable_followup_commands_section = diag_section(
        cx,
        "Runnable Follow-ups",
        "Bundle-local commands have concrete diag args and do not require a baseline selection.",
        vec![selected_runnable_followup_commands_blob],
    );
    let selected_manual_followup_commands_section = diag_section(
        cx,
        "Manual Compare Follow-ups",
        "Compare commands stay visible but are separated because they still require a baseline input.",
        vec![selected_manual_followup_commands_blob],
    );
    let selected_bundle_dirs_section = diag_section(
        cx,
        "Bundle Directories",
        "These are the concrete artifact roots attached to the selected non-passing summary.",
        vec![selected_bundle_dirs_blob],
    );
    let selected_capability_sources_section = diag_section(
        cx,
        "Capability Sources",
        "Capability provenance is shown separately from campaign-local check artifacts and prefers the additive source object when present.",
        vec![selected_capability_sources_blob],
    );
    let selected_capabilities_section = diag_section(
        cx,
        "Capability Checks",
        "Policy-skipped summaries can point at campaign capability check artifacts even when no bundle dir exists.",
        vec![selected_capabilities_blob],
    );
    let selected_perf_evidence_section = diag_section(
        cx,
        "Perf Evidence",
        "Perf summary paths, threshold artifacts, curated metrics, and threshold failures stay above the raw JSON.",
        vec![selected_perf_evidence_blob],
    );
    let selected_first_open_evidence_section = diag_section(
        cx,
        "First-open Evidence",
        "Canonical summary-level paths for triage, script results, screenshots, and share packs use the shared diagnostics drill-down projection.",
        vec![selected_first_open_evidence_blob],
    );
    let selected_share_artifacts_section = diag_section(
        cx,
        "Share Artifacts",
        "Compact handoff packages stay optional, but visible beside canonical evidence when the selected summary exposes them.",
        vec![selected_share_artifacts_blob],
    );
    let selected_raw_summary_section = diag_section(
        cx,
        "Raw Selected Summary",
        "Raw summary payload remains available for debugging, but below overview and actions.",
        vec![selected_raw_summary_blob],
    );

    let right_card = diag_card(
        cx,
        "Selected Summary",
        "Evidence actions and raw summary payload stay close to the current non-passing selection.",
        vec![
            selected_overview_section,
            selected_actions_section,
            selected_followup_readiness_section,
            selected_followup_run_status_section,
            selected_followup_result_detail_section,
            selected_followup_result_summary_section,
            selected_followup_result_history_section,
            selected_runnable_followup_actions_section,
            selected_baseline_compare_actions_section,
            selected_followup_result_json_section,
            selected_followup_commands_section,
            selected_runnable_followup_commands_section,
            selected_manual_followup_commands_section,
            selected_bundle_dirs_section,
            selected_capability_sources_section,
            selected_capabilities_section,
            selected_perf_evidence_section,
            selected_first_open_evidence_section,
            selected_share_artifacts_section,
            selected_raw_summary_section,
        ],
    );

    let split = ui::h_row(|cx| {
        [
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(372.0))
                        .h_full(),
                ),
                |_cx| [left_card],
            ),
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .flex_1()
                        .min_w_0()
                        .h_full(),
                ),
                |_cx| [right_card],
            ),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .items_start()
    .into_element(cx);

    let dashboard_debug_blob = text_blob_sized(cx, aggregate_preview.clone(), Px(96.0));
    let index_debug_blob = text_blob_sized(cx, index_json.clone(), Px(140.0));
    let summary_debug_blob = text_blob_sized(cx, summary_json.clone(), Px(140.0));
    let dashboard_debug_section = diag_section(
        cx,
        "Dashboard Preview",
        "Human-readable aggregate output for quick debugging and copy/paste.",
        vec![dashboard_debug_blob],
    );
    let index_debug_section = diag_section(
        cx,
        "regression.index.json",
        "Campaign index payload backing the aggregate workspace.",
        vec![index_debug_blob],
    );
    let summary_debug_section = diag_section(
        cx,
        "regression.summary.json",
        "Latest aggregate summary payload emitted by summarize/dashboard flows.",
        vec![summary_debug_blob],
    );
    let raw_payloads = diag_card(
        cx,
        "Aggregate Debug Payloads",
        "Keep dashboard and raw aggregate payloads available for debugging, but clearly below the main regression workflow.",
        vec![
            dashboard_debug_section,
            index_debug_section,
            summary_debug_section,
        ],
    );

    ui::v_stack(|_cx| [overview_card, split, raw_payloads])
        .gap(fret_ui_kit::Space::N2)
        .into_element(cx)
}
