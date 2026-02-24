use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
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

    let trunc_layout = LayoutRefinement::default().max_w(Px(112.0));

    let demo = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-demo");

    let basic = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-basic");

    let custom_separator = shadcn::Breadcrumb::new()
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

    let dropdown = {
        let crumb = shadcn::breadcrumb::primitives::Breadcrumb::new().into_element(cx, |cx| {
            let list =
                shadcn::breadcrumb::primitives::BreadcrumbList::new().into_element(cx, |cx| {
                    let home = shadcn::breadcrumb::primitives::BreadcrumbItem::new().into_element(
                        cx,
                        |cx| {
                            vec![
                                shadcn::breadcrumb::primitives::BreadcrumbLink::new("Home")
                                    .href("/home")
                                    .on_click(CMD_APP_OPEN)
                                    .into_element(cx)
                                    .test_id("ui-gallery-breadcrumb-dropdown-home-link"),
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

    let collapsed = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::ellipsis(),
            shadcn::BreadcrumbItem::new("Documentation"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-collapsed");

    let link_component = shadcn::Breadcrumb::new()
        .items([
            shadcn::BreadcrumbItem::new("Home")
                .href("https://example.com")
                // Keep this example deterministic under automation by default.
                // Remove to allow `Effect::OpenUrl` fallback.
                .on_activate(Arc::new(|_host, _acx, _reason| {}))
                .truncate(true)
                .refine_layout(trunc_layout.clone()),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-breadcrumb-link");

    let rtl = doc_layout::rtl(cx, |cx| {
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
                                                    .test_id(
                                                        "ui-gallery-breadcrumb-rtl-dropdown-docs",
                                                    ),
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
        crumb
    })
    .test_id("ui-gallery-breadcrumb-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
            "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
            "Dropdown and router-link samples use typed pressables/links (ADR 0115 avoids general Slot/`asChild` prop merging).",
            "Validate RTL with long labels to ensure truncation and separator spacing remain stable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Breadcrumb docs order for quick lookup and side-by-side behavior checks.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .title_test_id("ui-gallery-breadcrumb-section-title-demo")
                .description("Basic breadcrumb recipe with ellipsis and current page.")
                .code(
                    "rust",
                    r#"let breadcrumb = shadcn::Breadcrumb::new().items([
    shadcn::BreadcrumbItem::new("Home"),
    shadcn::BreadcrumbItem::ellipsis(),
    shadcn::BreadcrumbItem::new("Components"),
    shadcn::BreadcrumbItem::new("Breadcrumb"),
]);"#,
                ),
            DocSection::new("Basic", basic)
                .title_test_id("ui-gallery-breadcrumb-section-title-basic")
                .description("A minimal breadcrumb list with three items.")
                .code(
                    "rust",
                    r#"shadcn::Breadcrumb::new().items([
    shadcn::BreadcrumbItem::new("Home"),
    shadcn::BreadcrumbItem::new("Components"),
    shadcn::BreadcrumbItem::new("Breadcrumb"),
]);"#,
                ),
            DocSection::new("Custom Separator", custom_separator)
                .title_test_id("ui-gallery-breadcrumb-section-title-custom-separator")
                .description("Use a custom separator icon for parity with docs.")
                .code(
                    "rust",
                    r#"shadcn::Breadcrumb::new()
    .separator(shadcn::BreadcrumbSeparator::Icon {
        icon: fret_icons::IconId::new_static("lucide.dot"),
        size: Px(14.0),
    })
    .items([/* ... */]);"#,
                ),
            DocSection::new("Dropdown", dropdown)
                .title_test_id("ui-gallery-breadcrumb-section-title-dropdown")
                .description("Collapsed middle segment can expand via a dropdown menu.")
                .code(
                    "rust",
                    r#"// Primitives recipe: BreadcrumbItem hosts a DropdownMenu trigger.
let open = cx.app.models_mut().insert(false);

shadcn::breadcrumb::primitives::Breadcrumb::new().into_element(cx, |cx| {
    let list = shadcn::breadcrumb::primitives::BreadcrumbList::new().into_element(cx, |cx| {
        let home = shadcn::breadcrumb::primitives::BreadcrumbItem::new().into_element(cx, |cx| {
            vec![shadcn::breadcrumb::primitives::BreadcrumbLink::new("Home").into_element(cx)]
        });

        let components = shadcn::breadcrumb::primitives::BreadcrumbItem::new().into_element(cx, |cx| {
            let menu = shadcn::DropdownMenu::new(open.clone()).into_element(
                cx,
                |cx| shadcn::Button::new("Components").toggle_model(open.clone()).into_element(cx),
                |_cx| vec![
                    shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Documentation")),
                    shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Themes")),
                ],
            );
            vec![menu]
        });

        vec![home, components]
    });
    vec![list]
});"#,
                ),
            DocSection::new("Collapsed", collapsed)
                .title_test_id("ui-gallery-breadcrumb-section-title-collapsed")
                .description("Use `BreadcrumbItem::ellipsis` to keep paths readable in narrow layouts.")
                .code(
                    "rust",
                    r#"shadcn::Breadcrumb::new().items([
    shadcn::BreadcrumbItem::new("Home"),
    shadcn::BreadcrumbItem::ellipsis(),
    shadcn::BreadcrumbItem::new("Documentation"),
    shadcn::BreadcrumbItem::new("Components"),
    shadcn::BreadcrumbItem::new("Breadcrumb"),
]);"#,
                ),
            DocSection::new("Link Component", link_component)
                .title_test_id("ui-gallery-breadcrumb-section-title-link-component")
                .description("Example of a truncated router-link style item.")
                .code(
                    "rust",
                    r#"let trunc = LayoutRefinement::default().max_w(Px(112.0));

 shadcn::Breadcrumb::new().items([
    shadcn::BreadcrumbItem::new("Home")
        .href("https://example.com")
        .on_activate(Arc::new(|_host, _acx, _reason| {}))
        .truncate(true)
        .refine_layout(trunc),
    shadcn::BreadcrumbItem::new("Components"),
    shadcn::BreadcrumbItem::new("Breadcrumb"),
 ]);"#,
                ),
            DocSection::new("RTL", rtl)
                .title_test_id("ui-gallery-breadcrumb-section-title-rtl")
                .description("Breadcrumb layout should follow right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Breadcrumb::new().items([
            shadcn::BreadcrumbItem::new("Home"),
            shadcn::BreadcrumbItem::new("Components"),
            shadcn::BreadcrumbItem::new("Breadcrumb"),
        ])
        .into_element(cx)
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-breadcrumb-section-title-notes")
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-breadcrumb-component");

    vec![body]
}
