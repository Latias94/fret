use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_shimmer_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::shimmer_demo::render(cx);
    let durations = snippets::shimmer_duration_demo::render(cx);
    let elements = snippets::shimmer_elements_demo::render(cx);
    let typography = snippets::shimmer_typography_demo::render(cx);

    let features = doc_layout::notes_block([
            "Smooth animated shimmer sweep driven by the frame clock.",
            "Configurable `duration` / `duration_secs` and `spread` (defaults: 2s / 2).",
            "Default paragraph-like text surface with call-site heading, inline, and block variants.",
            "Theme-aware: uses `muted-foreground` as the base text and `background` as the highlight band.",
            "Keeps base text and overlay text on the same resolved typography snapshot.",
        ])
    .test_id("ui-gallery-ai-shimmer-features");

    let notes = doc_layout::notes_block([
            "Mechanism health looks good: wrap / overflow / baseline parity is already locked in `fret-ui-ai::Shimmer`, and the existing UI Gallery diag gate passes on the current surface.",
            "The main drift was on the teaching surface, not the runtime contract: this page and its snippets now follow the official AI Elements docs structure more closely.",
            "Upstream `children` is string-only JSX content, not arbitrary composable children. Fret intentionally keeps `Shimmer::new(text)` as the honest authoring seam because the component must measure and repaint the same text payload.",
            "Fret keeps extra typography hooks (`text_style`, `use_resolved_passive_text`) in the component/policy layer so callers can align shimmer with inherited shadcn/AI typography without widening `crates/fret-ui`.",
            "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` demo surfaces.",
        ])
    .test_id("ui-gallery-ai-shimmer-notes");

    let props = shimmer_props_table(cx).test_id("ui-gallery-ai-shimmer-props");
    let usage_section = DocSection::build(cx, "Usage", usage)
        .description("Rust/Fret analogue of the official AI Elements shimmer preview.")
        .test_id_prefix("ui-gallery-ai-shimmer-demo")
        .code_rust_from_file_region(snippets::shimmer_demo::SOURCE, "example");
    let features_section =
        DocSection::build(cx, "Features", features).description("Key behavior and API notes.");
    let durations_section = DocSection::build(cx, "Different Durations", durations)
        .description("Docs-aligned duration variants from the official AI Elements page.")
        .test_id_prefix("ui-gallery-ai-shimmer-duration")
        .code_rust_from_file_region(snippets::shimmer_duration_demo::SOURCE, "example");
    let elements_section = DocSection::build(cx, "Custom Elements", elements)
        .description("Paragraph, heading, inline, and custom block variants aligned with the official examples.")
        .test_id_prefix("ui-gallery-ai-shimmer-elements")
        .code_rust_from_file_region(snippets::shimmer_elements_demo::SOURCE, "example");
    let typography_section = DocSection::build(cx, "Fret Typography Surface", typography)
        .description("Fret-only authoring surface for explicit and inherited typography control.")
        .test_id_prefix("ui-gallery-ai-shimmer-typography")
        .code_rust_from_file_region(snippets::shimmer_typography_demo::SOURCE, "example");
    let props_section = DocSection::build(cx, "Props", props)
        .description("Fret API surface for `fret_ui_ai::Shimmer`; upstream JSX `children` maps to `Shimmer::new(text)`.");
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Parity findings and authoring-surface decisions for Shimmer.");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned Shimmer coverage for AI Elements: base preview, duration variants, custom element variants, and a separate Fret typography surface.",
        ),
        vec![
            usage_section,
            features_section,
            durations_section,
            elements_section,
            typography_section,
            props_section,
            notes_section,
        ],
    );

    vec![body.into_element(cx)]
}

fn shimmer_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Prop", "Type", "Default", "Description"],
        [
            [
                "Shimmer::new(text)",
                "impl Into<Arc<str>>",
                "-",
                "Required text payload. This is the Fret analogue of upstream JSX `children`.",
            ],
            [
                "duration / duration_secs",
                "f32",
                "2.0",
                "Animation duration in seconds (`duration` is the docs-aligned alias).",
            ],
            [
                "spread",
                "f32",
                "2.0",
                "Spread multiplier for the highlight band (multiplied by text length).",
            ],
            [
                "text_style",
                "TextStyle",
                "None",
                "Explicitly override the typography used for both base text and highlight band.",
            ],
            [
                "use_resolved_passive_text",
                "flag",
                "false",
                "Resolve typography from inherited subtree text-style / foreground; explicit `text_style` still wins if provided.",
            ],
            [
                "wrap",
                "TextWrap",
                "None",
                "Text wrapping policy (defaults to no wrap).",
            ],
            [
                "role",
                "SemanticsRole",
                "Text",
                "Accessibility role (rough mapping of upstream `as`).",
            ],
            [
                "test_id",
                "impl Into<Arc<str>>",
                "None",
                "Stable diagnostics selector for UI Gallery and scripted repros.",
            ],
        ],
        true,
    )
}
