pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let email_id = "ui-gallery-radio-group-invalid-email";
    let sms_id = "ui-gallery-radio-group-invalid-sms";
    let both_id = "ui-gallery-radio-group-invalid-both";

    let group = shadcn::RadioGroup::uncontrolled(Some("email"))
        .a11y_label("Notification Preferences")
        .refine_layout(LayoutRefinement::default().w_full())
        .item(
            shadcn::RadioGroupItem::new("email", "Email only")
                .aria_invalid(true)
                .control_id(email_id),
        )
        .item(
            shadcn::RadioGroupItem::new("sms", "SMS only")
                .aria_invalid(true)
                .control_id(sms_id),
        )
        .item(
            shadcn::RadioGroupItem::new("both", "Both Email & SMS")
                .aria_invalid(true)
                .control_id(both_id),
        )
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "email"),
                    shadcn::FieldLabel::new("Email only")
                        .for_control(email_id)
                        .into_element(cx),
                ])
                .invalid(true)
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "sms"),
                    shadcn::FieldLabel::new("SMS only")
                        .for_control(sms_id)
                        .into_element(cx),
                ])
                .invalid(true)
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "both"),
                    shadcn::FieldLabel::new("Both Email & SMS")
                        .for_control(both_id)
                        .into_element(cx),
                ])
                .invalid(true)
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        });

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
