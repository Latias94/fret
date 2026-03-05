use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret_ui_shadcn as shadcn;

pub(super) fn preview_ai_shimmer_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::shimmer_demo::render(cx);
    let durations = snippets::shimmer_duration_demo::render(cx);
    let elements = snippets::shimmer_elements_demo::render(cx);

    let features = doc_layout::notes(
        cx,
        [
            "Animated shimmer sweep aligned with AI Elements Shimmer.",
            "Configurable `duration_secs` and `spread` (defaults: 2s / 2).",
            "Theme-aware: uses `muted-foreground` as the base text and `background` as the highlight.",
            "Typography override via `text_style` for heading/inline variants.",
            "A11y role mapping via `role` (rough equivalent of upstream `as`).",
        ],
    )
    .test_id("ui-gallery-ai-shimmer-features");

    let props = shimmer_props_table(cx).test_id("ui-gallery-ai-shimmer-props");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The Shimmer component provides an animated shimmer effect that sweeps across text, useful for loading states and streaming indicators.",
        ),
        vec![
            DocSection::new("Shimmer", demo)
                .test_id_prefix("ui-gallery-ai-shimmer-demo")
                .code_rust_from_file_region(snippets::shimmer_demo::SOURCE, "example"),
            DocSection::new("Features", features).description("Key behavior and API notes."),
            DocSection::new("Different Durations", durations)
                .test_id_prefix("ui-gallery-ai-shimmer-duration")
                .code_rust_from_file_region(snippets::shimmer_duration_demo::SOURCE, "example"),
            DocSection::new("Custom Elements", elements)
                .test_id_prefix("ui-gallery-ai-shimmer-elements")
                .code_rust_from_file_region(snippets::shimmer_elements_demo::SOURCE, "example"),
            DocSection::new("Props", props)
                .description("Fret API surface for `fret_ui_ai::Shimmer`."),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-shimmer-demo")]
}

fn shimmer_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |cx: &mut ElementContext<'_, App>,
               prop: &'static str,
               ty: &'static str,
               default: &'static str,
               desc: &'static str| {
        shadcn::TableRow::new(
            4,
            vec![
                shadcn::TableCell::new(cx.text(prop)).into_element(cx),
                shadcn::TableCell::new(cx.text(ty)).into_element(cx),
                shadcn::TableCell::new(cx.text(default)).into_element(cx),
                shadcn::TableCell::new(cx.text(desc)).into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx)
    };

    shadcn::Table::new(vec![
        shadcn::TableHeader::new(vec![
            shadcn::TableRow::new(
                4,
                vec![
                    shadcn::TableHead::new("Prop").into_element(cx),
                    shadcn::TableHead::new("Type").into_element(cx),
                    shadcn::TableHead::new("Default").into_element(cx),
                    shadcn::TableHead::new("Description").into_element(cx),
                ],
            )
            .border_bottom(true)
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::TableBody::new(vec![
            row(
                cx,
                "text",
                "Arc<str>",
                "-",
                "Text content to apply the shimmer effect to.",
            ),
            row(
                cx,
                "duration_secs",
                "f32",
                "2.0",
                "Animation duration in seconds (frame-based; ~60fps).",
            ),
            row(
                cx,
                "spread",
                "f32",
                "2.0",
                "Spread multiplier for the highlight band (multiplied by text length).",
            ),
            row(
                cx,
                "text_style",
                "TextStyle",
                "None",
                "Override the typography used for both base text and highlight band.",
            ),
            row(
                cx,
                "wrap",
                "TextWrap",
                "None",
                "Text wrapping policy (defaults to no wrap).",
            ),
            row(
                cx,
                "role",
                "SemanticsRole",
                "Text",
                "Accessibility role (rough mapping of upstream `as`).",
            ),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
