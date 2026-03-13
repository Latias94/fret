use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert_dialog as snippets;

pub(super) fn preview_alert_dialog(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let small = snippets::small::render(cx);
    let media = snippets::media::render(cx);
    let small_with_media = snippets::small_with_media::render(cx);
    let destructive = snippets::destructive::render(cx);
    let parts = snippets::parts::render(cx);
    let detached_trigger = snippets::detached_trigger::render(cx);
    let rich_content = snippets::rich_content::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`AlertDialogContent::size(...)` accepts `AlertDialogContentSize::Default` and `AlertDialogContentSize::Sm`, matching the upstream `size=\"default\" | \"sm\"` surface.",
        "`AlertDialog::compose()` is the closest shadcn-style root today; the top-level authoring surface still uses a builder/closure bridge instead of a fully nested children API.",
        "`AlertDialogAction::from_scope(...)` and `AlertDialogCancel::from_scope(...)` keep footer composition close to shadcn docs without threading the same open model through every button.",
        "`AlertDialogTitle::new_children(...)` and `AlertDialogDescription::new_children(...)` already support composed or attributed subtree content when string-only labels are not enough.",
    ]);

    let extras = doc_layout::notes_block([
        "`Parts` documents the explicit part adapters (`Trigger` / `Portal` / `Overlay`) used by `compose()` call sites.",
        "`Detached Trigger` shows `AlertDialogHandle`, the supported path when the opener and the dialog content live in different subtrees and still need correct focus restore.",
        "`Rich Content` demonstrates the currently supported children-style extensions for title, description, and footer content.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/alert-dialog.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`, and `repo-ref/ui/apps/v4/registry/new-york-v4/examples/alert-dialog-demo.tsx`.",
        "Preview mirrors the shadcn docs path after skipping `Installation`: `Demo`, `Usage`, `Basic`, `Small`, `Media`, `Small with Media`, `Destructive`, `RTL`, and `API Reference`.",
        "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
        "Modal semantics follow Radix/Base UI outcomes: outside press does not dismiss, `role=alertdialog` is preserved, and initial focus prefers `AlertDialogCancel` when present.",
        "Current remaining differences are mostly authoring-surface follow-ups, not layout or dismissal-policy drift.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-alert-dialog-api-reference")
        .description("Public surface summary and current authoring-surface guidance.");
    let extras = DocSection::build(cx, "Fret Extras", extras)
        .no_shell()
        .test_id_prefix("ui-gallery-alert-dialog-extras")
        .description("Focused follow-ups that stay outside the upstream docs path.");
    let notes = DocSection::build(cx, "Notes", notes)
        .title_test_id("ui-gallery-section-notes-title")
        .test_id_prefix("ui-gallery-alert-dialog-notes")
        .description("Parity notes and implementation pointers.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Alert Dialog docs path after `Installation`, then keeps Fret-only follow-ups explicit under `Fret Extras`.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Default-sized modal alert dialog.")
                .test_id_prefix("ui-gallery-alert-dialog-demo-docsec")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Alert Dialog.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("A minimal alert dialog with default buttons.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Small", small)
                .description("Compact dialog size for short copy.")
                .test_id_prefix("ui-gallery-alert-dialog-small-docsec")
                .code_rust_from_file_region(snippets::small::SOURCE, "example"),
            DocSection::new("Media", media)
                .description("Dialogs can optionally show a leading media/icon in the header.")
                .test_id_prefix("ui-gallery-alert-dialog-media-docsec")
                .code_rust_from_file_region(snippets::media::SOURCE, "example"),
            DocSection::new("Small with Media", small_with_media)
                .description("Small size + media variant.")
                .code_rust_from_file_region(snippets::small_with_media::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .description("Destructive styling for irreversible actions.")
                .test_id_prefix("ui-gallery-alert-dialog-destructive-docsec")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            api_reference,
            extras,
            DocSection::new("Parts", parts)
                .description("Fret-specific part surface adapters for shadcn-style call sites.")
                .test_id_prefix("ui-gallery-alert-dialog-parts-docsec")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Detached Trigger", detached_trigger)
                .description("Use `AlertDialogHandle` when the opener and the dialog content are authored in different subtrees.")
                .test_id_prefix("ui-gallery-alert-dialog-detached-trigger-docsec")
                .code_rust_from_file_region(snippets::detached_trigger::SOURCE, "example"),
            DocSection::new("Rich Content", rich_content)
                .description("Composable title/description content plus custom footer button content using `new_children(...)` and `children(...)`.")
                .test_id_prefix("ui-gallery-alert-dialog-rich-content-docsec")
                .code_rust_from_file_region(snippets::rich_content::SOURCE, "example"),
            notes,
        ],
    );

    vec![body]
}
