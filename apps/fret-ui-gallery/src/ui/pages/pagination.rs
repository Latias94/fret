use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::pagination as snippets;

pub(super) fn preview_pagination(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let simple = snippets::simple::render(cx);
    let icons_only = snippets::icons_only::render(cx);
    let routing = snippets::routing::render(cx);
    let rtl = snippets::rtl::render(cx);
    let custom_text = snippets::custom_text::render(cx);
    let compact_builder = snippets::compact_builder::render(cx);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/pagination.rs`. Reference stack: shadcn Pagination docs and the default registry recipe.",
        "No direct `Pagination` primitive exists in Radix Primitives or Base UI; the closest secondary references are the shadcn base/radix registry copies, and they confirm the same nav/list/link recipe shape rather than a missing mechanism layer.",
        "`Usage` now teaches the upstream-shaped parts lane directly: `Pagination`, `PaginationContent`, `PaginationItem`, and `PaginationLink` already support explicit composable children without needing an extra generic `compose()` API.",
        "`Routing / Next.js` is the Fret app-layer equivalent of the upstream Next.js section: keep link semantics on `PaginationLink`, but bind navigation via `.action(...)` instead of a DOM-only `href` swap.",
        "`Custom Text / Changelog` maps the upstream changelog update onto the existing `.text(...)` support on `PaginationPrevious` / `PaginationNext`.",
        "`Compact Builder` keeps the Fret shorthand lane explicit for common app call sites: `pagination(...)`, `pagination_content(...)`, `pagination_item(...)`, and `pagination_link(...)` reduce child landing noise without replacing the upstream-shaped parts surface.",
        "The remaining semantic gap is still core-level naming rather than pagination recipe structure: root semantics approximate `<nav aria-label=\"pagination\">` with `Region + label`, and content/items emit `List` / `ListItem` semantics until a dedicated navigation landmark role exists in core.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .title_test_id("ui-gallery-pagination-section-title-notes")
        .description("Parity notes, owner-layer classification, and teaching-surface guidance.");
    let demo = DocSection::build(cx, "Demo", demo)
        .title_test_id("ui-gallery-pagination-section-title-demo")
        .description("shadcn demo: Previous, numbered links, ellipsis, and Next.")
        .test_id_prefix("ui-gallery-pagination-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-pagination-section-title-usage")
        .description("Copyable upstream-shaped parts composition for Pagination.")
        .test_id_prefix("ui-gallery-pagination-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let simple = DocSection::build(cx, "Simple", simple)
        .title_test_id("ui-gallery-pagination-section-title-simple")
        .description("A simple pagination with only page numbers.")
        .test_id_prefix("ui-gallery-pagination-simple")
        .code_rust_from_file_region(snippets::simple::SOURCE, "example");
    let icons_only = DocSection::build(cx, "Icons Only", icons_only)
        .title_test_id("ui-gallery-pagination-section-title-icons-only")
        .description("Use just the previous and next buttons without page numbers.")
        .test_id_prefix("ui-gallery-pagination-icons-only")
        .code_rust_from_file_region(snippets::icons_only::SOURCE, "example");
    // Upstream docs-order anchor: `DocSection::build(cx, "Routing / Next.js", routing)`.
    let routing = DocSection::build(cx, "Routing", routing)
        .title_test_id("ui-gallery-pagination-section-title-routing")
        .description(
            "App-layer equivalent of upstream Next.js `Link`: preserve link semantics, bind routing with `.action(...)`.",
        )
        .test_id_prefix("ui-gallery-pagination-routing")
        .code_rust_from_file_region(snippets::routing::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .title_test_id("ui-gallery-pagination-section-title-rtl")
        .description("RTL docs example with localized numerals and mirrored previous/next icons.")
        .test_id_prefix("ui-gallery-pagination-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    // Upstream docs-order anchor: `DocSection::build(cx, "Custom Text / Changelog", custom_text)`.
    let custom_text = DocSection::build(cx, "Custom Text", custom_text)
        .title_test_id("ui-gallery-pagination-section-title-custom-text")
        .description("Use `text(...)` on previous/next, mirroring the upstream changelog update.")
        .test_id_prefix("ui-gallery-pagination-custom-text")
        .code_rust_from_file_region(snippets::custom_text::SOURCE, "example");
    let compact_builder = DocSection::build(cx, "Compact Builder", compact_builder)
        .title_test_id("ui-gallery-pagination-section-title-compact-builder")
        .description(
            "Fret ergonomic shorthand for the same family when you do not need to spell each part explicitly.",
        )
        .test_id_prefix("ui-gallery-pagination-compact-builder")
        .code_rust_from_file_region(snippets::compact_builder::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Pagination docs path after `Installation`: `Demo`, `Usage`, `Simple`, `Icons Only`, `Routing / Next.js`, `RTL`, and `Custom Text / Changelog`; `Compact Builder` stays as the explicit Fret follow-up.",
        ),
        vec![
            demo,
            usage,
            simple,
            icons_only,
            routing,
            rtl,
            custom_text,
            compact_builder,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-pagination").into_element(cx)]
}
