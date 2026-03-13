use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::separator as snippets;

pub(super) fn preview_separator(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let vertical = snippets::vertical::render(cx);
    let menu = snippets::menu::render(cx);
    let list = snippets::list::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Separator::new()` and `Separator::orientation(...)` cover the public surface used by the upstream docs.",
        "Separator remains a leaf primitive: surrounding width/height negotiation stays caller-owned, while the 1px rule chrome stays recipe-owned.",
        "Vertical separators often want `.flex_stretch_cross_axis(true)` inside fixed-height flex rows to match the upstream `self-stretch` outcome.",
        "No extra generic `compose()` / `asChild` surface is needed here because upstream composition happens around the separator rather than through the separator itself.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Separator docs path first: Demo, Usage, Vertical, Menu, List, RTL, and API Reference.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Separator`.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Use `orientation=vertical` for vertical separators.")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("Menu", menu)
                .description("Vertical separators between menu-like items with descriptions.")
                .code_rust_from_file_region(snippets::menu::SOURCE, "example"),
            DocSection::new("List", list)
                .description("Horizontal separators between list items.")
                .code_rust_from_file_region(snippets::list::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Separator layout should hold under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            api_reference,
        ],
    );

    vec![body.test_id("ui-gallery-separator")]
}
