use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::hover_card as snippets;

pub(super) fn preview_hover_card(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let trigger_delays = snippets::trigger_delays::render(cx);
    let positioning = snippets::positioning::render(cx);
    let basic = snippets::basic::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`HoverCard::open_delay(...)` and `HoverCard::close_delay(...)` keep timing ownership on the root, matching the Radix/shadcn docs surface. Base UI's trigger-owned delay props remain a reference, not the public Fret API.",
        "`HoverCard::side(...)` / `align(...)` set placement defaults, and `HoverCardContent` also exposes `side(...)`, `align(...)`, `side_offset(...)`, and `align_offset(...)` for explicit geometry tuning.",
        "`HoverCard::new(cx, trigger, content)` remains the recipe-level entry point; `HoverCardTrigger::build(...)` and `HoverCardContent::build(cx, ...)` cover the typed compound-parts lane without adding a separate root compose API.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/hover_card.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx` and Radix Hover Card docs.",
        "Preview keeps the docs-aligned `@nextjs` demo first, then follows the Radix Hover Card topics (`Usage`, `Trigger Delays`, `Positioning`, `Sides`, `RTL`). The extra `Basic` section is a minimal Fret supplement.",
        "Hover card already exposes shadcn-style part names (`HoverCardTrigger`, `HoverCardContent`) plus typed builders for the copyable parts lane.",
        "Delay examples intentionally follow the Radix/shadcn root-owned timing model; Base UI's trigger-owned delay props are only a mechanism reference for cross-checking behavior.",
        "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
        "Sides and positioning are separated to make placement parity checks deterministic.",
        "RTL sample is included because side resolution can differ in right-to-left layouts.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-hover-card-api-reference")
        .description("Public surface summary and placement/timing ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("Implementation notes and regression guidelines.")
        .test_id_prefix("ui-gallery-hover-card-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Upstream shadcn demo composition: link trigger + 320px content (`w-80`) with avatar and text.",
        )
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description(
            "Copyable shadcn-style composition reference using typed trigger/content parts.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let trigger_delays = DocSection::build(cx, "Trigger Delays", trigger_delays)
        .description(
            "Use root-level `open_delay(...)` / `close_delay(...)` controls to tune hover-intent timing.",
        )
        .code_rust_from_file_region(snippets::trigger_delays::SOURCE, "example");
    let positioning = DocSection::build(cx, "Positioning", positioning)
        .description("Placement is controlled by `side` and `align` on `HoverCardContent`.")
        .code_rust_from_file_region(snippets::positioning::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Minimal supplemental recipe beneath the full docs-aligned demo.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let sides = DocSection::build(cx, "Sides", sides)
        .description("Visual sweep of side placements.")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Hover card should respect right-to-left direction context.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview leads with the docs-aligned `@nextjs` demo, then follows the Radix Hover Card topics; `Basic` remains a small Fret supplement.",
        ),
        vec![
            demo,
            usage,
            trigger_delays,
            positioning,
            basic,
            sides,
            rtl,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-hover-card").into_element(cx)]
}
