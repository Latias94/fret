pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::LayoutQueryRegionProps;
use fret_ui::element::{LayoutStyle, Length, TextProps};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
    let demo_value = cx.local_model_keyed("demo_value", || None::<Arc<str>>);
    let md_breakpoint_query_container =
        cx.local_model_keyed("md_breakpoint_query_container", || false);
    let use_container_query = cx
        .get_model_copied(&md_breakpoint_query_container, Invalidation::Layout)
        .unwrap_or(false);

    let list_item = |cx: &mut UiCx<'_>,
                     model: Model<Option<Arc<str>>>,
                     title: &'static str,
                     description: &'static str,
                     test_id: &'static str,
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
                ..Default::default()
            }),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });
        let mut description_layout = LayoutStyle::default();
        // Upstream `line-clamp-2` outcome.
        description_layout.size.max_height = Some(Length::Px(Px(40.0)));
        let description_el = cx.text_props(TextProps {
            layout: description_layout,
            text: Arc::from(description),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(Px(20.0)),
                letter_spacing_em: None,
                ..Default::default()
            }),
            color: Some(muted_foreground),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        let body = ui::v_stack(move |_cx| [title_el, description_el])
            .gap(Space::N1)
            .items_start()
            .into_element(cx);

        shadcn::NavigationMenuLink::new(model, [body])
            .label(title)
            .test_id(test_id)
            .action(command)
            .into_element(cx)
    };

    let icon_row = |cx: &mut UiCx<'_>,
                    model: Model<Option<Arc<str>>>,
                    icon: &'static str,
                    label: &'static str,
                    test_id: &'static str,
                    command: &'static str| {
        let icon_el = icon::icon(cx, fret_icons::IconId::new_static(icon));
        let label_el = cx.text(label);
        let row = ui::h_row(move |_cx| [icon_el, label_el])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);
        shadcn::NavigationMenuLink::new(model, [row])
            .label(label)
            .test_id(test_id)
            .action(command)
            .into_element(cx)
    };

    let toggle = ui::h_row(|cx| {
        vec![
            shadcn::Switch::new(md_breakpoint_query_container.clone())
                .a11y_label("Use container query breakpoints for the demo")
                .test_id("ui-gallery-navigation-menu-md-breakpoint-query-switch")
                .into_element(cx),
            cx.text("Use container query breakpoints (UI gallery)"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let demo_container_layout = cx.with_theme(|theme| {
        let layout = if use_container_query {
            // Keep the container below Tailwind's `md` threshold so the demo can showcase the
            // difference between viewport vs container breakpoints.
            LayoutRefinement::default().w_px(Px(560.0)).min_w_0()
        } else {
            LayoutRefinement::default().w_full().min_w_0()
        };
        fret_ui_kit::declarative::style::layout_style(theme, layout)
    });

    let demo = fret_ui_kit::declarative::container_query_region_with_id(
        cx,
        "ui-gallery.navigation-menu.demo",
        LayoutQueryRegionProps {
            layout: demo_container_layout,
            name: None,
        },
        move |cx, region_id| {
            let md_breakpoint = if use_container_query {
                fret_ui_kit::declarative::container_width_at_least(
                    cx,
                    region_id,
                    Invalidation::Layout,
                    false,
                    fret_ui_kit::declarative::container_queries::tailwind::MD,
                    fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                )
            } else {
                fret_ui_kit::declarative::viewport_width_at_least(
                    cx,
                    Invalidation::Layout,
                    fret_ui_kit::declarative::viewport_tailwind::MD,
                    fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
                )
            };

            let lg_breakpoint = if use_container_query {
                fret_ui_kit::declarative::container_width_at_least(
                    cx,
                    region_id,
                    Invalidation::Layout,
                    false,
                    fret_ui_kit::declarative::container_queries::tailwind::LG,
                    fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                )
            } else {
                fret_ui_kit::declarative::viewport_width_at_least(
                    cx,
                    Invalidation::Layout,
                    fret_ui_kit::declarative::viewport_tailwind::LG,
                    fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
                )
            };

            let getting_started = shadcn::NavigationMenuItem::new(
                "getting_started",
                "Getting started",
                [ui::v_stack(|cx| {
                    vec![
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Introduction",
                            "Re-usable components built with Tailwind CSS.",
                            "ui-gallery-navigation-menu-demo-link-introduction",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Installation",
                            "How to install dependencies and structure your app.",
                            "ui-gallery-navigation-menu-demo-link-installation",
                            CMD_APP_OPEN,
                        ),
                        list_item(
                            cx,
                            demo_value.clone(),
                            "Typography",
                            "Styles for headings, paragraphs, lists...etc",
                            "ui-gallery-navigation-menu-demo-link-typography",
                            CMD_APP_OPEN,
                        ),
                    ]
                })
                .gap(Space::N0)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(384.0)).min_w_0())
                .into_element(cx)],
            )
            .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-getting-started");

            let components_w_px = if lg_breakpoint {
                Px(600.0)
            } else if md_breakpoint {
                Px(500.0)
            } else {
                Px(400.0)
            };

            let components_specs = [
                (
                    "Alert Dialog",
                    "A modal dialog that interrupts the user with important content and expects a response.",
                    "ui-gallery-navigation-menu-demo-link-alert-dialog",
                    CMD_APP_OPEN,
                ),
                (
                    "Hover Card",
                    "For sighted users to preview content available behind a link.",
                    "ui-gallery-navigation-menu-demo-link-hover-card",
                    CMD_APP_OPEN,
                ),
                (
                    "Progress",
                    "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                    "ui-gallery-navigation-menu-demo-link-progress",
                    CMD_APP_OPEN,
                ),
                (
                    "Scroll-area",
                    "Visually or semantically separates content.",
                    "ui-gallery-navigation-menu-demo-link-scroll-area",
                    CMD_APP_SAVE,
                ),
                (
                    "Tabs",
                    "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                    "ui-gallery-navigation-menu-demo-link-tabs",
                    CMD_APP_SAVE,
                ),
                (
                    "Tooltip",
                    "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                    "ui-gallery-navigation-menu-demo-link-tooltip",
                    CMD_APP_SAVE,
                ),
            ];

            let components_content = if md_breakpoint {
                let mut col_left = Vec::new();
                let mut col_right = Vec::new();
                for (idx, (title, desc, test_id, command)) in components_specs.iter().enumerate() {
                    let el = list_item(cx, demo_value.clone(), title, desc, test_id, command);
                    if idx % 2 == 0 {
                        col_left.push(el);
                    } else {
                        col_right.push(el);
                    }
                }

                ui::h_row(move |cx| {
                    let left = ui::v_stack(move |_cx| col_left)
                        .gap(Space::N2)
                        .items_start()
                        .into_element(cx)
                        .test_id("ui-gallery-navigation-menu-demo-components-col-left");
                    let right = ui::v_stack(move |_cx| col_right)
                        .gap(Space::N2)
                        .items_start()
                        .into_element(cx)
                        .test_id("ui-gallery-navigation-menu-demo-components-col-right");
                    vec![left, right]
                })
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0())
                .into_element(cx)
                .test_id("ui-gallery-navigation-menu-demo-components-layout-two-col")
            } else {
                let demo_value_for_components = demo_value.clone();
                ui::v_stack(move |cx| {
                    components_specs
                        .into_iter()
                        .map(|(title, desc, test_id, command)| {
                            list_item(
                                cx,
                                demo_value_for_components.clone(),
                                title,
                                desc,
                                test_id,
                                command,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(components_w_px).min_w_0())
                .into_element(cx)
                .test_id("ui-gallery-navigation-menu-demo-components-layout-single-col")
            };

            let components =
                shadcn::NavigationMenuItem::new("components", "Components", [components_content])
                    .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-components");

            let with_icon = shadcn::NavigationMenuItem::new(
                "with_icon",
                "With Icon",
                [ui::v_stack(|cx| {
                    vec![
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle-question-mark",
                            "Backlog",
                            "ui-gallery-navigation-menu-demo-link-backlog",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle",
                            "To Do",
                            "ui-gallery-navigation-menu-demo-link-to-do",
                            CMD_APP_OPEN,
                        ),
                        icon_row(
                            cx,
                            demo_value.clone(),
                            "lucide.circle-check",
                            "Done",
                            "ui-gallery-navigation-menu-demo-link-done",
                            CMD_APP_OPEN,
                        ),
                    ]
                })
                .gap(Space::N0)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0())
                .into_element(cx)],
            )
            .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-with-icon");

            let docs = shadcn::NavigationMenuItem::new("docs", "Documentation", std::iter::empty())
                .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-docs")
                .action(CMD_APP_OPEN);

            let md_query = if use_container_query {
                shadcn::NavigationMenuMdBreakpointQuery::Container
            } else {
                shadcn::NavigationMenuMdBreakpointQuery::Viewport
            };

            let mut nav_menu = shadcn::navigation_menu(cx, demo_value.clone(), |_cx| {
                vec![getting_started, components, with_icon, docs]
            })
            .md_breakpoint_query(md_query)
            .viewport_test_id("ui-gallery-navigation-menu-demo-viewport");
            if use_container_query {
                nav_menu = nav_menu.container_query_region(region_id);
            }
            vec![
                nav_menu
                    .into_element(cx)
                    .test_id("ui-gallery-navigation-menu-demo"),
            ]
        },
    );

    ui::v_stack(move |_cx| vec![toggle, demo])
        .gap(Space::N3)
        .items_start()
        .into_element(cx)
}
// endregion: example
