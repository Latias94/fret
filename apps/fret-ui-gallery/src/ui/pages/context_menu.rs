use super::super::*;

pub(super) fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ContextMenuPageModels {
        submenu_open: Option<Model<bool>>,
        shortcuts_open: Option<Model<bool>>,
        groups_open: Option<Model<bool>>,
        icons_open: Option<Model<bool>>,
        checkboxes_open: Option<Model<bool>>,
        radio_open: Option<Model<bool>>,
        destructive_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        show_status_bar: Option<Model<bool>>,
        show_activity_bar: Option<Model<bool>>,
        show_line_numbers: Option<Model<bool>>,
        theme_mode: Option<Model<Option<Arc<str>>>>,
    }

    let (
        submenu_open,
        shortcuts_open,
        groups_open,
        icons_open,
        checkboxes_open,
        radio_open,
        destructive_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        show_line_numbers,
        theme_mode,
    ) = cx.with_state(ContextMenuPageModels::default, |st| {
        (
            st.submenu_open.clone(),
            st.shortcuts_open.clone(),
            st.groups_open.clone(),
            st.icons_open.clone(),
            st.checkboxes_open.clone(),
            st.radio_open.clone(),
            st.destructive_open.clone(),
            st.rtl_open.clone(),
            st.show_status_bar.clone(),
            st.show_activity_bar.clone(),
            st.show_line_numbers.clone(),
            st.theme_mode.clone(),
        )
    });

    let (
        submenu_open,
        shortcuts_open,
        groups_open,
        icons_open,
        checkboxes_open,
        radio_open,
        destructive_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        show_line_numbers,
        theme_mode,
    ) = match (
        submenu_open,
        shortcuts_open,
        groups_open,
        icons_open,
        checkboxes_open,
        radio_open,
        destructive_open,
        rtl_open,
        show_status_bar,
        show_activity_bar,
        show_line_numbers,
        theme_mode,
    ) {
        (
            Some(submenu_open),
            Some(shortcuts_open),
            Some(groups_open),
            Some(icons_open),
            Some(checkboxes_open),
            Some(radio_open),
            Some(destructive_open),
            Some(rtl_open),
            Some(show_status_bar),
            Some(show_activity_bar),
            Some(show_line_numbers),
            Some(theme_mode),
        ) => (
            submenu_open,
            shortcuts_open,
            groups_open,
            icons_open,
            checkboxes_open,
            radio_open,
            destructive_open,
            rtl_open,
            show_status_bar,
            show_activity_bar,
            show_line_numbers,
            theme_mode,
        ),
        _ => {
            let submenu_open = cx.app.models_mut().insert(false);
            let shortcuts_open = cx.app.models_mut().insert(false);
            let groups_open = cx.app.models_mut().insert(false);
            let icons_open = cx.app.models_mut().insert(false);
            let checkboxes_open = cx.app.models_mut().insert(false);
            let radio_open = cx.app.models_mut().insert(false);
            let destructive_open = cx.app.models_mut().insert(false);
            let rtl_open = cx.app.models_mut().insert(false);
            let show_status_bar = cx.app.models_mut().insert(true);
            let show_activity_bar = cx.app.models_mut().insert(true);
            let show_line_numbers = cx.app.models_mut().insert(false);
            let theme_mode = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));

            cx.with_state(ContextMenuPageModels::default, |st| {
                st.submenu_open = Some(submenu_open.clone());
                st.shortcuts_open = Some(shortcuts_open.clone());
                st.groups_open = Some(groups_open.clone());
                st.icons_open = Some(icons_open.clone());
                st.checkboxes_open = Some(checkboxes_open.clone());
                st.radio_open = Some(radio_open.clone());
                st.destructive_open = Some(destructive_open.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.show_status_bar = Some(show_status_bar.clone());
                st.show_activity_bar = Some(show_activity_bar.clone());
                st.show_line_numbers = Some(show_line_numbers.clone());
                st.theme_mode = Some(theme_mode.clone());
            });

            (
                submenu_open,
                shortcuts_open,
                groups_open,
                icons_open,
                checkboxes_open,
                radio_open,
                destructive_open,
                rtl_open,
                show_status_bar,
                show_activity_bar,
                show_line_numbers,
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

    let trigger_surface =
        |cx: &mut ElementContext<'_, App>, label: &'static str, test_id: &'static str| {
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .test_id(test_id)
                .into_element(cx)
        };

    let basic = {
        let menu = shadcn::ContextMenu::new(open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for actions",
                    "ui-gallery-context-menu-basic-trigger",
                )
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Profile")
                            .on_select(CMD_MENU_CONTEXT_ACTION)
                            .test_id("ui-gallery-context-menu-basic-profile"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Billing").on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Team").on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Subscription")
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                ]
            },
        );
        section_card(
            cx,
            "Basic",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-basic"),
            ),
        )
    };

    let submenu = {
        let menu = shadcn::ContextMenu::new(submenu_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for submenu",
                    "ui-gallery-context-menu-submenu-trigger",
                )
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Open")
                            .on_select(CMD_MENU_CONTEXT_ACTION)
                            .test_id("ui-gallery-context-menu-submenu-open"),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("More tools").submenu(vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Rename")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Duplicate")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Separator,
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Archive")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                        ]),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Share").on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                ]
            },
        );
        section_card(
            cx,
            "Submenu",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-submenu"),
            ),
        )
    };

    let shortcuts = {
        let menu = shadcn::ContextMenu::new(shortcuts_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for shortcuts",
                    "ui-gallery-context-menu-shortcuts-trigger",
                )
            },
            |cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Open File")
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+O").into_element(cx))
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Search in Files")
                            .trailing(
                                shadcn::ContextMenuShortcut::new("Cmd+Shift+F").into_element(cx),
                            )
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Toggle Sidebar")
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+B").into_element(cx))
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                ]
            },
        );
        section_card(
            cx,
            "Shortcuts",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-shortcuts"),
            ),
        )
    };

    let groups = {
        let menu = shadcn::ContextMenu::new(groups_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for grouped actions",
                    "ui-gallery-context-menu-groups-trigger",
                )
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new("File")),
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new([
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("New file")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Open recent")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ])),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new("Edit")),
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new([
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Cut").on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Copy").on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Paste")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ])),
                ]
            },
        );
        section_card(
            cx,
            "Groups",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-groups"),
            ),
        )
    };

    let icons = {
        let menu = shadcn::ContextMenu::new(icons_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for icon menu",
                    "ui-gallery-context-menu-icons-trigger",
                )
            },
            |cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Profile")
                            .leading(shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.user"),
                            ))
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Settings")
                            .leading(shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.settings"),
                            ))
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Download")
                            .leading(shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.download"),
                            ))
                            .on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                ]
            },
        );
        section_card(
            cx,
            "Icons",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-icons"),
            ),
        )
    };

    let checkboxes = {
        let menu = shadcn::ContextMenu::new(checkboxes_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for checkbox items",
                    "ui-gallery-context-menu-checkboxes-trigger",
                )
            },
            |cx| {
                vec![
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(
                            show_status_bar.clone(),
                            "Show status bar",
                        )
                        .trailing(shadcn::ContextMenuShortcut::new("Cmd+/").into_element(cx)),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(shadcn::ContextMenuCheckboxItem::new(
                        show_activity_bar.clone(),
                        "Show activity bar",
                    )),
                    shadcn::ContextMenuEntry::CheckboxItem(shadcn::ContextMenuCheckboxItem::new(
                        show_line_numbers.clone(),
                        "Show line numbers",
                    )),
                ]
            },
        );

        let status = cx.watch_model(&show_status_bar).copied().unwrap_or(true);
        let activity = cx.watch_model(&show_activity_bar).copied().unwrap_or(true);
        let line_numbers = cx.watch_model(&show_line_numbers).copied().unwrap_or(false);
        let state = cx.text(format!(
            "state: status_bar={status}, activity_bar={activity}, line_numbers={line_numbers}"
        ));

        section_card(
            cx,
            "Checkboxes",
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2).items_start(),
                |_cx| {
                    vec![
                        menu.attach_semantics(
                            SemanticsDecoration::default()
                                .test_id("ui-gallery-context-menu-checkboxes"),
                        ),
                        state,
                    ]
                },
            ),
        )
    };

    let radio = {
        let menu = shadcn::ContextMenu::new(radio_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for radio items",
                    "ui-gallery-context-menu-radio-trigger",
                )
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new("Theme mode")),
                    shadcn::ContextMenuEntry::RadioGroup(
                        shadcn::ContextMenuRadioGroup::new(theme_mode.clone())
                            .item(shadcn::ContextMenuRadioItemSpec::new("light", "Light"))
                            .item(shadcn::ContextMenuRadioItemSpec::new("dark", "Dark"))
                            .item(shadcn::ContextMenuRadioItemSpec::new("system", "System")),
                    ),
                ]
            },
        );

        let selected = cx
            .watch_model(&theme_mode)
            .cloned()
            .unwrap_or_else(|| Some(Arc::<str>::from("system")))
            .unwrap_or_else(|| Arc::<str>::from("<none>"));

        section_card(
            cx,
            "Radio",
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2).items_start(),
                |_cx| {
                    vec![
                        menu.attach_semantics(
                            SemanticsDecoration::default().test_id("ui-gallery-context-menu-radio"),
                        ),
                        cx.text(format!("selected theme: {selected}")),
                    ]
                },
            ),
        )
    };

    let destructive = {
        let menu = shadcn::ContextMenu::new(destructive_open.clone()).into_element(
            cx,
            |cx| {
                trigger_surface(
                    cx,
                    "Right click for destructive items",
                    "ui-gallery-context-menu-destructive-trigger",
                )
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Rename").on_select(CMD_MENU_CONTEXT_ACTION),
                    ),
                    shadcn::ContextMenuEntry::Separator,
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Delete project")
                            .variant(shadcn::ContextMenuItemVariant::Destructive)
                            .on_select(CMD_MENU_CONTEXT_ACTION)
                            .test_id("ui-gallery-context-menu-destructive-delete"),
                    ),
                ]
            },
        );
        section_card(
            cx,
            "Destructive",
            menu.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-destructive"),
            ),
        )
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::ContextMenu::new(rtl_open.clone()).into_element(
                    cx,
                    |cx| {
                        trigger_surface(
                            cx,
                            "Right click in RTL",
                            "ui-gallery-context-menu-rtl-trigger",
                        )
                    },
                    |_cx| {
                        vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Open")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Settings")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Separator,
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Delete")
                                    .variant(shadcn::ContextMenuItemVariant::Destructive)
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                        ]
                    },
                )
            },
        );

        section_card(
            cx,
            "RTL",
            rtl_content.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-context-menu-rtl"),
            ),
        )
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Context Menu docs order: Basic, Submenu, Shortcuts, Groups, Icons, Checkboxes, Radio, Destructive, RTL.",
    );

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        move |cx| {
            vec![
                preview_hint,
                cx.text(format!("last action: {last}")),
                basic,
                submenu,
                shortcuts,
                groups,
                icons,
                checkboxes,
                radio,
                destructive,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).attach_semantics(
        SemanticsDecoration::default().test_id("ui-gallery-context-menu-component"),
    );

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
                    "Basic + Submenu",
                    r#"ContextMenu::new(open).into_element(
    cx,
    |cx| trigger,
    |_cx| vec![
        ContextMenuEntry::Item(ContextMenuItem::new("Open")),
        ContextMenuEntry::Item(ContextMenuItem::new("More").submenu(vec![...]))
    ],
);"#,
                ),
                code_block(
                    cx,
                    "Shortcuts + Icons",
                    r#"ContextMenuItem::new("Open File")
    .trailing(ContextMenuShortcut::new("Cmd+O").into_element(cx));

ContextMenuItem::new("Settings")
    .leading(icon::icon(cx, IconId::new_static("lucide.settings")));"#,
                ),
                code_block(
                    cx,
                    "Checkboxes + Radio + RTL",
                    r#"ContextMenuEntry::CheckboxItem(
    ContextMenuCheckboxItem::new(show_status_bar, "Show status bar")
);

ContextMenuEntry::RadioGroup(
    ContextMenuRadioGroup::new(theme_mode)
        .item(ContextMenuRadioItemSpec::new("light", "Light"))
        .item(ContextMenuRadioItemSpec::new("dark", "Dark"))
);

with_direction_provider(LayoutDirection::Rtl, |cx| {
    ContextMenu::new(open).into_element(cx, |cx| trigger, |_cx| entries)
});"#,
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
                    "Keep context menu entries task-focused; destructive entries should be visually separated by a divider.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Prefer checkboxes/radio groups for persistent menu state so users can infer current mode before selecting.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use icon + shortcut combinations sparingly: icons improve scanning, shortcuts improve expert throughput.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep explicit RTL coverage in gallery so submenu direction and destructive styling stay parity-auditable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-context-menu",
        component_panel,
        code_panel,
        notes_panel,
    )
}
