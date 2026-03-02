use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::label as snippets;

pub(super) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let label_in_field = snippets::label_in_field::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/label.rs` (Label) and `ecosystem/fret-ui-shadcn/src/field.rs` (FieldLabel).",
            "Label is a lightweight text primitive; form semantics and helper/error text live in `Field`.",
            "Current Label API does not expose `htmlFor` binding; accessibility is handled by control a11y labels and Field composition.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Label docs order: Demo, Label in Field, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic label above an input.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Label in Field", label_in_field)
                .description("Prefer Field + FieldLabel for form layouts.")
                .code_rust_from_file_region(snippets::label_in_field::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Label and input alignment under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-label")]
}
