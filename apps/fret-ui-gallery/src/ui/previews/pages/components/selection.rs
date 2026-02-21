use super::super::super::super::*;

pub(in crate::ui) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let demo = cx
        .keyed("ui_gallery.radio_group.demo", |cx| {
            shadcn::RadioGroup::uncontrolled(Some("comfortable"))
                .a11y_label("Options")
                .refine_layout(w_fit.clone())
                .item(shadcn::RadioGroupItem::new("default", "Default"))
                .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
                .item(shadcn::RadioGroupItem::new("compact", "Compact"))
                .into_element(cx)
        })
        .test_id("ui-gallery-radio-group-demo");

    let plans = cx
        .keyed("ui_gallery.radio_group.plans", |cx| {
            shadcn::RadioGroup::uncontrolled(Some("starter"))
                .a11y_label("Plans")
                .refine_layout(max_w_sm.clone())
                .item(
                    shadcn::RadioGroupItem::new("starter", "Starter Plan")
                        .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                        .child(
                            shadcn::FieldContent::new([
                                shadcn::FieldTitle::new("Starter Plan").into_element(cx),
                                shadcn::FieldDescription::new(
                                    "Perfect for small businesses getting started with our platform",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                )
                .item(
                    shadcn::RadioGroupItem::new("pro", "Pro Plan")
                        .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                        .child(
                            shadcn::FieldContent::new([
                                shadcn::FieldTitle::new("Pro Plan").into_element(cx),
                                shadcn::FieldDescription::new(
                                    "Advanced features for growing businesses with higher demands",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                )
                .into_element(cx)
        })
        .test_id("ui-gallery-radio-group-plans");

    let extras = cx.keyed("ui_gallery.radio_group.extras", |cx| {
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
                .item(shadcn::RadioGroupItem::new("lifetime", "Lifetime ($299.99)"))
                .into_element(cx);

            shadcn::FieldSet::new([
                shadcn::FieldLegend::new("Subscription Plan")
                    .variant(shadcn::FieldLegendVariant::Label)
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "Yearly and lifetime plans offer significant savings.",
                )
                .into_element(cx),
                group,
            ])
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

            shadcn::FieldSet::new([
                shadcn::FieldLegend::new("Notification Preferences")
                    .variant(shadcn::FieldLegendVariant::Label)
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "Choose how you want to receive notifications.",
                )
                .into_element(cx),
                group,
            ])
            .refine_layout(max_w_xs.clone())
            .into_element(cx)
        };

        let rtl = {
            doc_layout::rtl(cx, |cx| {
                shadcn::RadioGroup::uncontrolled(Some("comfortable"))
                    .a11y_label("خيارات")
                    .refine_layout(w_fit.clone())
                    .item(
                        shadcn::RadioGroupItem::new("default", "افتراضي").child(
                            shadcn::FieldContent::new([
                                shadcn::FieldLabel::new("افتراضي").into_element(cx),
                                shadcn::FieldDescription::new(
                                    "تباعد قياسي لمعظم حالات الاستخدام.",
                                )
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
                                shadcn::FieldDescription::new(
                                    "تباعد أدنى للتخطيطات الكثيفة.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                    )
                    .into_element(cx)
            })
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "Extras are Fret-specific demos and regression gates (not part of upstream shadcn RadioGroupDemo).",
                    ),
                    description,
                    fieldset,
                    disabled,
                    invalid,
                    rtl,
                ]
            },
        )
        .test_id("ui-gallery-radio-group-extras")
    });

    let notes = doc_layout::notes(
        cx,
        ["Preview follows shadcn RadioGroup demo (new-york-v4)."],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn RadioGroup demo order: basic options, plan cards. Extras include invalid/fieldset/RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-radio-group-demo")
                .code(
                    "rust",
                    r#"shadcn::RadioGroup::uncontrolled(Some("comfortable"))
    .a11y_label("Options")
    .item(shadcn::RadioGroupItem::new("default", "Default"))
    .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
    .item(shadcn::RadioGroupItem::new("compact", "Compact"))
    .into_element(cx);"#,
                ),
            DocSection::new("Plans", plans)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-radio-group-plans")
                .code(
                    "rust",
                    r#"shadcn::RadioGroupItem::new("starter", "Starter Plan")
    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
    .child(shadcn::FieldContent::new([...]).into_element(cx));"#,
                ),
            DocSection::new("Extras", extras)
                .no_shell()
                .test_id_prefix("ui-gallery-radio-group-extras")
                .code(
                    "rust",
                    r#"// Invalid chrome + error text (app-level composition).
shadcn::RadioGroup::uncontrolled(Some("default"))
    .aria_invalid(true)
    .item(shadcn::RadioGroupItem::new("default", "Default"))
    .into_element(cx);

// RTL coverage.
doc_layout::rtl(cx, |cx| {
    shadcn::RadioGroup::uncontrolled(Some("default")).into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-radio-group-notes"),
        ],
    );

    vec![body]
}
