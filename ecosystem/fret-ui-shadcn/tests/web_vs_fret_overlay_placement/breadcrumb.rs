use super::*;

#[test]
fn web_vs_fret_breadcrumb_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "breadcrumb-demo",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Toggle menu"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![
                                            bc::BreadcrumbEllipsis::new()
                                                .size(Px(16.0))
                                                .into_element(cx),
                                        ]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Components").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Toggle menu"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_demo_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "breadcrumb-demo.vp1440x320",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Toggle menu"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![
                                            bc::BreadcrumbEllipsis::new()
                                                .size(Px(16.0))
                                                .into_element(cx),
                                        ]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Components").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Toggle menu"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "breadcrumb-dropdown",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let theme = fret_ui::Theme::global(&*cx.app).clone();
                                    let muted = theme.color_required("muted-foreground");

                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Components"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![cx.flex(
                                            fret_ui::element::FlexProps {
                                                layout: Default::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(4.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let text = cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: Arc::from("Components"),
                                                    style: Some(shadcn_text_style(
                                                        theme.metric_required("font.size"),
                                                        theme.metric_required("font.line_height"),
                                                        FontWeight::NORMAL,
                                                    )),
                                                    color: Some(muted),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                });

                                                let icon =
                                                    fret_ui_kit::declarative::icon::icon_with(
                                                        cx,
                                                        fret_icons::ids::ui::CHEVRON_DOWN,
                                                        Some(Px(14.0)),
                                                        Some(fret_ui_kit::ColorRef::Color(muted)),
                                                    );

                                                vec![text, icon]
                                            },
                                        )]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Components"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "breadcrumb-dropdown.vp375x240",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let theme = fret_ui::Theme::global(&*cx.app).clone();
                                    let muted = theme.color_required("muted-foreground");

                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Components"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![cx.flex(
                                            fret_ui::element::FlexProps {
                                                layout: Default::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(4.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let text = cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: Arc::from("Components"),
                                                    style: Some(shadcn_text_style(
                                                        theme.metric_required("font.size"),
                                                        theme.metric_required("font.line_height"),
                                                        FontWeight::NORMAL,
                                                    )),
                                                    color: Some(muted),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                });

                                                let icon =
                                                    fret_ui_kit::declarative::icon::icon_with(
                                                        cx,
                                                        fret_icons::ids::ui::CHEVRON_DOWN,
                                                        Some(Px(14.0)),
                                                        Some(fret_ui_kit::ColorRef::Color(muted)),
                                                    );

                                                vec![text, icon]
                                            },
                                        )]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Components"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_dropdown_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "breadcrumb-dropdown.vp1440x320",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let theme = fret_ui::Theme::global(&*cx.app).clone();
                                    let muted = theme.color_required("muted-foreground");

                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Components"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![cx.flex(
                                            fret_ui::element::FlexProps {
                                                layout: Default::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(4.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let text = cx.text_props(TextProps {
                                                    layout: Default::default(),
                                                    text: Arc::from("Components"),
                                                    style: Some(shadcn_text_style(
                                                        theme.metric_required("font.size"),
                                                        theme.metric_required("font.line_height"),
                                                        FontWeight::NORMAL,
                                                    )),
                                                    color: Some(muted),
                                                    wrap: TextWrap::Word,
                                                    overflow: TextOverflow::Clip,
                                                });

                                                let icon =
                                                    fret_ui_kit::declarative::icon::icon_with(
                                                        cx,
                                                        fret_icons::ids::ui::CHEVRON_DOWN,
                                                        Some(Px(14.0)),
                                                        Some(fret_ui_kit::ColorRef::Color(muted)),
                                                    );

                                                vec![text, icon]
                                            },
                                        )]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Themes")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new()
                            .kind(bc::BreadcrumbSeparatorKind::Slash)
                            .into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Components"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_responsive_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "breadcrumb-responsive",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
            };

            let dropdown = DropdownMenu::new(open.clone()).align(DropdownMenuAlign::Start);

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![dropdown.into_element(
                                cx,
                                |cx| {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Toggle menu"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![
                                            bc::BreadcrumbEllipsis::new()
                                                .size(Px(16.0))
                                                .into_element(cx),
                                        ]
                                    })
                                },
                                |_cx| {
                                    vec![
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Documentation",
                                        )),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new(
                                            "Building Your Application",
                                        )),
                                    ]
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Data Fetching").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![
                                bc::BreadcrumbPage::new("Caching and Revalidating")
                                    .into_element(cx),
                            ]
                        }),
                    ]
                })]
            })
        },
        SemanticsRole::Button,
        Some("Toggle menu"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_breadcrumb_responsive_mobile_drawer_overlay_insets_match() {
    assert_viewport_anchored_overlay_placement_matches(
        "breadcrumb-responsive.vp375x812",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            use fret_ui_shadcn::breadcrumb::primitives as bc;
            use fret_ui_shadcn::{
                Button, ButtonVariant, Drawer, DrawerContent, DrawerDescription, DrawerFooter,
                DrawerHeader, DrawerTitle,
            };

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let text_px = theme.metric_required("font.size");
            let line_height = theme.metric_required("font.line_height");

            let drawer = Drawer::new(open.clone());

            bc::Breadcrumb::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbList::new().into_element(cx, |cx| {
                    vec![
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![bc::BreadcrumbLink::new("Home").into_element(cx)]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            vec![drawer.into_element(
                                cx,
                                |cx| {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y.role = Some(SemanticsRole::Button);
                                    props.a11y.label = Some(Arc::from("Toggle Menu"));

                                    cx.pressable(props, move |cx, _st| {
                                        vec![
                                            bc::BreadcrumbEllipsis::new()
                                                .size(Px(16.0))
                                                .into_element(cx),
                                        ]
                                    })
                                },
                                |cx| {
                                    DrawerContent::new(vec![
                                        DrawerHeader::new(vec![
                                            DrawerTitle::new("Navigate to").into_element(cx),
                                            DrawerDescription::new("Select a page to navigate to.")
                                                .into_element(cx),
                                        ])
                                        .into_element(cx),
                                        cx.container(
                                            ContainerProps {
                                                layout: LayoutStyle::default(),
                                                padding: Edges::symmetric(Px(16.0), Px(0.0)),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![stack::vstack(
                                                    cx,
                                                    stack::VStackProps::default()
                                                        .gap(Space::N1)
                                                        .items_stretch(),
                                                    move |cx| {
                                                        let mut row = |text: &str| {
                                                            let text_sm = shadcn_text_style(
                                                                text_px,
                                                                line_height,
                                                                FontWeight::NORMAL,
                                                            );
                                                            let text: Arc<str> = Arc::from(text);
                                                            cx.container(
                                                                ContainerProps {
                                                                    layout: LayoutStyle::default(),
                                                                    padding: Edges::symmetric(
                                                                        Px(0.0),
                                                                        Px(4.0),
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                                move |cx| {
                                                                    vec![shadcn_text_with_layout(
                                                                        cx,
                                                                        text.clone(),
                                                                        text_sm,
                                                                        LayoutStyle::default(),
                                                                    )]
                                                                },
                                                            )
                                                        };
                                                        vec![
                                                            row("Documentation"),
                                                            row("Building Your Application"),
                                                        ]
                                                    },
                                                )]
                                            },
                                        ),
                                        DrawerFooter::new(vec![
                                            Button::new("Close")
                                                .variant(ButtonVariant::Outline)
                                                .into_element(cx),
                                        ])
                                        .into_element(cx),
                                    ])
                                    .into_element(cx)
                                },
                            )]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            let layout = LayoutRefinement::default().max_w(Px(80.0));
                            vec![
                                bc::BreadcrumbLink::new("Data Fetching")
                                    .truncate(true)
                                    .refine_layout(layout)
                                    .into_element(cx),
                            ]
                        }),
                        bc::BreadcrumbSeparator::new().into_element(cx),
                        bc::BreadcrumbItem::new().into_element(cx, |cx| {
                            let layout = LayoutRefinement::default().max_w(Px(80.0));
                            vec![
                                bc::BreadcrumbPage::new("Caching and Revalidating")
                                    .truncate(true)
                                    .refine_layout(layout)
                                    .into_element(cx),
                            ]
                        }),
                    ]
                })]
            })
        },
    );
}
