use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input as snippets;

pub(super) fn preview_input(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let basic = snippets::basic::render(cx);
    let field = snippets::field::render(cx);
    let field_group = snippets::field_group::render(cx);
    let label = snippets::label::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let file = snippets::file::render(cx);
    let inline = snippets::inline::render(cx);
    let grid = snippets::grid::render(cx);
    let required = snippets::required::render(cx);
    let badge = snippets::badge::render(cx);
    let input_group = snippets::input_group::render(cx);
    let button_group = snippets::button_group::render(cx);
    let form = snippets::form::render(cx);
    let rtl = snippets::rtl::render(cx);

    let usage = doc_layout::muted_full_width(
        cx,
        "Bind `Input` to a model, then compose it with `Field` parts when you need labels, descriptions, or validation copy.",
    );

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Input::new(model)` is the Fret equivalent of the upstream `<Input />`; `placeholder(...)`, `disabled(...)`, and `aria_invalid(...)` cover the documented core surface.",
            "`Input` root width/height defaults remain recipe-owned (`w-full min-w-0` plus the control height) because the upstream recipe defines those constraints on the component itself.",
            "`control_id(...)` plus `FieldLabel::for_control(...)` is the Fret path for label association; it stays a focused follow-up example instead of widening the base Input API.",
            "Native file inputs are authored as `Input` + `Browse` button composition; Fret does not mirror DOM `type=\"file\"` directly, and diagnostics runs mock the picker to keep scripted gates deterministic.",
            "Required markers remain label/call-site composition; the recipe owns input chrome, not surrounding form-policy affordances.",
            "Keep `ui-gallery-input-basic` stable for IME routing regression scripts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Input docs path first, then keeps `Label Association` and `API Reference` as explicit Fret follow-ups.",
        ),
        vec![
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Input` before the example matrix.")
                .code_rust(
                    r#"use fret_ui_shadcn::{facade as shadcn, prelude::*};

let value = cx.app.models_mut().insert(String::new());

shadcn::Input::new(value)
    .a11y_label("Search")
    .placeholder("Type to search...")
    .into_element(cx);"#,
                ),
            DocSection::new("Basic", basic)
                .description("Single input field (used by IME routing regression scripts).")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Field", field)
                .description("Field composition with label, description, and error slots.")
                .code_rust_from_file_region(snippets::field::SOURCE, "example"),
            DocSection::new("Field Group", field_group)
                .description("FieldGroup stacks related fields and action rows.")
                .code_rust_from_file_region(snippets::field_group::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled inputs should block focus/interaction and use muted styling.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Invalid state uses `aria_invalid` plus field-level error copy.")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("File", file)
                .description("Native file picking uses a file dialog; diagnostics runs mock it.")
                .code_rust_from_file_region(snippets::file::SOURCE, "example"),
            DocSection::new("Inline", inline)
                .description("Horizontal Field orientation is useful for compact toolbars.")
                .code_rust_from_file_region(snippets::inline::SOURCE, "example"),
            DocSection::new("Grid", grid)
                .description("Two-column input layout with shared row alignment.")
                .code_rust_from_file_region(snippets::grid::SOURCE, "example"),
            DocSection::new("Required", required)
                .description("Required affordance is represented by label composition in this gallery.")
                .code_rust_from_file_region(snippets::required::SOURCE, "example"),
            DocSection::new("Badge", badge)
                .description("Use Badge inside a label row.")
                .code_rust_from_file_region(snippets::badge::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .description("Inline addons and trailing buttons via InputGroup composition.")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("Button Group", button_group)
                .description("ButtonGroup composes an input and a button with shared chrome.")
                .code_rust_from_file_region(snippets::button_group::SOURCE, "example"),
            DocSection::new("Form", form)
                .description("Multi-field form layout using FieldGroup plus responsive rows.")
                .code_rust_from_file_region(snippets::form::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Input plus Field composition under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description(
                    "Use `FieldLabel::for_control` plus `Input::control_id` so label clicks focus the input and preserve labelled-by semantics.",
                )
                .test_id_prefix("ui-gallery-input-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-input-api-reference")
                .description("Public surface summary, ownership notes, and current caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-input")]
}
