use super::super::super::super::super::*;

pub(in crate::ui) fn preview_switch(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default)]
    struct SwitchModels {
        description: Option<Model<bool>>,
        choice_share: Option<Model<bool>>,
        choice_notifications: Option<Model<bool>>,
        invalid: Option<Model<bool>>,
        size_small: Option<Model<bool>>,
        size_default: Option<Model<bool>>,
        rtl: Option<Model<bool>>,
    }

    let (description, choice_share, choice_notifications, invalid, size_small, size_default, rtl) =
        cx.with_state(SwitchModels::default, |st| {
            (
                st.description.clone(),
                st.choice_share.clone(),
                st.choice_notifications.clone(),
                st.invalid.clone(),
                st.size_small.clone(),
                st.size_default.clone(),
                st.rtl.clone(),
            )
        });

    let (description, choice_share, choice_notifications, invalid, size_small, size_default, rtl) =
        match (
            description,
            choice_share,
            choice_notifications,
            invalid,
            size_small,
            size_default,
            rtl,
        ) {
            (
                Some(description),
                Some(choice_share),
                Some(choice_notifications),
                Some(invalid),
                Some(size_small),
                Some(size_default),
                Some(rtl),
            ) => (
                description,
                choice_share,
                choice_notifications,
                invalid,
                size_small,
                size_default,
                rtl,
            ),
            _ => {
                let description = cx.app.models_mut().insert(false);
                let choice_share = cx.app.models_mut().insert(false);
                let choice_notifications = cx.app.models_mut().insert(true);
                let invalid = cx.app.models_mut().insert(false);
                let size_small = cx.app.models_mut().insert(false);
                let size_default = cx.app.models_mut().insert(true);
                let rtl = cx.app.models_mut().insert(false);
                cx.with_state(SwitchModels::default, |st| {
                    st.description = Some(description.clone());
                    st.choice_share = Some(choice_share.clone());
                    st.choice_notifications = Some(choice_notifications.clone());
                    st.invalid = Some(invalid.clone());
                    st.size_small = Some(size_small.clone());
                    st.size_default = Some(size_default.clone());
                    st.rtl = Some(rtl.clone());
                });
                (
                    description,
                    choice_share,
                    choice_notifications,
                    invalid,
                    size_small,
                    size_default,
                    rtl,
                )
            }
        };

    let destructive = cx.with_theme(|theme| theme.color_token("destructive"));
    let theme = Theme::global(&*cx.app).clone();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let sizes = {
        let small = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new(size_small)
                        .a11y_label("Small switch")
                        .size(shadcn::SwitchSize::Sm)
                        .test_id("ui-gallery-switch-size-small")
                        .into_element(cx),
                    shadcn::Label::new("Small").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-sizes-sm");

        let default = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new(size_default)
                        .a11y_label("Default switch")
                        .test_id("ui-gallery-switch-size-default")
                        .into_element(cx),
                    shadcn::Label::new("Default").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-sizes-default");

        doc_layout::wrap_controls_row(cx, &theme, Space::N4, |_cx| vec![small, default])
            .test_id("ui-gallery-switch-sizes")
    };

    let airplane_mode = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new(model.clone())
                        .a11y_label("Airplane mode")
                        .test_id("ui-gallery-switch-airplane-toggle")
                        .into_element(cx),
                    shadcn::Label::new("Airplane Mode").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-airplane");

        centered(cx, row)
    };

    let bluetooth = {
        let blue = ColorRef::Color(CoreColor {
            r: 0.23,
            g: 0.51,
            b: 0.96,
            a: 1.0,
        });
        let style = shadcn::switch::SwitchStyle::default().track_background(
            fret_ui_kit::WidgetStateProperty::new(None)
                .when(fret_ui_kit::WidgetStates::SELECTED, Some(blue)),
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new_controllable(cx, None, true)
                        .a11y_label("Bluetooth")
                        .style(style)
                        .test_id("ui-gallery-switch-bluetooth-toggle")
                        .into_element(cx),
                    shadcn::Label::new("Bluetooth").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-bluetooth");

        centered(cx, row)
    };

    let label_card = {
        let blue = ColorRef::Color(CoreColor {
            r: 0.23,
            g: 0.51,
            b: 0.96,
            a: 1.0,
        });
        let style = shadcn::switch::SwitchStyle::default().track_background(
            fret_ui_kit::WidgetStateProperty::new(None)
                .when(fret_ui_kit::WidgetStates::SELECTED, Some(blue)),
        );

        let field = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(description)
                .a11y_label("Share across devices")
                .style(style)
                .test_id("ui-gallery-switch-label-card-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-label-card");

        centered(cx, field)
    };

    let extras_choice_cards = {
        let share = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(choice_share)
                .a11y_label("Share across devices")
                .test_id("ui-gallery-switch-choice-card-share")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx);

        let notifications = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                shadcn::FieldDescription::new(
                    "Receive notifications when focus mode is enabled or disabled.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(choice_notifications)
                .a11y_label("Enable notifications")
                .test_id("ui-gallery-switch-choice-card-notifications")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
            |_cx| vec![share, notifications],
        )
        .test_id("ui-gallery-switch-choice-card");

        centered(cx, group)
    };

    let disabled_section = {
        let row = shadcn::Field::new([
            shadcn::Switch::new(model.clone())
                .disabled(true)
                .a11y_label("Disabled switch")
                .test_id("ui-gallery-switch-disabled-toggle")
                .into_element(cx),
            shadcn::FieldLabel::new("Disabled").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .into_element(cx)
        .test_id("ui-gallery-switch-disabled");

        centered(cx, row)
    };

    let invalid_section = {
        let invalid_style = shadcn::switch::SwitchStyle::default().border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(destructive))),
        );

        let field = shadcn::Field::new([
            shadcn::FieldContent::new([
                ui::label(cx, "Accept terms and conditions")
                    .text_color(ColorRef::Color(destructive))
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "You must accept the terms and conditions to continue.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(invalid)
                .a11y_label("Accept terms and conditions")
                .style(invalid_style)
                .test_id("ui-gallery-switch-invalid-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-invalid");

        centered(cx, field)
    };

    let rtl_section = {
        doc_layout::rtl(cx, |cx| {
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Share across devices").into_element(cx),
                    shadcn::FieldDescription::new(
                        "Focus is shared across devices, and turns off when you leave the app.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Switch::new(rtl)
                    .a11y_label("Share across devices")
                    .test_id("ui-gallery-switch-rtl-toggle")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
            .into_element(cx)
        })
        .test_id("ui-gallery-switch-rtl")
    };

    let extras = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SwitchDemo).",
                ),
                extras_choice_cards,
                disabled_section,
                invalid_section,
            ]
        },
    )
    .test_id("ui-gallery-switch-extras");

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Switch demo (new-york-v4).",
            "Switch sizes are controlled via `SwitchSize` to match upstream `size=\"sm\" | \"default\"`.",
            "Use `SwitchStyle` (ADR 0220 override slots) for token-safe styling like checked track background changes.",
        ],
    );

    let rtl_centered = centered(cx, rtl_section);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Switch demo order: Sizes, Airplane Mode, Bluetooth, Label Card. Extras include invalid/disabled/RTL.",
        ),
        vec![
            DocSection::new("Sizes", sizes)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-sizes")
                .code(
                    "rust",
                    r#"shadcn::Switch::new(model)
    .size(shadcn::SwitchSize::Sm)
    .into_element(cx);"#,
                ),
            DocSection::new("Airplane Mode", airplane_mode)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-airplane")
                .code(
                    "rust",
                    r#"stack::hstack(cx, props, |cx| vec![
    shadcn::Switch::new(model).a11y_label("Airplane mode").into_element(cx),
    shadcn::Label::new("Airplane Mode").into_element(cx),
]);"#,
                ),
            DocSection::new("Bluetooth", bluetooth)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-bluetooth")
                .code(
                    "rust",
                    r#"let style = shadcn::switch::SwitchStyle::default().track_background(
    fret_ui_kit::WidgetStateProperty::new(None)
        .when(fret_ui_kit::WidgetStates::SELECTED, Some(ColorRef::Color(blue))),
);

shadcn::Switch::new_controllable(cx, None, true)
    .style(style)
    .into_element(cx);"#,
                ),
            DocSection::new("Label Card", label_card)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-switch-label-card")
                .code(
                    "rust",
                    r#"shadcn::Field::new([content, shadcn::Switch::new(model).into_element(cx)])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg).p(Space::N4))
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl_centered)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Switch::new(model).into_element(cx)
});"#,
                ),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-switch-extras")
                .code(
                    "rust",
                    r#"// Disabled
shadcn::Switch::new(model).disabled(true).into_element(cx);

// Invalid (style override)
shadcn::Switch::new(model).style(invalid_style).into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-switch")]
}
