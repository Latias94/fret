use super::super::*;
use fret::UiCx;

pub(super) fn preview_popover(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::popover as snippets;

    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let align = snippets::align::render(cx);
    let with_form = snippets::with_form::render(cx);
    let inline_children = snippets::inline_children::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Popover::new(cx, trigger, content)` is the closest Fret equivalent of upstream nested `<Popover><PopoverTrigger /><PopoverContent /></Popover>` composition.",
        "`Popover::side(...)` and `Popover::align(...)` cover the placement vocabulary documented by Radix/shadcn.",
        "`side_offset(...)` and `align_offset(...)` are available when placement needs fine-grained tuning beyond the default popper gap.",
        "`PopoverContent::build(cx, ...)` pairs with `PopoverTrigger::build(...)` for the typed compound-parts lane, while `PopoverContent::refine_layout(...)` remains the caller-owned path for explicit widths such as `w-72`, `w-64`, or the wider `w-80` demo.",
        "`PopoverContent` keeps panel chrome and default width recipe-owned, but it no longer stretches inline-sized children by default; page/container width negotiation stays caller-owned.",
        "A generic heterogeneous `children([...])` root API is not currently warranted here: unlike Dialog/Drawer, Popover root only needs trigger/content, while managed-open and anchor-aware cases already stay explicit on `from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/popover.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/base/popover.mdx`, `repo-ref/ui/apps/v4/content/docs/components/radix/popover.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx`, and `repo-ref/ui/apps/v4/registry/new-york-v4/examples/popover-demo.tsx`.",
        "Preview mirrors the shadcn/base Popover docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Align`, `With Form`, `RTL`, and `API Reference`; Fret-only regression sections stay afterwards.",
        "`Demo` keeps the official new-york `popover-demo` structure with the four dimensions rows, while `Basic` / `Align` / `With Form` / `RTL` track the docs examples.",
        "Default recipe-level root authoring stays `Popover::new(cx, trigger, content)`, while `PopoverTrigger::build(...)` and `PopoverContent::build(cx, ...)` cover the typed compound-parts lane; anchor-aware sizing and detached-trigger cases still use the explicit advanced seams (`from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`).",
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
    let inline_children = DocSection::build(cx, "Inline Children (Fret)", inline_children)
        .test_id_prefix("ui-gallery-popover-inline-children")
        .description(
            "Fret-only regression surface: inline-sized children inside `PopoverContent` should keep intrinsic width by default.",
        )
        .code_rust_from_file_region(snippets::inline_children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/base Popover docs path after `Installation`, using the dimensions form demo as the lead example; Fret-only regression sections stay afterwards.",
        ),
        vec![
            demo,
            usage,
            basic,
            align,
            with_form,
            rtl,
            api_reference,
            inline_children,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-popover").into_element(cx)]
}
