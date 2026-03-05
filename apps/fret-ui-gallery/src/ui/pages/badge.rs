use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::badge as snippets;

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let variants = snippets::variants::render(cx);
    let with_icon = snippets::icon::render(cx);
    let with_spinner = snippets::spinner::render(cx);
    let link = snippets::link::render(cx);
    let colors = snippets::colors::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Badge is a small status/label primitive; prefer concise text and keep contrast high.",
            "API reference: `ecosystem/fret-ui-shadcn/src/badge.rs`.",
            "Note: the Link render example installs a no-op `on_activate` so diag scripts do not launch a system browser; remove it to enable the default `Effect::OpenUrl` fallback.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Badge docs (demo + asChild-style link render)."),
        vec![
            DocSection::new("Demo", demo)
                .description("Default shadcn badge variants and common compositions.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Variants", variants)
                .description("Use the `variant` prop to change the badge variant.")
                .code_rust_from_file_region(snippets::variants::SOURCE, "example"),
            DocSection::new("With Icon", with_icon)
                .description(
                    "Render an icon inside the badge (inline-start / inline-end patterns).",
                )
                .code_rust_from_file_region(snippets::icon::SOURCE, "example"),
            DocSection::new("With Spinner", with_spinner)
                .description("Render a spinner inside the badge (useful for loading states).")
                .code_rust_from_file_region(snippets::spinner::SOURCE, "example"),
            DocSection::new("Link", link)
                .description("Badges can be composed with link semantics (shadcn `asChild`).")
                .code_rust_from_file_region(snippets::link::SOURCE, "example"),
            DocSection::new("Custom Colors", colors)
                .description("Customize badge colors with explicit background + text overrides.")
                .code_rust_from_file_region(snippets::colors::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Render the badge under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body]
}
