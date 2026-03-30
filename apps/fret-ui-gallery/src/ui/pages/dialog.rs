use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::dialog as snippets;

pub(super) fn preview_dialog(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let parts = snippets::parts::render(cx);
    let custom_close = snippets::custom_close_button::render(cx);
    let no_close = snippets::no_close_button::render(cx);
    let sticky_footer = snippets::sticky_footer::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let rtl = snippets::rtl::render(cx);
    let detached_trigger = snippets::detached_trigger::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Dialog::children([...])` is the default copyable root path for part-based composition, and `DialogPart` is available on the curated `shadcn` facade so the default import lane stays copyable.",
        "`DialogPart::content_with(...)` plus `DialogContent::with_children(...)`, `DialogHeader::with_children(...)`, and `DialogFooter::with_children(...)` form the default copyable content lane when child parts need the current dialog scope.",
        "`Dialog::children([...])` is already the warranted composable root API here because the component owns Trigger/Portal/Overlay/Content parts and the scope-sensitive `DialogClose::from_scope()` buttons must stay inside `DialogContent`; no broader untyped JSX-style root children API is warranted beyond the typed `DialogPart` lane.",
        "`DialogContent::build(...)`, `DialogHeader::build(...)`, and `DialogFooter::build(...)` still work for already-materialized sections, but they are now the secondary builder-first lane instead of the default teaching path.",
        "`DialogContent` owns the upstream-style default close affordance; opt out with `show_close_button(false)`.",
        "`DialogClose::from_scope().build(cx, button)` is the closest Fret equivalent to upstream `<DialogClose asChild>` for footer or custom close actions.",
        "`Dialog::compose()` remains available as a focused bridge when a page wants explicit builder-style `trigger(...).content_with(...)` assembly.",
        "`Detached Trigger` shows `DialogHandle`, the supported path when the opener and the dialog content live in different subtrees and still need correct focus restore.",
    ]);

    let extras = doc_layout::notes_block([
        "`Parts` documents the advanced part adapters (`Trigger` / `Portal` / `Overlay`) used by explicit ownership call sites.",
        "`Detached Trigger` documents the handle-based follow-up lane for Base UI-style detached opener ownership.",
        "Current remaining differences are mostly payload-bearing detached trigger follow-ups or richer Base UI-specific store features, not layout, motion, or dismissal-policy drift.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/dialog.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/base/dialog.mdx`, `repo-ref/ui/apps/v4/content/docs/components/radix/dialog.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`, `repo-ref/ui/apps/v4/examples/{base,radix}/dialog-{demo,close-button,no-close-button,sticky-footer,scrollable-content,rtl}.tsx`, `repo-ref/primitives/packages/react/dialog/src/dialog.tsx`, and `repo-ref/base-ui/packages/react/src/dialog/root/DialogRoot.tsx`.",
        "Preview mirrors the shadcn/base Dialog docs path after `Installation`: `Demo`, `Usage`, `Custom Close Button`, `No Close Button`, `Sticky Footer`, `Scrollable Content`, `RTL`, and `API Reference`.",
        "`Usage` is the default copyable path; `Parts` stays as the advanced adapter section for explicit `DialogTrigger` / `DialogPortal` / `DialogOverlay` ownership.",
        "`Usage` now teaches the root `children([...])` path plus deferred `content_with(...)` / `with_children(...)` composition because it is closer to upstream nested children composition and keeps `DialogClose::from_scope()` in scope for footer actions.",
        "Demo, Custom Close Button, and RTL now follow the upstream docs example copy and width caps more closely, including `sm:max-w-[425px]`, `sm:max-w-md`, and `sm:max-w-sm`-shaped content widths.",
        "Default close and footer close examples now use `DialogClose` semantics instead of teaching raw model toggles for dialog-local dismiss actions, while the primary `Save changes` action remains a normal action button rather than a forced close.",
        "Scrollable examples isolate long content in ScrollArea so footer/header placement remains predictable under constrained viewport sizes.",
        "Radix Primitives and Base UI agree on the relevant semantics axis here: modal dialog, outside press dismisses by default, dismissal can be intercepted, and focus restores to the trigger on close. Those outcomes are already handled in `fret-ui-kit` / `fret-ui-shadcn`, so this page is now mainly a docs/public-surface alignment task rather than a `fret-ui` mechanism bug.",
        "`Detached Trigger` is now the focused Base UI-style follow-up for opener/content split ownership; it stays outside the default shadcn/base docs path and does not change the typed `DialogPart` default lane.",
        "Each scenario has stable test IDs to support fretboard diag scripts and regression screenshots.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-dialog-api-reference")
        .description("Public surface summary and current authoring-surface guidance.");
    let extras = DocSection::build(cx, "Fret Extras", extras)
        .no_shell()
        .test_id_prefix("ui-gallery-dialog-extras")
        .description("Focused follow-ups that stay outside the upstream docs path.");
    let notes = DocSection::build(cx, "Notes", notes)
        .description(
            "Keep test IDs stable so fretboard diag scripts and regression screenshots remain reusable.",
        )
        .test_id_prefix("ui-gallery-dialog-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Basic dialog with header, form fields, and footer actions.")
        .test_id_prefix("ui-gallery-dialog-demo-docsec")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference using deferred content_with + with_children.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let custom_close = DocSection::build(cx, "Custom Close Button", custom_close)
        .description("Replace the close affordance with a custom footer action.")
        .test_id_prefix("ui-gallery-dialog-custom-close-docsec")
        .code_rust_from_file_region(snippets::custom_close_button::SOURCE, "example");
    let no_close = DocSection::build(cx, "No Close Button", no_close)
        .description("Hide the default close button and rely on Escape or overlay dismissal.")
        .test_id_prefix("ui-gallery-dialog-no-close-docsec")
        .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example");
    let sticky_footer = DocSection::build(cx, "Sticky Footer", sticky_footer)
        .description("Footer stays visible while the content scrolls.")
        .test_id_prefix("ui-gallery-dialog-sticky-footer-docsec")
        .code_rust_from_file_region(snippets::sticky_footer::SOURCE, "example");
    let scrollable_content = DocSection::build(cx, "Scrollable Content", scrollable_content)
        .description("Long body scrolls while keeping the header visible.")
        .test_id_prefix("ui-gallery-dialog-scrollable-docsec")
        .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Dialog layout should work under an RTL direction provider.")
        .test_id_prefix("ui-gallery-dialog-rtl-docsec")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership.",
        )
        .test_id_prefix("ui-gallery-dialog-parts-docsec")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");
    let detached_trigger = DocSection::build(cx, "Detached Trigger", detached_trigger)
        .description("Use `DialogHandle` when the opener and the dialog content are authored in different subtrees.")
        .test_id_prefix("ui-gallery-dialog-detached-trigger-docsec")
        .code_rust_from_file_region(snippets::detached_trigger::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/base Dialog docs path after `Installation`, then keeps Fret-only follow-ups explicit under `Fret Extras`.",
        ),
        vec![
            demo,
            usage,
            custom_close,
            no_close,
            sticky_footer,
            scrollable_content,
            rtl,
            api_reference,
            extras,
            parts,
            detached_trigger,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-dialog").into_element(cx)]
}
