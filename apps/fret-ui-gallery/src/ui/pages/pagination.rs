use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::pagination as snippets;

pub(super) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/pagination.rs`.",
            "Pagination already exposes the upstream-shaped parts surface (`Pagination`, `PaginationContent`, `PaginationItem`, `PaginationLink`, `PaginationPrevious`, `PaginationNext`, `PaginationEllipsis`), so the main parity gap here was usage clarity rather than missing mechanism or a generic compose builder.",
            "Fret keeps navigation wiring in the app layer: `PaginationLink` exposes command/action hooks instead of a DOM-specific `href`, while preserving link semantics and active-page state.",
            "RTL examples validate icon direction + number shaping under RTL.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Pagination docs flow: Demo -> Usage. Extras keep Fret-specific recipe examples and RTL regression coverage."),
        vec![
            DocSection::new("Demo", demo)
                .description("shadcn demo: Previous, numbered links, ellipsis, and Next.")
                .test_id_prefix("ui-gallery-pagination-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for the upstream-shaped composable parts API.")
                .test_id_prefix("ui-gallery-pagination-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("RTL smoke check for icon direction and localized numerals.")
                .test_id_prefix("ui-gallery-pagination-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .description("Fret-specific recipes such as simple pagination and icons-only/table-toolbar layouts.")
                .test_id_prefix("ui-gallery-pagination-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API surface and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-pagination")]
}
