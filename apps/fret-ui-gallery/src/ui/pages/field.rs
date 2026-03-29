use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::field as snippets;

pub(super) fn preview_field(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let anatomy = snippets::anatomy::render(cx);
    let input = snippets::input::render(cx);
    let textarea = snippets::textarea::render(cx);
    let select = snippets::select::render(cx);
    let slider = snippets::slider::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let switch = snippets::switch::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let composable_label = snippets::composable_label::render(cx);
    let field_group = snippets::field_group::render(cx);
    let rtl = snippets::rtl::render(cx);
    let responsive = snippets::responsive::render(cx);
    let validation_and_errors = snippets::validation_and_errors::render(cx);

    let usage_notes = doc_layout::muted_full_width(
        cx,
        "Import the Field parts and compose them as needed (examples below mirror the upstream docs).",
    );
    let anatomy_notes = doc_layout::muted_full_width(
        cx,
        "A typical Field groups a label + control + optional helper/error text.",
    );
    let form = doc_layout::notes_block([
        "Use `Field` as the low-level label / control / description shell.",
        "Use the `Form` recipe when you need validation adapters or form-library integration; that policy should not be pushed down into `Field` defaults.",
        "This mirrors the upstream docs split between reusable field primitives and higher-level form guides.",
    ]);
    let accessibility = doc_layout::notes_block([
        "Use `field_set(...)` + `FieldLegend` to group related controls for assistive technologies.",
        "Associate labels via `FieldLabel::for_control(...)` plus matching control ids; when parts need a shared field-local association context, use `Field::build(...)`.",
        "Use `FieldError` immediately after the control or inside `FieldContent`, and pair invalid styling with control-level `aria_invalid(true)`.",
        "Use `FieldSeparator` sparingly so grouped sections remain understandable to screen readers.",
    ]);
    let api_reference = doc_layout::notes_block([
        "`Field::new([...])` is the core wrapper for a single field; `orientation(...)` covers the documented `vertical`, `horizontal`, and `responsive` layouts.",
        "`Field::build(...)` is the composable children lane when parts should share field-local association state without manually threading ids.",
        "`field_set(...)` and `field_group(...)` are the default first-party grouped authoring entrypoints; `FieldSet` / `FieldGroup` remain the underlying typed recipe surface when direct builder access is useful.",
        "`FieldLegend` and `FieldSeparator` cover semantic grouping labels and section separation.",
        "`FieldContent`, `FieldLabel`, `FieldTitle`, `FieldDescription`, and `FieldError` cover the default typed slot path; when a richer wrapper is needed, keep using `FieldLabel::wrap(...)` and the typed `Field::build(...)` / `field_group(...)` builders instead of dropping to raw `AnyElement` seams too early.",
        "Width ownership stays deliberate: `FieldDescription` keeps recipe-owned full-width wrapping, while plain `FieldLabel` / `FieldTitle` keep intrinsic-width defaults unless the surrounding `Field` orientation or call site requests full width.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/field.rs` (Field, FieldSet, FieldGroup, FieldLabel, FieldDescription, FieldSeparator).",
        "Field page now mirrors the upstream docs path first, then adds one explicit Fret teaching seam: Composable Children via `FieldLabel::wrap(...)`.",
        "The Select example now uses `Field::build(...)` so Fret can preserve the upstream label + control + description order without forcing explicit ids into the snippet.",
        "`Field::build(...)` now also supports `Input` / `Textarea` auto association, but this page keeps explicit-id text-field examples so the default teaching surface stays closer to the upstream docs.",
        "Choice Card keeps the recipe shorthand (`RadioGroupItemVariant::ChoiceCard`) on purpose, while the next composable-children section shows the explicit wrapper lane.",
        "The responsive section keeps one gallery-only width toggle so the docs-aligned responsive layout can be exercised across the container breakpoint.",
        "Each section keeps a stable `test_id` so diag scripts can target specific examples.",
        "The current audit points to docs/public-surface drift rather than a `fret-ui` mechanism bug: the upstream layout semantics are already covered by the existing field web-parity tests.",
        "`FieldTitle` and plain `FieldLabel` keep upstream-like intrinsic width defaults; full-width behavior belongs to `Field` orientation rules, `RadioGroupItemVariant::ChoiceCard`, or wrapped card-style labels via `FieldLabel::wrap(...)`.",
    ]);
    let form = DocSection::build(cx, "Form", form)
        .no_shell()
        .description(
            "Keep low-level field layout separate from higher-level form adapters and validation policy.",
        );
    let accessibility = DocSection::build(cx, "Accessibility", accessibility)
        .no_shell()
        .description("Keyboard, labeling, grouping, and invalid-state guidance.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-field-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("API reference pointers and stability guidance.")
        .test_id_prefix("ui-gallery-field-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Overview composition aligned with the upstream `field-demo` preview.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = ui::v_stack(|cx| vec![usage.into_element(cx), usage_notes.into_element(cx)])
        .gap(Space::N3)
        .items_start();
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal imports plus a representative fieldset composition.")
        .test_id_prefix("ui-gallery-field-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let anatomy = ui::v_stack(|cx| vec![anatomy.into_element(cx), anatomy_notes.into_element(cx)])
        .gap(Space::N3)
        .items_start();
    let anatomy = DocSection::build(cx, "Anatomy", anatomy)
        .description("Aligns with the upstream shadcn Field anatomy section.")
        .test_id_prefix("ui-gallery-field-anatomy")
        .code_rust_from_file_region(snippets::anatomy::SOURCE, "example");
    let input = DocSection::build(cx, "Input", input)
        .description("Basic text inputs with labels + helper copy.")
        .code_rust_from_file_region(snippets::input::SOURCE, "example");
    let textarea = DocSection::build(cx, "Textarea", textarea)
        .description("Textarea field with explicit height and helper copy.")
        .code_rust_from_file_region(snippets::textarea::SOURCE, "example");
    let select = DocSection::build(cx, "Select", select)
        .description("Select composed inside a Field shell via `Field::build(...)`, matching the upstream authoring order without explicit ids.")
        .code_rust_from_file_region(snippets::select::SOURCE, "example");
    let slider = DocSection::build(cx, "Slider", slider)
        .description("Non-text controls should still use FieldTitle/Description for context.")
        .code_rust_from_file_region(snippets::slider::SOURCE, "example");
    let fieldset = DocSection::build(cx, "Fieldset", fieldset)
        .description("FieldSet groups multiple fields with a legend + description.")
        .code_rust_from_file_region(snippets::fieldset::SOURCE, "example");
    let checkbox = DocSection::build(cx, "Checkbox", checkbox)
        .description("Horizontal Field orientation keeps checkbox + label aligned.")
        .code_rust_from_file_region(snippets::checkbox::SOURCE, "example");
    let radio = DocSection::build(cx, "Radio", radio)
        .description(
            "RadioGroup keeps its compact recipe defaults while `RadioGroupItem::children(...)` and per-item `control_id(...)` hooks let callers mirror upstream-style separate field labels.",
        )
        .code_rust_from_file_region(snippets::radio::SOURCE, "example");
    let switch = DocSection::build(cx, "Switch", switch)
        .description("Minimal horizontal switch field aligned with the upstream docs example.")
        .code_rust_from_file_region(snippets::switch::SOURCE, "example");
    let choice_card = DocSection::build(cx, "Choice Card", choice_card)
        .description(
            "Choice-card radios keep whole-card activation on the recipe shorthand; the next section shows the explicit composable-children lane.",
        )
        .code_rust_from_file_region(snippets::choice_card::SOURCE, "example");
    let composable_label = DocSection::build(cx, "Composable Children", composable_label)
        .description(
            "Use `FieldLabel::wrap(...)` when a richer subtree should behave like one clickable label, mirroring the upstream wrapped-label pattern.",
        )
        .code_rust_from_file_region(snippets::composable_label::SOURCE, "example");
    let field_group = DocSection::build(cx, "Field Group", field_group)
        .description("FieldGroup provides separators and checkbox-group composition.")
        .code_rust_from_file_region(snippets::field_group::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Mirrors the fuller upstream payment-form preview under RTL direction.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let responsive = DocSection::build(cx, "Responsive Layout", responsive)
        .description(
            "Responsive orientation keeps the upstream layout shape while the gallery-only width toggle exercises the container breakpoint.",
        )
        .code_rust_from_file_region(snippets::responsive::SOURCE, "example");
    let validation_and_errors =
        DocSection::build(cx, "Validation and Errors", validation_and_errors)
            .description("Field invalid state + control `aria_invalid` styling.")
            .code_rust_from_file_region(snippets::validation_and_errors::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Field docs order first, then adds one Fret-specific authoring seam: Demo, Usage, Anatomy, Form, Input, Textarea, Select, Slider, Fieldset, Checkbox, Radio, Switch, Choice Card, Composable Children, Field Group, RTL, Responsive Layout, Validation and Errors, Accessibility, and API Reference.",
        ),
        vec![
            demo,
            usage,
            anatomy,
            form,
            input,
            textarea,
            select,
            slider,
            fieldset,
            checkbox,
            radio,
            switch,
            choice_card,
            composable_label,
            field_group,
            rtl,
            responsive,
            validation_and_errors,
            accessibility,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-field").into_element(cx)]
}
