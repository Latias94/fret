use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_shimmer_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::shimmer_demo::render(cx);
    let durations = snippets::shimmer_duration_demo::render(cx);
    let elements = snippets::shimmer_elements_demo::render(cx);

    let features = doc_layout::notes_block([
            "Animated shimmer sweep aligned with AI Elements Shimmer.",
            "Configurable `duration_secs` and `spread` (defaults: 2s / 2).",
            "Theme-aware: uses `muted-foreground` as the base text and `background` as the highlight.",
            "Supports both explicit `text_style` overrides and inherited subtree typography via `use_resolved_passive_text`.",
            "A11y role mapping via `role` (rough equivalent of upstream `as`).",
        ])
    .test_id("ui-gallery-ai-shimmer-features");

    let props = shimmer_props_table(cx).test_id("ui-gallery-ai-shimmer-props");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The Shimmer component provides an animated shimmer effect that sweeps across text, useful for loading states and streaming indicators.",
        ),
        vec![
            DocSection::build(cx, "Shimmer", demo)
                .test_id_prefix("ui-gallery-ai-shimmer-demo")
                .code_rust_from_file_region(snippets::shimmer_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features).description("Key behavior and API notes."),
            DocSection::build(cx, "Different Durations", durations)
                .test_id_prefix("ui-gallery-ai-shimmer-duration")
                .code_rust_from_file_region(snippets::shimmer_duration_demo::SOURCE, "example"),
            DocSection::build(cx, "Custom Elements", elements)
                .description(
                    "Default, explicit, inline, and inherited subtree-typography compositions.",
                )
                .test_id_prefix("ui-gallery-ai-shimmer-elements")
                .code_rust_from_file_region(snippets::shimmer_elements_demo::SOURCE, "example"),
            DocSection::build(cx, "Props", props)
                .description("Fret API surface for `fret_ui_ai::Shimmer`."),
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
                "text",
                "Arc<str>",
                "-",
                "Text content to apply the shimmer effect to.",
            ],
            [
                "duration_secs",
                "f32",
                "2.0",
                "Animation duration in seconds (frame-based; ~60fps).",
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
        ],
        true,
    )
}
