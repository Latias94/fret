pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_app::App;
use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::TextProps;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    #[derive(Default, Clone)]
    struct NavigationMenuModels {
        demo_value: Option<Model<Option<Arc<str>>>>,
    }

    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));

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

    let md_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        fret_ui_kit::declarative::viewport_tailwind::MD,
        fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
    );
    let is_mobile = !md_breakpoint;

    let list_item = |cx: &mut ElementContext<'_, App>,
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
                ..Default::default()
            }),
            color: Some(muted_foreground),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            move |_cx| [title_el, description_el],
        );

        shadcn::NavigationMenuLink::new(model, [body])
            .label(title)
            .test_id(test_id)
            .on_click(command)
            .into_element(cx)
    };

    let icon_row = |cx: &mut ElementContext<'_, App>,
                    model: Model<Option<Arc<str>>>,
                    icon: &'static str,
                    label: &'static str,
                    test_id: &'static str,
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
            .test_id(test_id)
            .on_click(command)
            .into_element(cx)
    };

    let home_card = |cx: &mut ElementContext<'_, App>,
                     model: Model<Option<Arc<str>>>,
                     layout: LayoutRefinement| {
        let mut bg: Color = cx.with_theme(|theme| theme.color_token("muted"));
        bg.a *= 0.75;

        let title = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from("shadcn/ui"),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(18.0),
                weight: FontWeight::MEDIUM,
                ..Default::default()
            }),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });
        let description = cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::from("Beautifully designed components built with Tailwind CSS."),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(14.0),
                weight: FontWeight::NORMAL,
                ..Default::default()
            }),
            color: Some(muted_foreground),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        let theme = cx.with_theme(|theme| theme.clone());
        let wrapper = fret_ui_kit::declarative::style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded_md()
                .bg(ColorRef::Color(bg))
                .p_4(),
            LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .merge(layout),
        );
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .justify_end()
                .layout(LayoutRefinement::default().w_full().h_full().min_w_0()),
            move |_cx| vec![title, description],
        );
        let card = cx.container(wrapper, move |_cx| vec![body]);

        shadcn::NavigationMenuLink::new(model, [card])
            .label("Home")
            .test_id("ui-gallery-navigation-menu-demo-link-home-card")
            .on_click(CMD_APP_OPEN)
            .refine_style(ChromeRefinement::default().p_0())
            .into_element(cx)
    };

    let home_content = if md_breakpoint {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(500.0)).min_w_0()),
            |cx| {
                let card = home_card(
                    cx,
                    demo_value.clone(),
                    LayoutRefinement::default().w_px(Px(180.0)).h_px(Px(192.0)),
                );

                let list = stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N2).items_start(),
                    |cx| {
                        vec![
                            list_item(
                                cx,
                                demo_value.clone(),
                                "Introduction",
                                "Re-usable components built using Radix UI and Tailwind CSS.",
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
                    },
                );

                vec![card, list]
            },
        )
    } else {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    home_card(
                        cx,
                        demo_value.clone(),
                        LayoutRefinement::default().h_px(Px(192.0)),
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Introduction",
                        "Re-usable components built using Radix UI and Tailwind CSS.",
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
            },
        )
    };

    let home = shadcn::NavigationMenuItem::new("home", "Home", [home_content])
        .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-home");

    let components_content = if md_breakpoint {
        stack::hstack(
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
                                "ui-gallery-navigation-menu-demo-link-alert-dialog",
                                CMD_APP_OPEN,
                            ),
                            list_item(
                                cx,
                                demo_value.clone(),
                                "Hover Card",
                                "For sighted users to preview content available behind a link.",
                                "ui-gallery-navigation-menu-demo-link-hover-card",
                                CMD_APP_OPEN,
                            ),
                            list_item(
                                cx,
                                demo_value.clone(),
                                "Progress",
                                "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                                "ui-gallery-navigation-menu-demo-link-progress",
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
                                "ui-gallery-navigation-menu-demo-link-scroll-area",
                                CMD_APP_SAVE,
                            ),
                            list_item(
                                cx,
                                demo_value.clone(),
                                "Tabs",
                                "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                                "ui-gallery-navigation-menu-demo-link-tabs",
                                CMD_APP_SAVE,
                            ),
                            list_item(
                                cx,
                                demo_value.clone(),
                                "Tooltip",
                                "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                                "ui-gallery-navigation-menu-demo-link-tooltip",
                                CMD_APP_SAVE,
                            ),
                        ]
                    },
                );

                vec![left, right]
            },
        )
    } else {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Alert Dialog",
                        "A modal dialog that interrupts the user with important content and expects a response.",
                        "ui-gallery-navigation-menu-demo-link-alert-dialog",
                        CMD_APP_OPEN,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Hover Card",
                        "For sighted users to preview content available behind a link.",
                        "ui-gallery-navigation-menu-demo-link-hover-card",
                        CMD_APP_OPEN,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Progress",
                        "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                        "ui-gallery-navigation-menu-demo-link-progress",
                        CMD_APP_OPEN,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Scroll-area",
                        "Visually or semantically separates content.",
                        "ui-gallery-navigation-menu-demo-link-scroll-area",
                        CMD_APP_SAVE,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Tabs",
                        "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                        "ui-gallery-navigation-menu-demo-link-tabs",
                        CMD_APP_SAVE,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Tooltip",
                        "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                        "ui-gallery-navigation-menu-demo-link-tooltip",
                        CMD_APP_SAVE,
                    ),
                ]
            },
        )
    };

    let components =
        shadcn::NavigationMenuItem::new("components", "Components", [components_content])
            .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-components");

    let docs = shadcn::NavigationMenuItem::new("docs", "Docs", std::iter::empty())
        .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-docs")
        .on_click(CMD_APP_OPEN);

    let list = shadcn::NavigationMenuItem::new(
        "list",
        "List",
        [stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(300.0)).min_w_0()),
            |cx| {
                vec![
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Components",
                        "Browse all components in the library.",
                        "ui-gallery-navigation-menu-demo-link-list-components",
                        CMD_APP_OPEN,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Documentation",
                        "Learn how to use the library.",
                        "ui-gallery-navigation-menu-demo-link-list-documentation",
                        CMD_APP_OPEN,
                    ),
                    list_item(
                        cx,
                        demo_value.clone(),
                        "Blog",
                        "Read our latest blog posts.",
                        "ui-gallery-navigation-menu-demo-link-list-blog",
                        CMD_APP_OPEN,
                    ),
                ]
            },
        )],
    )
    .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-list");

    let simple = shadcn::NavigationMenuItem::new(
        "simple",
        "Simple",
        [stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()),
            |cx| {
                vec![
                    shadcn::NavigationMenuLink::new(demo_value.clone(), [cx.text("Components")])
                        .label("Components")
                        .test_id("ui-gallery-navigation-menu-demo-link-simple-components")
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                    shadcn::NavigationMenuLink::new(demo_value.clone(), [cx.text("Documentation")])
                        .label("Documentation")
                        .test_id("ui-gallery-navigation-menu-demo-link-simple-documentation")
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                    shadcn::NavigationMenuLink::new(demo_value.clone(), [cx.text("Blocks")])
                        .label("Blocks")
                        .test_id("ui-gallery-navigation-menu-demo-link-simple-blocks")
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                ]
            },
        )],
    )
    .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-simple");

    let with_icon = shadcn::NavigationMenuItem::new(
        "with_icon",
        "With Icon",
        [stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(200.0)).min_w_0()),
            |cx| {
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
            },
        )],
    )
    .trigger_test_id("ui-gallery-navigation-menu-demo-trigger-with-icon");

    let mut items = vec![home, components, docs];
    if md_breakpoint {
        items.push(list);
        items.push(simple);
        items.push(with_icon);
    }

    shadcn::NavigationMenu::new(demo_value.clone())
        .viewport(is_mobile)
        .list(shadcn::NavigationMenuList::new(items))
        .viewport_test_id("ui-gallery-navigation-menu-demo-viewport")
        .into_element(cx)
        .test_id("ui-gallery-navigation-menu-demo")
}
// endregion: example
