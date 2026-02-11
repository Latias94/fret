use super::super::super::super::*;

pub(in crate::ui) fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    _value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let primary = cx.with_theme(|theme| theme.color_required("primary"));
    let line_style = shadcn::tabs::TabsStyle::default()
        .trigger_background(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(CoreColor::TRANSPARENT),
        )))
        .trigger_border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(CoreColor::TRANSPARENT)))
                .when(
                    fret_ui_kit::WidgetStates::SELECTED,
                    Some(ColorRef::Color(primary)),
                ),
        );

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

    let card_panel = |cx: &mut ElementContext<'_, App>,
                      title: &'static str,
                      description: &'static str,
                      content: &'static str| {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(title).into_element(cx),
                shadcn::CardDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![shadcn::typography::muted(cx, content)]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .into_element(cx)
    };

    let demo = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "overview",
                    "Overview",
                    [card_panel(
                        cx,
                        "Overview",
                        "View your key metrics and recent project activity.",
                        "You have 12 active projects and 3 pending tasks.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "analytics",
                    "Analytics",
                    [card_panel(
                        cx,
                        "Analytics",
                        "Track performance and user engagement metrics.",
                        "Page views are up 25% compared to last month.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "reports",
                    "Reports",
                    [card_panel(
                        cx,
                        "Reports",
                        "Generate and download your detailed reports.",
                        "You have 5 reports ready and available to export.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Settings",
                    [card_panel(
                        cx,
                        "Settings",
                        "Manage your account preferences and options.",
                        "Configure notifications, security, and themes.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-demo");

        let demo_shell = shell(cx, tabs);
        let body = centered(cx, demo_shell);
        section(cx, "Demo", body)
    };

    let line = {
        let tabs = shadcn::Tabs::uncontrolled(Some("overview"))
            .style(line_style.clone())
            .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new()),
                shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new()),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-line");

        let group = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
            vec![
                tabs,
                shadcn::typography::muted(
                    cx,
                    "Line variant is approximated with trigger style overrides in current API.",
                ),
            ]
        });
        let body = centered(cx, group);
        section(cx, "Line", body)
    };

    let vertical = {
        let tabs = shadcn::Tabs::uncontrolled(Some("account"))
            .orientation(shadcn::tabs::TabsOrientation::Vertical)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .items([
                shadcn::TabsItem::new(
                    "account",
                    "Account",
                    [card_panel(
                        cx,
                        "Account",
                        "Update your account details and profile settings.",
                        "Display name and avatar were updated 2 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "password",
                    "Password",
                    [card_panel(
                        cx,
                        "Password",
                        "Change your password and keep your account secure.",
                        "Last password update was 28 days ago.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "notifications",
                    "Notifications",
                    [card_panel(
                        cx,
                        "Notifications",
                        "Choose how and when you receive updates.",
                        "Email alerts are enabled for build failures.",
                    )],
                ),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-vertical");

        let vertical_shell = shell(cx, tabs);
        let body = centered(cx, vertical_shell);
        section(cx, "Vertical", body)
    };

    let disabled = {
        let tabs = shadcn::Tabs::uncontrolled(Some("home"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "home",
                    "Home",
                    [card_panel(
                        cx,
                        "Home",
                        "This panel remains interactive.",
                        "The disabled tab cannot be focused or activated.",
                    )],
                ),
                shadcn::TabsItem::new(
                    "settings",
                    "Disabled",
                    [card_panel(
                        cx,
                        "Disabled",
                        "This panel should not become active.",
                        "",
                    )],
                )
                .disabled(true),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-disabled");

        let disabled_shell = shell(cx, tabs);
        let body = centered(cx, disabled_shell);
        section(cx, "Disabled", body)
    };

    let icons = {
        let preview_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.app-window")),
                    cx.text("Preview"),
                ]
            },
        );
        let code_trigger = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.code")),
                    cx.text("Code"),
                ]
            },
        );

        let tabs = shadcn::Tabs::uncontrolled(Some("preview"))
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new(
                    "preview",
                    "Preview",
                    [card_panel(
                        cx,
                        "Preview",
                        "Visual output for the current component.",
                        "Switch between preview and code using icon tabs.",
                    )],
                )
                .trigger_child(preview_trigger),
                shadcn::TabsItem::new(
                    "code",
                    "Code",
                    [card_panel(
                        cx,
                        "Code",
                        "Implementation details and source view.",
                        "This panel can host syntax-highlighted snippets.",
                    )],
                )
                .trigger_child(code_trigger),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-icons");

        let icons_shell = shell(cx, tabs);
        let body = centered(cx, icons_shell);
        section(cx, "Icons", body)
    };

    let rtl = {
        let tabs = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Tabs::uncontrolled(Some("overview"))
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
                    .items([
                        shadcn::TabsItem::new(
                            "overview",
                            "Overview",
                            [card_panel(
                                cx,
                                "Overview",
                                "RTL layout should keep keyboard and focus behavior intact.",
                                "Direction-sensitive navigation is provided by direction context.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "analytics",
                            "Analytics",
                            [card_panel(
                                cx,
                                "Analytics",
                                "Arrow-key movement follows RTL expectations.",
                                "Verify trigger order and selected styling in RTL mode.",
                            )],
                        ),
                        shadcn::TabsItem::new(
                            "reports",
                            "Reports",
                            [card_panel(
                                cx,
                                "Reports",
                                "Panel composition remains identical under RTL.",
                                "Only directional behavior should change.",
                            )],
                        ),
                    ])
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-tabs-rtl");

        let rtl_shell = shell(cx, tabs);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A set of layered sections of content that are displayed one at a time."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, line, vertical, disabled, icons, rtl],
        ),
    ]
}
