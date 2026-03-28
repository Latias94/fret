use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::label as snippets;

pub(super) fn preview_label(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let children = snippets::children::render(cx);
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let label_in_field = snippets::label_in_field::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Label::new(text)` remains the copyable docs-path surface; the lead demo now mirrors the official checkbox preview, while `for_control(...)` covers the documented association path.",
        "`Label::children(...)` is the Fret-specific inline composition lane for shadcn's generic child slot: it prepends inline content while preserving the label text as the accessible name and control-facing association label.",
        "`Label` remains a lightweight text primitive; form structure, helper text, and error presentation belong to `Field`, `FieldLabel`, and related field parts.",
        "`Label::for_control(...)` plus a control-side `control_id(...)` is the Fret bridge for the upstream `htmlFor` / `id` pairing and keeps click-to-focus behavior out of page code.",
        "`Label::wrap(...)` stays available when a whole inline subtree should own the visible content, but richer form-specific structure still belongs on `FieldLabel::wrap(...)`.",
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
    let children = DocSection::build(cx, "Composable Content", children)
        .description("Fret-only inline children lane for labels that still own text and control association.")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Label docs path first, then adds one Fret-only composable-content appendix for the inline children lane.",
        ),
        vec![demo, usage, label_in_field, rtl, children, api_reference],
    );

    vec![body.test_id("ui-gallery-label").into_element(cx)]
}
