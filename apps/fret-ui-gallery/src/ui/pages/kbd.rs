use super::super::*;

pub(super) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct KbdPageModels {
        input_group_value: Option<Model<String>>,
    }

    let input_group_value =
        cx.with_state(KbdPageModels::default, |st| st.input_group_value.clone());
    let input_group_value = match input_group_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(KbdPageModels::default, |st| {
                st.input_group_value = Some(model.clone())
            });
            model
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

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let demo = {
        let content = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    shadcn::Kbd::new("K").into_element(cx),
                    shadcn::Kbd::new("Enter").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-kbd-demo");
        section_card(cx, "Demo", content)
    };

    let group = {
        let content = shadcn::KbdGroup::new([
            shadcn::Kbd::new("Cmd").into_element(cx),
            shadcn::Kbd::new("Shift").into_element(cx),
            shadcn::Kbd::new("P").into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-kbd-group");
        section_card(cx, "Group", content)
    };

    let button = {
        let content = shadcn::Button::new("Command Palette")
            .variant(shadcn::ButtonVariant::Outline)
            .children([shadcn::KbdGroup::new([
                shadcn::Kbd::new("Cmd").into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
            ])
            .into_element(cx)])
            .on_click(CMD_APP_OPEN)
            .into_element(cx)
            .test_id("ui-gallery-kbd-button");
        section_card(cx, "Button", content)
    };

    let tooltip = {
        let content = shadcn::TooltipProvider::new()
            .delay_duration_frames(10)
            .skip_delay_duration_frames(5)
            .with(cx, |cx| {
                vec![
                    shadcn::Tooltip::new(
                        shadcn::Button::new("Save")
                            .variant(shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        shadcn::TooltipContent::new(vec![stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |cx| {
                                vec![
                                    cx.text("Save file"),
                                    shadcn::Kbd::new("Cmd").into_element(cx),
                                    shadcn::Kbd::new("S").into_element(cx),
                                ]
                            },
                        )])
                        .into_element(cx),
                    )
                    .arrow(true)
                    .open_delay_frames(10)
                    .close_delay_frames(10)
                    .into_element(cx)
                    .test_id("ui-gallery-kbd-tooltip"),
                ]
            })
            .into_iter()
            .next()
            .expect("kbd tooltip provider should return one root");

        section_card(cx, "Tooltip", content)
    };

    let input_group = {
        let content = shadcn::InputGroup::new(input_group_value)
            .a11y_label("Search")
            .trailing([shadcn::KbdGroup::new([
                shadcn::Kbd::new("Ctrl").into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
            ])
            .into_element(cx)])
            .trailing_has_kbd(true)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
            .into_element(cx)
            .test_id("ui-gallery-kbd-input-group");

        section_card(cx, "Input Group", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    shadcn::Kbd::new("Shift").into_element(cx),
                    shadcn::Kbd::new("B").into_element(cx),
                ])
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-kbd-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Kbd docs order: Demo, Group, Button, Tooltip, Input Group, RTL.",
                ),
                demo,
                group,
                button,
                tooltip,
                input_group,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_panel_body).test_id("ui-gallery-kbd-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Base Usage").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(cx, r#"Kbd::new("Ctrl")"#).into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Group and Button").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"Button::new("Command Palette").children([KbdGroup::new([Kbd::new("Cmd"), Kbd::new("K")])])"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Input Group").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"InputGroup::new(model).trailing([KbdGroup::new([Kbd::new("Ctrl"), Kbd::new("K")])]).trailing_has_kbd(true)"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Kbd uses tokenized muted surfaces and is intended for shortcut display rather than free text chips.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`Tooltip` and `Input Group` examples are composition patterns from shadcn docs.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-kbd",
        component_panel,
        code_panel,
        notes_panel,
    )
}
