use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::hover_card as snippets;

pub(super) fn preview_hover_card(
    cx: &mut UiCx<'_>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx, avatar_image);
    let usage = snippets::usage::render(cx);
    let trigger_delays = snippets::trigger_delays::render(cx);
    let positioning = snippets::positioning::render(cx);
    let basic = snippets::basic::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`HoverCard::open_delay(...)` and `HoverCard::close_delay(...)` match the upstream timing controls described in the docs.",
            "`HoverCard::side(...)` / `align(...)` set placement defaults, and `HoverCardContent` also exposes `side_offset(...)` and `align_offset(...)` for explicit geometry tuning.",
            "`HoverCard::new(trigger, content)` remains the recipe-level entry point; no extra compose layer is required for the documented usage surface.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/hover_card.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx` and Radix Hover Card docs.",
            "Preview mirrors the shadcn Hover Card docs path after `Installation`: `Demo`, `Usage`, `Trigger Delays`, `Positioning`, `Basic`, `Sides`, `RTL`, and `API Reference`.",
            "Hover card already exposes shadcn-style part names (`HoverCardTrigger`, `HoverCardContent`), and `HoverCard::new(trigger, content)` is the recipe-level composition entry point.",
            "Gallery sections mirror the upstream docs order directly: `Trigger Delays` and `Positioning` are upstream API sections, not Fret-only extras.",
            "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
            "Sides and positioning are separated to make placement parity checks deterministic.",
            "RTL sample is included because side resolution can differ in right-to-left layouts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Hover Card docs path after `Installation`: `Demo`, `Usage`, `Trigger Delays`, `Positioning`, `Basic`, `Sides`, `RTL`, and `API Reference`.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Upstream shadcn demo composition: link trigger + 320px content (`w-80`) with avatar and text.",
                )
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Hover Card.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Trigger Delays", trigger_delays)
                .description("Use trigger `delay` / `closeDelay` style controls to tune open and close timing.")
                .code_rust_from_file_region(snippets::trigger_delays::SOURCE, "example"),
            DocSection::new("Positioning", positioning)
                .description("Placement is controlled by `side` and `align` on `HoverCardContent`.")
                .code_rust_from_file_region(snippets::positioning::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Minimal hover card usage surface.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Sides", sides)
                .description("Visual sweep of side placements.")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Hover card should respect right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-hover-card-api-reference")
                .description("Public surface summary and placement/timing ownership notes."),
            DocSection::new("Notes", notes)
                .no_shell()
                .description("Implementation notes and regression guidelines.")
                .test_id_prefix("ui-gallery-hover-card-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-hover-card")]
}
