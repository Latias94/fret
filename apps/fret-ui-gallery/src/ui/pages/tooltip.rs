use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tooltip as snippets;

pub(super) fn preview_tooltip(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo_tooltip = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let long_content_tooltip = snippets::long_content::render(cx);
    let focus_row = snippets::keyboard_focus::render(cx);
    let side_row = snippets::sides::render(cx);
    let keyboard_tooltip = snippets::keyboard_shortcut::render(cx);
    let disabled_tooltip = snippets::disabled_button::render(cx);
    let rtl_row = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "Tooltip already exposes shadcn-style part names (`TooltipTrigger`, `TooltipContent`, `TooltipProvider`), and `Tooltip::new(cx, trigger, content)` is the recipe-level composition entry point.",
        "Gallery sections mirror shadcn docs first; `Long Content` and `Keyboard Focus` are Fret-specific parity sections appended afterward.",
        "Wrap related tooltips in one `TooltipProvider` to get consistent delay-group behavior.",
        "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
        "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
        "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Implementation notes and regression guidelines.");
    let demo_tooltip = DocSection::build(cx, "Demo", demo_tooltip)
        .description("Basic tooltip with an arrow and a short content label.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference for Tooltip.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let side_row = DocSection::build(cx, "Side", side_row)
        .description("Use the `side` prop to change the position of the tooltip.")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let keyboard_tooltip = DocSection::build(cx, "With Keyboard Shortcut", keyboard_tooltip)
        .description("Compose richer content such as key hints inside the tooltip panel.")
        .code_rust_from_file_region(snippets::keyboard_shortcut::SOURCE, "example");
    let disabled_tooltip = DocSection::build(cx, "Disabled Button", disabled_tooltip)
        .description(
            "Use a non-disabled wrapper as the trigger so hover/focus can still open the tooltip.",
        )
        .code_rust_from_file_region(snippets::disabled_button::SOURCE, "example");
    let rtl_row = DocSection::build(cx, "RTL", rtl_row)
        .description("Tooltip placement and alignment should work under RTL.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let long_content_tooltip = DocSection::build(cx, "Long Content", long_content_tooltip)
        .description(
            "Longer tooltip content should wrap at the max-width boundary without collapsing to a narrow column.",
        )
        .code_rust_from_file_region(snippets::long_content::SOURCE, "example");
    let focus_row = DocSection::build(cx, "Keyboard Focus", focus_row)
        .description("Tooltips should open when the trigger receives keyboard focus.")
        .code_rust_from_file_region(snippets::keyboard_focus::SOURCE, "example");

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Tooltip docs order first, then appends Fret-specific `Long Content` and `Keyboard Focus` parity sections.",
        ),
        vec![
            demo_tooltip,
            usage,
            side_row,
            keyboard_tooltip,
            disabled_tooltip,
            rtl_row,
            long_content_tooltip,
            focus_row,
            notes,
        ],
    )
    .test_id("ui-gallery-tooltip-component");

    vec![page.into_element(cx)]
}
