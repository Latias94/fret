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

    let api_reference = doc_layout::notes_block([
        "`Dialog::children([...])` is the default copyable root path for part-based composition, and `DialogPart` is available on the curated `shadcn` facade so the default import lane stays copyable.",
        "`DialogContent::build(...)` is the typed content-side companion on that same lane, so copyable snippets do not need to hand-land already-built content arrays.",
        "`DialogContent` owns the upstream-style default close affordance; opt out with `show_close_button(false)`.",
        "`DialogClose::from_scope().build(cx, button)` is the closest Fret equivalent to upstream `<DialogClose asChild>` for footer or custom close actions.",
        "`Dialog::compose()` remains available as a focused bridge when a page wants explicit builder-style `trigger(...).content_with(...)` assembly.",
    ]);

    let extras = doc_layout::notes_block([
        "`Parts` documents the advanced part adapters (`Trigger` / `Portal` / `Overlay`) used by explicit ownership call sites.",
        "Current remaining differences are mostly advanced authoring-surface tradeoffs, not layout, motion, or dismissal-policy drift.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/dialog.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/base/dialog.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`, and `repo-ref/ui/apps/v4/registry/new-york-v4/examples/dialog-demo.tsx`.",
        "Preview mirrors the shadcn/base Dialog docs path after `Installation`: `Demo`, `Usage`, `Custom Close Button`, `No Close Button`, `Sticky Footer`, `Scrollable Content`, `RTL`, and `API Reference`.",
        "`Usage` now teaches the root `children([...])` path because it is closer to upstream nested children composition; `Parts` keeps the explicit adapter lane for portal/overlay ownership.",
        "Default close and footer close examples now use `DialogClose` semantics instead of teaching raw model toggles for dialog-local dismiss actions.",
        "Scrollable examples isolate long content in ScrollArea so footer/header placement remains predictable under constrained viewport sizes.",
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
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference using typed content builders.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let custom_close = DocSection::build(cx, "Custom Close Button", custom_close)
        .description("Replace the close affordance with a custom footer action.")
        .code_rust_from_file_region(snippets::custom_close_button::SOURCE, "example");
    let no_close = DocSection::build(cx, "No Close Button", no_close)
        .description("Hide the default close button and rely on Escape or overlay dismissal.")
        .code_rust_from_file_region(snippets::no_close_button::SOURCE, "example");
    let sticky_footer = DocSection::build(cx, "Sticky Footer", sticky_footer)
        .description("Footer stays visible while the content scrolls.")
        .code_rust_from_file_region(snippets::sticky_footer::SOURCE, "example");
    let scrollable_content = DocSection::build(cx, "Scrollable Content", scrollable_content)
        .description("Long body scrolls while keeping the header visible.")
        .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Dialog layout should work under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership.",
        )
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

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
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-dialog").into_element(cx)]
}
