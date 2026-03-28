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
        "`Sheet::new_controllable(cx, None, false).children([...])` plus `SheetPart::content_with(...)` is the closest Fret equivalent to upstream nested `SheetTrigger` / `SheetContent` composition while keeping the default import lane copyable.",
        "`SheetContent::new([]).with_children(cx, ...)` plus `SheetHeader::new([]).with_children(cx, ...)` / `SheetFooter::new([]).with_children(cx, ...)` is the default copyable content lane for upstream-like nested composition.",
        "`SheetContent::build(...)`, `SheetHeader::build(...)`, and `SheetFooter::build(...)` remain the builder-first follow-up when a snippet genuinely wants `push_ui(...)` assembly instead of composable nested sections.",
        "`Sheet::side(...)` accepts the documented `top`, `right`, `bottom`, and `left` placements through `SheetSide`.",
        "`SheetContent::show_close_button(false)` is the Fret equivalent of upstream `showCloseButton={false}`.",
        "`SheetClose::from_scope().build(cx, button)` is the closest Fret equivalent to upstream `<SheetClose asChild>` for footer or custom close actions.",
        "`Sheet::compose()` plus explicit `SheetPortal` / `SheetOverlay` stays as the advanced adapter follow-up instead of displacing the docs-aligned default lane.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/sheet.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/sheet.mdx`, `repo-ref/ui/apps/v4/content/docs/components/base/sheet.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`, `repo-ref/primitives/packages/react/dialog/src/dialog.tsx`, and `repo-ref/base-ui/packages/react/src/dialog/`.",
        "Preview mirrors the shadcn Sheet docs path after `Installation`: `Demo`, `Usage`, `Side`, `No Close Button`, `RTL`, and `API Reference`.",
        "Radix/Base UI semantics are already largely covered by the existing overlay, dismissal, focus-restore, and sizing tests in `ecosystem/fret-ui-shadcn/src/sheet.rs`; the remaining drift addressed here is recipe/public-surface parity rather than a `fret-ui` mechanism bug.",
        "`Usage` is the default copyable `children([...])` path, while `Parts` stays after `API Reference` as a focused advanced follow-up for explicit part adapters (`SheetTrigger` / `SheetPortal` / `SheetOverlay`).",
        "The docs-path examples now share the same `Sheet::children([...])` root lane plus `SheetContent::new([]).with_children(cx, ...)` content lane, while `compose()` and `SheetContent::build(...)` remain focused builder-first follow-ups.",
        "A broader generic heterogeneous root children API is not warranted beyond `Sheet::children([...])`: the root structure is still typed as trigger/content plus optional portal/overlay adapters, and `with_children(...)` now covers the nested section-composition cliff inside the content surface.",
        "Default close affordance lives in `SheetContent`, matching upstream; disable it with `show_close_button(false)`.",
        "`No Close Button` and `RTL` now stay closer to the upstream docs copy and structure so the gallery page teaches the component instead of only probing the mechanism.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sheet-api-reference")
        .description("Public surface summary and composable children-lane guidance.");
    let notes = DocSection::build(cx, "Notes", notes)
        .max_w(Px(980.0))
        .no_shell()
        .description("Implementation notes, source axes, and remaining parity boundaries.")
        .test_id_prefix("ui-gallery-sheet-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-demo")
        .description("Official shadcn sheet demo with profile fields and footer actions.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .max_w(Px(980.0))
        .title_test_id("ui-gallery-section-usage-title")
        .description(
            "Default copyable `children([...])` root lane with composable `with_children(...)` content sections.",
        )
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let side = DocSection::build(cx, "Side", side)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-side")
        .description(
            "Use the `side` prop to set the edge of the screen where the sheet appears. Values are `top`, `right`, `bottom`, and `left`.",
        )
        .code_rust_from_file_region(snippets::side::SOURCE, "example");
    let no_close_button = DocSection::build(cx, "No Close Button", no_close_button)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-no-close")
        .description("Hide the default close button and rely on outside press or Escape.")
        .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sheet-rtl")
        .description("Sheet layout should follow right-to-left direction context.")
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
