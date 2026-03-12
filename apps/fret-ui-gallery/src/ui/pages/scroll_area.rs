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

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/scroll_area.rs` (ScrollArea, ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaCorner).",
            "ScrollArea already exposes both a compact builder and a Radix-shaped composable surface, so the main parity gap here is usage clarity rather than missing authoring APIs.",
            "ScrollArea is for custom scrollbars + consistent styling; use native scrolling when you don't need custom chrome.",
            "Keep scroll region sizes explicit in docs to avoid layout drift.",
            "Horizontal rails are easiest to reason about when the child has a fixed width.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn ScrollArea docs flow: Demo -> Usage -> Horizontal. Fret-only diagnostics follow after the upstream-shaped examples."),
        vec![
            DocSection::new("Demo", demo)
                .description("Vertical scroll region with tags and separators.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `ScrollArea` with explicit viewport size.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Scrollbar drag baseline", drag_baseline)
                .description(
                    "Diagnostics harness (Fret-only): drag the thumb, then arm content growth. The thumb should not jump when extents change mid-drag.",
                )
                .code_rust_from_file_region(snippets::drag_baseline::SOURCE, "example"),
            DocSection::new("Expand at bottom", expand_at_bottom)
                .description(
                    "Diagnostics harness: content expands while already at the scroll extent edge (pinned extents regression).",
                )
                .code_rust_from_file_region(snippets::expand_at_bottom::SOURCE, "example"),
            DocSection::new("Horizontal", horizontal)
                .description("Horizontal rail (fixed-size items) inside a scroll area.")
                .code_rust_from_file_region(snippets::horizontal::SOURCE, "example"),
            DocSection::new("Nested scroll routing", nested_scroll_routing)
                .description("Inner horizontal scroll area should not consume vertical wheel.")
                .code_rust_from_file_region(snippets::nested_scroll_routing::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("ScrollArea behavior under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Usage notes and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-scroll-area")]
}
