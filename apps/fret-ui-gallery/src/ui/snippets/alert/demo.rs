pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn bullet_row(text: &'static str) -> impl UiChild + use<> {
    ui::h_flex(move |cx| {
        vec![
            ui::text("•").font_medium().into_element(cx),
            ui::text_block(text).into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Success! Your changes have been saved"),
                    shadcn::AlertDescription::new(
                        "This is an alert with icon, title and description.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-success"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("This Alert has a title and an icon. No description."),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-title-only"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Unable to process your payment."),
                    shadcn::AlertDescription::build(|cx, out| {
                        out.push(
                            ui::text_block("Please verify your billing information and try again.")
                                .into_element(cx),
                        );
                        out.push(
                            ui::v_flex(|cx| {
                                vec![
                                    bullet_row("Check your card details").into_element(cx),
                                    bullet_row("Ensure sufficient funds").into_element(cx),
                                    bullet_row("Verify billing address").into_element(cx),
                                ]
                            })
                            .gap(Space::N1)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx),
                        );
                    }),
                ]
            })
            .variant(shadcn::AlertVariant::Destructive)
            .into_element(cx)
            .test_id("ui-gallery-alert-demo-payment-error"),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(576.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-alert-demo")
}
// endregion: example
