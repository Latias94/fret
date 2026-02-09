use super::super::*;

pub(super) fn preview_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DropdownMenuPageModels {
        basic_open: Option<Model<bool>>,
        submenu_open: Option<Model<bool>>,
        shortcuts_open: Option<Model<bool>>,
        icons_open: Option<Model<bool>>,
        checkboxes_open: Option<Model<bool>>,
        checkboxes_icons_open: Option<Model<bool>>,
        radio_open: Option<Model<bool>>,
        radio_icons_open: Option<Model<bool>>,
        destructive_open: Option<Model<bool>>,
        avatar_open: Option<Model<bool>>,
        complex_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        show_status_bar: Option<Model<bool>>,
        show_activity_bar: Option<Model<bool>>,
        theme_mode: Option<Model<Option<Arc<str>>>>,
    }

    let (
        basic_open,
        submenu_open,
        shortcuts_open,
        icons_open,
        checkboxes_open,
        checkboxes_icons_open,
        radio_open,
        radio_icons_open,
        destructive_open,
        avatar_open,
        complex_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        theme_mode,
    ) = cx.with_state(DropdownMenuPageModels::default, |st| {
        (
            st.basic_open.clone(),
            st.submenu_open.clone(),
            st.shortcuts_open.clone(),
            st.icons_open.clone(),
            st.checkboxes_open.clone(),
            st.checkboxes_icons_open.clone(),
            st.radio_open.clone(),
            st.radio_icons_open.clone(),
            st.destructive_open.clone(),
            st.avatar_open.clone(),
            st.complex_open.clone(),
            st.rtl_open.clone(),
            st.show_status_bar.clone(),
            st.show_activity_bar.clone(),
            st.theme_mode.clone(),
        )
    });

    let (
        basic_open,
        submenu_open,
        shortcuts_open,
        icons_open,
        checkboxes_open,
        checkboxes_icons_open,
        radio_open,
        radio_icons_open,
        destructive_open,
        avatar_open,
        complex_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        theme_mode,
    ) = match (
        basic_open,
        submenu_open,
        shortcuts_open,
        icons_open,
        checkboxes_open,
        checkboxes_icons_open,
        radio_open,
        radio_icons_open,
        destructive_open,
        avatar_open,
        complex_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        theme_mode,
    ) {
        (
            Some(basic_open),
            Some(submenu_open),
            Some(shortcuts_open),
            Some(icons_open),
            Some(checkboxes_open),
            Some(checkboxes_icons_open),
            Some(radio_open),
            Some(radio_icons_open),
            Some(destructive_open),
            Some(avatar_open),
            Some(complex_open),
            Some(rtl_open),
            Some(show_status_bar),
            Some(show_activity_bar),
            Some(theme_mode),
        ) => (
            basic_open,
            submenu_open,
            shortcuts_open,
            icons_open,
            checkboxes_open,
            checkboxes_icons_open,
            radio_open,
            radio_icons_open,
            destructive_open,
            avatar_open,
            complex_open,
            rtl_open,
            show_status_bar,
            show_activity_bar,
            theme_mode,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let submenu_open = cx.app.models_mut().insert(false);
            let shortcuts_open = cx.app.models_mut().insert(false);
            let icons_open = cx.app.models_mut().insert(false);
            let checkboxes_open = cx.app.models_mut().insert(false);
            let checkboxes_icons_open = cx.app.models_mut().insert(false);
            let radio_open = cx.app.models_mut().insert(false);
            let radio_icons_open = cx.app.models_mut().insert(false);
            let destructive_open = cx.app.models_mut().insert(false);
            let avatar_open = cx.app.models_mut().insert(false);
            let complex_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);
            let show_status_bar = cx.app.models_mut().insert(true);
            let show_activity_bar = cx.app.models_mut().insert(false);
            let theme_mode = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));

            cx.with_state(DropdownMenuPageModels::default, |st| {
                st.basic_open = Some(basic_open.clone());
                st.submenu_open = Some(submenu_open.clone());
                st.shortcuts_open = Some(shortcuts_open.clone());
                st.icons_open = Some(icons_open.clone());
                st.checkboxes_open = Some(checkboxes_open.clone());
                st.checkboxes_icons_open = Some(checkboxes_icons_open.clone());
                st.radio_open = Some(radio_open.clone());
                st.radio_icons_open = Some(radio_icons_open.clone());
                st.destructive_open = Some(destructive_open.clone());
                st.avatar_open = Some(avatar_open.clone());
                st.complex_open = Some(complex_open.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.show_status_bar = Some(show_status_bar.clone());
                st.show_activity_bar = Some(show_activity_bar.clone());
                st.theme_mode = Some(theme_mode.clone());
            });

            (
                basic_open,
                submenu_open,
                shortcuts_open,
                icons_open,
                checkboxes_open,
                checkboxes_icons_open,
                radio_open,
                radio_icons_open,
                destructive_open,
                avatar_open,
                complex_open,
                rtl_open,
                show_status_bar,
                show_activity_bar,
                theme_mode,
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

    let button_trigger = |cx: &mut ElementContext<'_, App>,
                          open_model: Model<bool>,
                          label: &'static str,
                          test_id: &'static str| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .toggle_model(open_model.clone())
            .test_id(test_id)
            .into_element(cx)
    };

    let icon = |cx: &mut ElementContext<'_, App>, id: &'static str| {
        shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
    };

    let demo_content = shadcn::DropdownMenu::new(open.clone())
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open menu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(open.clone())
                    .test_id("ui-gallery-dropdown-menu-demo-trigger")
                    .into_element(cx)
            },
            |cx| {
                vec![
                    shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("My Account")),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Profile")
                            .leading(icon(cx, "lucide.user"))
                            .trailing(shadcn::DropdownMenuShortcut::new("Cmd+P").into_element(cx))
                            .on_select(CMD_MENU_DROPDOWN_APPLE)
                            .test_id("ui-gallery-dropdown-menu-demo-profile"),
                    ),
                    shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("More").submenu(
                        vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Invite users")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Preferences")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                        ],
                    )),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-demo");
    let demo = section_card(cx, "Demo", demo_content);

    let basic_content = shadcn::DropdownMenu::new(basic_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                basic_open.clone(),
                "Basic",
                "ui-gallery-dropdown-menu-basic-trigger",
            )
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("My Account")),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile")
                        .test_id("ui-gallery-dropdown-menu-basic-profile")
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Billing").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
            ]
        },
    );
    let basic = section_card(cx, "Basic", basic_content);

    let submenu_content = shadcn::DropdownMenu::new(submenu_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                submenu_open.clone(),
                "Submenu",
                "ui-gallery-dropdown-menu-submenu-trigger",
            )
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Open").on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("More tools").submenu(vec![
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Rename")
                                .on_select(CMD_MENU_DROPDOWN_APPLE),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Duplicate")
                                .on_select(CMD_MENU_DROPDOWN_ORANGE),
                        ),
                    ]),
                ),
            ]
        },
    );
    let submenu = section_card(cx, "Submenu", submenu_content);

    let shortcuts_content = shadcn::DropdownMenu::new(shortcuts_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                shortcuts_open.clone(),
                "Shortcuts",
                "ui-gallery-dropdown-menu-shortcuts-trigger",
            )
        },
        |cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Open file")
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+O").into_element(cx))
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Search")
                        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+F").into_element(cx))
                        .on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
            ]
        },
    );
    let shortcuts = section_card(cx, "Shortcuts", shortcuts_content);

    let icons_content = shadcn::DropdownMenu::new(icons_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                icons_open.clone(),
                "Icons",
                "ui-gallery-dropdown-menu-icons-trigger",
            )
        },
        |cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading(icon(cx, "lucide.user"))
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Settings")
                        .leading(icon(cx, "lucide.settings"))
                        .on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
            ]
        },
    );
    let icons = section_card(cx, "Icons", icons_content);

    let checkboxes_content = shadcn::DropdownMenu::new(checkboxes_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                checkboxes_open.clone(),
                "Checkboxes",
                "ui-gallery-dropdown-menu-checkboxes-trigger",
            )
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::CheckboxItem(shadcn::DropdownMenuCheckboxItem::new(
                    show_status_bar.clone(),
                    "Status Bar",
                )),
                shadcn::DropdownMenuEntry::CheckboxItem(shadcn::DropdownMenuCheckboxItem::new(
                    show_activity_bar.clone(),
                    "Activity Bar",
                )),
            ]
        },
    );
    let checkboxes = section_card(cx, "Checkboxes", checkboxes_content);

    let checkboxes_icons_content = shadcn::DropdownMenu::new(checkboxes_icons_open.clone())
        .into_element(
            cx,
            |cx| {
                button_trigger(
                    cx,
                    checkboxes_icons_open.clone(),
                    "Checkboxes Icons",
                    "ui-gallery-dropdown-menu-checkboxes-icons-trigger",
                )
            },
            |cx| {
                vec![
                    shadcn::DropdownMenuEntry::CheckboxItem(
                        shadcn::DropdownMenuCheckboxItem::new(
                            show_status_bar.clone(),
                            "Status Bar",
                        )
                        .leading(icon(cx, "lucide.panel-bottom")),
                    ),
                    shadcn::DropdownMenuEntry::CheckboxItem(
                        shadcn::DropdownMenuCheckboxItem::new(
                            show_activity_bar.clone(),
                            "Activity Bar",
                        )
                        .leading(icon(cx, "lucide.panel-left")),
                    ),
                ]
            },
        );
    let checkboxes_icons = section_card(cx, "Checkboxes Icons", checkboxes_icons_content);

    let radio_group_content = shadcn::DropdownMenu::new(radio_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                radio_open.clone(),
                "Radio Group",
                "ui-gallery-dropdown-menu-radio-trigger",
            )
        },
        |_cx| {
            vec![shadcn::DropdownMenuEntry::RadioGroup(
                shadcn::DropdownMenuRadioGroup::new(theme_mode.clone())
                    .item(shadcn::DropdownMenuRadioItemSpec::new("light", "Light"))
                    .item(shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark"))
                    .item(shadcn::DropdownMenuRadioItemSpec::new("system", "System")),
            )]
        },
    );
    let radio_group = section_card(cx, "Radio Group", radio_group_content);

    let radio_icons_content = shadcn::DropdownMenu::new(radio_icons_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                radio_icons_open.clone(),
                "Radio Icons",
                "ui-gallery-dropdown-menu-radio-icons-trigger",
            )
        },
        |cx| {
            vec![shadcn::DropdownMenuEntry::RadioGroup(
                shadcn::DropdownMenuRadioGroup::new(theme_mode.clone())
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("light", "Light")
                            .leading(icon(cx, "lucide.sun")),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                            .leading(icon(cx, "lucide.moon")),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("system", "System")
                            .leading(icon(cx, "lucide.monitor")),
                    ),
            )]
        },
    );
    let radio_icons = section_card(cx, "Radio Icons", radio_icons_content);

    let destructive_content = shadcn::DropdownMenu::new(destructive_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                destructive_open.clone(),
                "Destructive",
                "ui-gallery-dropdown-menu-destructive-trigger",
            )
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Rename").on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Delete")
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
            ]
        },
    );
    let destructive = section_card(cx, "Destructive", destructive_content);

    let avatar_content = shadcn::DropdownMenu::new(avatar_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_px(Px(36.0)).h_px(Px(36.0)))
                .into_element(cx)
                .test_id("ui-gallery-dropdown-menu-avatar-trigger")
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("john@fret.dev")),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile").on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Log out").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
            ]
        },
    );
    let avatar = section_card(cx, "Avatar", avatar_content);

    let complex_content = shadcn::DropdownMenu::new(complex_open.clone())
        .arrow(true)
        .into_element(
            cx,
            |cx| {
                button_trigger(
                    cx,
                    complex_open.clone(),
                    "Complex",
                    "ui-gallery-dropdown-menu-complex-trigger",
                )
            },
            |cx| {
                vec![
                    shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("Actions")),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Open")
                            .leading(icon(cx, "lucide.folder-open"))
                            .on_select(CMD_MENU_DROPDOWN_APPLE),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Share").submenu(vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Invite")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Copy link")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                        ]),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Delete")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .on_select(CMD_MENU_DROPDOWN_ORANGE),
                    ),
                ]
            },
        );
    let complex = section_card(cx, "Complex", complex_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::DropdownMenu::new(rtl_open.clone()).into_element(
                cx,
                |cx| {
                    button_trigger(
                        cx,
                        rtl_open.clone(),
                        "RTL",
                        "ui-gallery-dropdown-menu-rtl-trigger",
                    )
                },
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Dashboard")
                                .on_select(CMD_MENU_DROPDOWN_APPLE),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Settings")
                                .on_select(CMD_MENU_DROPDOWN_ORANGE),
                        ),
                    ]
                },
            )
        },
    );
    let rtl = section_card(cx, "RTL", rtl_content);

    let action_text = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

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
                    "Preview follows shadcn Dropdown Menu docs order: Demo, Basic, Submenu, Shortcuts, Icons, Checkboxes, Checkboxes Icons, Radio Group, Radio Icons, Destructive, Avatar, Complex, RTL.",
                ),
                shadcn::typography::muted(cx, format!("last action: {action_text}")),
                demo,
                basic,
                submenu,
                shortcuts,
                icons,
                checkboxes,
                checkboxes_icons,
                radio_group,
                radio_icons,
                destructive,
                avatar,
                complex,
                rtl,
            ]
        },
    );
    let component_panel =
        shell(cx, component_panel_body).test_id("ui-gallery-dropdown-menu-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Basic").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"shadcn::DropdownMenu::new(open).into_element(
    cx,
    |cx| trigger,
    |_cx| vec![
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Profile")),
        shadcn::DropdownMenuEntry::Separator,
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Billing")),
    ],
);"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Checkbox + Radio").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"shadcn::DropdownMenuEntry::CheckboxItem(
    shadcn::DropdownMenuCheckboxItem::new(show_status_bar, "Status Bar"),
);

shadcn::DropdownMenuEntry::RadioGroup(
    shadcn::DropdownMenuRadioGroup::new(theme_mode)
        .item(shadcn::DropdownMenuRadioItemSpec::new("light", "Light")),
);"#,
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
                    "Dropdown Menu page follows docs sequence to keep parity review deterministic.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Checkbox and radio examples are stateful so selection persists across open-close cycles.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Avatar example uses fallback avatar trigger for deterministic rendering in gallery runs.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-dropdown-menu",
        component_panel,
        code_panel,
        notes_panel,
    )
}
