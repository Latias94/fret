use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::collapsible as snippets;

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let controlled_state = snippets::controlled_state::render(cx);
    let basic = snippets::basic::render(cx);
    let settings = snippets::settings_panel::render(cx);
    let file_tree = snippets::file_tree::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Collapsible::new(Model<bool>)` and `Collapsible::uncontrolled(default_open)` cover the documented controlled and uncontrolled authoring paths.",
            "`fret_ui_shadcn::collapsible::primitives::{Collapsible, CollapsibleTrigger, CollapsibleContent}` is the source-aligned children surface for free-form composition.",
            "The top-level `fret_ui_shadcn::Collapsible` wrapper stays a compact Fret-first builder for dense editor UIs, so no extra generic `compose()` API is needed here.",
            "Disclosure state, trigger semantics, and measured open/close motion remain recipe/primitive-owned; surrounding width, gap, and card/layout constraints remain caller-owned.",
            "This page is docs/public-surface parity work, not a mechanism-layer fix.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Collapsible docs path first: Demo, Usage, Controlled State, Basic, Settings Panel, File Tree, RTL, and API Reference.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Uncontrolled disclosure with a compact trigger and a details list.")
                .test_id_prefix("ui-gallery-collapsible-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable composable usage for `Collapsible` via the primitives path.")
                .test_id_prefix("ui-gallery-collapsible-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Controlled State", controlled_state)
                .description("Controlled via `Model<bool>` when state must be driven externally.")
                .test_id_prefix("ui-gallery-collapsible-controlled")
                .code_rust_from_file_region(snippets::controlled_state::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description(
                    "Basic collapsible card outcome, authored through the compact Fret wrapper.",
                )
                .test_id_prefix("ui-gallery-collapsible-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Settings Panel", settings)
                .description("Use a trigger button to reveal additional settings.")
                .test_id_prefix("ui-gallery-collapsible-settings")
                .code_rust_from_file_region(snippets::settings_panel::SOURCE, "example"),
            DocSection::new("File Tree", file_tree)
                .description("Use nested collapsibles to build a file tree.")
                .test_id_prefix("ui-gallery-collapsible-file-tree")
                .code_rust_from_file_region(snippets::file_tree::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description(
                    "Direction provider keeps trigger and content alignment stable under RTL.",
                )
                .test_id_prefix("ui-gallery-collapsible-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-collapsible-api-reference")
                .description("Public surface summary and ownership notes."),
        ],
    );

    vec![body.test_id("ui-gallery-collapsible-component")]
}
