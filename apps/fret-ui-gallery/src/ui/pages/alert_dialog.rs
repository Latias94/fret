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
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Alert Dialog is modal by default and should be reserved for destructive or irreversible decisions.",
            "Use `AlertDialogCancel` + `AlertDialogAction` with the same open model to guarantee close behavior stays predictable.",
            "Keep dialog copy concise and explicit, and ensure destructive actions have clear labels and visual hierarchy.",
        ],
    );

    let usage = doc_layout::notes(
        cx,
        [
            "Use `AlertDialog` when you need explicit confirmation for destructive/irreversible actions.",
            "Prefer `AlertDialogCancel` + `AlertDialogAction` over custom buttons to preserve consistent semantics and focus handling.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Alert Dialog docs order and keeps each state in a separate section for quick lookup.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Default-sized modal alert dialog.")
                .test_id_prefix("ui-gallery-alert-dialog-demo-docsec")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
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
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Quick reference for composing an alert dialog."),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-section-notes-title")
                .description("Guidelines and best practices for alert dialogs."),
        ],
    );

    vec![body]
}
