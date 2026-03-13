pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Something went wrong!"),
                    shadcn::AlertDescription::new(
                        "Your session has expired. Please log in again.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Destructive)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-destructive-session"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    fret_ui_shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Unable to process your payment."),
                    shadcn::AlertDescription::new_children([
                        ui::text("Please verify your billing information and try again.")
                            .into_element(cx),
                        ui::v_flex(|cx| {
                            vec![
                                ui::text("• Check your card details").into_element(cx),
                                ui::text("• Ensure sufficient funds").into_element(cx),
                                ui::text("• Verify billing address").into_element(cx),
                            ]
                        })
                        .gap(Space::N0p5)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                    ]),
                ]
            })
            .variant(shadcn::AlertVariant::Destructive)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-destructive-payment"),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-alert-destructive")
}
// endregion: example
