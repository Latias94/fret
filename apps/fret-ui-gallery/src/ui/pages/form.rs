use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::form as snippets;

pub(super) fn preview_forms(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let upstream_demo = snippets::upstream_demo::render(cx);
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let input = snippets::input::render(cx);
    let textarea = snippets::textarea::render(cx);
    let controls = snippets::controls::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = snippets::notes::render(cx);
    let notes = DocSection::build(cx, "Notes", notes)
        .description("API reference pointers and authoring notes.")
        .code_rust_from_file_region(snippets::notes::SOURCE, "example");
    let upstream_demo = DocSection::build(cx, "Form Demo", upstream_demo)
        .description("Aligned with shadcn/ui `form-demo.tsx` (new-york-v4).")
        .code_rust_from_file_region(snippets::upstream_demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description(
            "Copyable minimal usage for the framework-agnostic `Form` + `FormField` surface, including field-level `required` ownership on `FormField::required(true)`, keeping field-level required semantics on `FormField`, and wrapper-owned invalid decoration from `FormState`.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("FieldSet + FieldGroup recipe with multiple controls.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let input = DocSection::build(cx, "Input", input)
        .description("A model-bound input control.")
        .code_rust_from_file_region(snippets::input::SOURCE, "example");
    let textarea = DocSection::build(cx, "Textarea", textarea)
        .description("A model-bound textarea control with fixed height.")
        .code_rust_from_file_region(snippets::textarea::SOURCE, "example");
    let controls = DocSection::build(cx, "Checkbox + Switch", controls)
        .description("Basic checkbox + switch controls with labels.")
        .code_rust_from_file_region(snippets::controls::SOURCE, "example");
    let fieldset = DocSection::build(cx, "Fieldset", fieldset)
        .description("FieldSet recipe with grouped fields and action row.")
        .code_rust_from_file_region(snippets::fieldset::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description(
            "Focused Fret RTL follow-up: logical end-aligned field text plus explicit horizontal row composition.",
        )
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Start with an upstream-aligned FormDemo, then provide a copyable Usage section and gallery recipes for composing Input/Textarea/Checkbox/Switch/FieldSet while keeping field-level required semantics and `FormState`-driven invalid decoration on `FormField`.",
        ),
        vec![
            upstream_demo,
            usage,
            demo,
            input,
            textarea,
            controls,
            fieldset,
            rtl,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-form").into_element(cx)]
}
