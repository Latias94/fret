use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

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

    let trigger_surface =
        |cx: &mut ElementContext<'_, App>, label: &'static str, test_id: &'static str| {
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .test_id(test_id)
                .into_element(cx)
        };

    let basic = {
        let menu = shadcn::ContextMenu::new(open.clone())
            .content_test_id("ui-gallery-context-menu-basic-content")
            .into_element(
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
                            shadcn::ContextMenuItem::new("Billing")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
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

        let last = cx
            .app
            .models()
            .get_cloned(&last_action)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    menu,
                    shadcn::typography::muted(cx, format!("last action: {last}"))
                        .test_id("ui-gallery-context-menu-last-action"),
                ]
            },
        )
    };

    let submenu = {
        let menu = shadcn::ContextMenu::new(submenu_open.clone())
            .content_test_id("ui-gallery-context-menu-submenu-content")
            .into_element(
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
                            shadcn::ContextMenuItem::new("More tools")
                                .test_id("ui-gallery-context-menu-submenu-more-tools")
                                .submenu(vec![
                                    shadcn::ContextMenuEntry::Item(
                                        shadcn::ContextMenuItem::new("Rename")
                                            .on_select(CMD_MENU_CONTEXT_ACTION)
                                            .test_id("ui-gallery-context-menu-submenu-rename"),
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
                            shadcn::ContextMenuItem::new("Share")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ]
                },
            );
        menu
    };

    let shortcuts = {
        let menu = shadcn::ContextMenu::new(shortcuts_open.clone())
            .content_test_id("ui-gallery-context-menu-shortcuts-content")
            .into_element(
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
                                .trailing(
                                    shadcn::ContextMenuShortcut::new("Cmd+O").into_element(cx),
                                )
                                .test_id("ui-gallery-context-menu-shortcuts-open-file")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Search in Files")
                                .trailing(
                                    shadcn::ContextMenuShortcut::new("Cmd+Shift+F")
                                        .into_element(cx),
                                )
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Toggle Sidebar")
                                .trailing(
                                    shadcn::ContextMenuShortcut::new("Cmd+B").into_element(cx),
                                )
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ]
                },
            );
        menu
    };

    let groups = {
        let menu = shadcn::ContextMenu::new(groups_open.clone())
            .content_test_id("ui-gallery-context-menu-groups-content")
            .into_element(
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
                                shadcn::ContextMenuItem::new("Cut")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Copy")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Paste")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                        ])),
                    ]
                },
            );
        menu
    };

    let icons = {
        let menu = shadcn::ContextMenu::new(icons_open.clone())
            .content_test_id("ui-gallery-context-menu-icons-content")
            .into_element(
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
                                .leading_icon(fret_icons::IconId::new_static("lucide.user"))
                                .test_id("ui-gallery-context-menu-icons-profile")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Settings")
                                .leading_icon(fret_icons::IconId::new_static("lucide.settings"))
                                .test_id("ui-gallery-context-menu-icons-settings")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Download")
                                .leading_icon(fret_icons::IconId::new_static("lucide.download"))
                                .test_id("ui-gallery-context-menu-icons-download")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ]
                },
            );
        menu
    };

    let checkboxes = {
        let menu = shadcn::ContextMenu::new(checkboxes_open.clone())
            .content_test_id("ui-gallery-context-menu-checkboxes-content")
            .into_element(
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
                            .test_id("ui-gallery-context-menu-checkboxes-status-bar")
                            .trailing(shadcn::ContextMenuShortcut::new("Cmd+/").into_element(cx)),
                        ),
                        shadcn::ContextMenuEntry::CheckboxItem(
                            shadcn::ContextMenuCheckboxItem::new(
                                show_activity_bar.clone(),
                                "Show activity bar",
                            ),
                        ),
                        shadcn::ContextMenuEntry::CheckboxItem(
                            shadcn::ContextMenuCheckboxItem::new(
                                show_line_numbers.clone(),
                                "Show line numbers",
                            ),
                        ),
                    ]
                },
            );

        let status = cx.watch_model(&show_status_bar).copied().unwrap_or(true);
        let activity = cx.watch_model(&show_activity_bar).copied().unwrap_or(true);
        let line_numbers = cx.watch_model(&show_line_numbers).copied().unwrap_or(false);
        let state = cx.text(format!(
            "state: status_bar={status}, activity_bar={activity}, line_numbers={line_numbers}"
        ));

        let checkboxes_content = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |_cx| vec![menu, state],
        );
        checkboxes_content
    };

    let radio = {
        let menu = shadcn::ContextMenu::new(radio_open.clone())
            .content_test_id("ui-gallery-context-menu-radio-content")
            .into_element(
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
                        shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new(
                            "Theme mode",
                        )),
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

        let selected_text = cx.text(format!("selected theme: {selected}"));
        let radio_content = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |_cx| vec![menu, selected_text],
        );
        radio_content
    };

    let destructive = {
        let menu = shadcn::ContextMenu::new(destructive_open.clone())
            .content_test_id("ui-gallery-context-menu-destructive-content")
            .into_element(
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
                            shadcn::ContextMenuItem::new("Rename")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Delete project")
                                .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive)
                                .on_select(CMD_MENU_CONTEXT_ACTION)
                                .test_id("ui-gallery-context-menu-destructive-delete"),
                        ),
                    ]
                },
            );
        menu
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::ContextMenu::new(rtl_open.clone())
            .content_test_id("ui-gallery-context-menu-rtl-content")
            .into_element(
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
                            shadcn::ContextMenuItem::new("Open").on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Settings")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Delete")
                                .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive)
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                    ]
                },
            )
    });

    let notes = doc_layout::notes(
        cx,
        [
            "Keep context menu entries task-focused; destructive entries should be visually separated by a divider.",
            "Prefer checkboxes/radio groups for persistent menu state so users can infer current mode before selecting.",
            "Use icon + shortcut combinations sparingly: icons improve scanning, shortcuts improve expert throughput.",
            "Keep explicit RTL coverage in gallery so submenu direction and destructive styling stay parity-auditable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Context Menu docs order: Basic, Submenu, Shortcuts, Groups, Icons, Checkboxes, Radio, Destructive, RTL.",
        ),
        vec![
            DocSection::new("Basic", basic)
                .description("Right click on the trigger surface to open the menu.")
                .code(
                    "rust",
                    r#"let open = app.models_mut().insert(false);
let menu = shadcn::ContextMenu::new(open).into_element(cx, trigger, |_cx| entries);"#,
                ),
            DocSection::new("Submenu", submenu)
                .description("Nested submenu entries for grouped actions.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::Item(
    shadcn::ContextMenuItem::new("More tools").submenu(vec![
        shadcn::ContextMenuEntry::Item(
            shadcn::ContextMenuItem::new("Rename").on_select(CMD_MENU_CONTEXT_ACTION),
        ),
        shadcn::ContextMenuEntry::Item(
            shadcn::ContextMenuItem::new("Duplicate").on_select(CMD_MENU_CONTEXT_ACTION),
        ),
        shadcn::ContextMenuEntry::Separator,
        shadcn::ContextMenuEntry::Item(
            shadcn::ContextMenuItem::new("Archive").on_select(CMD_MENU_CONTEXT_ACTION),
        ),
    ]),
);"#,
                ),
            DocSection::new("Shortcuts", shortcuts)
                .description("Trailing shortcuts for command discovery.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::Item(
    shadcn::ContextMenuItem::new("Open File")
        .trailing(shadcn::ContextMenuShortcut::new("Cmd+O").into_element(cx))
        .on_select(CMD_MENU_CONTEXT_ACTION),
);"#,
                ),
            DocSection::new("Groups", groups)
                .description("Explicit groups and labels for structured menus.")
                .code(
                    "rust",
                    r#"vec![
    shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new("Actions")),
    shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Open")),
    shadcn::ContextMenuEntry::Separator,
    shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new("View")),
    shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Toggle sidebar")),
];"#,
                ),
            DocSection::new("Icons", icons)
                .description("Leading icons for visual scanning.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::Item(
    shadcn::ContextMenuItem::new("Settings")
        .leading_icon(fret_icons::IconId::new_static("lucide.settings"))
        .on_select(CMD_MENU_CONTEXT_ACTION),
);"#,
                ),
            DocSection::new("Checkboxes", checkboxes)
                .description("Checkbox items are bound to boolean models.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::CheckboxItem(
    shadcn::ContextMenuCheckboxItem::new(show_status_bar, "Show status bar"),
);"#,
                ),
            DocSection::new("Radio", radio)
                .description("Radio groups are bound to a single selected value.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::RadioGroup(
    shadcn::ContextMenuRadioGroup::new(theme_mode)
        .item(shadcn::ContextMenuRadioItemSpec::new("light", "Light"))
        .item(shadcn::ContextMenuRadioItemSpec::new("dark", "Dark")),
);"#,
                ),
            DocSection::new("Destructive", destructive)
                .description("Destructive items use a dedicated visual variant.")
                .code(
                    "rust",
                    r#"shadcn::ContextMenuEntry::Item(
    shadcn::ContextMenuItem::new("Delete project")
        .variant(shadcn::context_menu::ContextMenuItemVariant::Destructive)
        .on_select(CMD_MENU_CONTEXT_ACTION),
);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Menu layout should follow right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::ContextMenu::new(open).into_element(cx, trigger, |_cx| {
            vec![shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Open"))]
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
