use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::field as snippets;

pub(super) fn preview_field(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
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
    let form = doc_layout::notes(
        cx,
        [
            "Use `Field` as the low-level label / control / description shell.",
            "Use the `Form` recipe when you need validation adapters or form-library integration; that policy should not be pushed down into `Field` defaults.",
            "This mirrors the upstream docs split between reusable field primitives and higher-level form guides.",
        ],
    );
    let accessibility = doc_layout::notes(
        cx,
        [
            "Use `FieldSet` + `FieldLegend` to group related controls for assistive technologies.",
            "Associate labels via `FieldLabel::for_control(...)` plus matching control ids (or wrap rich choice-card content with `FieldLabel::wrap(...)`).",
            "Use `FieldError` immediately after the control or inside `FieldContent`, and pair invalid styling with control-level `aria_invalid(true)`.",
            "Use `FieldSeparator` sparingly so grouped sections remain understandable to screen readers.",
        ],
    );
    let api_reference = doc_layout::notes(
        cx,
        [
            "`Field::new([...])` is the core wrapper for a single field; `orientation(...)` covers the documented `vertical`, `horizontal`, and `responsive` layouts.",
            "`FieldSet`, `FieldLegend`, `FieldGroup`, and `FieldSeparator` cover semantic grouping and section separation.",
            "`FieldContent`, `FieldLabel`, `FieldTitle`, `FieldDescription`, and `FieldError` cover the documented content slots without needing an extra generic children / compose API.",
            "Width ownership stays deliberate: `FieldDescription` keeps recipe-owned full-width wrapping, while plain `FieldLabel` / `FieldTitle` keep intrinsic-width defaults unless the surrounding `Field` orientation or call site requests full width.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/field.rs` (Field, FieldSet, FieldGroup, FieldLabel, FieldDescription, FieldSeparator).",
            "Field page now mirrors the upstream docs path first: Usage, Anatomy, Form, the example set through Field Group, RTL, Responsive Layout, Validation and Errors, Accessibility, and API Reference.",
            "Each section keeps a stable `test_id` so diag scripts can target specific examples.",
            "No mechanism bug is indicated here; the current work is docs/public-surface parity and source-of-truth cleanup toward the base docs/examples.",
            "`FieldTitle` and plain `FieldLabel` keep upstream-like intrinsic width defaults; full-width behavior belongs to `Field` orientation rules or wrapped card-style labels.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Field docs order first: Usage, Anatomy, Form, Input, Textarea, Select, Slider, Fieldset, Checkbox, Radio, Switch, Choice Card, Field Group, RTL, Responsive Layout, Validation and Errors, Accessibility, and API Reference.",
        ),
        vec![
            DocSection::new("Usage", usage)
                .description("Copyable minimal imports plus a representative fieldset composition.")
                .code_rust(
                    r#"use fret_ui_shadcn::{facade as shadcn, prelude::*};

let full_name = cx.local_model_keyed("full_name", String::new);
let newsletter = cx.local_model_keyed("newsletter", || false);

shadcn::FieldSet::new([
    shadcn::FieldLegend::new("Profile").into_element(cx),
    shadcn::FieldDescription::new("This appears on invoices and emails.").into_element(cx),
    shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Full name")
                .for_control("profile-name")
                .into_element(cx),
            shadcn::Input::new(full_name)
                .control_id("profile-name")
                .placeholder("Evil Rabbit")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("Subscribe to the newsletter")
                    .for_control("newsletter")
                    .into_element(cx),
                shadcn::FieldDescription::new("Receive product updates by email.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(newsletter)
                .control_id("newsletter")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .into_element(cx),
])
.into_element(cx);"#,
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
            DocSection::new("Form", form)
                .no_shell()
                .description("Keep low-level field layout separate from higher-level form adapters and validation policy."),
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
            DocSection::new("RTL", rtl)
                .description("All Field compositions should render correctly under RTL direction.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Responsive Layout", responsive)
                .description(
                    "Responsive orientation collapses label/content layouts for narrow containers.",
                )
                .code_rust_from_file_region(snippets::responsive::SOURCE, "example"),
            DocSection::new("Validation and Errors", validation_and_errors)
                .description("Field invalid state + control `aria_invalid` styling.")
                .code_rust_from_file_region(snippets::validation_and_errors::SOURCE, "example"),
            DocSection::new("Accessibility", accessibility)
                .no_shell()
                .description("Keyboard, labeling, grouping, and invalid-state guidance."),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Public surface summary and ownership notes."),
            DocSection::new("Notes", notes)
                .no_shell()
                .description("API reference pointers and stability guidance.")
                .test_id_prefix("ui-gallery-field-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-field")]
}
