use super::super::*;

pub(super) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct NavigationMenuModels {
        demo_value: Option<Model<Option<Arc<str>>>>,
        rtl_value: Option<Model<Option<Arc<str>>>>,
    }

    let muted_foreground = cx.with_theme(|theme| theme.color_required("muted-foreground"));

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

    let state = cx.with_state(NavigationMenuModels::default, |st| st.clone());
    let demo_value = match state.demo_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(NavigationMenuModels::default, |st| {
                st.demo_value = Some(model.clone())
            });
            model
        }
    };
    let rtl_value = match state.rtl_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(NavigationMenuModels::default, |st| {
                st.rtl_value = Some(model.clone())
            });
            model
        }
    };

    let list_item = |cx: &mut ElementContext<'_, App>,
                     model: Model<Option<Arc<str>>>,
                     title: &'static str,
                     description: &'static str,
                     command: &'static str| {
        let title_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(title),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: None,
                letter_spacing_em: None,
            }),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });
        let description_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from(description),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: None,
                letter_spacing_em: None,
            }),
            color: Some(muted_foreground),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            move |_cx| [title_el, description_el],
        );

        shadcn::NavigationMenuLink::new(model, [body])
            .label(title)
            .on_click(command)
            .into_element(cx)
    };

    let icon_row = |cx: &mut ElementContext<'_, App>,
                    model: Model<Option<Arc<str>>>,
                    icon: &'static str,
                    label: &'static str,
                    command: &'static str| {
        let icon_el = shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon));
        let label_el = cx.text(label);
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |_cx| [icon_el, label_el],
        );
        shadcn::NavigationMenuLink::new(model, [row])
            .label(label)
            .on_click(command)
            .into_element(cx)
    };

    let demo = {
        let getting_started = shadcn::NavigationMenuItem::new(
            "getting_started",
            "Getting started",
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(384.0)).min_w_0()),
                |cx| {
                    vec![
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Introduction",
                            "Re-usable components built with Tailwind CSS.",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Installation",
                            "How to install dependencies and structure your app.",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Typography",
                            "Styles for headings, paragraphs, lists...etc",
                            CMD_APP_OPEN,
                        ),
                    ]
                },
            )],
        );

        let components = shadcn::NavigationMenuItem::new(
            "components",
            "Components",
            [stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(600.0)).min_w_0()),
                |cx| {
                    let left = stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Alert Dialog",
                                    "A modal dialog that interrupts the user with important content and expects a response.",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Hover Card",
                                    "For sighted users to preview content available behind a link.",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Progress",
                                    "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                                    CMD_APP_OPEN,
                                ),
                            ]
                        },
                    );

                    let right = stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Scroll-area",
                                    "Visually or semantically separates content.",
                                    CMD_APP_SAVE,
                                ),
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Tabs",
                                    "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                                    CMD_APP_SAVE,
                                ),
                                list_item(
                                    cx,
                                    demo_value.clone(),
                                    "Tooltip",
                                    "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                                    CMD_APP_SAVE,
                                ),
                            ]
                        },
                    );

                    [left, right]
                },
            )],
        );

        let with_icon = shadcn::NavigationMenuItem::new(
            "with_icon",
            "With Icon",
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()),
                |cx| {
                    vec![
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle-alert",
                            "Backlog",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle-dashed",
                            "To Do",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle-check",
                            "Done",
                            CMD_APP_OPEN,
                        ),
                    ]
                },
            )],
        );

        let docs = shadcn::NavigationMenuItem::new("docs", "Docs", std::iter::empty());

        let menu = shadcn::NavigationMenu::new(demo_value.clone())
            .list(shadcn::NavigationMenuList::new([
                getting_started,
                components,
                with_icon,
                docs,
            ]))
            .into_element(cx);
        let body = centered(cx, menu);
        section(cx, "Demo", body)
    };

    let rtl = {
        let menu = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let getting_started = shadcn::NavigationMenuItem::new(
                    "getting_started",
                    "البدء",
                    [stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N1)
                            .items_start()
                            .layout(LayoutRefinement::default().w_px(Px(384.0)).min_w_0()),
                        |cx| {
                            vec![
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "مقدمة",
                                    "مكونات قابلة لإعادة الاستخدام مبنية باستخدام Tailwind CSS.",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "التثبيت",
                                    "كيفية تثبيت التبعيات وتنظيم تطبيقك.",
                                    CMD_APP_OPEN,
                                ),
                                list_item(
                                    cx,
                                    rtl_value.clone(),
                                    "الطباعة",
                                    "أنماط للعناوين والفقرات والقوائم...إلخ",
                                    CMD_APP_OPEN,
                                ),
                            ]
                        },
                    )],
                );

                let components = shadcn::NavigationMenuItem::new(
                    "components",
                    "المكونات",
                    [stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_start()
                            .layout(LayoutRefinement::default().w_px(Px(600.0)).min_w_0()),
                        |cx| {
                            let left = stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N2).items_start(),
                                |cx| {
                                    vec![
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "حوار التنبيه",
                                            "حوار نافذة يقطع المستخدم بمحتوى مهم ويتوقع استجابة.",
                                            CMD_APP_OPEN,
                                        ),
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "بطاقة التحويم",
                                            "للمستخدمين المبصرين لمعاينة المحتوى المتاح خلف الرابط.",
                                            CMD_APP_OPEN,
                                        ),
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "التقدم",
                                            "يعرض مؤشرًا يوضح تقدم إتمام المهمة، عادةً يتم عرضه كشريط تقدم.",
                                            CMD_APP_OPEN,
                                        ),
                                    ]
                                },
                            );

                            let right = stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N2).items_start(),
                                |cx| {
                                    vec![
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "منطقة التمرير",
                                            "يفصل المحتوى بصريًا أو دلاليًا.",
                                            CMD_APP_SAVE,
                                        ),
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "التبويبات",
                                            "مجموعة من أقسام المحتوى المتعددة الطبقات—المعروفة بألواح التبويب—التي يتم عرضها واحدة في كل مرة.",
                                            CMD_APP_SAVE,
                                        ),
                                        list_item(
                                            cx,
                                            rtl_value.clone(),
                                            "تلميح",
                                            "نافذة منبثقة تعرض معلومات متعلقة بعنصر عندما يتلقى العنصر التركيز على لوحة المفاتيح أو عند تحويم الماوس فوقه.",
                                            CMD_APP_SAVE,
                                        ),
                                    ]
                                },
                            );

                            [left, right]
                        },
                    )],
                );

                let with_icon = shadcn::NavigationMenuItem::new(
                    "with_icon",
                    "مع أيقونة",
                    [stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N1)
                            .items_start()
                            .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()),
                        |cx| {
                            vec![
                                icon_row(
                                    cx,
                                    rtl_value.clone(),
                                    "lucide.circle-alert",
                                    "قائمة الانتظار",
                                    CMD_APP_OPEN,
                                ),
                                icon_row(
                                    cx,
                                    rtl_value.clone(),
                                    "lucide.circle-dashed",
                                    "المهام",
                                    CMD_APP_OPEN,
                                ),
                                icon_row(
                                    cx,
                                    rtl_value.clone(),
                                    "lucide.circle-check",
                                    "منجز",
                                    CMD_APP_OPEN,
                                ),
                            ]
                        },
                    )],
                );

                let docs = shadcn::NavigationMenuItem::new("docs", "الوثائق", std::iter::empty());

                shadcn::NavigationMenu::new(rtl_value.clone())
                    .list(shadcn::NavigationMenuList::new([
                        getting_started,
                        components,
                        with_icon,
                        docs,
                    ]))
                    .into_element(cx)
            },
        );
        let body = centered(cx, menu);
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N6).items_start(),
        |_cx| vec![demo, rtl],
    )]
}
