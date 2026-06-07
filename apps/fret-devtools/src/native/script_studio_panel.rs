use std::path::PathBuf;
use std::sync::Arc;

use fret_app::{App, Effect};
use fret_core::Px;
use fret_diag_protocol::{UiActionScriptV1, UiActionScriptV2};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::State;
use super::command_catalog::{
    CMD_COPY_PACK_PATH, CMD_OPEN_VIEWER_URL, CMD_PACK_LAST_BUNDLE, CMD_SCRIPT_APPLY_PICK,
    CMD_SCRIPT_FORK, CMD_SCRIPT_PUSH, CMD_SCRIPT_RUN, CMD_SCRIPT_RUN_AND_PACK, CMD_SCRIPT_SAVE,
    CMD_SCRIPTS_REFRESH,
};
use super::script_studio;
use super::ui_primitives::{diag_card, text_blob};

pub(super) fn center_panel(
    cx: &mut ElementContext<'_, App>,
    theme: fret_ui::ThemeSnapshot,
    st: &State,
) -> AnyElement {
    let script_text = cx
        .app
        .models()
        .read(&st.script_text, |v| v.clone())
        .unwrap_or_default();
    let pick_text = cx
        .app
        .models()
        .read(&st.last_pick_json, |v| v.clone())
        .unwrap_or_default();
    let apply_pointer = cx
        .app
        .models()
        .read(&st.script_apply_pointer, |v| v.clone())
        .unwrap_or_default();
    let scripts = cx
        .app
        .models()
        .read(&st.script_library, |v| v.clone())
        .unwrap_or_default();
    let loaded_origin = cx
        .app
        .models()
        .read(&st.loaded_script_origin, |v| *v)
        .ok()
        .flatten();
    let loaded_path = cx
        .app
        .models()
        .read(&st.loaded_script_path, |v| v.clone())
        .ok()
        .flatten();
    let script_last_stage = cx
        .app
        .models()
        .read(&st.script_last_stage, |v| v.clone())
        .ok()
        .flatten();
    let script_last_step_index = cx
        .app
        .models()
        .read(&st.script_last_step_index, |v| *v)
        .ok()
        .flatten();
    let script_last_reason = cx
        .app
        .models()
        .read(&st.script_last_reason, |v| v.clone())
        .ok()
        .flatten();
    let pack_after_run = cx
        .app
        .models()
        .read(&st.script_pack_after_run, |v| *v)
        .unwrap_or(false);

    let target_out_dir = cx
        .app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dir_abs = cx
        .app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.clone())
        .ok()
        .flatten();
    let last_bundle_dump_bundle_json = cx
        .app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.clone())
        .ok()
        .flatten();
    let last_pack_path = cx
        .app
        .models()
        .read(&st.last_pack_path, |v| v.clone())
        .ok()
        .flatten();
    let pack_in_flight = cx
        .app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    let pack_last_error = cx
        .app
        .models()
        .read(&st.pack_last_error, |v| v.clone())
        .ok()
        .flatten();
    let viewer_url = cx
        .app
        .models()
        .read(&st.viewer_url, |v| v.clone())
        .unwrap_or_default();

    let consume_clicks = cx
        .app
        .models()
        .read(&st.inspect_consume_clicks, |v| *v)
        .unwrap_or(false);

    let consume_toggle = shadcn::Checkbox::new(st.inspect_consume_clicks.clone())
        .a11y_label("Consume clicks while inspecting")
        .into_element(cx);

    let has_session = cx
        .app
        .models()
        .read(&st.selected_session_id, |v| v.is_some())
        .unwrap_or(false);

    let can_fork = loaded_origin == Some(script_studio::ScriptOrigin::WorkspaceTools);
    let can_save = loaded_origin == Some(script_studio::ScriptOrigin::UserLocal);
    let can_apply_pick = !pick_text.trim().is_empty() && !apply_pointer.trim().is_empty();
    let can_pack = last_bundle_dir_abs.is_some() || last_bundle_dump_bundle_json.is_some();

    let pointer_input = shadcn::Input::new(st.script_apply_pointer.clone())
        .a11y_label("JSON pointer")
        .placeholder("/steps/0/target")
        .into_element(cx);

    let viewer_url_input = shadcn::Input::new(st.viewer_url.clone())
        .a11y_label("Bundle viewer URL")
        .placeholder("http://localhost:5173")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(st.script_text.clone())
        .a11y_label("Script JSON")
        .min_height(Px(360.0))
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx);

    let (script_summary, script_is_valid) = script_summary_line(&script_text);
    let script_steps = script_steps_len(&script_text).unwrap_or(0);
    let script_schema_version = infer_script_schema_version(&script_text).unwrap_or(1);
    let pack_status_line = {
        let err = pack_last_error
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!(
            "pack_in_flight={} pack_last_error={err}",
            if pack_in_flight { "true" } else { "false" }
        )
    };

    let primary_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Push Script")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_PUSH)
                .into_element(cx),
            shadcn::Button::new("Run Script")
                .variant(shadcn::ButtonVariant::Default)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_RUN)
                .into_element(cx),
            shadcn::Button::new("Run & Pack")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!has_session || !script_is_valid)
                .on_click(CMD_SCRIPT_RUN_AND_PACK)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let library_actions = ui::h_row(|cx| {
        [
            shadcn::Button::new("Refresh Scripts")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_SCRIPTS_REFRESH)
                .into_element(cx),
            shadcn::Button::new("Fork")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_fork)
                .on_click(CMD_SCRIPT_FORK)
                .into_element(cx),
            shadcn::Button::new("Save")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_save)
                .on_click(CMD_SCRIPT_SAVE)
                .into_element(cx),
            consume_toggle,
            shadcn::Badge::new(if consume_clicks {
                "Consume clicks on"
            } else {
                "Consume clicks off"
            })
            .variant(if consume_clicks {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let pack_row = ui::h_row(|cx| {
        let copy_enabled = last_pack_path.is_some();
        [
            cx.text("Artifacts:"),
            shadcn::Button::new("Pack last bundle")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_pack || pack_in_flight)
                .on_click(CMD_PACK_LAST_BUNDLE)
                .into_element(cx),
            shadcn::Button::new("Copy pack path")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!copy_enabled)
                .on_click(CMD_COPY_PACK_PATH)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let viewer_row = ui::h_row(|cx| {
        [
            cx.text("Viewer:"),
            viewer_url_input,
            shadcn::Button::new("Open viewer")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(viewer_url.trim().is_empty())
                .on_click(CMD_OPEN_VIEWER_URL)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let apply_row = ui::h_row(|cx| {
        [
            cx.text("Pick-to-fill:"),
            pointer_input,
            shadcn::Button::new("Apply Pick")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!can_apply_pick)
                .on_click(CMD_SCRIPT_APPLY_PICK)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let out_dir_line = match target_out_dir.as_deref() {
        Some(dir) => format!("Target diag out_dir: {dir}"),
        None => "Target diag out_dir: <unknown>".to_string(),
    };
    let loaded_summary_line = match (loaded_origin, loaded_path.as_deref()) {
        (Some(origin), Some(path)) => format!("Loaded [{}] {}", origin.label(), path),
        _ => "Loaded <none>".to_string(),
    };
    let run_summary_line = {
        let stage = script_last_stage
            .as_ref()
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|| "None".to_string());
        let step = script_last_step_index
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!("Run status: {stage} | step {step}/{script_steps}")
    };
    let reason_summary_line = script_last_reason
        .as_deref()
        .map(|s| format!("Reason: {s}"))
        .unwrap_or_else(|| "Reason: -".to_string());
    let pack_summary_line = match last_pack_path.as_deref() {
        Some(path) => format!("Pack output: {path}"),
        None => format!("Pack output: <none> | {pack_status_line}"),
    };
    let script_status_badges = ui::h_row(|cx| {
        [
            shadcn::Badge::new(if has_session {
                "Session connected"
            } else {
                "No session"
            })
            .variant(if has_session {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if script_is_valid {
                format!("Schema v{script_schema_version} valid")
            } else {
                format!("Schema v{script_schema_version} invalid")
            })
            .variant(if script_is_valid {
                shadcn::BadgeVariant::Secondary
            } else {
                shadcn::BadgeVariant::Destructive
            })
            .into_element(cx),
            shadcn::Badge::new(if pack_in_flight {
                "Pack busy"
            } else {
                "Pack idle"
            })
            .variant(if pack_in_flight {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(if pack_after_run {
                "Run&Pack enabled"
            } else {
                "Run-only mode"
            })
            .variant(if pack_after_run {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Outline
            })
            .into_element(cx),
            shadcn::Badge::new(format!("Library {}", scripts.len()))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .items_center()
    .into_element(cx);

    let mut script_rows: Vec<AnyElement> = Vec::new();
    for item in scripts.iter() {
        let label = format!("[{}] {}", item.origin.label(), item.file_name);
        let is_loaded = loaded_path
            .as_deref()
            .is_some_and(|p| PathBuf::from(p) == item.path);

        let variant = if is_loaded {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        };

        let origin_for_activate = item.origin;
        let path_for_activate = item.path.clone();
        let script_text_for_activate = st.script_text.clone();
        let loaded_origin_for_activate = st.loaded_script_origin.clone();
        let loaded_path_for_activate = st.loaded_script_path.clone();
        let log_lines_for_activate = st.log_lines.clone();

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let text = match std::fs::read_to_string(&path_for_activate) {
                Ok(text) => text,
                Err(err) => {
                    let line = format!("script load failed: {err}");
                    let _ = host.models_mut().update(&log_lines_for_activate, |v| {
                        v.push(Arc::<str>::from(line));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                    host.request_redraw(action_cx.window);
                    return;
                }
            };

            let _ = host.models_mut().update(&script_text_for_activate, |v| {
                *v = text;
            });
            let _ = host.models_mut().update(&loaded_origin_for_activate, |v| {
                *v = Some(origin_for_activate)
            });
            let _ = host.models_mut().update(&loaded_path_for_activate, |v| {
                *v = Some(Arc::<str>::from(
                    path_for_activate.to_string_lossy().to_string(),
                ))
            });

            host.request_redraw(action_cx.window);
            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                action_cx.window,
            ));
        });

        script_rows.push(
            shadcn::Button::new(label)
                .variant(variant)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let scripts_list = shadcn::ScrollArea::new([ui::v_stack(|_cx| script_rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .into_element(cx);

    let pointer_candidates = script_studio::collect_common_json_pointers(&script_text);

    let step_index_input = shadcn::Input::new(st.script_step_insert_index.clone())
        .a11y_label("Step insert index")
        .placeholder("(append)")
        .into_element(cx);

    let mut step_buttons: Vec<AnyElement> = Vec::new();
    for t in step_templates_for_schema(script_schema_version) {
        let script_text_model = st.script_text.clone();
        let insert_index_model = st.script_step_insert_index.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let step_value = t.step.clone();
        let label = t.label;

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let index_text = host
                .models_mut()
                .read(&insert_index_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            let index = index_text.trim().parse::<usize>().ok();

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();

            let len_before = script_steps_len(&current).unwrap_or(0);
            let insert_at = index.unwrap_or(len_before);
            let inserted_index = insert_at.min(len_before);

            let updated = match index {
                Some(i) => script_studio::insert_step_json(&current, i, step_value.clone()),
                None => script_studio::append_step_json(&current, step_value.clone()),
            };

            match updated {
                Ok(text) => {
                    let _ = host.models_mut().update(&script_text_model, |v| *v = text);
                    if let Some(suffix) = primary_pointer_suffix_for_step_json(&step_value) {
                        let ptr = format!("/steps/{inserted_index}{suffix}");
                        let _ = host.models_mut().update(&pointer_model, |v| *v = ptr);
                    }
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!(
                            "insert step failed ({label}): {err}"
                        )));
                        if v.len() > 2000 {
                            let drain = v.len().saturating_sub(2000);
                            v.drain(0..drain);
                        }
                    });
                }
            }

            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });

        step_buttons.push(
            shadcn::Button::new(t.label)
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx),
        );
    }

    let steps_tab = shadcn::ScrollArea::new([ui::v_stack(|cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        out.push(cx.text(format!("Schema v{script_schema_version} step palette")));
        out.push(step_index_input);
        out.extend(step_buttons);
        if !pointer_candidates.is_empty() {
            out.push(cx.text("Pointer candidates:"));
            for p in pointer_candidates.iter().take(64) {
                let pointer_model = st.script_apply_pointer.clone();
                let p_value = p.clone();
                let p_label = p.clone();
                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&pointer_model, |v| *v = p_value.clone());
                        host.request_redraw(action_cx.window);
                        host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
                    });
                out.push(
                    shadcn::Button::new(p_label)
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_activate)
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
                        .into_element(cx),
                );
            }
        }
        out
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx)])
    .into_element(cx);

    let selector_kind_items = [
        shadcn::SelectItem::new("test_id", "test_id"),
        shadcn::SelectItem::new("role_and_name", "role_and_name"),
        shadcn::SelectItem::new("role_and_path", "role_and_path"),
        shadcn::SelectItem::new("node_id", "node_id"),
        shadcn::SelectItem::new("global_element_id", "global_element_id"),
    ];
    let selector_kind_select = shadcn::Select::new(
        st.script_selector_kind.clone(),
        st.script_selector_kind_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("selector kind"))
    .items(selector_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let selector_kind = cx
        .app
        .models()
        .read(&st.script_selector_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("test_id"));
    let selector_value = selector_value_from_models(cx, st, selector_kind.as_ref());
    let selector_json =
        serde_json::to_string_pretty(&selector_value).unwrap_or_else(|_| "{}".to_string());

    let selector_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let selector_value = selector_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply selector refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                selector_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply selector failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let selector_copy = {
        let selector_json = selector_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: selector_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let selector_tab = ui::v_stack(|cx| {
        let fields = selector_fields(cx, st, selector_kind.as_ref());
        let preview = text_blob(cx, selector_json.clone());
        [
            selector_kind_select,
            fields,
            ui::h_row(|cx| {
                [
                    shadcn::Button::new("Apply to pointer")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(selector_apply)
                        .into_element(cx),
                    shadcn::Button::new("Copy JSON")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(selector_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx),
            preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let predicate_kind_items = [
        shadcn::SelectItem::new("exists", "exists"),
        shadcn::SelectItem::new("not_exists", "not_exists"),
        shadcn::SelectItem::new("focus_is", "focus_is"),
        shadcn::SelectItem::new("role_is", "role_is"),
        shadcn::SelectItem::new("checked_is", "checked_is"),
        shadcn::SelectItem::new("checked_is_none", "checked_is_none"),
        shadcn::SelectItem::new("label_len_is", "label_len_is"),
        shadcn::SelectItem::new("label_len_ge", "label_len_ge"),
        shadcn::SelectItem::new("value_len_is", "value_len_is"),
        shadcn::SelectItem::new("value_len_ge", "value_len_ge"),
        shadcn::SelectItem::new("barrier_roots", "barrier_roots"),
        shadcn::SelectItem::new("visible_in_window", "visible_in_window"),
        shadcn::SelectItem::new("bounds_within_window", "bounds_within_window"),
        shadcn::SelectItem::new("bounds_min_size", "bounds_min_size"),
        shadcn::SelectItem::new("bounds_non_overlapping", "bounds_non_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping", "bounds_overlapping"),
        shadcn::SelectItem::new("bounds_overlapping_x", "bounds_overlapping_x"),
        shadcn::SelectItem::new("bounds_overlapping_y", "bounds_overlapping_y"),
    ];
    let predicate_kind_select = shadcn::Select::new(
        st.script_predicate_kind.clone(),
        st.script_predicate_kind_open.clone(),
    )
    .value(shadcn::SelectValue::new().placeholder("predicate kind"))
    .items(predicate_kind_items)
    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let predicate_kind = cx
        .app
        .models()
        .read(&st.script_predicate_kind, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("exists"));
    let predicate_value =
        predicate_value_from_models(cx, st, predicate_kind.as_ref(), selector_value.clone());
    let predicate_json =
        serde_json::to_string_pretty(&predicate_value).unwrap_or_else(|_| "{}".to_string());

    let predicate_apply = {
        let script_text_model = st.script_text.clone();
        let pointer_model = st.script_apply_pointer.clone();
        let log_lines = st.log_lines.clone();
        let predicate_value = predicate_value.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pointer = host
                .models_mut()
                .read(&pointer_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            if pointer.trim().is_empty() {
                let _ = host.models_mut().update(&log_lines, |v| {
                    v.push(Arc::<str>::from(
                        "apply predicate refused (empty json pointer)",
                    ));
                });
                host.request_redraw(action_cx.window);
                return;
            }

            let current = host
                .models_mut()
                .read(&script_text_model, |v: &String| v.clone())
                .ok()
                .unwrap_or_default();
            match script_studio::apply_json_value_to_json_pointer(
                &current,
                &pointer,
                predicate_value.clone(),
            ) {
                Ok(updated) => {
                    let _ = host
                        .models_mut()
                        .update(&script_text_model, |v| *v = updated);
                }
                Err(err) => {
                    let _ = host.models_mut().update(&log_lines, |v| {
                        v.push(Arc::<str>::from(format!("apply predicate failed: {err}")));
                    });
                }
            }
            host.request_redraw(action_cx.window);
            host.push_effect(Effect::RequestAnimationFrame(action_cx.window));
        });
        on_activate
    };

    let predicate_copy = {
        let predicate_json = predicate_json.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: predicate_json.clone(),
            });
            host.request_redraw(action_cx.window);
        });
        on_activate
    };

    let predicate_tab = ui::v_stack(|cx| {
        let fields = predicate_fields(cx, st, predicate_kind.as_ref());
        let preview = text_blob(cx, predicate_json.clone());
        [
            predicate_kind_select,
            fields,
            ui::h_row(|cx| {
                [
                    shadcn::Button::new("Apply to pointer")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(predicate_apply)
                        .into_element(cx),
                    shadcn::Button::new("Copy JSON")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(predicate_copy)
                        .into_element(cx),
                ]
            })
            .gap(fret_ui_kit::Space::N2)
            .items_center()
            .into_element(cx),
            preview,
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    let helpers_tabs = shadcn::Tabs::new(st.script_studio_helper_tab.clone())
        .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("steps", "Steps", [steps_tab]),
            shadcn::TabsItem::new("selector", "Selector", [selector_tab]),
            shadcn::TabsItem::new("predicate", "Predicate", [predicate_tab]),
        ])
        .into_element(cx);

    let validate_summary = cx.text(format!("Validate: {script_summary}"));
    let run_summary = cx.text(run_summary_line);
    let reason_summary = cx.text(reason_summary_line);
    let loaded_summary = cx.text(loaded_summary_line);
    let out_dir_summary = cx.text(out_dir_line);
    let pack_summary = cx.text(pack_summary_line);

    let workflow_controls = diag_card(
        cx,
        "Workflow Controls",
        "Select a script, validate it, and decide whether the next run also produces evidence.",
        vec![
            primary_actions,
            library_actions,
            script_status_badges,
            validate_summary,
            run_summary,
            reason_summary,
        ],
    );

    let workflow_outputs = diag_card(
        cx,
        "Outputs & Bundles",
        "Apply captured picks, package the latest bundle, and hand off to the offline viewer.",
        vec![
            apply_row,
            pack_row,
            viewer_row,
            loaded_summary,
            out_dir_summary,
            pack_summary,
        ],
    );

    let workflow_summary = ui::h_row(|_cx| [workflow_controls, workflow_outputs])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .items_start()
        .into_element(cx);

    let scripts_sidebar = diag_card(
        cx,
        "Script Source",
        format!(
            "Workspace tools and local scripts available: {}",
            scripts.len()
        ),
        vec![scripts_list],
    );

    let editor_workspace = diag_card(
        cx,
        "Editor",
        format!("Current script payload size: {} bytes", script_text.len()),
        vec![textarea],
    );

    let helper_workspace = diag_card(
        cx,
        "Helpers",
        "Build reusable steps, selectors, and predicates without leaving the editor flow.",
        vec![helpers_tabs],
    );

    let split = ui::h_row(|cx| {
        [
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(224.0))
                        .h_full(),
                ),
                |_cx| [scripts_sidebar],
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
                |_cx| [editor_workspace],
            ),
            cx.container(
                fret_ui_kit::declarative::style::container_props(
                    &theme,
                    fret_ui_kit::ChromeRefinement::default(),
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(304.0))
                        .h_full(),
                ),
                |_cx| [helper_workspace],
            ),
        ]
    })
    .gap(fret_ui_kit::Space::N2)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
    .items_start()
    .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Script Studio").into_element(cx),
            shadcn::CardDescription::new(
                "A compact workflow for selecting scripts, editing payloads, and packaging evidence.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([workflow_summary, split]).into_element(cx),
    ])
    .into_element(cx)
}

fn script_steps_len(script_text: &str) -> Option<usize> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    v.get("steps").and_then(|v| v.as_array()).map(|a| a.len())
}

fn script_summary_line(script_text: &str) -> (String, bool) {
    let v: serde_json::Value = match serde_json::from_str(script_text) {
        Ok(v) => v,
        Err(err) => return (format!("parse_error: {err}"), false),
    };

    let schema = match validate_script_json_value(&v) {
        Ok(schema) => schema,
        Err(err) => return (format!("invalid: {err}"), false),
    };

    let steps = v.get("steps").and_then(|v| v.as_array()).map(|a| a.len());
    let steps = steps
        .map(|n| n.to_string())
        .unwrap_or_else(|| "<missing>".to_string());
    (format!("ok schema_version={schema} steps={steps}"), true)
}

pub(super) fn validate_script_json_value(script: &serde_json::Value) -> Result<u32, String> {
    let schema_version = script
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing schema_version".to_string())?;
    let schema_version = schema_version.min(u32::MAX as u64) as u32;

    match schema_version {
        1 => {
            let parsed: UiActionScriptV1 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 1 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(1)
        }
        2 => {
            let parsed: UiActionScriptV2 =
                serde_json::from_value(script.clone()).map_err(|e| e.to_string())?;
            if parsed.schema_version != 2 {
                return Err("schema_version mismatch".to_string());
            }
            Ok(2)
        }
        other => Err(format!("unsupported schema_version: {other}")),
    }
}

#[derive(Clone)]
struct StepTemplate {
    label: &'static str,
    step: serde_json::Value,
}

fn infer_script_schema_version(script_text: &str) -> Option<u32> {
    let v: serde_json::Value = serde_json::from_str(script_text).ok()?;
    let schema = v.get("schema_version").and_then(|v| v.as_u64())?;
    Some(schema.min(u32::MAX as u64) as u32)
}

fn placeholder_selector_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "test_id",
        "id": "TODO",
    })
}

fn placeholder_predicate_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "exists",
        "target": placeholder_selector_value(),
    })
}

fn primary_pointer_suffix_for_step_json(step: &serde_json::Value) -> Option<&'static str> {
    let obj = step.as_object()?;
    let has = |k: &str| obj.contains_key(k);
    if has("target") {
        return Some("/target");
    }
    if has("predicate") {
        return Some("/predicate");
    }
    if has("container") {
        return Some("/container");
    }
    if has("from") {
        return Some("/from");
    }
    if has("to") {
        return Some("/to");
    }
    if has("menu") {
        return Some("/menu");
    }
    if has("item") {
        return Some("/item");
    }
    if has("path") {
        return Some("/path/0");
    }
    None
}

fn step_templates_for_schema(schema_version: u32) -> Vec<StepTemplate> {
    let target = placeholder_selector_value();
    let predicate = placeholder_predicate_value();

    let v1 = vec![
        StepTemplate {
            label: "click",
            step: serde_json::json!({
                "type": "click",
                "target": target.clone(),
                "button": "left",
            }),
        },
        StepTemplate {
            label: "drag_pointer",
            step: serde_json::json!({
                "type": "drag_pointer",
                "target": placeholder_selector_value(),
                "button": "left",
                "delta_x": 120.0,
                "delta_y": 0.0,
                "steps": 8,
            }),
        },
        StepTemplate {
            label: "move_pointer",
            step: serde_json::json!({
                "type": "move_pointer",
                "target": placeholder_selector_value(),
            }),
        },
        StepTemplate {
            label: "wheel",
            step: serde_json::json!({
                "type": "wheel",
                "target": placeholder_selector_value(),
                "delta_x": 0.0,
                "delta_y": -120.0,
            }),
        },
        StepTemplate {
            label: "press_key",
            step: serde_json::json!({
                "type": "press_key",
                "key": "Enter",
                "modifiers": { "shift": false, "ctrl": false, "alt": false, "meta": false },
                "repeat": false,
            }),
        },
        StepTemplate {
            label: "type_text",
            step: serde_json::json!({
                "type": "type_text",
                "text": "TODO",
            }),
        },
        StepTemplate {
            label: "wait_frames",
            step: serde_json::json!({
                "type": "wait_frames",
                "n": 30,
            }),
        },
        StepTemplate {
            label: "wait_until",
            step: serde_json::json!({
                "type": "wait_until",
                "predicate": predicate.clone(),
                "timeout_frames": 180,
            }),
        },
        StepTemplate {
            label: "assert",
            step: serde_json::json!({
                "type": "assert",
                "predicate": placeholder_predicate_value(),
            }),
        },
        StepTemplate {
            label: "capture_bundle",
            step: serde_json::json!({
                "type": "capture_bundle",
                "label": "devtools",
            }),
        },
        StepTemplate {
            label: "capture_screenshot",
            step: serde_json::json!({
                "type": "capture_screenshot",
                "label": "devtools",
                "timeout_frames": 300,
            }),
        },
        StepTemplate {
            label: "reset_diagnostics",
            step: serde_json::json!({
                "type": "reset_diagnostics",
            }),
        },
    ];

    if schema_version <= 1 {
        return v1;
    }

    let window = serde_json::json!({ "kind": "current" });

    let mut v2 = Vec::new();
    v2.push(StepTemplate {
        label: "click",
        step: serde_json::json!({
            "type": "click",
            "window": window.clone(),
            "target": target.clone(),
            "button": "left",
        }),
    });
    v2.push(StepTemplate {
        label: "drag_pointer",
        step: serde_json::json!({
            "type": "drag_pointer",
            "window": window.clone(),
            "target": placeholder_selector_value(),
            "button": "left",
            "delta_x": 120.0,
            "delta_y": 0.0,
            "steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_down",
        step: serde_json::json!({
            "type": "pointer_down",
            "window": window.clone(),
            "target": placeholder_selector_value(),
            "button": "left",
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_move",
        step: serde_json::json!({
            "type": "pointer_move",
            "window": window.clone(),
            "delta_x": 120.0,
            "delta_y": 0.0,
            "steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "pointer_up",
        step: serde_json::json!({
            "type": "pointer_up",
            "window": window.clone(),
        }),
    });
    v2.push(StepTemplate {
        label: "move_pointer",
        step: serde_json::json!({
            "type": "move_pointer",
            "target": placeholder_selector_value(),
        }),
    });
    v2.push(StepTemplate {
        label: "wheel",
        step: serde_json::json!({
            "type": "wheel",
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": -120.0,
        }),
    });
    v2.push(StepTemplate {
        label: "press_key",
        step: serde_json::json!({
            "type": "press_key",
            "key": "Enter",
            "modifiers": { "shift": false, "ctrl": false, "alt": false, "meta": false },
            "repeat": false,
        }),
    });
    v2.push(StepTemplate {
        label: "type_text",
        step: serde_json::json!({
            "type": "type_text",
            "text": "TODO",
        }),
    });
    v2.push(StepTemplate {
        label: "wait_frames",
        step: serde_json::json!({
            "type": "wait_frames",
            "n": 30,
        }),
    });
    v2.push(StepTemplate {
        label: "wait_until",
        step: serde_json::json!({
            "type": "wait_until",
            "window": window.clone(),
            "predicate": predicate.clone(),
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "assert",
        step: serde_json::json!({
            "type": "assert",
            "window": window.clone(),
            "predicate": placeholder_predicate_value(),
        }),
    });
    v2.push(StepTemplate {
        label: "capture_bundle",
        step: serde_json::json!({
            "type": "capture_bundle",
            "label": "devtools",
        }),
    });
    v2.push(StepTemplate {
        label: "capture_screenshot",
        step: serde_json::json!({
            "type": "capture_screenshot",
            "label": "devtools",
            "timeout_frames": 300,
        }),
    });
    v2.push(StepTemplate {
        label: "reset_diagnostics",
        step: serde_json::json!({
            "type": "reset_diagnostics",
        }),
    });

    v2.push(StepTemplate {
        label: "press_shortcut",
        step: serde_json::json!({
            "type": "press_shortcut",
            "shortcut": "Ctrl+P",
            "repeat": false,
        }),
    });
    v2.push(StepTemplate {
        label: "move_pointer_sweep",
        step: serde_json::json!({
            "type": "move_pointer_sweep",
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": 40.0,
            "steps": 8,
            "frames_per_step": 1,
        }),
    });
    v2.push(StepTemplate {
        label: "click_stable",
        step: serde_json::json!({
            "type": "click_stable",
            "target": placeholder_selector_value(),
            "button": "left",
            "stable_frames": 2,
            "max_move_px": 1.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "ensure_visible",
        step: serde_json::json!({
            "type": "ensure_visible",
            "target": placeholder_selector_value(),
            "within_window": true,
            "padding_px": 0.0,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "scroll_into_view",
        step: serde_json::json!({
            "type": "scroll_into_view",
            "container": placeholder_selector_value(),
            "target": placeholder_selector_value(),
            "delta_x": 0.0,
            "delta_y": -120.0,
            "require_fully_within_container": false,
            "require_fully_within_window": false,
            "padding_px": 0.0,
            "padding_insets_px": null,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "type_text_into",
        step: serde_json::json!({
            "type": "type_text_into",
            "target": placeholder_selector_value(),
            "text": "TODO",
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select",
        step: serde_json::json!({
            "type": "menu_select",
            "menu": placeholder_selector_value(),
            "item": placeholder_selector_value(),
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "menu_select_path",
        step: serde_json::json!({
            "type": "menu_select_path",
            "path": [placeholder_selector_value()],
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "drag_to",
        step: serde_json::json!({
            "type": "drag_to",
            "window": window.clone(),
            "from": placeholder_selector_value(),
            "to": placeholder_selector_value(),
            "button": "left",
            "steps": 8,
            "timeout_frames": 180,
        }),
    });
    v2.push(StepTemplate {
        label: "set_slider_value",
        step: serde_json::json!({
            "type": "set_slider_value",
            "target": placeholder_selector_value(),
            "value": 50.0,
            "min": 0.0,
            "max": 100.0,
            "epsilon": 0.5,
            "timeout_frames": 180,
            "drag_steps": 8,
        }),
    });
    v2.push(StepTemplate {
        label: "set_window_inner_size",
        step: serde_json::json!({
            "type": "set_window_inner_size",
            "window": window.clone(),
            "width_px": 1280.0,
            "height_px": 720.0,
        }),
    });
    v2.push(StepTemplate {
        label: "set_window_outer_position",
        step: serde_json::json!({
            "type": "set_window_outer_position",
            "window": window.clone(),
            "x_px": 100.0,
            "y_px": 100.0
        }),
    });
    v2.push(StepTemplate {
        label: "set_cursor_at_host_monitor",
        step: serde_json::json!({
            "type": "set_cursor_at_host_monitor",
            "selector": "highest_scale_factor",
            "x_fraction": 0.5,
            "y_fraction": 0.5,
            "offset_x_px": 0.0,
            "offset_y_px": 0.0
        }),
    });
    v2.push(StepTemplate {
        label: "set_cursor_in_window",
        step: serde_json::json!({
            "type": "set_cursor_in_window",
            "window": window.clone(),
            "x_px": 200.0,
            "y_px": 200.0,
        }),
    });
    v2.push(StepTemplate {
        label: "raise_window",
        step: serde_json::json!({
            "type": "raise_window",
            "window": window.clone(),
        }),
    });

    v2
}

fn selector_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let test_id = shadcn::Input::new(st.script_selector_test_id.clone())
        .a11y_label("test_id")
        .placeholder("button.ok")
        .into_element(cx);
    let role = shadcn::Input::new(st.script_selector_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let name = shadcn::Input::new(st.script_selector_name.clone())
        .a11y_label("name")
        .placeholder("OK")
        .into_element(cx);
    let ancestors = shadcn::Textarea::new(st.script_selector_ancestors.clone())
        .a11y_label("ancestors (role:name per line)")
        .min_height(Px(120.0))
        .into_element(cx);
    let node_id = shadcn::Input::new(st.script_selector_node_id.clone())
        .a11y_label("node_id")
        .placeholder("123")
        .into_element(cx);
    let element_id = shadcn::Input::new(st.script_selector_element_id.clone())
        .a11y_label("global_element_id")
        .placeholder("123")
        .into_element(cx);

    match kind {
        "test_id" => ui::v_stack(|_cx| [test_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "role_and_name" => ui::v_stack(|_cx| [role, name])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "role_and_path" => ui::v_stack(|_cx| [role, name, ancestors])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "node_id" => ui::v_stack(|_cx| [node_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "global_element_id" => ui::v_stack(|_cx| [element_id])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        _ => cx.text("unknown selector kind"),
    }
}

fn selector_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
) -> serde_json::Value {
    let test_id = cx
        .app
        .models()
        .read(&st.script_selector_test_id, |v| v.clone())
        .unwrap_or_default();
    let role = cx
        .app
        .models()
        .read(&st.script_selector_role, |v| v.clone())
        .unwrap_or_default();
    let name = cx
        .app
        .models()
        .read(&st.script_selector_name, |v| v.clone())
        .unwrap_or_default();
    let ancestors_text = cx
        .app
        .models()
        .read(&st.script_selector_ancestors, |v| v.clone())
        .unwrap_or_default();
    let node_id = cx
        .app
        .models()
        .read(&st.script_selector_node_id, |v| v.clone())
        .unwrap_or_default();
    let element_id = cx
        .app
        .models()
        .read(&st.script_selector_element_id, |v| v.clone())
        .unwrap_or_default();

    match kind {
        "test_id" => serde_json::json!({
            "kind": "test_id",
            "id": test_id.trim(),
        }),
        "role_and_name" => serde_json::json!({
            "kind": "role_and_name",
            "role": role.trim(),
            "name": name.trim(),
        }),
        "role_and_path" => serde_json::json!({
            "kind": "role_and_path",
            "role": role.trim(),
            "name": name.trim(),
            "ancestors": parse_ancestors_lines(&ancestors_text),
        }),
        "node_id" => serde_json::json!({
            "kind": "node_id",
            "node": node_id.trim().parse::<u64>().unwrap_or(0),
        }),
        "global_element_id" => serde_json::json!({
            "kind": "global_element_id",
            "element": element_id.trim().parse::<u64>().unwrap_or(0),
        }),
        _ => placeholder_selector_value(),
    }
}

fn predicate_fields(cx: &mut ElementContext<'_, App>, st: &State, kind: &str) -> AnyElement {
    let role = shadcn::Input::new(st.script_predicate_role.clone())
        .a11y_label("role")
        .placeholder("button")
        .into_element(cx);
    let checked = shadcn::Checkbox::new(st.script_predicate_checked.clone())
        .a11y_label("checked")
        .into_element(cx);
    let len_bytes = shadcn::Input::new(st.script_predicate_len_bytes.clone())
        .a11y_label("len_bytes")
        .placeholder("0")
        .into_element(cx);
    let min_len_bytes = shadcn::Input::new(st.script_predicate_len_bytes.clone())
        .a11y_label("min_len_bytes")
        .placeholder("0")
        .into_element(cx);
    let padding = shadcn::Input::new(st.script_predicate_padding_px.clone())
        .a11y_label("padding_px")
        .placeholder("0")
        .into_element(cx);
    let eps = shadcn::Input::new(st.script_predicate_eps_px.clone())
        .a11y_label("eps_px")
        .placeholder("0")
        .into_element(cx);
    let min_w = shadcn::Input::new(st.script_predicate_min_w_px.clone())
        .a11y_label("min_w_px")
        .placeholder("0")
        .into_element(cx);
    let min_h = shadcn::Input::new(st.script_predicate_min_h_px.clone())
        .a11y_label("min_h_px")
        .placeholder("0")
        .into_element(cx);

    match kind {
        "role_is" => role,
        "checked_is" => checked,
        "label_len_is" | "value_len_is" => len_bytes,
        "label_len_ge" | "value_len_ge" => min_len_bytes,
        "barrier_roots" => {
            let barrier_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let focus_root_items = [
                shadcn::SelectItem::new("any", "any"),
                shadcn::SelectItem::new("none", "none"),
                shadcn::SelectItem::new("some", "some"),
            ];
            let require_items = [
                shadcn::SelectItem::new("unset", "unset"),
                shadcn::SelectItem::new("true", "true"),
                shadcn::SelectItem::new("false", "false"),
            ];

            let barrier_root = shadcn::Select::new(
                st.script_predicate_barrier_root.clone(),
                st.script_predicate_barrier_root_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("barrier_root"))
            .items(barrier_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let focus_root = shadcn::Select::new(
                st.script_predicate_focus_barrier_root.clone(),
                st.script_predicate_focus_barrier_root_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("focus_barrier_root"))
            .items(focus_root_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let require_equal = shadcn::Select::new(
                st.script_predicate_require_equal.clone(),
                st.script_predicate_require_equal_open.clone(),
            )
            .value(shadcn::SelectValue::new().placeholder("require_equal"))
            .items(require_items)
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx);

            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("other selector (optional)")
                    .min_height(Px(96.0))
                    .into_element(cx);

            ui::v_stack(|_cx| [barrier_root, focus_root, require_equal, other_selector])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx)
        }
        "bounds_within_window" => ui::v_stack(|_cx| [padding, eps])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "bounds_min_size" => ui::v_stack(|_cx| [min_w, min_h, eps])
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx),
        "bounds_non_overlapping"
        | "bounds_overlapping"
        | "bounds_overlapping_x"
        | "bounds_overlapping_y" => {
            let other_selector =
                shadcn::Textarea::new(st.script_predicate_other_selector_json.clone())
                    .a11y_label("selector B (JSON)")
                    .min_height(Px(120.0))
                    .into_element(cx);
            ui::v_stack(|_cx| [eps, other_selector])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                .into_element(cx)
        }
        _ => cx.text(""),
    }
}

fn predicate_value_from_models(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    kind: &str,
    selector: serde_json::Value,
) -> serde_json::Value {
    let role = cx
        .app
        .models()
        .read(&st.script_predicate_role, |v| v.clone())
        .unwrap_or_default();
    let other_selector_json = cx
        .app
        .models()
        .read(&st.script_predicate_other_selector_json, |v| v.clone())
        .unwrap_or_default();
    let checked = cx
        .app
        .models()
        .read(&st.script_predicate_checked, |v| *v)
        .unwrap_or(false);
    let len_bytes = parse_u32_model(cx, &st.script_predicate_len_bytes);
    let padding_px = parse_f32_model(cx, &st.script_predicate_padding_px);
    let eps_px = parse_f32_model(cx, &st.script_predicate_eps_px);
    let min_w_px = parse_f32_model(cx, &st.script_predicate_min_w_px);
    let min_h_px = parse_f32_model(cx, &st.script_predicate_min_h_px);

    let other_selector = serde_json::from_str::<serde_json::Value>(&other_selector_json)
        .ok()
        .unwrap_or_else(placeholder_selector_value);

    match kind {
        "exists" => serde_json::json!({
            "kind": "exists",
            "target": selector,
        }),
        "not_exists" => serde_json::json!({
            "kind": "not_exists",
            "target": selector,
        }),
        "focus_is" => serde_json::json!({
            "kind": "focus_is",
            "target": selector,
        }),
        "role_is" => serde_json::json!({
            "kind": "role_is",
            "target": selector,
            "role": role.trim(),
        }),
        "checked_is" => serde_json::json!({
            "kind": "checked_is",
            "target": selector,
            "checked": checked,
        }),
        "checked_is_none" => serde_json::json!({
            "kind": "checked_is_none",
            "target": selector,
        }),
        "label_len_is" => serde_json::json!({
            "kind": "label_len_is",
            "target": selector,
            "len_bytes": len_bytes,
        }),
        "label_len_ge" => serde_json::json!({
            "kind": "label_len_ge",
            "target": selector,
            "min_len_bytes": len_bytes,
        }),
        "value_len_is" => serde_json::json!({
            "kind": "value_len_is",
            "target": selector,
            "len_bytes": len_bytes,
        }),
        "value_len_ge" => serde_json::json!({
            "kind": "value_len_ge",
            "target": selector,
            "min_len_bytes": len_bytes,
        }),
        "barrier_roots" => {
            let barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let focus_barrier_root = cx
                .app
                .models()
                .read(&st.script_predicate_focus_barrier_root, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("any"));
            let require_equal = cx
                .app
                .models()
                .read(&st.script_predicate_require_equal, |v| v.clone())
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from("unset"));

            let mut obj = serde_json::Map::new();
            obj.insert(
                "kind".to_string(),
                serde_json::Value::String("barrier_roots".to_string()),
            );
            obj.insert(
                "barrier_root".to_string(),
                serde_json::Value::String(barrier_root.to_string()),
            );
            obj.insert(
                "focus_barrier_root".to_string(),
                serde_json::Value::String(focus_barrier_root.to_string()),
            );
            if require_equal.as_ref() == "true" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(true));
            } else if require_equal.as_ref() == "false" {
                obj.insert("require_equal".to_string(), serde_json::Value::Bool(false));
            }
            serde_json::Value::Object(obj)
        }
        "visible_in_window" => serde_json::json!({
            "kind": "visible_in_window",
            "target": selector,
        }),
        "bounds_within_window" => serde_json::json!({
            "kind": "bounds_within_window",
            "target": selector,
            "padding_px": padding_px,
            "eps_px": eps_px,
        }),
        "bounds_min_size" => serde_json::json!({
            "kind": "bounds_min_size",
            "target": selector,
            "min_w_px": min_w_px,
            "min_h_px": min_h_px,
            "eps_px": eps_px,
        }),
        "bounds_non_overlapping" => serde_json::json!({
            "kind": "bounds_non_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping" => serde_json::json!({
            "kind": "bounds_overlapping",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_x" => serde_json::json!({
            "kind": "bounds_overlapping_x",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        "bounds_overlapping_y" => serde_json::json!({
            "kind": "bounds_overlapping_y",
            "a": selector,
            "b": other_selector,
            "eps_px": eps_px,
        }),
        _ => placeholder_predicate_value(),
    }
}

fn parse_f32_model(cx: &mut ElementContext<'_, App>, m: &Model<String>) -> f32 {
    cx.app
        .models()
        .read(m, |v| v.trim().parse::<f32>().ok())
        .ok()
        .flatten()
        .unwrap_or(0.0)
}

fn parse_u32_model(cx: &mut ElementContext<'_, App>, m: &Model<String>) -> u32 {
    cx.app
        .models()
        .read(m, |v| v.trim().parse::<u32>().ok())
        .ok()
        .flatten()
        .unwrap_or(0)
}

fn parse_ancestors_lines(text: &str) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((role, name)) = line.split_once(':') else {
            continue;
        };
        let role = role.trim();
        let name = name.trim();
        if role.is_empty() || name.is_empty() {
            continue;
        }
        out.push(serde_json::json!({
            "role": role,
            "name": name,
        }));
    }
    out
}
