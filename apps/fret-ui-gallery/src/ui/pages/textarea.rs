use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::textarea as snippets;

pub(super) fn preview_textarea(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let disabled = snippets::disabled::render(cx);
    let with_label = snippets::field::render(cx);
    let with_text = snippets::with_text::render(cx);
    let invalid = snippets::invalid::render(cx);
    let button = snippets::button::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/textarea.rs` (Textarea).",
            "Textarea is a leaf text control, so the main parity gap here is usage clarity rather than missing composition APIs.",
            "Placeholder text is rendered when the model is empty.",
            "Drag the bottom-right corner to resize the textarea height.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Textarea docs flow: Demo -> Usage -> With Label -> Disabled -> With Text -> With Button. Invalid and RTL remain Fret-specific audit variants.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-textarea-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Textarea`.")
                .test_id_prefix("ui-gallery-textarea-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("With Label", with_label)
                .test_id_prefix("ui-gallery-textarea-field")
                .code_rust_from_file_region(snippets::field::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .test_id_prefix("ui-gallery-textarea-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("With Text", with_text)
                .test_id_prefix("ui-gallery-textarea-with-text")
                .code_rust_from_file_region(snippets::with_text::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-textarea-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("With Button", button)
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
