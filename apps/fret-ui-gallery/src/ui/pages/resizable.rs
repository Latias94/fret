use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::resizable as snippets;

pub(super) fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx, h_fractions, v_fractions);
    let usage = snippets::usage::render(cx);
    let handle = snippets::handle::render(cx);
    let vertical = snippets::vertical::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = snippets::notes::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Resizable docs flow: Demo -> Usage, with handle, vertical, and RTL examples kept as gallery coverage."),
        vec![
            DocSection::new("Demo", demo)
                .description("Nested vertical panels inside a horizontal group.")
                .test_id_prefix("ui-gallery-resizable-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `ResizablePanelGroup`, `ResizablePanel`, and `ResizableHandle`.")
                .test_id_prefix("ui-gallery-resizable-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Handle", handle)
                .description("A handle with a visual grabber (`withHandle`).")
                .test_id_prefix("ui-gallery-resizable-handle")
                .code_rust_from_file_region(snippets::handle::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation.")
                .test_id_prefix("ui-gallery-resizable-vertical")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Direction provider coverage for hit-testing and handle affordances.")
                .test_id_prefix("ui-gallery-resizable-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Parity notes and references.")
                .test_id_prefix("ui-gallery-resizable-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-resizable")]
}
