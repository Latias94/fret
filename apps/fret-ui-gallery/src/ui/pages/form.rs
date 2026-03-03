use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::form as snippets;

pub(super) fn preview_forms(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let upstream_demo = snippets::upstream_demo::render(cx);
    let demo = snippets::demo::render(cx);
    let input = snippets::input::render(cx);
    let textarea = snippets::textarea::render(cx);
    let controls = snippets::controls::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = snippets::notes::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Start with an upstream-aligned FormDemo, then keep a set of gallery recipes for composing Input/Textarea/Checkbox/Switch/FieldSet.",
        ),
        vec![
            DocSection::new("Form Demo", upstream_demo)
                .description("Aligned with shadcn/ui `form-demo.tsx` (new-york-v4).")
                .code_rust_from_file_region(snippets::upstream_demo::SOURCE, "example"),
            DocSection::new("Demo", demo)
                .description("FieldSet + FieldGroup recipe with multiple controls.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Input", input)
                .description("A model-bound input control.")
                .code_rust_from_file_region(snippets::input::SOURCE, "example"),
            DocSection::new("Textarea", textarea)
                .description("A model-bound textarea control with fixed height.")
                .code_rust_from_file_region(snippets::textarea::SOURCE, "example"),
            DocSection::new("Checkbox + Switch", controls)
                .description("Basic checkbox + switch controls with labels.")
                .code_rust_from_file_region(snippets::controls::SOURCE, "example"),
            DocSection::new("Fieldset", fieldset)
                .description("FieldSet recipe with grouped fields and action row.")
                .code_rust_from_file_region(snippets::fieldset::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Form composition under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .code_rust_from_file_region(snippets::notes::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-form")]
}
