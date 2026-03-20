use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::kbd as snippets;

pub(super) fn preview_kbd(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let group = snippets::group::render(cx);
    let button = snippets::button::render(cx);
    let tooltip = snippets::tooltip::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Kbd::new(text)` is the default docs-aligned lane for a single key token such as `Ctrl`, `Esc`, or `⌘`.",
        "`KbdGroup::new([...])` groups adjacent keycaps or separators for shortcut chords while keeping spacing consistent.",
        "`Kbd::from_children([...])` / `.children([...])` remain explicit escape hatches for icon-only or mixed-content caps, so no broader generic `asChild` / `compose()` surface is warranted here.",
        "Fixed height, padding, radius, muted chrome, and tooltip-slot color inversion remain recipe-owned.",
        "Composition into buttons, tooltips, and input-group addons stays caller-owned, matching the upstream docs layering.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .test_id_prefix("ui-gallery-kbd-api-reference")
        .no_shell()
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-kbd-demo")
        .description("Two shortcut display patterns: modifier-key glyphs and a textual chord.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .test_id_prefix("ui-gallery-kbd-usage")
        .description("Copyable minimal usage for a single `Kbd` token.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let group = DocSection::build(cx, "Group", group)
        .test_id_prefix("ui-gallery-kbd-group")
        .description("Use `KbdGroup` to keep spacing consistent across tokens.")
        .code_rust_from_file_region(snippets::group::SOURCE, "example");
    let button = DocSection::build(cx, "Button", button)
        .test_id_prefix("ui-gallery-kbd-button")
        .description("kbd tokens can be composed into button labels for discoverability.")
        .code_rust_from_file_region(snippets::button::SOURCE, "example");
    let tooltip = DocSection::build(cx, "Tooltip", tooltip)
        .test_id_prefix("ui-gallery-kbd-tooltip")
        .description("Tooltips often include shortcut hints for expert users.")
        .code_rust_from_file_region(snippets::tooltip::SOURCE, "example");
    let input_group = DocSection::build(cx, "Input Group", input_group)
        .test_id_prefix("ui-gallery-kbd-input-group")
        .description("Trailing kbd hints can be rendered inside an input group.")
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-kbd-rtl")
        .description("kbd token order should respect right-to-left direction context.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Kbd docs path first: Demo, Usage, Group, Button, Tooltip, Input Group, RTL, and API Reference.",
        ),
        vec![demo, usage, group, button, tooltip, input_group, rtl, api_reference],
    )
    .test_id("ui-gallery-kbd-component");

    vec![body.into_element(cx)]
}
