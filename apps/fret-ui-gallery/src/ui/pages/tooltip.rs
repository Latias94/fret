use super::super::*;

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
                LayoutRefinement::default().w_full().max_w(Px(560.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let make_tooltip = |cx: &mut ElementContext<'_, App>,
                        label: &'static str,
                        side: shadcn::TooltipSide,
                        content: &'static str| {
        shadcn::Tooltip::new(
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, content)])
                .into_element(cx),
        )
        .arrow(true)
        .side(side)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
    };

    let component_panel = shadcn::TooltipProvider::new()
        .delay_duration_frames(10)
        .skip_delay_duration_frames(5)
        .with(cx, |cx| {
            let preview_hint = shadcn::typography::muted(
                cx,
                "Preview follows shadcn Tooltip docs order for quick visual lookup.",
            );

            let demo_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Hover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx),
            )
            .arrow(true)
            .side(shadcn::TooltipSide::Top)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-demo");
            let demo = {
                let body = centered(cx, demo_tooltip);
                section(cx, "Demo", body)
            };

            let side_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        make_tooltip(cx, "Left", shadcn::TooltipSide::Left, "Add to library"),
                        make_tooltip(cx, "Top", shadcn::TooltipSide::Top, "Add to library"),
                        make_tooltip(cx, "Bottom", shadcn::TooltipSide::Bottom, "Add to library"),
                        make_tooltip(cx, "Right", shadcn::TooltipSide::Right, "Add to library"),
                    ]
                },
            )
            .test_id("ui-gallery-tooltip-sides");
            let side = {
                let body = centered(cx, side_row);
                section(cx, "Side", body)
            };

            let keyboard_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconSm)
                    .children([shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.save"),
                    )])
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            cx.text("Save Changes"),
                            shadcn::Kbd::new("S").into_element(cx),
                        ]
                    },
                )])
                .into_element(cx),
            )
            .side(shadcn::TooltipSide::Top)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-keyboard");
            let with_keyboard = {
                let body = centered(cx, keyboard_tooltip);
                section(cx, "With Keyboard Shortcut", body)
            };

            let disabled_trigger =
                stack::hstack(cx, stack::HStackProps::default().items_center(), |cx| {
                    vec![
                        shadcn::Button::new("Disabled")
                            .variant(shadcn::ButtonVariant::Outline)
                            .disabled(true)
                            .into_element(cx),
                    ]
                });
            let disabled_tooltip = shadcn::Tooltip::new(
                disabled_trigger,
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "This feature is currently unavailable",
                )])
                .into_element(cx),
            )
            .side(shadcn::TooltipSide::Top)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-disabled");
            let disabled = {
                let body = centered(cx, disabled_tooltip);
                section(cx, "Disabled Button", body)
            };

            let rtl_row = fret_ui_kit::primitives::direction::with_direction_provider(
                cx,
                fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
                |cx| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                make_tooltip(
                                    cx,
                                    "يسار",
                                    shadcn::TooltipSide::Left,
                                    "إضافة إلى المكتبة",
                                ),
                                make_tooltip(
                                    cx,
                                    "أعلى",
                                    shadcn::TooltipSide::Top,
                                    "إضافة إلى المكتبة",
                                ),
                                make_tooltip(
                                    cx,
                                    "أسفل",
                                    shadcn::TooltipSide::Bottom,
                                    "إضافة إلى المكتبة",
                                ),
                                make_tooltip(
                                    cx,
                                    "يمين",
                                    shadcn::TooltipSide::Right,
                                    "إضافة إلى المكتبة",
                                ),
                            ]
                        },
                    )
                },
            )
            .test_id("ui-gallery-tooltip-rtl");
            let rtl = {
                let body = centered(cx, rtl_row);
                section(cx, "RTL", body)
            };

            let component_stack = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N6)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![preview_hint, demo, side, with_keyboard, disabled, rtl],
            );
            let component_shell =
                shell(cx, component_stack).test_id("ui-gallery-tooltip-component");
            vec![component_shell]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Demo",
                    "Tooltip::new(trigger, content)\n    .side(TooltipSide::Top)\n    .open_delay_frames(10)\n    .close_delay_frames(10)",
                ),
                code_block(
                    cx,
                    "Side",
                    "for side in [Left, Top, Bottom, Right] {\n    Tooltip::new(Button::new(side), TooltipContent::text(...)).side(side)\n}",
                ),
                code_block(
                    cx,
                    "Disabled + RTL",
                    "TooltipProvider::new().with(...)\nTooltip::new(disabled_button_wrapper, content)\nwith_direction_provider(LayoutDirection::Rtl, ...)",
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
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
                    "Wrap related tooltips in one TooltipProvider to get consistent delay-group behavior.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-tooltip",
        component_panel,
        code_panel,
        notes_panel,
    )
}
