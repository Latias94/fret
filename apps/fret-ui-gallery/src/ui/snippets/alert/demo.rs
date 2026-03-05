pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Alert::new([
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-check")),
                    shadcn::AlertTitle::new("Success! Your changes have been saved")
                        .into_element(cx),
                    shadcn::AlertDescription::new(
                        "This is an alert with icon, title and description.",
                    )
                    .into_element(cx),
                ])
                .variant(shadcn::AlertVariant::Default)
                .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
                .into_element(cx)
                .test_id("ui-gallery-alert-demo-success"),
                shadcn::Alert::new([
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.popcorn")),
                    shadcn::AlertTitle::new("This Alert has a title and an icon. No description.")
                        .into_element(cx),
                ])
                .variant(shadcn::AlertVariant::Default)
                .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
                .into_element(cx)
                .test_id("ui-gallery-alert-demo-info"),
                shadcn::Alert::new([
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                    shadcn::AlertTitle::new("Unable to process your payment.").into_element(cx),
                    shadcn::AlertDescription::new_children([
                        ui::text("Please verify your billing information and try again.")
                            .wrap(TextWrap::Word)
                            .into_element(cx),
                        stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N0p5)
                                .items_start()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    ui::text("• Check your card number.")
                                        .text_sm()
                                        .into_element(cx),
                                    ui::text("• Check the expiration date.")
                                        .text_sm()
                                        .into_element(cx),
                                    ui::text("• Verify your billing address.")
                                        .text_sm()
                                        .into_element(cx),
                                ]
                            },
                        ),
                    ])
                    .into_element(cx),
                ])
                .variant(shadcn::AlertVariant::Destructive)
                .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
                .into_element(cx)
                .test_id("ui-gallery-alert-demo-destructive"),
            ]
        },
    )
    .test_id("ui-gallery-alert-demo")
}
// endregion: example
