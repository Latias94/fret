use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::collapsible as snippets;

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let controlled_state = snippets::controlled_state::render(cx);
    let basic = snippets::basic::render(cx);
    let settings = snippets::settings_panel::render(cx);
    let file_tree = snippets::file_tree::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/collapsible.rs`.",
            "Use controlled mode (`Model<bool>`) when outside state (URL/query, form mode, or saved layout) needs to drive disclosure.",
            "For dense editor UIs, keep trigger chrome compact and put expensive children under `CollapsibleContent`.",
            "Nested collapsibles in file trees should keep each node state independent and keyed for stable toggling.",
            "Always verify RTL with long trigger labels to ensure direction and alignment remain predictable.",
        ],
    )
    .test_id("ui-gallery-collapsible-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Collapsible docs flow: Demo, Controlled State, Basic, Settings Panel, File Tree, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Uncontrolled disclosure with a compact trigger and a details list.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Controlled State", controlled_state)
                .description("Controlled via `Model<bool>` when state must be driven externally.")
                .code_rust_from_file_region(snippets::controlled_state::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Uncontrolled disclosure with a simple text content body.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
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
