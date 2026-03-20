use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input as snippets;

pub(super) fn preview_input(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
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

    let api_reference = doc_layout::notes_block([
        "`Input::new(model)` is the Fret equivalent of the upstream `<Input />`; `placeholder(...)`, `disabled(...)`, `aria_invalid(...)`, `aria_required(...)`, and `password()` cover the current documented core surface.",
        "`Input` root width/height defaults remain recipe-owned (`w-full min-w-0` plus the control height) because the upstream recipe defines those constraints on the component itself.",
        "`control_id(...)` plus `FieldLabel::for_control(...)` is the Fret path for label association; it stays a focused follow-up example instead of widening the base Input API.",
        "`Input` stays a leaf control; labels/descriptions/errors belong in `Field`, and inline adornments belong in `InputGroup` / `ButtonGroup`, so no generic `children(...)` / `asChild` surface is needed here.",
        "The self-drawn input surface does not yet expose generic browser `type=` hints (`email/search/url/tel/file`) as a public API; `password()` is the direct parity path today, and the file example remains an explicit composition translation.",
        "Native file inputs are authored as `Input` + `Browse` button composition; Fret does not mirror DOM `type=\"file\"` directly, and diagnostics runs mock the picker to keep scripted gates deterministic.",
        "Required markers remain label/call-site composition; the recipe owns input chrome, not surrounding form-policy affordances.",
        "Keep `ui-gallery-input-basic` stable for IME routing regression scripts.",
    ]);
    let notes = doc_layout::notes_block([
        "This page follows the upstream Input docs path first: Demo, Usage, Basic, Field, Field Group, Disabled, Invalid, File, Inline, Grid, Required, Badge, Input Group, Button Group, Form, RTL.",
        "Use plain `Input` when you only need an editable value; compose with `Field` parts or `InputGroup` once labels, descriptions, validation copy, or inline addons become part of the authoring surface.",
        "Copyable gallery snippets standardize on `use fret_ui_shadcn::{facade as shadcn, prelude::*};` for the curated shadcn lane.",
        "Most regressions here come from IME routing, label/control wiring, width negotiation, or docs-page drift, so keep `ui-gallery-input-*` ids stable when extending the page or adding diag scripts.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-input-api-reference")
        .description("Public surface summary, ownership notes, and current caveats.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-input-notes")
        .description("Usage guidance and parity notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-input-demo")
        .description("Default input preview matching the upstream top-of-page demo.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .test_id_prefix("ui-gallery-input-usage")
        .description("Copyable minimal usage for `Input` before the example matrix.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .test_id_prefix("ui-gallery-input-basic")
        .description("Single input field (used by IME routing regression scripts).")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let field = DocSection::build(cx, "Field", field)
        .test_id_prefix("ui-gallery-input-field")
        .description("Field composition with label, description, and error slots.")
        .code_rust_from_file_region(snippets::field::SOURCE, "example");
    let field_group = DocSection::build(cx, "Field Group", field_group)
        .test_id_prefix("ui-gallery-input-field-group")
        .description("FieldGroup stacks related fields and action rows.")
        .code_rust_from_file_region(snippets::field_group::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .test_id_prefix("ui-gallery-input-disabled")
        .description("Disabled inputs should block focus/interaction and use muted styling.")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .test_id_prefix("ui-gallery-input-invalid")
        .description("Invalid state uses `aria_invalid` plus field-level invalid styling.")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let file = DocSection::build(cx, "File", file)
        .test_id_prefix("ui-gallery-input-file-section")
        .description("Fret translates the upstream file-input example into deterministic input + browse composition.")
        .code_rust_from_file_region(snippets::file::SOURCE, "example");
    let inline = DocSection::build(cx, "Inline", inline)
        .test_id_prefix("ui-gallery-input-inline")
        .description("Horizontal Field orientation is useful for compact toolbars.")
        .code_rust_from_file_region(snippets::inline::SOURCE, "example");
    let grid = DocSection::build(cx, "Grid", grid)
        .test_id_prefix("ui-gallery-input-grid")
        .description("Two-column input layout with shared row alignment.")
        .code_rust_from_file_region(snippets::grid::SOURCE, "example");
    let required = DocSection::build(cx, "Required", required)
        .test_id_prefix("ui-gallery-input-required")
        .description("Required affordance is represented by label composition in this gallery.")
        .code_rust_from_file_region(snippets::required::SOURCE, "example");
    let badge = DocSection::build(cx, "Badge", badge)
        .test_id_prefix("ui-gallery-input-badge")
        .description("Use Badge inside a label row.")
        .code_rust_from_file_region(snippets::badge::SOURCE, "example");
    let input_group = DocSection::build(cx, "Input Group", input_group)
        .test_id_prefix("ui-gallery-input-input-group")
        .description("Inline addons and trailing buttons via InputGroup composition.")
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let button_group = DocSection::build(cx, "Button Group", button_group)
        .test_id_prefix("ui-gallery-input-button-group")
        .description("ButtonGroup composes an input and a text button with shared chrome.")
        .code_rust_from_file_region(snippets::button_group::SOURCE, "example");
    let form = DocSection::build(cx, "Form", form)
        .test_id_prefix("ui-gallery-input-form")
        .description("Multi-field form layout using FieldGroup plus responsive rows.")
        .code_rust_from_file_region(snippets::form::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-input-rtl")
        .description("Password input plus Field composition under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description(
            "Use `FieldLabel::for_control` plus `Input::control_id` so label clicks focus the input and preserve labelled-by semantics.",
        )
        .test_id_prefix("ui-gallery-input-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Input docs path first: Demo, Usage, Basic, Field, Field Group, Disabled, Invalid, File, Inline, Grid, Required, Badge, Input Group, Button Group, Form, RTL. `Label Association` and `API Reference` stay as explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            basic,
            field,
            field_group,
            disabled,
            invalid,
            file,
            inline,
            grid,
            required,
            badge,
            input_group,
            button_group,
            form,
            rtl,
            label,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-input").into_element(cx)]
}
