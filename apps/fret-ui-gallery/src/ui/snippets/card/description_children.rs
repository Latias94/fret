pub const SOURCE: &str = include_str!("description_children.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
use fret_icons::IconId;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rich_description_text() -> AttributedText {
    let text: Arc<str> = Arc::from("Updated 2 hours ago by platform design");
    let prefix = "Updated 2 hours ago by ";
    let emphasis = "platform design";

    let plain = TextSpan::new(prefix.len());

    let mut underlined = TextSpan::new(emphasis.len());
    underlined.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });

    AttributedText::new(text, Arc::<[TextSpan]>::from([plain, underlined]))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                    shadcn::card_title("Deploy status"),
                    shadcn::card_description_children(|cx| {
                        ui::children![
                            cx;
                            ui::h_flex(|cx| {
                                vec![
                                    icon::icon(cx, IconId::new_static("lucide.info")),
                                    cx.styled_text(rich_description_text()),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        ]
                    }),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::text(
                        "Use `card_description_children(...)` when supporting text needs attributed spans or a caller-owned inline composition root.",
                    )
                    .text_sm(),
                ]
            }),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-description-children")
}
// endregion: example
