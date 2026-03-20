use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::resizable as snippets;
use fret::UiCx;

pub(super) fn preview_resizable(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let vertical = snippets::vertical::render(cx);
    let handle = snippets::handle::render(cx);
    let rtl = snippets::rtl::render(cx);
    let notes = snippets::notes::render(cx);
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Parity notes and references.")
        .test_id_prefix("ui-gallery-resizable-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Nested vertical panels inside a horizontal group.")
        .test_id_prefix("ui-gallery-resizable-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description(
            "Copyable minimal usage for `resizable_panel_group(...)`, `ResizablePanel`, and `ResizableHandle`.",
        )
        .test_id_prefix("ui-gallery-resizable-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Vertical orientation.")
        .test_id_prefix("ui-gallery-resizable-vertical")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let handle = DocSection::build(cx, "Handle", handle)
        .description("A handle with a visual grabber (`withHandle`).")
        .test_id_prefix("ui-gallery-resizable-handle")
        .code_rust_from_file_region(snippets::handle::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider coverage for hit-testing and handle affordances.")
        .test_id_prefix("ui-gallery-resizable-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Resizable docs flow: Demo, Usage, Vertical, Handle, and RTL. Notes capture the Fret-specific parity conclusions.",
        ),
        vec![demo, usage, vertical, handle, rtl, notes],
    );

    let component = body.test_id("ui-gallery-resizable").into_element(cx);
    let page = ui::v_flex(move |_cx| vec![component])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start();

    vec![page.into_element(cx)]
}
