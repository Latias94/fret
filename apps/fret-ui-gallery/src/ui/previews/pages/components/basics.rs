use super::super::super::super::*;

pub(in crate::ui) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_alert(cx)
}

pub(in crate::ui) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_shadcn_extras(cx)
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
