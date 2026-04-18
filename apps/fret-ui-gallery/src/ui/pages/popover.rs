use super::super::*;
use fret::AppComponentCx;

pub(super) fn preview_popover(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::popover as snippets;

    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let align = snippets::align::render(cx);
    let with_form = snippets::with_form::render(cx);
    let detached_trigger = snippets::detached_trigger::render(cx);
    let open_on_hover = snippets::open_on_hover::render(cx);
    let inline_children = snippets::inline_children::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Popover::new(cx, trigger, content)` remains the default recipe-level entry point and already covers the upstream nested `<Popover><PopoverTrigger /><PopoverContent /></Popover>` composition plus the custom-trigger / `asChild` story, because `trigger` can be any landed or late-landed element.",
        "`Popover::side(...)` and `Popover::align(...)` cover the placement vocabulary documented by Radix/shadcn.",
        "`side_offset(...)` and `align_offset(...)` are available when placement needs fine-grained tuning beyond the default popper gap.",
        "`PopoverTrigger::build(...)`, `PopoverContent::build(cx, ...)`, and `PopoverContent::new([...])` cover the copyable compound-parts lane plus the landed-children follow-up without adding a separate heterogeneous root `children([...])` / `compose()` API.",
        "`PopoverContent::refine_layout(...)` remains the caller-owned path for explicit widths such as `w-72`, `w-64`, or the wider `w-80` demo.",
        "`PopoverContent` keeps panel chrome and default width recipe-owned, but it no longer stretches inline-sized children by default; page/container width negotiation stays caller-owned.",
        "`Popover::trigger_element(...)` and `Popover::anchor_element(...)` cover the detached-trigger, anchor-aware follow-up without widening the default shadcn docs lane.",
        "`Popover::open_on_hover(...)`, `hover_open_delay_frames(...)`, and `hover_close_delay_frames(...)` cover the Base UI hover-open follow-up while staying outside the default docs path.",
        "A generic heterogeneous `children([...])` root API is not currently warranted here: unlike Dialog/Drawer, Popover root only needs trigger/content, while managed-open and anchor-aware cases already stay explicit on `from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/popover.rs`. Reference stack: shadcn/base Popover docs and demo, the default registry recipe, Radix Primitives popover semantics, and Base UI popover ownership.",
        "Preview mirrors the shadcn/base Popover docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Align`, `With Form`, `RTL`, and `API Reference`. `Detached Trigger (Fret)`, `Open on Hover (Base UI/Fret)`, and `Inline Children (Fret)` stay as explicit follow-ups.",
        "`Demo` keeps the official new-york `popover-demo` structure with the four dimensions rows, while `Basic` / `Align` / `With Form` / `RTL` track the docs examples.",
        "`RTL` now keeps both the physical sides and the logical `inline-start` / `inline-end` cases from the upstream Base UI docs so direction-aware placement stays reviewable.",
        "`Detached Trigger (Fret)` documents the current advanced seam via `trigger_element(...)` / `anchor_element(...)`; Base UI's handle-driven multi-trigger/payload surface would be a public-surface follow-up rather than a mechanism fix.",
        "Default recipe-level root authoring stays `Popover::new(cx, trigger, content)`, while `PopoverTrigger::build(...)` and `PopoverContent::build(cx, ...)` cover the typed compound-parts lane; anchor-aware sizing and detached-trigger cases still use the explicit advanced seams (`from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`).",
        "Base UI documents trigger-owned `openOnHover` / `delay` props, but Fret keeps hover-open as an explicit popover-root follow-up because the hover-intent corridor belongs to the renderer-owned mechanism layer.",
        "Popover dismissal, focus restore, outside-press routing, placement, and inline-child sizing are already covered by the existing Radix/web contract tests and UI Gallery overlay gates; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "Keep content width explicit when the upstream example does (for example `w-40`, `w-64`, or `w-80`) so recipe chrome and page-level layout ownership stay separate.",
        "For dense input rows, prefer `Field` / `FieldGroup` recipes to keep spacing consistent with other form surfaces.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-popover-api-reference")
        .description("Public surface summary and placement ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-popover-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-popover-demo")
        .description("Official new-york demo with the dimensions form inside the popover.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .test_id_prefix("ui-gallery-popover-usage")
        .description(
            "Copyable shadcn-style composition reference using typed trigger/content parts.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .test_id_prefix("ui-gallery-popover-basic")
        .description("A simple popover with a header, title, and description.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let align = DocSection::build(cx, "Align", align)
        .test_id_prefix("ui-gallery-popover-align")
        .description("Use `align` on `PopoverContent` to control horizontal alignment.")
        .code_rust_from_file_region(snippets::align::SOURCE, "example");
    let with_form = DocSection::build(cx, "With Form", with_form)
        .test_id_prefix("ui-gallery-popover-with-form")
        .description("A popover with form fields inside.")
        .code_rust_from_file_region(snippets::with_form::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-popover-rtl")
        .description("Popover layout should follow a right-to-left direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let detached_trigger = DocSection::build(cx, "Detached Trigger (Fret)", detached_trigger)
        .test_id_prefix("ui-gallery-popover-detached-trigger")
        .description(
            "Advanced follow-up: keep the trigger outside the popover root and wire it explicitly with `trigger_element(...)` / `anchor_element(...)`.",
        )
        .code_rust_from_file_region(snippets::detached_trigger::SOURCE, "example");
    let open_on_hover = DocSection::build(cx, "Open on Hover (Base UI/Fret)", open_on_hover)
        .test_id_prefix("ui-gallery-popover-open-on-hover")
        .description(
            "Base UI/Fret follow-up: hover intent can open the popover without changing the default click-first docs lane.",
        )
        .code_rust_from_file_region(snippets::open_on_hover::SOURCE, "example");
    let inline_children = DocSection::build(cx, "Inline Children (Fret)", inline_children)
        .test_id_prefix("ui-gallery-popover-inline-children")
        .description(
            "Fret-only regression surface: inline-sized children inside `PopoverContent` should keep intrinsic width by default.",
        )
        .code_rust_from_file_region(snippets::inline_children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/base Popover docs path through `API Reference`, using the dimensions form demo as the lead example; detached-trigger, hover-open, and inline-children follow-ups stay explicit afterwards.",
        ),
        vec![
            demo,
            usage,
            basic,
            align,
            with_form,
            rtl,
            api_reference,
            detached_trigger,
            open_on_hover,
            inline_children,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-popover").into_element(cx)]
}
