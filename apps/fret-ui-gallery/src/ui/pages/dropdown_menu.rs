use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

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

    let button_trigger =
        |cx: &mut ElementContext<'_, App>, label: &'static str, test_id: &'static str| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .test_id(test_id)
            .into_element(cx)
    };

    let action_text = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let demo_menu = shadcn::DropdownMenu::new(open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Open menu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dropdown-menu-demo-trigger")
                .into_element(cx)
        },
        |cx| {
            vec![
                shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("My Account")),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading_icon(fret_icons::IconId::new_static("lucide.user"))
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
    );

    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                demo_menu,
                shadcn::typography::muted(cx, format!("last action: {action_text}"))
                    .test_id("ui-gallery-dropdown-menu-last-action"),
            ]
        },
    );

    let basic = shadcn::DropdownMenu::new(basic_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let submenu = shadcn::DropdownMenu::new(submenu_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let shortcuts = shadcn::DropdownMenu::new(shortcuts_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let icons = shadcn::DropdownMenu::new(icons_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                "Icons",
                "ui-gallery-dropdown-menu-icons-trigger",
            )
        },
        |cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Profile")
                        .leading_icon(fret_icons::IconId::new_static("lucide.user"))
                        .on_select(CMD_MENU_DROPDOWN_APPLE)
                        .test_id("ui-gallery-dropdown-menu-icons-profile"),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Settings")
                        .leading_icon(fret_icons::IconId::new_static("lucide.settings"))
                        .on_select(CMD_MENU_DROPDOWN_ORANGE)
                        .test_id("ui-gallery-dropdown-menu-icons-settings"),
                ),
            ]
        },
    );

    let checkboxes = shadcn::DropdownMenu::new(checkboxes_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let checkboxes_icons = shadcn::DropdownMenu::new(checkboxes_icons_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                "Checkboxes Icons",
                "ui-gallery-dropdown-menu-checkboxes-icons-trigger",
            )
        },
        |cx| {
            vec![
                shadcn::DropdownMenuEntry::CheckboxItem(
                    shadcn::DropdownMenuCheckboxItem::new(show_status_bar.clone(), "Status Bar")
                        .leading_icon(fret_icons::IconId::new_static("lucide.panel-bottom"))
                        .test_id("ui-gallery-dropdown-menu-checkboxes-icons-status-bar"),
                ),
                shadcn::DropdownMenuEntry::CheckboxItem(
                    shadcn::DropdownMenuCheckboxItem::new(
                        show_activity_bar.clone(),
                        "Activity Bar",
                    )
                    .leading_icon(fret_icons::IconId::new_static("lucide.panel-left"))
                    .disabled(true)
                    .test_id("ui-gallery-dropdown-menu-checkboxes-icons-activity-bar"),
                ),
            ]
        },
    );

    let radio_group = shadcn::DropdownMenu::new(radio_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let radio_icons = shadcn::DropdownMenu::new(radio_icons_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
                "Radio Icons",
                "ui-gallery-dropdown-menu-radio-icons-trigger",
            )
        },
        |cx| {
            vec![shadcn::DropdownMenuEntry::RadioGroup(
                shadcn::DropdownMenuRadioGroup::new(theme_mode.clone())
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("light", "Light")
                            .leading_icon(fret_icons::IconId::new_static("lucide.sun"))
                            .test_id("ui-gallery-dropdown-menu-radio-icons-light"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                            .leading_icon(fret_icons::IconId::new_static("lucide.moon"))
                            .test_id("ui-gallery-dropdown-menu-radio-icons-dark"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("system", "System")
                            .leading_icon(fret_icons::IconId::new_static("lucide.monitor"))
                            .disabled(true)
                            .test_id("ui-gallery-dropdown-menu-radio-icons-system"),
                    ),
            )]
        },
    );

    let destructive = shadcn::DropdownMenu::new(destructive_open.clone()).into_element(
        cx,
        |cx| {
            button_trigger(
                cx,
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

    let avatar = shadcn::DropdownMenu::new(avatar_open.clone()).into_element(
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

    let complex = shadcn::DropdownMenu::new(complex_open.clone())
        .arrow(true)
        .into_element(
            cx,
            |cx| {
                button_trigger(
                    cx,
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
                            .leading_icon(fret_icons::IconId::new_static("lucide.folder-open"))
                            .on_select(CMD_MENU_DROPDOWN_APPLE),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Share").submenu(vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Invite")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Native share sheet")
                                    .on_select(crate::spec::CMD_SHELL_SHARE_SHEET_SMOKE),
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

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::DropdownMenu::new(rtl_open.clone()).into_element(
            cx,
            |cx| {
                button_trigger(
                    cx,
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
    });

    let notes = doc_layout::notes(
        cx,
        [
            "Dropdown Menu page follows docs sequence to keep parity review deterministic.",
            "Checkbox and radio examples are stateful so selection persists across open-close cycles.",
            "Avatar example uses fallback avatar trigger for deterministic rendering in gallery runs.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Dropdown Menu docs order: Demo, Basic, Submenu, Shortcuts, Icons, Checkboxes, Checkboxes Icons, Radio Group, Radio Icons, Destructive, Avatar, Complex, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Minimal menu surface with label, separator, and submenu.")
                .code(
                    "rust",
                    r#"let open = app.models_mut().insert(false);
let menu = shadcn::DropdownMenu::new(open).into_element(
    cx,
    |cx| shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx),
    |_cx| vec![/* entries */],
);"#,
                ),
            DocSection::new("Basic", basic)
                .description("A simple menu with label, separator, and items.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenu::new(open).into_element(cx, trigger, |_cx| {
    vec![
        shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("My Account")),
        shadcn::DropdownMenuEntry::Separator,
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Profile")),
    ]
});"#,
                ),
            DocSection::new("Submenu", submenu)
                .description("Nested submenu entries for grouped actions.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::Item(
    shadcn::DropdownMenuItem::new("More tools").submenu(vec![
        shadcn::DropdownMenuEntry::Item(
            shadcn::DropdownMenuItem::new("Rename").on_select(CMD_MENU_DROPDOWN_APPLE),
        ),
        shadcn::DropdownMenuEntry::Item(
            shadcn::DropdownMenuItem::new("Duplicate").on_select(CMD_MENU_DROPDOWN_ORANGE),
        ),
    ]),
);"#,
                ),
            DocSection::new("Shortcuts", shortcuts)
                .description("Trailing shortcuts for command discovery.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::Item(
    shadcn::DropdownMenuItem::new("Open file")
        .trailing(shadcn::DropdownMenuShortcut::new("Cmd+O").into_element(cx))
        .on_select(CMD_MENU_DROPDOWN_APPLE),
);"#,
                ),
            DocSection::new("Icons", icons)
                .description("Leading icons for visual scanning.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::Item(
    shadcn::DropdownMenuItem::new("Settings")
        .leading_icon(fret_icons::IconId::new_static("lucide.settings"))
        .on_select(CMD_MENU_DROPDOWN_ORANGE),
);"#,
                ),
            DocSection::new("Checkboxes", checkboxes)
                .description("Checkbox items are bound to boolean models.")
                .code(
                    "rust",
                    r#"let show_status_bar = cx.app.models_mut().insert(true);

shadcn::DropdownMenuEntry::CheckboxItem(shadcn::DropdownMenuCheckboxItem::new(
    show_status_bar,
    "Status Bar",
));"#,
                ),
            DocSection::new("Checkboxes Icons", checkboxes_icons)
                .description("Checkbox items can also render leading icons.")
                .code(
                    "rust",
                    r#"let show_activity_bar = cx.app.models_mut().insert(false);

shadcn::DropdownMenuEntry::CheckboxItem(
    shadcn::DropdownMenuCheckboxItem::new(show_activity_bar, "Activity Bar")
        .leading_icon(fret_icons::IconId::new_static("lucide.panel-left")),
);"#,
                ),
            DocSection::new("Radio Group", radio_group)
                .description("Radio groups are bound to a single selected value.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::RadioGroup(
    shadcn::DropdownMenuRadioGroup::new(theme_mode)
        .item(shadcn::DropdownMenuRadioItemSpec::new("light", "Light"))
        .item(shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")),
);"#,
                ),
            DocSection::new("Radio Icons", radio_icons)
                .description("Radio items can render leading icons.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::RadioGroup(
    shadcn::DropdownMenuRadioGroup::new(theme_mode)
        .item(
            shadcn::DropdownMenuRadioItemSpec::new("light", "Light")
                .leading_icon(fret_icons::IconId::new_static("lucide.sun")),
        )
        .item(
            shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                .leading_icon(fret_icons::IconId::new_static("lucide.moon")),
        ),
);"#,
                ),
            DocSection::new("Destructive", destructive)
                .description("Destructive items use a dedicated visual variant.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenuEntry::Item(
    shadcn::DropdownMenuItem::new("Delete")
        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
        .on_select(CMD_MENU_DROPDOWN_ORANGE),
);"#,
                ),
            DocSection::new("Avatar", avatar)
                .description("Menu triggers can be non-button elements (e.g. avatar).")
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);

shadcn::DropdownMenu::new(open.clone()).into_element(
    cx,
    |cx| shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)]).into_element(cx),
    |_cx| {
        vec![shadcn::DropdownMenuEntry::Item(
            shadcn::DropdownMenuItem::new("Log out").on_select(CMD_MENU_DROPDOWN_ORANGE),
        )]
    },
);"#,
                ),
            DocSection::new("Complex", complex)
                .description("Composed menu with arrows, submenus, and destructive actions.")
                .code(
                    "rust",
                    r#"shadcn::DropdownMenu::new(open)
    .arrow(true)
    .into_element(cx, trigger, |cx| {
        vec![
            shadcn::DropdownMenuEntry::Label(shadcn::DropdownMenuLabel::new("Actions")),
            shadcn::DropdownMenuEntry::Separator,
            shadcn::DropdownMenuEntry::Item(
                shadcn::DropdownMenuItem::new("Share").submenu(vec![
                ]),
            ),
            shadcn::DropdownMenuEntry::Item(
                shadcn::DropdownMenuItem::new("Delete")
                    .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
            ),
        ]
    });"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Menu layout should follow right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::DropdownMenu::new(open).into_element(cx, trigger, |_cx| {
            vec![shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Dashboard"))]
        })
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
