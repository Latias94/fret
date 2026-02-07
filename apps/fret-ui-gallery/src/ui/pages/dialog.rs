use super::super::*;

pub(super) fn preview_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DialogPageModels {
        custom_open: Option<Model<bool>>,
        no_close_open: Option<Model<bool>>,
        sticky_open: Option<Model<bool>>,
        scrollable_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        demo_name: Option<Model<String>>,
        demo_username: Option<Model<String>>,
        rtl_name: Option<Model<String>>,
        rtl_username: Option<Model<String>>,
        share_link: Option<Model<String>>,
    }

    let (
        custom_open,
        no_close_open,
        sticky_open,
        scrollable_open,
        rtl_open,
        demo_name,
        demo_username,
        rtl_name,
        rtl_username,
        share_link,
    ) = cx.with_state(DialogPageModels::default, |st| {
        (
            st.custom_open.clone(),
            st.no_close_open.clone(),
            st.sticky_open.clone(),
            st.scrollable_open.clone(),
            st.rtl_open.clone(),
            st.demo_name.clone(),
            st.demo_username.clone(),
            st.rtl_name.clone(),
            st.rtl_username.clone(),
            st.share_link.clone(),
        )
    });

    let (
        custom_open,
        no_close_open,
        sticky_open,
        scrollable_open,
        rtl_open,
        demo_name,
        demo_username,
        rtl_name,
        rtl_username,
        share_link,
    ) = match (
        custom_open,
        no_close_open,
        sticky_open,
        scrollable_open,
        rtl_open,
        demo_name,
        demo_username,
        rtl_name,
        rtl_username,
        share_link,
    ) {
        (
            Some(custom_open),
            Some(no_close_open),
            Some(sticky_open),
            Some(scrollable_open),
            Some(rtl_open),
            Some(demo_name),
            Some(demo_username),
            Some(rtl_name),
            Some(rtl_username),
            Some(share_link),
        ) => (
            custom_open,
            no_close_open,
            sticky_open,
            scrollable_open,
            rtl_open,
            demo_name,
            demo_username,
            rtl_name,
            rtl_username,
            share_link,
        ),
        _ => {
            let custom_open = cx.app.models_mut().insert(false);
            let no_close_open = cx.app.models_mut().insert(false);
            let sticky_open = cx.app.models_mut().insert(false);
            let scrollable_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);
            let demo_name = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            let demo_username = cx.app.models_mut().insert(String::from("@peduarte"));
            let rtl_name = cx.app.models_mut().insert(String::from("RTL user"));
            let rtl_username = cx.app.models_mut().insert(String::from("@fret-user"));
            let share_link = cx
                .app
                .models_mut()
                .insert(String::from("https://ui.shadcn.com/docs/components/dialog"));

            cx.with_state(DialogPageModels::default, |st| {
                st.custom_open = Some(custom_open.clone());
                st.no_close_open = Some(no_close_open.clone());
                st.sticky_open = Some(sticky_open.clone());
                st.scrollable_open = Some(scrollable_open.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.demo_name = Some(demo_name.clone());
                st.demo_username = Some(demo_username.clone());
                st.rtl_name = Some(rtl_name.clone());
                st.rtl_username = Some(rtl_username.clone());
                st.share_link = Some(share_link.clone());
            });

            (
                custom_open,
                no_close_open,
                sticky_open,
                scrollable_open,
                rtl_open,
                demo_name,
                demo_username,
                rtl_name,
                rtl_username,
                share_link,
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

    let lorem_block = |cx: &mut ElementContext<'_, App>, prefix: &'static str, lines: usize| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                (0..lines)
                    .map(|index| {
                        cx.text(format!(
                            "{prefix} {}: This dialog row is intentionally verbose to validate scroll behavior and footer visibility.",
                            index + 1
                        ))
                    })
                    .collect::<Vec<_>>()
            },
        )
    };

    let demo = {
        let trigger_open = open.clone();
        let close_open = open.clone();
        let save_open = open.clone();
        let name_model = demo_name.clone();
        let username_model = demo_username.clone();

        let content = shadcn::Dialog::new(open.clone())
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Open Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-demo-trigger")
                        .toggle_model(trigger_open.clone())
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DialogContent::new([
                        shadcn::DialogClose::new(close_open.clone()).into_element(cx),
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Edit profile").into_element(cx),
                            shadcn::DialogDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        profile_fields(cx, name_model.clone(), username_model.clone()),
                        shadcn::DialogFooter::new([
                            shadcn::Button::new("Cancel")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Save changes")
                                .toggle_model(save_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id("ui-gallery-dialog-demo-content"),
                    )
                },
            )
            .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-dialog-demo"));

        section_card(cx, "Demo", content)
    };

    let custom_close = {
        let open_for_trigger = custom_open.clone();
        let open_for_footer = custom_open.clone();
        let link_model = share_link.clone();

        let content = shadcn::Dialog::new(custom_open.clone())
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Share")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-custom-close-trigger")
                        .toggle_model(open_for_trigger.clone())
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DialogContent::new([
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Share link").into_element(cx),
                            shadcn::DialogDescription::new(
                                "Replace the close affordance with a custom footer action.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::Input::new(link_model.clone())
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        shadcn::DialogFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-dialog-custom-close-footer")
                            .toggle_model(open_for_footer.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-dialog-custom-close-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-dialog-custom-close"),
            );

        section_card(cx, "Custom Close Button", content)
    };

    let no_close = {
        let open_for_trigger = no_close_open.clone();

        let content = shadcn::Dialog::new(no_close_open.clone()).into_element(
            cx,
            move |cx| {
                shadcn::Button::new("No Close Button")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-no-close-trigger")
                    .toggle_model(open_for_trigger.clone())
                    .into_element(cx)
            },
            move |cx| {
                shadcn::DialogContent::new([
                    shadcn::DialogHeader::new([
                        shadcn::DialogTitle::new("No Close Button").into_element(cx),
                        shadcn::DialogDescription::new(
                            "This dialog omits explicit close controls and relies on Escape or overlay dismissal.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
                .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-dialog-no-close-content"))
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-dialog-no-close"));

        section_card(cx, "No Close Button", content)
    };

    let sticky_footer = {
        let open_for_trigger = sticky_open.clone();
        let close_open = sticky_open.clone();

        let content = shadcn::Dialog::new(sticky_open.clone())
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Sticky Footer")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-sticky-footer-trigger")
                        .toggle_model(open_for_trigger.clone())
                        .into_element(cx)
                },
                move |cx| {
                    let scroll_body = shadcn::ScrollArea::new([lorem_block(cx, "Sticky", 14)])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(Px(220.0))
                                .min_w_0()
                                .min_h_0(),
                        )
                        .viewport_test_id("ui-gallery-dialog-sticky-footer-viewport")
                        .into_element(cx);

                    shadcn::DialogContent::new([
                        shadcn::DialogClose::new(close_open.clone()).into_element(cx),
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Sticky Footer").into_element(cx),
                            shadcn::DialogDescription::new(
                                "The footer remains visible while the content area scrolls.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        scroll_body,
                        shadcn::DialogFooter::new([
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Save changes").into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-dialog-sticky-footer-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-dialog-sticky-footer"),
            );

        section_card(cx, "Sticky Footer", content)
    };

    let scrollable_content = {
        let open_for_trigger = scrollable_open.clone();
        let close_open = scrollable_open.clone();

        let content = shadcn::Dialog::new(scrollable_open.clone())
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Scrollable Content")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-scrollable-trigger")
                        .toggle_model(open_for_trigger.clone())
                        .into_element(cx)
                },
                move |cx| {
                    let scroll_body = shadcn::ScrollArea::new([lorem_block(cx, "Scrollable", 14)])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(Px(240.0))
                                .min_w_0()
                                .min_h_0(),
                        )
                        .viewport_test_id("ui-gallery-dialog-scrollable-viewport")
                        .into_element(cx);

                    shadcn::DialogContent::new([
                        shadcn::DialogClose::new(close_open.clone()).into_element(cx),
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Scrollable Content").into_element(cx),
                            shadcn::DialogDescription::new(
                                "Long content can scroll while the header stays in view.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        scroll_body,
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-dialog-scrollable-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-dialog-scrollable"),
            );

        section_card(cx, "Scrollable Content", content)
    };

    let rtl = {
        let open_for_trigger = rtl_open.clone();
        let close_open = rtl_open.clone();
        let save_open = rtl_open.clone();
        let name_model = rtl_name.clone();
        let username_model = rtl_username.clone();

        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            move |cx| {
                shadcn::Dialog::new(rtl_open.clone()).into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open RTL Dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-dialog-rtl-trigger")
                            .toggle_model(open_for_trigger.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::DialogContent::new([
                            shadcn::DialogClose::new(close_open.clone()).into_element(cx),
                            shadcn::DialogHeader::new([
                                shadcn::DialogTitle::new("RTL Profile").into_element(cx),
                                shadcn::DialogDescription::new(
                                    "This example renders dialog layout in right-to-left direction.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            profile_fields(cx, name_model.clone(), username_model.clone()),
                            shadcn::DialogFooter::new([
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(close_open.clone())
                                    .into_element(cx),
                                shadcn::Button::new("Save")
                                    .toggle_model(save_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                        .attach_semantics(
                            SemanticsDecoration::default().test_id("ui-gallery-dialog-rtl-content"),
                        )
                    },
                )
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-dialog-rtl"));

        section_card(cx, "RTL", content)
    };

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Dialog docs order: Demo, Custom Close Button, No Close Button, Sticky Footer, Scrollable Content, RTL.",
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
                custom_close,
                no_close,
                sticky_footer,
                scrollable_content,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-dialog-component"));

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
                    "Basic Dialog",
                    r#"let dialog = shadcn::Dialog::new(open).into_element(
    cx,
    |cx| shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx),
    |cx| {
        shadcn::DialogContent::new([
            shadcn::DialogClose::new(open.clone()).into_element(cx),
            shadcn::DialogHeader::new([
                shadcn::DialogTitle::new("Edit profile").into_element(cx),
            ]).into_element(cx),
        ]).into_element(cx)
    },
);"#,
                ),
                code_block(
                    cx,
                    "No Close Button",
                    r#"shadcn::DialogContent::new([
    shadcn::DialogHeader::new([
        shadcn::DialogTitle::new("No Close Button").into_element(cx),
    ]).into_element(cx),
]).into_element(cx); // omit DialogClose"#,
                ),
                code_block(
                    cx,
                    "Scrollable + Sticky Footer",
                    r#"let body = shadcn::ScrollArea::new([rows])
    .refine_layout(LayoutRefinement::default().h_px(Px(220.0)))
    .into_element(cx);

shadcn::DialogContent::new([
    shadcn::DialogHeader::new([title, description]).into_element(cx),
    body,
    shadcn::DialogFooter::new([close_button, save_button]).into_element(cx),
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
                    "Docs parity uses the same section sequence as upstream: custom close, no close, sticky footer, scrollable content, then RTL.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current Fret API models close controls explicitly with DialogClose; omitting it is equivalent to showCloseButton={false} in shadcn docs.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Scrollable examples isolate long content in ScrollArea so footer/header placement remains predictable under constrained viewport sizes.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each scenario has stable test IDs to support fretboard diag scripts and regression screenshots.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-dialog",
        component_panel,
        code_panel,
        notes_panel,
    )
}
