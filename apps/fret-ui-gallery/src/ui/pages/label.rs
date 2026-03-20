use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::label as snippets;

pub(super) fn preview_label(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label_in_field = snippets::label_in_field::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Label::new(text)` remains the copyable docs-path surface; the lead demo now mirrors the official checkbox preview, while `for_control(...)` covers the documented association path.",
        "`Label` remains a lightweight text primitive; form structure, helper text, and error presentation belong to `Field`, `FieldLabel`, and related field parts.",
        "`Label::for_control(...)` plus a control-side `control_id(...)` is the Fret bridge for the upstream `htmlFor` / `id` pairing and keeps click-to-focus behavior out of page code.",
        "We intentionally do not widen `Label` into a generic compound-children API here: the current shadcn/base/radix docs path only needs text children plus association, while richer clickable label subtrees already have a recipe-owned home in `FieldLabel::wrap(...)`.",
        "Mechanism fixes in this pass stay narrow: disabled associated-label opacity now matches the upstream `opacity-50` outcome, and plain associated labels no longer let ambient pressable shells suppress the documented click-to-toggle path.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official checkbox + label preview from the docs page.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `Label` plus control association.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let label_in_field = DocSection::build(cx, "Label in Field", label_in_field)
        .description("Prefer `Field` plus `FieldLabel` for form layouts with helper text.")
        .code_rust_from_file_region(snippets::label_in_field::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Label and input alignment under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Label docs path first: checkbox Demo, Usage, Label in Field, RTL, and API Reference.",
        ),
        vec![demo, usage, label_in_field, rtl, api_reference],
    );

    vec![body.test_id("ui-gallery-label").into_element(cx)]
}
