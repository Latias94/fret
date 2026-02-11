use super::super::super::super::*;

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
