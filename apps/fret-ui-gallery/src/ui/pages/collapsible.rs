use super::super::*;

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CollapsibleModels {
        controlled_open: Option<Model<bool>>,
        settings_open: Option<Model<bool>>,
        radius_x: Option<Model<String>>,
        radius_y: Option<Model<String>>,
        radius_bl: Option<Model<String>>,
        radius_br: Option<Model<String>>,
        tree_components_open: Option<Model<bool>>,
        tree_src_open: Option<Model<bool>>,
        tree_src_ui_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
    }

    let (
        controlled_open,
        settings_open,
        radius_x,
        radius_y,
        radius_bl,
        radius_br,
        tree_components_open,
        tree_src_open,
        tree_src_ui_open,
        rtl_open,
    ) = cx.with_state(CollapsibleModels::default, |st| {
        (
            st.controlled_open.clone(),
            st.settings_open.clone(),
            st.radius_x.clone(),
            st.radius_y.clone(),
            st.radius_bl.clone(),
            st.radius_br.clone(),
            st.tree_components_open.clone(),
            st.tree_src_open.clone(),
            st.tree_src_ui_open.clone(),
            st.rtl_open.clone(),
        )
    });

    let (
        controlled_open,
        settings_open,
        radius_x,
        radius_y,
        radius_bl,
        radius_br,
        tree_components_open,
        tree_src_open,
        tree_src_ui_open,
        rtl_open,
    ) = match (
        controlled_open,
        settings_open,
        radius_x,
        radius_y,
        radius_bl,
        radius_br,
        tree_components_open,
        tree_src_open,
        tree_src_ui_open,
        rtl_open,
    ) {
        (
            Some(controlled_open),
            Some(settings_open),
            Some(radius_x),
            Some(radius_y),
            Some(radius_bl),
            Some(radius_br),
            Some(tree_components_open),
            Some(tree_src_open),
            Some(tree_src_ui_open),
            Some(rtl_open),
        ) => (
            controlled_open,
            settings_open,
            radius_x,
            radius_y,
            radius_bl,
            radius_br,
            tree_components_open,
            tree_src_open,
            tree_src_ui_open,
            rtl_open,
        ),
        _ => {
            let controlled_open = cx.app.models_mut().insert(false);
            let settings_open = cx.app.models_mut().insert(false);
            let radius_x = cx.app.models_mut().insert(String::from("0"));
            let radius_y = cx.app.models_mut().insert(String::from("0"));
            let radius_bl = cx.app.models_mut().insert(String::from("8"));
            let radius_br = cx.app.models_mut().insert(String::from("8"));
            let tree_components_open = cx.app.models_mut().insert(true);
            let tree_src_open = cx.app.models_mut().insert(false);
            let tree_src_ui_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);

            cx.with_state(CollapsibleModels::default, |st| {
                st.controlled_open = Some(controlled_open.clone());
                st.settings_open = Some(settings_open.clone());
                st.radius_x = Some(radius_x.clone());
                st.radius_y = Some(radius_y.clone());
                st.radius_bl = Some(radius_bl.clone());
                st.radius_br = Some(radius_br.clone());
                st.tree_components_open = Some(tree_components_open.clone());
                st.tree_src_open = Some(tree_src_open.clone());
                st.tree_src_ui_open = Some(tree_src_ui_open.clone());
                st.rtl_open = Some(rtl_open.clone());
            });

            (
                controlled_open,
                settings_open,
                radius_x,
                radius_y,
                radius_bl,
                radius_br,
                tree_components_open,
                tree_src_open,
                tree_src_ui_open,
                rtl_open,
            )
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

    let container_props =
        |cx: &mut ElementContext<'_, App>, chrome: ChromeRefinement, layout: LayoutRefinement| {
            cx.with_theme(|theme| decl_style::container_props(theme, chrome, layout))
        };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = container_props(
            cx,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().max_w(Px(760.0)),
        );
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let details_collapsible = |cx: &mut ElementContext<'_, App>,
                               test_id_prefix: &'static str,
                               open: Option<Model<bool>>,
                               label: &'static str,
                               status: &'static str| {
        let details_content = |cx: &mut ElementContext<'_, App>| {
            shadcn::CollapsibleContent::new(vec![
                {
                    let props = container_props(
                        cx,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Sm)
                            .px(Space::N4)
                            .py(Space::N2),
                        LayoutRefinement::default().w_full(),
                    );
                    cx.container(props, |cx| {
                        vec![stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between(),
                            |cx| {
                                vec![
                                    shadcn::typography::muted(cx, "Shipping address"),
                                    cx.text("100 Market St, San Francisco"),
                                ]
                            },
                        )]
                    })
                },
                {
                    let props = container_props(
                        cx,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Sm)
                            .px(Space::N4)
                            .py(Space::N2),
                        LayoutRefinement::default().w_full(),
                    );
                    cx.container(props, |cx| {
                        vec![stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between(),
                            |cx| {
                                vec![
                                    shadcn::typography::muted(cx, "Items"),
                                    cx.text("2x Studio Headphones"),
                                ]
                            },
                        )]
                    })
                },
            ])
            .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
            .into_element(cx)
        };

        let collapsible = match open {
            Some(open_model) => shadcn::Collapsible::new(open_model).into_element_with_open_model(
                cx,
                |cx, open, is_open| {
                    let toggle_text = if is_open {
                        "Hide details"
                    } else {
                        "Show details"
                    };
                    shadcn::Button::new(toggle_text)
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .toggle_model(open)
                        .test_id(format!("{test_id_prefix}-trigger"))
                        .into_element(cx)
                },
                |cx| details_content(cx),
            ),
            None => shadcn::Collapsible::uncontrolled(false).into_element_with_open_model(
                cx,
                |cx, open, is_open| {
                    let toggle_text = if is_open {
                        "Hide details"
                    } else {
                        "Show details"
                    };
                    shadcn::Button::new(toggle_text)
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Icon)
                        .toggle_model(open)
                        .test_id(format!("{test_id_prefix}-trigger"))
                        .into_element(cx)
                },
                |cx| details_content(cx),
            ),
        };

        let wrapper_props = container_props(
            cx,
            ChromeRefinement::default().px(Space::N3).py(Space::N2),
            LayoutRefinement::default().w_full(),
        );
        cx.container(wrapper_props, |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_between(),
                    |cx| vec![cx.text(label), cx.text(status)],
                ),
                collapsible,
            ]
        })
        .test_id(test_id_prefix)
    };

    let demo_content = details_collapsible(
        cx,
        "ui-gallery-collapsible-demo",
        None,
        "Order #4189",
        "Shipped",
    );
    let demo = section_card(cx, "Demo", demo_content);

    let controlled_now = cx
        .get_model_copied(&controlled_open, Invalidation::Layout)
        .unwrap_or(false);
    let controlled_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    if controlled_now {
                        "open=true (controlled)"
                    } else {
                        "open=false (controlled)"
                    },
                ),
                shadcn::Collapsible::new(controlled_open.clone()).into_element_with_open_model(
                    cx,
                    |cx, open, is_open| {
                        shadcn::Button::new(if is_open {
                            "Collapse"
                        } else {
                            "Expand"
                        })
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open)
                        .test_id("ui-gallery-collapsible-controlled-trigger")
                        .into_element(cx)
                    },
                    |cx| {
                        shadcn::CollapsibleContent::new(vec![
                            cx.text("This panel is controlled by `Model<bool>` and mirrors shadcn open/onOpenChange behavior."),
                        ])
                        .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
                        .into_element(cx)
                    },
                ),
            ]
        },
    )
    .test_id("ui-gallery-collapsible-controlled");
    let controlled_state = section_card(cx, "Controlled State", controlled_content);

    let basic_content = shadcn::Collapsible::uncontrolled(false)
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                shadcn::Button::new(if is_open {
                    "Product details"
                } else {
                    "Show product details"
                })
                .variant(shadcn::ButtonVariant::Ghost)
                .toggle_model(open)
                .test_id("ui-gallery-collapsible-basic-trigger")
                .into_element(cx)
            },
            |cx| {
                shadcn::CollapsibleContent::new(vec![
                    cx.text(
                        "This panel can be expanded or collapsed to reveal additional content.",
                    ),
                    shadcn::Button::new("Learn more")
                        .size(shadcn::ButtonSize::Sm)
                        .variant(shadcn::ButtonVariant::Secondary)
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-collapsible-basic");
    let basic = section_card(cx, "Basic", basic_content);

    let input_field = |cx: &mut ElementContext<'_, App>,
                       test_id: &'static str,
                       label: &'static str,
                       value: Model<String>| {
        shadcn::Field::new([
            shadcn::FieldLabel::new(label).into_element(cx),
            shadcn::Input::new(value)
                .a11y_label(label)
                .into_element(cx)
                .test_id(test_id),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let settings_content = shadcn::Collapsible::new(settings_open.clone())
        .into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                shadcn::Button::new(if is_open { "Advanced" } else { "More settings" })
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(open)
                    .test_id("ui-gallery-collapsible-settings-trigger")
                    .into_element(cx)
            },
            |cx| {
                shadcn::CollapsibleContent::new(vec![stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            input_field(
                                cx,
                                "ui-gallery-collapsible-settings-radius-bl",
                                "Bottom-left",
                                radius_bl.clone(),
                            ),
                            input_field(
                                cx,
                                "ui-gallery-collapsible-settings-radius-br",
                                "Bottom-right",
                                radius_br.clone(),
                            ),
                        ]
                    },
                )])
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-collapsible-settings");

    let settings_panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            input_field(
                                cx,
                                "ui-gallery-collapsible-settings-radius-x",
                                "Radius X",
                                radius_x.clone(),
                            ),
                            input_field(
                                cx,
                                "ui-gallery-collapsible-settings-radius-y",
                                "Radius Y",
                                radius_y.clone(),
                            ),
                        ]
                    },
                ),
                settings_content,
            ]
        },
    );
    let settings = section_card(cx, "Settings Panel", settings_panel);

    let file_leaf = |cx: &mut ElementContext<'_, App>, label: &'static str| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Sm)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
    };

    let folder = |cx: &mut ElementContext<'_, App>,
                  key: &'static str,
                  label: &'static str,
                  open_model: Model<bool>,
                  children: Vec<AnyElement>| {
        shadcn::Collapsible::new(open_model).into_element_with_open_model(
            cx,
            |cx, open, is_open| {
                shadcn::Button::new(format!("{} {}", if is_open { "?" } else { "?" }, label))
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .toggle_model(open)
                    .test_id(format!("ui-gallery-collapsible-tree-trigger-{key}"))
                    .into_element(cx)
            },
            |cx| {
                shadcn::CollapsibleContent::new(vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N1)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().ml(Space::N4)),
                    |_cx| children,
                )])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
            },
        )
    };

    let file_tree_content = {
        let ui_button = file_leaf(cx, "button.rs");
        let ui_dialog = file_leaf(cx, "dialog.rs");
        let ui_folder = folder(
            cx,
            "src-ui",
            "ui",
            tree_src_ui_open.clone(),
            vec![ui_button, ui_dialog],
        );

        let src_main = file_leaf(cx, "main.rs");
        let src_folder = folder(
            cx,
            "src",
            "src",
            tree_src_open.clone(),
            vec![ui_folder, src_main],
        );

        let comp_card = file_leaf(cx, "card.rs");
        let comp_table = file_leaf(cx, "table.rs");
        let components_folder = folder(
            cx,
            "components",
            "components",
            tree_components_open.clone(),
            vec![comp_card, comp_table],
        );

        let cargo_toml = file_leaf(cx, "Cargo.toml");
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N1)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(360.0))),
            |_cx| vec![components_folder, src_folder, cargo_toml],
        )
        .test_id("ui-gallery-collapsible-file-tree")
    };
    let file_tree = section_card(cx, "File Tree", file_tree_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            details_collapsible(
                cx,
                "ui-gallery-collapsible-rtl",
                Some(rtl_open.clone()),
                "Order #4189",
                "Shipped",
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Collapsible docs flow: Demo -> Controlled State -> Basic -> Settings Panel -> File Tree -> RTL.",
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
                controlled_state,
                basic,
                settings,
                file_tree,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-collapsible-component");

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
                    "Basic",
                    r#"let demo = shadcn::Collapsible::uncontrolled(false)
    .into_element_with_open_model(cx, |cx, open, is_open| {
        shadcn::Button::new(if is_open { "Hide details" } else { "Show details" })
            .toggle_model(open)
            .into_element(cx)
    }, |cx| shadcn::CollapsibleContent::new(vec![cx.text("Content")]).into_element(cx));"#,
                ),
                code_block(
                    cx,
                    "Controlled State",
                    r#"let open: Model<bool> = cx.app.models_mut().insert(false);
let controlled = shadcn::Collapsible::new(open.clone()).into_element_with_open_model(...);"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Collapsible::new(open_model).into_element_with_open_model(...)
})"#,
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
                    "Use controlled mode (`Model<bool>`) when outside state (URL/query, form mode, or saved layout) needs to drive disclosure.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For dense editor UIs, keep trigger chrome compact and put expensive children under `CollapsibleContent`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Nested collapsibles in file trees should keep each node state independent and keyed for stable toggling.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Always verify RTL with long trigger labels to ensure direction and alignment remain predictable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-collapsible",
        component_panel,
        code_panel,
        notes_panel,
    )
}
