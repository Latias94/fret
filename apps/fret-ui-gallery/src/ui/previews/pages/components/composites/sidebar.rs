use super::super::super::super::super::*;

pub(in crate::ui) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SidebarModels {
        demo_collapsed: Option<Model<bool>>,
        demo_selected: Option<Model<Arc<str>>>,
        controlled_collapsed: Option<Model<bool>>,
        controlled_selected: Option<Model<Arc<str>>>,
        rtl_selected: Option<Model<Arc<str>>>,
    }

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
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let state = cx.with_state(SidebarModels::default, |st| st.clone());

    let demo_collapsed = match state.demo_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.demo_collapsed = Some(model.clone())
            });
            model
        }
    };

    let demo_selected = match state.demo_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.demo_selected = Some(model.clone())
            });
            model
        }
    };

    let controlled_collapsed = match state.controlled_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_collapsed = Some(model.clone())
            });
            model
        }
    };

    let controlled_selected = match state.controlled_selected {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Arc::<str>::from("design-engineering"));
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let resolve_selected =
        |cx: &mut ElementContext<'_, App>, model: &Model<Arc<str>>, fallback: &'static str| {
            cx.get_model_cloned(model, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from(fallback))
        };

    let menu_button = |cx: &mut ElementContext<'_, App>,
                       selected_model: Model<Arc<str>>,
                       active_value: &Arc<str>,
                       value: &'static str,
                       label: &'static str,
                       icon: &'static str,
                       collapsed: bool,
                       test_id: Arc<str>| {
        let is_active = active_value.as_ref() == value;
        let selected_for_activate = selected_model.clone();
        let value_for_activate: Arc<str> = Arc::from(value);
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_for_activate, |v| *v = value_for_activate.clone());
            host.request_redraw(action_cx.window);
        });

        shadcn::SidebarMenuButton::new(label)
            .icon(fret_icons::IconId::new_static(icon))
            .active(is_active)
            .collapsed(collapsed)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let demo = {
        let is_collapsed = cx
            .watch_model(&demo_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &demo_selected, "playground");

        let toolbar = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Toggle")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .toggle_model(demo_collapsed.clone())
                        .test_id("ui-gallery-sidebar-demo-toggle")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        if is_collapsed {
                            "Collapsed to icon rail"
                        } else {
                            "Expanded"
                        },
                    ),
                    shadcn::typography::muted(cx, format!("active={}", selected_value.as_ref())),
                ]
            },
        );

        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Platform")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "playground",
                    "Playground",
                    "lucide.square-terminal",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-playground"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "models",
                    "Models",
                    "lucide.bot",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-models"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "documentation",
                    "Documentation",
                    "lucide.book-open",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-documentation"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "settings",
                    "Settings",
                    "lucide.settings-2",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-settings"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-travel"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([
            shadcn::SidebarHeader::new([shadcn::typography::small(cx, "Acme Inc.")])
                .into_element(cx),
            shadcn::SidebarContent::new([platform, projects])
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarFooter::new([shadcn::typography::small(cx, "shadcn")]).into_element(cx),
        ])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let content = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Content").into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("A sidebar that collapses to icon mode."),
                cx.text("Select any menu item to verify active and hover states."),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(360.0))),
            |_cx| vec![sidebar, content],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-demo"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![toolbar, framed],
        );
        section(cx, "Demo", body)
    };

    let controlled = {
        let is_collapsed = cx
            .watch_model(&controlled_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &controlled_selected, "design-engineering");

        let header = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::Button::new(if is_collapsed {
                        "Open Sidebar"
                    } else {
                        "Close Sidebar"
                    })
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(controlled_collapsed.clone())
                    .test_id("ui-gallery-sidebar-controlled-toggle")
                    .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "Controlled via model (approximation of SidebarProvider open state).",
                    ),
                ]
            },
        );

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-travel"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "support",
                    "Support",
                    "lucide.life-buoy",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-support"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "feedback",
                    "Feedback",
                    "lucide.send",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-feedback"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([projects])
            .collapsed(is_collapsed)
            .into_element(cx)])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let inset = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Sidebar Inset").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("Use a main content panel next to Sidebar when controlled."),
                cx.text(format!("selected={}", selected_value.as_ref())),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
            |_cx| vec![sidebar, inset],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-controlled"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![header, framed],
        );

        section(cx, "Controlled", body)
    };

    let rtl = {
        let selected_value = resolve_selected(cx, &rtl_selected, "playground");

        let rtl_layout = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let platform = shadcn::SidebarGroup::new([
                    shadcn::SidebarGroupLabel::new("??????")
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarMenu::new([
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "playground",
                            "????",
                            "lucide.square-terminal",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-playground"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "documentation",
                            "???????",
                            "lucide.book-open",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-documentation"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "settings",
                            "?????????",
                            "lucide.settings-2",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-settings"),
                        ))
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx);

                let sidebar = shadcn::Sidebar::new([
                    shadcn::SidebarHeader::new([shadcn::typography::small(cx, "?????? ???????")])
                        .into_element(cx),
                    shadcn::SidebarContent::new([platform])
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarFooter::new([shadcn::typography::small(cx, "??????")])
                        .into_element(cx),
                ])
                .collapsed(false)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

                let content = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("RTL").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        cx.text("Direction provider flips layout and inline icon/text flow."),
                        cx.text(format!("active={}", selected_value.as_ref())),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
                    |_cx| vec![content, sidebar],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-rtl"),
        );

        let framed = shell(cx, rtl_layout);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A composable, themeable and customizable sidebar component."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, controlled, rtl]
        }),
    ]
}
