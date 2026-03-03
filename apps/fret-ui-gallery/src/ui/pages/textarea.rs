use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::textarea as snippets;

pub(super) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    _value: Model<String>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let field = snippets::field::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let button = snippets::button::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn/ui v4 Textarea docs (radix/base nova).",
            "Placeholder text is rendered when the model is empty.",
            "Drag the bottom-right corner to resize the textarea height.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn/ui Textarea docs examples: Demo, Field, Disabled, Invalid, Button, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-textarea-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Field", field)
                .test_id_prefix("ui-gallery-textarea-field")
                .code_rust_from_file_region(snippets::field::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .test_id_prefix("ui-gallery-textarea-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-textarea-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("Button", button)
                .test_id_prefix("ui-gallery-textarea-button")
                .code_rust_from_file_region(snippets::button::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-textarea-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-textarea")]
}
