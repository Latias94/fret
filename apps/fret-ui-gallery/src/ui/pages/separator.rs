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
        "`Separator::new()`, `Separator::orientation(...)`, and `Separator::decorative(...)` cover the public surface that matters for shadcn/Radix/Base UI parity.",
        "shadcn-style separators stay decorative by default; opt into `.decorative(false)` only when the divider should participate in the accessibility tree as a real separator.",
        "Vertical separators now translate the upstream `data-vertical:self-stretch` policy directly on the shadcn recipe surface, so callers own surrounding row height but should not need to restate self-stretch in normal docs-path usage.",
        "Separator remains a leaf primitive: surrounding width/height negotiation stays caller-owned, while the 1px rule chrome and vertical self-stretch default stay recipe-owned.",
        "No extra generic `children` / `compose()` / `asChild` surface is needed here because upstream composition happens around the separator rather than through the separator itself.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");

    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Official top-of-page preview from the docs: horizontal copy plus a vertical nav lane.",
        )
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for a decorative separator; use `.decorative(false)` only when the divider should be announced.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Use `orientation=vertical`; the shadcn surface already carries the upstream self-stretch default.")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let menu = DocSection::build(cx, "Menu", menu)
        .description("Vertical separators between menu-like items with descriptions.")
        .code_rust_from_file_region(snippets::menu::SOURCE, "example");
    let list = DocSection::build(cx, "List", list)
        .description("Horizontal separators between list items.")
        .code_rust_from_file_region(snippets::list::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Separator layout should hold under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Separator docs path first: Demo, Usage, Vertical, Menu, List, RTL, and API Reference.",
        ),
        vec![demo, usage, vertical, menu, list, rtl, api_reference],
    );

    vec![body.test_id("ui-gallery-separator").into_element(cx)]
}
