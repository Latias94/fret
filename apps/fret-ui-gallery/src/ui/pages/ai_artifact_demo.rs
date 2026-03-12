use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

fn render_notes(cx: &mut UiCx<'_>) -> AnyElement {
    fret_ui_kit::ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Artifact root/header/content defaults already line up with the upstream AI Elements chrome; the main drift was the UI Gallery example, not the underlying mechanism layer.",
            ),
            cx.text(
                "The docs-aligned example keeps Artifact chrome-only and leaves the embedded code view app-owned, which matches the intended layering in `fret-ui-ai`.",
            ),
            cx.text(
                "`ArtifactClose` now accepts custom children like upstream. Container parts were already composable; leaf text helpers remain intentionally text-first wrappers.",
            ),
        ]
    })
    .layout(fret_ui_kit::LayoutRefinement::default().w_full().min_w_0())
    .gap(fret_ui_kit::Space::N2)
    .items_start()
    .into_element(cx)
}

pub(super) fn preview_ai_artifact_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let mut sections = Vec::new();

    #[cfg(feature = "gallery-dev")]
    {
        sections.push(
            DocSection::new("With Code Display", snippets::artifact_code_display::render(cx))
                .description(
                    "Matches the official AI Elements preview structure: title + description on the left, action group on the right, and a code artifact body.",
                )
                .test_id_prefix("ui-gallery-ai-artifact-docs")
                .code_rust_from_file_region(snippets::artifact_code_display::SOURCE, "example"),
        );
    }

    sections.push(
        DocSection::new("Close Toggle", snippets::artifact_demo::render(cx))
            .description(
                "Keeps the existing close/reset interaction demo and diag gate for lifecycle behavior.",
            )
            .test_id_prefix("ui-gallery-ai-artifact-demo")
            .code_rust_from_file_region(snippets::artifact_demo::SOURCE, "example"),
    );

    sections.push(
        DocSection::new("Notes", render_notes(cx))
            .description("Parity findings and current API notes for Artifact."),
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Artifact examples using the same compound-parts composition model as the official AI Elements page.",
        ),
        sections,
    );

    vec![body.test_id("ui-gallery-page-ai-artifact-demo")]
}
