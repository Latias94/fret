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
    let children = snippets::children::render(cx);

    let api_reference = doc_layout::notes_block([
        "`HoverCard::open_delay(...)` and `HoverCard::close_delay(...)` keep timing ownership on the root, matching the Radix/shadcn docs surface. Base UI's trigger-owned `delay` / `closeDelay` props remain a mechanism reference, not the public Fret API.",
        "`HoverCard::side(...)` / `align(...)` set placement defaults, and `HoverCardContent` also exposes `side(...)`, `align(...)`, `side_offset(...)`, and `align_offset(...)` for explicit geometry tuning.",
        "`HoverCard::new(cx, trigger, content)` remains the recipe-level entry point and already covers the upstream nested `<HoverCard><HoverCardTrigger /><HoverCardContent /></HoverCard>` composition plus the custom-trigger / `asChild` story, because `trigger` can be any landed or late-landed element.",
        "`HoverCardTrigger::build(...)`, `HoverCardContent::new([...])`, and `HoverCardContent::build(cx, ...)` cover the composable slot lanes without adding a separate heterogeneous root `children([...])` API.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/hover_card.rs`. Reference stack: shadcn Hover Card docs, the default registry recipe, Radix Primitives hover-card semantics, and Base UI preview-card ownership.",
        "Preview now mirrors the shadcn Hover Card docs path directly: `Demo`, `Usage`, `Trigger Delays`, `Positioning`, `Basic`, `Sides`, `RTL`, and `API Reference`. `Children (Fret)` and `Notes` stay as the explicit follow-ups.",
        "Hover card already exposes shadcn-style part names (`HoverCardTrigger`, `HoverCardContent`) plus typed builders for the copyable parts lane; the content slot already has a composable children surface, so a generic root `children([...])` API would mostly duplicate `HoverCard::new(...)`.",
        "Hover-card behavior itself is already covered by the existing Radix/web geometry, chrome, and UI Gallery interaction gates; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "Delay examples intentionally follow the Radix/shadcn root-owned timing model; Base UI's trigger-owned delay props are only a mechanism cross-check for behavior and do not define the Fret recipe surface.",
        "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
        "`Basic` and `Sides` stay on the docs path because upstream documents them as examples, not as Fret-only supplements.",
        "`Children (Fret)` is the focused follow-up for caller-owned content-slot composition via `HoverCardContent::new([...])`; it does not imply a new root `children([...])` API.",
        "Sides and positioning are separated to keep placement parity checks deterministic.",
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
        .test_id_prefix("ui-gallery-hover-card-demo-section")
        .description(
            "Upstream shadcn demo composition: link trigger + 320px content (`w-80`) with avatar and text.",
        )
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .test_id_prefix("ui-gallery-hover-card-usage-section")
        .description(
            "Copyable shadcn-style composition reference using typed trigger/content parts.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let trigger_delays = DocSection::build(cx, "Trigger Delays", trigger_delays)
        .test_id_prefix("ui-gallery-hover-card-trigger-delays-section")
        .description(
            "Use root-level `open_delay(...)` / `close_delay(...)` controls to tune hover-intent timing.",
        )
        .code_rust_from_file_region(snippets::trigger_delays::SOURCE, "example");
    let positioning = DocSection::build(cx, "Positioning", positioning)
        .test_id_prefix("ui-gallery-hover-card-positioning-section")
        .description("Placement is controlled by `side` and `align` on `HoverCardContent`.")
        .code_rust_from_file_region(snippets::positioning::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .test_id_prefix("ui-gallery-hover-card-basic-section")
        .description("Minimal hover card example from the upstream docs examples set.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let sides = DocSection::build(cx, "Sides", sides)
        .test_id_prefix("ui-gallery-hover-card-sides-section")
        .description("Visual sweep of side placements.")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-hover-card-rtl-section")
        .description("Hover card should respect right-to-left direction context.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let children = DocSection::build(cx, "Children (Fret)", children)
        .description(
            "Use `HoverCardContent::new([...])` when the panel body is already built or caller-owned.",
        )
        .test_id_prefix("ui-gallery-hover-card-children")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview leads with the docs-aligned `@nextjs` demo, then follows the shadcn Hover Card docs path through `API Reference`; `Children (Fret)` and `Notes` stay as the explicit Fret/source-alignment follow-ups.",
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
            children,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-hover-card").into_element(cx)]
}
