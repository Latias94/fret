use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelCx, InspectorPanelOptions, InspectorPanelSearchAssistOptions,
};

use super::editor_advanced::render_editor_advanced_surface;
use super::editor_gradient::render_editor_gradient_surface;
use super::editor_material::render_editor_material_surface;
use super::editor_object::render_editor_object_surface;
use super::editor_state::{
    editor_demo_search_assist_active_item_model, editor_demo_search_assist_dismissed_query_model,
    editor_demo_search_model,
};
use super::editor_text_assist::editor_demo_search_assist_items;
use super::proof_helpers::proof_empty_state_text;

#[derive(Clone)]
pub(super) struct EditorInspectorModels {
    search: Model<String>,
    search_assist_dismissed_query: Model<String>,
    search_assist_active_item: Model<Option<Arc<str>>>,
}

pub(super) fn editor_inspector_models(
    cx: &mut ElementContext<'_, KernelApp>,
) -> EditorInspectorModels {
    EditorInspectorModels {
        search: editor_demo_search_model(cx),
        search_assist_dismissed_query: editor_demo_search_assist_dismissed_query_model(cx),
        search_assist_active_item: editor_demo_search_assist_active_item_model(cx),
    }
}

pub(super) fn render_editor_inspector_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorInspectorModels,
    editor_review_layout: bool,
) -> AnyElement {
    InspectorPanel::new(Some(models.search.clone()))
        .options(InspectorPanelOptions {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: if editor_review_layout {
                        Length::Fill
                    } else {
                        Length::Auto
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            test_id: Some(Arc::from("imui-editor-proof.editor.inspector")),
            header_test_id: Some(Arc::from("imui-editor-proof.editor.inspector.header")),
            search_test_id: Some(Arc::from("imui-editor-proof.editor.search")),
            search_clear_test_id: Some(Arc::from("imui-editor-proof.editor.search.clear")),
            search_assist: Some(InspectorPanelSearchAssistOptions {
                dismissed_query_model: models.search_assist_dismissed_query.clone(),
                active_item_id_model: models.search_assist_active_item.clone(),
                items: editor_demo_search_assist_items(cx),
                list_label: Arc::from("Inspector search history"),
                empty_label: Arc::from("No search history matches"),
                key_options: Default::default(),
                list_test_id: Some(Arc::from("imui-editor-proof.editor.search.list")),
                item_test_id_prefix: Some(Arc::from("imui-editor-proof.editor.search.list.item")),
                empty_test_id: Some(Arc::from("imui-editor-proof.editor.search.no-matches")),
                max_list_height: None,
            }),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx, _panel_cx| Vec::new(),
            move |cx, panel_cx| render_editor_inspector_content(cx, panel_cx),
        )
}

fn render_editor_inspector_content(
    cx: &mut ElementContext<'_, KernelApp>,
    panel_cx: &InspectorPanelCx,
) -> Vec<AnyElement> {
    let mut out = Vec::new();

    let object_surface = render_editor_object_surface(cx, panel_cx);
    let object_any_match = object_surface.any_match;
    if let Some(element) = object_surface.element {
        out.push(element);
    }

    let material_surface = render_editor_material_surface(cx, panel_cx);
    let material_any_match = material_surface.any_match;
    if let Some(element) = material_surface.element {
        out.push(element);
    }

    let gradient_surface = render_editor_gradient_surface(cx, panel_cx);
    let gradient_any_match = gradient_surface.any_match;
    if let Some(element) = gradient_surface.element {
        out.push(element);
    }

    let advanced_surface = render_editor_advanced_surface(cx, panel_cx);
    let advanced_any_match = advanced_surface.any_match;
    if let Some(element) = advanced_surface.element {
        out.push(element);
    }

    let any_match =
        object_any_match || material_any_match || gradient_any_match || advanced_any_match;

    if !panel_cx.is_query_empty() && !any_match {
        out.push(proof_empty_state_text(
            cx,
            "No matches",
            "imui-editor-proof.editor.no-matches",
        ));
    }

    out
}
