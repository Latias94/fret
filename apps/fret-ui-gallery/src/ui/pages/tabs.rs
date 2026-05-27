use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tabs as snippets;

const TABS_PAGE_INTRO: &str = "Preview mirrors the current shadcn Tabs docs path first: `Demo` and `Usage`. `Line (Base/Radix)`, `Vertical (Base/Radix)`, `Disabled (Base/Radix)`, `Icons (Base/Radix)`, and `List (Base/Radix)` follow Base/Radix registry examples; `RTL (Fret)`, `API Reference (Fret)`, `Composable Parts (Fret)`, `Vertical Line (Fret)`, `Extras (Fret)`, and `Notes` stay as focused follow-ups.";

pub(super) fn preview_tabs(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let line = snippets::line::render(cx);
    let vertical = snippets::vertical::render(cx);
    let disabled = snippets::disabled::render(cx);
    let icons = snippets::icons::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);
    let list = snippets::list::render(cx);
    let vertical_line = snippets::vertical_line::render(cx);
    let extras = snippets::extras::render(cx);

    let api_reference = doc_layout::notes_block([
        "Reference stack: current shadcn Tabs docs and new-york-v4 source, with Base/Radix registry examples as secondary references.",
        "Current docs path stays `Demo` and `Usage`; richer line, vertical, disabled, icon, and basic-list examples stay labeled as Base/Radix registry follow-ups instead of being treated as current shadcn docs-path sections.",
        "API reference: `ecosystem/fret-ui-shadcn/src/tabs.rs`.",
        "`tabs_uncontrolled(cx, default, |cx| ..)` and `tabs(cx, model, |cx| ..)` remain the default copyable root lane for common app code.",
        "`TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` already provide the composable compound-parts lane, so Tabs does not need a second root `children([...])` API just to match upstream nested authoring.",
        "`TabsTrigger::children(...)` and `TabsItem::trigger_children(...)` cover caller-owned trigger content when the compact label/icon helpers are too narrow.",
        "Demo shell (`w-full max-w-sm`) and usage width (`w-[400px]`) stay caller-owned, while list/trigger/content chrome and `TabsContent` fill-width defaults stay recipe-owned.",
    ]);
    let notes = doc_layout::notes_block([
        "No `fret-ui` runtime change was needed for Tabs semantics/layout; the only mechanism-adjacent parity fix stayed in `fret-ui-shadcn` and makes Base UI-style `activation_direction` metadata follow physical movement, so the physical Right Arrow in RTL maps to the logical previous tab instead of the logical next tab.",
        "The docs surface keeps the upstream width split explicit: `Demo` mirrors the `w-full max-w-sm` shell, while `Usage` keeps the `w-[400px]` call-site width from the docs block.",
        "`Line (Base/Radix)`, `Vertical (Base/Radix)`, and `Disabled (Base/Radix)` keep the same text/value shape as Base/Radix registry examples, while `Icons (Base/Radix)` demonstrates icon + label trigger composition through `TabsItem::trigger_children(...)` without leaving the default builder lane.",
        "Password fields use `Input::password()` to mirror `type=\"password\"` in shadcn/ui examples.",
        "The `RTL (Fret)` section now uses a fuller registry-style four-tab card example instead of a gallery-only two-tab keynav gate, making the mirrored content shell easier to compare while keeping RTL as a Fret follow-up.",
        "List-only, vertical-line, and flex-1 examples stay after the docs path because they are regression/follow-up surfaces rather than upstream section headings.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference (Fret)", api_reference)
        .description("Public surface summary and owner split.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-tabs-api-reference");
    let notes = DocSection::build(cx, "Notes", notes).description("Parity notes and references.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Account/password card example with inputs and footer actions.")
        .test_id_prefix("ui-gallery-tabs-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable builder-preserving usage for the common tabs authoring path.")
        .test_id_prefix("ui-gallery-tabs-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let line = DocSection::build(cx, "Line (Base/Radix)", line)
        .description("Base/Radix registry line-style list with transparent background.")
        .test_id_prefix("ui-gallery-tabs-line")
        .code_rust_from_file_region(snippets::line::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical (Base/Radix)", vertical)
        .description("Base/Radix registry vertical orientation follow-up.")
        .test_id_prefix("ui-gallery-tabs-vertical")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled (Base/Radix)", disabled)
        .description("Base/Radix registry disabled trigger follow-up.")
        .test_id_prefix("ui-gallery-tabs-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let icons = DocSection::build(cx, "Icons (Base/Radix)", icons)
        .description("Base/Radix registry icon + label trigger composition while staying on the builder lane.")
        .test_id_prefix("ui-gallery-tabs-icons")
        .code_rust_from_file_region(snippets::icons::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL (Fret)", rtl)
        .description(
            "Fret RTL parity for logical previous/next movement, flipped `activation_direction` metadata, and the fuller registry-style card example.",
        )
        .test_id_prefix("ui-gallery-tabs-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Composable Parts (Fret)", parts)
        .description(
            "Copyable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` lane for explicit slot ownership and custom trigger children.",
        )
        .test_id_prefix("ui-gallery-tabs-parts")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");
    let list = DocSection::build(cx, "List (Base/Radix)", list)
        .description(
            "Base/Radix Basic registry example rendered as a list without mounted content.",
        )
        .test_id_prefix("ui-gallery-tabs-list")
        .code_rust_from_file_region(snippets::list::SOURCE, "example");
    let vertical_line = DocSection::build(cx, "Vertical Line (Fret)", vertical_line)
        .description("Fret follow-up combining vertical orientation with the line variant.")
        .test_id_prefix("ui-gallery-tabs-vertical-line")
        .code_rust_from_file_region(snippets::vertical_line::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras (Fret)", extras)
        .description("Fret-specific regression gates (flex-1 triggers).")
        .test_id_prefix("ui-gallery-tabs-extras")
        .code_rust_from_file_region(snippets::extras::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(TABS_PAGE_INTRO),
        vec![
            demo,
            usage,
            line,
            vertical,
            disabled,
            icons,
            rtl,
            api_reference,
            parts,
            list,
            vertical_line,
            extras,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-tabs").into_element(cx)]
}
