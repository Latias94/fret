use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tabs as snippets;

const TABS_PAGE_INTRO: &str = "Preview mirrors the shadcn Tabs docs path after `Installation`: `Demo`, `Usage`, `Line`, `Vertical`, `Disabled`, `Icons`, `RTL`, and `API Reference`; `Composable Parts (Fret)`, `List`, `Vertical (Line)`, `Extras`, and `Notes` stay as focused follow-ups.";

pub(super) fn preview_tabs(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
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
        "API reference: `ecosystem/fret-ui-shadcn/src/tabs.rs`.",
        "`tabs_uncontrolled(cx, default, |cx| ..)` and `tabs(cx, model, |cx| ..)` remain the default copyable root lane for common app code.",
        "`TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` already provide the composable compound-parts lane, so Tabs does not need a second root `children([...])` API just to match upstream nested authoring.",
        "`TabsTrigger::children(...)` and `TabsItem::trigger_children(...)` cover caller-owned trigger content when the compact label/icon helpers are too narrow.",
        "Root width stays caller-owned (`w-[400px]` upstream), while list/trigger/content chrome and `TabsContent` fill-width defaults stay recipe-owned.",
    ]);
    let notes = doc_layout::notes_block([
        "This review did not indicate a missing `fret-ui` mechanism-layer fix: existing semantics/layout gates already cover selection, roving, trigger foreground inheritance, and content fill.",
        "The remaining drift was on the docs/public-surface side: the gallery was not surfacing the explicit `API Reference` section or a copyable compound-parts example even though the underlying surface already existed.",
        "Password fields use `Input::password()` to mirror `type=\"password\"` in shadcn/ui examples.",
        "List-only, vertical-line, and flex-1 examples stay after the docs path because they are regression/follow-up surfaces rather than upstream section headings.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
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
    let line = DocSection::build(cx, "Line", line)
        .description("Line-style list with transparent background.")
        .test_id_prefix("ui-gallery-tabs-line")
        .code_rust_from_file_region(snippets::line::SOURCE, "example");
    let vertical = DocSection::build(cx, "Vertical", vertical)
        .description("Vertical orientation (Radix parity).")
        .test_id_prefix("ui-gallery-tabs-vertical")
        .code_rust_from_file_region(snippets::vertical::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disable individual triggers.")
        .test_id_prefix("ui-gallery-tabs-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let icons = DocSection::build(cx, "Icons", icons)
        .description("Compose icons into triggers.")
        .test_id_prefix("ui-gallery-tabs-icons")
        .code_rust_from_file_region(snippets::icons::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL key navigation and indicator placement parity.")
        .test_id_prefix("ui-gallery-tabs-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Composable Parts (Fret)", parts)
        .description(
            "Copyable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` lane for explicit slot ownership and custom trigger children.",
        )
        .test_id_prefix("ui-gallery-tabs-parts")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");
    let list = DocSection::build(cx, "List", list)
        .description("Tabs list without any mounted content.")
        .test_id_prefix("ui-gallery-tabs-list")
        .code_rust_from_file_region(snippets::list::SOURCE, "example");
    let vertical_line = DocSection::build(cx, "Vertical (Line)", vertical_line)
        .description("Vertical + line style.")
        .test_id_prefix("ui-gallery-tabs-vertical-line")
        .code_rust_from_file_region(snippets::vertical_line::SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
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
