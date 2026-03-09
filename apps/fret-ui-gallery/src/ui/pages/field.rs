use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::field as snippets;

pub(super) fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let input = snippets::input::render(cx);
    let textarea = snippets::textarea::render(cx);
    let select = snippets::select::render(cx);
    let slider = snippets::slider::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let switch = snippets::switch::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let field_group = snippets::field_group::render(cx);
    let rtl = snippets::rtl::render(cx);
    let responsive = snippets::responsive::render(cx);
    let validation_and_errors = snippets::validation_and_errors::render(cx);

    let usage = doc_layout::muted_full_width(
        cx,
        "Import the Field parts and compose them as needed (examples below mirror the upstream docs).",
    );
    let anatomy = doc_layout::muted_full_width(
        cx,
        "A typical Field groups a label + control + optional helper/error text.",
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/field.rs` (Field, FieldSet, FieldGroup, FieldLabel, FieldDescription, FieldSeparator).",
            "Field page follows upstream docs section order to keep parity checks deterministic.",
            "Each section keeps a stable `test_id` so diag scripts can target specific examples.",
            "RTL and Responsive samples are included to exercise orientation and direction contracts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Field docs order: Usage, Anatomy, Input, Textarea, Select, Slider, Fieldset, Checkbox, Radio, Switch, Choice Card, Field Group, Responsive Layout, Validation and Errors (plus an extra RTL section).",
        ),
        vec![
            DocSection::new("Usage", usage)
                .description("Rust imports mirror the upstream shadcn `Field` API surface.")
                .code_rust(
                    r#"use fret_ui_shadcn::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};"#,
                ),
            DocSection::new("Anatomy", anatomy)
                .description("Aligns with the upstream shadcn Field anatomy section.")
                .code_rust(
                    r#"Field::new([
    FieldLabel::new("Label").into_element(cx),
    /* Input / Select / Switch / ... */,
    FieldDescription::new("Optional helper text.").into_element(cx),
    FieldError::new("Validation message.").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Input", input)
                .description("Basic text inputs with labels + helper copy.")
                .code_rust_from_file_region(snippets::input::SOURCE, "example"),
            DocSection::new("Textarea", textarea)
                .description("Textarea field with explicit height and helper copy.")
                .code_rust_from_file_region(snippets::textarea::SOURCE, "example"),
            DocSection::new("Select", select)
                .description("Select composed inside a Field shell.")
                .code_rust_from_file_region(snippets::select::SOURCE, "example"),
            DocSection::new("Slider", slider)
                .description(
                    "Non-text controls should still use FieldTitle/Description for context.",
                )
                .code_rust_from_file_region(snippets::slider::SOURCE, "example"),
            DocSection::new("Fieldset", fieldset)
                .description("FieldSet groups multiple fields with a legend + description.")
                .code_rust_from_file_region(snippets::fieldset::SOURCE, "example"),
            DocSection::new("Checkbox", checkbox)
                .description("Horizontal Field orientation keeps checkbox + label aligned.")
                .code_rust_from_file_region(snippets::checkbox::SOURCE, "example"),
            DocSection::new("Radio", radio)
                .description("RadioGroup nested under Field for label copy.")
                .code_rust_from_file_region(snippets::radio::SOURCE, "example"),
            DocSection::new("Switch", switch)
                .description("Switch composed with title + description.")
                .code_rust_from_file_region(snippets::switch::SOURCE, "example"),
            DocSection::new("Choice Card", choice_card)
                .description("Choice-card radios combine FieldContent with rich labels.")
                .code_rust_from_file_region(snippets::choice_card::SOURCE, "example"),
            DocSection::new("Field Group", field_group)
                .description("FieldGroup provides separators and checkbox-group composition.")
                .code_rust_from_file_region(snippets::field_group::SOURCE, "example"),
            DocSection::new("Responsive Layout", responsive)
                .description(
                    "Responsive orientation collapses label/content layouts for narrow containers.",
                )
                .code_rust_from_file_region(snippets::responsive::SOURCE, "example"),
            DocSection::new("Validation and Errors", validation_and_errors)
                .description("Field invalid state + control `aria_invalid` styling.")
                .code_rust_from_file_region(snippets::validation_and_errors::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("All Field compositions should render correctly under RTL direction.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and stability guidance.")
                .test_id_prefix("ui-gallery-field-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-field")]
}
