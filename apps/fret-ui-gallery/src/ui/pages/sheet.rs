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

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Sheet::side(...)` accepts the documented `top`, `right`, `bottom`, and `left` placements through `SheetSide`.",
            "`SheetContent::show_close_button(false)` is the Fret equivalent of upstream `showCloseButton={false}`.",
            "`SheetClose::from_scope()` remains available for additional close affordances inside sheet content without threading the same open model into each button.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/sheet.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/sheet.mdx` and `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`.",
            "Preview mirrors the shadcn Sheet docs path after `Installation`: `Demo`, `Usage`, `Side`, `No Close Button`, `RTL`, and `API Reference`.",
            "`Sheet::compose()` is a recipe-level bridge for shadcn-style part composition without pushing children API concerns into the mechanism layer.",
            "Default close affordance lives in `SheetContent`, matching upstream; disable it with `show_close_button(false)`.",
            "`Parts` stays after `API Reference` as a focused Fret follow-up for explicit part adapters (`SheetTrigger` / `SheetPortal` / `SheetOverlay`).",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Sheet docs path after `Installation`, then keeps the Fret-only `Parts` adapter section explicit.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .max_w(Px(980.0))
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Sheet.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Side", side)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-side")
                .code_rust_from_file_region(snippets::side::SOURCE, "example"),
            DocSection::new("No Close Button", no_close_button)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-no-close")
                .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .max_w(Px(980.0))
                .no_shell()
                .test_id_prefix("ui-gallery-sheet-api-reference")
                .description("Public surface summary and close-affordance ownership notes."),
            DocSection::new("Parts", parts)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-parts")
                .description("Fret-specific part surface adapters (Trigger/Portal/Overlay).")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sheet-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-sheet")]
}
