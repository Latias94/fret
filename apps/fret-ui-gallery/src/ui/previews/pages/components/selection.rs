use super::super::super::super::*;

pub(in crate::ui) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::primitives::direction as direction_prim;

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let demo = cx.keyed("ui_gallery.radio_group.demo", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(shadcn::RadioGroupItem::new("default", "Default"))
            .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
            .item(shadcn::RadioGroupItem::new("compact", "Compact"))
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Demo", body)
    });

    let description = cx.keyed("ui_gallery.radio_group.description", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("comfortable"))
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
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Description", body)
    });

    let choice_card = cx.keyed("ui_gallery.radio_group.choice_card", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("plus"))
            .a11y_label("Subscription plans")
            .refine_layout(max_w_sm.clone())
            .item(
                shadcn::RadioGroupItem::new("plus", "Plus")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Plus").into_element(cx),
                            shadcn::FieldDescription::new("For individuals and small teams.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("pro", "Pro")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Pro").into_element(cx),
                            shadcn::FieldDescription::new("For growing businesses.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("enterprise", "Enterprise")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Enterprise").into_element(cx),
                            shadcn::FieldDescription::new("For large teams and enterprises.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Choice Card", body)
    });

    let fieldset = cx.keyed("ui_gallery.radio_group.fieldset", |cx| {
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

        let fieldset = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Subscription Plan")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings.")
                .into_element(cx),
            group,
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx);

        let body = centered(cx, fieldset);
        section(cx, "Fieldset", body)
    });

    let disabled = cx.keyed("ui_gallery.radio_group.disabled", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("option2"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(shadcn::RadioGroupItem::new("option1", "Disabled").disabled(true))
            .item(shadcn::RadioGroupItem::new("option2", "Option 2"))
            .item(shadcn::RadioGroupItem::new("option3", "Option 3"))
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Disabled", body)
    });

    let invalid = cx.keyed("ui_gallery.radio_group.invalid", |cx| {
        let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

        let group = shadcn::RadioGroup::uncontrolled(Some("email"))
            .a11y_label("Notification Preferences")
            .refine_layout(LayoutRefinement::default().w_full())
            .item(
                shadcn::RadioGroupItem::new("email", "Email only")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "Email only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("sms", "SMS only")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "SMS only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("both", "Both Email & SMS")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "Both Email & SMS")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .into_element(cx);

        let fieldset = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Notification Preferences")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Choose how you want to receive notifications.")
                .into_element(cx),
            group,
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx);

        let body = centered(cx, fieldset);
        section(cx, "Invalid", body)
    });

    let rtl = cx.keyed("ui_gallery.radio_group.rtl", |cx| {
        let group = direction_prim::with_direction_provider(
            cx,
            direction_prim::LayoutDirection::Rtl,
            |cx| {
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
                                shadcn::FieldDescription::new("مساحة أكبر بين العناصر.")
                                    .into_element(cx),
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
            },
        );

        let body = centered(cx, group);
        section(cx, "RTL", body)
    });

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![description, choice_card, fieldset, disabled, invalid, rtl],
    );

    vec![demo, examples]
}
