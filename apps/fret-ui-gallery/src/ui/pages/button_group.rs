use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::button_group as snippets;

pub(super) fn preview_button_group(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let accessibility = snippets::accessibility::render(cx);
    let orientation = snippets::orientation::render(cx);
    let size = snippets::size::render(cx);
    let nested = snippets::nested::render(cx);
    let separator = snippets::separator::render(cx);
    let split = snippets::split::render(cx);
    let input = snippets::input::render(cx);
    let input_group = snippets::input_group::render(cx);
    let dropdown = snippets::dropdown_menu::render(cx);
    let select = snippets::button_group_select::render(cx);
    let popover = snippets::popover::render(cx);
    let rtl = snippets::rtl::render(cx);
    let text = snippets::text::render(cx);
    let flex_1 = snippets::flex_1_items::render(cx);

    let vs_toggle_group = doc_layout::notes_block([
        "Use `ButtonGroup` when grouped controls trigger actions such as submit, archive, or open-menu.",
        "Use `ToggleGroup` when buttons represent pressed or selected state. Fret keeps that distinction as a separate typed surface instead of overloading `ButtonGroup` with toggle semantics.",
    ]);

    let api_reference = doc_layout::notes_block([
        "`ButtonGroup` exposes typed composition points for buttons, inputs, input groups, selects, dropdowns, popovers, and nested button groups, plus `orientation(...)` and `a11y_label(...)`.",
        "`ButtonGroupSeparator` keeps divider ownership explicit through `orientation(...)` and layout refinements; default divider thickness remains recipe-owned.",
        "`ButtonGroupText` uses `new(...)` for plain text and `new_children(...)` for custom inline content. This is Fret's explicit alternative to generic `asChild` prop merging (ADR 0115).",
    ]);

    let notes = doc_layout::notes_block([
        "Gallery sections now mirror shadcn Button Group docs first: Demo, Usage, Accessibility, ButtonGroup vs ToggleGroup, examples, RTL, API Reference.",
        "`ButtonGroupText` and `Flex-1 items` remain after the upstream path as focused Fret follow-ups: one documents the explicit `new_children(...)` surface, the other demonstrates caller-owned flex negotiation.",
        "Default-style ownership stays aligned with upstream: merged borders, outer radii, separator thickness, and nested-group gap are recipe-owned; width/flex growth remains caller-owned.",
        "Accessibility matches the upstream intent: the root stamps `role=group`, and `a11y_label(...)` provides the `aria-label` equivalent without introducing a DOM-only API.",
    ]);

    let vs_toggle_group = DocSection::build(cx, "ButtonGroup vs ToggleGroup", vs_toggle_group)
        .no_shell()
        .test_id_prefix("ui-gallery-button-group-vs-toggle-group");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .description("Public surface summary plus copyable examples for the core API pieces.")
        .test_id_prefix("ui-gallery-button-group-api-reference")
        .code_rust_from_file_region(snippets::api_reference::SOURCE, "example");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-button-group-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Button Group docs order first, then appends Fret-specific follow-ups for `ButtonGroupText` and caller-owned flex growth.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-button-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .test_id_prefix("ui-gallery-button-group-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Accessibility", accessibility)
                .test_id_prefix("ui-gallery-button-group-accessibility")
                .code_rust_from_file_region(snippets::accessibility::SOURCE, "example"),
            vs_toggle_group,
            DocSection::new("Orientation", orientation)
                .test_id_prefix("ui-gallery-button-group-orientation")
                .code_rust_from_file_region(snippets::orientation::SOURCE, "example"),
            DocSection::new("Size", size)
                .test_id_prefix("ui-gallery-button-group-size")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Nested", nested)
                .test_id_prefix("ui-gallery-button-group-nested")
                .code_rust_from_file_region(snippets::nested::SOURCE, "example"),
            DocSection::new("Separator", separator)
                .test_id_prefix("ui-gallery-button-group-separator")
                .code_rust_from_file_region(snippets::separator::SOURCE, "example"),
            DocSection::new("Split", split)
                .test_id_prefix("ui-gallery-button-group-split")
                .code_rust_from_file_region(snippets::split::SOURCE, "example"),
            DocSection::new("Input", input)
                .test_id_prefix("ui-gallery-button-group-input")
                .code_rust_from_file_region(snippets::input::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .test_id_prefix("ui-gallery-button-group-input-group")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("Dropdown Menu", dropdown)
                .test_id_prefix("ui-gallery-button-group-dropdown")
                .code_rust_from_file_region(snippets::dropdown_menu::SOURCE, "example"),
            DocSection::new("Select", select)
                .test_id_prefix("ui-gallery-button-group-select")
                .code_rust_from_file_region(snippets::button_group_select::SOURCE, "example"),
            DocSection::new("Popover", popover)
                .test_id_prefix("ui-gallery-button-group-popover")
                .code_rust_from_file_region(snippets::popover::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-button-group-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            api_reference,
            DocSection::new("ButtonGroupText", text)
                .test_id_prefix("ui-gallery-button-group-text")
                .code_rust_from_file_region(snippets::text::SOURCE, "example"),
            DocSection::new("Flex-1 items (Fret)", flex_1)
                .test_id_prefix("ui-gallery-button-group-flex1")
                .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example"),
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-button-group")]
}
