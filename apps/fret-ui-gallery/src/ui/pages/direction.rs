use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::direction as snippets;

pub(super) fn preview_direction(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let use_direction = snippets::use_direction::render(cx);
    let composed_children = snippets::composed_children::render(cx);

    let api_reference = doc_layout::notes_block([
        "`DirectionProvider` is a policy-layer/provider surface in `fret-ui-shadcn`; the underlying direction resolution contract already lives in `fret-ui-kit::primitives::direction`.",
        "`DirectionProvider::new(dir)` is the concise Rust lane. `direction(...)` and `dir(...)` are upstream-shaped aliases when you want docs/source parity.",
        "Use `into_element(...)` for the default single-subtree lane that mirrors the upstream docs. Keep `with(...)` for the explicit provider-owned siblings lane when you want to avoid an extra wrapper element.",
        "`use_direction(cx, local)` matches the Radix rule `local || inherited || ltr` and is the Fret equivalent of upstream `useDirection()`.",
        "For app-wide direction, `fret-bootstrap` can install a root `LayoutDirection` global once; `DirectionProvider` remains the local subtree override, analogous to the web docs separating host `dir` from the provider surface.",
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
        .description("Single-subtree copyable lane aligned with the upstream Direction docs usage.")
        .test_id_prefix("ui-gallery-direction-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let use_direction = DocSection::build(cx, "useDirection", use_direction)
        .description("Resolve the effective direction from local override, inherited scope, and the LTR fallback.")
        .test_id_prefix("ui-gallery-direction-use-direction")
        .code_rust_from_file_region(snippets::use_direction::SOURCE, "example");
    let composed_children = DocSection::build(cx, "Composable Children (Fret)", composed_children)
        .description("Fret follow-up: use `with(...)` when one direction scope should own multiple sibling children without an extra wrapper.")
        .test_id_prefix("ui-gallery-direction-composed-children")
        .code_rust_from_file_region(snippets::composed_children::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/Base UI Direction docs path after skipping `Installation`: `Demo`, `Usage`, `useDirection`, and `API Reference`. `Composable Children (Fret)` stays as the explicit provider-owned siblings follow-up.",
        ),
        vec![demo, usage, use_direction, api_reference, composed_children],
    );

    vec![body.into_element(cx)]
}
