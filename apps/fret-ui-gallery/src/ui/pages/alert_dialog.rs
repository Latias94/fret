use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert_dialog as snippets;

pub(super) fn preview_alert_dialog(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let small = snippets::small::render(cx);
    let media = snippets::media::render(cx);
    let small_with_media = snippets::small_with_media::render(cx);
    let destructive = snippets::destructive::render(cx);
    let parts = snippets::parts::render(cx);
    let detached_trigger = snippets::detached_trigger::render(cx);
    let rich_content = snippets::rich_content::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/alert-dialog.mdx`, `repo-ref/ui/apps/v4/registry/radix-vega/examples/alert-dialog-example.tsx`, and `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert-dialog.tsx`.",
            "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
            "Current Fret parity is strongest on semantics and policy defaults: outside press does not dismiss, and initial focus prefers AlertDialogCancel when present.",
            "Authoring ergonomics improved: AlertDialogAction and AlertDialogCancel can now resolve the current open model from AlertDialog content scope via `from_scope(...)`, expose `children(...)` for custom button content, and AlertDialogTitle / AlertDialogDescription now accept `new_children(...)` for attributed or precomposed subtrees.",
            "Detached triggers are now supported through `AlertDialogHandle`; the main remaining gap is at the root surface, where Fret still uses a closure/compose root instead of a fully nested children API.",
            "Base UI remains a mechanism reference for modal defaults like `role=alertdialog` and disabled pointer dismissal; payload wiring and function-as-children composition are still not implemented for Alert Dialog today.",
            "Keep dialog copy concise and explicit, and ensure destructive actions have clear labels and visual hierarchy.",
        ],
    );

    let usage = doc_layout::notes(
        cx,
        [
            "Use `AlertDialog` when you need explicit confirmation for destructive or irreversible actions.",
            "Start with the parts adapter (`into_element_parts`) when you want a call site that stays closest to shadcn docs composition.",
            "Use `AlertDialogHandle` when triggers and content live in different subtrees. The last activated detached trigger becomes the focus-restore target.",
            "Prefer `AlertDialogCancel` + `AlertDialogAction` over custom buttons to preserve consistent semantics and focus handling.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Alert Dialog docs order first, then adds a Fret-specific rich-content example to demonstrate composable title/description subtrees.",
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
            DocSection::new("Parts", parts)
                .description("Part surface adapters for shadcn-style call sites.")
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
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-section-notes-title")
                .test_id_prefix("ui-gallery-alert-dialog-notes")
                .description("Guidelines and best practices for alert dialogs."),
        ],
    );

    vec![body]
}
