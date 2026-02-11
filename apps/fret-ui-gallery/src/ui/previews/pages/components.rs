use super::super::super::*;

pub(in crate::ui) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let outline_fg = ColorRef::Color(theme.color_required("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));
    let muted_fg = ColorRef::Color(theme.color_required("muted-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    let content_text = |cx: &mut ElementContext<'_, App>, text: &'static str, fg: ColorRef| {
        ui::text(cx, text)
            .font_medium()
            .nowrap()
            .text_color(fg)
            .into_element(cx)
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let size = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Small")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconSm)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Default")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Icon)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Large")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Lg)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconLg)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Size", body)
    };

    let default_body = shadcn::Button::new("Button").into_element(cx);
    let default = section(cx, "Default", default_body);

    let outline_body = shadcn::Button::new("Outline")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx);
    let outline = section(cx, "Outline", outline_body);

    let secondary_body = shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .into_element(cx);
    let secondary = section(cx, "Secondary", secondary_body);

    let ghost_body = shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .into_element(cx);
    let ghost = section(cx, "Ghost", ghost_body);

    let destructive_body = shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .into_element(cx);
    let destructive = section(cx, "Destructive", destructive_body);

    let link_body = shadcn::Button::new("Link")
        .variant(shadcn::ButtonVariant::Link)
        .into_element(cx);
    let link = section(cx, "Link", link_body);

    let icon_only_body = shadcn::Button::new("Submit")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
        .into_element(cx);
    let icon_only = section(cx, "Icon", icon_only_body);

    let with_icon = {
        let body = shadcn::Button::new("New Branch")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .children([
                icon(cx, "lucide.git-branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-icon"),
                content_text(cx, "New Branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-label"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-button-with-icon");
        section(cx, "With Icon", body)
    };

    let rounded_body = shadcn::Button::new("Scroll to top")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up", outline_fg.clone())])
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .into_element(cx);
    let rounded = section(cx, "Rounded", rounded_body);

    let spinner = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Generating")
                        .variant(shadcn::ButtonVariant::Outline)
                        .disabled(true)
                        .children([
                            shadcn::Spinner::new()
                                .color(outline_fg.clone())
                                .into_element(cx),
                            content_text(cx, "Generating", outline_fg.clone()),
                        ])
                        .into_element(cx),
                    shadcn::Button::new("Downloading")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(true)
                        .children([
                            content_text(cx, "Downloading", secondary_fg.clone()),
                            shadcn::Spinner::new()
                                .color(secondary_fg.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Spinner", body)
    };

    let button_group = {
        let demo = preview_button_group(cx)
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("ButtonGroup demo is missing"));
        section(cx, "Button Group", demo)
    };

    let render_link = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |cx| {
                vec![
                    shadcn::Button::new("Documentation")
                        .variant(shadcn::ButtonVariant::Outline)
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                    ui::text(cx, "TODO: `Button::render` / `asChild` composition is not implemented yet in fret-ui-shadcn. For now, use `variant=Link` or a dedicated link component.")
                        .text_color(muted_fg.clone())
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Link (render)", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Button::new("التالي")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                            shadcn::Button::new("السابق")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                        ]
                    },
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_start(),
        |_cx| {
            vec![
                size,
                default,
                outline,
                secondary,
                ghost,
                destructive,
                link,
                icon_only,
                with_icon,
                rounded,
                spinner,
                button_group,
                render_link,
                rtl,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_alert(cx)
}

pub(in crate::ui) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_shadcn_extras(cx)
}

pub(in crate::ui) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_checkbox(cx, model)
}

pub(in crate::ui) fn preview_switch(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
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

    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

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

    let demo = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new(model.clone())
                        .a11y_label("Airplane mode")
                        .test_id("ui-gallery-switch-demo-toggle")
                        .into_element(cx),
                    shadcn::Label::new("Airplane Mode").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-demo");
        let body = centered(cx, row);
        section(cx, "Demo", body)
    };

    let description_section = {
        let field = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(description)
                .a11y_label("Share across devices")
                .test_id("ui-gallery-switch-description-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-description");

        let body = centered(cx, field);
        section(cx, "Description", body)
    };

    let choice_card = {
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
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |_cx| vec![share, notifications],
        )
        .test_id("ui-gallery-switch-choice-card");

        let body = centered(cx, group);
        section(cx, "Choice Card", body)
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

        let body = centered(cx, row);
        section(cx, "Disabled", body)
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

        let body = centered(cx, field);
        section(cx, "Invalid", body)
    };

    let size_section = {
        let small = shadcn::Field::new([
            shadcn::Switch::new(size_small)
                .a11y_label("Small switch")
                .refine_layout(LayoutRefinement::default().w_px(Px(28.0)).h_px(Px(16.0)))
                .test_id("ui-gallery-switch-size-small")
                .into_element(cx),
            shadcn::FieldLabel::new("Small").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx);

        let default = shadcn::Field::new([
            shadcn::Switch::new(size_default)
                .a11y_label("Default switch")
                .test_id("ui-gallery-switch-size-default")
                .into_element(cx),
            shadcn::FieldLabel::new("Default").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full().max_w(Px(160.0))),
            |_cx| vec![small, default],
        )
        .test_id("ui-gallery-switch-size");

        let body = centered(cx, group);
        section(cx, "Size", body)
    };

    let rtl_section = {
        let rtl_field = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
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
            },
        )
        .test_id("ui-gallery-switch-rtl");

        let body = centered(cx, rtl_field);
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                description_section,
                choice_card,
                disabled_section,
                invalid_section,
                size_section,
                rtl_section,
            ]
        },
    );

    let note = shadcn::typography::muted(
        cx,
        "Note: size/invalid are approximated with layout/style overrides because this Switch API has no dedicated size/aria-invalid props."
            .to_string(),
    );

    vec![demo, examples, note]
}

pub(in crate::ui) fn preview_input(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_input(cx, value)
}

pub(in crate::ui) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TextareaModels {
        field: Option<Model<String>>,
        disabled: Option<Model<String>>,
        invalid: Option<Model<String>>,
        button: Option<Model<String>>,
        rtl: Option<Model<String>>,
    }

    let state = cx.with_state(TextareaModels::default, |st| st.clone());
    let (field_value, disabled_value, invalid_value, button_value, rtl_value) = match (
        state.field,
        state.disabled,
        state.invalid,
        state.button,
        state.rtl,
    ) {
        (
            Some(field_value),
            Some(disabled_value),
            Some(invalid_value),
            Some(button_value),
            Some(rtl_value),
        ) => (
            field_value,
            disabled_value,
            invalid_value,
            button_value,
            rtl_value,
        ),
        _ => {
            let field_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::new());
            let invalid_value = cx.app.models_mut().insert(String::new());
            let button_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());
            cx.with_state(TextareaModels::default, |st| {
                st.field = Some(field_value.clone());
                st.disabled = Some(disabled_value.clone());
                st.invalid = Some(invalid_value.clone());
                st.button = Some(button_value.clone());
                st.rtl = Some(rtl_value.clone());
            });
            (
                field_value,
                disabled_value,
                invalid_value,
                button_value,
                rtl_value,
            )
        }
    };

    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

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

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(420.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let area_layout = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let area = shadcn::Textarea::new(value)
            .a11y_label("Message")
            .min_height(Px(96.0))
            .refine_layout(area_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-textarea-demo");

        let body = centered(cx, area);
        section(cx, "Demo", body)
    };

    let field = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::FieldDescription::new("Enter your message below.").into_element(cx),
            shadcn::Textarea::new(field_value)
                .a11y_label("Message field")
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-field");

        let body = centered(cx, field);
        section(cx, "Field", body)
    };

    let disabled = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::Textarea::new(disabled_value)
                .a11y_label("Disabled message")
                .disabled(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-disabled");

        let body = centered(cx, field);
        section(cx, "Disabled", body)
    };

    let invalid = {
        let field = shadcn::Field::new([
            ui::label(cx, "Message")
                .text_color(ColorRef::Color(destructive))
                .into_element(cx),
            shadcn::Textarea::new(invalid_value)
                .a11y_label("Invalid message")
                .aria_invalid(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
            shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-invalid");

        let body = centered(cx, field);
        section(cx, "Invalid", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Textarea::new(button_value)
                        .a11y_label("Send message")
                        .min_height(Px(96.0))
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Button::new("Send message").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-textarea-button");

        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let rtl = {
        let rtl_field = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Feedback").into_element(cx),
                    shadcn::Textarea::new(rtl_value)
                        .a11y_label("Feedback")
                        .min_height(Px(96.0))
                        .refine_layout(area_layout.clone())
                        .into_element(cx),
                    shadcn::FieldDescription::new("Share your thoughts about our service.")
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-textarea-rtl");

        let rtl_shell = shell(cx, rtl_field);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Displays a form textarea or a component that looks like a textarea."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, field, disabled, invalid, button, rtl],
        ),
    ]
}

pub(in crate::ui) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_label(cx)
}

pub(in crate::ui) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_kbd(cx)
}
pub(in crate::ui) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let top = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            |cx| {
                vec![
                    shadcn::typography::small(cx, "Radix Primitives"),
                    shadcn::typography::muted(cx, "An open-source UI component library."),
                ]
            },
        );

        let links = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().w_full().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    top,
                    shadcn::Separator::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    links,
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-demo"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let vertical = {
        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-vertical"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Vertical", body)
    };

    let menu = {
        let menu_item =
            |cx: &mut ElementContext<'_, App>, title: &'static str, desc: &'static str| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    move |cx| {
                        vec![
                            shadcn::typography::small(cx, title),
                            shadcn::typography::muted(cx, desc),
                        ]
                    },
                )
            };

        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_center()
                .layout(LayoutRefinement::default().w_full().max_w(Px(560.0))),
            |cx| {
                vec![
                    menu_item(cx, "Settings", "Manage preferences"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Account", "Profile & security"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Help", "Support & docs"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-menu"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Menu", body)
    };

    let list = {
        let row = |cx: &mut ElementContext<'_, App>, key: &'static str, value: &'static str| {
            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .justify_between()
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| vec![cx.text(key), shadcn::typography::muted(cx, value)],
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    row(cx, "Item 1", "Value 1"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 2", "Value 2"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 3", "Value 3"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-list"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "List", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
                    |cx| {
                        vec![
                            stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N1).items_start(),
                                |cx| {
                                    vec![
                                        shadcn::typography::small(cx, "shadcn/ui"),
                                        shadcn::typography::muted(cx, "أساس نظام التصميم الخاص بك"),
                                    ]
                                },
                            ),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "مجموعة مكونات مصممة بشكل جميل يمكنك تخصيصها وتوسيعها.",
                            ),
                        ]
                    },
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Visually or semantically separates content."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, vertical, menu, list, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct SpinnerModels {
        input_value: Option<Model<String>>,
        textarea_value: Option<Model<String>>,
    }

    let (input_value, textarea_value) = cx.with_state(SpinnerModels::default, |st| {
        (st.input_value.clone(), st.textarea_value.clone())
    });
    let (input_value, textarea_value) = match (input_value, textarea_value) {
        (Some(input_value), Some(textarea_value)) => (input_value, textarea_value),
        _ => {
            let input_value = cx.app.models_mut().insert(String::new());
            let textarea_value = cx.app.models_mut().insert(String::new());
            cx.with_state(SpinnerModels::default, |st| {
                st.input_value = Some(input_value.clone());
                st.textarea_value = Some(textarea_value.clone());
            });
            (input_value, textarea_value)
        }
    };

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

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let item = shadcn::Item::new([
            shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Processing payment...").into_element(cx)
            ])
            .into_element(cx),
            shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Muted)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-demo");
        let body = centered(cx, item);
        section(cx, "Demo", body)
    };

    let custom = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new().into_element(cx),
                    shadcn::Spinner::new()
                        .icon(fret_icons::ids::ui::SETTINGS)
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-custom");
        let body = centered(cx, row);
        section(cx, "Customization", body)
    };

    let size = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(12.0)).h_px(Px(12.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(16.0)).h_px(Px(16.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-size");
        let body = centered(cx, row);
        section(cx, "Size", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Loading...")
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Please wait")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Processing")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-button");
        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let badge = {
        let (secondary_fg, outline_fg) = cx.with_theme(|theme| {
            (
                ColorRef::Color(theme.color_required("secondary-foreground")),
                ColorRef::Color(theme.color_required("foreground")),
            )
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Badge::new("Syncing")
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Updating")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([shadcn::Spinner::new()
                            .color(secondary_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Processing")
                        .variant(shadcn::BadgeVariant::Outline)
                        .children([shadcn::Spinner::new()
                            .color(outline_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-badge");
        let body = centered(cx, row);
        section(cx, "Badge", body)
    };

    let input_group = {
        let input = shadcn::InputGroup::new(input_value)
            .a11y_label("Send a message")
            .trailing([shadcn::Spinner::new().into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let textarea = shadcn::InputGroup::new(textarea_value)
            .textarea()
            .a11y_label("Send a message textarea")
            .block_end([stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        shadcn::Spinner::new().into_element(cx),
                        shadcn::typography::muted(cx, "Validating..."),
                        shadcn::InputGroupButton::new("")
                            .size(shadcn::InputGroupButtonSize::IconSm)
                            .children([shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.arrow-up"),
                            )])
                            .into_element(cx),
                    ]
                },
            )])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![input, textarea],
        );

        let card = shell(
            cx,
            LayoutRefinement::default().w_full().max_w(Px(480.0)),
            group,
        )
        .test_id("ui-gallery-spinner-input-group");

        let body = centered(cx, card);
        section(cx, "Input Group", body)
    };

    let empty = {
        let card = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("Processing your request").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "Please wait while we process your request. Do not refresh the page.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-empty");

        let body = centered(cx, card);
        section(cx, "Empty", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Item::new([
                    shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::ItemContent::new([
                        shadcn::ItemTitle::new("Processing payment...").into_element(cx)
                    ])
                    .into_element(cx),
                    shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
                ])
                .variant(shadcn::ItemVariant::Muted)
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-spinner-rtl");

        let centered_body = centered(cx, body);
        section(cx, "RTL", centered_body)
    };

    vec![
        cx.text("An indicator that can be used to show a loading state."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, custom, size, button, badge, input_group, empty, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_aspect_ratio(cx)
}

pub(in crate::ui) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_breadcrumb(cx, last_action)
}

pub(in crate::ui) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ButtonGroupModels {
        search_value: Option<Model<String>>,
        message_value: Option<Model<String>>,
        amount_value: Option<Model<String>>,
        dropdown_open: Option<Model<bool>>,
        select_open: Option<Model<bool>>,
        select_value: Option<Model<Option<Arc<str>>>>,
        popover_open: Option<Model<bool>>,
        popover_text: Option<Model<String>>,
    }

    let search_value = cx.with_state(ButtonGroupModels::default, |st| st.search_value.clone());
    let search_value = match search_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.search_value = Some(model.clone())
            });
            model
        }
    };

    let message_value = cx.with_state(ButtonGroupModels::default, |st| st.message_value.clone());
    let message_value = match message_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.message_value = Some(model.clone())
            });
            model
        }
    };

    let amount_value = cx.with_state(ButtonGroupModels::default, |st| st.amount_value.clone());
    let amount_value = match amount_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.amount_value = Some(model.clone())
            });
            model
        }
    };

    let dropdown_open = cx.with_state(ButtonGroupModels::default, |st| st.dropdown_open.clone());
    let dropdown_open = match dropdown_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.dropdown_open = Some(model.clone())
            });
            model
        }
    };

    let select_open = cx.with_state(ButtonGroupModels::default, |st| st.select_open.clone());
    let select_open = match select_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.select_open = Some(model.clone())
            });
            model
        }
    };

    let select_value = cx.with_state(ButtonGroupModels::default, |st| st.select_value.clone());
    let select_value = match select_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("$")));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.select_value = Some(model.clone())
            });
            model
        }
    };

    let popover_open = cx.with_state(ButtonGroupModels::default, |st| st.popover_open.clone());
    let popover_open = match popover_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_open = Some(model.clone())
            });
            model
        }
    };

    let popover_text = cx.with_state(ButtonGroupModels::default, |st| st.popover_text.clone());
    let popover_text = match popover_text {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(String::from("Describe your task in natural language."));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_text = Some(model.clone())
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let outline_fg = ColorRef::Color(theme.color_required("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    // Mirrors the top-level `button-group-demo` preview slot.
    let demo = shadcn::ButtonGroup::new([
        shadcn::Button::new("Left").into(),
        shadcn::Button::new("Middle").into(),
        shadcn::Button::new("Right").into(),
    ])
    .a11y_label("Button group")
    .into_element(cx);

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

    let orientation = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Increase")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
            shadcn::Button::new("Decrease")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.minus", outline_fg.clone())])
                .into(),
        ])
        .orientation(shadcn::ButtonGroupOrientation::Vertical)
        .a11y_label("Media controls")
        .into_element(cx);
        section(cx, "Orientation", body)
    };

    let size = {
        let small = shadcn::ButtonGroup::new([
            shadcn::Button::new("Small")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let medium = shadcn::ButtonGroup::new([
            shadcn::Button::new("Default")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let large = shadcn::ButtonGroup::new([
            shadcn::Button::new("Large")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconLg)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N4), |_cx| {
            vec![small, medium, large]
        });
        section(cx, "Size", body)
    };

    let nested = {
        let digits = shadcn::ButtonGroup::new([
            shadcn::Button::new("1")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("2")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("3")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("4")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("5")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
        ]);

        let nav = shadcn::ButtonGroup::new([
            shadcn::Button::new("Previous")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.arrow-left", outline_fg.clone())])
                .into(),
            shadcn::Button::new("Next")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.arrow-right", outline_fg.clone())])
                .into(),
        ]);

        let body = shadcn::ButtonGroup::new([digits.into(), nav.into()]).into_element(cx);
        section(cx, "Nested", body)
    };

    let separator = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Copy")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into(),
            shadcn::Button::new("Paste")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .into(),
        ])
        .into_element(cx);
        section(cx, "Separator", body)
    };

    let split = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Secondary)
                .into(),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", secondary_fg.clone())])
                .into(),
        ])
        .into_element(cx);
        section(cx, "Split", body)
    };

    let input = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Input::new(search_value.clone())
                .a11y_label("Search")
                .placeholder("Search...")
                .refine_layout(LayoutRefinement::default().w_px(Px(220.0)))
                .into_element(cx)
                .into(),
            shadcn::Button::new("Search")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.search", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);
        section(cx, "Input", body)
    };

    let input_group = {
        let group = shadcn::InputGroup::new(message_value.clone())
            .a11y_label("Message")
            .leading([shadcn::InputGroupText::new("To").into_element(cx)])
            .trailing([shadcn::InputGroupButton::new("Send").into_element(cx)]);

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
            group.into(),
        ])
        .into_element(cx);
        section(cx, "Input Group", body)
    };

    let dropdown = {
        let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("More")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .children([icon(cx, "lucide.chevron-down", outline_fg.clone())])
                    .toggle_model(dropdown_open.clone())
                    .into_element(cx)
            },
            |cx| {
                vec![
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mute Conversation").leading(icon(
                            cx,
                            "lucide.volume-x",
                            outline_fg.clone(),
                        )),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mark as Read").leading(icon(
                            cx,
                            "lucide.check",
                            outline_fg.clone(),
                        )),
                    ),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Delete Conversation")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .leading(icon(cx, "lucide.trash", outline_fg.clone())),
                    ),
                ]
            },
        );

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Follow")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            dropdown.into(),
        ])
        .into_element(cx);
        section(cx, "Dropdown Menu", body)
    };

    let select = {
        let currency = shadcn::Select::new(select_value.clone(), select_open.clone())
            .placeholder("$")
            .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
            .items([
                shadcn::SelectItem::new("$", "US Dollar"),
                shadcn::SelectItem::new("€", "Euro"),
                shadcn::SelectItem::new("£", "British Pound"),
            ])
            .into_element(cx);

        let amount = shadcn::Input::new(amount_value.clone())
            .a11y_label("Amount")
            .placeholder("10.00")
            .refine_layout(LayoutRefinement::default().w_px(Px(140.0)))
            .into_element(cx);

        let send = shadcn::Button::new("Send")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .children([icon(cx, "lucide.arrow-right", outline_fg.clone())]);

        let body = shadcn::ButtonGroup::new([
            shadcn::ButtonGroup::new([currency.into(), amount.into()]).into(),
            shadcn::ButtonGroup::new([send.into()]).into(),
        ])
        .into_element(cx);
        section(cx, "Select", body)
    };

    let popover = {
        let popover = shadcn::Popover::new(popover_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Icon)
                        .children([icon(cx, "lucide.chevron-down", outline_fg.clone())])
                        .toggle_model(popover_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new(vec![
                        shadcn::PopoverTitle::new("Agent Tasks").into_element(cx),
                        shadcn::Separator::new().into_element(cx),
                        shadcn::Textarea::new(popover_text.clone())
                            .a11y_label("Task")
                            .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
                            .into_element(cx),
                    ])
                    .into_element(cx)
                },
            );

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Copilot")
                .variant(shadcn::ButtonVariant::Outline)
                .children([icon(cx, "lucide.bot", outline_fg.clone())])
                .into(),
            popover.into(),
        ])
        .into_element(cx);
        section(cx, "Popover", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::ButtonGroup::new([
                    shadcn::Button::new("التالي")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into(),
                    shadcn::Button::new("السابق")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into(),
                ])
                .into_element(cx)
            },
        );
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                orientation,
                size,
                nested,
                separator,
                split,
                input,
                input_group,
                dropdown,
                select,
                popover,
                rtl,
            ]
        },
    );

    vec![demo, examples]
}

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    use fret_ui_headless::calendar::DateRangeSelection;

    let theme = Theme::global(&*cx.app).snapshot();
    let today = time::OffsetDateTime::now_utc().date();

    #[derive(Default, Clone)]
    struct CalendarModels {
        caption_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        caption_selected: Option<Model<Option<Date>>>,
        range_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        presets_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        presets_selected: Option<Model<Option<Date>>>,
        time_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        time_selected: Option<Model<Option<Date>>>,
        time_from: Option<Model<String>>,
        time_to: Option<Model<String>>,
        booked_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        booked_selected: Option<Model<Option<Date>>>,
        custom_cell_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        custom_cell_selected: Option<Model<Option<Date>>>,
        week_number_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        week_number_selected: Option<Model<Option<Date>>>,
        rtl_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
    }

    let initial_month = cx
        .get_model_copied(&month, Invalidation::Layout)
        .unwrap_or_else(|| fret_ui_headless::calendar::CalendarMonth::from_date(today));

    let state = cx.with_state(CalendarModels::default, |st| st.clone());

    let caption_month = match state.caption_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_month = Some(model.clone())
            });
            model
        }
    };
    let caption_selected = match state.caption_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_selected = Some(model.clone())
            });
            model
        }
    };

    let range_month = match state.range_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.range_month = Some(model.clone())
            });
            model
        }
    };
    let range_selected = match state.range_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(DateRangeSelection::default());
            cx.with_state(CalendarModels::default, |st| {
                st.range_selected = Some(model.clone())
            });
            model
        }
    };

    let preset_date = time::Date::from_calendar_date(today.year(), time::Month::February, 12)
        .expect("valid preset date");
    let presets_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(preset_date);
    let presets_month = match state.presets_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(presets_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.presets_month = Some(model.clone())
            });
            model
        }
    };
    let presets_selected = match state.presets_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(preset_date));
            cx.with_state(CalendarModels::default, |st| {
                st.presets_selected = Some(model.clone())
            });
            model
        }
    };

    let time_date = time::Date::from_calendar_date(today.year(), today.month(), 12)
        .expect("valid time picker date");
    let time_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(time_date);
    let time_month = match state.time_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(time_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.time_month = Some(model.clone())
            });
            model
        }
    };
    let time_selected = match state.time_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(time_date));
            cx.with_state(CalendarModels::default, |st| {
                st.time_selected = Some(model.clone())
            });
            model
        }
    };
    let time_from = match state.time_from {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_from = Some(model.clone())
            });
            model
        }
    };
    let time_to = match state.time_to {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("12:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_to = Some(model.clone())
            });
            model
        }
    };

    let booked_month = match state.booked_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_month = Some(model.clone())
            });
            model
        }
    };
    let booked_selected = match state.booked_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_selected = Some(model.clone())
            });
            model
        }
    };

    let custom_cell_month = match state.custom_cell_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_month = Some(model.clone())
            });
            model
        }
    };
    let custom_cell_selected = match state.custom_cell_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_selected = Some(model.clone())
            });
            model
        }
    };

    let week_number_month = match state.week_number_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_month = Some(model.clone())
            });
            model
        }
    };
    let week_number_selected = match state.week_number_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_month = match state.rtl_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_month = Some(model.clone())
            });
            model
        }
    };
    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(today));
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let basic = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let selected_str = cx
                    .get_model_copied(&selected, Invalidation::Layout)
                    .flatten()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::Calendar::new(month.clone(), selected.clone())
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![cx.text_props(TextProps {
                                layout: Default::default(),
                                text: Arc::from(format!("selected={}", selected_str)),
                                style: None,
                                color: Some(theme.color_required("muted-foreground")),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            })]
                        },
                    ),
                ]
            },
        );
        section(cx, "Basic", body)
    };

    let range = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let range = cx
                    .get_model_copied(&range_selected, Invalidation::Layout)
                    .unwrap_or_default();
                let from = range
                    .from
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());
                let to = range
                    .to
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
                        .number_of_months(2)
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("from={}", from)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("to={}", to)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Range Calendar", body)
    };

    let month_year_selector = {
        let body = shadcn::Calendar::new(caption_month.clone(), caption_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Month and Year Selector", body)
    };

    let presets = {
        let preset_button =
            |cx: &mut ElementContext<'_, App>, label: &'static str, days: i64| -> AnyElement {
                let month = presets_month.clone();
                let selected = presets_selected.clone();
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().flex_1().w_full())
                    .on_activate(Arc::new(move |host, _acx, _reason| {
                        let new_date = today + time::Duration::days(days);
                        let _ = host.models_mut().update(&selected, |v| *v = Some(new_date));
                        let _ = host.models_mut().update(&month, |m| {
                            *m = fret_ui_headless::calendar::CalendarMonth::from_date(new_date);
                        });
                    }))
                    .into_element(cx)
            };

        let calendar = shadcn::Calendar::new(presets_month.clone(), presets_selected.clone())
            .cell_size(Px(38.0))
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full())
                .items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "Today", 0),
                                preset_button(cx, "Tomorrow", 1),
                                preset_button(cx, "In 3 days", 3),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "In a week", 7),
                                preset_button(cx, "In 2 weeks", 14),
                            ]
                        },
                    ),
                ]
            },
        );

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(
            LayoutRefinement::default()
                .max_w(MetricRef::Px(Px(300.0)))
                .min_w_0(),
        )
        .into_element(cx);

        section(cx, "Presets", card)
    };

    let date_and_time_picker = {
        let clock_fg = ColorRef::Color(theme.color_required("muted-foreground"));
        let clock_icon = |cx: &mut ElementContext<'_, App>| {
            shadcn::icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.clock-2"),
                None,
                Some(clock_fg.clone()),
            )
        };

        let calendar = shadcn::Calendar::new(time_month.clone(), time_selected.clone())
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Start Time").into_element(cx),
                shadcn::InputGroup::new(time_from.clone())
                    .a11y_label("Start Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("End Time").into_element(cx),
                shadcn::InputGroup::new(time_to.clone())
                    .a11y_label("End Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(LayoutRefinement::default().min_w_0())
        .into_element(cx);

        section(cx, "Date and Time Picker", card)
    };

    let booked_dates = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                vec![
                    shadcn::Calendar::new(booked_month.clone(), booked_selected.clone())
                        .disabled_by(|d| {
                            matches!(d.weekday(), time::Weekday::Saturday | time::Weekday::Sunday)
                        })
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::from("Disabled: weekends"),
                        style: None,
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }),
                ]
            },
        );
        section(cx, "Booked dates", body)
    };

    let custom_cell_size = {
        let body = shadcn::Calendar::new(custom_cell_month.clone(), custom_cell_selected.clone())
            .cell_size(Px(44.0))
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Custom Cell Size", body)
    };

    let week_numbers = {
        let body = shadcn::Calendar::new(week_number_month.clone(), week_number_selected.clone())
            .show_week_number(true)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Week Numbers", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Calendar::new(rtl_month.clone(), rtl_selected.clone())
                    .cell_size(Px(36.0))
                    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                    .into_element(cx)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                basic,
                range,
                month_year_selector,
                presets,
                date_and_time_picker,
                booked_dates,
                custom_cell_size,
                week_numbers,
                rtl,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_collapsible(cx)
}

pub(in crate::ui) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_drawer(cx)
}

pub(in crate::ui) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_hover_card(cx)
}

pub(in crate::ui) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_group(cx)
}

pub(in crate::ui) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_otp(cx)
}

pub(in crate::ui) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_menubar(cx)
}
pub(in crate::ui) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_navigation_menu(cx)
}
pub(in crate::ui) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct PaginationModels {
        rows_per_page: Option<Model<Option<Arc<str>>>>,
        rows_per_page_open: Option<Model<bool>>,
    }

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

    let state = cx.with_state(PaginationModels::default, |st| st.clone());
    let rows_per_page = match state.rows_per_page {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("25")));
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page = Some(model.clone())
            });
            model
        }
    };
    let rows_per_page_open = match state.rows_per_page_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page_open = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Demo", body)
    };

    let simple = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("4")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("5")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Simple", body)
    };

    let icons_only = {
        let rows_per_page = shadcn::Select::new(rows_per_page.clone(), rows_per_page_open.clone())
            .placeholder("25")
            .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
            .items([
                shadcn::SelectItem::new("10", "10"),
                shadcn::SelectItem::new("25", "25"),
                shadcn::SelectItem::new("50", "50"),
                shadcn::SelectItem::new("100", "100"),
            ])
            .into_element(cx);

        let rows_field = shadcn::Field::new([
            shadcn::FieldLabel::new("Rows per page").into_element(cx),
            rows_per_page,
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .into_element(cx);

        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content])
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .into_element(cx);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .justify_between()
                .gap(Space::N4),
            move |_cx| [rows_field, pagination],
        );

        section(cx, "Icons Only", row)
    };

    let rtl = {
        fn to_arabic_numerals(num: u32) -> String {
            const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
            num.to_string()
                .chars()
                .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
                .collect()
        }

        let pagination = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let content = shadcn::PaginationContent::new([
                    shadcn::PaginationItem::new(
                        shadcn::PaginationPrevious::new()
                            .text("السابق")
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(1))])
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(2))])
                            .on_click(CMD_APP_SAVE)
                            .active(true)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(3))])
                            .on_click(CMD_APP_SAVE)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                        .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationNext::new()
                            .text("التالي")
                            .on_click(CMD_APP_SAVE)
                            .into_element(cx),
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                shadcn::Pagination::new([content]).into_element(cx)
            },
        );

        let body = centered(cx, pagination);
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N6).items_start(),
        |_cx| vec![demo, simple, icons_only, rtl],
    )]
}

pub(in crate::ui) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_carousel(cx)
}

pub(in crate::ui) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_chart(cx)
}

pub(in crate::ui) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_item(cx)
}
pub(in crate::ui) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_native_select(cx)
}

pub(in crate::ui) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SidebarModels {
        demo_collapsed: Option<Model<bool>>,
        demo_selected: Option<Model<Arc<str>>>,
        controlled_collapsed: Option<Model<bool>>,
        controlled_selected: Option<Model<Arc<str>>>,
        rtl_selected: Option<Model<Arc<str>>>,
    }

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

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let state = cx.with_state(SidebarModels::default, |st| st.clone());

    let demo_collapsed = match state.demo_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.demo_collapsed = Some(model.clone())
            });
            model
        }
    };

    let demo_selected = match state.demo_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.demo_selected = Some(model.clone())
            });
            model
        }
    };

    let controlled_collapsed = match state.controlled_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_collapsed = Some(model.clone())
            });
            model
        }
    };

    let controlled_selected = match state.controlled_selected {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Arc::<str>::from("design-engineering"));
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let resolve_selected =
        |cx: &mut ElementContext<'_, App>, model: &Model<Arc<str>>, fallback: &'static str| {
            cx.get_model_cloned(model, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from(fallback))
        };

    let menu_button = |cx: &mut ElementContext<'_, App>,
                       selected_model: Model<Arc<str>>,
                       active_value: &Arc<str>,
                       value: &'static str,
                       label: &'static str,
                       icon: &'static str,
                       collapsed: bool,
                       test_id: Arc<str>| {
        let is_active = active_value.as_ref() == value;
        let selected_for_activate = selected_model.clone();
        let value_for_activate: Arc<str> = Arc::from(value);
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_for_activate, |v| *v = value_for_activate.clone());
            host.request_redraw(action_cx.window);
        });

        shadcn::SidebarMenuButton::new(label)
            .icon(fret_icons::IconId::new_static(icon))
            .active(is_active)
            .collapsed(collapsed)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let demo = {
        let is_collapsed = cx
            .watch_model(&demo_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &demo_selected, "playground");

        let toolbar = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Toggle")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .toggle_model(demo_collapsed.clone())
                        .test_id("ui-gallery-sidebar-demo-toggle")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        if is_collapsed {
                            "Collapsed to icon rail"
                        } else {
                            "Expanded"
                        },
                    ),
                    shadcn::typography::muted(cx, format!("active={}", selected_value.as_ref())),
                ]
            },
        );

        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Platform")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "playground",
                    "Playground",
                    "lucide.square-terminal",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-playground"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "models",
                    "Models",
                    "lucide.bot",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-models"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "documentation",
                    "Documentation",
                    "lucide.book-open",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-documentation"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "settings",
                    "Settings",
                    "lucide.settings-2",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-settings"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-travel"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([
            shadcn::SidebarHeader::new([shadcn::typography::small(cx, "Acme Inc.")])
                .into_element(cx),
            shadcn::SidebarContent::new([platform, projects])
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarFooter::new([shadcn::typography::small(cx, "shadcn")]).into_element(cx),
        ])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let content = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Content").into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("A sidebar that collapses to icon mode."),
                cx.text("Select any menu item to verify active and hover states."),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(360.0))),
            |_cx| vec![sidebar, content],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-demo"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![toolbar, framed],
        );
        section(cx, "Demo", body)
    };

    let controlled = {
        let is_collapsed = cx
            .watch_model(&controlled_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &controlled_selected, "design-engineering");

        let header = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::Button::new(if is_collapsed {
                        "Open Sidebar"
                    } else {
                        "Close Sidebar"
                    })
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(controlled_collapsed.clone())
                    .test_id("ui-gallery-sidebar-controlled-toggle")
                    .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "Controlled via model (approximation of SidebarProvider open state).",
                    ),
                ]
            },
        );

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-travel"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "support",
                    "Support",
                    "lucide.life-buoy",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-support"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "feedback",
                    "Feedback",
                    "lucide.send",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-feedback"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([projects])
            .collapsed(is_collapsed)
            .into_element(cx)])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let inset = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Sidebar Inset").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("Use a main content panel next to Sidebar when controlled."),
                cx.text(format!("selected={}", selected_value.as_ref())),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
            |_cx| vec![sidebar, inset],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-controlled"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![header, framed],
        );

        section(cx, "Controlled", body)
    };

    let rtl = {
        let selected_value = resolve_selected(cx, &rtl_selected, "playground");

        let rtl_layout = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let platform = shadcn::SidebarGroup::new([
                    shadcn::SidebarGroupLabel::new("??????")
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarMenu::new([
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "playground",
                            "????",
                            "lucide.square-terminal",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-playground"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "documentation",
                            "???????",
                            "lucide.book-open",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-documentation"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "settings",
                            "?????????",
                            "lucide.settings-2",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-settings"),
                        ))
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx);

                let sidebar = shadcn::Sidebar::new([
                    shadcn::SidebarHeader::new([shadcn::typography::small(cx, "?????? ???????")])
                        .into_element(cx),
                    shadcn::SidebarContent::new([platform])
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarFooter::new([shadcn::typography::small(cx, "??????")])
                        .into_element(cx),
                ])
                .collapsed(false)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

                let content = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("RTL").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        cx.text("Direction provider flips layout and inline icon/text flow."),
                        cx.text(format!("active={}", selected_value.as_ref())),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
                    |_cx| vec![content, sidebar],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-rtl"),
        );

        let framed = shell(cx, rtl_layout);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A composable, themeable and customizable sidebar component."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, controlled, rtl]
        }),
    ]
}

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

pub(in crate::ui) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle(cx)
}

pub(in crate::ui) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle_group(cx)
}

pub(in crate::ui) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_typography(cx)
}

pub(in crate::ui) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_alert_dialog(cx, open)
}

pub(in crate::ui) fn preview_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_dialog(cx, open)
}

pub(in crate::ui) fn preview_popover(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct PopoverModels {
        demo_width: Option<Model<String>>,
        demo_max_width: Option<Model<String>>,
        demo_height: Option<Model<String>>,
        demo_max_height: Option<Model<String>>,
        form_width: Option<Model<String>>,
        form_height: Option<Model<String>>,
    }

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

    let state = cx.with_state(PopoverModels::default, |st| st.clone());
    let demo_width = match state.demo_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_width = Some(model.clone())
            });
            model
        }
    };
    let demo_max_width = match state.demo_max_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("300px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_width = Some(model.clone())
            });
            model
        }
    };
    let demo_height = match state.demo_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_height = Some(model.clone())
            });
            model
        }
    };
    let demo_max_height = match state.demo_max_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("none"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_height = Some(model.clone())
            });
            model
        }
    };
    let form_width = match state.form_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_width = Some(model.clone())
            });
            model
        }
    };
    let form_height = match state.form_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_height = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let popover = shadcn::Popover::new_controllable(cx, None, false).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                let row = |cx: &mut ElementContext<'_, App>,
                           label: &'static str,
                           model: Model<_>| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4)
                            .items_center(),
                        move |cx| {
                            vec![
                                ui::label(cx, label)
                                    .layout(
                                        LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0(),
                                    )
                                    .into_element(cx),
                                shadcn::Input::new(model)
                                    .size(fret_ui_kit::Size::Small)
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let header = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ]
                    },
                );

                let fields = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    move |cx| {
                        vec![
                            row(cx, "Width", demo_width.clone()),
                            row(cx, "Max. width", demo_max_width.clone()),
                            row(cx, "Height", demo_height.clone()),
                            row(cx, "Max. height", demo_max_height.clone()),
                        ]
                    },
                );

                shadcn::PopoverContent::new([header, fields])
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                    .into_element(cx)
            },
        );
        let body = centered(cx, popover);
        section(cx, "Demo", body)
    };

    let basic = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx)])
                    .into_element(cx)
                },
            );
        let body = centered(cx, popover);
        section(cx, "Basic", body)
    };

    let align = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N6)
                .items_center()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            |cx| {
                vec![
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Start)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("Start")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to start")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Center)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("Center")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to center")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::End)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("End")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to end")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                ]
            },
        );
        section(cx, "Align", body)
    };

    let with_form = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([
                        shadcn::PopoverHeader::new([
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::FieldGroup::new([
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Width")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_width.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Height")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_height.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                        ])
                        .gap(Space::N4)
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_px(Px(256.0)))
                    .into_element(cx)
                },
            );
        let body = centered(cx, popover);
        section(cx, "With Form", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let popover = |cx: &mut ElementContext<'_, App>,
                               label: &'static str,
                               side: shadcn::PopoverSide| {
                    shadcn::Popover::new_controllable(cx, None, false)
                        .side(side)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new(label)
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                                    shadcn::PopoverTitle::new("الأبعاد").into_element(cx),
                                    shadcn::PopoverDescription::new("تعيين الأبعاد للطبقة.")
                                        .into_element(cx),
                                ])
                                .into_element(cx)])
                                .into_element(cx)
                            },
                        )
                };

                let physical = stack::hstack_build(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |cx, out| {
                        for (id, label, side) in [
                            ("left", "يسار", shadcn::PopoverSide::Left),
                            ("top", "أعلى", shadcn::PopoverSide::Top),
                            ("bottom", "أسفل", shadcn::PopoverSide::Bottom),
                            ("right", "يمين", shadcn::PopoverSide::Right),
                        ] {
                            out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                        }
                    },
                );

                let logical = stack::hstack_build(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |cx, out| {
                        for (id, label, side) in [
                            (
                                "inline-start",
                                "بداية السطر",
                                shadcn::PopoverSide::InlineStart,
                            ),
                            ("inline-end", "نهاية السطر", shadcn::PopoverSide::InlineEnd),
                        ] {
                            out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                        }
                    },
                );

                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    move |_cx| [physical, logical],
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![demo, basic, align, with_form, rtl]
}

pub(in crate::ui) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SheetModels {
        demo_name: Option<Model<String>>,
        demo_username: Option<Model<String>>,
        side_top_open: Option<Model<bool>>,
        side_right_open: Option<Model<bool>>,
        side_bottom_open: Option<Model<bool>>,
        side_left_open: Option<Model<bool>>,
        no_close_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        rtl_name: Option<Model<String>>,
        rtl_username: Option<Model<String>>,
    }

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

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let state = cx.with_state(SheetModels::default, |st| st.clone());

    let demo_name = match state.demo_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_name = Some(model.clone())
            });
            model
        }
    };

    let demo_username = match state.demo_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("@peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_username = Some(model.clone())
            });
            model
        }
    };

    let side_top_open = match state.side_top_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_top_open = Some(model.clone())
            });
            model
        }
    };

    let side_right_open = match state.side_right_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_right_open = Some(model.clone())
            });
            model
        }
    };

    let side_bottom_open = match state.side_bottom_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_bottom_open = Some(model.clone())
            });
            model
        }
    };

    let side_left_open = match state.side_left_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_left_open = Some(model.clone())
            });
            model
        }
    };

    let no_close_open = match state.no_close_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.no_close_open = Some(model.clone())
            });
            model
        }
    };

    let rtl_open = match state.rtl_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| st.rtl_open = Some(model.clone()));
            model
        }
    };

    let rtl_name = match state.rtl_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| st.rtl_name = Some(model.clone()));
            model
        }
    };

    let rtl_username = match state.rtl_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.rtl_username = Some(model.clone())
            });
            model
        }
    };

    let profile_fields =
        |cx: &mut ElementContext<'_, App>, name: Model<String>, username: Model<String>| {
            let field =
                |cx: &mut ElementContext<'_, App>, label: &'static str, model: Model<String>| {
                    shadcn::Field::new([
                        shadcn::FieldLabel::new(label).into_element(cx),
                        shadcn::Input::new(model)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    ])
                    .into_element(cx)
                };

            shadcn::FieldSet::new([field(cx, "Name", name), field(cx, "Username", username)])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
        };

    let demo = {
        let trigger_open = open.clone();
        let save_open = open.clone();
        let close_open = open.clone();
        let name_model = demo_name.clone();
        let username_model = demo_username.clone();

        let demo_sheet = shadcn::Sheet::new(open.clone())
            .side(shadcn::SheetSide::Right)
            .size(Px(420.0))
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::SheetContent::new([
                        shadcn::SheetHeader::new([
                            shadcn::SheetTitle::new("Edit profile").into_element(cx),
                            shadcn::SheetDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        profile_fields(cx, name_model.clone(), username_model.clone()),
                        shadcn::SheetFooter::new([
                            shadcn::Button::new("Save changes")
                                .toggle_model(save_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id("ui-gallery-sheet-demo-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sheet-demo"),
            );

        let card = shell(cx, LayoutRefinement::default(), demo_sheet);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let side = {
        let side_sheet = |cx: &mut ElementContext<'_, App>,
                          id: &'static str,
                          label: &'static str,
                          side: shadcn::SheetSide,
                          open_model: Model<bool>| {
            let trigger_open = open_model.clone();
            let save_open = open_model.clone();
            let cancel_open = open_model.clone();
            let size = if matches!(side, shadcn::SheetSide::Top | shadcn::SheetSide::Bottom) {
                Px(320.0)
            } else {
                Px(420.0)
            };

            shadcn::Sheet::new(open_model)
                .side(side)
                .size(size)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new(label)
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(trigger_open.clone())
                            .test_id(format!("ui-gallery-sheet-side-{id}-trigger"))
                            .into_element(cx)
                    },
                    |cx| {
                        let paragraphs = stack::vstack(
                            cx,
                            stack::VStackProps::default().gap(Space::N2),
                            |cx| {
                                (0..8)
                                    .map(|idx| {
                                        shadcn::typography::muted(
                                            cx,
                                            format!(
                                                "Profile section line {}. Keep this content scrollable for constrained sheets.",
                                                idx + 1
                                            ),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            },
                        );

                        let scroll = shadcn::ScrollArea::new([paragraphs])
                            .axis(fret_ui::element::ScrollAxis::Y)
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(180.0)))
                            .into_element(cx);

                        shadcn::SheetContent::new([
                            shadcn::SheetHeader::new([
                                shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                shadcn::SheetDescription::new(
                                    "Use side to control which edge the sheet appears from.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            scroll,
                            shadcn::SheetFooter::new([
                                shadcn::Button::new("Save changes")
                                    .toggle_model(save_open.clone())
                                    .into_element(cx),
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(cancel_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    },
                )
        };

        let row = stack::hstack_build(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx, out| {
                let items = [
                    ("top", "Top", shadcn::SheetSide::Top, side_top_open.clone()),
                    (
                        "right",
                        "Right",
                        shadcn::SheetSide::Right,
                        side_right_open.clone(),
                    ),
                    (
                        "bottom",
                        "Bottom",
                        shadcn::SheetSide::Bottom,
                        side_bottom_open.clone(),
                    ),
                    (
                        "left",
                        "Left",
                        shadcn::SheetSide::Left,
                        side_left_open.clone(),
                    ),
                ];
                for (id, label, side, open_model) in items {
                    out.push(
                        cx.keyed(id, |cx| side_sheet(cx, id, label, side, open_model.clone())),
                    );
                }
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-side"),
        );

        let card = shell(cx, LayoutRefinement::default(), row);
        let body = centered(cx, card);
        section(cx, "Side", body)
    };

    let no_close_button = {
        let trigger_open = no_close_open.clone();

        let sheet = shadcn::Sheet::new(no_close_open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("No Close Button").into_element(cx),
                        shadcn::SheetDescription::new(
                            "This example intentionally omits footer actions. Use outside press or Escape to close.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-no-close-button"),
        );

        let card = shell(cx, LayoutRefinement::default(), sheet);
        let body = centered(cx, card);
        section(cx, "No Close Button", body)
    };

    let rtl = {
        let rtl_demo = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let trigger_open = rtl_open.clone();
                let save_open = rtl_open.clone();
                let close_open = rtl_open.clone();
                let name_model = rtl_name.clone();
                let username_model = rtl_username.clone();

                shadcn::Sheet::new(rtl_open.clone())
                    .side(shadcn::SheetSide::Left)
                    .size(Px(420.0))
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(trigger_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            shadcn::SheetContent::new([
                                shadcn::SheetHeader::new([
                                    shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                    shadcn::SheetDescription::new(
                                        "RTL layout keeps spacing and focus flow aligned.",
                                    )
                                    .into_element(cx),
                                ])
                                .into_element(cx),
                                profile_fields(cx, name_model.clone(), username_model.clone()),
                                shadcn::SheetFooter::new([
                                    shadcn::Button::new("Save changes")
                                        .toggle_model(save_open.clone())
                                        .into_element(cx),
                                    shadcn::Button::new("Close")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .toggle_model(close_open.clone())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ])
                            .into_element(cx)
                        },
                    )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_demo);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Extends dialog to display side-aligned panels for supplementary tasks."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, side, no_close_button, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_empty(cx)
}
