pub const SOURCE: &str = include_str!("title_children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
use fret_icons::IconId;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rich_title_text() -> AttributedText {
    let text: Arc<str> = Arc::from("Roadmap review: GPU text shaping");
    let prefix = "Roadmap review: ";
    let emphasis = "GPU text shaping";

    let plain = TextSpan::new(prefix.len());

    let mut underlined = TextSpan::new(emphasis.len());
    underlined.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });

    AttributedText::new(text, Arc::<[TextSpan]>::from([plain, underlined]))
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title_children(|cx| {
                        ui::children![
                            cx;
                            ui::h_flex(|cx| {
                                vec![
                                    icon::icon(cx, IconId::new_static("lucide.sparkles")),
                                    cx.styled_text(rich_title_text()),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        ]
                    }),
                    shadcn::card_description(
                        "Use `card_title_children(...)` when the title needs attributed text or a caller-owned inline composition root.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::text(
                        "The title keeps card typography while the caller still owns the exact child subtree.",
                    )
                    .text_sm(),
                ]
            }),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-title-children")
}
// endregion: example
