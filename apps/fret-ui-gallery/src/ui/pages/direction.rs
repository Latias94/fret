use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::direction as snippets;

pub(super) fn preview_direction(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let use_direction = snippets::use_direction::render(cx);
    let composed_children = snippets::composed_children::render(cx);

    let api_reference = doc_layout::notes_block([
        "`DirectionProvider` is a policy-layer/provider surface in `fret-ui-shadcn`; the underlying direction resolution contract already lives in `fret-ui-kit::primitives::direction`.",
        "`DirectionProvider::new(dir)` is the concise Rust lane. `direction(...)` and `dir(...)` are upstream-shaped aliases when you want docs/source parity.",
        "`use_direction(cx, local)` matches the Radix rule `local || inherited || ltr` and is the Fret equivalent of upstream `useDirection()`.",
        "`DirectionProvider::with(...)` is the default multi-child authoring lane for provider-owned subtrees; `into_element(...)` remains the single-composed-child lane.",
        "This page closes a docs/public-surface gap. No `fret-ui` mechanism change was needed for `direction` itself.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary, ownership notes, and authoring-lane guidance.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Card-style RTL preview aligned with the upstream Direction docs teaching surface.",
        )
        .test_id_prefix("ui-gallery-direction-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Wrap a subtree once and let descendants resolve RTL automatically.")
        .test_id_prefix("ui-gallery-direction-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let use_direction = DocSection::build(cx, "use_direction", use_direction)
        .description("Resolve the effective direction from local override, inherited scope, and the LTR fallback.")
        .test_id_prefix("ui-gallery-direction-use-direction")
        .code_rust_from_file_region(snippets::use_direction::SOURCE, "example");
    let composed_children = DocSection::build(cx, "Composed Children", composed_children)
        .description("Fret-specific provider lane for composing multiple sibling children inside one direction scope.")
        .test_id_prefix("ui-gallery-direction-composed-children")
        .code_rust_from_file_region(snippets::composed_children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the upstream Direction docs order first (`Demo`, `Usage`, `useDirection`) and then appends a Fret-specific `Composed Children` authoring section.",
        ),
        vec![demo, usage, use_direction, composed_children, api_reference],
    );

    vec![body.test_id("ui-gallery-page-direction").into_element(cx)]
}
