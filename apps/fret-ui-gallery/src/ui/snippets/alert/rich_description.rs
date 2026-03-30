pub const SOURCE: &str = include_str!("rich_description.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn bullet_row(text: &'static str, test_id: &'static str) -> impl UiChild + use<> {
    ui::h_flex(move |cx| {
        vec![
            ui::text("•").font_medium().into_element(cx),
            ui::text_block(text).test_id(test_id).into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::alert(|cx| {
        ui::children![
            cx;
            icon::icon(
                cx,
                fret_icons::IconId::new_static("lucide.circle-alert"),
            ),
            shadcn::AlertTitle::new("Unable to process your payment.")
                .ui()
                .test_id("ui-gallery-alert-rich-description-card-title"),
            shadcn::AlertDescription::build(|cx, out| {
                out.push(
                    ui::text_block("Please verify your billing information and try again.")
                        .test_id("ui-gallery-alert-rich-description-card-paragraph")
                        .into_element(cx),
                );
                out.push(
                    ui::v_flex(|cx| {
                        vec![
                            bullet_row(
                                "Check your card details",
                                "ui-gallery-alert-rich-description-card-item-card",
                            )
                            .into_element(cx),
                            bullet_row(
                                "Ensure sufficient funds",
                                "ui-gallery-alert-rich-description-card-item-funds",
                            )
                            .into_element(cx),
                            bullet_row(
                                "Verify billing address",
                                "ui-gallery-alert-rich-description-card-item-address",
                            )
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .test_id("ui-gallery-alert-rich-description-card-list")
                    .into_element(cx),
                );
            }),
        ]
    })
    .variant(shadcn::AlertVariant::Destructive)
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-rich-description-card")
}
// endregion: example
