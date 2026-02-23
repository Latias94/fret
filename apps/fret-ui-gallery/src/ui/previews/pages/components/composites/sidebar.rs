use super::super::super::super::super::*;

pub(in crate::ui) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default, Clone)]
    struct SidebarModels {
        demo_selected: Option<Model<Arc<str>>>,
        controlled_open: Option<Model<bool>>,
        controlled_selected: Option<Model<Arc<str>>>,
        rtl_selected: Option<Model<Arc<str>>>,
    }

    let state = cx.with_state(SidebarModels::default, |st| st.clone());

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

    let controlled_open = match state.controlled_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_open = Some(model.clone())
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
                       test_id: Arc<str>| {
        let collapsed =
            shadcn::use_sidebar(cx).is_some_and(|ctx| !ctx.is_mobile && ctx.collapsed());
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
        let content = shadcn::SidebarProvider::new().with(cx, |cx| {
            let selected_value = resolve_selected(cx, &demo_selected, "playground");
            let collapsed =
                shadcn::use_sidebar(cx).is_some_and(|ctx| !ctx.is_mobile && ctx.collapsed());

            let header = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        shadcn::SidebarTrigger::new()
                            .test_id("ui-gallery-sidebar-demo-toggle")
                            .into_element(cx),
                        shadcn::Button::new("Focus")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id("ui-gallery-sidebar-demo-focus")
                            .into_element(cx),
                        shadcn::typography::muted(
                            cx,
                            if collapsed {
                                "Collapsed to icon rail"
                            } else {
                                "Expanded"
                            },
                        ),
                        shadcn::typography::muted(
                            cx,
                            format!("active={}", selected_value.as_ref()),
                        ),
                    ]
                },
            );

            let platform = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Platform").into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        demo_selected.clone(),
                        &selected_value,
                        "playground",
                        "Playground",
                        "lucide.square-terminal",
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
                        Arc::from("ui-gallery-sidebar-demo-item-settings"),
                    ))
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let projects = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        demo_selected.clone(),
                        &selected_value,
                        "design-engineering",
                        "Design Engineering",
                        "lucide.frame",
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
                shadcn::SidebarContent::new([platform, projects]).into_element(cx),
                shadcn::SidebarFooter::new([shadcn::typography::small(cx, "shadcn")])
                    .into_element(cx),
            ])
            .collapsible(shadcn::SidebarCollapsible::Icon)
            .refine_layout(LayoutRefinement::default().h_full())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-demo-sidebar"),
            );

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

            let inset = shadcn::SidebarInset::new([stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().h_full()),
                |_cx| vec![header, content],
            )])
            .into_element(cx);

            let frame = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().h_px(Px(360.0))),
                |_cx| vec![sidebar, inset],
            )
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sidebar-demo"),
            );

            vec![frame]
        });

        content
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("Missing sidebar demo content."))
    };

    let controlled = {
        let content = shadcn::SidebarProvider::new()
            .open(Some(controlled_open.clone()))
            .with(cx, |cx| {
                let open_now = cx
                    .watch_model(&controlled_open)
                    .layout()
                    .copied()
                    .unwrap_or(true);
                let selected_value =
                    resolve_selected(cx, &controlled_selected, "design-engineering");

                let header = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            shadcn::Button::new(if open_now {
                                "Close Sidebar"
                            } else {
                                "Open Sidebar"
                            })
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .toggle_model(controlled_open.clone())
                            .test_id("ui-gallery-sidebar-controlled-toggle")
                            .into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "Controlled via SidebarProvider.open(Some(model)).",
                            ),
                        ]
                    },
                );

                let projects = shadcn::SidebarGroup::new([
                    shadcn::SidebarGroupLabel::new("Projects").into_element(cx),
                    shadcn::SidebarMenu::new([
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            controlled_selected.clone(),
                            &selected_value,
                            "design-engineering",
                            "Design Engineering",
                            "lucide.frame",
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
                            Arc::from("ui-gallery-sidebar-controlled-item-feedback"),
                        ))
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx);

                let sidebar =
                    shadcn::Sidebar::new(
                        [shadcn::SidebarContent::new([projects]).into_element(cx)],
                    )
                    .collapsible(shadcn::SidebarCollapsible::Icon)
                    .refine_layout(LayoutRefinement::default().h_full())
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Group)
                            .test_id("ui-gallery-sidebar-controlled-sidebar"),
                    );

                let trigger = shadcn::SidebarTrigger::new()
                    .test_id("ui-gallery-sidebar-controlled-trigger")
                    .into_element(cx);

                let inset = shadcn::SidebarInset::new([shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Sidebar Inset").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        cx.text("Use a main content panel next to Sidebar when controlled."),
                        cx.text(format!("open={}", open_now)),
                        cx.text(format!("selected={}", selected_value.as_ref())),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx)])
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

                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N3)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| vec![header, trigger, frame],
                );

                vec![body]
            });

        content
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("Missing sidebar controlled content."))
    };

    let rtl = {
        let selected_value = resolve_selected(cx, &rtl_selected, "playground");

        let rtl_layout = doc_layout::rtl(cx, |cx| {
            let platform = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("المنصة")
                    .collapsed(false)
                    .into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        rtl_selected.clone(),
                        &selected_value,
                        "playground",
                        "ساحة اللعب",
                        "lucide.square-terminal",
                        Arc::from("ui-gallery-sidebar-rtl-item-playground"),
                    ))
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        rtl_selected.clone(),
                        &selected_value,
                        "documentation",
                        "الوثائق",
                        "lucide.book-open",
                        Arc::from("ui-gallery-sidebar-rtl-item-documentation"),
                    ))
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(menu_button(
                        cx,
                        rtl_selected.clone(),
                        &selected_value,
                        "settings",
                        "الإعدادات",
                        "lucide.settings-2",
                        Arc::from("ui-gallery-sidebar-rtl-item-settings"),
                    ))
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let sidebar = shadcn::Sidebar::new([
                shadcn::SidebarHeader::new([shadcn::typography::small(cx, "مؤسسة مثال")])
                    .into_element(cx),
                shadcn::SidebarContent::new([platform])
                    .collapsed(false)
                    .into_element(cx),
                shadcn::SidebarFooter::new([shadcn::typography::small(cx, "الدعم")])
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
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-rtl"),
        );

        rtl_layout
    };

    let notes = doc_layout::notes(
        cx,
        [
            "This is a Fret-specific demo (not part of upstream shadcn sink components).",
            "The controlled example demonstrates SidebarProvider controlled open state via a `Model<bool>`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A composable, themeable and customizable sidebar component."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-demo")
                .code(
                    "rust",
                    r#"shadcn::SidebarProvider::new().with(cx, |cx| {
    let sidebar = shadcn::Sidebar::new([
        shadcn::SidebarHeader::new([...]).into_element(cx),
        shadcn::SidebarContent::new([...]).into_element(cx),
        shadcn::SidebarFooter::new([...]).into_element(cx),
    ]).into_element(cx);

    let main = shadcn::SidebarInset::new([
        shadcn::SidebarTrigger::new().into_element(cx),
        // ...
    ]).into_element(cx);

    [stack::hstack(cx, stack::HStackProps::default(), |_cx| vec![sidebar, main])]
});"#,
                ),
            DocSection::new("Controlled", controlled)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-controlled")
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(true);

shadcn::SidebarProvider::new()
    .open(Some(open.clone()))
    .with(cx, |cx| {
        let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([...]).into_element(cx)])
            .into_element(cx);
        let main = shadcn::SidebarInset::new([shadcn::SidebarTrigger::new().into_element(cx)])
            .into_element(cx);
        [stack::hstack(cx, stack::HStackProps::default(), |_cx| vec![sidebar, main])]
    });"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Sidebar::new([...]).into_element(cx)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sidebar-notes"),
        ],
    );

    vec![body]
}
