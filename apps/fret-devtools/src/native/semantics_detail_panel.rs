use std::sync::Arc;

use fret_app::App;
use fret_core::Px;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::ui_primitives::text_blob;
use super::{semantics, State};

pub(super) fn sem_node_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    let fallback = cx
        .app
        .models()
        .read(&st.semantics_selected_node_json, |v| v.clone())
        .unwrap_or_default();
    let live = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_json, |v| v.clone())
        .unwrap_or_default();
    let live_status = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_status, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("unknown"));
    let live_updated = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_updated_unix_ms, |v| *v)
        .ok()
        .flatten();
    let children = cx
        .app
        .models()
        .read(&st.semantics_selected_node_live_children, |v| v.clone())
        .unwrap_or_default();
    let hit_test_explain = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_json, |v| v.clone())
        .unwrap_or_default();
    let hit_test_explain_summary = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_summary, |v| {
            v.clone()
        })
        .unwrap_or_default();
    let hit_test_explain_status = cx
        .app
        .models()
        .read(&st.semantics_selected_hit_test_explain_status, |v| {
            v.clone()
        })
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("unknown"));
    let hit_test_explain_updated = cx
        .app
        .models()
        .read(
            &st.semantics_selected_hit_test_explain_updated_unix_ms,
            |v| *v,
        )
        .ok()
        .flatten();
    let live_enabled = cx
        .app
        .models()
        .read(&st.semantics_live_enabled, |v| *v)
        .unwrap_or(true);
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();

    let status_line = {
        let mut line = format!(
            "live_enabled={live_enabled} status={}",
            live_status.as_ref()
        );
        if let Some(ts) = live_updated {
            line.push_str(&format!(" updated_unix_ms={ts}"));
        }
        line
    };

    let hit_test_explain_status_line = {
        let mut line = format!(
            "hit_test.explain status={}",
            hit_test_explain_status.as_ref()
        );
        if let Some(ts) = hit_test_explain_updated {
            line.push_str(&format!(" updated_unix_ms={ts}"));
        }
        line
    };

    let live_toggle_label = if live_enabled {
        "Live: On"
    } else {
        "Live: Off"
    };
    let live_enabled_model = st.semantics_live_enabled.clone();
    let force_nonce_model = st.semantics_live_force_nonce.clone();
    let on_toggle: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&live_enabled_model, |v| *v = !*v);
        let _ = host
            .models_mut()
            .update(&force_nonce_model, |v| *v = v.saturating_add(1));
        host.request_redraw(action_cx.window);
    });

    let on_refresh: fret_ui::action::OnActivate = {
        let force_nonce_model = st.semantics_live_force_nonce.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&force_nonce_model, |v| *v = v.saturating_add(1));
            host.request_redraw(action_cx.window);
        })
    };

    let live_toggle_btn = shadcn::Button::new(live_toggle_label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_toggle)
        .into_element(cx);
    let refresh_btn = shadcn::Button::new("Refresh")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(on_refresh)
        .into_element(cx);
    let status_elem = cx.text(status_line);

    let header = ui::h_row(|_cx| [live_toggle_btn, refresh_btn, status_elem])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);

    let mut child_buttons: Vec<AnyElement> = Vec::new();
    child_buttons.reserve(children.len().min(64));
    if let (Some(index), Some(_selected)) = (index, selected_id) {
        for child in children.iter().take(200) {
            let id = *child;
            let label = index
                .node(id)
                .map(semantics::node_label)
                .unwrap_or_else(|| format!("id={id}"));

            let selected_id_model = st.semantics_selected_id.clone();
            let selected_json_model = st.semantics_selected_node_json.clone();
            let selected_live_json_model = st.semantics_selected_node_live_json.clone();
            let selected_live_status_model = st.semantics_selected_node_live_status.clone();
            let selected_live_updated_model =
                st.semantics_selected_node_live_updated_unix_ms.clone();
            let selected_live_children_model = st.semantics_selected_node_live_children.clone();
            let selected_hit_test_explain_json_model =
                st.semantics_selected_hit_test_explain_json.clone();
            let selected_hit_test_explain_summary_model =
                st.semantics_selected_hit_test_explain_summary.clone();
            let selected_hit_test_explain_status_model =
                st.semantics_selected_hit_test_explain_status.clone();
            let selected_hit_test_explain_updated_model = st
                .semantics_selected_hit_test_explain_updated_unix_ms
                .clone();
            let index_for_select = Arc::clone(&index);
            let on_child: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&selected_id_model, |v| *v = Some(id));
                    let text = semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                    let _ = host
                        .models_mut()
                        .update(&selected_json_model, |v| *v = text);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_json_model, |v| v.clear());
                    let _ = host.models_mut().update(&selected_live_status_model, |v| {
                        *v = None;
                    });
                    let _ = host
                        .models_mut()
                        .update(&selected_live_updated_model, |v| *v = None);
                    let _ = host
                        .models_mut()
                        .update(&selected_live_children_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_json_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_summary_model, |v| v.clear());
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_status_model, |v| *v = None);
                    let _ = host
                        .models_mut()
                        .update(&selected_hit_test_explain_updated_model, |v| *v = None);
                    host.request_redraw(action_cx.window);
                });

            child_buttons.push(
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_child)
                    .into_element(cx),
            );
        }
    }

    let children_panel = if child_buttons.is_empty() {
        cx.text("children: <none>")
    } else {
        shadcn::ScrollArea::new([ui::v_stack(|_cx| child_buttons)
            .gap(fret_ui_kit::Space::N1)
            .layout(fret_ui_kit::LayoutRefinement::default().w_full())
            .into_element(cx)])
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .h_px(Px(160.0)),
        )
        .into_element(cx)
    };

    let json_text = if !live.is_empty() { live } else { fallback };
    let live_body_title = cx.text("Live semantics JSON");
    let live_body_content = text_blob(cx, json_text);
    let live_body = ui::v_stack(|_cx| [live_body_title, live_body_content])
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx);
    let hit_test_explain_summary_body = if hit_test_explain_summary.is_empty() {
        cx.text("<no summary yet>")
    } else {
        text_blob(cx, hit_test_explain_summary)
    };
    let hit_test_explain_body = if hit_test_explain.is_empty() {
        cx.text("<no hit_test.explain_ack yet>")
    } else {
        text_blob(cx, hit_test_explain)
    };
    let hit_test_explain_status_text = cx.text(hit_test_explain_status_line);
    let hit_test_explain_summary_title = cx.text("Readable summary");
    let hit_test_explain_panel = ui::v_stack(|_cx| {
        [
            hit_test_explain_status_text,
            hit_test_explain_summary_title,
            hit_test_explain_summary_body,
            hit_test_explain_body,
        ]
    })
    .gap(fret_ui_kit::Space::N1)
    .layout(fret_ui_kit::LayoutRefinement::default().w_full())
    .into_element(cx);

    ui::v_stack(|_cx| [header, children_panel, live_body, hit_test_explain_panel])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
}
