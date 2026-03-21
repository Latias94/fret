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

    let api_reference = doc_layout::notes_block([
        "`Tooltip::new(cx, trigger, content)` is the closest Fret equivalent of upstream nested `<Tooltip><TooltipTrigger /><TooltipContent /></Tooltip>` composition.",
        "`TooltipProvider` owns shared delay-group policy, while `Tooltip::open_delay(...)`, `close_delay(...)`, `track_cursor_axis(...)`, and `anchor_element(...)` stay available as explicit root-level tuning seams.",
        "`TooltipTrigger::build(...)` and `TooltipContent::build(cx, ...)` cover the typed compound-parts lane for copyable first-party snippets.",
        "No extra generic `children([...])` / `compose()` root API is currently warranted: tooltip root only needs trigger/content, and `Tooltip::new(...)` already models that contract without hidden collection or scope state.",
    ]);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/tooltip.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/base/tooltip.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`, `repo-ref/primitives/packages/react/tooltip/src/tooltip.tsx`, and `repo-ref/base-ui/packages/react/src/tooltip/root/TooltipRoot.tsx`.",
        "Preview mirrors the shadcn/base Tooltip docs path after `Installation`: `Demo`, `Usage`, `Side`, `With Keyboard Shortcut`, `Disabled Button`, `RTL`, and `API Reference`. `Long Content` and `Keyboard Focus` remain explicit Fret parity follow-ups.",
        "`Usage` keeps a local `TooltipProvider` wrapper so the code tab is standalone and copyable, while the page still teaches the default `Tooltip::new(...)` root lane plus typed trigger/content builders.",
        "Wrap related tooltips in one `TooltipProvider` to get consistent delay-group behavior.",
        "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
        "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
        "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-tooltip-api-reference")
        .description("Public surface summary, timing ownership, and children API guidance.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-tooltip-notes")
        .description("Implementation notes and regression guidelines.");
    let demo_tooltip = DocSection::build(cx, "Demo", demo_tooltip)
        .test_id_prefix("ui-gallery-tooltip-demo")
        .description("Basic tooltip with an arrow and a short content label.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .test_id_prefix("ui-gallery-tooltip-usage")
        .description(
            "Copyable shadcn-style composition reference using the default `Tooltip::new(...)` root plus typed trigger/content parts.",
        )
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
            "Preview follows the shadcn/base Tooltip docs path through `API Reference`, then appends Fret-specific `Long Content` and `Keyboard Focus` parity sections.",
        ),
        vec![
            demo_tooltip,
            usage,
            side_row,
            keyboard_tooltip,
            disabled_tooltip,
            rtl_row,
            api_reference,
            long_content_tooltip,
            focus_row,
            notes,
        ],
    )
    .test_id("ui-gallery-tooltip-component");

    vec![page.into_element(cx)]
}
