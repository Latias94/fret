use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::badge as snippets;

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let link = snippets::link::render(cx);

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
            DocSection::new("Link", link)
                .description("Badges can be composed with link semantics (shadcn `asChild`).")
                .code_rust_from_file_region(snippets::link::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-badge")]
}
