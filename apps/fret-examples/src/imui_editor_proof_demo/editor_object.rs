use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_editor::composites::{PropertyGrid, PropertyGroup, PropertyGroupOptions, PropertyRow};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, TextField, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome,
};

use super::editor_text_assist::{
    editor_demo_name_assist_items, record_text_field_outcome, render_editor_name_assist_surface,
};
use super::proof_helpers::{
    committed_char_count_label, committed_line_count_label, editor_string_model_readout,
    editor_text_assist_readout, editor_text_field_readout, proof_compact_readout,
};

#[derive(Clone)]
pub(super) struct EditorObjectModels {
    pub(super) name: Model<String>,
    pub(super) buffered_name: Model<String>,
    pub(super) inline_rename: Model<String>,
    pub(super) inline_rename_outcome: Model<String>,
    pub(super) name_assist: Model<String>,
    pub(super) name_assist_dismissed_query: Model<String>,
    pub(super) name_assist_active_item: Model<Option<Arc<str>>>,
    pub(super) name_assist_accepted: Model<String>,
    pub(super) password: Model<String>,
    pub(super) password_outcome: Model<String>,
    pub(super) notes: Model<String>,
    pub(super) notes_outcome: Model<String>,
}

pub(super) fn render_editor_object_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorObjectModels,
) -> AnyElement {
    PropertyGroup::new("Object")
        .options(PropertyGroupOptions {
            test_id: Some(Arc::from("imui-editor-proof.editor.group.object")),
            header_test_id: Some(Arc::from("imui-editor-proof.editor.group.object.header")),
            content_test_id: Some(Arc::from("imui-editor-proof.editor.group.object.content")),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx| None,
            move |cx| vec![render_editor_object_grid(cx, models)],
        )
}

fn render_editor_object_grid(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorObjectModels,
) -> AnyElement {
    PropertyGrid::new().into_element(cx, move |cx, row_cx| {
        let mut rows = Vec::new();

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Name"),
            |cx| {
                TextField::new(models.name.clone())
                    .options(TextFieldOptions {
                        placeholder: Some(Arc::from("Untitled")),
                        clear_button: true,
                        selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                        test_id: Some(Arc::from("imui-editor-proof.editor.object.name")),
                        clear_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.object.name.clear",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Inline rename"),
            |cx| {
                let outcome_model = models.inline_rename_outcome.clone();
                TextField::new(models.inline_rename.clone())
                    .on_outcome(Some(Arc::new(
                        move |host, action_cx, outcome: TextFieldOutcome| {
                            record_text_field_outcome(host, action_cx, &outcome_model, outcome);
                        },
                    )))
                    .options(TextFieldOptions {
                        id_source: Some(Arc::from("imui-editor-proof.editor.object.inline-rename")),
                        placeholder: Some(Arc::from("Rename selection")),
                        clear_button: true,
                        selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                        blur_behavior: TextFieldBlurBehavior::Cancel,
                        test_id: Some(Arc::from("imui-editor-proof.editor.object.inline-rename")),
                        clear_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.object.inline-rename.clear",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        ));

        let inline_rename_readout =
            editor_text_field_readout(cx, &models.inline_rename, &models.inline_rename_outcome);
        let inline_rename_committed = inline_rename_readout.committed.clone();
        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Rename committed"),
            move |cx| {
                proof_compact_readout(
                    cx,
                    inline_rename_committed.clone(),
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.inline-rename.committed",
                    )),
                )
            },
            |_cx| None,
        ));

        let inline_rename_outcome = inline_rename_readout.outcome;
        if !inline_rename_outcome.trim().is_empty() {
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new(),
                |cx| row_cx.label_text(cx, "Rename outcome"),
                move |cx| {
                    proof_compact_readout(
                        cx,
                        inline_rename_outcome.clone(),
                        Some(Arc::from(
                            "imui-editor-proof.editor.object.inline-rename.outcome",
                        )),
                    )
                },
                |_cx| None,
            ));
        }

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Buffered name"),
            |cx| {
                TextField::new(models.buffered_name.clone())
                    .options(TextFieldOptions {
                        id_source: Some(Arc::from("imui-editor-proof.editor.object.buffered-name")),
                        placeholder: Some(Arc::from("Buffered session")),
                        clear_button: true,
                        test_id: Some(Arc::from("imui-editor-proof.editor.object.buffered-name")),
                        clear_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.object.buffered-name.clear",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Password"),
            |cx| {
                let outcome_model = models.password_outcome.clone();
                TextField::new(models.password.clone())
                    .on_outcome(Some(Arc::new(
                        move |host, action_cx, outcome: TextFieldOutcome| {
                            record_text_field_outcome(host, action_cx, &outcome_model, outcome);
                        },
                    )))
                    .options(TextFieldOptions {
                        id_source: Some(Arc::from("imui-editor-proof.editor.object.password")),
                        placeholder: Some(Arc::from("Editor password")),
                        clear_button: true,
                        mode: TextFieldMode::Password,
                        test_id: Some(Arc::from("imui-editor-proof.editor.object.password")),
                        clear_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.object.password.clear",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        ));

        let password_readout =
            editor_text_field_readout(cx, &models.password, &models.password_outcome);
        let password_committed = password_readout.committed.clone();
        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Secret length"),
            move |cx| {
                let readout = committed_char_count_label(&password_committed);
                proof_compact_readout(
                    cx,
                    readout,
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.password.committed-length",
                    )),
                )
            },
            |_cx| None,
        ));

        let password_outcome = password_readout.outcome;
        if !password_outcome.trim().is_empty() {
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new(),
                |cx| row_cx.label_text(cx, "Password outcome"),
                move |cx| {
                    proof_compact_readout(
                        cx,
                        password_outcome.clone(),
                        Some(Arc::from(
                            "imui-editor-proof.editor.object.password.outcome",
                        )),
                    )
                },
                |_cx| None,
            ));
        }

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Committed"),
            |cx| {
                let committed = editor_string_model_readout(cx, &models.buffered_name);
                proof_compact_readout(
                    cx,
                    committed,
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.buffered-name.committed",
                    )),
                )
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Name assist"),
            |cx| {
                render_editor_name_assist_surface(
                    cx,
                    models.name_assist.clone(),
                    models.name_assist_dismissed_query.clone(),
                    models.name_assist_active_item.clone(),
                    models.name_assist_accepted.clone(),
                )
                .into_element(cx)
            },
            |_cx| None,
        ));

        let name_assist_items = editor_demo_name_assist_items(cx);
        let name_assist_readout = editor_text_assist_readout(
            cx,
            name_assist_items,
            &models.name_assist,
            &models.name_assist_dismissed_query,
            &models.name_assist_active_item,
        );
        let name_assist_state = name_assist_readout.state_label.clone();
        let name_assist_active = name_assist_readout.active_label.clone();

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Assist state"),
            move |cx| {
                proof_compact_readout(
                    cx,
                    name_assist_state.clone(),
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.name-assist.state",
                    )),
                )
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Active assist"),
            move |cx| {
                proof_compact_readout(
                    cx,
                    name_assist_active.clone(),
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.name-assist.active",
                    )),
                )
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Accepted assist"),
            |cx| {
                let accepted = editor_string_model_readout(cx, &models.name_assist_accepted);
                let readout = if accepted.trim().is_empty() {
                    "None".to_string()
                } else {
                    accepted
                };
                proof_compact_readout(
                    cx,
                    readout,
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.name-assist.accepted",
                    )),
                )
            },
            |_cx| None,
        ));

        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Notes"),
            |cx| {
                let outcome_model = models.notes_outcome.clone();
                TextField::new(models.notes.clone())
                    .on_outcome(Some(Arc::new(
                        move |host, action_cx, outcome: TextFieldOutcome| {
                            record_text_field_outcome(host, action_cx, &outcome_model, outcome);
                        },
                    )))
                    .options(TextFieldOptions {
                        id_source: Some(Arc::from("imui-editor-proof.editor.object.notes")),
                        multiline: true,
                        min_height: Some(Px(96.0)),
                        clear_button: true,
                        blur_behavior: TextFieldBlurBehavior::PreserveDraft,
                        test_id: Some(Arc::from("imui-editor-proof.editor.object.notes")),
                        clear_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.object.notes.clear",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
            },
            |_cx| None,
        ));

        let notes_readout = editor_text_field_readout(cx, &models.notes, &models.notes_outcome);
        let notes_committed = notes_readout.committed.clone();
        rows.push(row_cx.row_with(
            cx,
            PropertyRow::new(),
            |cx| row_cx.label_text(cx, "Notes committed"),
            move |cx| {
                let readout = committed_line_count_label(&notes_committed);
                proof_compact_readout(
                    cx,
                    readout,
                    Some(Arc::from(
                        "imui-editor-proof.editor.object.notes.committed-lines",
                    )),
                )
            },
            |_cx| None,
        ));

        let notes_outcome = notes_readout.outcome;
        if !notes_outcome.trim().is_empty() {
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new(),
                |cx| row_cx.label_text(cx, "Notes outcome"),
                move |cx| {
                    proof_compact_readout(
                        cx,
                        notes_outcome.clone(),
                        Some(Arc::from("imui-editor-proof.editor.object.notes.outcome")),
                    )
                },
                |_cx| None,
            ));
        }

        rows
    })
}
