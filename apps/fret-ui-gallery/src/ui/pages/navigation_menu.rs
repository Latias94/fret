use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::navigation_menu as snippets;

pub(super) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let docs_demo = snippets::docs_demo::render(cx);
    let usage = snippets::usage::render(cx);
    let demo_with_toggle = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Navigation Menu already exposes a shadcn-friendly builder surface, so the main parity gap here is documentation clarity rather than mechanism coverage.",
            "Viewport and indicator are explicit builder options in Fret because they are composition outcomes in upstream shadcn, not separate runtime contracts.",
            "Container query toggle is a Fret-specific extra used to audit viewport-vs-container breakpoint behavior.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Navigation Menu docs order: Demo, Usage, RTL. Container query toggle remains a Fret-specific extra.",
        ),
        vec![
            DocSection::new("Demo", docs_demo)
                .description("Docs-aligned navigation menu demo with shared viewport behavior.")
                .code_rust_from_file_region(snippets::docs_demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style builder usage for Navigation Menu.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Container Query Toggle", demo_with_toggle)
                .description("Compare viewport-driven and container-driven md breakpoint behavior.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description(
                    "Navigation Menu should preserve placement and viewport alignment under RTL.",
                )
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-navigation-menu")]
}
