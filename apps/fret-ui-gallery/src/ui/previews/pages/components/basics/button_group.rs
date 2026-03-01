use super::super::super::super::super::*;

pub(in crate::ui) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default)]
    struct ButtonGroupModels {
        demo_menu_open: Option<Model<bool>>,
        demo_label_value: Option<Model<Option<Arc<str>>>>,
        search_value: Option<Model<String>>,
        url_value: Option<Model<String>>,
        message_value: Option<Model<String>>,
        amount_value: Option<Model<String>>,
        voice_enabled: Option<Model<bool>>,
        dropdown_open: Option<Model<bool>>,
        select_open: Option<Model<bool>>,
        select_value: Option<Model<Option<Arc<str>>>>,
        popover_open: Option<Model<bool>>,
        popover_text: Option<Model<String>>,
    }

    let demo_menu_open = cx.with_state(ButtonGroupModels::default, |st| st.demo_menu_open.clone());
    let demo_menu_open = match demo_menu_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.demo_menu_open = Some(model.clone());
            });
            model
        }
    };

    let demo_label_value =
        cx.with_state(ButtonGroupModels::default, |st| st.demo_label_value.clone());
    let demo_label_value = match demo_label_value {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("personal")));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.demo_label_value = Some(model.clone());
            });
            model
        }
    };

    let search_value = cx.with_state(ButtonGroupModels::default, |st| st.search_value.clone());
    let search_value = match search_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.search_value = Some(model.clone());
            });
            model
        }
    };

    let url_value = cx.with_state(ButtonGroupModels::default, |st| st.url_value.clone());
    let url_value = match url_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.url_value = Some(model.clone());
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
                st.message_value = Some(model.clone());
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
                st.amount_value = Some(model.clone());
            });
            model
        }
    };

    let voice_enabled = cx.with_state(ButtonGroupModels::default, |st| st.voice_enabled.clone());
    let voice_enabled = match voice_enabled {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.voice_enabled = Some(model.clone());
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
                st.dropdown_open = Some(model.clone());
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
                st.select_open = Some(model.clone());
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
                st.select_value = Some(model.clone());
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
                st.popover_open = Some(model.clone());
            });
            model
        }
    };

    let popover_text = cx.with_state(ButtonGroupModels::default, |st| st.popover_text.clone());
    let popover_text = match popover_text {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_text = Some(model.clone());
            });
            model
        }
    };

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    let demo = {
        let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
        let corners_last = Corners {
            top_left: Px(0.0),
            bottom_left: Px(0.0),
            top_right: radius,
            bottom_right: radius,
        };

        let menu_trigger = shadcn::Button::new("")
            .a11y_label("More Options")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.more-horizontal"))
            .toggle_model(demo_menu_open.clone())
            .border_left_width_override(Px(0.0))
            .corner_radii_override(corners_last)
            .into_element(cx);

        let menu = shadcn::DropdownMenu::new(demo_menu_open.clone())
            .align(shadcn::DropdownMenuAlign::End)
            .into_element(
                cx,
                |_cx| menu_trigger,
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Mark as Read")
                                    .leading_icon(icon_id("lucide.mail-check")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Archive")
                                    .leading_icon(icon_id("lucide.archive")),
                            ),
                        ])),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Snooze")
                                    .leading_icon(icon_id("lucide.clock")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Add to Calendar")
                                    .leading_icon(icon_id("lucide.calendar-plus")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Add to List")
                                    .leading_icon(icon_id("lucide.list-filter")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Label As...")
                                    .leading_icon(icon_id("lucide.tag"))
                                    .submenu([shadcn::DropdownMenuEntry::RadioGroup(
                                        shadcn::DropdownMenuRadioGroup::new(
                                            demo_label_value.clone(),
                                        )
                                        .item(shadcn::DropdownMenuRadioItemSpec::new(
                                            "personal", "Personal",
                                        ))
                                        .item(shadcn::DropdownMenuRadioItemSpec::new(
                                            "work", "Work",
                                        ))
                                        .item(
                                            shadcn::DropdownMenuRadioItemSpec::new(
                                                "other", "Other",
                                            ),
                                        ),
                                    )]),
                            ),
                        ])),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Trash")
                                    .variant(
                                        shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    )
                                    .leading_icon(icon_id("lucide.trash")),
                            ),
                        ])),
                    ]
                },
            );

        let back = shadcn::ButtonGroup::new([shadcn::Button::new("")
            .a11y_label("Go Back")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.arrow-left"))
            .into()]);

        let actions = shadcn::ButtonGroup::new([
            shadcn::Button::new("Archive")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Report")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
        ]);

        let snooze = shadcn::ButtonGroup::new([
            shadcn::Button::new("Snooze")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            menu.into(),
        ]);

        shadcn::ButtonGroup::new([back.into(), actions.into(), snooze.into()])
            .a11y_label("Button group")
            .into_element(cx)
            .test_id("ui-gallery-button-group-demo")
    };

    let orientation = shadcn::ButtonGroup::new([
        shadcn::Button::new("")
            .a11y_label("Increase")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into(),
        shadcn::Button::new("")
            .a11y_label("Decrease")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.minus"))
            .into(),
    ])
    .orientation(shadcn::ButtonGroupOrientation::Vertical)
    .a11y_label("Media controls")
    .into_element(cx)
    .test_id("ui-gallery-button-group-orientation");

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
            shadcn::Button::new("")
                .a11y_label("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .icon(icon_id("lucide.plus"))
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
            shadcn::Button::new("")
                .a11y_label("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .icon(icon_id("lucide.plus"))
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
            shadcn::Button::new("")
                .a11y_label("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconLg)
                .icon(icon_id("lucide.plus"))
                .into(),
        ])
        .into_element(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![small, medium, large],
        )
        .test_id("ui-gallery-button-group-size")
    };

    let nested = {
        let voice_tooltip = shadcn::Tooltip::new(
            shadcn::InputGroupButton::new("")
                .a11y_label("Voice Mode")
                .size(shadcn::InputGroupButtonSize::IconSm)
                .icon(icon_id("lucide.audio-lines"))
                .into_element(cx),
            shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
                .into_element(cx),
        )
        .arrow(true)
        .side(shadcn::TooltipSide::Top)
        .into_element(cx);

        let input_group = shadcn::InputGroup::new(message_value.clone())
            .a11y_label("Message")
            .trailing([voice_tooltip])
            .trailing_has_button(true);

        let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into()]);

        let message = shadcn::ButtonGroup::new([input_group.into()])
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

        shadcn::ButtonGroup::new([plus.into(), message.into()])
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_w(Px(560.0)),
            )
            .into_element(cx)
            .test_id("ui-gallery-button-group-nested")
    };

    let separator = shadcn::ButtonGroup::new([
        shadcn::Button::new("Copy")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::ButtonGroupSeparator::new().into(),
        shadcn::Button::new("Paste")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-separator");

    let split = shadcn::ButtonGroup::new([
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Secondary)
            .into(),
        shadcn::ButtonGroupSeparator::new().into(),
        shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-split");

    let text = shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new("https://").into(),
        shadcn::Input::new(url_value.clone())
            .a11y_label("URL")
            .placeholder("example")
            .refine_layout(LayoutRefinement::default().w_px(Px(220.0)).min_w_0())
            .into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-text");

    let flex_1 = shadcn::ButtonGroup::new([
        shadcn::Button::new("Overview")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-overview")
            .into(),
        shadcn::Button::new("Analytics")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-analytics")
            .into(),
        shadcn::Button::new("Reports")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .test_id("ui-gallery-button-group-flex1-reports")
            .into(),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
    .into_element(cx)
    .test_id("ui-gallery-button-group-flex1");

    let input = shadcn::ButtonGroup::new([
        shadcn::Input::new(search_value.clone())
            .a11y_label("Search")
            .placeholder("Search...")
            .into(),
        shadcn::Button::new("")
            .a11y_label("Search")
            .variant(shadcn::ButtonVariant::Outline)
            .children([doc_layout::icon(cx, "lucide.search")])
            .into(),
    ])
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(420.0)),
    )
    .into_element(cx)
    .test_id("ui-gallery-button-group-input");

    let input_group = {
        let voice_tooltip = shadcn::Tooltip::new(
            shadcn::Button::new("")
                .a11y_label("Voice Mode")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::IconSm)
                .icon(icon_id("lucide.audio-lines"))
                .toggle_model(voice_enabled.clone())
                .into_element(cx),
            shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
                .into_element(cx),
        )
        .arrow(true)
        .side(shadcn::TooltipSide::Top)
        .into_element(cx);

        let voice_enabled_now = cx
            .get_model_cloned(&voice_enabled, fret_ui::Invalidation::Paint)
            .unwrap_or(false);
        let placeholder = if voice_enabled_now {
            "Record and send audio..."
        } else {
            "Send a message..."
        };

        let group = shadcn::InputGroup::new(message_value.clone())
            .a11y_label("Message")
            .placeholder(placeholder)
            .disabled(voice_enabled_now)
            .trailing([voice_tooltip])
            .trailing_has_button(true)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0());

        let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into()]);

        let message = shadcn::ButtonGroup::new([group.into()])
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

        shadcn::ButtonGroup::new([plus.into(), message.into()])
            .radius_override(Radius::Full)
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_w(Px(760.0)),
            )
            .into_element(cx)
            .test_id("ui-gallery-button-group-input-group")
    };

    let dropdown = {
        let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
        let corners_last = Corners {
            top_left: Px(0.0),
            bottom_left: Px(0.0),
            top_right: radius,
            bottom_right: radius,
        };

        let dropdown_trigger = shadcn::Button::new("")
            .a11y_label("More")
            .variant(shadcn::ButtonVariant::Outline)
            .refine_style(ChromeRefinement::default().pl(Space::N2))
            .children([doc_layout::icon(cx, "lucide.chevron-down")])
            .toggle_model(dropdown_open.clone())
            .border_left_width_override(Px(0.0))
            .corner_radii_override(corners_last)
            .into_element(cx);

        let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone())
            .align(shadcn::DropdownMenuAlign::End)
            .into_element(
                cx,
                |_cx| dropdown_trigger,
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Mute Conversation")
                                    .leading_icon(icon_id("lucide.volume-x")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Mark as Read")
                                    .leading_icon(icon_id("lucide.check")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Report Conversation")
                                    .leading_icon(icon_id("lucide.alert-triangle")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Block User")
                                    .leading_icon(icon_id("lucide.user-round-x")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Share Conversation")
                                    .leading_icon(icon_id("lucide.share")),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Copy Conversation")
                                    .leading_icon(icon_id("lucide.copy")),
                            ),
                        ])),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Delete Conversation")
                                    .variant(
                                        shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    )
                                    .leading_icon(icon_id("lucide.trash")),
                            ),
                        ])),
                    ]
                },
            );

        shadcn::ButtonGroup::new([
            shadcn::Button::new("Follow")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            dropdown.into(),
        ])
        .into_element(cx)
        .test_id("ui-gallery-button-group-dropdown")
    };

    let select = {
        let currency = shadcn::Select::new(select_value.clone(), select_open.clone())
            .placeholder("$")
            .trigger_value_as_label()
            .trigger_font_monospace()
            .content(shadcn::SelectContent::new().align_item_with_trigger(false))
            .align(shadcn::SelectAlign::Start)
            .items([
                shadcn::SelectItem::new("$", "US Dollar").show_value_in_list(true),
                shadcn::SelectItem::new("€", "Euro").show_value_in_list(true),
                shadcn::SelectItem::new("£", "British Pound").show_value_in_list(true),
            ]);

        let amount = shadcn::Input::new(amount_value.clone())
            .a11y_label("Amount")
            .placeholder("10.00");

        let send = shadcn::Button::new("")
            .a11y_label("Send")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.arrow-right"));

        shadcn::ButtonGroup::new([
            shadcn::ButtonGroup::new([currency.into(), amount.into()]).into(),
            shadcn::ButtonGroup::new([send.into()]).into(),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(760.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-button-group-select")
    };

    let popover = {
        let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
        let corners_last = Corners {
            top_left: Px(0.0),
            bottom_left: Px(0.0),
            top_right: radius,
            bottom_right: radius,
        };

        let popover = shadcn::Popover::new(popover_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("")
                        .a11y_label("Open popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Icon)
                        .icon(icon_id("lucide.chevron-down"))
                        .toggle_model(popover_open.clone())
                        .border_left_width_override(Px(0.0))
                        .corner_radii_override(corners_last)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new(vec![
                        shadcn::PopoverHeader::new([
                            shadcn::PopoverTitle::new("Start a new task with Copilot")
                                .into_element(cx),
                            shadcn::PopoverDescription::new(
                                "Describe your task in natural language.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::Field::new([
                            fret_ui_kit::primitives::visually_hidden::visually_hidden_label(
                                cx,
                                "Task Description",
                            ),
                            shadcn::Textarea::new(popover_text.clone())
                                .a11y_label("Task Description")
                                .placeholder("I need to...")
                                .resizable(false)
                                .into_element(cx),
                            shadcn::FieldDescription::new(
                                "Copilot will open a pull request for review.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
                    .into_element(cx)
                },
            );

        shadcn::ButtonGroup::new([
            shadcn::Button::new("Copilot")
                .variant(shadcn::ButtonVariant::Outline)
                .leading_icon(icon_id("lucide.bot"))
                .into(),
            popover.into(),
        ])
        .into_element(cx)
        .test_id("ui-gallery-button-group-popover")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::ButtonGroup::new([
            shadcn::Button::new("التالي")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("السابق")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
        ])
        .into_element(cx)
    })
    .test_id("ui-gallery-button-group-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn ButtonGroup demo (new-york-v4).",
            "Fret provides `ButtonGroupSeparator` / `ButtonGroupText` to match upstream docs recipes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the upstream intent: compose buttons + inputs/menus into a shared control group.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-button-group-demo")
                .code(
                    "rust",
                    r#"use fret_ui_shadcn as shadcn;
use std::sync::Arc;

let menu_open = cx.app.models_mut().insert(false);
let label = cx
    .app
    .models_mut()
    .insert(Some(Arc::<str>::from("personal")));

let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
let corners_last = fret_core::Corners {
    top_left: fret_core::Px(0.0),
    bottom_left: fret_core::Px(0.0),
    top_right: radius,
    bottom_right: radius,
};

let menu_trigger = shadcn::Button::new("")
    .a11y_label("More Options")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .icon(fret_icons::IconId::new_static("lucide.more-horizontal"))
    .toggle_model(menu_open.clone())
    .border_left_width_override(fret_core::Px(0.0))
    .corner_radii_override(corners_last)
    .into_element(cx);

let menu = shadcn::DropdownMenu::new(menu_open.clone())
    .align(shadcn::DropdownMenuAlign::End)
    .into_element(
        cx,
        |_cx| menu_trigger,
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mark as Read")
                            .leading_icon(fret_icons::IconId::new_static("lucide.mail-check")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Archive")
                            .leading_icon(fret_icons::IconId::new_static("lucide.archive")),
                    ),
                ])),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Snooze")
                            .leading_icon(fret_icons::IconId::new_static("lucide.clock")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Add to Calendar")
                            .leading_icon(fret_icons::IconId::new_static("lucide.calendar-plus")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Add to List")
                            .leading_icon(fret_icons::IconId::new_static("lucide.list-filter")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Label As...")
                            .leading_icon(fret_icons::IconId::new_static("lucide.tag"))
                            .submenu([shadcn::DropdownMenuEntry::RadioGroup(
                                shadcn::DropdownMenuRadioGroup::new(label.clone())
                                    .item(shadcn::DropdownMenuRadioItemSpec::new(
                                        "personal", "Personal",
                                    ))
                                    .item(shadcn::DropdownMenuRadioItemSpec::new("work", "Work"))
                                    .item(shadcn::DropdownMenuRadioItemSpec::new(
                                        "other", "Other",
                                    )),
                            )]),
                    ),
                ])),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Trash")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .leading_icon(fret_icons::IconId::new_static("lucide.trash")),
                    ),
                ])),
            ]
        },
    );

let back = shadcn::ButtonGroup::new([shadcn::Button::new("")
    .a11y_label("Go Back")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .icon(fret_icons::IconId::new_static("lucide.arrow-left"))
    .into()]);

let actions = shadcn::ButtonGroup::new([
    shadcn::Button::new("Archive")
        .variant(shadcn::ButtonVariant::Outline)
        .into(),
    shadcn::Button::new("Report")
        .variant(shadcn::ButtonVariant::Outline)
        .into(),
]);

let snooze = shadcn::ButtonGroup::new([
    shadcn::Button::new("Snooze")
        .variant(shadcn::ButtonVariant::Outline)
        .into(),
    menu.into(),
]);

shadcn::ButtonGroup::new([back.into(), actions.into(), snooze.into()])
    .a11y_label("Button group")
    .into_element(cx);"#,
                ),
            DocSection::new("Orientation", orientation)
                .test_id_prefix("ui-gallery-button-group-orientation")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("")
        .a11y_label("Increase")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static("lucide.plus"))
        .into(),
    shadcn::Button::new("")
        .a11y_label("Decrease")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static("lucide.minus"))
        .into(),
])
.orientation(shadcn::ButtonGroupOrientation::Vertical)
.a11y_label("Media controls")
.into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .no_shell()
                .test_id_prefix("ui-gallery-button-group-size")
                .code(
                    "rust",
                    r#"let small = shadcn::ButtonGroup::new([
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
    shadcn::Button::new("")
        .a11y_label("Add")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .icon(fret_icons::IconId::new_static("lucide.plus"))
        .into(),
])
.into_element(cx);

let medium = shadcn::ButtonGroup::new([
    shadcn::Button::new("Default").variant(shadcn::ButtonVariant::Outline).into(),
    shadcn::Button::new("Button").variant(shadcn::ButtonVariant::Outline).into(),
    shadcn::Button::new("Group").variant(shadcn::ButtonVariant::Outline).into(),
    shadcn::Button::new("")
        .a11y_label("Add")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static("lucide.plus"))
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
    shadcn::Button::new("")
        .a11y_label("Add")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconLg)
        .icon(fret_icons::IconId::new_static("lucide.plus"))
        .into(),
])
.into_element(cx);

stack::vstack(
    cx,
    stack::VStackProps::default()
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0()),
    |_cx| vec![small, medium, large],
)"#,
                ),
            DocSection::new("Nested", nested)
                .test_id_prefix("ui-gallery-button-group-nested")
                .code(
                    "rust",
                    r#"let model = cx.app.models_mut().insert(String::new());

let voice_tooltip = shadcn::Tooltip::new(
    shadcn::InputGroupButton::new("")
        .a11y_label("Voice Mode")
        .size(shadcn::InputGroupButtonSize::IconSm)
        .icon(fret_icons::IconId::new_static("lucide.audio-lines"))
        .into_element(cx),
    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
        .into_element(cx),
)
.arrow(true)
.side(shadcn::TooltipSide::Top)
.into_element(cx);

let input_group = shadcn::InputGroup::new(model)
    .a11y_label("Message")
    .trailing([voice_tooltip])
    .trailing_has_button(true);

let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .icon(fret_icons::IconId::new_static("lucide.plus"))
    .into()]);

let message = shadcn::ButtonGroup::new([input_group.into()]);

shadcn::ButtonGroup::new([plus.into(), message.into()])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(560.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Separator", separator)
                .test_id_prefix("ui-gallery-button-group-separator")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Copy").variant(shadcn::ButtonVariant::Secondary).into(),
    shadcn::ButtonGroupSeparator::new().into(),
    shadcn::Button::new("Paste").variant(shadcn::ButtonVariant::Secondary).into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Split", split)
                .test_id_prefix("ui-gallery-button-group-split")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Button").variant(shadcn::ButtonVariant::Secondary).into(),
    shadcn::ButtonGroupSeparator::new().into(),
    shadcn::Button::new("")
        .a11y_label("Add")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Icon)
        .icon(fret_icons::IconId::new_static("lucide.plus"))
        .into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Text", text)
                .test_id_prefix("ui-gallery-button-group-text")
                .code(
                    "rust",
                    r#"let model = cx.app.models_mut().insert(String::new());

shadcn::ButtonGroup::new([
    shadcn::ButtonGroupText::new("https://").into(),
    shadcn::Input::new(model)
        .a11y_label("URL")
        .placeholder("example")
        .refine_layout(LayoutRefinement::default().w_px(Px(220.0)).min_w_0())
        .into(),
    shadcn::ButtonGroupText::new(".com").into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Flex-1 items", flex_1)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-button-group-flex1")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Overview")
        .variant(shadcn::ButtonVariant::Outline)
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into(),
    shadcn::Button::new("Analytics")
        .variant(shadcn::ButtonVariant::Outline)
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into(),
])
.refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
.into_element(cx);"#,
                ),
            DocSection::new("Input", input)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-button-group-input")
                .code(
                    "rust",
                    r#"let model = cx.app.models_mut().insert(String::new());

shadcn::ButtonGroup::new([
    shadcn::Input::new(model)
        .a11y_label("Search")
        .placeholder("Search...")
        .into(),
    shadcn::Button::new("")
        .a11y_label("Search")
        .variant(shadcn::ButtonVariant::Outline)
        .children([shadcn::icon::icon(
            cx,
            fret_icons::IconId::new_static("lucide.search"),
        )])
        .into(),
])
.refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(420.0)))
.into_element(cx);"#,
                ),
            DocSection::new("Input Group", input_group)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-input-group")
                .code(
                    "rust",
                    r#"let model = cx.app.models_mut().insert(String::new());
	let voice_enabled = cx.app.models_mut().insert(false);

	let voice_tooltip = shadcn::Tooltip::new(
	    shadcn::Button::new("")
	        .a11y_label("Voice Mode")
	        .variant(shadcn::ButtonVariant::Ghost)
	        .size(shadcn::ButtonSize::IconSm)
	        .icon(fret_icons::IconId::new_static("lucide.audio-lines"))
	        .toggle_model(voice_enabled.clone())
	        .into_element(cx),
	    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, "Voice Mode")])
	        .into_element(cx),
	)
	.arrow(true)
	.side(shadcn::TooltipSide::Top)
	.into_element(cx);

	let voice_enabled_now = cx
	    .get_model_cloned(&voice_enabled, fret_ui::Invalidation::Paint)
	    .unwrap_or(false);
	let placeholder = if voice_enabled_now {
	    "Record and send audio..."
	} else {
	    "Send a message..."
	};

	let group = shadcn::InputGroup::new(model)
	    .a11y_label("Message")
	    .placeholder(placeholder)
	    .disabled(voice_enabled_now)
	    .trailing([voice_tooltip])
	    .trailing_has_button(true)
	    .refine_layout(LayoutRefinement::default().w_full().min_w_0());

	let plus = shadcn::ButtonGroup::new([shadcn::Button::new("")
	    .a11y_label("Add")
	    .variant(shadcn::ButtonVariant::Outline)
	    .size(shadcn::ButtonSize::Icon)
	    .icon(fret_icons::IconId::new_static("lucide.plus"))
	    .into()]);

	let message = shadcn::ButtonGroup::new([group.into()])
	    .refine_layout(LayoutRefinement::default().flex_1().min_w_0());

	shadcn::ButtonGroup::new([
	    plus.into(),
	    message.into(),
	])
.radius_override(Radius::Full)
.refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(760.0)))
.into_element(cx);"#,
                ),
            DocSection::new("Dropdown Menu", dropdown)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-dropdown")
                .code(
                    "rust",
                    r#"let open_model = cx.app.models_mut().insert(false);

let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
let corners_last = fret_core::Corners {
    top_left: fret_core::Px(0.0),
    bottom_left: fret_core::Px(0.0),
    top_right: radius,
    bottom_right: radius,
};

let menu_trigger = shadcn::Button::new("")
    .a11y_label("More")
    .variant(shadcn::ButtonVariant::Outline)
    .refine_style(ChromeRefinement::default().pl(Space::N2))
    .children([shadcn::icon::icon(
        cx,
        fret_icons::IconId::new_static("lucide.chevron-down"),
    )])
    .toggle_model(open_model.clone())
    .border_left_width_override(fret_core::Px(0.0))
    .corner_radii_override(corners_last)
    .into_element(cx);

let menu = shadcn::DropdownMenu::new(open_model.clone())
    .align(shadcn::DropdownMenuAlign::End)
    .into_element(
        cx,
        |_cx| menu_trigger,
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mute Conversation")
                            .leading_icon(fret_icons::IconId::new_static("lucide.volume-x")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mark as Read")
                            .leading_icon(fret_icons::IconId::new_static("lucide.check")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Report Conversation")
                            .leading_icon(fret_icons::IconId::new_static("lucide.alert-triangle")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Block User")
                            .leading_icon(fret_icons::IconId::new_static("lucide.user-round-x")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Share Conversation")
                            .leading_icon(fret_icons::IconId::new_static("lucide.share")),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Copy Conversation")
                            .leading_icon(fret_icons::IconId::new_static("lucide.copy")),
                    ),
                ])),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Delete Conversation")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .leading_icon(fret_icons::IconId::new_static("lucide.trash")),
                    ),
                ])),
            ]
        },
    );

shadcn::ButtonGroup::new([
    shadcn::Button::new("Follow")
        .variant(shadcn::ButtonVariant::Outline)
        .into(),
    menu.into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Select", select)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-select")
                .code(
                    "rust",
                    r#"let select_open = cx.app.models_mut().insert(false);
	let select_value = cx.app.models_mut().insert(Some("$".into()));
	let amount_value = cx.app.models_mut().insert(String::new());

	let currency = shadcn::Select::new(select_value.clone(), select_open.clone())
	    .placeholder("$")
	    .trigger_value_as_label()
	    .trigger_font_monospace()
	    .content(shadcn::SelectContent::new().align_item_with_trigger(false))
	    .align(shadcn::SelectAlign::Start)
	    .items([
	        shadcn::SelectItem::new("$", "US Dollar").show_value_in_list(true),
	        shadcn::SelectItem::new("€", "Euro").show_value_in_list(true),
	        shadcn::SelectItem::new("£", "British Pound").show_value_in_list(true),
	    ])
	    ;

let amount = shadcn::Input::new(amount_value)
    .a11y_label("Amount")
    .placeholder("10.00")
    ;

let send = shadcn::Button::new("")
    .a11y_label("Send")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .icon(fret_icons::IconId::new_static("lucide.arrow-right"));

shadcn::ButtonGroup::new([
    shadcn::ButtonGroup::new([currency.into(), amount.into()]).into(),
    shadcn::ButtonGroup::new([send.into()]).into(),
])
.refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(760.0)))
.into_element(cx);"#,
                ),
            DocSection::new("Popover", popover)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-popover")
                .code(
                    "rust",
                    r#"let open_model = cx.app.models_mut().insert(false);
		let text_model = cx.app.models_mut().insert(String::new());

let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
let corners_last = fret_core::Corners {
    top_left: fret_core::Px(0.0),
    bottom_left: fret_core::Px(0.0),
    top_right: radius,
    bottom_right: radius,
};
	
	let popover = shadcn::Popover::new(open_model.clone())
	    .side(shadcn::PopoverSide::Bottom)
	    .align(shadcn::PopoverAlign::End)
	    .into_element(
	        cx,
	        |cx| {
	            shadcn::Button::new("")
	                .a11y_label("Open popover")
	                .variant(shadcn::ButtonVariant::Outline)
	                .size(shadcn::ButtonSize::Icon)
	                .icon(fret_icons::IconId::new_static("lucide.chevron-down"))
	                .toggle_model(open_model.clone())
	                .border_left_width_override(fret_core::Px(0.0))
	                .corner_radii_override(corners_last)
	                .into_element(cx)
	        },
	        |cx| {
	            shadcn::PopoverContent::new(vec![
	                shadcn::PopoverHeader::new([
	                    shadcn::PopoverTitle::new("Start a new task with Copilot").into_element(cx),
	                    shadcn::PopoverDescription::new("Describe your task in natural language.")
	                        .into_element(cx),
	                ])
	                .into_element(cx),
	                shadcn::Field::new([
	                    fret_ui_kit::primitives::visually_hidden::visually_hidden_label(
	                        cx,
	                        "Task Description",
	                    ),
	                    shadcn::Textarea::new(text_model.clone())
	                        .a11y_label("Task Description")
	                        .placeholder("I need to...")
	                        .resizable(false)
	                        .into_element(cx),
	                    shadcn::FieldDescription::new(
	                        "Copilot will open a pull request for review.",
	                    )
	                    .into_element(cx),
	                ])
	                .into_element(cx),
	            ])
	            .refine_style(ChromeRefinement::default().rounded(Radius::Xl))
	            .into_element(cx)
	        },
	    );

shadcn::ButtonGroup::new([
    shadcn::Button::new("Copilot")
        .variant(shadcn::ButtonVariant::Outline)
        .leading_icon(fret_icons::IconId::new_static("lucide.bot"))
        .into(),
    popover.into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-button-group-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("التالي").variant(shadcn::ButtonVariant::Outline).into(),
        shadcn::Button::new("السابق").variant(shadcn::ButtonVariant::Outline).into(),
    ])
    .into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-button-group-notes"),
        ],
    );

    vec![body]
}
