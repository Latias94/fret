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
        "`ButtonGroup` exposes typed composition points for buttons, inputs, input groups, selects, dropdowns, popovers, and nested button groups, plus `orientation(...)`, `a11y_label(...)`, and `labelled_by_element(...)`.",
        "`ButtonGroupSeparator` keeps divider ownership explicit through `orientation(...)` and layout refinements; default divider thickness remains recipe-owned.",
        "`ButtonGroupText` uses `new(...)` for plain text and `new_children(...)` for custom inline content. `Label::for_control(...)` inside `ButtonGroupText::new_children(...)` is the Rust-native mapping for the upstream `asChild` label example, without widening the recipe to generic slot merging (ADR 0115).",
    ]);

    let notes = doc_layout::notes_block([
        "Gallery sections now mirror shadcn Button Group docs first: Demo, Usage, Accessibility, ButtonGroup vs ToggleGroup, examples, RTL, API Reference.",
        "`ButtonGroupText` and `Flex-1 items` remain after the upstream path as focused Fret follow-ups: one shows the explicit `new_children(...)` + `Label::for_control(...)` mapping for the upstream `asChild` label lane, the other demonstrates caller-owned flex negotiation.",
        "Default-style ownership stays aligned with upstream: merged borders, outer radii, separator thickness, and nested-group gap are recipe-owned; width/flex growth remains caller-owned.",
        "Accessibility matches the upstream intent: the root stamps `role=group`, `a11y_label(...)` covers the `aria-label` path, and `labelled_by_element(...)` covers the `aria-labelledby` path without introducing a DOM-only API.",
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

    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-button-group-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .test_id_prefix("ui-gallery-button-group-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let accessibility = DocSection::build(cx, "Accessibility", accessibility)
        .test_id_prefix("ui-gallery-button-group-accessibility")
        .code_rust_from_file_region(snippets::accessibility::SOURCE, "example");
    let orientation = DocSection::build(cx, "Orientation", orientation)
        .test_id_prefix("ui-gallery-button-group-orientation")
        .code_rust_from_file_region(snippets::orientation::SOURCE, "example");
    let size = DocSection::build(cx, "Size", size)
        .test_id_prefix("ui-gallery-button-group-size")
        .code_rust_from_file_region(snippets::size::SOURCE, "example");
    let nested = DocSection::build(cx, "Nested", nested)
        .test_id_prefix("ui-gallery-button-group-nested")
        .code_rust_from_file_region(snippets::nested::SOURCE, "example");
    let separator = DocSection::build(cx, "Separator", separator)
        .test_id_prefix("ui-gallery-button-group-separator")
        .code_rust_from_file_region(snippets::separator::SOURCE, "example");
    let split = DocSection::build(cx, "Split", split)
        .test_id_prefix("ui-gallery-button-group-split")
        .code_rust_from_file_region(snippets::split::SOURCE, "example");
    let input = DocSection::build(cx, "Input", input)
        .test_id_prefix("ui-gallery-button-group-input")
        .code_rust_from_file_region(snippets::input::SOURCE, "example");
    let input_group = DocSection::build(cx, "Input Group", input_group)
        .test_id_prefix("ui-gallery-button-group-input-group")
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let dropdown = DocSection::build(cx, "Dropdown Menu", dropdown)
        .test_id_prefix("ui-gallery-button-group-dropdown")
        .code_rust_from_file_region(snippets::dropdown_menu::SOURCE, "example");
    let select = DocSection::build(cx, "Select", select)
        .test_id_prefix("ui-gallery-button-group-select")
        .code_rust_from_file_region(snippets::button_group_select::SOURCE, "example");
    let popover = DocSection::build(cx, "Popover", popover)
        .test_id_prefix("ui-gallery-button-group-popover")
        .code_rust_from_file_region(snippets::popover::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-button-group-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let text = DocSection::build(cx, "ButtonGroupText", text)
        .test_id_prefix("ui-gallery-button-group-text")
        .code_rust_from_file_region(snippets::text::SOURCE, "example");
    let flex_1 = DocSection::build(cx, "Flex-1 items (Fret)", flex_1)
        .test_id_prefix("ui-gallery-button-group-flex1")
        .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Button Group docs order first, then appends Fret-specific follow-ups for `ButtonGroupText` and caller-owned flex growth.",
        ),
        vec![
            demo,
            usage,
            accessibility,
            vs_toggle_group,
            orientation,
            size,
            nested,
            separator,
            split,
            input,
            input_group,
            dropdown,
            select,
            popover,
            rtl,
            api_reference,
            text,
            flex_1,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-button-group").into_element(cx)]
}
