use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::scroll_area as snippets;

pub(super) fn preview_scroll_area(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let drag_baseline = snippets::drag_baseline::render(cx);
    let expand_at_bottom = snippets::expand_at_bottom::render(cx);
    let horizontal = snippets::horizontal::render(cx);
    let nested_scroll_routing = snippets::nested_scroll_routing::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/scroll_area.rs` (ScrollArea, ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaCorner).",
        "First-party docs now teach `scroll_area(...)` as the default compact helper, while `ScrollArea::new(...)` and the Radix-shaped composable parts remain explicit advanced seams.",
        "ScrollArea is for custom scrollbars + consistent styling; use native scrolling when you don't need custom chrome.",
        "Keep scroll region sizes explicit in docs to avoid layout drift.",
        "Horizontal rails are easiest to reason about when the child has a fixed width.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).description("Usage notes and caveats.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Vertical scroll region with tags and separators.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `scroll_area(...)` with explicit viewport size.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let horizontal = DocSection::build(cx, "Horizontal", horizontal)
        .description("Horizontal rail (fixed-size items) inside a scroll area.")
        .code_rust_from_file_region(snippets::horizontal::SOURCE, "example");
    let nested_scroll_routing =
        DocSection::build(cx, "Nested scroll routing", nested_scroll_routing)
            .description("Inner horizontal scroll area should not consume vertical wheel.")
            .code_rust_from_file_region(snippets::nested_scroll_routing::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("ScrollArea behavior under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let drag_baseline = DocSection::new("Scrollbar drag baseline", drag_baseline)
        .description(
            "Diagnostics harness (Fret-only): drag the thumb, then arm content growth. The thumb should not jump when extents change mid-drag.",
        )
        .code_rust_from_file_region(snippets::drag_baseline::SOURCE, "example");
    let expand_at_bottom = DocSection::new("Expand at bottom", expand_at_bottom)
        .description(
            "Diagnostics harness: content expands while already at the scroll extent edge (pinned extents regression).",
        )
        .code_rust_from_file_region(snippets::expand_at_bottom::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn ScrollArea docs flow: Demo -> Usage -> Horizontal, keeps nested routing / RTL as app-facing follow-ups, then places Fret-only diagnostics last.",
        ),
        vec![
            demo,
            usage,
            horizontal,
            nested_scroll_routing,
            rtl,
            drag_baseline,
            expand_at_bottom,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-scroll-area")]
}
