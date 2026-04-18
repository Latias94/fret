use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tooltip as snippets;

pub(super) fn preview_tooltip(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo_tooltip = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let long_content_tooltip = snippets::long_content::render(cx);
    let focus_row = snippets::keyboard_focus::render(cx);
    let side_row = snippets::sides::render(cx);
    let keyboard_tooltip = snippets::keyboard_shortcut::render(cx);
    let disabled_tooltip = snippets::disabled_button::render(cx);
    let rtl_row = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Tooltip::new(cx, trigger, content)` already acts as the default copyable root lane and covers the upstream nested `<Tooltip><TooltipTrigger /><TooltipContent /></Tooltip>` composition plus the `TooltipTrigger asChild` custom-trigger story, because `trigger` can be any landed or late-landed element.",
        "`TooltipProvider` owns shared delay-group policy, while `Tooltip::open_delay(...)`, `close_delay(...)`, `track_cursor_axis(...)`, and `anchor_element(...)` stay available as explicit root-level tuning seams.",
        "`TooltipTrigger::build(...)` and `TooltipContent::build(cx, ...)` cover the typed compound-parts lane for copyable first-party snippets, while `TooltipContent::new([...])` remains the landed-content follow-up when you already own the children.",
        "No extra generic `children([...])` / `compose()` root API is currently warranted: tooltip root only needs trigger/content, and `Tooltip::new(...)` already models that contract without hidden collection or scope state.",
        "No extra generic `children([...])` / `compose()` / `asChild` root API is currently warranted: tooltip root only needs trigger/content, and `Tooltip::new(...)` already models that contract without hidden collection or scope state.",
    ]);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/tooltip.rs`. Reference stack: shadcn Tooltip docs, the default registry chrome, Radix Primitives tooltip semantics/lifecycle, and Base UI tooltip lifecycle.",
        "Tooltip hover/focus, Escape/outside-press dismissal, scroll-close, and Radix web parity are already covered by the existing tooltip tests in `ecosystem/fret-ui-shadcn`; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "Preview mirrors the shadcn/base Tooltip docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Side`, `With Keyboard Shortcut`, `Disabled Button`, `RTL`, and `API Reference`. `Long Content`, `Keyboard Focus`, and `Notes` stay as explicit Fret follow-ups.",
        "`Usage` keeps a local `TooltipProvider` wrapper so the code tab is standalone and copyable, while the page still teaches the default `Tooltip::new(...)` root lane plus typed trigger/content builders.",
        "Wrap related tooltips in one `TooltipProvider` to get consistent delay-group behavior.",
        "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
        "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
        "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-tooltip-api-reference")
        .description(
            "Public surface summary, trigger/content composition, and children API guidance.",
        );
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
        .test_id_prefix("ui-gallery-tooltip-side")
        .description("Use the `side` prop to change the position of the tooltip.")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let keyboard_tooltip = DocSection::build(cx, "With Keyboard Shortcut", keyboard_tooltip)
        .test_id_prefix("ui-gallery-tooltip-keyboard-shortcut")
        .description("Compose richer content such as key hints inside the tooltip panel.")
        .code_rust_from_file_region(snippets::keyboard_shortcut::SOURCE, "example");
    let disabled_tooltip = DocSection::build(cx, "Disabled Button", disabled_tooltip)
        .test_id_prefix("ui-gallery-tooltip-disabled-button")
        .description(
            "Use a non-disabled wrapper as the trigger so hover/focus can still open the tooltip.",
        )
        .code_rust_from_file_region(snippets::disabled_button::SOURCE, "example");
    let rtl_row = DocSection::build(cx, "RTL", rtl_row)
        .test_id_prefix("ui-gallery-tooltip-rtl")
        .description("Tooltip placement and alignment should work under RTL.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let long_content_tooltip = DocSection::build(cx, "Long Content", long_content_tooltip)
        .test_id_prefix("ui-gallery-tooltip-long-content")
        .description(
            "Longer tooltip content should wrap at the max-width boundary without collapsing to a narrow column.",
        )
        .code_rust_from_file_region(snippets::long_content::SOURCE, "example");
    let focus_row = DocSection::build(cx, "Keyboard Focus", focus_row)
        .test_id_prefix("ui-gallery-tooltip-keyboard-focus")
        .description("Tooltips should open when the trigger receives keyboard focus.")
        .code_rust_from_file_region(snippets::keyboard_focus::SOURCE, "example");

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/base Tooltip docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Side`, `With Keyboard Shortcut`, `Disabled Button`, `RTL`, and `API Reference`. `Long Content`, `Keyboard Focus`, and `Notes` stay as explicit Fret follow-ups.",
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
