pub const SOURCE: &str = include_str!("app_sidebar.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct TeamSpec {
    name: &'static str,
    plan: &'static str,
    icon: &'static str,
}

#[derive(Clone, Copy)]
struct NavLeafSpec {
    key: &'static str,
    title: &'static str,
}

#[derive(Clone, Copy)]
struct NavSectionSpec {
    key: &'static str,
    title: &'static str,
    icon: &'static str,
    default_open: bool,
    items: &'static [NavLeafSpec],
}

#[derive(Clone, Copy)]
struct ProjectSpec {
    key: &'static str,
    name: &'static str,
    icon: &'static str,
}

const TEAMS: &[TeamSpec] = &[
    TeamSpec {
        name: "Acme Inc.",
        plan: "Enterprise",
        icon: "lucide.gallery-vertical-end",
    },
    TeamSpec {
        name: "Acme Corp.",
        plan: "Startup",
        icon: "lucide.audio-lines",
    },
    TeamSpec {
        name: "Evil Corp.",
        plan: "Free",
        icon: "lucide.terminal",
    },
];

const PLAYGROUND_ITEMS: &[NavLeafSpec] = &[
    NavLeafSpec {
        key: "history",
        title: "History",
    },
    NavLeafSpec {
        key: "starred",
        title: "Starred",
    },
    NavLeafSpec {
        key: "playground-settings",
        title: "Settings",
    },
];

const MODELS_ITEMS: &[NavLeafSpec] = &[
    NavLeafSpec {
        key: "genesis",
        title: "Genesis",
    },
    NavLeafSpec {
        key: "explorer",
        title: "Explorer",
    },
    NavLeafSpec {
        key: "quantum",
        title: "Quantum",
    },
];

const DOCS_ITEMS: &[NavLeafSpec] = &[
    NavLeafSpec {
        key: "introduction",
        title: "Introduction",
    },
    NavLeafSpec {
        key: "get-started",
        title: "Get Started",
    },
    NavLeafSpec {
        key: "tutorials",
        title: "Tutorials",
    },
    NavLeafSpec {
        key: "changelog",
        title: "Changelog",
    },
];

const SETTINGS_ITEMS: &[NavLeafSpec] = &[
    NavLeafSpec {
        key: "general",
        title: "General",
    },
    NavLeafSpec {
        key: "team",
        title: "Team",
    },
    NavLeafSpec {
        key: "billing",
        title: "Billing",
    },
    NavLeafSpec {
        key: "limits",
        title: "Limits",
    },
];

const NAV_SECTIONS: &[NavSectionSpec] = &[
    NavSectionSpec {
        key: "playground",
        title: "Playground",
        icon: "lucide.square-terminal",
        default_open: true,
        items: PLAYGROUND_ITEMS,
    },
    NavSectionSpec {
        key: "models",
        title: "Models",
        icon: "lucide.bot",
        default_open: false,
        items: MODELS_ITEMS,
    },
    NavSectionSpec {
        key: "documentation",
        title: "Documentation",
        icon: "lucide.book-open",
        default_open: false,
        items: DOCS_ITEMS,
    },
    NavSectionSpec {
        key: "settings",
        title: "Settings",
        icon: "lucide.settings-2",
        default_open: false,
        items: SETTINGS_ITEMS,
    },
];

const PROJECTS: &[ProjectSpec] = &[
    ProjectSpec {
        key: "design-engineering",
        name: "Design Engineering",
        icon: "lucide.frame",
    },
    ProjectSpec {
        key: "sales-marketing",
        name: "Sales & Marketing",
        icon: "lucide.chart-pie",
    },
    ProjectSpec {
        key: "travel",
        name: "Travel",
        icon: "lucide.map",
    },
];

fn sidebar_icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement {
    shadcn::raw::icon::icon(cx, IconId::new_static(id)).into_element(cx)
}

fn avatar_badge(cx: &mut UiCx<'_>, initials: &'static str) -> AnyElement {
    shadcn::Avatar::new([shadcn::AvatarFallback::new(initials).into_element(cx)])
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
}

fn copy_stack(
    cx: &mut UiCx<'_>,
    title: impl Into<Arc<str>>,
    subtitle: impl Into<Arc<str>>,
) -> AnyElement {
    let title = title.into();
    let subtitle = subtitle.into();
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::small(title.clone()).into_element(cx),
            shadcn::raw::typography::muted(subtitle.clone()).into_element(cx),
        ]
    })
    .gap(Space::N0)
    .items_start()
    .layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx)
}

fn set_text_model(model: Model<Arc<str>>, value: &'static str) -> OnActivate {
    let value = Arc::<str>::from(value);
    Arc::new(
        move |host: &mut dyn UiActionHost, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host
                .models_mut()
                .update(&model, |current| *current = value.clone());
            host.request_redraw(action_cx.window);
        },
    )
}

fn set_team_model(
    active_team: Model<usize>,
    last_action: Model<Arc<str>>,
    index: usize,
    action: &'static str,
) -> OnActivate {
    let action = Arc::<str>::from(action);
    Arc::new(
        move |host: &mut dyn UiActionHost, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host
                .models_mut()
                .update(&active_team, |current| *current = index);
            let _ = host
                .models_mut()
                .update(&last_action, |current| *current = action.clone());
            host.request_redraw(action_cx.window);
        },
    )
}

fn set_selected_model(
    selected: Model<Arc<str>>,
    last_action: Model<Arc<str>>,
    value: &'static str,
    action: &'static str,
) -> OnActivate {
    let value = Arc::<str>::from(value);
    let action = Arc::<str>::from(action);
    Arc::new(
        move |host: &mut dyn UiActionHost, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host
                .models_mut()
                .update(&selected, |current| *current = value.clone());
            let _ = host
                .models_mut()
                .update(&last_action, |current| *current = action.clone());
            host.request_redraw(action_cx.window);
        },
    )
}

fn team_switcher(
    cx: &mut UiCx<'_>,
    active_team: Model<usize>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let active_team_index = cx
        .get_model_cloned(&active_team, Invalidation::Layout)
        .unwrap_or(0)
        .min(TEAMS.len().saturating_sub(1));
    let team = TEAMS.get(active_team_index).copied().unwrap_or(TEAMS[0]);
    let sidebar_ctx = shadcn::use_sidebar(cx).expect("sidebar context");
    let side = if sidebar_ctx.is_mobile {
        shadcn::DropdownMenuSide::Bottom
    } else {
        shadcn::DropdownMenuSide::Right
    };
    let align = if sidebar_ctx.is_mobile {
        shadcn::DropdownMenuAlign::End
    } else {
        shadcn::DropdownMenuAlign::Start
    };

    let trigger = shadcn::SidebarMenuButton::new(team.name)
        .size(shadcn::SidebarMenuButtonSize::Lg)
        .children([
            sidebar_icon(cx, team.icon),
            copy_stack(cx, team.name, team.plan),
            sidebar_icon(cx, "lucide.chevrons-up-down"),
        ])
        .test_id("ui-gallery-sidebar-app-sidebar-team-trigger")
        .into_element(cx);

    let mut team_entries = vec![shadcn::DropdownMenuLabel::new("Teams").into()];
    for (index, option) in TEAMS.iter().enumerate() {
        team_entries.push(
            shadcn::DropdownMenuItem::new(option.name)
                .on_activate(set_team_model(
                    active_team.clone(),
                    last_action.clone(),
                    index,
                    option.name,
                ))
                .into(),
        );
    }

    shadcn::SidebarMenu::new([shadcn::SidebarMenuItem::new(
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(shadcn::DropdownMenuTrigger::new(trigger))
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(align)
                    .side(side)
                    .side_offset(Px(4.0))
                    .min_width(Px(220.0)),
            )
            .entries([
                shadcn::DropdownMenuGroup::new(team_entries).into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuItem::new("Add team")
                    .on_activate(set_text_model(last_action, "add-team"))
                    .into(),
            ])
            .into_element(cx),
    )
    .into_element(cx)])
    .into_element(cx)
}

fn nav_main(
    cx: &mut UiCx<'_>,
    selected: Model<Arc<str>>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let selected_value = cx
        .get_model_cloned(&selected, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("history"));

    let mut sections = Vec::new();
    sections.push(shadcn::SidebarGroupLabel::new("Platform").into_element(cx));

    let mut items = Vec::new();
    for section in NAV_SECTIONS {
        let open = cx
            .local_model_keyed(format!("sidebar.app_sidebar.open.{}", section.key), || {
                section.default_open
            });
        let section_is_active = section
            .items
            .iter()
            .any(|item| item.key == selected_value.as_ref());

        let collapsible = shadcn::CollapsibleRoot::new()
            .open(open)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx, |cx| {
                let button = shadcn::SidebarMenuButton::new(section.title)
                    .active(section_is_active)
                    .children([
                        sidebar_icon(cx, section.icon),
                        shadcn::raw::typography::small(section.title).into_element(cx),
                        sidebar_icon(cx, "lucide.chevron-right"),
                    ])
                    .test_id(format!(
                        "ui-gallery-sidebar-app-sidebar-section-{}",
                        section.key
                    ))
                    .into_element(cx);

                let trigger = shadcn::CollapsibleTriggerPart::new([button])
                    .as_child(true)
                    .into_element(cx);

                let mut sub_items = Vec::new();
                for item in section.items {
                    sub_items.push(
                        shadcn::SidebarMenuSubItem::new(
                            shadcn::SidebarMenuSubButton::new(item.title)
                                .active(selected_value.as_ref() == item.key)
                                .on_activate(set_selected_model(
                                    selected.clone(),
                                    last_action.clone(),
                                    item.key,
                                    item.title,
                                ))
                                .into_element(cx),
                        )
                        .into_element(cx),
                    );
                }
                let content =
                    shadcn::CollapsibleContentPart::new([
                        shadcn::SidebarMenuSub::new(sub_items).into_element(cx)
                    ])
                    .into_element(cx);

                vec![
                    shadcn::SidebarMenuItem::new(trigger).into_element(cx),
                    content,
                ]
            });

        items.push(collapsible);
    }

    sections.push(shadcn::SidebarMenu::new(items).into_element(cx));
    shadcn::SidebarGroup::new(sections).into_element(cx)
}

fn project_groups(
    cx: &mut UiCx<'_>,
    selected: Model<Arc<str>>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let sidebar_ctx = shadcn::use_sidebar(cx).expect("sidebar context");
    let collapsed = !sidebar_ctx.is_mobile && sidebar_ctx.collapsed();
    if collapsed {
        return cx.text("");
    }

    let mut children = vec![
        shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
        shadcn::SidebarGroupAction::new([sidebar_icon(cx, "lucide.plus")])
            .a11y_label("Add project")
            .on_activate(set_text_model(last_action.clone(), "add-project"))
            .into_element(cx),
    ];

    let mut project_rows = Vec::new();
    let selected_value = cx
        .get_model_cloned(&selected, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("history"));

    for project in PROJECTS {
        let button = shadcn::SidebarMenuButton::new(project.name)
            .active(selected_value.as_ref() == project.key)
            .children([
                sidebar_icon(cx, project.icon),
                shadcn::raw::typography::small(project.name).into_element(cx),
            ])
            .on_activate(set_selected_model(
                selected.clone(),
                last_action.clone(),
                project.key,
                project.name,
            ))
            .into_element(cx);

        let action_trigger =
            shadcn::SidebarMenuAction::new([sidebar_icon(cx, "lucide.more-horizontal")])
                .a11y_label(format!("Project actions for {}", project.name))
                .show_on_hover(true)
                .test_id(format!(
                    "ui-gallery-sidebar-app-sidebar-project-action-{}",
                    project.key
                ))
                .into_element(cx);

        let menu = shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(shadcn::DropdownMenuTrigger::new(action_trigger))
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side(shadcn::DropdownMenuSide::Right)
                    .side_offset(Px(4.0))
                    .min_width(Px(180.0)),
            )
            .entries([
                shadcn::DropdownMenuItem::new("View project")
                    .on_activate(set_selected_model(
                        selected.clone(),
                        last_action.clone(),
                        project.key,
                        "view-project",
                    ))
                    .into(),
                shadcn::DropdownMenuItem::new("Share project")
                    .on_activate(set_text_model(last_action.clone(), "share-project"))
                    .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuItem::new("Archive project")
                    .on_activate(set_text_model(last_action.clone(), "archive-project"))
                    .into(),
            ])
            .into_element(cx);

        project_rows.push(
            shadcn::SidebarMenuItem::new(button)
                .extend_children([menu])
                .into_element(cx),
        );
    }

    children.push(shadcn::SidebarMenu::new(project_rows).into_element(cx));
    shadcn::SidebarGroup::new(children).into_element(cx)
}

fn nav_user(cx: &mut UiCx<'_>, last_action: Model<Arc<str>>) -> AnyElement {
    let sidebar_ctx = shadcn::use_sidebar(cx).expect("sidebar context");
    let side = if sidebar_ctx.is_mobile {
        shadcn::DropdownMenuSide::Bottom
    } else {
        shadcn::DropdownMenuSide::Right
    };

    let trigger = shadcn::SidebarMenuButton::new("shadcn")
        .size(shadcn::SidebarMenuButtonSize::Lg)
        .children([
            avatar_badge(cx, "SC"),
            copy_stack(cx, "shadcn", "m@example.com"),
            sidebar_icon(cx, "lucide.chevrons-up-down"),
        ])
        .test_id("ui-gallery-sidebar-app-sidebar-user-trigger")
        .into_element(cx);

    shadcn::SidebarMenu::new([shadcn::SidebarMenuItem::new(
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(shadcn::DropdownMenuTrigger::new(trigger))
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::End)
                    .side(side)
                    .side_offset(Px(4.0))
                    .min_width(Px(208.0)),
            )
            .entries([
                shadcn::DropdownMenuLabel::new("Account").into(),
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Profile")
                        .on_activate(set_text_model(last_action.clone(), "profile"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Billing")
                        .on_activate(set_text_model(last_action.clone(), "billing"))
                        .into(),
                    shadcn::DropdownMenuItem::new("Notifications")
                        .on_activate(set_text_model(last_action.clone(), "notifications"))
                        .into(),
                ])
                .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuLabel::new("shadcn · m@example.com")
                    .inset(true)
                    .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuItem::new("Log out")
                    .on_activate(set_text_model(last_action, "log-out"))
                    .into(),
            ])
            .into_element(cx),
    )
    .into_element(cx)])
    .into_element(cx)
}

fn app_sidebar(
    cx: &mut UiCx<'_>,
    active_team: Model<usize>,
    selected: Model<Arc<str>>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let header = shadcn::SidebarHeader::new([team_switcher(cx, active_team, last_action.clone())])
        .into_element(cx);
    let content = shadcn::SidebarContent::new([
        nav_main(cx, selected.clone(), last_action.clone()),
        project_groups(cx, selected, last_action.clone()),
    ])
    .into_element(cx);
    let footer = shadcn::SidebarFooter::new([nav_user(cx, last_action)]).into_element(cx);

    shadcn::Sidebar::new([
        header,
        content,
        footer,
        shadcn::SidebarRail::new()
            .test_id("ui-gallery-sidebar-app-sidebar-rail")
            .into_element(cx),
    ])
    .collapsible(shadcn::SidebarCollapsible::Icon)
    .refine_layout(LayoutRefinement::default().h_full())
    .into_element(cx)
}

fn shell_card(cx: &mut UiCx<'_>, title: &'static str, body: impl Into<Arc<str>>) -> AnyElement {
    let body = body.into();
    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| ui::children![cx; shadcn::card_title(title)]),
            shadcn::card_content(|cx| vec![cx.text(body.clone())]),
        ]
    })
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0().h_full())
    .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    cx.scope(|cx| {
        let active_team = cx.local_model_keyed("active_team", || 0usize);
        let selected = cx.local_model_keyed("selected", || Arc::<str>::from("history"));
        let last_action = cx.local_model_keyed("last_action", || Arc::<str>::from("ready"));

        let content = shadcn::SidebarProvider::new()
            .width(Px(288.0))
            .width_icon(Px(56.0))
            .width_mobile(Px(320.0))
            .with(cx, |cx| {
                let active_team_index = cx
                    .get_model_cloned(&active_team, Invalidation::Layout)
                    .unwrap_or(0)
                    .min(TEAMS.len().saturating_sub(1));
                let team = TEAMS
                    .get(active_team_index)
                    .copied()
                    .unwrap_or(TEAMS[0]);
                let selected_value = cx
                    .get_model_cloned(&selected, Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from("history"));
                let last_action_value = cx
                    .get_model_cloned(&last_action, Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from("ready"));

                let header = ui::h_flex(|cx| {
                    vec![
                        shadcn::SidebarTrigger::new()
                            .test_id("ui-gallery-sidebar-app-sidebar-trigger")
                            .into_element(cx),
                        shadcn::Separator::new()
                            .orientation(shadcn::SeparatorOrientation::Vertical)
                            .into_element(cx),
                        copy_stack(
                            cx,
                            "Build Your Application",
                            selected_value.as_ref(),
                        ),
                    ]
                })
                .gap(Space::N3)
                .items_center()
                .layout(LayoutRefinement::default().w_full().h_px(Px(40.0)).min_w_0())
                .into_element(cx);

                let stats_row = ui::h_flex(|cx| {
                    vec![
                        shell_card(cx, "Team", team.name),
                        shell_card(cx, "Selected", selected_value.as_ref()),
                        shell_card(cx, "Last action", last_action_value.as_ref()),
                    ]
                })
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0().h_px(Px(132.0)))
                .into_element(cx);

                let activity = shadcn::card(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_header(
                            |cx| ui::children![cx; shadcn::card_title("Sidebar-07 aligned shell")],
                        ),
                        shadcn::card_content(|cx| {
                            vec![
                                cx.text("This example keeps the upstream AppSidebar information architecture but inlines the helper files into one copyable Fret snippet."),
                                cx.text(format!("team={} selected={} last_action={}", team.name, selected_value.as_ref(), last_action_value.as_ref())),
                            ]
                        }),
                    ]
                })
                .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx);

                let inset = shadcn::SidebarInset::new([
                    ui::v_flex(|_cx| vec![header, stats_row, activity])
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                        .into_element(cx),
                ])
                .into_element(cx);
                let sidebar = app_sidebar(
                    cx,
                    active_team.clone(),
                    selected.clone(),
                    last_action.clone(),
                );

                let frame = ui::h_flex(|_cx| vec![sidebar, inset])
                .gap(Space::N0)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(520.0)))
                .into_element(cx)
                .test_id("ui-gallery-sidebar-app-sidebar-shell");

                vec![frame]
            });

        content
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("Missing AppSidebar content."))
    })
}

// endregion: example
