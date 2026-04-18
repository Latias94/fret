use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sonner as snippets;
use fret::AppComponentCx;

fn prose_block<const N: usize>(
    cx: &mut AppComponentCx<'_>,
    lines: [&'static str; N],
) -> AnyElement {
    ui::v_flex(move |cx| {
        lines
            .into_iter()
            .map(|line| doc_layout::muted_full_width(cx, line).into_element(cx))
            .collect::<Vec<_>>()
    })
    .gap(Space::N1)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub(super) fn preview_sonner(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx);
    let about = prose_block(
        cx,
        [
            "Sonner is an opinionated toast component by Emil Kowalski.",
            "Fret keeps Sonner as the shadcn-facing recipe surface while fret-ui-kit owns the toast pipeline underneath.",
        ],
    );
    let types = snippets::types::render(cx);
    let description = snippets::description::render(cx);
    let position = snippets::position::render(cx);
    let examples = prose_block(
        cx,
        [
            "The previews below mirror the upstream Examples group: Types, Description, and Position.",
        ],
    );
    let api_reference = prose_block(
        cx,
        [
            "Toaster::new() is the mount-once entry point and keeps the shadcn wrapper defaults.",
            "Sonner::global(app) plus the message helpers and ToastMessageOptions cover the docs-facing lane.",
        ],
    );
    let setup = snippets::setup::render(cx);
    let extras = snippets::extras::render(cx);
    let notes = doc_layout::notes_block([
        "Reference stack: shadcn Sonner docs, the default registry recipe, the demo/types examples, Radix Primitives Toast, and Base UI Toast.",
        "Docs path stays `Demo`, `About`, `Usage`, `Examples`, `Types`, `Description`, `Position`, and `API Reference`, with `Mounting (Fret)` and `Extras` kept as Fret-only follow-ups.",
        "The shadcn Sonner lane stays message-oriented today: a generic composable `children([...])` API is not warranted here, and this pass did not identify a missing `fret-ui` mechanism bug.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .max_w(Px(980.0))
        .no_shell()
        .description("Parity notes and source anchors.")
        .test_id_prefix("ui-gallery-sonner-notes");
    let about = DocSection::build(cx, "About", about)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sonner-about");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal mounted usage with one message toast.")
        .test_id_prefix("ui-gallery-sonner-usage")
        .code_rust_from_file_region(snippets::usage::DOCS_SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Top preview matching the docs page.")
        .test_id_prefix("ui-gallery-sonner-demo")
        .code_rust_from_file_region(snippets::demo::DOCS_SOURCE, "example");
    let examples = DocSection::build(cx, "Examples", examples)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sonner-examples");
    let types = DocSection::build(cx, "Types", types)
        .description("Default, status, and promise toast variants.")
        .test_id_prefix("ui-gallery-sonner-types")
        .code_rust_from_file_region(snippets::types::DOCS_SOURCE, "example");
    let description = DocSection::build(cx, "Description", description)
        .description("Toast with supporting copy, matching the docs example.")
        .test_id_prefix("ui-gallery-sonner-description")
        .code_rust_from_file_region(snippets::description::DOCS_SOURCE, "example");
    let position = DocSection::build(cx, "Position", position)
        .description(
            "Use position to move the toast placement; the gallery keeps the toaster local so placements stay deterministic.",
        )
        .test_id_prefix("ui-gallery-sonner-position")
        .code_rust_from_file_region(snippets::position::DOCS_SOURCE, "example");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .max_w(Px(980.0))
        .no_shell()
        .test_id_prefix("ui-gallery-sonner-api-reference");
    let setup = DocSection::build(cx, "Mounting (Fret)", setup)
        .description("Mount a Toaster once per window. The usage snippets below inline this too, but this is the smallest focused install surface.")
        .test_id_prefix("ui-gallery-sonner-mounting")
        .code_rust_from_file_region(snippets::setup::DOCS_SOURCE, "example");
    let extras = DocSection::build(cx, "Extras", extras)
        .description(
            "Fret-specific extras after docs parity examples: action/cancel + swipe-dismiss.",
        )
        .test_id_prefix("ui-gallery-sonner-extras")
        .code_rust_from_file_region(snippets::extras::DOCS_SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Docs path follows the shadcn Sonner page first, then keeps Fret-specific mounting and extras after that.",
        ),
        vec![
            demo,
            about,
            usage,
            examples,
            types,
            description,
            position,
            api_reference,
            setup,
            extras,
            notes,
        ],
    );
    let toaster = snippets::local_toaster(cx).into_element(cx);

    vec![body.test_id("ui-gallery-sonner").into_element(cx), toaster]
}
