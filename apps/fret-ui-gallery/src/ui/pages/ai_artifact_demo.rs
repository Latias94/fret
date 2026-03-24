use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Fret surface"],
        [
            [
                "Artifact",
                "Chrome-only root shell aligned with the official AI Elements container. Use `test_id_root(...)`, `refine_layout(...)`, and `refine_style(...)` for diagnostics and recipe overrides.",
            ],
            [
                "ArtifactHeader",
                "Header row with the same muted background + bottom border structure as upstream. Callers still own how title/description and action groups are arranged inside it.",
            ],
            [
                "ArtifactTitle / ArtifactDescription",
                "Paragraph-like text helpers. `new(text)` stays concise, while `new_children(...)` now supports upstream-style composable children without pushing this concern into `crates/fret-ui`.",
            ],
            [
                "ArtifactActions",
                "Small flex row for action affordances. Keeps the gap/default alignment from the official docs surface.",
            ],
            [
                "ArtifactAction",
                "Ghost icon button with optional tooltip, label, icon, `action(...)`, `action_payload(...)`, or `on_activate(...)` hooks.",
            ],
            [
                "ArtifactClose",
                "Close affordance with the upstream ghost-icon default plus optional custom children.",
            ],
            [
                "ArtifactContent",
                "Scrollable content viewport. Owns padding/scroll area chrome, while the embedded renderer (for example `CodeBlock`) stays app-owned.",
            ],
        ],
        false,
    )
}

fn render_features(_cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::notes_block([
        "Structured container with header and scrollable content areas, matching the official AI Elements `artifact` taxonomy.",
        "Header actions keep the upstream ghost-icon + tooltip pattern while staying action-first on the Rust surface.",
        "Artifact chrome remains recipe/policy-level; embedded rendering such as code blocks, documents, or export effects stays app-owned.",
        "Both `ArtifactTitle` and `ArtifactDescription` now support `new_children(...)`, so paragraph-like docs composition can stay in the component layer instead of leaking into runtime mechanisms.",
        "The UI Gallery page keeps the main preview aligned with the official `With Code Display` example and leaves lifecycle/close behavior in a separate focused section.",
    ])
    .test_id("ui-gallery-ai-artifact-features")
}

fn render_notes(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let _ = cx;
    doc_layout::notes_block([
        "Artifact root/header/content defaults already line up with the upstream AI Elements chrome; the main drift was the UI Gallery docs surface, not the underlying mechanism layer.",
        "The docs-aligned preview now keeps the visible `With Code Display` example faithful to the official page and moves diagnostics state into a non-visual marker.",
        "`ArtifactTitle`, `ArtifactDescription`, and `ArtifactClose` all support upstream-style custom children where that improves authoring, while the artifact body renderer remains intentionally app-owned.",
    ])
    .test_id("ui-gallery-ai-artifact-notes")
}

pub(super) fn preview_ai_artifact_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let mut sections = Vec::new();

    #[cfg(feature = "gallery-dev")]
    {
        // Source-policy marker: `DocSection::build(cx, "With Code Display", snippets::artifact_code_display::render(cx))`
        let artifact_code_display = snippets::artifact_code_display::render(cx);
        sections.push(
            DocSection::build(cx, "With Code Display", artifact_code_display)
                .description(
                    "Matches the official AI Elements preview structure: title + description on the left, action group on the right, and a code artifact body.",
                )
                .test_id_prefix("ui-gallery-ai-artifact-docs")
                .code_rust_from_file_region(snippets::artifact_code_display::SOURCE, "example"),
        );
    }

    let artifact_demo = snippets::artifact_demo::render(cx);
    // Source-policy marker: `DocSection::build(cx, "Close Toggle", snippets::artifact_demo::render(cx))`
    sections.push(
        DocSection::build(cx, "Close Toggle", artifact_demo)
            .description(
                "Keeps the existing close/reset interaction demo and diag gate for lifecycle behavior.",
            )
            .test_id_prefix("ui-gallery-ai-artifact-demo")
            .code_rust_from_file_region(snippets::artifact_demo::SOURCE, "example"),
    );

    let features = render_features(cx);
    sections.push(
        DocSection::build(cx, "Features", features)
            .description("High-signal parity notes against the official AI Elements Artifact docs.")
            .no_shell(),
    );

    let parts = parts_table(cx);
    sections.push(
        DocSection::build(cx, "Parts & Props", parts)
            .description("Mapping from the official compound parts to the current Fret surface.")
            .no_shell(),
    );

    let notes = render_notes(cx);
    // Source-policy marker: `DocSection::build(cx, "Notes", render_notes(cx))`
    sections.push(
        DocSection::build(cx, "Notes", notes)
            .description("Parity findings and current API notes for Artifact."),
    );

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned Artifact coverage for AI Elements: the official code-display example first, then the focused close interaction demo, features, and the current Rust surface map.",
        ),
        sections,
        cx,
    );

    vec![body.into_element(cx)]
}
