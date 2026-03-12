use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert as snippets;

pub(super) fn preview_alert(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let rich_title = snippets::rich_title::render(cx);
    let destructive = snippets::destructive::render(cx);
    let action = snippets::action::render(cx);
    let interactive_links = snippets::interactive_links::render(cx);
    let custom_colors = snippets::custom_colors::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/alert.rs` and `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`.",
            "Modern upstream reference: `repo-ref/ui/apps/v4/registry/radix-vega/examples/alert-example.tsx` and `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert.tsx`.",
            "Keep alert copy concise and action-oriented; reserve longer guidance for Dialog or Sheet.",
            "Prefer `AlertTitle::new_children(...)` when the title needs attributed text or a precomposed child subtree.",
            "Prefer `AlertDescription::new_children(...)` when the description needs multiple paragraphs, lists, or rich text.",
            "Gallery link examples open safe URLs in normal runs; scripted diag runs keep them deterministic by recording activation instead of launching the browser.",
            "Use `Destructive` only for high-risk or blocking failures to preserve visual hierarchy.",
            "Validate RTL + narrow layout so icon/title/description remain readable in editor sidebars.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the modern shadcn/Radix example grouping first—Basic, With Icons, Destructive, With Actions—then adds Fret-specific copy-paste surfaces.",
        ),
        vec![
            DocSection::new("Basic", basic)
                .description("Modern upstream basic patterns: title-only, title + description, and description-only.")
                .test_id_prefix("ui-gallery-alert-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("With Icons", demo)
                .description("Modern upstream icon patterns, including rich title/description content and long-text wrapping.")
                .test_id_prefix("ui-gallery-alert")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .description("Modern upstream destructive patterns: simple failure state plus multi-paragraph recovery guidance.")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("With Actions", action)
                .description("Modern upstream action-slot patterns: compact xs button plus inline badge action.")
                .code_rust_from_file_region(snippets::action::SOURCE, "example"),
            DocSection::new("Rich Title", rich_title)
                .description("Composable title content using `AlertTitle::new_children(...)`.")
                .test_id_prefix("ui-gallery-alert-rich-title")
                .code_rust_from_file_region(snippets::rich_title::SOURCE, "example"),
            DocSection::new("Interactive Links", interactive_links)
                .description("A Fret-specific text-link pattern: normal runs open safe URLs, while diagnostics still keep deterministic activation evidence.")
                .code_rust_from_file_region(snippets::interactive_links::SOURCE, "example"),
            DocSection::new("Custom Colors", custom_colors)
                .description("Custom chrome override for special emphasis.")
                .code_rust_from_file_region(snippets::custom_colors::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Alert layout under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and caveats.")
                .test_id_prefix("ui-gallery-alert-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-alert-component")]
}
