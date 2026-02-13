use super::super::*;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;

pub(super) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct BreadcrumbModels {
        dropdown_open: Option<Model<bool>>,
        dropdown_rtl_open: Option<Model<bool>>,
    }

    let (dropdown_open, dropdown_rtl_open) = cx.with_state(BreadcrumbModels::default, |st| {
        (st.dropdown_open.clone(), st.dropdown_rtl_open.clone())
    });
    let (dropdown_open, dropdown_rtl_open) = match (dropdown_open, dropdown_rtl_open) {
        (Some(dropdown_open), Some(dropdown_rtl_open)) => (dropdown_open, dropdown_rtl_open),
        _ => {
            let dropdown_open = cx.app.models_mut().insert(false);
            let dropdown_rtl_open = cx.app.models_mut().insert(false);
            cx.with_state(BreadcrumbModels::default, |st| {
                st.dropdown_open = Some(dropdown_open.clone());
                st.dropdown_rtl_open = Some(dropdown_rtl_open.clone());
            });
            (dropdown_open, dropdown_rtl_open)
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    fn slugify_section_title(title: &str) -> String {
        title
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        let title_test_id = format!(
            "ui-gallery-breadcrumb-section-title-{}",
            slugify_section_title(title)
        );
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    shadcn::typography::h4(cx, title).test_id(title_test_id),
                    body,
                ]
            },
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

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let trunc_layout = LayoutRefinement::default().max_w(Px(112.0));

    let demo_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-demo");
    let demo = section_card(cx, "Demo", demo_content);

    let basic_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-basic");
    let basic = section_card(cx, "Basic", basic_content);

    let custom_separator_content = shadcn::Breadcrumb::new()
        .separator(shadcn::BreadcrumbSeparator::Icon {
            icon: fret_icons::IconId::new_static("lucide.dot"),
            size: Px(14.0),
        })
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-separator");
    let custom_separator = section_card(cx, "Custom Separator", custom_separator_content);

    let dropdown_content = {
        let crumb = shadcn::breadcrumb::primitives::Breadcrumb::new().into_element(cx, |cx| {
            let list =
                shadcn::breadcrumb::primitives::BreadcrumbList::new().into_element(cx, |cx| {
                    let home = shadcn::breadcrumb::primitives::BreadcrumbItem::new().into_element(
                        cx,
                        |cx| {
                            vec![
                                shadcn::breadcrumb::primitives::BreadcrumbLink::new("Home")
                                    .into_element(cx),
                            ]
                        },
                    );

                    let dot = |cx: &mut ElementContext<'_, App>| {
                        shadcn::breadcrumb::primitives::BreadcrumbSeparator::new()
                            .kind(
                                shadcn::breadcrumb::primitives::BreadcrumbSeparatorKind::Icon {
                                    icon: fret_icons::IconId::new_static("lucide.dot"),
                                    size: Px(14.0),
                                },
                            )
                            .into_element(cx)
                    };

                    let components_dropdown = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                        .into_element(cx, |cx| {
                            let open_for_trigger = dropdown_open.clone();
                            let menu = shadcn::DropdownMenu::new(dropdown_open.clone())
                                .align(shadcn::DropdownMenuAlign::Start)
                                .into_element(
                                    cx,
                                    move |cx| {
                                        let (fg, muted) = cx.with_theme(|theme| {
                                            (
                                                theme.color_token("foreground"),
                                                theme.color_token("muted-foreground"),
                                            )
                                        });
                                        let mut props = fret_ui::element::PressableProps::default();
                                        props.a11y.label = Some(Arc::<str>::from("Components"));
                                        props.a11y.test_id = Some(Arc::<str>::from(
                                            "ui-gallery-breadcrumb-dropdown-trigger",
                                        ));

                                        cx.pressable(props, move |cx, st| {
                                            cx.pressable_toggle_bool(&open_for_trigger);
                                            let color = if st.hovered { fg } else { muted };
                                            let label = ui::text(cx, "Components")
                                                .text_color(fret_ui_kit::ColorRef::Color(color))
                                                .nowrap()
                                                .into_element(cx);
                                            let chevron = shadcn::icon::icon_with(
                                                cx,
                                                fret_icons::IconId::new_static(
                                                    "lucide.chevron-down",
                                                ),
                                                Some(Px(14.0)),
                                                Some(fret_ui_kit::ColorRef::Color(color)),
                                            );

                                            vec![stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap_x(Space::N1)
                                                    .items_center(),
                                                move |_cx| vec![label, chevron],
                                            )]
                                        })
                                    },
                                    |_cx| {
                                        vec![
                                            shadcn::DropdownMenuEntry::Item(
                                                shadcn::DropdownMenuItem::new("Documentation")
                                                    .on_select(CMD_APP_OPEN)
                                                    .test_id("ui-gallery-breadcrumb-dropdown-docs"),
                                            ),
                                            shadcn::DropdownMenuEntry::Item(
                                                shadcn::DropdownMenuItem::new("Themes")
                                                    .on_select(CMD_APP_OPEN),
                                            ),
                                            shadcn::DropdownMenuEntry::Item(
                                                shadcn::DropdownMenuItem::new("GitHub")
                                                    .on_select(CMD_APP_OPEN),
                                            ),
                                        ]
                                    },
                                );
                            vec![menu]
                        });

                    let page = shadcn::breadcrumb::primitives::BreadcrumbItem::new().into_element(
                        cx,
                        |cx| {
                            vec![
                                shadcn::breadcrumb::primitives::BreadcrumbPage::new("Breadcrumb")
                                    .into_element(cx),
                            ]
                        },
                    );

                    vec![home, dot(cx), components_dropdown, dot(cx), page]
                });

            vec![list]
        });
        crumb.test_id("ui-gallery-breadcrumb-dropdown")
    };
    let dropdown = section_card(cx, "Dropdown", dropdown_content);

    let collapsed_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Documentation"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-collapsed");
    let collapsed = section_card(cx, "Collapsed", collapsed_content);

    let link_component_content = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home (router link)")
                .truncate(true)
                .refine_layout(trunc_layout.clone()),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-link");
    let link_component = section_card(cx, "Link Component", link_component_content);

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            let crumb = shadcn::breadcrumb::primitives::Breadcrumb::new().into_element(cx, |cx| {
                let list =
                    shadcn::breadcrumb::primitives::BreadcrumbList::new().into_element(cx, |cx| {
                        let home = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                            .into_element(cx, |cx| {
                                vec![shadcn::breadcrumb::primitives::BreadcrumbLink::new("Home")
                                    .into_element(cx)]
                            });
                        let dot = |cx: &mut ElementContext<'_, App>| {
                            shadcn::breadcrumb::primitives::BreadcrumbSeparator::new()
                                .kind(shadcn::breadcrumb::primitives::BreadcrumbSeparatorKind::Icon {
                                    icon: fret_icons::IconId::new_static("lucide.dot"),
                                    size: Px(14.0),
                                })
                                .into_element(cx)
                        };
                        let components_dropdown = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                            .into_element(cx, |cx| {
                                let open_for_trigger = dropdown_rtl_open.clone();
                                let menu = shadcn::DropdownMenu::new(dropdown_rtl_open.clone())
                                    .align(shadcn::DropdownMenuAlign::End)
                                    .into_element(
                                        cx,
                                        move |cx| {
                                            let (fg, muted) = cx.with_theme(|theme| {
                                                (
                                                    theme.color_token("foreground"),
                                                    theme.color_token("muted-foreground"),
                                                )
                                            });
                                            let mut props = fret_ui::element::PressableProps::default();
                                            props.a11y.label = Some(Arc::<str>::from("Components"));
                                            props.a11y.test_id = Some(Arc::<str>::from(
                                                "ui-gallery-breadcrumb-rtl-dropdown-trigger",
                                            ));

                                            cx.pressable(props, move |cx, st| {
                                                cx.pressable_toggle_bool(&open_for_trigger);
                                                let color = if st.hovered { fg } else { muted };
                                                let label = ui::text(cx, "Components")
                                                    .text_color(fret_ui_kit::ColorRef::Color(color))
                                                    .nowrap()
                                                    .into_element(cx);
                                                let chevron = shadcn::icon::icon_with(
                                                    cx,
                                                    fret_icons::IconId::new_static("lucide.chevron-down"),
                                                    Some(Px(14.0)),
                                                    Some(fret_ui_kit::ColorRef::Color(color)),
                                                );

                                                vec![stack::hstack(
                                                    cx,
                                                    stack::HStackProps::default()
                                                        .gap_x(Space::N1)
                                                        .items_center(),
                                                    move |_cx| vec![label, chevron],
                                                )]
                                            })
                                        },
                                        |_cx| {
                                            vec![
                                                shadcn::DropdownMenuEntry::Item(
                                                    shadcn::DropdownMenuItem::new("Documentation")
                                                        .on_select(CMD_APP_OPEN)
                                                        .test_id("ui-gallery-breadcrumb-rtl-dropdown-docs"),
                                                ),
                                                shadcn::DropdownMenuEntry::Item(
                                                    shadcn::DropdownMenuItem::new("Themes")
                                                        .on_select(CMD_APP_OPEN),
                                                ),
                                                shadcn::DropdownMenuEntry::Item(
                                                    shadcn::DropdownMenuItem::new("GitHub")
                                                        .on_select(CMD_APP_OPEN),
                                                ),
                                            ]
                                        },
                                    );
                                vec![menu]
                            });
                        let page = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                            .into_element(cx, |cx| {
                                vec![shadcn::breadcrumb::primitives::BreadcrumbPage::new("Breadcrumb")
                                    .into_element(cx)]
                            });

                        vec![home, dot(cx), components_dropdown, dot(cx), page]
                    });
                vec![list]
            });
            crumb
        },
    )
    .test_id("ui-gallery-breadcrumb-rtl");
    let rtl = section_card(cx, "RTL", rtl_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Breadcrumb docs order for quick lookup and side-by-side behavior checks.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                basic,
                custom_separator,
                dropdown,
                collapsed,
                link_component,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-breadcrumb-component");

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
                    "Basic",
                    r#"Breadcrumb::new().items([
    BreadcrumbItem::new("Home"),
    BreadcrumbItem::new("Components"),
    BreadcrumbItem::new("Breadcrumb"),
])"#,
                ),
                code_block(
                    cx,
                    "Custom Separator + Collapsed",
                    r#"Breadcrumb::new()
    .separator(BreadcrumbSeparator::Icon {
        icon: IconId::new_static("lucide.dot"),
        size: Px(14.0),
    })
    .items([BreadcrumbItem::new("Home"), BreadcrumbItem::ellipsis(), ...])"#,
                ),
                code_block(
                    cx,
                    "Link + RTL",
                    r#"BreadcrumbItem::new("Home (router link)").truncate(true)
with_direction_provider(LayoutDirection::Rtl, |cx| Breadcrumb::new().items([...]).into_element(cx))"#,
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
                    "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current dropdown and router-link samples are visual approximations; full `asChild` composition can be added in a follow-up primitive demo.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Validate RTL with long labels to ensure truncation and separator spacing remain stable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-breadcrumb",
        component_panel,
        code_panel,
        notes_panel,
    )
}
