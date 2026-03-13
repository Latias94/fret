pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let description = {
        shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(
                shadcn::RadioGroupItem::new("default", "Default").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Default").into_element(cx),
                        shadcn::FieldDescription::new("Standard spacing for most use cases.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("comfortable", "Comfortable").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Comfortable").into_element(cx),
                        shadcn::FieldDescription::new("More space between elements.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("compact", "Compact").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Compact").into_element(cx),
                        shadcn::FieldDescription::new("Minimal spacing for dense layouts.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .into_element(cx)
            .test_id("ui-gallery-radio-group-description")
    };

    let fieldset = {
        let group = shadcn::RadioGroup::uncontrolled(Some("monthly"))
            .a11y_label("Subscription plan")
            .item(shadcn::RadioGroupItem::new(
                "monthly",
                "Monthly ($9.99/month)",
            ))
            .item(shadcn::RadioGroupItem::new(
                "yearly",
                "Yearly ($99.99/year)",
            ))
            .item(shadcn::RadioGroupItem::new(
                "lifetime",
                "Lifetime ($299.99)",
            ))
            .into_element(cx);

        shadcn::field_set(|cx| {
            ui::children![
                cx;
                shadcn::FieldLegend::new("Subscription Plan")
                    .variant(shadcn::FieldLegendVariant::Label),
                shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings."),
                group,
            ]
        })
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
    };

    let disabled = {
        shadcn::RadioGroup::uncontrolled(Some("option2"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(shadcn::RadioGroupItem::new("option1", "Disabled").disabled(true))
            .item(shadcn::RadioGroupItem::new("option2", "Option 2"))
            .item(shadcn::RadioGroupItem::new("option3", "Option 3"))
            .into_element(cx)
    };

    let invalid = {
        let destructive = cx.with_theme(|theme| theme.color_token("destructive"));

        let group = shadcn::RadioGroup::uncontrolled(Some("email"))
            .a11y_label("Notification Preferences")
            .refine_layout(LayoutRefinement::default().w_full())
            .item(
                shadcn::RadioGroupItem::new("email", "Email only")
                    .aria_invalid(true)
                    .child(
                        ui::label("Email only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("sms", "SMS only")
                    .aria_invalid(true)
                    .child(
                        ui::label("SMS only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("both", "Both Email & SMS")
                    .aria_invalid(true)
                    .child(
                        ui::label("Both Email & SMS")
                            .text_color(ColorRef::Color(destructive))
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
        .refine_layout(max_w_xs.clone())
        .into_element(cx)
    };

    let rtl = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("خيارات")
            .refine_layout(w_fit.clone())
            .item(
                shadcn::RadioGroupItem::new("default", "افتراضي").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("افتراضي").into_element(cx),
                        shadcn::FieldDescription::new("تباعد قياسي لمعظم حالات الاستخدام.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("comfortable", "مريح").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("مريح").into_element(cx),
                        shadcn::FieldDescription::new("مساحة أكبر بين العناصر.").into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("compact", "مضغوط").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("مضغوط").into_element(cx),
                        shadcn::FieldDescription::new("تباعد أدنى للتخطيطات الكثيفة.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .into_element(cx)
    })
    .test_id("ui-gallery-radio-group-rtl");

    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                "Extras are Fret-specific demos and regression gates (not part of upstream shadcn RadioGroupDemo).",
            )
            .into_element(cx),
            description,
            fieldset,
            disabled,
            invalid,
            rtl,
        ]
    })
    .gap(Space::N6)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-radio-group-extras")
}

// endregion: example
