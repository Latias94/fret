use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tooltip as snippets;

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo_tooltip = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let long_content_tooltip = snippets::long_content::render(cx);
    let focus_row = snippets::keyboard_focus::render(cx);
    let side_row = snippets::sides::render(cx);
    let keyboard_tooltip = snippets::keyboard_shortcut::render(cx);
    let disabled_tooltip = snippets::disabled_button::render(cx);
    let rtl_row = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Tooltip already exposes shadcn-style part names (`TooltipTrigger`, `TooltipContent`, `TooltipProvider`), and `Tooltip::new(trigger, content)` is the recipe-level composition entry point.",
            "Wrap related tooltips in one TooltipProvider to get consistent delay-group behavior.",
            "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
            "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
            "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
        ],
    );

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Tooltip docs order: Demo, Usage, then Fret-specific parity checks for layout, focus, sides, richer content, disabled triggers, and RTL.",
        ),
        vec![
            DocSection::new("Demo", demo_tooltip)
                .description("Basic tooltip with an arrow and a short content label.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Tooltip.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Long Content", long_content_tooltip)
                .description(
                    "Longer tooltip content should wrap at the max width boundary without collapsing to a narrow column.",
                )
                .code_rust_from_file_region(snippets::long_content::SOURCE, "example"),
            DocSection::new("Keyboard Focus", focus_row)
                .description("Tooltips should open when the trigger receives keyboard focus.")
                .code_rust_from_file_region(snippets::keyboard_focus::SOURCE, "example"),
            DocSection::new("Side", side_row)
                .description("Tooltips can be placed on the four sides of the trigger.")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("With Keyboard Shortcut", keyboard_tooltip)
                .description("Compose richer content (e.g. key hints) inside the tooltip panel.")
                .code_rust_from_file_region(snippets::keyboard_shortcut::SOURCE, "example"),
            DocSection::new("Disabled Button", disabled_tooltip)
                .description(
                    "Use a non-disabled wrapper as the trigger so hover/focus can still open the tooltip.",
                )
                .code_rust_from_file_region(snippets::disabled_button::SOURCE, "example"),
            DocSection::new("RTL", rtl_row)
                .description("Tooltip placement and alignment should work under RTL.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-tooltip-component");

    vec![page]
}
