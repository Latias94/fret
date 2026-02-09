use super::super::*;

pub(super) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DrawerPageModels {
        demo_open: Option<Model<bool>>,
        scroll_open: Option<Model<bool>>,
        side_top_open: Option<Model<bool>>,
        side_right_open: Option<Model<bool>>,
        side_bottom_open: Option<Model<bool>>,
        side_left_open: Option<Model<bool>>,
        responsive_dialog_open: Option<Model<bool>>,
        responsive_drawer_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
    }

    let (
        demo_open,
        scroll_open,
        side_top_open,
        side_right_open,
        side_bottom_open,
        side_left_open,
        responsive_dialog_open,
        responsive_drawer_open,
        rtl_open,
    ) = cx.with_state(DrawerPageModels::default, |st| {
        (
            st.demo_open.clone(),
            st.scroll_open.clone(),
            st.side_top_open.clone(),
            st.side_right_open.clone(),
            st.side_bottom_open.clone(),
            st.side_left_open.clone(),
            st.responsive_dialog_open.clone(),
            st.responsive_drawer_open.clone(),
            st.rtl_open.clone(),
        )
    });

    let (
        demo_open,
        scroll_open,
        side_top_open,
        side_right_open,
        side_bottom_open,
        side_left_open,
        responsive_dialog_open,
        responsive_drawer_open,
        rtl_open,
    ) = match (
        demo_open,
        scroll_open,
        side_top_open,
        side_right_open,
        side_bottom_open,
        side_left_open,
        responsive_dialog_open,
        responsive_drawer_open,
        rtl_open,
    ) {
        (
            Some(demo_open),
            Some(scroll_open),
            Some(side_top_open),
            Some(side_right_open),
            Some(side_bottom_open),
            Some(side_left_open),
            Some(responsive_dialog_open),
            Some(responsive_drawer_open),
            Some(rtl_open),
        ) => (
            demo_open,
            scroll_open,
            side_top_open,
            side_right_open,
            side_bottom_open,
            side_left_open,
            responsive_dialog_open,
            responsive_drawer_open,
            rtl_open,
        ),
        _ => {
            let demo_open = cx.app.models_mut().insert(false);
            let scroll_open = cx.app.models_mut().insert(false);
            let side_top_open = cx.app.models_mut().insert(false);
            let side_right_open = cx.app.models_mut().insert(false);
            let side_bottom_open = cx.app.models_mut().insert(false);
            let side_left_open = cx.app.models_mut().insert(false);
            let responsive_dialog_open = cx.app.models_mut().insert(false);
            let responsive_drawer_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);

            cx.with_state(DrawerPageModels::default, |st| {
                st.demo_open = Some(demo_open.clone());
                st.scroll_open = Some(scroll_open.clone());
                st.side_top_open = Some(side_top_open.clone());
                st.side_right_open = Some(side_right_open.clone());
                st.side_bottom_open = Some(side_bottom_open.clone());
                st.side_left_open = Some(side_left_open.clone());
                st.responsive_dialog_open = Some(responsive_dialog_open.clone());
                st.responsive_drawer_open = Some(responsive_drawer_open.clone());
                st.rtl_open = Some(rtl_open.clone());
            });

            (
                demo_open,
                scroll_open,
                side_top_open,
                side_right_open,
                side_bottom_open,
                side_left_open,
                responsive_dialog_open,
                responsive_drawer_open,
                rtl_open,
            )
        }
    };

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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(780.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let paragraph_block = |cx: &mut ElementContext<'_, App>, prefix: &'static str, rows: usize| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                (0..rows)
                    .map(|index| {
                        cx.text(format!(
                            "{prefix} {}: Drawer scroll content for parity checks.",
                            index + 1
                        ))
                    })
                    .collect::<Vec<_>>()
            },
        )
    };

    let demo = {
        let trigger_open = demo_open.clone();
        let close_open = demo_open.clone();

        let drawer = shadcn::Drawer::new(demo_open.clone()).into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Open Drawer")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .test_id("ui-gallery-drawer-demo-trigger")
                    .into_element(cx)
            },
            move |cx| {
                shadcn::DrawerContent::new([
                    shadcn::DrawerHeader::new([
                        shadcn::DrawerTitle::new("Move Goal").into_element(cx),
                        shadcn::DrawerDescription::new("Set your daily activity goal.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DrawerFooter::new([
                        shadcn::Button::new("Submit").into_element(cx),
                        shadcn::Button::new("Cancel")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(close_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-drawer-demo-content")
            },
        );

        section_card(cx, "Demo", drawer).test_id("ui-gallery-drawer-demo")
    };

    let scrollable_content = {
        let trigger_open = scroll_open.clone();
        let close_open = scroll_open.clone();
        let theme_for_scrollable_content = theme.clone();

        let drawer = shadcn::Drawer::new(scroll_open.clone())
            .side(shadcn::DrawerSide::Right)
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Scrollable Content")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .test_id("ui-gallery-drawer-scrollable-trigger")
                        .into_element(cx)
                },
                move |cx| {
                    let scroller = shadcn::ScrollArea::new([paragraph_block(cx, "Scrollable", 14)])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(Px(220.0))
                                .min_w_0()
                                .min_h_0(),
                        )
                        .viewport_test_id("ui-gallery-drawer-scrollable-viewport")
                        .into_element(cx);

                    shadcn::DrawerContent::new([
                        shadcn::DrawerHeader::new([
                            shadcn::DrawerTitle::new("Scrollable Content").into_element(cx),
                            shadcn::DrawerDescription::new(
                                "Keep actions visible while the content scrolls.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        cx.container(
                            decl_style::container_props(
                                &theme_for_scrollable_content,
                                ChromeRefinement::default().px(Space::N4),
                                LayoutRefinement::default().w_full(),
                            ),
                            move |_cx| [scroller],
                        ),
                        shadcn::DrawerFooter::new([
                            shadcn::Button::new("Submit").into_element(cx),
                            shadcn::Button::new("Cancel")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-drawer-scrollable-content"),
                    )
                },
            );

        section_card(cx, "Scrollable Content", drawer).test_id("ui-gallery-drawer-scrollable")
    };

    let sides = {
        let side_button = |cx: &mut ElementContext<'_, App>,
                           title: &'static str,
                           side: shadcn::DrawerSide,
                           open: Model<bool>,
                           test_id_prefix: &'static str| {
            let open_for_trigger = open.clone();
            let open_for_close = open.clone();
            shadcn::Drawer::new(open.clone()).side(side).into_element(
                cx,
                move |cx| {
                    shadcn::Button::new(title)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open_for_trigger.clone())
                        .test_id(format!("{test_id_prefix}-trigger"))
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DrawerContent::new([
                        shadcn::DrawerHeader::new([
                            shadcn::DrawerTitle::new(format!("{title} Drawer")).into_element(cx),
                            shadcn::DrawerDescription::new(
                                "Use the `side` prop to control drawer placement.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id(format!("{test_id_prefix}-content")),
                    )
                },
            )
        };

        let buttons = ui::h_flex(cx, |cx| {
            vec![
                side_button(
                    cx,
                    "Top",
                    shadcn::DrawerSide::Top,
                    side_top_open.clone(),
                    "ui-gallery-drawer-side-top",
                ),
                side_button(
                    cx,
                    "Right",
                    shadcn::DrawerSide::Right,
                    side_right_open.clone(),
                    "ui-gallery-drawer-side-right",
                ),
                side_button(
                    cx,
                    "Bottom",
                    shadcn::DrawerSide::Bottom,
                    side_bottom_open.clone(),
                    "ui-gallery-drawer-side-bottom",
                ),
                side_button(
                    cx,
                    "Left",
                    shadcn::DrawerSide::Left,
                    side_left_open.clone(),
                    "ui-gallery-drawer-side-left",
                ),
            ]
        })
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
        .test_id("ui-gallery-drawer-sides");

        section_card(cx, "Sides", buttons)
    };

    let responsive_dialog = {
        let desktop_open_trigger = responsive_dialog_open.clone();
        let desktop_open_close = responsive_dialog_open.clone();
        let mobile_open_trigger = responsive_drawer_open.clone();
        let mobile_open_close = responsive_drawer_open.clone();

        let desktop_dialog = shadcn::Dialog::new(responsive_dialog_open.clone()).into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Desktop Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(desktop_open_trigger.clone())
                    .test_id("ui-gallery-drawer-responsive-desktop-trigger")
                    .into_element(cx)
            },
            move |cx| {
                shadcn::DialogContent::new([
                    shadcn::DialogHeader::new([
                        shadcn::DialogTitle::new("Responsive Dialog").into_element(cx),
                        shadcn::DialogDescription::new(
                            "Desktop branch uses Dialog in the responsive pattern.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DialogFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(desktop_open_close.clone())
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-drawer-responsive-desktop-content"),
                )
            },
        );

        let mobile_drawer = shadcn::Drawer::new(responsive_drawer_open.clone()).into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Mobile Drawer")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(mobile_open_trigger.clone())
                    .test_id("ui-gallery-drawer-responsive-mobile-trigger")
                    .into_element(cx)
            },
            move |cx| {
                shadcn::DrawerContent::new([
                    shadcn::DrawerHeader::new([
                        shadcn::DrawerTitle::new("Responsive Drawer").into_element(cx),
                        shadcn::DrawerDescription::new(
                            "Mobile branch uses Drawer in the responsive pattern.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(mobile_open_close.clone())
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-drawer-responsive-mobile-content"),
                )
            },
        );

        let row = ui::h_flex(cx, move |_cx| [desktop_dialog, mobile_drawer])
            .gap(Space::N2)
            .wrap()
            .w_full()
            .items_center()
            .into_element(cx);

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    row,
                    shadcn::typography::muted(
                        cx,
                        "Current gallery renders both branches explicitly for deterministic testing instead of runtime viewport switches.",
                    ),
                ]
            },
        );

        section_card(cx, "Responsive Dialog", body)
    };

    let rtl = {
        let open_for_trigger = rtl_open.clone();
        let open_for_close = rtl_open.clone();

        let drawer = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            move |cx| {
                shadcn::Drawer::new(rtl_open.clone()).into_element(
                    cx,
                    move |cx| {
                        shadcn::Button::new("Open RTL Drawer")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_trigger.clone())
                            .test_id("ui-gallery-drawer-rtl-trigger")
                            .into_element(cx)
                    },
                    move |cx| {
                        shadcn::DrawerContent::new([
                            shadcn::DrawerHeader::new([
                                shadcn::DrawerTitle::new("RTL Drawer").into_element(cx),
                                shadcn::DrawerDescription::new(
                                    "Drawer layout should follow right-to-left direction context.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(open_for_close.clone())
                                .into_element(cx)])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-drawer-rtl-content")
                    },
                )
            },
        )
        .test_id("ui-gallery-drawer-rtl");

        section_card(cx, "RTL", drawer)
    };

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Drawer docs order: Demo, Scrollable Content, Sides, Responsive Dialog, RTL.",
    );

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                scrollable_content,
                sides,
                responsive_dialog,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-drawer-component");

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
                    "Basic Drawer",
                    r#"let drawer = shadcn::Drawer::new(open).into_element(
    cx,
    |cx| shadcn::Button::new("Open Drawer").toggle_model(open.clone()).into_element(cx),
    |cx| {
        shadcn::DrawerContent::new([
            shadcn::DrawerHeader::new([title, description]).into_element(cx),
            shadcn::DrawerFooter::new([submit, cancel]).into_element(cx),
        ]).into_element(cx)
    },
);"#,
                ),
                code_block(
                    cx,
                    "Sides",
                    r#"shadcn::Drawer::new(open)
    .side(shadcn::DrawerSide::Right)
    .into_element(cx, trigger, content);"#,
                ),
                code_block(
                    cx,
                    "Scrollable Content",
                    r#"let body = shadcn::ScrollArea::new([rows])
    .refine_layout(LayoutRefinement::default().h_px(Px(220.0)))
    .into_element(cx);

shadcn::DrawerContent::new([
    shadcn::DrawerHeader::new([title, description]).into_element(cx),
    body,
    shadcn::DrawerFooter::new([submit, cancel]).into_element(cx),
]).into_element(cx);"#,
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
                    "Docs parity follows the upstream order: scrollable content and sides are explicit recipes after the basic demo.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Responsive dialog recipe is represented as explicit desktop/mobile branches for deterministic gallery validation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use stable test IDs on every scenario so diag scripts can capture open/close and layout outcomes reliably.",
                ),
                shadcn::typography::muted(
                    cx,
                    "DrawerClose-as-child composition is not modeled yet; current examples close through toggle_model actions.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-drawer",
        component_panel,
        code_panel,
        notes_panel,
    )
}
