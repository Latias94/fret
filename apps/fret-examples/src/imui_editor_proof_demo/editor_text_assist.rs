use std::sync::Arc;

use fret::advanced::KernelApp;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface,
    TextFieldOptions, TextFieldOutcome,
};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::headless::text_assist::{TextAssistItem, TextAssistMatch};

use super::editor_state::named_demo_state;
use super::proof_helpers::edit_session_outcome_label;

pub(super) fn editor_demo_name_assist_items(
    cx: &mut ElementContext<'_, KernelApp>,
) -> Arc<[TextAssistItem]> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.name_assist_items",
        |_cx| {
            vec![
                TextAssistItem::new("cube", "Cube").aliases(vec![Arc::from("box")]),
                TextAssistItem::new("cylinder", "Cylinder"),
                TextAssistItem::new("capsule", "Capsule"),
                TextAssistItem::new("camera", "Camera").aliases(vec![Arc::from("cam")]),
                TextAssistItem::new("curve-editor", "Curve Editor"),
                TextAssistItem::new("directional-light", "Directional Light")
                    .aliases(vec![Arc::from("dir light")]),
            ]
            .into()
        },
    )
}

pub(super) fn editor_demo_search_assist_items(
    cx: &mut ElementContext<'_, KernelApp>,
) -> Arc<[TextAssistItem]> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.search_assist_items",
        |_cx| {
            vec![
                TextAssistItem::new("assist", "Assist"),
                TextAssistItem::new("material", "Material"),
                TextAssistItem::new("buffered", "Buffered"),
                TextAssistItem::new("gradient", "Gradient"),
                TextAssistItem::new("password", "Password"),
                TextAssistItem::new("validation", "Validation")
                    .aliases(vec![Arc::from("error"), Arc::from("invalid")]),
            ]
            .into()
        },
    )
}

fn record_editor_text_assist_accept(
    host: &mut dyn UiActionHost,
    accepted_label_model: &Model<String>,
    active: TextAssistMatch,
) {
    let next_query = active.label.as_ref().to_string();
    let _ = host.models_mut().update(accepted_label_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
}

pub(super) fn record_text_field_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    outcome_model: &Model<String>,
    outcome: TextFieldOutcome,
) {
    let next = edit_session_outcome_label(outcome);
    let _ = host.models_mut().update(outcome_model, |text| {
        text.clear();
        text.push_str(next);
    });
    host.request_redraw(action_cx.window);
}

pub(super) fn render_editor_name_assist_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    accepted_label_model: Model<String>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let items = editor_demo_name_assist_items(cx);
    TextAssistField::new(
        query_model,
        dismissed_query_model,
        active_item_id_model,
        items,
    )
    .options(TextAssistFieldOptions {
        field: TextFieldOptions {
            id_source: Some(Arc::from("imui-editor-proof.editor.object.name-assist")),
            placeholder: Some(Arc::from("Type to search object history")),
            clear_button: true,
            buffered: false,
            selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
            test_id: Some(Arc::from("imui-editor-proof.editor.object.name-assist")),
            clear_test_id: Some(Arc::from(
                "imui-editor-proof.editor.object.name-assist.clear",
            )),
            ..Default::default()
        },
        surface: TextAssistFieldSurface::AnchoredOverlay,
        list_label: Arc::from("Name history suggestions"),
        list_test_id: Some(Arc::from(
            "imui-editor-proof.editor.object.name-assist.list",
        )),
        empty_test_id: Some(Arc::from(
            "imui-editor-proof.editor.object.name-assist.no-matches",
        )),
        ..Default::default()
    })
    .on_accept(Some(Arc::new(move |host, _action_cx, active| {
        record_editor_text_assist_accept(host, &accepted_label_model, active);
    })))
    .into_element(cx)
}
