use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::facade as shadcn;

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
            "Supports both explicit `text_style` overrides and inherited subtree typography via `use_resolved_passive_text`.",
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
                .description(
                    "Default, explicit, inline, and inherited subtree-typography compositions.",
                )
                .test_id_prefix("ui-gallery-ai-shimmer-elements")
                .code_rust_from_file_region(snippets::shimmer_elements_demo::SOURCE, "example"),
            DocSection::new("Props", props)
                .description("Fret API surface for `fret_ui_ai::Shimmer`."),
        ],
    );

    vec![body]
}

fn shimmer_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let row = |prop: &'static str, ty: &'static str, default: &'static str, desc: &'static str| {
        shadcn::TableRow::build(4, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(prop)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(ty)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(default)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(desc)));
        })
        .border_bottom(true)
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(4, |cx, out| {
                        out.push(shadcn::TableHead::new("Prop").into_element(cx));
                        out.push(shadcn::TableHead::new("Type").into_element(cx));
                        out.push(shadcn::TableHead::new("Default").into_element(cx));
                        out.push(shadcn::TableHead::new("Description").into_element(cx));
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("text", "Arc<str>", "-", "Text content to apply the shimmer effect to."));
                out.push_ui(cx, row("duration_secs", "f32", "2.0", "Animation duration in seconds (frame-based; ~60fps)."));
                out.push_ui(cx, row("spread", "f32", "2.0", "Spread multiplier for the highlight band (multiplied by text length)."));
                out.push_ui(cx, row("text_style", "TextStyle", "None", "Explicitly override the typography used for both base text and highlight band."));
                out.push_ui(cx, row("use_resolved_passive_text", "flag", "false", "Resolve typography from inherited subtree text-style / foreground; explicit `text_style` still wins if provided."));
                out.push_ui(cx, row("wrap", "TextWrap", "None", "Text wrapping policy (defaults to no wrap)."));
                out.push_ui(cx, row("role", "SemanticsRole", "Text", "Accessibility role (rough mapping of upstream `as`)."));
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
