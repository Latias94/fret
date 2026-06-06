use std::sync::Arc;

use fret_app::App;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::{gate_run, short_artifact_result_path, text_blob_sized, workflow_run};

pub(super) fn gate_run_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[gate_run::GateRunResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(cx, "gate run history: <none>".to_string(), Px(84.0));
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_result_path_model, |value| {
                    *value = Some(Arc::<str>::from(result_path.clone()))
                });
            host.request_redraw(action_cx.window);
        });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}

pub(super) fn workflow_run_history_list(
    cx: &mut ElementContext<'_, App>,
    selected_result_path_model: &Model<Option<Arc<str>>>,
    entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    active_result_path: Option<&str>,
) -> AnyElement {
    if entries.is_empty() {
        return text_blob_sized(cx, "workflow run history: <none>".to_string(), Px(84.0));
    }

    let mut rows: Vec<AnyElement> = Vec::new();
    for entry in entries.iter().take(8) {
        let is_selected = active_result_path.is_some_and(|path| path == entry.result_path);
        let result_path = entry.result_path.clone();
        let selected_result_path_model = selected_result_path_model.clone();
        let label = format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        );
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_result_path_model, |value| {
                    *value = Some(Arc::<str>::from(result_path.clone()))
                });
            host.request_redraw(action_cx.window);
        });
        rows.push(
            shadcn::Button::new(label)
                .variant(if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                })
                .size(shadcn::ButtonSize::Sm)
                .on_activate(on_activate)
                .into_element(cx),
        );
    }

    shadcn::ScrollArea::new([ui::v_stack(|_cx| rows)
        .gap(fret_ui_kit::Space::N1)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)])
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .min_h(Px(116.0)),
    )
    .into_element(cx)
}
