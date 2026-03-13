pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let destructive = cx.with_theme(|theme| theme.color_token("destructive"));

    let group = shadcn::RadioGroup::uncontrolled(Some("email"))
        .a11y_label("Notification Preferences")
        .refine_layout(LayoutRefinement::default().w_full())
        .item(
            shadcn::RadioGroupItem::new("email", "Email only")
                .aria_invalid(true)
                .child(
                    shadcn::Field::new([ui::label("Email only")
                        .font_normal()
                        .text_color(ColorRef::Color(destructive))
                        .w_full()
                        .min_w_0()
                        .into_element(cx)])
                    .invalid(true)
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ),
        )
        .item(
            shadcn::RadioGroupItem::new("sms", "SMS only")
                .aria_invalid(true)
                .child(
                    shadcn::Field::new([ui::label("SMS only")
                        .font_normal()
                        .text_color(ColorRef::Color(destructive))
                        .w_full()
                        .min_w_0()
                        .into_element(cx)])
                    .invalid(true)
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ),
        )
        .item(
            shadcn::RadioGroupItem::new("both", "Both Email & SMS")
                .aria_invalid(true)
                .child(
                    shadcn::Field::new([ui::label("Both Email & SMS")
                        .font_normal()
                        .text_color(ColorRef::Color(destructive))
                        .w_full()
                        .min_w_0()
                        .into_element(cx)])
                    .invalid(true)
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ),
        )
        .into_element(cx);

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Notification Preferences")
                .variant(shadcn::FieldLegendVariant::Label),
            shadcn::FieldDescription::new("Choose how you want to receive notifications."),
            group,
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-invalid")
}
// endregion: example
