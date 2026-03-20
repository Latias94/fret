pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let full_name = cx.local_model_keyed("full_name", String::new);
    let username = cx.local_model_keyed("username", String::new);
    let newsletter = cx.local_model_keyed("newsletter", || false);

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Profile"),
            shadcn::FieldDescription::new("This appears on invoices and emails."),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Full name")
                            .for_control("ui-gallery-field-usage-full-name")
                            .into_element(cx),
                        shadcn::Input::new(full_name)
                            .control_id("ui-gallery-field-usage-full-name")
                            .placeholder("Evil Rabbit")
                            .a11y_label("Full name")
                            .into_element(cx),
                        shadcn::FieldDescription::new("This appears on invoices and emails.")
                            .for_control("ui-gallery-field-usage-full-name")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Username")
                            .for_control("ui-gallery-field-usage-username")
                            .into_element(cx),
                        shadcn::Input::new(username)
                            .control_id("ui-gallery-field-usage-username")
                            .a11y_label("Username")
                            .placeholder("evil-rabbit")
                            .aria_invalid(true)
                            .into_element(cx),
                        shadcn::FieldError::new("Choose another username.")
                            .for_control("ui-gallery-field-usage-username")
                            .into_element(cx),
                    ])
                    .invalid(true)
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::Switch::new(newsletter)
                            .control_id("ui-gallery-field-usage-newsletter")
                            .a11y_label("Subscribe to the newsletter")
                            .into_element(cx),
                        shadcn::FieldLabel::new("Subscribe to the newsletter")
                            .for_control("ui-gallery-field-usage-newsletter")
                            .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ]
            }),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-field-usage")
}
// endregion: example
