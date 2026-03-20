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
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Popover::side(...)` and `Popover::align(...)` cover the placement vocabulary documented by Radix/shadcn.",
        "`side_offset(...)` and `align_offset(...)` are available when placement needs fine-grained tuning beyond the default popper gap.",
        "`PopoverContent::build(cx, ...)` pairs with `PopoverTrigger::build(...)` for the typed compound-parts lane, while `PopoverContent::refine_layout(...)` remains the caller-owned path for explicit widths such as the default 288px example or the wider form demo.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/popover.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/popover.mdx` and Radix Popover docs.",
        "Preview mirrors the shadcn Popover docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Align`, `With Form`, `RTL`, and `API Reference`.",
        "Default recipe-level root authoring now uses `Popover::new(cx, trigger, content)`, while `PopoverTrigger::build(...)` and `PopoverContent::build(cx, ...)` cover the typed compound-parts lane; anchor-aware sizing and detached-trigger cases still use the explicit advanced seams (`from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`).",
        "Keep content width explicit (for example `w-72` / 288px in default content, or 320px in the form demo) for predictable layout.",
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
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description(
            "Copyable shadcn-style composition reference using typed trigger/content parts.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .test_id_prefix("ui-gallery-popover-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let align = DocSection::build(cx, "Align", align)
        .test_id_prefix("ui-gallery-popover-align")
        .code_rust_from_file_region(snippets::align::SOURCE, "example");
    let with_form = DocSection::build(cx, "With Form", with_form)
        .test_id_prefix("ui-gallery-popover-with-form")
        .code_rust_from_file_region(snippets::with_form::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-popover-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Popover docs path after `Installation`, using the dimensions form demo as the lead example.",
        ),
        vec![
            demo,
            usage,
            basic,
            align,
            with_form,
            rtl,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-popover").into_element(cx)]
}
