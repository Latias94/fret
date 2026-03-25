use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_schema_display_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::schema_display_demo::render(cx);
    let basic = snippets::schema_display_basic::render(cx);
    let params = snippets::schema_display_params::render(cx);
    let body = snippets::schema_display_body::render(cx);
    let nested = snippets::schema_display_nested::render(cx);
    let composable = snippets::schema_display_composable::render(cx);

    let features = doc_layout::notes_block([
        "Mechanism health looks good after the audit: the existing `gallery-dev` diag script passed, and the visible breakage traced back to `fret-ui-ai` recipe code rather than a `crates/fret-ui` runtime contract gap.",
        "This page now follows the official AI Elements docs surface more closely: one full usage example first, then focused examples for parameters, request/response bodies, and nested properties.",
        "The component now keeps root children stacked correctly, uses the upstream orange default for `PUT`, and exposes section-level `children([...])` overrides on `SchemaDisplayParameters`, `SchemaDisplayRequest`, and `SchemaDisplayResponse`.",
    ]);
    let method_colors = doc_layout::text_table(
        cx,
        ["Method", "Color"],
        [
            ["GET", "Green"],
            ["POST", "Blue"],
            ["PUT", "Orange"],
            ["PATCH", "Yellow"],
            ["DELETE", "Red"],
        ],
        false,
    );
    let notes = doc_layout::notes_block([
        "Remaining public-surface gap: Fret still does not provide an upstream-style context-driven `SchemaDisplayMethod()` / `SchemaDisplayPath()` lane, so fully custom root composition repeats method/path/description values.",
        "The main usage example keeps stable diagnostics anchors on the root plus the request/response property trees, so the existing screenshot and bundle gates continue to work under `gallery-dev`.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` surfaces in UI Gallery.",
    ]);

    let sections = vec![
        DocSection::build(cx, "Usage", usage)
            .description("Rust/Fret analogue of the official full Schema Display preview, including parameters plus request/response trees.")
            .test_id_prefix("ui-gallery-ai-schema-display-demo")
            .code_rust_from_file_region(snippets::schema_display_demo::SOURCE, "example"),
        DocSection::build(cx, "Features", features)
            .description("High-signal parity findings and what changed in this pass.")
            .no_shell(),
        DocSection::build(cx, "Method Colors", method_colors)
            .description("Default HTTP method colors aligned to the upstream AI Elements docs.")
            .no_shell(),
        DocSection::build(cx, "Basic Usage", basic)
            .description("Minimal endpoint summary, matching the official basic example.")
            .test_id_prefix("ui-gallery-ai-schema-display-basic")
            .code_rust_from_file_region(snippets::schema_display_basic::SOURCE, "example"),
        DocSection::build(cx, "With Parameters", params)
            .description("Path/query parameter display with highlighted path placeholders.")
            .test_id_prefix("ui-gallery-ai-schema-display-params")
            .code_rust_from_file_region(snippets::schema_display_params::SOURCE, "example"),
        DocSection::build(cx, "With Request/Response Bodies", body)
            .description("Focused request/response schema sections without the larger usage example payload.")
            .test_id_prefix("ui-gallery-ai-schema-display-body")
            .code_rust_from_file_region(snippets::schema_display_body::SOURCE, "example"),
        DocSection::build(cx, "Nested Properties", nested)
            .description("Recursive object-property rendering with shallow levels open by default.")
            .test_id_prefix("ui-gallery-ai-schema-display-nested")
            .code_rust_from_file_region(snippets::schema_display_nested::SOURCE, "example"),
        DocSection::build(cx, "Composable Children", composable)
            .description("Fret-specific custom composition lane: keep the shared chrome, then override only the section body you need.")
            .test_id_prefix("ui-gallery-ai-schema-display-composable")
            .code_rust_from_file_region(snippets::schema_display_composable::SOURCE, "example"),
        DocSection::build(cx, "Notes", notes)
            .description("What is aligned now, what remains explicit in Fret, and why.")
            .no_shell(),
    ];

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned SchemaDisplay coverage for AI Elements: full usage first, then focused examples, method-color defaults, and the current Fret composability trade-offs.",
        ),
        sections,
        cx,
    );

    vec![body.into_element(cx)]
}
