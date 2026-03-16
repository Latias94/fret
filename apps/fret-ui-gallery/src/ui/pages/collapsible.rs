use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::collapsible as snippets;

pub(super) fn preview_collapsible(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let controlled_state = snippets::controlled_state::render(cx);
    let basic = snippets::basic::render(cx);
    let settings = snippets::settings_panel::render(cx);
    let file_tree = snippets::file_tree::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Collapsible::new(Model<bool>)` and `Collapsible::uncontrolled(default_open)` cover the documented controlled and uncontrolled authoring paths.",
        "`fret_ui_shadcn::raw::collapsible::primitives::{Collapsible, CollapsibleTrigger, CollapsibleContent}` is the source-aligned children surface for free-form composition.",
        "The top-level `shadcn::Collapsible` wrapper stays a compact Fret-first builder for dense editor UIs, so no extra generic `compose()` API is needed here.",
        "Disclosure state, trigger semantics, and measured open/close motion remain recipe/primitive-owned; surrounding width, gap, and card/layout constraints remain caller-owned.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-collapsible-api-reference")
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Uncontrolled disclosure with a compact trigger and a details list.")
        .test_id_prefix("ui-gallery-collapsible-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable composable usage for `Collapsible` via the explicit raw primitives path.")
        .test_id_prefix("ui-gallery-collapsible-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let controlled_state = DocSection::build(cx, "Controlled State", controlled_state)
        .description("Controlled via `Model<bool>` when state must be driven externally.")
        .test_id_prefix("ui-gallery-collapsible-controlled")
        .code_rust_from_file_region(snippets::controlled_state::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Basic collapsible card outcome, authored through the compact Fret wrapper.")
        .test_id_prefix("ui-gallery-collapsible-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let settings = DocSection::build(cx, "Settings Panel", settings)
        .description("Use a trigger button to reveal additional settings.")
        .test_id_prefix("ui-gallery-collapsible-settings")
        .code_rust_from_file_region(snippets::settings_panel::SOURCE, "example");
    let file_tree = DocSection::build(cx, "File Tree", file_tree)
        .description("Use nested collapsibles to build a file tree.")
        .test_id_prefix("ui-gallery-collapsible-file-tree")
        .code_rust_from_file_region(snippets::file_tree::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider keeps trigger and content alignment stable under RTL.")
        .test_id_prefix("ui-gallery-collapsible-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Collapsible docs path first: Demo, Usage, Controlled State, Basic, Settings Panel, File Tree, RTL, and API Reference.",
        ),
        vec![
            demo,
            usage,
            controlled_state,
            basic,
            settings,
            file_tree,
            rtl,
            api_reference,
        ],
    );

    let body = body.test_id("ui-gallery-collapsible-component");
    vec![body.into_element(cx)]
}
