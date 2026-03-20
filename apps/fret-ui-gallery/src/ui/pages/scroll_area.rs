use super::super::*;
use fret::UiCx;

use crate::ui::diagnostics::scroll_area as diagnostics;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::scroll_area as snippets;

pub(super) fn preview_scroll_area(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let horizontal = snippets::horizontal::render(cx);
    let rtl = snippets::rtl::render(cx);
    let compact_helper = snippets::compact_helper::render(cx);
    let nested_scroll_routing = snippets::nested_scroll_routing::render(cx);
    let drag_baseline = diagnostics::drag_baseline::render(cx);
    let expand_at_bottom = diagnostics::expand_at_bottom::render(cx);

    let api_reference = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/radix/scroll-area.mdx`; Base UI reference: `repo-ref/ui/apps/v4/content/docs/components/base/scroll-area.mdx`; registry chrome reference: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/scroll-area.tsx`.",
        "`ScrollArea::new([...])` is now the default copyable wrapper lane for the docs surface. `scroll_area(cx, |cx| [...])` remains the compact Fret-first shorthand instead of the primary teaching surface.",
        "`ScrollAreaRoot::new(ScrollAreaViewport::new([...])).scrollbar(ScrollBar::new().orientation(...))` already covers the shadcn/Radix mixed `ScrollArea` + `ScrollBar` examples without widening this family into an untyped arbitrary-children API.",
        "Base UI's extra `Content` / `Thumb` parts are useful headless references, but Fret keeps the viewport content wrapper and thumb as runtime-owned implementation details; the public shadcn lane does not need separate promoted parts for them today.",
        "No mechanism or default-style regression was identified in this pass. The remaining drift was the first-party docs/teaching surface.",
    ]);
    let notes = doc_layout::notes_block([
        "Preview now mirrors the upstream shadcn/Base UI docs path first: `Demo`, `Usage`, `Horizontal`, `RTL`, and `API Reference`.",
        "The `Horizontal` snippet intentionally uses the explicit parts lane so the copyable code tab still teaches the `ScrollBar` vocabulary that appears in the upstream docs examples.",
        "`Compact helper` keeps the Fret-only `scroll_area(...)` shorthand discoverable without displacing the parity lane.",
        "ScrollArea is for custom scrollbars + consistent styling; use native scrolling when you do not need custom chrome.",
        "Keep `ui-gallery-scroll-area-*` and `ui-gallery-page-scroll-area` test IDs stable; existing diagnostics scripts depend on them.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-scroll-area-api-reference")
        .description(
            "Public surface summary, ownership notes, and the current children API conclusion.",
        );
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-scroll-area-notes")
        .description("Parity notes, follow-up guidance, and diagnostics ownership.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Vertical tag list aligned with the upstream `scroll-area-demo.tsx` example.")
        .test_id_prefix("ui-gallery-scroll-area-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description(
            "Copyable minimal usage for the default `ScrollArea::new([...])` wrapper lane.",
        )
        .test_id_prefix("ui-gallery-scroll-area-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let horizontal = DocSection::build(cx, "Horizontal", horizontal)
        .description("Horizontal rail using the explicit `ScrollBar` parts lane that maps cleanly to the upstream docs example.")
        .test_id_prefix("ui-gallery-scroll-area-horizontal")
        .code_rust_from_file_region(snippets::horizontal::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("ScrollArea behavior under an RTL direction provider.")
        .test_id_prefix("ui-gallery-scroll-area-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let compact_helper = DocSection::build(cx, "Compact helper", compact_helper)
        .description("Fret-only shorthand for the common case when you do not need explicit scrollbar parts.")
        .test_id_prefix("ui-gallery-scroll-area-compact-helper")
        .code_rust_from_file_region(snippets::compact_helper::SOURCE, "example");
    let nested_scroll_routing =
        DocSection::build(cx, "Nested scroll routing", nested_scroll_routing)
            .description("Inner horizontal scroll area should not consume vertical wheel.")
            .test_id_prefix("ui-gallery-scroll-area-nested-scroll-routing")
            .code_rust_from_file_region(snippets::nested_scroll_routing::SOURCE, "example");
    let drag_baseline = DocSection::build_diagnostics(cx, "Scrollbar drag baseline", drag_baseline)
        .description(
            "Diagnostics harness (Fret-only): drag the thumb, then arm content growth. The thumb should not jump when extents change mid-drag.",
        )
        .code_rust_from_file_region(diagnostics::drag_baseline::SOURCE, "example");
    let expand_at_bottom = DocSection::build_diagnostics(cx, "Expand at bottom", expand_at_bottom)
        .description(
            "Diagnostics harness: content expands while already at the scroll extent edge (pinned extents regression).",
        )
        .code_rust_from_file_region(diagnostics::expand_at_bottom::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the upstream shadcn/Base UI Scroll Area docs path first, then keeps the compact helper, nested routing, and diagnostics harnesses as explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            horizontal,
            rtl,
            api_reference,
            compact_helper,
            nested_scroll_routing,
            drag_baseline,
            expand_at_bottom,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-scroll-area").into_element(cx)]
}
