use super::super::*;
use fret::UiCx;

pub(super) fn preview_sheet(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::sheet as snippets;

    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let parts = snippets::parts::render(cx);
    let side = snippets::side::render(cx);
    let no_close_button = snippets::no_close_button::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Sheet::side(...)` accepts the documented `top`, `right`, `bottom`, and `left` placements through `SheetSide`.",
        "`SheetContent::show_close_button(false)` is the Fret equivalent of upstream `showCloseButton={false}`.",
        "`SheetClose::from_scope()` remains available for additional close affordances inside sheet content without threading the same open model into each button.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/sheet.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/sheet.mdx` and `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`.",
        "Preview mirrors the shadcn Sheet docs path after `Installation`: `Demo`, `Usage`, `Side`, `No Close Button`, `RTL`, and `API Reference`.",
        "`Sheet::compose()` is a recipe-level bridge for shadcn-style part composition without pushing children API concerns into the mechanism layer.",
        "Default close affordance lives in `SheetContent`, matching upstream; disable it with `show_close_button(false)`.",
        "`Usage` is the default copyable path; `Parts` stays after `API Reference` as a focused advanced follow-up for explicit part adapters (`SheetTrigger` / `SheetPortal` / `SheetOverlay`).",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sheet-api-reference")
        .description("Public surface summary and close-affordance ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sheet-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .max_w(Px(980.0))
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference for Sheet.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let side = DocSection::build(cx, "Side", side)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-side")
        .code_rust_from_file_region(snippets::side::SOURCE, "example");
    let no_close_button = DocSection::build(cx, "No Close Button", no_close_button)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-no-close")
        .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-parts")
        .description(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership.",
        )
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Sheet docs path after `Installation`, then keeps the advanced Fret-only `Parts` adapter section explicit.",
        ),
        vec![
            demo,
            usage,
            side,
            no_close_button,
            rtl,
            api_reference,
            parts,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-sheet").into_element(cx)]
}
