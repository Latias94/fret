use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::collapsible as snippets;

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let controlled_state = snippets::controlled_state::render(cx);
    let builder = snippets::basic::render(cx);
    let settings = snippets::settings_panel::render(cx);
    let file_tree = snippets::file_tree::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API references: `ecosystem/fret-ui-shadcn/src/collapsible.rs` and `ecosystem/fret-ui-shadcn/src/collapsible_primitives.rs`.",
            "shadcn-style children composition is available via `fret_ui_shadcn::collapsible::primitives` (legacy path: `fret_ui_shadcn::collapsible_primitives`).",
            "Use controlled mode (`Model<bool>`) when outside state (URL/query, form mode, or saved layout) needs to drive disclosure.",
            "The top-level `fret_ui_shadcn::Collapsible` wrapper remains a compact Fret-first builder for dense editor UIs.",
            "Nested collapsibles in file trees should keep each node state independent and keyed for stable toggling.",
            "Always verify RTL with long trigger labels to ensure direction and alignment remain predictable.",
        ],
    )
    .test_id("ui-gallery-collapsible-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Demo and Usage mirror the upstream shadcn Collapsible docs; Builder API and the remaining sections document Fret-first authoring and regression scenarios.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Uncontrolled disclosure with a compact trigger and a details list.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composable usage for Collapsible via the nested primitives path.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Builder API", builder)
                .description("Fret-first ergonomic wrapper when you want trigger/content closures instead of free-form children composition.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Controlled State", controlled_state)
                .description("Controlled via `Model<bool>` when state must be driven externally.")
                .code_rust_from_file_region(snippets::controlled_state::SOURCE, "example"),
            DocSection::new("Settings Panel", settings)
                .description("Collapsible used to hide optional/advanced form fields.")
                .code_rust_from_file_region(snippets::settings_panel::SOURCE, "example"),
            DocSection::new("File Tree", file_tree)
                .description("Nested collapsibles with independent open state per node.")
                .code_rust_from_file_region(snippets::file_tree::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Direction provider should keep trigger/content alignment stable.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-collapsible-component")]
}
