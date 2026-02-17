use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let ratio_example = |cx: &mut ElementContext<'_, App>,
                         ratio: f32,
                         max_w: Px,
                         ratio_label: &'static str,
                         caption: &'static str,
                         test_id: &'static str| {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_center()
                .justify_center()
                .gap(Space::N1),
            move |cx| {
                vec![
                    shadcn::typography::h4(cx, ratio_label),
                    shadcn::typography::muted(cx, caption),
                ]
            },
        );

        let (muted_bg, border) =
            cx.with_theme(|theme| (theme.color_token("muted"), theme.color_token("border")));

        shadcn::AspectRatio::new(ratio, content)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .bg(ColorRef::Color(muted_bg))
                    .border_color(ColorRef::Color(border)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
            .into_element(cx)
            .test_id(test_id)
    };

    let demo_content = ratio_example(
        cx,
        16.0 / 9.0,
        Px(384.0),
        "16:9",
        "Landscape media",
        "ui-gallery-aspect-ratio-demo",
    );
    let demo = demo_content;

    let square_content = ratio_example(
        cx,
        1.0,
        Px(192.0),
        "1:1",
        "Square media",
        "ui-gallery-aspect-ratio-square",
    );
    let square = square_content;

    let portrait_content = ratio_example(
        cx,
        9.0 / 16.0,
        Px(160.0),
        "9:16",
        "Portrait media",
        "ui-gallery-aspect-ratio-portrait",
    );
    let portrait = portrait_content;

    let rtl_content = doc_layout::rtl(cx, |cx| {
        ratio_example(
            cx,
            16.0 / 9.0,
            Px(384.0),
            "16:9",
            "RTL layout sample",
            "ui-gallery-aspect-ratio-rtl",
        )
    });
    let rtl = rtl_content;

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`.",
            "Use `AspectRatio` to lock geometry first, then style radius/border/background around it.",
            "Pick ratio by content type: 16:9 for landscape previews, 1:1 for avatars/thumbnails, 9:16 for reels or short video cards.",
            "Keep max width explicit on narrow ratios to avoid over-tall layouts in dense editor sidebars.",
            "Validate RTL and constrained width together so captions and controls remain stable during localization.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Aspect Ratio docs order: Demo, Square, Portrait, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("16:9 landscape media frame with an inline caption.")
                .code(
                    "rust",
                    r#"let content = cx.text("...");
shadcn::AspectRatio::new(16.0 / 9.0, content)
    .refine_layout(LayoutRefinement::default().max_w(Px(384.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Square", square)
                .description("1:1 square media for avatars/thumbnails.")
                .code(
                    "rust",
                    r#"let content = cx.text("1:1");
shadcn::AspectRatio::new(1.0, content)
    .refine_layout(LayoutRefinement::default().max_w(Px(320.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Portrait", portrait)
                .description("9:16 portrait media for reels/short video cards.")
                .code(
                    "rust",
                    r#"let content = cx.text("9:16");
shadcn::AspectRatio::new(9.0 / 16.0, content)
    .refine_layout(LayoutRefinement::default().max_w(Px(240.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("AspectRatio should remain direction-agnostic.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        let content = cx.text("RTL layout sample");
        shadcn::AspectRatio::new(16.0 / 9.0, content).into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-aspect-ratio-component")]
}
