use super::super::super::super::super::*;

pub(in crate::ui) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

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
                st.search_value = Some(model.clone());
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
            let model = cx
                .app
                .models_mut()
                .insert(String::from("Describe your task in natural language."));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_text = Some(model.clone());
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let outline_fg = ColorRef::Color(theme.color_token("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_token("secondary-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    let demo = shadcn::ButtonGroup::new([
        shadcn::Button::new("Button").into(),
        shadcn::Button::new("Get Started")
            .children([icon(cx, "lucide.arrow-right", outline_fg.clone())])
            .into(),
    ])
    .a11y_label("Button group")
    .into_element(cx)
    .test_id("ui-gallery-button-group-demo");

    let orientation = shadcn::ButtonGroup::new([
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

        shadcn::ButtonGroup::new([digits.into(), nav.into()])
            .into_element(cx)
            .test_id("ui-gallery-button-group-nested")
    };

    let separator = shadcn::ButtonGroup::new([
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
    .into_element(cx)
    .test_id("ui-gallery-button-group-separator");

    let split = shadcn::ButtonGroup::new([
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
    .into_element(cx)
    .test_id("ui-gallery-button-group-split");

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
            .refine_layout(LayoutRefinement::default().w_px(Px(220.0)))
            .into_element(cx)
            .into(),
        shadcn::Button::new("Search")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .children([icon(cx, "lucide.search", outline_fg.clone())])
            .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-input");

    let input_group = {
        let group = shadcn::InputGroup::new(message_value.clone())
            .a11y_label("Message")
            .leading([shadcn::InputGroupText::new("To").into_element(cx)])
            .trailing([shadcn::InputGroupButton::new("Send").into_element(cx)]);

        shadcn::ButtonGroup::new([
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
            group.into(),
        ])
        .into_element(cx)
        .test_id("ui-gallery-button-group-input-group")
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

        shadcn::ButtonGroup::new([
            shadcn::ButtonGroup::new([currency.into(), amount.into()]).into(),
            shadcn::ButtonGroup::new([send.into()]).into(),
        ])
        .into_element(cx)
        .test_id("ui-gallery-button-group-select")
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

        shadcn::ButtonGroup::new([
            shadcn::Button::new("Copilot")
                .variant(shadcn::ButtonVariant::Outline)
                .children([icon(cx, "lucide.bot", outline_fg.clone())])
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
            "Fret currently uses `Separator` between group items; upstream also exposes `ButtonGroupSeparator` / `ButtonGroupText` variants.",
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
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Button").into(),
    shadcn::Button::new("Get Started").into(),
]).into_element(cx);"#,
                ),
            DocSection::new("Orientation", orientation)
                .test_id_prefix("ui-gallery-button-group-orientation")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([...])
    .orientation(shadcn::ButtonGroupOrientation::Vertical)
    .into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .no_shell()
                .test_id_prefix("ui-gallery-button-group-size")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Small")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into(),
    shadcn::Button::new("Add")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .children([icon(cx, "lucide.plus", fg)])
        .into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Nested", nested)
                .test_id_prefix("ui-gallery-button-group-nested")
                .code(
                    "rust",
                    r#"let digits = shadcn::ButtonGroup::new([
    shadcn::Button::new("1").variant(shadcn::ButtonVariant::Outline).into(),
    shadcn::Button::new("2").variant(shadcn::ButtonVariant::Outline).into(),
]);

let nav = shadcn::ButtonGroup::new([
    shadcn::Button::new("Prev").variant(shadcn::ButtonVariant::Outline).into(),
    shadcn::Button::new("Next").variant(shadcn::ButtonVariant::Outline).into(),
]);

shadcn::ButtonGroup::new([digits.into(), nav.into()]).into_element(cx);"#,
                ),
            DocSection::new("Separator", separator)
                .test_id_prefix("ui-gallery-button-group-separator")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Copy").variant(shadcn::ButtonVariant::Secondary).into(),
    shadcn::Separator::new()
        .orientation(shadcn::SeparatorOrientation::Vertical)
        .into(),
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
    shadcn::Separator::new()
        .orientation(shadcn::SeparatorOrientation::Vertical)
        .into(),
    shadcn::Button::new("Add")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.plus", fg)])
        .into(),
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
                    r#"shadcn::ButtonGroup::new([
    shadcn::Input::new(model).placeholder("Search...").into_element(cx).into(),
    shadcn::Button::new("Search").into(),
]).into_element(cx);"#,
                ),
            DocSection::new("Input Group", input_group)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-input-group")
                .code(
                    "rust",
                    r#"let group = shadcn::InputGroup::new(model)
    .leading([shadcn::InputGroupText::new("To").into_element(cx)])
    .trailing([shadcn::InputGroupButton::new("Send").into_element(cx)]);

shadcn::ButtonGroup::new([
    shadcn::Button::new("Add").variant(shadcn::ButtonVariant::Outline).into(),
    group.into(),
])
.into_element(cx);"#,
                ),
            DocSection::new("Dropdown Menu", dropdown)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-dropdown")
                .code(
                    "rust",
                    r#"let menu = shadcn::DropdownMenu::new(open_model).into_element(
    cx,
    |cx| shadcn::Button::new("More").toggle_model(open_model).into_element(cx),
    |_cx| vec![shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Profile"))],
);

shadcn::ButtonGroup::new([shadcn::Button::new("Edit").into(), menu.into()])
    .into_element(cx);"#,
                ),
            DocSection::new("Select", select)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-select")
                .code(
                    "rust",
                    r#"let select = shadcn::Select::new(value_model, open_model)
    .into_element(
        cx,
        |cx, trigger| trigger.into_element(cx),
        |_cx| vec![shadcn::SelectEntry::Item(shadcn::SelectItem::new("Apple"))],
    );

shadcn::ButtonGroup::new([select.into(), shadcn::Button::new("Apply").into()])
    .into_element(cx);"#,
                ),
            DocSection::new("Popover", popover)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-popover")
                .code(
                    "rust",
                    r#"let popover = shadcn::Popover::new(open_model)
    .into_element(cx, |cx| trigger_button, |cx| popover_content);

shadcn::ButtonGroup::new([shadcn::Button::new("Filter").into(), popover.into()])
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
