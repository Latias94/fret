use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tabs as snippets;

pub(super) fn preview_tabs(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let rtl = snippets::rtl::render(cx);
    let list = snippets::list::render(cx);
    let disabled = snippets::disabled::render(cx);
    let icons = snippets::icons::render(cx);
    let line = snippets::line::render(cx);
    let vertical = snippets::vertical::render(cx);
    let vertical_line = snippets::vertical_line::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes_block([
        "Preview follows upstream shadcn Tabs docs order first: Demo, Usage, Line, Vertical, Disabled, Icons, RTL; Fret-specific sections follow afterwards.",
        "`tabs_uncontrolled(cx, default, |cx| ..)` and `tabs(cx, model, |cx| ..)` are the default first-party teaching lane; the composable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` surface remains available for explicit slot-level assembly.",
        "Keep root width caller-owned in examples such as Usage (`w-[400px]` upstream), while list/trigger chrome remains recipe-owned.",
        "Password fields use `Input::password()` to mirror `type=\"password\"` in shadcn/ui examples.",
        "API reference: `ecosystem/fret-ui-shadcn/src/tabs.rs`.",
    ]);
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
        Some("A set of layered sections of content that are displayed one at a time."),
        vec![
            demo,
            usage,
            line,
            vertical,
            disabled,
            icons,
            rtl,
            list,
            vertical_line,
            extras,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-tabs").into_element(cx)]
}
