use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::pagination as snippets;

pub(super) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let simple = snippets::simple::render(cx);
    let icons_only = snippets::icons_only::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/pagination.rs`.",
            "Gallery sections mirror shadcn Pagination docs first: Demo, Usage, Simple, Icons Only, RTL.",
            "Pagination already exposes the upstream-shaped parts surface (`Pagination`, `PaginationContent`, `PaginationItem`, `PaginationLink`, `PaginationPrevious`, `PaginationNext`, `PaginationEllipsis`), so the main parity gap here was usage clarity rather than missing mechanism or a generic compose builder.",
            "Fret keeps navigation wiring in the app layer: `PaginationLink` exposes command/action hooks instead of a DOM-specific `href`, while preserving link semantics and active-page state.",
            "The Next.js and changelog sections in upstream docs map to app-layer routing guidance plus the existing `text(...)` parity on `PaginationPrevious` / `PaginationNext`, so they stay documented here as notes rather than separate demos.",
            "The root approximates upstream `<nav aria-label=\"pagination\">` with `Region + label`, and the content/items emit `List` / `ListItem` semantics to mirror the upstream `ul/li` structure.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Pagination docs order directly, with app-layer routing notes captured in the notes section.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("shadcn demo: Previous, numbered links, ellipsis, and Next.")
                .test_id_prefix("ui-gallery-pagination-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for the upstream-shaped composable parts API.")
                .test_id_prefix("ui-gallery-pagination-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Simple", simple)
                .description("A simple pagination with only page numbers.")
                .test_id_prefix("ui-gallery-pagination-simple")
                .code_rust_from_file_region(snippets::simple::SOURCE, "example"),
            DocSection::new("Icons Only", icons_only)
                .description("Use just the previous and next buttons without page numbers.")
                .test_id_prefix("ui-gallery-pagination-icons-only")
                .code_rust_from_file_region(snippets::icons_only::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("RTL smoke check for icon direction and localized numerals.")
                .test_id_prefix("ui-gallery-pagination-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API surface and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-pagination")]
}
