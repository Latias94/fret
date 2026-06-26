use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_editor::composites::{
    InspectorPanelCx, PropertyGrid, PropertyGroup, PropertyGroupOptions, PropertyRow,
};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, TextField, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome,
};

use super::super::editor_state::{
    editor_demo_buffered_name_model, editor_demo_inline_rename_model,
    editor_demo_inline_rename_outcome_model, editor_demo_name_assist_accepted_model,
    editor_demo_name_assist_active_item_model, editor_demo_name_assist_dismissed_query_model,
    editor_demo_name_assist_model, editor_demo_name_model, editor_demo_notes_model,
    editor_demo_notes_outcome_model, editor_demo_password_model,
    editor_demo_password_outcome_model,
};
use super::super::editor_text_assist::{
    editor_demo_name_assist_items, record_text_field_outcome, render_editor_name_assist_surface,
};
use super::super::proof_helpers::{
    committed_char_count_label, committed_line_count_label, editor_string_model_readout,
    editor_text_assist_readout, editor_text_field_readout, proof_compact_readout,
    proof_empty_state_text,
};

#[derive(Clone)]
pub struct EditorObjectModels {
    pub name: Model<String>,
    pub buffered_name: Model<String>,
    pub inline_rename: Model<String>,
    pub inline_rename_outcome: Model<String>,
    pub name_assist: Model<String>,
    pub name_assist_dismissed_query: Model<String>,
    pub name_assist_active_item: Model<Option<Arc<str>>>,
    pub name_assist_accepted: Model<String>,
    pub password: Model<String>,
    pub password_outcome: Model<String>,
    pub notes: Model<String>,
    pub notes_outcome: Model<String>,
}

pub struct EditorObjectSurface {
    pub element: Option<AnyElement>,
    pub any_match: bool,
}

fn editor_object_models(cx: &mut ElementContext<'_, KernelApp>) -> EditorObjectModels {
    EditorObjectModels {
        name: editor_demo_name_model(cx),
        buffered_name: editor_demo_buffered_name_model(cx),
        inline_rename: editor_demo_inline_rename_model(cx),
        inline_rename_outcome: editor_demo_inline_rename_outcome_model(cx),
        name_assist: editor_demo_name_assist_model(cx),
        name_assist_dismissed_query: editor_demo_name_assist_dismissed_query_model(cx),
        name_assist_active_item: editor_demo_name_assist_active_item_model(cx),
        name_assist_accepted: editor_demo_name_assist_accepted_model(cx),
        password: editor_demo_password_model(cx),
        password_outcome: editor_demo_password_outcome_model(cx),
        notes: editor_demo_notes_model(cx),
        notes_outcome: editor_demo_notes_outcome_model(cx),
    }
}

#[derive(Clone, Copy)]
struct EditorObjectVisibility {
    name: bool,
    inline_rename: bool,
    rename_committed: bool,
    rename_outcome: bool,
    buffered_name: bool,
    password: bool,
    secret_length: bool,
    password_outcome: bool,
    committed: bool,
    name_assist: bool,
    assist_state: bool,
    active_assist: bool,
    accepted_assist: bool,
    notes: bool,
    notes_committed: bool,
    notes_outcome: bool,
}

impl EditorObjectVisibility {
    fn from_panel(panel_cx: &InspectorPanelCx) -> Self {
        let object_show_all = panel_cx.matches("object");
        Self {
            name: object_show_all || panel_cx.matches("name"),
            inline_rename: object_show_all
                || panel_cx.matches("inline")
                || panel_cx.matches("rename"),
            rename_committed: object_show_all
                || panel_cx.matches("rename")
                || panel_cx.matches("committed"),
            rename_outcome: object_show_all
                || panel_cx.matches("rename")
                || panel_cx.matches("outcome"),
            buffered_name: object_show_all
                || panel_cx.matches("buffered")
                || panel_cx.matches("name"),
            password: object_show_all || panel_cx.matches("password"),
            secret_length: object_show_all
                || panel_cx.matches("secret")
                || panel_cx.matches("length")
                || panel_cx.matches("password"),
            password_outcome: object_show_all
                || panel_cx.matches("password")
                || panel_cx.matches("outcome"),
            committed: object_show_all
                || panel_cx.matches("buffered")
                || panel_cx.matches("name")
                || panel_cx.matches("committed"),
            name_assist: object_show_all || panel_cx.matches("name") || panel_cx.matches("assist"),
            assist_state: object_show_all
                || panel_cx.matches("assist")
                || panel_cx.matches("state"),
            active_assist: object_show_all
                || panel_cx.matches("assist")
                || panel_cx.matches("active"),
            accepted_assist: object_show_all
                || panel_cx.matches("assist")
                || panel_cx.matches("accepted"),
            notes: object_show_all || panel_cx.matches("notes"),
            notes_committed: object_show_all
                || panel_cx.matches("notes")
                || panel_cx.matches("committed")
                || panel_cx.matches("lines"),
            notes_outcome: object_show_all
                || panel_cx.matches("notes")
                || panel_cx.matches("outcome"),
        }
    }

    fn any_match(self) -> bool {
        self.name
            || self.inline_rename
            || self.rename_committed
            || self.rename_outcome
            || self.buffered_name
            || self.password
            || self.secret_length
            || self.password_outcome
            || self.committed
            || self.name_assist
            || self.assist_state
            || self.active_assist
            || self.accepted_assist
            || self.notes
            || self.notes_committed
            || self.notes_outcome
    }
}

pub fn render_editor_object_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    panel_cx: &InspectorPanelCx,
) -> EditorObjectSurface {
    let visibility = EditorObjectVisibility::from_panel(panel_cx);
    let any_match = visibility.any_match();
    let element = if any_match {
        let models = editor_object_models(cx);
        Some(
            PropertyGroup::new("Object")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    test_id: Some(Arc::from("imui-editor-proof.editor.group.object")),
                    header_test_id: Some(Arc::from("imui-editor-proof.editor.group.object.header")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx| None,
                    move |cx| vec![render_editor_object_grid(cx, visibility, models)],
                ),
        )
    } else {
        None
    };

    EditorObjectSurface { element, any_match }
}

fn render_editor_object_grid(
    cx: &mut ElementContext<'_, KernelApp>,
    visibility: EditorObjectVisibility,
    models: EditorObjectModels,
) -> AnyElement {
    PropertyGrid::new().into_element(cx, move |cx, row_cx| {
        let mut rows = Vec::new();

        if visibility.name {
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
        }

        if visibility.inline_rename || visibility.rename_committed || visibility.rename_outcome {
            let inline_rename_readout =
                editor_text_field_readout(cx, &models.inline_rename, &models.inline_rename_outcome);
            let inline_rename_committed = inline_rename_readout.committed.clone();

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
                            id_source: Some(Arc::from(
                                "imui-editor-proof.editor.object.inline-rename",
                            )),
                            placeholder: Some(Arc::from("Rename selection")),
                            clear_button: true,
                            selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                            blur_behavior: TextFieldBlurBehavior::Cancel,
                            test_id: Some(Arc::from(
                                "imui-editor-proof.editor.object.inline-rename",
                            )),
                            clear_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.object.inline-rename.clear",
                            )),
                            ..Default::default()
                        })
                        .into_element(cx)
                },
                |_cx| None,
            ));

            if visibility.rename_committed {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Rename committed"),
                    |cx| {
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
            }

            let inline_rename_outcome = inline_rename_readout.outcome;
            if visibility.rename_outcome && !inline_rename_outcome.trim().is_empty() {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Rename outcome"),
                    |cx| {
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
        }

        if visibility.buffered_name || visibility.committed {
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new(),
                |cx| row_cx.label_text(cx, "Buffered name"),
                |cx| {
                    TextField::new(models.buffered_name.clone())
                        .options(TextFieldOptions {
                            id_source: Some(Arc::from(
                                "imui-editor-proof.editor.object.buffered-name",
                            )),
                            placeholder: Some(Arc::from("Buffered session")),
                            clear_button: true,
                            test_id: Some(Arc::from(
                                "imui-editor-proof.editor.object.buffered-name",
                            )),
                            clear_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.object.buffered-name.clear",
                            )),
                            ..Default::default()
                        })
                        .into_element(cx)
                },
                |_cx| None,
            ));
        }

        if visibility.password || visibility.secret_length || visibility.password_outcome {
            let password_readout =
                editor_text_field_readout(cx, &models.password, &models.password_outcome);
            let password_committed = password_readout.committed.clone();

            if visibility.password {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Password"),
                    |cx| {
                        let outcome_model = models.password_outcome.clone();
                        TextField::new(models.password.clone())
                            .on_outcome(Some(Arc::new(
                                move |host, action_cx, outcome: TextFieldOutcome| {
                                    record_text_field_outcome(
                                        host,
                                        action_cx,
                                        &outcome_model,
                                        outcome,
                                    );
                                },
                            )))
                            .options(TextFieldOptions {
                                id_source: Some(Arc::from(
                                    "imui-editor-proof.editor.object.password",
                                )),
                                placeholder: Some(Arc::from("Editor password")),
                                clear_button: true,
                                mode: TextFieldMode::Password,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.editor.object.password",
                                )),
                                clear_test_id: Some(Arc::from(
                                    "imui-editor-proof.editor.object.password.clear",
                                )),
                                ..Default::default()
                            })
                            .into_element(cx)
                    },
                    |_cx| None,
                ));
            }

            if visibility.secret_length {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Secret length"),
                    |cx| {
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
            }

            let password_outcome = password_readout.outcome;
            if visibility.password_outcome && !password_outcome.trim().is_empty() {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Password outcome"),
                    |cx| {
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
        }

        if visibility.committed {
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
        }

        if visibility.name_assist
            || visibility.assist_state
            || visibility.active_assist
            || visibility.accepted_assist
        {
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

            if visibility.name_assist {
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
            }

            if visibility.assist_state {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Assist state"),
                    |cx| {
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
            }

            if visibility.active_assist {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Active assist"),
                    |cx| {
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
            }

            if visibility.accepted_assist {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Accepted assist"),
                    |cx| {
                        let accepted =
                            editor_string_model_readout(cx, &models.name_assist_accepted);
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
            }
        }

        if visibility.notes || visibility.notes_committed || visibility.notes_outcome {
            let notes_readout = editor_text_field_readout(cx, &models.notes, &models.notes_outcome);
            let notes_committed = notes_readout.committed.clone();

            if visibility.notes {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Notes"),
                    |cx| {
                        let outcome_model = models.notes_outcome.clone();
                        TextField::new(models.notes.clone())
                            .on_outcome(Some(Arc::new(
                                move |host, action_cx, outcome: TextFieldOutcome| {
                                    record_text_field_outcome(
                                        host,
                                        action_cx,
                                        &outcome_model,
                                        outcome,
                                    );
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
            }

            if visibility.notes_committed {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Notes committed"),
                    |cx| {
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
            }

            let notes_outcome = notes_readout.outcome;
            if visibility.notes_outcome && !notes_outcome.trim().is_empty() {
                rows.push(row_cx.row_with(
                    cx,
                    PropertyRow::new(),
                    |cx| row_cx.label_text(cx, "Notes outcome"),
                    |cx| {
                        proof_compact_readout(
                            cx,
                            notes_outcome.clone(),
                            Some(Arc::from("imui-editor-proof.editor.object.notes.outcome")),
                        )
                    },
                    |_cx| None,
                ));
            }
        }

        if rows.is_empty() {
            rows.push(proof_empty_state_text(
                cx,
                "No matches",
                "imui-editor-proof.editor.object.no-matches",
            ));
        }

        rows
    })
}
