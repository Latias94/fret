use fret_app::App;
use fret_core::{
    AppWindowId, Edges, Event, FontId, FontWeight, FrameId, KeyCode, Modifiers, MouseButton, Point,
    PointerEvent, PointerType, Px, Rect, SemanticsRole, SemanticsSnapshot, Size as CoreSize,
    TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{Effect, Model};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, GridProps, LayoutStyle, Length, MainAlign,
    MarginEdge, RowProps, TextProps,
};
use fret_ui::elements::{GlobalElementId, bounds_for_element};
use fret_ui::tree::UiTree;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, OverlayController, Space};
use serde::Deserialize;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    #[allow(dead_code)]
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    viewport: Option<WebViewport>,
    #[serde(default)]
    open: Option<WebOpenMeta>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebViewport {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebOpenMeta {
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    selector: String,
    point: WebPoint,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    #[serde(default)]
    active: bool,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy)]
enum Align {
    Start,
    Center,
    End,
}

fn bounds_for_web_theme(theme: &WebGoldenTheme) -> Rect {
    let w = theme.viewport.map(|v| v.w).unwrap_or(1440.0);
    let h = theme.viewport.map(|v| v.h).unwrap_or(900.0);
    Rect::new(Point::new(Px(0.0), Px(0.0)), CoreSize::new(Px(w), Px(h)))
}

fn pad_root<H: UiHost>(cx: &mut ElementContext<'_, H>, pad: Px, child: AnyElement) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            },
            padding: Edges::all(pad),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

fn shadcn_text_style(size: Px, line_height: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::default(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn shadcn_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn shadcn_text_with_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
    layout: LayoutStyle,
) -> AnyElement {
    cx.text_props(TextProps {
        layout,
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn shadcn_text_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    style: TextStyle,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn shadcn_nav_menu_demo_home_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    let gap = Px(8.0); // Tailwind `gap-2` (0.5rem).

    let link_title_style = shadcn_text_style(Px(14.0), Px(14.0), FontWeight::MEDIUM); // text-sm leading-none font-medium
    let link_desc_style = shadcn_text_style(Px(14.0), Px(19.25), FontWeight::NORMAL); // text-sm leading-snug

    let tile_title_style = shadcn_text_style(Px(18.0), Px(28.0), FontWeight::MEDIUM); // text-lg font-medium
    let tile_desc_style = shadcn_text_style(Px(14.0), Px(17.5), FontWeight::NORMAL); // text-sm leading-tight

    let tile = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill; // `w-full`
                layout
            },
            padding: Edges::all(Px(16.0)), // p-4
            ..Default::default()
        },
        move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    // NavigationMenuLink: `gap-1` + title: `mb-2` => 12px total.
                    gap: Px(4.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::End, // `justify-end`
                    align: CrossAlign::Stretch,
                },
                move |cx| {
                    vec![
                        shadcn_text_with_layout(cx, "shadcn/ui", tile_title_style, {
                            let mut layout = LayoutStyle::default();
                            layout.margin.bottom = MarginEdge::Px(Px(8.0)); // mb-2
                            layout
                        }),
                        shadcn_text(
                            cx,
                            "Beautifully designed components built with Tailwind CSS.",
                            tile_desc_style,
                        ),
                    ]
                },
            )]
        },
    );

    fn list_item<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        title: &'static str,
        description: &'static str,
        title_style: &TextStyle,
        desc_style: &TextStyle,
    ) -> AnyElement {
        let title = Arc::<str>::from(title);
        let description = Arc::<str>::from(description);
        let label = title.clone();
        let title_style = title_style.clone();
        let desc_style = desc_style.clone();

        let body = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges::all(Px(8.0)), // NavigationMenuLink: p-2
                ..Default::default()
            },
            move |cx| {
                let desc_style = desc_style.clone();
                vec![cx.column(
                    ColumnProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        gap: Px(4.0), // NavigationMenuLink: gap-1
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        let desc_style = desc_style.clone();
                        let desc_box = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.height = Length::Px(Px(desc_style
                                        .line_height
                                        .unwrap_or(Px(19.25))
                                        .0
                                        * 2.0));
                                    layout.overflow = fret_ui::element::Overflow::Clip;
                                    layout
                                },
                                padding: Edges::all(Px(0.0)),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![shadcn_text(cx, description.clone(), desc_style.clone())]
                            },
                        );

                        vec![shadcn_text(cx, title.clone(), title_style), desc_box]
                    },
                )]
            },
        );

        fret_ui_shadcn::NavigationMenuLink::new(model, vec![body])
            .label(label)
            .into_element(cx)
    }

    let intro = list_item(
        cx,
        model.clone(),
        "Introduction",
        "Re-usable components built using Radix UI and Tailwind CSS.",
        &link_title_style,
        &link_desc_style,
    );
    let install = list_item(
        cx,
        model.clone(),
        "Installation",
        "How to install dependencies and structure your app.",
        &link_title_style,
        &link_desc_style,
    );
    let typography = list_item(
        cx,
        model,
        "Typography",
        "Styles for headings, paragraphs, lists...etc",
        &link_title_style,
        &link_desc_style,
    );

    cx.grid(
        GridProps {
            layout: {
                let mut layout = LayoutStyle::default();
                // Match the extracted shadcn web golden (mobile viewport content width).
                layout.size.width = Length::Px(Px(271.76044));
                layout
            },
            cols: 1,
            rows: None,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |_cx| vec![tile, intro, install, typography],
    )
}

fn shadcn_nav_menu_demo_home_desktop_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _model: Model<Option<Arc<str>>>,
) -> AnyElement {
    let gap = Px(8.0); // Tailwind `gap-2` (0.5rem).

    let link_title_style = shadcn_text_style(Px(14.0), Px(14.0), FontWeight::MEDIUM); // text-sm leading-none font-medium
    let link_desc_style = shadcn_text_style(Px(14.0), Px(19.25), FontWeight::NORMAL); // text-sm leading-snug

    let tile_title_style = shadcn_text_style(Px(18.0), Px(28.0), FontWeight::MEDIUM); // text-lg font-medium
    let tile_desc_style = shadcn_text_style(Px(14.0), Px(17.5), FontWeight::NORMAL); // text-sm leading-tight

    // `lg:grid-cols-[.75fr_1fr]` at `lg:w-[500px]`.
    let available = 500.0 - gap.0;
    let left_w = Px(available * 0.75 / 1.75);
    let right_w = Px(available * 1.0 / 1.75);

    let tile = cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(left_w);
                layout
            },
            padding: Edges::all(Px(24.0)), // md:p-6
            ..Default::default()
        },
        move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    // NavigationMenuLink: `gap-1` + title: `mb-2` => 12px total.
                    gap: Px(4.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::End, // `justify-end`
                    align: CrossAlign::Stretch,
                },
                move |cx| {
                    vec![
                        shadcn_text_with_layout(cx, "shadcn/ui", tile_title_style, {
                            let mut layout = LayoutStyle::default();
                            layout.margin.bottom = MarginEdge::Px(Px(8.0)); // mb-2
                            layout
                        }),
                        shadcn_text(
                            cx,
                            "Beautifully designed components built with Tailwind CSS.",
                            tile_desc_style,
                        ),
                    ]
                },
            )]
        },
    );

    fn list_item<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        title: &'static str,
        description: &'static str,
        title_style: &TextStyle,
        desc_style: &TextStyle,
        desc_single_line: bool,
    ) -> AnyElement {
        let title = Arc::<str>::from(title);
        let description = Arc::<str>::from(description);
        let title_style = title_style.clone();
        let desc_style = desc_style.clone();
        let desc_single_line = desc_single_line;

        let body = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges::all(Px(8.0)), // NavigationMenuLink: p-2
                ..Default::default()
            },
            move |cx| {
                let desc_style = desc_style.clone();
                vec![cx.column(
                    ColumnProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        gap: Px(4.0), // NavigationMenuLink: gap-1
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        let desc = if desc_single_line {
                            shadcn_text_line(cx, description.clone(), desc_style.clone())
                        } else {
                            shadcn_text(cx, description.clone(), desc_style.clone())
                        };
                        vec![shadcn_text(cx, title.clone(), title_style), desc]
                    },
                )]
            },
        );

        body
    }

    let intro = list_item(
        cx,
        "Introduction",
        "Re-usable components built using Radix UI and Tailwind CSS.",
        &link_title_style,
        &link_desc_style,
        false,
    );
    let install = list_item(
        cx,
        "Installation",
        "How to install dependencies and structure your app.",
        &link_title_style,
        &link_desc_style,
        false,
    );
    let typography = list_item(
        cx,
        "Typography",
        "Styles for headings, paragraphs, lists...etc",
        &link_title_style,
        &link_desc_style,
        true,
    );

    let right = cx.column(
        ColumnProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(right_w);
                layout
            },
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |_cx| vec![intro, install, typography],
    );

    cx.row(
        RowProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(500.0)); // lg:w-[500px]
                layout
            },
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |_cx| vec![tile, right],
    )
}

fn shadcn_nav_menu_demo_simple_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::NavigationMenuLink;

    let link_style = shadcn_text_style(Px(14.0), Px(20.0), FontWeight::NORMAL); // text-sm

    fn link<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        label: &'static str,
        style: &TextStyle,
    ) -> AnyElement {
        let label_arc: Arc<str> = Arc::from(label);
        let style = style.clone();
        let body = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(Px(8.0)), // p-2
                ..Default::default()
            },
            move |cx| vec![shadcn_text_line(cx, label_arc.clone(), style.clone())],
        );

        NavigationMenuLink::child(model, body)
            .label(label)
            .into_element(cx)
    }

    let links = cx.column(
        ColumnProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(200.0)); // `w-[200px]`
                layout
            },
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |cx| {
            vec![
                link(cx, model.clone(), "Components", &link_style),
                link(cx, model.clone(), "Documentation", &link_style),
                link(cx, model.clone(), "Blocks", &link_style),
            ]
        },
    );

    links
}

fn shadcn_nav_menu_demo_with_icon_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::NavigationMenuLink;

    let link_style = shadcn_text_style(Px(14.0), Px(20.0), FontWeight::NORMAL); // text-sm

    fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    fn link<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        label: &'static str,
        style: &TextStyle,
    ) -> AnyElement {
        let label_arc: Arc<str> = Arc::from(label);
        let style = style.clone();
        let body = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(Px(8.0)), // p-2
                ..Default::default()
            },
            move |cx| {
                vec![cx.row(
                    RowProps {
                        layout: LayoutStyle::default(),
                        gap: Px(8.0), // gap-2
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                    },
                    move |cx| {
                        vec![
                            icon_stub(cx),
                            shadcn_text_line(cx, label_arc.clone(), style.clone()),
                        ]
                    },
                )]
            },
        );

        NavigationMenuLink::child(model, body)
            .label(label)
            .into_element(cx)
    }

    cx.column(
        ColumnProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(200.0)); // `w-[200px]`
                layout
            },
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |cx| {
            vec![
                link(cx, model.clone(), "Alert Dialog", &link_style),
                link(cx, model.clone(), "Hover Card", &link_style),
                link(cx, model.clone(), "Progress", &link_style),
            ]
        },
    )
}

fn shadcn_nav_menu_demo_list_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::NavigationMenuLink;

    let title_style = shadcn_text_style(Px(14.0), Px(20.0), FontWeight::MEDIUM); // text-sm font-medium
    let desc_style = shadcn_text_style(Px(14.0), Px(20.0), FontWeight::NORMAL); // text-sm

    fn link<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        title: &'static str,
        desc: &'static str,
        title_style: &TextStyle,
        desc_style: &TextStyle,
    ) -> AnyElement {
        let title_style = title_style.clone();
        let desc_style = desc_style.clone();
        let title_arc: Arc<str> = Arc::from(title);
        let desc_arc: Arc<str> = Arc::from(desc);

        let body = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(Px(8.0)), // p-2
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    ColumnProps {
                        layout: LayoutStyle::default(),
                        gap: Px(4.0), // gap-1
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        vec![
                            shadcn_text_line(cx, title_arc.clone(), title_style.clone()),
                            shadcn_text_line(cx, desc_arc.clone(), desc_style.clone()),
                        ]
                    },
                )]
            },
        );

        NavigationMenuLink::child(model, body)
            .label(title)
            .into_element(cx)
    }

    cx.column(
        ColumnProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(300.0)); // `w-[300px]`
                layout
            },
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |cx| {
            vec![
                link(
                    cx,
                    model.clone(),
                    "Components",
                    "Browse all components in the library.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Documentation",
                    "Learn how to use the library.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Blog",
                    "Read our latest blog posts.",
                    &title_style,
                    &desc_style,
                ),
            ]
        },
    )
}

fn shadcn_nav_menu_demo_components_panel_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    grid_width: Option<Px>,
    cols: u16,
) -> AnyElement {
    use fret_ui_shadcn::NavigationMenuLink;

    let title_style = shadcn_text_style(Px(14.0), Px(14.0), FontWeight::MEDIUM); // text-sm leading-none font-medium
    let desc_style = shadcn_text_style(Px(14.0), Px(19.25), FontWeight::NORMAL); // text-sm leading-snug

    fn link<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        model: Model<Option<Arc<str>>>,
        title: &'static str,
        desc: &'static str,
        title_style: &TextStyle,
        desc_style: &TextStyle,
    ) -> AnyElement {
        let title_style = title_style.clone();
        let desc_style = desc_style.clone();
        let title_arc: Arc<str> = Arc::from(title);
        let desc_arc: Arc<str> = Arc::from(desc);

        let body = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(Px(8.0)), // p-2
                ..Default::default()
            },
            move |cx| {
                let desc_style = desc_style.clone();
                vec![cx.column(
                    ColumnProps {
                        layout: LayoutStyle::default(),
                        gap: Px(4.0), // gap-1
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                    },
                    move |cx| {
                        let desc_box = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.height =
                                        Length::Px(Px(desc_style.line_height.unwrap().0 * 2.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            move |cx| vec![shadcn_text(cx, desc_arc.clone(), desc_style.clone())],
                        );

                        vec![
                            shadcn_text_line(cx, title_arc.clone(), title_style.clone()),
                            desc_box,
                        ]
                    },
                )]
            },
        );

        NavigationMenuLink::child(model, body)
            .label(title)
            .into_element(cx)
    }

    let gap = Px(8.0); // gap-2
    cx.grid(
        GridProps {
            layout: {
                let mut layout = LayoutStyle::default();
                if let Some(width) = grid_width {
                    layout.size.width = Length::Px(width);
                }
                layout
            },
            cols,
            rows: None,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        },
        move |cx| {
            vec![
                link(
                    cx,
                    model.clone(),
                    "Alert Dialog",
                    "A modal dialog that interrupts the user with important content and expects a response.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Hover Card",
                    "For sighted users to preview content available behind a link.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Progress",
                    "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Scroll Area",
                    "Visually or semantically separates content.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Tabs",
                    "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                    &title_style,
                    &desc_style,
                ),
                link(
                    cx,
                    model.clone(),
                    "Tooltip",
                    "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                    &title_style,
                    &desc_style,
                ),
            ]
        },
    )
}

fn shadcn_nav_menu_demo_components_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    // new-york-v4 `navigation-menu-demo`: `lg:w-[600px] md:grid-cols-2`.
    shadcn_nav_menu_demo_components_panel_impl(cx, model, Some(Px(600.0)), 2)
}

fn shadcn_nav_menu_demo_components_mobile_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
) -> AnyElement {
    // Match extracted shadcn web golden (mobile viewport content width).
    //
    // In upstream shadcn/ui, the viewport element width is driven by measured content, while the
    // content itself uses `p-2 pr-2.5` (8px left + 10px right). We approximate that by fixing the
    // grid width so that (grid + padding) matches the ~296px viewport rect from the web golden.
    shadcn_nav_menu_demo_components_panel_impl(cx, model, Some(Px(277.99)), 1)
}

fn build_context_menu_demo<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry,
        ContextMenuItem, ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItemSpec,
        ContextMenuShortcut,
    };

    ContextMenu::new(open)
        // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
        .min_width(Px(208.0))
        // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Right click here")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Back")
                            .inset(true)
                            .test_id("context-menu.back")
                            .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Forward")
                            .inset(true)
                            .disabled(true)
                            .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Reload")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("More Tools")
                            .inset(true)
                            .test_id("context-menu.more_tools")
                            .submenu(vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                    fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                )),
                            ]),
                    ),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_bookmarks,
                        "Show Bookmarks",
                    )),
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_full_urls,
                        "Show Full URLs",
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                    ContextMenuEntry::RadioGroup(
                        ContextMenuRadioGroup::new(radio_person)
                            .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                            .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                    ),
                ]
            },
        )
}

fn build_menubar_demo<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    view_bookmarks_bar: Model<bool>,
    view_full_urls: Model<bool>,
    profile_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab")
                    .test_id("menubar.file.new_tab")
                    .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(
                MenubarItem::new("Share")
                    .test_id("menubar.file.share")
                    .submenu(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ]),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]),
        MenubarMenu::new("Edit").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Search the web")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]),
        MenubarMenu::new("View").entries(vec![
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar,
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls,
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Toggle Fullscreen").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Hide Sidebar").inset(true)),
        ]),
        MenubarMenu::new("Profiles").entries(vec![
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile_value)
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("Luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]),
    ])
    .into_element(cx)
}

fn first_container_px_size(element: &AnyElement) -> Option<(f32, f32)> {
    fn visit(node: &AnyElement) -> Option<(f32, f32)> {
        if let fret_ui::element::ElementKind::Container(props) = &node.kind {
            if let (Length::Px(w), Length::Px(h)) =
                (props.layout.size.width, props.layout.size.height)
            {
                return Some((w.0, h.0));
            }
        }
        for child in &node.children {
            if let Some(found) = visit(child) {
                return Some(found);
            }
        }
        None
    }
    visit(element)
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_open_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.open.json"))
}

fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_open_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via (in-process server):\n  node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 {name} --modes=open --update\n\nOr (external server):\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web open golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

fn find_first<'a>(
    node: &'a WebNode,
    pred: &(impl Fn(&'a WebNode) -> bool + ?Sized),
) -> Option<&'a WebNode> {
    if pred(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first(child, pred) {
            return Some(found);
        }
    }
    None
}

fn web_find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
    })
}

fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

fn web_portal_node_by_data_slot<'a>(theme: &'a WebGoldenTheme, slot: &str) -> &'a WebNode {
    for portal in &theme.portals {
        if let Some(found) = web_find_by_data_slot(portal, slot) {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) = web_find_by_data_slot(wrapper, slot) {
            return found;
        }
    }
    panic!("missing web portal node with data-slot={slot}")
}

fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        for child in &node.children {
            if let Some(found) = find_attr_in_subtree(child, key) {
                return Some(found);
            }
        }
        None
    })
}

fn parse_side(value: &str) -> Option<Side> {
    Some(match value {
        "top" => Side::Top,
        "right" => Side::Right,
        "bottom" => Side::Bottom,
        "left" => Side::Left,
        _ => return None,
    })
}

fn parse_align(value: &str) -> Option<Align> {
    Some(match value {
        "start" => Align::Start,
        "center" => Align::Center,
        "end" => Align::End,
        _ => return None,
    })
}

fn parse_px(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    let value = value.strip_suffix("px").unwrap_or(value);
    value.parse::<f32>().ok()
}

fn rect_right(r: WebRect) -> f32 {
    r.x + r.w
}

fn web_rect_contains(outer: WebRect, inner: WebRect) -> bool {
    let eps = 0.01;
    inner.x + eps >= outer.x
        && inner.y + eps >= outer.y
        && rect_right(inner) <= rect_right(outer) + eps
        && rect_bottom(inner) <= rect_bottom(outer) + eps
}

fn web_unrotated_rect_for_rotated_square(node: &WebNode) -> WebRect {
    let size_w = web_css_px(node, "width").unwrap_or(node.rect.w);
    let size_h = web_css_px(node, "height").unwrap_or(node.rect.h);
    let dx = (node.rect.w - size_w) * 0.5;
    let dy = (node.rect.h - size_h) * 0.5;
    WebRect {
        x: node.rect.x + dx,
        y: node.rect.y + dy,
        w: size_w,
        h: size_h,
    }
}

fn fret_rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}

fn rect_bottom(r: WebRect) -> f32 {
    r.y + r.h
}

fn rect_center_x(r: WebRect) -> f32 {
    r.x + r.w * 0.5
}

fn rect_center_y(r: WebRect) -> f32 {
    r.y + r.h * 0.5
}

fn point_rect(p: WebPoint) -> WebRect {
    WebRect {
        x: p.x,
        y: p.y,
        w: 0.0,
        h: 0.0,
    }
}

fn web_css_px(node: &WebNode, key: &str) -> Option<f32> {
    node.computed_style.get(key).and_then(|v| parse_px(v))
}

fn web_portal_nodes_by_data_slot<'a>(theme: &'a WebGoldenTheme, slot: &str) -> Vec<&'a WebNode> {
    let mut nodes = Vec::new();
    let mut walk = |root: &'a WebNode| {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node
                .attrs
                .get("data-slot")
                .is_some_and(|s| s.as_str() == slot)
            {
                nodes.push(node);
            }
            for child in &node.children {
                stack.push(child);
            }
        }
    };

    for portal in &theme.portals {
        walk(portal);
    }
    for wrapper in &theme.portal_wrappers {
        walk(wrapper);
    }

    nodes
}

fn rect_main_gap(side: Side, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Bottom => content.y - rect_bottom(trigger),
        Side::Top => trigger.y - rect_bottom(content),
        Side::Right => content.x - rect_right(trigger),
        Side::Left => trigger.x - rect_right(content),
    }
}

fn rect_cross_delta(side: Side, align: Align, trigger: WebRect, content: WebRect) -> f32 {
    match side {
        Side::Top | Side::Bottom => match align {
            Align::Start => content.x - trigger.x,
            Align::Center => rect_center_x(content) - rect_center_x(trigger),
            Align::End => rect_right(content) - rect_right(trigger),
        },
        Side::Left | Side::Right => match align {
            Align::Start => content.y - trigger.y,
            Align::Center => rect_center_y(content) - rect_center_y(trigger),
            Align::End => rect_bottom(content) - rect_bottom(trigger),
        },
    }
}

fn infer_side(trigger: WebRect, content: WebRect) -> Side {
    let candidates = [
        (Side::Bottom, rect_main_gap(Side::Bottom, trigger, content)),
        (Side::Top, rect_main_gap(Side::Top, trigger, content)),
        (Side::Right, rect_main_gap(Side::Right, trigger, content)),
        (Side::Left, rect_main_gap(Side::Left, trigger, content)),
    ];
    candidates
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(side, _)| side)
        .unwrap_or(Side::Bottom)
}

fn infer_align(side: Side, trigger: WebRect, content: WebRect) -> Align {
    let candidates = [
        (
            Align::Start,
            rect_cross_delta(side, Align::Start, trigger, content).abs(),
        ),
        (
            Align::Center,
            rect_cross_delta(side, Align::Center, trigger, content).abs(),
        ),
        (
            Align::End,
            rect_cross_delta(side, Align::End, trigger, content).abs(),
        ),
    ];
    candidates
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(align, _)| align)
        .unwrap_or(Align::Start)
}

fn assert_close(label: &str, actual: f32, expected: f32, tol: f32) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected＞{expected} (㊣{tol}) got={actual} (忖={delta})"
    );
}

fn web_portal_slot_heights(theme: &WebGoldenTheme, slots: &[&str]) -> Vec<f32> {
    let mut heights = Vec::new();

    let mut walk = |root: &WebNode| {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if let Some(slot) = node.attrs.get("data-slot") {
                if slots.iter().any(|s| slot == s) {
                    heights.push(node.rect.h);
                }
            }
            for child in &node.children {
                stack.push(child);
            }
        }
    };

    for portal in &theme.portals {
        walk(portal);
    }
    for portal in &theme.portal_wrappers {
        walk(portal);
    }

    heights
}

fn web_portal_slot_rects(theme: &WebGoldenTheme, slot: &str) -> Vec<WebRect> {
    let mut rects = Vec::new();

    let mut walk = |root: &WebNode| {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.attrs.get("data-slot").is_some_and(|s| s == slot) {
                rects.push(node.rect);
            }
            for child in &node.children {
                stack.push(child);
            }
        }
    };

    for portal in &theme.portals {
        walk(portal);
    }
    for portal in &theme.portal_wrappers {
        walk(portal);
    }

    rects
}

fn web_portal_slot_rect_within(theme: &WebGoldenTheme, slot: &str, container: WebRect) -> WebRect {
    let eps = 1.0;
    let rects = web_portal_slot_rects(theme, slot);
    let within = |r: WebRect| {
        r.x + eps >= container.x
            && rect_right(r) <= rect_right(container) + eps
            && r.y + eps >= container.y
            && rect_bottom(r) <= rect_bottom(container) + eps
    };

    rects
        .into_iter()
        .find(|&r| within(r))
        .unwrap_or_else(|| panic!("web slot {slot} had no rect within {container:?}"))
}

fn web_portal_role_rects(theme: &WebGoldenTheme, role: &str) -> Vec<WebRect> {
    let mut rects = Vec::new();

    let mut walk = |root: &WebNode| {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.attrs.get("role").is_some_and(|s| s == role) {
                rects.push(node.rect);
            }
            for child in &node.children {
                stack.push(child);
            }
        }
    };

    for portal in &theme.portals {
        walk(portal);
    }
    for portal in &theme.portal_wrappers {
        walk(portal);
    }

    rects
}

fn web_portal_role_rect_largest_within(
    theme: &WebGoldenTheme,
    role: &str,
    container: WebRect,
) -> WebRect {
    let eps = 1.0;
    let within = |r: WebRect| {
        r.x + eps >= container.x
            && rect_right(r) <= rect_right(container) + eps
            && r.y + eps >= container.y
            && rect_bottom(r) <= rect_bottom(container) + eps
    };

    web_portal_role_rects(theme, role)
        .into_iter()
        .filter(|&r| within(r))
        .max_by(|a, b| (a.w * a.h).total_cmp(&(b.w * b.h)))
        .unwrap_or_else(|| panic!("web role={role} had no rect within {container:?}"))
}

fn fret_menu_item_heights_in_menus(snap: &fret_core::SemanticsSnapshot) -> Vec<f32> {
    let debug = std::env::var("FRET_DEBUG_MENU_SEMANTICS")
        .ok()
        .is_some_and(|v| v == "1");
    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    if menus.is_empty() {
        if debug {
            let mut roles: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            for n in &snap.nodes {
                *roles.entry(format!("{:?}", n.role)).or_insert(0) += 1;
            }
            eprintln!("fret_menu_item_heights_in_menus: no Menu nodes; roles={roles:?}");
        }
        return Vec::new();
    }

    let menu_contains = |node: &fret_core::SemanticsNode| {
        menus
            .iter()
            .any(|menu| fret_rect_contains(menu.bounds, node.bounds))
    };

    let items: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.role,
                SemanticsRole::MenuItem
                    | SemanticsRole::MenuItemCheckbox
                    | SemanticsRole::MenuItemRadio
            )
        })
        .collect();

    if debug {
        eprintln!(
            "fret_menu_item_heights_in_menus: menus={} items={}",
            menus.len(),
            items.len()
        );
        for (idx, menu) in menus.iter().take(2).enumerate() {
            eprintln!("  menu[{idx}] bounds={:?}", menu.bounds);
        }
        for (idx, item) in items.iter().take(6).enumerate() {
            eprintln!(
                "  item[{idx}] role={:?} label={:?} bounds={:?} in_menu={}",
                item.role,
                item.label.as_deref(),
                item.bounds,
                menu_contains(item)
            );
        }
    }

    items
        .into_iter()
        .filter(|n| menu_contains(n))
        .map(|n| n.bounds.size.height.0)
        .collect()
}

fn fret_largest_menu_height(snap: &fret_core::SemanticsSnapshot) -> Option<f32> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .max_by(|a, b| {
            let area_a = a.bounds.size.width.0 * a.bounds.size.height.0;
            let area_b = b.bounds.size.width.0 * b.bounds.size.height.0;
            area_a.total_cmp(&area_b)
        })
        .map(|n| n.bounds.size.height.0)
}

fn fret_menu_heights(snap: &fret_core::SemanticsSnapshot) -> Vec<f32> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .map(|n| n.bounds.size.height.0)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InsetTriplet {
    left: f32,
    top_to_first_item: f32,
    right: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InsetQuad {
    left: f32,
    top_to_first_option: f32,
    right: f32,
    bottom_from_last_option: f32,
}

fn web_menu_content_inset(menu: &WebNode) -> InsetTriplet {
    let is_menu_item_role = |node: &WebNode| {
        matches!(
            node.attrs.get("role").map(String::as_str),
            Some("menuitem") | Some("menuitemcheckbox") | Some("menuitemradio")
        )
    };

    let mut item_stack = vec![menu];
    let mut min_x = None::<f32>;
    let mut min_y = None::<f32>;
    let mut max_right = None::<f32>;

    while let Some(item_node) = item_stack.pop() {
        if is_menu_item_role(item_node) {
            let eps = 0.01;
            let menu_left = menu.rect.x;
            let menu_right = rect_right(menu.rect);
            let menu_top = menu.rect.y;
            let item_left = item_node.rect.x;
            let item_right = rect_right(item_node.rect);
            let item_top = item_node.rect.y;

            let within_panel = item_left + eps >= menu_left
                && item_right <= menu_right + eps
                && item_top + eps >= menu_top;
            if !within_panel {
                continue;
            }

            min_x = Some(min_x.unwrap_or(item_node.rect.x).min(item_node.rect.x));
            min_y = Some(min_y.unwrap_or(item_node.rect.y).min(item_node.rect.y));
            let r = rect_right(item_node.rect);
            max_right = Some(max_right.unwrap_or(r).max(r));
        }
        for child in &item_node.children {
            item_stack.push(child);
        }
    }

    let min_x = min_x.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));
    let min_y = min_y.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));
    let max_right = max_right.unwrap_or_else(|| panic!("web menu missing menuitem descendants"));

    InsetTriplet {
        left: min_x - menu.rect.x,
        top_to_first_item: min_y - menu.rect.y,
        right: rect_right(menu.rect) - max_right,
    }
}

fn web_first_visible_menu_item_label<'a>(menu: &WebNode, labels: &'a [&'a str]) -> Option<&'a str> {
    let is_menu_item_role = |node: &WebNode| {
        matches!(
            node.attrs.get("role").map(String::as_str),
            Some("menuitem") | Some("menuitemcheckbox") | Some("menuitemradio")
        )
    };

    let eps = 0.01;
    let menu_left = menu.rect.x;
    let menu_right = rect_right(menu.rect);
    let menu_top = menu.rect.y;

    let mut best: Option<(f32, &str)> = None;
    let mut stack = vec![menu];
    while let Some(node) = stack.pop() {
        if is_menu_item_role(node) {
            let item_left = node.rect.x;
            let item_right = rect_right(node.rect);
            let item_top = node.rect.y;

            let within_panel = item_left + eps >= menu_left
                && item_right <= menu_right + eps
                && item_top + eps >= menu_top;
            if within_panel {
                if let Some(text) = node.text.as_deref() {
                    if let Some(label) = labels.iter().copied().find(|l| text.starts_with(l)) {
                        let better = best.is_none_or(|(y, _)| item_top < y);
                        if better {
                            best = Some((item_top, label));
                        }
                    }
                }
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    best.map(|(_, label)| label)
}

fn fret_first_visible_menu_item_label<'a>(
    snap: &SemanticsSnapshot,
    menu_bounds: Rect,
    labels: &'a [&'a str],
) -> Option<&'a str> {
    let is_menu_item_role = |role: SemanticsRole| {
        matches!(
            role,
            SemanticsRole::MenuItem
                | SemanticsRole::MenuItemCheckbox
                | SemanticsRole::MenuItemRadio
        )
    };

    let eps = 0.01;
    let menu_left = menu_bounds.origin.x.0;
    let menu_right = menu_bounds.origin.x.0 + menu_bounds.size.width.0;
    let menu_top = menu_bounds.origin.y.0;

    let mut best: Option<(f32, &str)> = None;
    for node in &snap.nodes {
        if !is_menu_item_role(node.role) {
            continue;
        }

        let item_left = node.bounds.origin.x.0;
        let item_right = node.bounds.origin.x.0 + node.bounds.size.width.0;
        let item_top = node.bounds.origin.y.0;

        let within_panel = item_left + eps >= menu_left
            && item_right <= menu_right + eps
            && item_top + eps >= menu_top;
        if !within_panel {
            continue;
        }

        let Some(text) = node.label.as_deref() else {
            continue;
        };
        let Some(label) = labels.iter().copied().find(|l| text.starts_with(l)) else {
            continue;
        };

        let better = best.is_none_or(|(y, _)| item_top < y);
        if better {
            best = Some((item_top, label));
        }
    }

    best.map(|(_, label)| label)
}

fn fret_focused_label<'a>(snap: &'a SemanticsSnapshot) -> Option<&'a str> {
    let focused = snap.focus?;
    let node = snap.nodes.iter().find(|n| n.id == focused)?;
    let active = node
        .active_descendant
        .and_then(|id| snap.nodes.iter().find(|n| n.id == id))
        .unwrap_or(node);
    active.label.as_deref()
}

fn web_menu_content_insets_for_slots(theme: &WebGoldenTheme, slots: &[&str]) -> Vec<InsetTriplet> {
    slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(theme, slot))
        .map(web_menu_content_inset)
        .collect()
}

fn web_select_listbox<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    theme
        .portals
        .iter()
        .chain(theme.portal_wrappers.iter())
        .find(|n| n.attrs.get("role").is_some_and(|v| v == "listbox"))
        .or_else(|| {
            theme
                .portals
                .iter()
                .chain(theme.portal_wrappers.iter())
                .find_map(|portal| {
                    find_first(portal, &|n| {
                        n.attrs.get("role").is_some_and(|v| v == "listbox")
                    })
                })
        })
        .unwrap_or_else(|| panic!("missing web select listbox portal"))
}

fn web_select_combobox<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .unwrap_or_else(|| panic!("missing web select combobox"))
}

fn web_select_content_option_inset(listbox: &WebNode) -> InsetQuad {
    let mut option_stack = vec![listbox];
    let mut min_x = None::<f32>;
    let mut min_y = None::<f32>;
    let mut max_right = None::<f32>;
    let mut max_bottom = None::<f32>;

    while let Some(option_node) = option_stack.pop() {
        if option_node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option")
        {
            let eps = 0.01;
            let panel_left = listbox.rect.x;
            let panel_right = rect_right(listbox.rect);
            let panel_top = listbox.rect.y;
            let panel_bottom = rect_bottom(listbox.rect);
            let option_left = option_node.rect.x;
            let option_right = rect_right(option_node.rect);
            let option_top = option_node.rect.y;
            let option_bottom = rect_bottom(option_node.rect);

            let within_panel = option_left + eps >= panel_left
                && option_right <= panel_right + eps
                && option_top + eps >= panel_top
                && option_bottom <= panel_bottom + eps;
            if !within_panel {
                continue;
            }

            min_x = Some(min_x.unwrap_or(option_left).min(option_left));
            min_y = Some(min_y.unwrap_or(option_top).min(option_top));
            max_right = Some(max_right.unwrap_or(option_right).max(option_right));
            max_bottom = Some(max_bottom.unwrap_or(option_bottom).max(option_bottom));
        }
        for child in &option_node.children {
            option_stack.push(child);
        }
    }

    let min_x = min_x.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let min_y = min_y.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let max_right =
        max_right.unwrap_or_else(|| panic!("web select listbox missing option descendants"));
    let max_bottom =
        max_bottom.unwrap_or_else(|| panic!("web select listbox missing option descendants"));

    InsetQuad {
        left: min_x - listbox.rect.x,
        top_to_first_option: min_y - listbox.rect.y,
        right: rect_right(listbox.rect) - max_right,
        bottom_from_last_option: rect_bottom(listbox.rect) - max_bottom,
    }
}

fn web_select_listbox_option_heights(listbox: &WebNode) -> Vec<f32> {
    let eps = 0.5;
    let panel_left = listbox.rect.x;
    let panel_right = rect_right(listbox.rect);
    let panel_top = listbox.rect.y;
    let panel_bottom = rect_bottom(listbox.rect);

    let mut heights = Vec::new();
    let mut stack = vec![listbox];
    while let Some(node) = stack.pop() {
        if node
            .attrs
            .get("role")
            .is_some_and(|v| v.as_str() == "option")
        {
            let option_left = node.rect.x;
            let option_right = rect_right(node.rect);
            let option_top = node.rect.y;
            let option_bottom = rect_bottom(node.rect);

            let within_panel = option_left + eps >= panel_left
                && option_right <= panel_right + eps
                && option_top + eps >= panel_top
                && option_bottom <= panel_bottom + eps;
            if within_panel {
                heights.push(node.rect.h);
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }

    heights
}

fn fret_menu_content_insets(snap: &fret_core::SemanticsSnapshot) -> Vec<InsetTriplet> {
    let debug = std::env::var("FRET_DEBUG_MENU_SEMANTICS")
        .ok()
        .is_some_and(|v| v == "1");
    let is_menu_item = |n: &fret_core::SemanticsNode| {
        matches!(
            n.role,
            SemanticsRole::MenuItem
                | SemanticsRole::MenuItemCheckbox
                | SemanticsRole::MenuItemRadio
        )
    };

    let mut insets = Vec::new();

    for menu in snap.nodes.iter().filter(|n| n.role == SemanticsRole::Menu) {
        let items: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| is_menu_item(n))
            .filter(|n| fret_rect_contains(menu.bounds, n.bounds))
            .collect();

        if items.is_empty() {
            continue;
        }

        if debug {
            let mut items_sorted = items.clone();
            items_sorted.sort_by(|a, b| {
                a.bounds
                    .origin
                    .y
                    .0
                    .total_cmp(&b.bounds.origin.y.0)
                    .then_with(|| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
            });
            eprintln!(
                "fret_menu_content_insets: menu bounds={:?} items={}",
                menu.bounds,
                items_sorted.len()
            );
            for (ix, item) in items_sorted.iter().take(12).enumerate() {
                eprintln!(
                    "  item[{ix}] role={:?} label={:?} bounds={:?}",
                    item.role, item.label, item.bounds
                );
            }
        }

        let mut min_x = items[0].bounds.origin.x.0;
        let mut min_y = items[0].bounds.origin.y.0;
        let mut max_right = items[0].bounds.origin.x.0 + items[0].bounds.size.width.0;
        for item in items.iter().skip(1) {
            min_x = min_x.min(item.bounds.origin.x.0);
            min_y = min_y.min(item.bounds.origin.y.0);
            max_right = max_right.max(item.bounds.origin.x.0 + item.bounds.size.width.0);
        }

        let menu_right = menu.bounds.origin.x.0 + menu.bounds.size.width.0;
        insets.push(InsetTriplet {
            left: min_x - menu.bounds.origin.x.0,
            top_to_first_item: min_y - menu.bounds.origin.y.0,
            right: menu_right - max_right,
        });
    }

    insets
}

fn assert_sorted_insets_match(web_name: &str, actual: &[InsetTriplet], expected: &[InsetTriplet]) {
    if expected.is_empty() {
        panic!("missing web menu insets for {web_name}");
    }
    if actual.is_empty() {
        panic!("missing fret menu insets for {web_name}");
    }
    assert!(
        actual.len() == expected.len(),
        "{web_name} expected {} menus, got {}",
        expected.len(),
        actual.len()
    );

    let mut expected_sorted = expected.to_vec();
    let mut actual_sorted = actual.to_vec();
    let sort_key = |v: &InsetTriplet| (round_i32(v.top_to_first_item), round_i32(v.left));
    expected_sorted.sort_by_key(sort_key);
    actual_sorted.sort_by_key(sort_key);

    for (i, (a, e)) in actual_sorted.iter().zip(expected_sorted.iter()).enumerate() {
        assert_close(
            &format!("{web_name} menu[{i}] left_inset"),
            a.left,
            e.left,
            1.0,
        );
        assert_close(
            &format!("{web_name} menu[{i}] top_to_first_item"),
            a.top_to_first_item,
            e.top_to_first_item,
            1.5,
        );
        assert_close(
            &format!("{web_name} menu[{i}] right_inset"),
            a.right,
            e.right,
            1.0,
        );
    }
}

fn fret_select_content_option_inset(snap: &fret_core::SemanticsSnapshot) -> InsetQuad {
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox"));

    let options: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .collect();

    if options.is_empty() {
        panic!("missing fret listbox options");
    }

    let mut min_x = options[0].bounds.origin.x.0;
    let mut min_y = options[0].bounds.origin.y.0;
    let mut max_right = options[0].bounds.origin.x.0 + options[0].bounds.size.width.0;
    let mut max_bottom = options[0].bounds.origin.y.0 + options[0].bounds.size.height.0;
    for option in options.iter().skip(1) {
        min_x = min_x.min(option.bounds.origin.x.0);
        min_y = min_y.min(option.bounds.origin.y.0);
        max_right = max_right.max(option.bounds.origin.x.0 + option.bounds.size.width.0);
        max_bottom = max_bottom.max(option.bounds.origin.y.0 + option.bounds.size.height.0);
    }

    let panel_right = listbox.bounds.origin.x.0 + listbox.bounds.size.width.0;
    let panel_bottom = listbox.bounds.origin.y.0 + listbox.bounds.size.height.0;

    InsetQuad {
        left: min_x - listbox.bounds.origin.x.0,
        top_to_first_option: min_y - listbox.bounds.origin.y.0,
        right: panel_right - max_right,
        bottom_from_last_option: panel_bottom - max_bottom,
    }
}

fn fret_listbox_option_heights_in_listbox(snap: &fret_core::SemanticsSnapshot) -> Vec<f32> {
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox"));

    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .map(|n| n.bounds.size.height.0)
        .collect()
}

fn fret_listbox_height(snap: &fret_core::SemanticsSnapshot) -> f32 {
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox"))
        .bounds
        .size
        .height
        .0
}

fn fret_node_heights_by_test_id(snap: &fret_core::SemanticsSnapshot, test_id: &str) -> Vec<f32> {
    snap.nodes
        .iter()
        .filter(|n| n.test_id.as_deref() == Some(test_id))
        .map(|n| n.bounds.size.height.0)
        .collect()
}

fn fret_nodes_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Vec<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .filter(|n| n.test_id.as_deref() == Some(test_id))
        .collect()
}

fn assert_select_inset_match(web_name: &str, actual: InsetQuad, expected: InsetQuad) {
    assert_close(
        &format!("{web_name} listbox left_inset"),
        actual.left,
        expected.left,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox top_to_first_option"),
        actual.top_to_first_option,
        expected.top_to_first_option,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox right_inset"),
        actual.right,
        expected.right,
        1.0,
    );
    assert_close(
        &format!("{web_name} listbox bottom_from_last_option"),
        actual.bottom_from_last_option,
        expected.bottom_from_last_option,
        1.0,
    );
}

fn round_i32(v: f32) -> i32 {
    v.round() as i32
}

fn assert_menu_item_row_height_matches(
    web_name: &str,
    expected_h: f32,
    actual_hs: &[f32],
    tol: f32,
) {
    if actual_hs.is_empty() {
        panic!("missing fret menu items for {web_name}");
    }

    let unique: std::collections::BTreeSet<i32> =
        actual_hs.iter().copied().map(round_i32).collect();
    assert!(
        unique.len() == 1,
        "{web_name} expected uniform menu item row height; got {unique:?}"
    );
    assert_close(
        &format!("{web_name} menu_item_row_h"),
        unique.iter().next().copied().unwrap_or_default() as f32,
        expected_h,
        tol,
    );
}

#[derive(Default)]
struct StyleAwareServices;

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style.clone()),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base.clone()),
            _ => (input.text(), fret_core::TextStyle::default()),
        };
        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        fn estimate_width_px(text: &str, font_size: f32, weight: FontWeight) -> Px {
            let mut units = 0.0f32;
            for ch in text.chars() {
                units += match ch {
                    // Most overlay placement goldens use Geist @ 14px. We approximate its advance
                    // widths with a small heuristic table so both short labels ("Open popover") and
                    // long mixed-case strings ("Australian Western Standard Time (AWST)") land close
                    // to the web snapshots.
                    ' ' => 0.28,
                    '(' | ')' => 0.28,
                    // Narrow glyphs.
                    'i' | 'l' | 'I' | 't' | 'f' | 'j' | 'r' => 0.32,
                    // Wide glyphs.
                    'm' | 'w' | 'M' | 'W' => 0.75,
                    // Round glyphs.
                    'o' | 'O' | 'p' | 'P' => 0.62,
                    // Uppercase baseline.
                    'A'..='Z' => 0.62,
                    // Default lowercase baseline.
                    'a'..='z' => 0.56,
                    // Everything else (digits/punctuation) uses a neutral baseline.
                    _ => 0.56,
                };
            }
            let weight_mul = match weight.0 {
                0..=450 => 1.0,
                451..=550 => 1.024,
                551..=650 => 1.04,
                651..=750 => 1.055,
                _ => 1.065,
            };
            Px((units * font_size * weight_mul).max(1.0))
        }

        let est_w = estimate_width_px(text, style.size.0, style.weight);

        let max_w = constraints.max_width.unwrap_or(est_w);
        let (lines, w) = match constraints.wrap {
            fret_core::TextWrap::Word if max_w.0.is_finite() && max_w.0 > 0.0 => {
                let lines = (est_w.0 / max_w.0).ceil().max(1.0) as u32;
                (lines, Px(est_w.0.min(max_w.0)))
            }
            _ => (1, est_w),
        };

        let h = Px(line_height.0 * lines as f32);

        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: CoreSize::new(w, h),
                baseline: Px(h.0 * 0.8),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for StyleAwareServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for StyleAwareServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

fn setup_app_with_shadcn_theme(app: &mut App) {
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

fn deliver_all_timers_from_effects(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) -> usize {
    let effects = app.flush_effects();
    let mut timers = Vec::new();
    for effect in effects {
        match effect {
            Effect::SetTimer { token, .. } => timers.push(token),
            other => app.push_effect(other),
        }
    }

    let count = timers.len();
    for token in timers {
        ui.dispatch_event(app, services, &Event::Timer { token });
    }

    count
}

fn render_frame<I, F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: FrameId,
    request_semantics: bool,
    render: F,
) where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    app.set_frame_id(frame_id);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "web-vs-fret-overlay-placement",
        render,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_overlay_placement_matches(
    web_name: &str,
    web_portal_role: Option<&str>,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    fret_trigger_role: SemanticsRole,
    fret_trigger_label: Option<&str>,
    fret_portal_role: SemanticsRole,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = if web_portal_role == Some("listbox") {
        fn find_in_theme<'a>(
            t: &'a WebGoldenTheme,
            pred: &dyn Fn(&WebNode) -> bool,
        ) -> Option<&'a WebNode> {
            find_first(&t.root, pred)
                .or_else(|| t.portal_wrappers.iter().find_map(|n| find_first(n, pred)))
                .or_else(|| t.portals.iter().find_map(|n| find_first(n, pred)))
        }

        // Some pages (notably nested demos like `date-picker-with-presets.select-open.open`) have
        // multiple openable buttons, but only one `role=combobox` anchor that controls the listbox
        // portal. Prefer that anchor for placement math.
        let is_open_combobox = |n: &WebNode| {
            n.tag == "button"
                && n.attrs.get("role").is_some_and(|v| v == "combobox")
                && (n
                    .attrs
                    .get("data-state")
                    .is_some_and(|v| v.as_str() == "open")
                    || n.attrs
                        .get("aria-expanded")
                        .is_some_and(|v| v.as_str() == "true"))
        };

        find_in_theme(&web.themes["light"], &is_open_combobox)
            .or_else(|| find_in_theme(&web.themes["dark"], &is_open_combobox))
            .or_else(|| {
                find_in_theme(&web.themes["light"], &|n| {
                    n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
                })
            })
            .or_else(|| {
                find_in_theme(&web.themes["dark"], &|n| {
                    n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
                })
            })
            .expect("web trigger (combobox)")
    } else {
        match web_name {
            "context-menu-demo" => find_first(&web.themes["light"].root, &|n| {
                n.text
                    .as_deref()
                    .is_some_and(|t| t.contains("Right click here"))
            })
            .or_else(|| {
                find_first(&web.themes["dark"].root, &|n| {
                    n.text
                        .as_deref()
                        .is_some_and(|t| t.contains("Right click here"))
                })
            })
            .expect("web trigger (context menu)"),
            _ => {
                let is_open_trigger = |n: &WebNode| {
                    n.tag == "button"
                        && (n
                            .attrs
                            .get("data-state")
                            .is_some_and(|v| v.as_str() == "open")
                            || n.attrs
                                .get("aria-expanded")
                                .is_some_and(|v| v.as_str() == "true"))
                };

                find_first(&web.themes["light"].root, &is_open_trigger)
                    .or_else(|| find_first(&web.themes["dark"].root, &is_open_trigger))
                    .or_else(|| find_first(&web.themes["light"].root, &|n| n.tag == "button"))
                    .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
                    .expect("web trigger (button)")
            }
        }
    };

    let web_portal_index = if let Some(web_portal_role) = web_portal_role {
        theme
            .portals
            .iter()
            .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
            .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"))
    } else {
        if theme.portals.is_empty() {
            panic!("missing web portals for {web_name}");
        }
        0
    };
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = if web_portal_role == Some("listbox") {
        // For Radix Select-style listboxes, the `data-align` attribute lives on the inner content
        // element, while our placement comparisons target the positioned wrapper rect (the popper
        // content wrapper). Infer the effective alignment from geometry to avoid false mismatches
        // under collision shifting/clamping.
        infer_align(web_side, web_trigger.rect, web_portal.rect)
    } else {
        find_attr_in_subtree(web_portal_leaf, "data-align")
            .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
            .and_then(parse_align)
            .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect))
    };

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let snap_closed = ui
        .semantics_snapshot()
        .expect("semantics snapshot (closed)")
        .clone();

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for step in 0..=settle_frames {
        let frame = 2 + step;
        let request_semantics = step == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap_open = ui
        .semantics_snapshot()
        .expect("semantics snapshot (open)")
        .clone();

    // Most overlays have a trigger that exists even when the portal is closed. Nested overlay
    // scenarios (e.g. a Select inside an open Popover) only have their trigger once the parent
    // overlay is visible. Fall back to the open snapshot in that case.
    fn find_trigger_in<'a>(
        snap: &'a SemanticsSnapshot,
        role: SemanticsRole,
        label: Option<&str>,
    ) -> Option<&'a fret_core::SemanticsNode> {
        snap.nodes.iter().find(|n| {
            if n.role != role {
                return false;
            }
            if let Some(label) = label {
                return n.label.as_deref() == Some(label);
            }
            true
        })
    }

    let trigger = find_trigger_in(&snap_closed, fret_trigger_role, fret_trigger_label)
        .or_else(|| find_trigger_in(&snap_open, fret_trigger_role, fret_trigger_label))
        .unwrap_or_else(|| {
            use std::collections::BTreeMap;

            fn role_name(role: SemanticsRole) -> &'static str {
                match role {
                    SemanticsRole::Generic => "Generic",
                    SemanticsRole::Window => "Window",
                    SemanticsRole::Panel => "Panel",
                    SemanticsRole::Group => "Group",
                    SemanticsRole::Dialog => "Dialog",
                    SemanticsRole::AlertDialog => "AlertDialog",
                    SemanticsRole::Alert => "Alert",
                    SemanticsRole::Button => "Button",
                    SemanticsRole::Checkbox => "Checkbox",
                    SemanticsRole::Switch => "Switch",
                    SemanticsRole::Slider => "Slider",
                    SemanticsRole::ComboBox => "ComboBox",
                    SemanticsRole::RadioGroup => "RadioGroup",
                    SemanticsRole::RadioButton => "RadioButton",
                    SemanticsRole::TabList => "TabList",
                    SemanticsRole::Tab => "Tab",
                    SemanticsRole::TabPanel => "TabPanel",
                    SemanticsRole::MenuBar => "MenuBar",
                    SemanticsRole::Menu => "Menu",
                    SemanticsRole::MenuItem => "MenuItem",
                    SemanticsRole::MenuItemCheckbox => "MenuItemCheckbox",
                    SemanticsRole::MenuItemRadio => "MenuItemRadio",
                    SemanticsRole::Tooltip => "Tooltip",
                    SemanticsRole::Text => "Text",
                    SemanticsRole::TextField => "TextField",
                    SemanticsRole::List => "List",
                    SemanticsRole::ListItem => "ListItem",
                    SemanticsRole::ListBox => "ListBox",
                    SemanticsRole::ListBoxOption => "ListBoxOption",
                    SemanticsRole::TreeItem => "TreeItem",
                    SemanticsRole::Viewport => "Viewport",
                    _ => "Unknown",
                }
            }

            let mut role_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
            for n in &snap_open.nodes {
                *role_counts.entry(role_name(n.role)).or_insert(0) += 1;
            }

            let candidates: Vec<(Option<&str>, Option<&str>)> = snap_open
                .nodes
                .iter()
                .filter(|n| n.role == fret_trigger_role)
                .map(|n| (n.label.as_deref(), n.test_id.as_deref()))
                .collect();

            let roots: Vec<_> = snap_open.roots.iter().map(|r| r.root).collect();

            let sample: Vec<(SemanticsRole, Option<&str>, Option<&str>)> = snap_open
                .nodes
                .iter()
                .take(30)
                .map(|n| (n.role, n.label.as_deref(), n.test_id.as_deref()))
                .collect();

            panic!(
                "missing fret trigger role={fret_trigger_role:?} label={fret_trigger_label:?}; roots={roots:?}; role_counts={role_counts:?}; candidates(label,test_id)={candidates:?}; sample(role,label,test_id)={sample:?}"
            );
        });

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap_open
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;

            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?}"));

    let trigger = if fret_portal_role == SemanticsRole::ListBox
        && fret_trigger_role == SemanticsRole::ComboBox
    {
        snap_open
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.controls.contains(&portal.id))
            .unwrap_or(trigger)
    } else {
        trigger
    };

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    if debug {
        let candidates: Vec<_> = snap_open
            .nodes
            .iter()
            .filter(|n| n.role == fret_portal_role)
            .collect();
        eprintln!(
            "{web_name} fret portal candidates role={fret_portal_role:?}: {}",
            candidates.len()
        );
        for (idx, n) in candidates.iter().enumerate().take(6) {
            eprintln!(
                "  [{idx}] bounds={:?} label={:?} flags={:?}",
                n.bounds, n.label, n.flags
            );
        }
        if fret_portal_role == SemanticsRole::ListBox {
            let comboboxes: Vec<_> = snap_open
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::ComboBox)
                .collect();
            eprintln!("{web_name} fret combobox candidates: {}", comboboxes.len());
            for (idx, n) in comboboxes.iter().enumerate().take(10) {
                let controls_listbox = n.controls.contains(&portal.id);
                eprintln!(
                    "  [{idx}] id={:?} bounds={:?} test_id={:?} label={:?} controls.len={} controls_listbox={controls_listbox}",
                    n.id,
                    n.bounds,
                    n.test_id,
                    n.label,
                    n.controls.len(),
                );
            }
        }
        eprintln!(
            "{web_name} web side={web_side:?} align={web_align:?}\n  web trigger={:?}\n  web portal={:?}\n  fret trigger={:?}\n  fret portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, fret_portal
        );
    }

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );

    if matches!(
        fret_portal_role,
        SemanticsRole::Menu | SemanticsRole::ListBox
    ) {
        assert_close(
            &format!("{web_name} portal_w"),
            fret_portal.w,
            expected_portal_w,
            2.0,
        );
        assert_close(
            &format!("{web_name} portal_h"),
            fret_portal.h,
            expected_portal_h,
            2.0,
        );
    }
}

fn assert_centered_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_center_x = rect_center_x(web_portal.rect);
    let expected_center_y = rect_center_y(web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for step in 0..=settle_frames {
        let frame = 2 + step;
        let request_semantics = step == settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let center = (rect_center_x(r) - expected_center_x).abs()
                    + (rect_center_y(r) - expected_center_y).abs();
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                center + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    if debug {
        eprintln!(
            "{web_name} web portal={:?} expected_center=({}, {})",
            web_portal.rect, expected_center_x, expected_center_y
        );
        eprintln!("{web_name} selected fret portal={:?}", fret_portal);
    }

    assert_close(
        &format!("{web_name} center_x"),
        rect_center_x(fret_portal),
        expected_center_x,
        2.0,
    );
    assert_close(
        &format!("{web_name} width"),
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        &format!("{web_name} center_y"),
        rect_center_y(fret_portal),
        expected_center_y,
        2.0,
    );
}

fn assert_viewport_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let viewport_w = theme.viewport.map(|v| v.w).unwrap_or(1440.0);
    let viewport_h = theme.viewport.map(|v| v.h).unwrap_or(900.0);

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role} for {web_name}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let expected_left = web_portal.rect.x;
    let expected_top = web_portal.rect.y;
    let expected_right = viewport_w - rect_right(web_portal.rect);
    let expected_bottom = viewport_h - rect_bottom(web_portal.rect);

    let anchor_tol = 2.0;
    let anchor_left = expected_left.abs() <= anchor_tol;
    let anchor_top = expected_top.abs() <= anchor_tol;
    let anchor_right = expected_right.abs() <= anchor_tol;
    let anchor_bottom = expected_bottom.abs() <= anchor_tol;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2;
    for frame_id in 2..=(2 + settle_frames) {
        let request_semantics = frame_id == 2 + settle_frames;
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_id),
            request_semantics,
            |cx| {
                let content = build_frame(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let left = r.x;
                let top = r.y;
                let right = viewport_w - rect_right(r);
                let bottom = viewport_h - rect_bottom(r);

                let mut score = 0.0;
                if anchor_left {
                    score += (left - expected_left).abs();
                }
                if anchor_top {
                    score += (top - expected_top).abs();
                }
                if anchor_right {
                    score += (right - expected_right).abs();
                }
                if anchor_bottom {
                    score += (bottom - expected_bottom).abs();
                }

                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                score + 0.02 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?} for {web_name}"));

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_left = fret_portal.x;
    let actual_top = fret_portal.y;
    let actual_right = viewport_w - rect_right(fret_portal);
    let actual_bottom = viewport_h - rect_bottom(fret_portal);

    if debug {
        eprintln!(
            "{web_name} anchors: left={anchor_left} top={anchor_top} right={anchor_right} bottom={anchor_bottom}"
        );
        eprintln!(
            "{web_name} web portal={:?} expected_insets=(l={expected_left}, t={expected_top}, r={expected_right}, b={expected_bottom})",
            web_portal.rect
        );
        eprintln!(
            "{web_name} fret portal={:?} actual_insets=(l={actual_left}, t={actual_top}, r={actual_right}, b={actual_bottom})",
            fret_portal
        );
    }

    if anchor_left {
        assert_close(
            &format!("{web_name} inset_left"),
            actual_left,
            expected_left,
            2.0,
        );
    }
    if anchor_top {
        assert_close(
            &format!("{web_name} inset_top"),
            actual_top,
            expected_top,
            2.0,
        );
    }
    if anchor_right {
        assert_close(
            &format!("{web_name} inset_right"),
            actual_right,
            expected_right,
            2.0,
        );
    }
    if anchor_bottom {
        assert_close(
            &format!("{web_name} inset_bottom"),
            actual_bottom,
            expected_bottom,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_popover_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "popover-demo",
        Some("dialog"),
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    fret_ui_shadcn::Button::new("Open popover")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    let content = fret_ui_shadcn::PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
                        )
                        .into_element(cx);
                    if std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
                        .ok()
                        .is_some_and(|v| v == "1")
                    {
                        eprintln!(
                            "popover-demo content container px size={:?}",
                            first_container_px_size(&content)
                        );
                    }
                    content
                },
            )
        },
        SemanticsRole::Button,
        Some("Open popover"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_popover_demo_overlay_placement_matches_tiny_viewport() {
    assert_overlay_placement_matches(
        "popover-demo.vp1440x240",
        Some("dialog"),
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    fret_ui_shadcn::Button::new("Open popover")
                        .variant(fret_ui_shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    fret_ui_shadcn::PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
                        )
                        .into_element(cx)
                },
            )
        },
        SemanticsRole::Button,
        Some("Open popover"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_date_picker_demo_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "date-picker-demo",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use time::Month;

            let month: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2026, Month::January));
            let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::DatePicker::new(open.clone(), month, selected)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
                .into_element(cx)
        },
        SemanticsRole::Button,
        Some("Pick a date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_date_picker_with_presets_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "date-picker-with-presets",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use time::Month;

            let month: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2026, Month::January));
            let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::DatePickerWithPresets::new(open.clone(), month, selected)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
                .into_element(cx)
        },
        SemanticsRole::Button,
        Some("Pick a date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_date_picker_with_presets_select_listbox_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "date-picker-with-presets.select-open",
        Some("listbox"),
        |cx, open| {
            use fret_ui_kit::{ChromeRefinement, LengthRefinement, MetricRef};
            use fret_ui_shadcn::select::SelectPosition;

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);

            fret_ui_shadcn::Popover::new(open.clone())
                .align(fret_ui_shadcn::PopoverAlign::Start)
                .side(fret_ui_shadcn::PopoverSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        fret_ui_shadcn::Button::new("Pick a date")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))),
                            )
                            .into_element(cx)
                    },
                    move |cx| {
                        let select = fret_ui_shadcn::Select::new(value.clone(), open.clone())
                            .placeholder("Select")
                            .position(SelectPosition::Popper)
                            .items([
                                fret_ui_shadcn::SelectItem::new("0", "Today"),
                                fret_ui_shadcn::SelectItem::new("1", "Tomorrow"),
                                fret_ui_shadcn::SelectItem::new("3", "In 3 days"),
                                fret_ui_shadcn::SelectItem::new("7", "In a week"),
                            ])
                            .into_element(cx);

                        let body = stack::vstack(
                            cx,
                            stack::VStackProps::default().gap(Space::N2).items_stretch(),
                            move |_cx| vec![select],
                        );

                        fret_ui_shadcn::PopoverContent::new([body])
                            .refine_style(ChromeRefinement::default().p(Space::N2))
                            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                            .into_element(cx)
                    },
                )
        },
        SemanticsRole::ComboBox,
        None,
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_date_picker_with_range_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "date-picker-with-range",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use time::{Date, Month};

            let month: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2022, Month::January));
            let selected: Model<DateRangeSelection> =
                cx.app.models_mut().insert(DateRangeSelection {
                    from: Some(
                        Date::from_calendar_date(2022, Month::January, 20).expect("from date"),
                    ),
                    to: Some(Date::from_calendar_date(2022, Month::February, 9).expect("to date")),
                });

            fret_ui_shadcn::DateRangePicker::new(open.clone(), month, selected)
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(300.0))))
                .into_element(cx)
        },
        SemanticsRole::Button,
        Some("Jan 20, 2022 - Feb 09, 2022"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_22_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-22",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::Month;

            let trigger = Button::new("Select date")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(192.0)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Date of birth").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_23_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-23",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::Month;

            let trigger = Button::new("Select date")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(224.0)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_24_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-24",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::Month;

            let trigger = Button::new("Select date")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(128.0)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Date").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_25_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-25",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::Month;

            let trigger = Button::new("Select date")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(1440.0)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Date").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_26_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-26",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("Jun 01, 2025")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(154.66667)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Check-in").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                            Date::from_calendar_date(2025, Month::June, 1).expect("valid date"),
                        ));
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Jun 01, 2025"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_27_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-27",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("6/5/2025 - 6/20/2025").variant(ButtonVariant::Outline);

            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::End)
                .window_margin(Px(0.0))
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<DateRangeSelection> =
                            cx.app.models_mut().insert(DateRangeSelection {
                                from: Some(
                                    Date::from_calendar_date(2025, Month::June, 5)
                                        .expect("valid date"),
                                ),
                                to: Some(
                                    Date::from_calendar_date(2025, Month::June, 20)
                                        .expect("valid date"),
                                ),
                            });
                        let calendar =
                            fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);
                        fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
                    },
                );

            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_end(),
                move |_cx| vec![popover],
            )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_28_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-28",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("Select date")
                .variant(ButtonVariant::Ghost)
                .refine_layout(LayoutRefinement::default().mr(Space::N2));
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::End)
                .align_offset(Px(-8.0))
                .side_offset(Px(10.0))
                .window_margin(Px(0.0))
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                            Date::from_calendar_date(2025, Month::June, 1).expect("valid date"),
                        ));
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
                        fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
                    },
                );

            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_end(),
                move |_cx| vec![popover],
            )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_29_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-29",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("Select date").variant(ButtonVariant::Ghost);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::End)
                .window_margin(Px(0.0))
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                            Date::from_calendar_date(2025, Month::June, 3).expect("valid date"),
                        ));
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
                        fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
                    },
                );

            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_end(),
                move |_cx| vec![popover],
            )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_calendar_30_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-30",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("Jun 4 - 10, 2025")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(224.0)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
            let popover = fret_ui_shadcn::Popover::new(open.clone())
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| trigger.into_element(cx),
                    |cx| {
                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2025, Month::June));
                        let selected: Model<DateRangeSelection> =
                            cx.app.models_mut().insert(DateRangeSelection {
                                from: Some(
                                    Date::from_calendar_date(2025, Month::June, 4)
                                        .expect("valid date"),
                                ),
                                to: Some(
                                    Date::from_calendar_date(2025, Month::June, 10)
                                        .expect("valid date"),
                                ),
                            });
                        let calendar =
                            fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Jun 4 - 10, 2025"),
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_dialog_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-dialog",
        Some("menu"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
            };

            use fret_ui_kit::declarative::icon as decl_icon;

            let button = Button::new("")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(32.0)))
                        .h_px(MetricRef::Px(Px(32.0))),
                )
                .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-dialog: `DropdownMenuContent className="w-40"`.
                .min_width(Px(160.0))
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| button.into_element(cx),
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("File Actions")),
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("New File...")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Share...")),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Download").disabled(true),
                                ),
                            ])),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_item_height_matches() {
    let web_name = "dropdown-menu-dialog";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_content_insets_match() {
    let web_name = "dropdown-menu-dialog";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
}

fn build_dropdown_menu_dialog_open_snapshot(
    theme: &WebGoldenTheme,
) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_kit::declarative::icon as decl_icon;
        use fret_ui_kit::{LayoutRefinement, MetricRef};
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
        };

        let button = Button::new("")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(32.0)))
                    .h_px(MetricRef::Px(Px(32.0))),
            )
            .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

        DropdownMenu::new(open.clone())
            .min_width(Px(160.0))
            .align(DropdownMenuAlign::End)
            .into_element(
                cx,
                |cx| button.clone().into_element(cx),
                |_cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("File Actions")),
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("New File...")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Share...")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Download").disabled(true),
                            ),
                        ])),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    ui.semantics_snapshot().expect("semantics snapshot").clone()
}

#[test]
fn web_vs_fret_item_dropdown_overlay_placement_matches() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let is_open_trigger = |n: &WebNode| {
        n.tag == "button"
            && (n
                .attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == "open")
                || n.attrs
                    .get("aria-expanded")
                    .is_some_and(|v| v.as_str() == "true"))
    };
    let web_trigger = find_first(&web.themes["light"].root, &is_open_trigger)
        .or_else(|| find_first(&web.themes["dark"].root, &is_open_trigger))
        .expect("web trigger (button)");
    let trigger_rect = web_trigger.rect;

    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let item_h = expected_item_h.round();

    assert_overlay_placement_matches(
        web_name,
        Some("menu"),
        move |cx, open| {
            use fret_ui::element::LayoutStyle;
            use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
            use fret_ui_shadcn::{
                Avatar, AvatarFallback, Button, ButtonSize, ButtonVariant, DropdownMenu,
                DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, Item, ItemContent,
                ItemDescription, ItemMedia, ItemSize, ItemTitle,
            };

            use fret_ui_kit::declarative::icon as decl_icon;

            let button = Button::new("Select")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(trigger_rect.w)))
                        .h_px(MetricRef::Px(Px(trigger_rect.h))),
                )
                .children([decl_icon::icon(cx, fret_icons::ids::ui::CHEVRON_DOWN)]);

            let people = vec![
                ("shadcn", "shadcn@vercel.com"),
                ("maxleiter", "maxleiter@vercel.com"),
                ("evilrabbit", "evilrabbit@vercel.com"),
            ];

            let entries: Vec<DropdownMenuEntry> = people
                .into_iter()
                .map(|(username, email)| {
                    let content = Item::new(vec![
                        ItemMedia::new(vec![
                            Avatar::new(vec![
                                AvatarFallback::new(
                                    username
                                        .chars()
                                        .next()
                                        .map(|ch| ch.to_string())
                                        .unwrap_or_else(|| "?".to_owned()),
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        ItemContent::new(vec![
                            ItemTitle::new(username).into_element(cx),
                            ItemDescription::new(email).into_element(cx),
                        ])
                        .gap(Px(2.0))
                        .into_element(cx),
                    ])
                    .size(ItemSize::Sm)
                    .refine_style(
                        ChromeRefinement::default()
                            .p(Space::N2)
                            .rounded(fret_ui_kit::Radius::Md),
                    )
                    .refine_layout(
                        LayoutRefinement::default()
                            .w_full()
                            .h_px(MetricRef::Px(Px(item_h))),
                    )
                    .into_element(cx);

                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new(username)
                            .padding(Edges::all(Px(0.0)))
                            .estimated_height(Px(item_h))
                            .content(content),
                    )
                })
                .collect();

            let dropdown = DropdownMenu::new(open.clone())
                // new-york-v4 item-dropdown: `DropdownMenuContent className="w-72"`.
                .min_width(Px(288.0))
                .align(DropdownMenuAlign::End)
                .into_element(cx, |cx| button.into_element(cx), |_cx| entries);

            cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    padding: Edges {
                        left: Px(trigger_rect.x),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| vec![dropdown],
            )
        },
        SemanticsRole::Button,
        Some("Select"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_item_dropdown_menu_item_height_matches() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_item_dropdown_open_snapshot(theme, expected_h.round());
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_item_dropdown_menu_content_insets_match() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let snap = build_item_dropdown_open_snapshot(theme, expected_item_h.round());
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn build_item_dropdown_open_snapshot(
    theme: &WebGoldenTheme,
    expected_item_h: f32,
) -> fret_core::SemanticsSnapshot {
    let is_open_trigger = |n: &WebNode| {
        n.tag == "button"
            && (n
                .attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == "open")
                || n.attrs
                    .get("aria-expanded")
                    .is_some_and(|v| v.as_str() == "true"))
    };
    let web_trigger = find_first(&theme.root, &is_open_trigger).unwrap_or_else(|| {
        panic!("missing web item-dropdown trigger button (expected in light theme root)")
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_core::Corners;
        use fret_ui::element::LayoutStyle;
        use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
        use fret_ui_shadcn::{
            Avatar, AvatarFallback, Button, ButtonSize, ButtonVariant, DropdownMenu,
            DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, Item, ItemContent,
            ItemDescription, ItemMedia, ItemSize, ItemTitle,
        };

        use fret_ui_kit::declarative::icon as decl_icon;

        let button = Button::new("Select")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(MetricRef::Px(Px(web_trigger.rect.w)))
                    .h_px(MetricRef::Px(Px(web_trigger.rect.h))),
            )
            .children([decl_icon::icon(cx, fret_icons::ids::ui::CHEVRON_DOWN)]);

        let people = vec![
            ("shadcn", "shadcn@vercel.com"),
            ("maxleiter", "maxleiter@vercel.com"),
            ("evilrabbit", "evilrabbit@vercel.com"),
        ];

        let entries: Vec<DropdownMenuEntry> = people
            .into_iter()
            .map(|(username, email)| {
                let content = Item::new(vec![
                    ItemMedia::new(vec![
                        Avatar::new(vec![
                            AvatarFallback::new(
                                username
                                    .chars()
                                    .next()
                                    .map(|ch| ch.to_string())
                                    .unwrap_or_else(|| "?".to_owned()),
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    ItemContent::new(vec![
                        ItemTitle::new(username).into_element(cx),
                        ItemDescription::new(email).into_element(cx),
                    ])
                    .gap(Px(2.0))
                    .into_element(cx),
                ])
                .size(ItemSize::Sm)
                .refine_style(ChromeRefinement::default().p(Space::N2))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(MetricRef::Px(Px(expected_item_h))),
                )
                .into_element(cx);

                DropdownMenuEntry::Item(
                    DropdownMenuItem::new(username)
                        .padding(Edges::all(Px(0.0)))
                        .estimated_height(Px(expected_item_h))
                        .content(content),
                )
            })
            .collect();

        let dropdown = DropdownMenu::new(open.clone())
            .min_width(Px(288.0))
            .align(DropdownMenuAlign::End)
            .into_element(cx, |cx| button.clone().into_element(cx), |_cx| entries);

        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                padding: Edges {
                    left: Px(web_trigger.rect.x),
                    ..Default::default()
                },
                corner_radii: Corners::all(Px(0.0)),
                ..Default::default()
            },
            |_cx| vec![dropdown],
        )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    ui.semantics_snapshot().expect("semantics snapshot").clone()
}

#[test]
fn web_vs_fret_dropdown_menu_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-demo",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
                DropdownMenuLabel, DropdownMenuShortcut,
            };

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Profile")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Billing")
                                    .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Settings")
                                    .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Keyboard shortcuts")
                                    .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ],
                            )),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("New Team")
                                    .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Log out")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
                            ),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

fn build_dropdown_menu_checkboxes_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    checked_status_bar: Model<bool>,
    checked_activity_bar: Model<bool>,
    checked_panel: Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry,
        DropdownMenuLabel,
    };

    DropdownMenu::new(open.clone())
        // new-york-v4 dropdown-menu-checkboxes: `DropdownMenuContent className="w-56"`.
        .min_width(Px(224.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |_cx| {
                vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new("Appearance")),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                        checked_status_bar,
                        "Status Bar",
                    )),
                    DropdownMenuEntry::CheckboxItem(
                        DropdownMenuCheckboxItem::new(checked_activity_bar, "Activity Bar")
                            .disabled(true),
                    ),
                    DropdownMenuEntry::CheckboxItem(DropdownMenuCheckboxItem::new(
                        checked_panel,
                        "Panel",
                    )),
                ]
            },
        )
}

fn build_dropdown_menu_radio_group_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    position: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuLabel,
        DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    DropdownMenu::new(open.clone())
        // new-york-v4 dropdown-menu-radio-group: `DropdownMenuContent className="w-56"`.
        .min_width(Px(224.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |_cx| {
                vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new("Panel Position")),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::RadioGroup(
                        DropdownMenuRadioGroup::new(position)
                            .item(DropdownMenuRadioItemSpec::new("top", "Top"))
                            .item(DropdownMenuRadioItemSpec::new("bottom", "Bottom"))
                            .item(DropdownMenuRadioItemSpec::new("right", "Right")),
                    ),
                ]
            },
        )
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-checkboxes",
        Some("menu"),
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
                };

            build_dropdown_menu_checkboxes_demo(
                cx,
                open,
                checked_status_bar,
                checked_activity_bar,
                checked_panel,
            )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-radio-group",
        Some("menu"),
        |cx, open| {
            #[derive(Default)]
            struct Models {
                position: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| st.position.as_ref().cloned());

            let position = if let Some(existing) = existing {
                existing
            } else {
                let position = cx.app.models_mut().insert(Some(Arc::from("bottom")));
                cx.with_state(Models::default, |st| st.position = Some(position.clone()));
                position
            };

            build_dropdown_menu_radio_group_demo(cx, open, position)
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_button_group_demo_dropdown_menu_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "button-group-demo",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonGroup, ButtonGroupOrientation, ButtonSize, ButtonVariant,
                DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuGroup,
                DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
            };

            fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
                cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            }

            let radius = fret_ui::Theme::global(&*cx.app).metric_required("metric.radius.md");

            let left_button = Button::new("Snooze")
                .variant(ButtonVariant::Outline)
                .corner_radii_override(fret_core::Corners {
                    top_left: radius,
                    bottom_left: radius,
                    top_right: Px(0.0),
                    bottom_right: Px(0.0),
                });

            let right_button = Button::new("More Options")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Icon)
                .border_left_width_override(Px(0.0))
                .corner_radii_override(fret_core::Corners {
                    top_left: Px(0.0),
                    bottom_left: Px(0.0),
                    top_right: radius,
                    bottom_right: radius,
                })
                .children([icon_stub(cx)]);

            let label: Model<Option<Arc<str>>> =
                cx.app.models_mut().insert(Some(Arc::from("personal")));

            let dropdown = DropdownMenu::new(open.clone())
                .align(DropdownMenuAlign::End)
                // new-york-v4 button-group-demo: `DropdownMenuContent className="w-52"`.
                .min_width(Px(208.0))
                .into_element(
                    cx,
                    |cx| right_button.clone().into_element(cx),
                    |cx| {
                        vec![
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                                ),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                                ),
                            ])),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                                ),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                                ),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                                ),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Label As...")
                                        .leading(icon_stub(cx))
                                        .submenu(vec![DropdownMenuEntry::RadioGroup(
                                            DropdownMenuRadioGroup::new(label.clone())
                                                .item(DropdownMenuRadioItemSpec::new(
                                                    "personal", "Personal",
                                                ))
                                                .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                                .item(DropdownMenuRadioItemSpec::new(
                                                    "other", "Other",
                                                )),
                                        )]),
                                ),
                            ])),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Trash")
                                        .leading(icon_stub(cx))
                                        .variant(
                                            fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                        ),
                                ),
                            ])),
                        ]
                    },
                );

            let group3 = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        fret_ui::element::FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |cx| vec![left_button.clone().into_element(cx), dropdown.clone()],
                    )]
                },
            );

            ButtonGroup::new(vec![
                ButtonGroup::new(vec![
                    Button::new("Go Back")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into(),
                ])
                .into(),
                ButtonGroup::new(vec![
                    Button::new("Archive")
                        .variant(ButtonVariant::Outline)
                        .into(),
                    Button::new("Report").variant(ButtonVariant::Outline).into(),
                ])
                .into(),
                group3.into(),
            ])
            .orientation(ButtonGroupOrientation::Horizontal)
            .into_element(cx)
        },
        SemanticsRole::Button,
        Some("More Options"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_mode_toggle_dropdown_menu_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "mode-toggle",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuItem,
            };

            fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
                cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            }

            DropdownMenu::new(open.clone())
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Toggle theme")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Icon)
                            .children([icon_stub(cx)])
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Toggle theme"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_button_group_demo_menu_item_height_matches() {
    assert_button_group_demo_constrained_menu_item_height_matches("button-group-demo");
}

fn assert_button_group_demo_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs =
        web_portal_slot_heights(&theme, &["dropdown-menu-item", "dropdown-menu-sub-trigger"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
        };

        fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(16.0));
                        layout.size.height = Length::Px(Px(16.0));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        }

        DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            // new-york-v4 button-group-demo: `DropdownMenuContent className="w-52"`.
            .min_width(Px(208.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("More Options")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Label As...")
                                    .leading(icon_stub(cx))
                                    .submenu(vec![DropdownMenuEntry::RadioGroup(
                                        DropdownMenuRadioGroup::new(label_value.clone())
                                            .item(DropdownMenuRadioItemSpec::new(
                                                "personal", "Personal",
                                            ))
                                            .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                            .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                    )]),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Trash")
                                    .leading(icon_stub(cx))
                                    .variant(
                                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    ),
                            ),
                        ])),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_button_group_demo_menu_content_insets_match() {
    assert_button_group_demo_constrained_menu_content_insets_match("button-group-demo");
}

fn assert_button_group_demo_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
        };

        fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(16.0));
                        layout.size.height = Length::Px(Px(16.0));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        }

        DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            .min_width(Px(208.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("More Options")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Label As...")
                                    .leading(icon_stub(cx))
                                    .submenu(vec![DropdownMenuEntry::RadioGroup(
                                        DropdownMenuRadioGroup::new(label_value.clone())
                                            .item(DropdownMenuRadioItemSpec::new(
                                                "personal", "Personal",
                                            ))
                                            .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                            .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                    )]),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Trash")
                                    .leading(icon_stub(cx))
                                    .variant(
                                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    ),
                            ),
                        ])),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_mode_toggle_menu_item_height_matches() {
    assert_mode_toggle_constrained_menu_item_height_matches("mode-toggle");
}

fn assert_mode_toggle_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuItem,
        };

        fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(16.0));
                        layout.size.height = Length::Px(Px(16.0));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        }

        DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            .into_element(
                cx,
                |cx| {
                    Button::new("Toggle theme")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_mode_toggle_menu_content_insets_match() {
    assert_mode_toggle_constrained_menu_content_insets_match("mode-toggle");
}

fn assert_mode_toggle_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuItem,
        };

        fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(16.0));
                        layout.size.height = Length::Px(Px(16.0));
                        layout
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )
        }

        DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            .into_element(
                cx,
                |cx| {
                    Button::new("Toggle theme")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "combobox-dropdown-menu",
        Some("menu"),
        |cx, open| {
            use fret_ui_kit::declarative::icon as decl_icon;
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
                DropdownMenuShortcut,
            };

            let button = Button::new("More")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

            let dropdown = DropdownMenu::new(open.clone())
                .align(DropdownMenuAlign::End)
                // new-york-v4 combobox-dropdown-menu: `DropdownMenuContent className="w-[200px]"`.
                .min_width(Px(200.0))
                .into_element(
                    cx,
                    |cx| button.clone().into_element(cx),
                    |cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("Actions")),
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Assign to...")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Set due date...")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Apply label")
                                        .submenu(vec![DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("feature"),
                                        )]),
                                ),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Delete")
                                        .variant(
                                            fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                        )
                                        .trailing(
                                            DropdownMenuShortcut::new("⌘⌫").into_element(cx),
                                        ),
                                ),
                            ])),
                        ]
                    },
                );

            cx.row(
                RowProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    gap: Px(0.0),
                    padding: Edges {
                        top: Px(12.0),   // `py-3`
                        right: Px(16.0), // `px-4`
                        bottom: Px(12.0),
                        left: Px(16.0),
                    },
                    justify: MainAlign::End,
                    align: CrossAlign::Start,
                },
                |_cx| vec![dropdown],
            )
        },
        SemanticsRole::Button,
        Some("More"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_item_height_matches() {
    assert_combobox_dropdown_menu_constrained_menu_item_height_matches("combobox-dropdown-menu");
}

fn assert_combobox_dropdown_menu_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs =
        web_portal_slot_heights(&theme, &["dropdown-menu-item", "dropdown-menu-sub-trigger"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_kit::declarative::icon as decl_icon;
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuShortcut,
        };

        let button = Button::new("More")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

        let dropdown = DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            .min_width(Px(200.0))
            .into_element(
                cx,
                |cx| button.clone().into_element(cx),
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("Actions")),
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Assign to...")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Set due date...")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Apply label").submenu(vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("feature")),
                                ]),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Delete")
                                    .variant(
                                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    )
                                    .trailing(DropdownMenuShortcut::new("⌘⌫").into_element(cx)),
                            ),
                        ])),
                    ]
                },
            );

        cx.row(
            RowProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                gap: Px(0.0),
                padding: Edges {
                    top: Px(12.0),
                    right: Px(16.0),
                    bottom: Px(12.0),
                    left: Px(16.0),
                },
                justify: MainAlign::End,
                align: CrossAlign::Start,
            },
            |_cx| vec![dropdown],
        )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_combobox_dropdown_menu_menu_content_insets_match() {
    assert_combobox_dropdown_menu_constrained_menu_content_insets_match("combobox-dropdown-menu");
}

fn assert_combobox_dropdown_menu_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_kit::declarative::icon as decl_icon;
        use fret_ui_shadcn::{
            Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
            DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuShortcut,
        };

        let button = Button::new("More")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

        let dropdown = DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            .min_width(Px(200.0))
            .into_element(
                cx,
                |cx| button.clone().into_element(cx),
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("Actions")),
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Assign to...")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Set due date...")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Apply label").submenu(vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("feature")),
                                ]),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Delete")
                                    .variant(
                                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    )
                                    .trailing(DropdownMenuShortcut::new("⌘⌫").into_element(cx)),
                            ),
                        ])),
                    ]
                },
            );

        cx.row(
            RowProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                gap: Px(0.0),
                padding: Edges {
                    top: Px(12.0),
                    right: Px(16.0),
                    bottom: Px(12.0),
                    left: Px(16.0),
                },
                justify: MainAlign::End,
                align: CrossAlign::Start,
            },
            |_cx| vec![dropdown],
        )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

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

fn assert_dropdown_menu_demo_constrained_overlay_placement_matches(web_name: &str) {
    assert_overlay_placement_matches(
        web_name,
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
                DropdownMenuLabel, DropdownMenuShortcut,
            };

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Profile")
                                    .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Billing")
                                    .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Settings")
                                    .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Keyboard shortcuts")
                                    .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ],
                            )),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("New Team")
                                    .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Log out")
                                    .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                            ),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_constrained_overlay_placement_matches(
        "dropdown-menu-demo.vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_constrained_overlay_placement_matches(
        "dropdown-menu-demo.vp1440x240",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo.vp1440x240");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo");
}

fn assert_dropdown_menu_demo_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "dropdown-menu-item",
            "dropdown-menu-checkbox-item",
            "dropdown-menu-radio-item",
            "dropdown-menu-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_demo_profile_item_padding_and_shortcut_match() {
    assert_dropdown_menu_demo_profile_item_padding_and_shortcut_match_impl("dropdown-menu-demo");
}

fn assert_dropdown_menu_demo_profile_item_padding_and_shortcut_match_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_profile_item = web_portal_nodes_by_data_slot(&theme, "dropdown-menu-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Profile"))
        })
        .unwrap_or_else(|| panic!("missing web Profile dropdown-menu-item node for {web_name}"));

    let expected_pad_left = web_css_px(web_profile_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Profile paddingLeft for {web_name}"));
    let expected_pad_right = web_css_px(web_profile_item, "paddingRight")
        .unwrap_or_else(|| panic!("missing web Profile paddingRight for {web_name}"));
    let expected_gap = web_css_px(web_profile_item, "gap")
        .unwrap_or_else(|| panic!("missing web Profile gap for {web_name}"));
    assert_close(
        &format!("{web_name} web Profile gap px"),
        expected_gap,
        8.0,
        0.1,
    );

    let web_profile_shortcut = find_first(web_profile_item, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == "dropdown-menu-shortcut")
    })
    .unwrap_or_else(|| panic!("missing web Profile dropdown-menu-shortcut node for {web_name}"));
    let expected_shortcut_right_inset =
        rect_right(web_profile_item.rect) - rect_right(web_profile_shortcut.rect);
    assert_close(
        &format!("{web_name} web Profile shortcut right inset == paddingRight"),
        expected_shortcut_right_inset,
        expected_pad_right,
        0.25,
    );

    let web_sub_trigger = web_portal_node_by_data_slot(&theme, "dropdown-menu-sub-trigger");
    let web_sub_trigger_pad_right =
        web_css_px(web_sub_trigger, "paddingRight").unwrap_or_else(|| {
            panic!("missing web dropdown-menu-sub-trigger paddingRight for {web_name}")
        });
    let web_sub_trigger_chevron =
        find_first(web_sub_trigger, &|n| n.tag == "svg").unwrap_or_else(|| {
            panic!("missing web dropdown-menu-sub-trigger chevron svg for {web_name}")
        });
    let expected_chevron_right_inset =
        rect_right(web_sub_trigger.rect) - rect_right(web_sub_trigger_chevron.rect);
    assert_close(
        &format!("{web_name} web Invite users chevron right inset == paddingRight"),
        expected_chevron_right_inset,
        web_sub_trigger_pad_right,
        0.25,
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .test_id("dropdown-menu.profile")
                                .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .max_by(|a, b| {
            let area_a = a.bounds.size.width.0 * a.bounds.size.height.0;
            let area_b = b.bounds.size.width.0 * b.bounds.size.height.0;
            area_a.total_cmp(&area_b)
        })
        .unwrap_or_else(|| panic!("missing fret Menu semantics for {web_name}"));

    let profile_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id.as_deref() == Some("dropdown-menu.profile")
                && fret_rect_contains(menu.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Profile MenuItem semantics for {web_name}"));
    let profile_label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Profile")
                && fret_rect_contains(profile_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Profile Text semantics for {web_name}"));
    let profile_shortcut_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("⇧⌘P")
                && fret_rect_contains(profile_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Profile shortcut Text semantics for {web_name}"));

    let actual_pad_left = profile_label_text.bounds.origin.x.0 - profile_item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Profile paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );
    let profile_right = profile_item.bounds.origin.x.0 + profile_item.bounds.size.width.0;
    let shortcut_right =
        profile_shortcut_text.bounds.origin.x.0 + profile_shortcut_text.bounds.size.width.0;
    let actual_shortcut_right_inset = profile_right - shortcut_right;
    assert_close(
        &format!("{web_name} Profile shortcut right inset"),
        actual_shortcut_right_inset,
        expected_shortcut_right_inset,
        1.0,
    );

    let invite_users_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.label.as_deref() == Some("Invite users")
                && fret_rect_contains(menu.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Invite users MenuItem semantics for {web_name}"));
    let invite_right = invite_users_item.bounds.origin.x.0 + invite_users_item.bounds.size.width.0;
    let chevron_candidates: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| fret_rect_contains(invite_users_item.bounds, n.bounds))
        .filter(|n| {
            (n.bounds.size.width.0 - 16.0).abs() <= 1.0
                && (n.bounds.size.height.0 - 16.0).abs() <= 1.0
        })
        .filter(|n| n.bounds.origin.x.0 >= invite_right - 48.0)
        .collect();
    let chevron = chevron_candidates
        .into_iter()
        .max_by(|a, b| {
            let right_a = a.bounds.origin.x.0 + a.bounds.size.width.0;
            let right_b = b.bounds.origin.x.0 + b.bounds.size.width.0;
            right_a.total_cmp(&right_b)
        })
        .unwrap_or_else(|| {
            let sample: Vec<_> = snap
                .nodes
                .iter()
                .filter(|n| fret_rect_contains(invite_users_item.bounds, n.bounds))
                .map(|n| (n.role, n.label.as_deref(), n.bounds))
                .take(24)
                .collect();
            panic!(
                "missing fret Invite users chevron candidate for {web_name}; sample(role,label,bounds)={sample:?}"
            )
        });

    let chevron_right = chevron.bounds.origin.x.0 + chevron.bounds.size.width.0;
    let actual_chevron_right_inset = invite_right - chevron_right;
    assert_close(
        &format!("{web_name} Invite users chevron right inset"),
        actual_chevron_right_inset,
        expected_chevron_right_inset,
        1.0,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_checkbox_indicator_slot_inset_matches_web() {
    assert_dropdown_menu_checkboxes_indicator_slot_inset_matches_web_impl(
        "dropdown-menu-checkboxes",
    );
}

fn assert_dropdown_menu_checkboxes_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "dropdown-menu-checkbox-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Status Bar"))
        })
        .unwrap_or_else(|| {
            panic!("missing web Status Bar dropdown-menu-checkbox-item node for {web_name}")
        });
    let expected_pad_left = web_css_px(web_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Status Bar paddingLeft for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItemCheckbox && n.label.as_deref() == Some("Status Bar")
        })
        .unwrap_or_else(|| panic!("missing fret Status Bar MenuItemCheckbox for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Status Bar for {web_name}"));
    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Status Bar")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Status Bar Text node for {web_name}"));

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Status Bar paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_content_insets_match() {
    let web_name = "dropdown-menu-checkboxes";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_item_height_matches() {
    let web_name = "dropdown-menu-checkboxes";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-checkbox-item"]);
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web dropdown-menu-checkbox-item height for {web_name}")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_radio_indicator_slot_inset_matches_web() {
    assert_dropdown_menu_radio_group_indicator_slot_inset_matches_web_impl(
        "dropdown-menu-radio-group",
    );
}

fn assert_dropdown_menu_radio_group_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "dropdown-menu-radio-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Bottom"))
        })
        .unwrap_or_else(|| {
            panic!("missing web Bottom dropdown-menu-radio-item node for {web_name}")
        });
    let expected_pad_left = web_css_px(web_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Bottom paddingLeft for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let position: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("bottom")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItemRadio && n.label.as_deref() == Some("Bottom"))
        .unwrap_or_else(|| panic!("missing fret Bottom MenuItemRadio for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Bottom for {web_name}"));

    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Bottom")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Bottom Text node for {web_name}"));

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Bottom paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_menu_content_insets_match() {
    let web_name = "dropdown-menu-radio-group";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let position: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("bottom")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_dropdown_menu_radio_group_menu_item_height_matches() {
    let web_name = "dropdown-menu-radio-group";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-radio-item"]);
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web dropdown-menu-radio-item height for {web_name}")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let position: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("bottom")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        let request_semantics = frame == 2 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_dropdown_menu_radio_group_demo(cx, &open, position.clone());
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match(
        "dropdown-menu-demo.vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match(
        "dropdown-menu-demo.vp1440x240",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_scroll_state_matches() {
    assert_dropdown_menu_demo_constrained_scroll_state_matches("dropdown-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_scroll_state_matches() {
    assert_dropdown_menu_demo_constrained_scroll_state_matches("dropdown-menu-demo.vp1440x240");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match("dropdown-menu-demo");
}

fn assert_dropdown_menu_demo_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_dropdown_menu_demo_constrained_scroll_state_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_first_visible_label = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(&theme, "dropdown-menu-content"),
        &[
            "Profile",
            "Billing",
            "Settings",
            "Keyboard shortcuts",
            "Team",
            "Invite users",
            "New Team",
            "GitHub",
            "Support",
            "API",
            "Log out",
        ],
    )
    .unwrap_or_else(|| panic!("missing web first visible menu item for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let labels = [
        "Profile",
        "Billing",
        "Settings",
        "Keyboard shortcuts",
        "Team",
        "Invite users",
        "New Team",
        "GitHub",
        "Support",
        "API",
        "Log out",
    ];

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    // Web goldens capture shortly after hover (`wait=50ms`), so we compare against that same
    // mid-transition point rather than fully settling the viewport size animation.
    let hover_settle_frames =
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 / 2 + 1;
    for tick in 0..hover_settle_frames {
        let request_semantics = tick + 1 == hover_settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let root_menu = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("fret root menu semantics");

    let first_visible =
        fret_first_visible_menu_item_label(&snap, root_menu.bounds, &labels).unwrap_or("<missing>");
    assert_eq!(
        first_visible, expected_first_visible_label,
        "{web_name}: first visible menu item label mismatch"
    );
}

fn assert_dropdown_menu_demo_submenu_overlay_placement_matches(web_name: &str) {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| Button::new("Open").into_element(cx),
                    |_cx| {
                        vec![DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Invite users").submenu(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ]),
                        )]
                    },
                );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = DropdownMenu::new(open.clone())
                .min_width(Px(224.0))
                .into_element(
                    cx,
                    |cx| Button::new("Open").into_element(cx),
                    |_cx| {
                        vec![DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Invite users").submenu(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ]),
                        )]
                    },
                );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics");
    ui.set_focus(Some(trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| {
                let el = DropdownMenu::new(open.clone())
                    .min_width(Px(224.0))
                    .into_element(
                        cx,
                        |cx| Button::new("Open").into_element(cx),
                        |_cx| {
                            vec![DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Invite users").submenu(vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ]),
                            )]
                        },
                    );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics (final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);

    // Ensure the root menu is also present (guards against selecting some unrelated menu).
    assert!(
        root_menu.bounds.size.width.0 > 0.0 && root_menu.bounds.size.height.0 > 0.0,
        "expected root menu bounds to be non-zero"
    );
}

fn assert_button_group_demo_submenu_overlay_placement_matches(web_name: &str) {
    let (web, snap) = build_button_group_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "dropdown-menu-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Label As..."))
        .expect("fret submenu trigger semantics (final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);

    assert!(
        root_menu.bounds.size.width.0 > 0.0 && root_menu.bounds.size.height.0 > 0.0,
        "expected root menu bounds to be non-zero"
    );
}

fn build_button_group_demo_submenu_snapshot(web_name: &str) -> (WebGolden, SemanticsSnapshot) {
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let root_labels = &[
        "Mark as Read",
        "Archive",
        "Snooze",
        "Add to Calendar",
        "Add to List",
        "Label As...",
        "Trash",
    ];
    let expected_first_visible_label = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(theme, "dropdown-menu-content"),
        root_labels,
    )
    .unwrap_or_else(|| panic!("missing web first visible menu item for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    let render = |cx: &mut ElementContext<'_, App>| {
        DropdownMenu::new(open.clone())
            .align(DropdownMenuAlign::End)
            // new-york-v4 button-group-demo: `DropdownMenuContent className="w-52"`.
            .min_width(Px(208.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("More Options")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children([icon_stub(cx)])
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Label As...")
                                    .leading(icon_stub(cx))
                                    .submenu(vec![DropdownMenuEntry::RadioGroup(
                                        DropdownMenuRadioGroup::new(label_value.clone())
                                            .item(DropdownMenuRadioItemSpec::new(
                                                "personal", "Personal",
                                            ))
                                            .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                            .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                    )]),
                            ),
                        ])),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Trash")
                                    .leading(icon_stub(cx))
                                    .variant(
                                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                    ),
                            ),
                        ])),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let mut frame: u64 = 3;

    // Match the web golden extraction script behavior:
    // - `scrollIntoView({ block: "center" })` on the submenu trigger element
    // - focus the trigger and press ArrowRight
    let mut snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    for _ in 0..3 {
        let root_menu = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Menu)
            .expect("fret root menu semantics");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Label As...")
            })
            .expect("fret submenu trigger semantics (Label As...)");

        let root_center_y = root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5;
        let trigger_center_y = trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5;
        let dy = trigger_center_y - root_center_y;
        if dy.abs() <= 1.0 {
            break;
        }

        let wheel_pos = Point::new(
            Px(root_menu.bounds.origin.x.0 + root_menu.bounds.size.width.0 * 0.5),
            Px(root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: wheel_pos,
                delta: Point::new(Px(0.0), Px(-dy)),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
        frame += 1;
        snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    }

    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Label As..."))
        .expect("fret submenu trigger semantics (Label As..., scrolled)");
    let focus_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: focus_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: focus_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(frame),
        true,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    frame += 1;

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let focused = fret_focused_label(&snap);
    assert_eq!(
        focused,
        Some("Label As..."),
        "{web_name}: failed to focus submenu trigger (Label As...); focused={focused:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick as u64),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Label As..."))
        .expect("fret submenu trigger semantics (Label As..., final)");
    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    let root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("fret root menu contains submenu trigger");
    let actual_first_visible =
        fret_first_visible_menu_item_label(&snap, root_menu.bounds, root_labels)
            .unwrap_or("<missing>");
    assert_eq!(
        actual_first_visible, expected_first_visible_label,
        "{web_name}: root menu scroll state mismatch"
    );
    (web, snap)
}

fn assert_button_group_demo_submenu_constrained_menu_content_insets_match(web_name: &str) {
    let (web, snap) = build_button_group_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_slots = ["dropdown-menu-content", "dropdown-menu-sub-content"];
    let expected = web_menu_content_insets_for_slots(theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(theme, slot).rect.h)
        .collect();

    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "{web_name} expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(&format!("{web_name} menu[{i}] height"), *a, *e, 2.0);
    }
}

fn assert_button_group_demo_submenu_menu_item_height_matches(web_name: &str) {
    let (web, snap) = build_button_group_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_hs =
        web_portal_slot_heights(theme, &["dropdown-menu-item", "dropdown-menu-radio-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu");
}

#[test]
fn web_vs_fret_button_group_demo_submenu_overlay_placement_matches() {
    assert_button_group_demo_submenu_overlay_placement_matches("button-group-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_button_group_demo_submenu_menu_content_insets_match() {
    assert_button_group_demo_submenu_constrained_menu_content_insets_match(
        "button-group-demo.submenu-kbd",
    );
}

#[test]
fn web_vs_fret_button_group_demo_submenu_menu_item_height_matches() {
    assert_button_group_demo_submenu_menu_item_height_matches("button-group-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}

fn build_dropdown_menu_demo_submenu_snapshot(web_name: &str) -> (WebGolden, SemanticsSnapshot) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let root_labels = &[
        "Profile",
        "Billing",
        "Settings",
        "Keyboard shortcuts",
        "Team",
        "Invite users",
        "New Team",
        "GitHub",
        "Support",
        "API",
        "Log out",
    ];
    let expected_first_visible_label = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(theme, "dropdown-menu-content"),
        root_labels,
    )
    .unwrap_or_else(|| panic!("missing web first visible menu item for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
            DropdownMenuLabel, DropdownMenuShortcut,
        };

        DropdownMenu::new(open.clone())
            .min_width(Px(224.0))
            .into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    vec![
                        DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Profile")
                                .trailing(DropdownMenuShortcut::new("??P").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Billing")
                                .trailing(DropdownMenuShortcut::new("?B").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Settings")
                                .trailing(DropdownMenuShortcut::new("?S").into_element(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Keyboard shortcuts")
                                .trailing(DropdownMenuShortcut::new("?K").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                            vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                DropdownMenuEntry::Separator,
                                DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                            ],
                        )),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("New Team")
                                .trailing(DropdownMenuShortcut::new("?+T").into_element(cx)),
                        ),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Log out")
                                .trailing(DropdownMenuShortcut::new("??Q").into_element(cx)),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let mut frame: u64 = 3;

    if web_name.contains("submenu-kbd") {
        // Match the web golden extraction script behavior:
        // - `scrollIntoView({ block: "center" })` on the submenu trigger element
        // - focus the trigger and press ArrowRight
        let mut snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        for _ in 0..3 {
            let root_menu = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Menu)
                .expect("fret root menu semantics");
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users")
                })
                .expect("fret submenu trigger semantics (Invite users)");

            let root_center_y = root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5;
            let trigger_center_y = trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5;
            let dy = trigger_center_y - root_center_y;
            if dy.abs() <= 1.0 {
                break;
            }

            let wheel_pos = Point::new(
                Px(root_menu.bounds.origin.x.0 + root_menu.bounds.size.width.0 * 0.5),
                Px(root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5),
            );
            ui.dispatch_event(
                &mut app,
                &mut services,
                &Event::Pointer(PointerEvent::Wheel {
                    pointer_id: fret_core::PointerId::default(),
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-dy)),
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(frame),
                true,
                |cx| {
                    let el = render(cx);
                    vec![pad_root(cx, Px(0.0), el)]
                },
            );
            frame += 1;
            snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        }

        let trigger = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users")
            })
            .expect("fret submenu trigger semantics (Invite users, scrolled)");
        let focus_point = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: focus_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId::default(),
                position: focus_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
        frame += 1;

        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let focused = fret_focused_label(&snap);
        assert_eq!(
            focused,
            Some("Invite users"),
            "{web_name}: failed to focus submenu trigger (Invite users); focused={focused:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    } else {
        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let root_menu = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Menu)
            .expect("fret root menu semantics");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users")
            })
            .expect("fret submenu trigger semantics (Invite users)");
        assert!(
            fret_rect_contains(root_menu.bounds, trigger.bounds),
            "{web_name}: submenu trigger is not visible in root menu panel (expected to open submenu by hover)"
        );

        let hover_point = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: hover_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
        deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);
    }

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick as u64),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics (Invite users, final)");
    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    let root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("fret root menu contains submenu trigger");
    let actual_first_visible =
        fret_first_visible_menu_item_label(&snap, root_menu.bounds, root_labels)
            .unwrap_or("<missing>");
    assert_eq!(
        actual_first_visible, expected_first_visible_label,
        "{web_name}: root menu scroll state mismatch"
    );
    (web, snap)
}

fn assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(web_name: &str) {
    let (web, snap) = build_dropdown_menu_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_slots = ["dropdown-menu-content", "dropdown-menu-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "{web_name} expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(&format!("{web_name} menu[{i}] height"), *a, *e, 2.0);
    }
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu",
    );
}

fn assert_dropdown_menu_demo_submenu_menu_item_height_matches(web_name: &str) {
    let (web, snap) = build_dropdown_menu_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        theme,
        &[
            "dropdown-menu-item",
            "dropdown-menu-checkbox-item",
            "dropdown-menu-radio-item",
            "dropdown-menu-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

fn assert_dropdown_menu_demo_submenu_first_visible_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let labels = ["Email", "Message", "More..."];
    let expected = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(&theme, "dropdown-menu-sub-content"),
        &labels,
    )
    .unwrap_or_else(|| panic!("missing web first visible submenu item for {web_name}"));

    let (_, snap) = build_dropdown_menu_demo_submenu_snapshot(web_name);
    let _trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Invite users"))
        .expect("fret submenu trigger semantics (Invite users)");
    let email = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email"))
        .expect("fret submenu item semantics (Email)");
    let submenu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|m| fret_rect_contains(m.bounds, email.bounds))
        .expect("submenu menu does not contain submenu items");

    let actual =
        fret_first_visible_menu_item_label(&snap, submenu.bounds, &labels).unwrap_or("<missing>");
    assert_eq!(
        actual, expected,
        "{web_name}: submenu first visible item mismatch"
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches("dropdown-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches("dropdown-menu-demo.submenu");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_menu_item_height_matches() {
    assert_dropdown_menu_demo_submenu_menu_item_height_matches("dropdown-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_menu_item_height_matches() {
    assert_dropdown_menu_demo_submenu_menu_item_height_matches("dropdown-menu-demo.submenu");
}

#[test]
fn web_vs_fret_select_scrollable_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x450",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_select_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-demo",
        Some("listbox"),
        |cx, open| {
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("Fruits").into(),
                    SelectItem::new("apple", "Apple").into(),
                    SelectItem::new("banana", "Banana").into(),
                    SelectItem::new("blueberry", "Blueberry").into(),
                    SelectItem::new("grapes", "Grapes").into(),
                    SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a fruit")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

#[test]
fn web_vs_fret_select_demo_open_option_metrics_match() {
    fn collect_nodes_with_role<'a>(node: &'a WebNode, role: &str, out: &mut Vec<&'a WebNode>) {
        if node.attrs.get("role").is_some_and(|v| v.as_str() == role) {
            out.push(node);
        }
        for child in &node.children {
            collect_nodes_with_role(child, role, out);
        }
    }

    let web = read_web_golden_open("select-demo");
    let theme = web_theme(&web);
    let web_listbox = theme
        .portals
        .iter()
        .find(|n| n.attrs.get("role").is_some_and(|v| v.as_str() == "listbox"))
        .expect("web listbox portal");

    let mut web_option_nodes = Vec::new();
    collect_nodes_with_role(web_listbox, "option", &mut web_option_nodes);
    let mut web_options: Vec<WebRect> = web_option_nodes
        .into_iter()
        .filter(|n| {
            n.attrs
                .get("data-slot")
                .is_some_and(|v| v.as_str() == "select-item")
        })
        .map(|n| n.rect)
        .collect();
    web_options.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));

    if web_options.is_empty() {
        panic!("missing web options");
    }

    let expected_left_inset = web_options[0].x - web_listbox.rect.x;
    let expected_right_inset =
        (web_listbox.rect.x + web_listbox.rect.w) - (web_options[0].x + web_options[0].w);
    let expected_row_h = web_options[0].h;
    let expected_top_to_first = web_options[0].y - web_listbox.rect.y;
    let expected_bottom_from_last = (web_listbox.rect.y + web_listbox.rect.h)
        - (web_options[web_options.len() - 1].y + web_options[web_options.len() - 1].h);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);
    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("Fruits").into(),
                    SelectItem::new("apple", "Apple").into(),
                    SelectItem::new("banana", "Banana").into(),
                    SelectItem::new("blueberry", "Blueberry").into(),
                    SelectItem::new("grapes", "Grapes").into(),
                    SelectItem::new("pineapple", "Pineapple").into(),
                ])
                .into(),
            ];

            let content = fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a fruit")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
                .entries(entries)
                .into_element(cx);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..=settle_frames {
        let frame = 2 + tick;
        let request_semantics = tick == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
                use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

                let entries: Vec<SelectEntry> = vec![
                    SelectGroup::new(vec![
                        SelectLabel::new("Fruits").into(),
                        SelectItem::new("apple", "Apple").into(),
                        SelectItem::new("banana", "Banana").into(),
                        SelectItem::new("blueberry", "Blueberry").into(),
                        SelectItem::new("grapes", "Grapes").into(),
                        SelectItem::new("pineapple", "Pineapple").into(),
                    ])
                    .into(),
                ];

                let content = fret_ui_shadcn::Select::new(value, open.clone())
                    .a11y_label("Select")
                    .placeholder("Select a fruit")
                    .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)))
                    .entries(entries)
                    .into_element(cx);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let expected_portal_w = web_listbox.rect.w;
    let expected_portal_h = web_listbox.rect.h;
    let fret_listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;
            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret listbox portal");

    let listbox_bounds = fret_listbox.bounds;
    let mut fret_options: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| {
            let b = n.bounds;
            b.origin.x.0 >= listbox_bounds.origin.x.0
                && b.origin.y.0 >= listbox_bounds.origin.y.0
                && b.origin.x.0 + b.size.width.0
                    <= listbox_bounds.origin.x.0 + listbox_bounds.size.width.0
                && b.origin.y.0 + b.size.height.0
                    <= listbox_bounds.origin.y.0 + listbox_bounds.size.height.0
        })
        .collect();
    fret_options.sort_by(|a, b| {
        a.bounds
            .origin
            .y
            .0
            .partial_cmp(&b.bounds.origin.y.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if fret_options.len() != web_options.len() {
        panic!(
            "option count mismatch: web={} fret={}",
            web_options.len(),
            fret_options.len()
        );
    }

    let tol = 1.0;
    for (idx, option) in fret_options.iter().enumerate() {
        let b = option.bounds;
        let got_row_h = b.size.height.0;
        if (got_row_h - expected_row_h).abs() > tol {
            panic!(
                "row height mismatch idx={idx}: got={} expected={}",
                got_row_h, expected_row_h
            );
        }

        let got_left_inset = b.origin.x.0 - listbox_bounds.origin.x.0;
        if (got_left_inset - expected_left_inset).abs() > tol {
            panic!(
                "left inset mismatch idx={idx}: got={} expected={}",
                got_left_inset, expected_left_inset
            );
        }
        let got_right_inset = (listbox_bounds.origin.x.0 + listbox_bounds.size.width.0)
            - (b.origin.x.0 + b.size.width.0);
        if (got_right_inset - expected_right_inset).abs() > tol {
            panic!(
                "right inset mismatch idx={idx}: got={} expected={}",
                got_right_inset, expected_right_inset
            );
        }

        let expected_y = web_options[idx].y - web_listbox.rect.y;
        let got_y = b.origin.y.0 - listbox_bounds.origin.y.0;
        if (got_y - expected_y).abs() > 2.0 {
            panic!(
                "option y mismatch idx={idx}: got={} expected={}",
                got_y, expected_y
            );
        }
    }

    let first = fret_options[0].bounds;
    let last = fret_options[fret_options.len() - 1].bounds;
    let got_top_to_first = first.origin.y.0 - listbox_bounds.origin.y.0;
    let got_bottom_from_last = (listbox_bounds.origin.y.0 + listbox_bounds.size.height.0)
        - (last.origin.y.0 + last.size.height.0);

    if (got_top_to_first - expected_top_to_first).abs() > 2.0 {
        panic!(
            "top-to-first mismatch: got={} expected={}",
            got_top_to_first, expected_top_to_first
        );
    }
    if (got_bottom_from_last - expected_bottom_from_last).abs() > 2.0 {
        panic!(
            "bottom-from-last mismatch: got={} expected={}",
            got_bottom_from_last, expected_bottom_from_last
        );
    }
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "select-scrollable.vp1440x240",
        Some("listbox"),
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

            let entries: Vec<SelectEntry> = vec![
                SelectGroup::new(vec![
                    SelectLabel::new("North America").into(),
                    SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                    SelectItem::new("cst", "Central Standard Time (CST)").into(),
                    SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                    SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                    SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                    SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Europe & Africa").into(),
                    SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                    SelectItem::new("cet", "Central European Time (CET)").into(),
                    SelectItem::new("eet", "Eastern European Time (EET)").into(),
                    SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                    SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                    SelectItem::new("eat", "East Africa Time (EAT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Asia").into(),
                    SelectItem::new("msk", "Moscow Time (MSK)").into(),
                    SelectItem::new("ist", "India Standard Time (IST)").into(),
                    SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                    SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                    SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                    SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)")
                        .into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("Australia & Pacific").into(),
                    SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                    SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                    SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                    SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                    SelectItem::new("fjt", "Fiji Time (FJT)").into(),
                ])
                .into(),
                SelectGroup::new(vec![
                    SelectLabel::new("South America").into(),
                    SelectItem::new("art", "Argentina Time (ART)").into(),
                    SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                    SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                    SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
                ])
                .into(),
            ];

            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .placeholder("Select a timezone")
                .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
                .entries(entries)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        Some("Select"),
        SemanticsRole::ListBox,
    );
}

fn assert_select_scrollable_listbox_option_insets_match(web_name: &str) {
    let debug = std::env::var("FRET_DEBUG_SELECT_SCROLLABLE")
        .ok()
        .is_some_and(|v| v == "1");

    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);
    let expected_h = web_listbox.rect.h;
    let expected_inset = web_select_content_option_inset(web_listbox);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value.clone(), open.clone())
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    if debug {
        let mut options: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
            .collect();
        options.sort_by(|a, b| {
            a.bounds
                .origin
                .y
                .0
                .total_cmp(&b.bounds.origin.y.0)
                .then_with(|| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
        });

        eprintln!(
            "[{web_name}] fret listbox y={} h={}",
            listbox.bounds.origin.y.0, listbox.bounds.size.height.0
        );
        for (idx, opt) in options.iter().take(8).enumerate() {
            eprintln!(
                "  opt[{idx}] y={} h={} label={:?}",
                opt.bounds.origin.y.0,
                opt.bounds.size.height.0,
                opt.label.as_deref()
            );
        }
        let total = options.len();
        for (i, opt) in options.iter().rev().take(8).enumerate() {
            let idx = total.saturating_sub(1 + i);
            let bottom = opt.bounds.origin.y.0 + opt.bounds.size.height.0;
            eprintln!(
                "  opt[{idx}] y={} h={} bottom={} label={:?}",
                opt.bounds.origin.y.0,
                opt.bounds.size.height.0,
                bottom,
                opt.label.as_deref()
            );
        }

        let scroll_buttons: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Button)
            .filter(|n| {
                n.label.as_deref() == Some("Scroll up") || n.label.as_deref() == Some("Scroll down")
            })
            .collect();
        for btn in scroll_buttons {
            eprintln!(
                "  button {:?} y={} h={}",
                btn.label.as_deref(),
                btn.bounds.origin.y.0,
                btn.bounds.size.height.0
            );
        }
    }

    assert_close(
        &format!("{web_name} listbox_h"),
        listbox.bounds.size.height.0,
        expected_h,
        1.0,
    );

    let actual_inset = fret_select_content_option_inset(&snap);
    assert_select_inset_match(web_name, actual_inset, expected_inset);
}

#[test]
fn web_vs_fret_select_scrollable_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_option_insets_match() {
    assert_select_scrollable_listbox_option_insets_match("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_listbox_option_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);

    let expected: std::collections::BTreeSet<i32> = web_select_listbox_option_heights(web_listbox)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        expected.len() == 1,
        "{web_name} expected uniform web listbox option height; got {expected:?}"
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        let value = value.clone();
        let open = open.clone();

        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};
        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value, open)
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual: std::collections::BTreeSet<i32> = fret_listbox_option_heights_in_listbox(&snap)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        actual.len() == 1,
        "{web_name} expected uniform fret listbox option height; got {actual:?}"
    );

    let expected_h = expected.iter().next().copied().unwrap_or_default() as f32;
    let actual_h = actual.iter().next().copied().unwrap_or_default() as f32;
    assert_close(
        &format!("{web_name} listbox_option_h"),
        actual_h,
        expected_h,
        1.0,
    );
}

#[test]
fn web_vs_fret_select_scrollable_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_option_height_matches() {
    assert_select_scrollable_listbox_option_height_matches("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_listbox_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);
    let expected_h = web_listbox.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        let value = value.clone();
        let open = open.clone();

        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};
        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value, open)
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_h = fret_listbox_height(&snap);
    assert_close(&format!("{web_name} listbox_h"), actual_h, expected_h, 2.0);
}

#[test]
fn web_vs_fret_select_scrollable_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_height_matches() {
    assert_select_scrollable_listbox_height_matches("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_scroll_button_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);

    let expected: std::collections::BTreeSet<i32> = web_portal_slot_heights(
        theme,
        &["select-scroll-up-button", "select-scroll-down-button"],
    )
    .into_iter()
    .map(round_i32)
    .collect();
    assert!(
        expected.len() == 1,
        "{web_name} expected uniform web select scroll button height; got {expected:?}"
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        let value = value.clone();
        let open = open.clone();

        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};
        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value, open)
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    let up_nodes = fret_nodes_by_test_id(&snap, "select-scroll-up-button");
    let down_nodes = fret_nodes_by_test_id(&snap, "select-scroll-down-button");

    assert!(
        !up_nodes.is_empty(),
        "{web_name} missing fret scroll-up node"
    );
    assert!(
        !down_nodes.is_empty(),
        "{web_name} missing fret scroll-down node"
    );

    // Match web: these nodes exist for geometry but are aria-hidden (not in the a11y tree).
    for (label, nodes) in [("scroll-up", &up_nodes), ("scroll-down", &down_nodes)] {
        assert!(
            nodes.iter().all(|n| n.role == SemanticsRole::Generic),
            "{web_name} expected {label} role=Generic; got roles={:?}",
            nodes.iter().map(|n| n.role).collect::<Vec<_>>()
        );
        assert!(
            nodes.iter().all(|n| !n.actions.focus && !n.actions.invoke),
            "{web_name} expected {label} to have no focus/invoke actions"
        );
    }

    let up: std::collections::BTreeSet<i32> =
        fret_node_heights_by_test_id(&snap, "select-scroll-up-button")
            .into_iter()
            .map(round_i32)
            .collect();
    let down: std::collections::BTreeSet<i32> =
        fret_node_heights_by_test_id(&snap, "select-scroll-down-button")
            .into_iter()
            .map(round_i32)
            .collect();

    assert!(
        up.len() == 1,
        "{web_name} expected 1 fret scroll-up node height; got {up:?}"
    );
    assert!(
        down.len() == 1,
        "{web_name} expected 1 fret scroll-down node height; got {down:?}"
    );

    let expected_h = expected.iter().next().copied().unwrap_or_default() as f32;
    let up_h = up.iter().next().copied().unwrap_or_default() as f32;
    let down_h = down.iter().next().copied().unwrap_or_default() as f32;

    assert_close(&format!("{web_name} scroll_up_h"), up_h, expected_h, 1.0);
    assert_close(
        &format!("{web_name} scroll_down_h"),
        down_h,
        expected_h,
        1.0,
    );

    let web_up = web_portal_slot_rect_within(theme, "select-scroll-up-button", web_listbox.rect);
    let web_down =
        web_portal_slot_rect_within(theme, "select-scroll-down-button", web_listbox.rect);

    let fret_up = up_nodes
        .iter()
        .copied()
        .find(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .unwrap_or(up_nodes[0])
        .bounds;
    let fret_down = down_nodes
        .iter()
        .copied()
        .find(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .unwrap_or(down_nodes[0])
        .bounds;

    let web_left = web_up.x - web_listbox.rect.x;
    let web_right = rect_right(web_listbox.rect) - rect_right(web_up);
    let web_top = web_up.y - web_listbox.rect.y;
    assert_close(
        &format!("{web_name} scroll_up_left"),
        fret_up.origin.x.0 - listbox.bounds.origin.x.0,
        web_left,
        1.0,
    );
    assert_close(
        &format!("{web_name} scroll_up_right"),
        (listbox.bounds.origin.x.0 + listbox.bounds.size.width.0)
            - (fret_up.origin.x.0 + fret_up.size.width.0),
        web_right,
        1.0,
    );
    assert_close(
        &format!("{web_name} scroll_up_top"),
        fret_up.origin.y.0 - listbox.bounds.origin.y.0,
        web_top,
        1.0,
    );

    let web_left = web_down.x - web_listbox.rect.x;
    let web_right = rect_right(web_listbox.rect) - rect_right(web_down);
    let web_bottom = rect_bottom(web_listbox.rect) - rect_bottom(web_down);
    assert_close(
        &format!("{web_name} scroll_down_left"),
        fret_down.origin.x.0 - listbox.bounds.origin.x.0,
        web_left,
        1.0,
    );
    assert_close(
        &format!("{web_name} scroll_down_right"),
        (listbox.bounds.origin.x.0 + listbox.bounds.size.width.0)
            - (fret_down.origin.x.0 + fret_down.size.width.0),
        web_right,
        1.0,
    );
    assert_close(
        &format!("{web_name} scroll_down_bottom"),
        (listbox.bounds.origin.y.0 + listbox.bounds.size.height.0)
            - (fret_down.origin.y.0 + fret_down.size.height.0),
        web_bottom,
        1.0,
    );
}

#[test]
fn web_vs_fret_select_scrollable_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_scroll_button_height_matches() {
    assert_select_scrollable_scroll_button_height_matches("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_viewport_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);

    let web_up = web_portal_slot_rect_within(theme, "select-scroll-up-button", web_listbox.rect);
    let web_down =
        web_portal_slot_rect_within(theme, "select-scroll-down-button", web_listbox.rect);
    let web_viewport = web_portal_role_rect_largest_within(theme, "presentation", web_listbox.rect);

    let expected_left = web_viewport.x - web_listbox.rect.x;
    let expected_right = rect_right(web_listbox.rect) - rect_right(web_viewport);
    let expected_top = web_viewport.y - web_listbox.rect.y;
    let expected_bottom = rect_bottom(web_listbox.rect) - rect_bottom(web_viewport);

    let expected_up_gap = web_viewport.y - rect_bottom(web_up);
    let expected_down_gap = web_down.y - rect_bottom(web_viewport);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        let value = value.clone();
        let open = open.clone();

        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};
        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value, open)
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    let viewport = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-scroll-viewport"))
        .unwrap_or_else(|| panic!("missing fret select viewport for {web_name}"));

    let up = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-scroll-up-button"))
        .unwrap_or_else(|| panic!("missing fret scroll-up node for {web_name}"));
    let down = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-scroll-down-button"))
        .unwrap_or_else(|| panic!("missing fret scroll-down node for {web_name}"));

    let actual_left = viewport.bounds.origin.x.0 - listbox.bounds.origin.x.0;
    let actual_right = (listbox.bounds.origin.x.0 + listbox.bounds.size.width.0)
        - (viewport.bounds.origin.x.0 + viewport.bounds.size.width.0);
    let actual_top = viewport.bounds.origin.y.0 - listbox.bounds.origin.y.0;
    let actual_bottom = (listbox.bounds.origin.y.0 + listbox.bounds.size.height.0)
        - (viewport.bounds.origin.y.0 + viewport.bounds.size.height.0);

    assert_close(
        &format!("{web_name} viewport_left"),
        actual_left,
        expected_left,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_right"),
        actual_right,
        expected_right,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_top"),
        actual_top,
        expected_top,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_bottom"),
        actual_bottom,
        expected_bottom,
        1.0,
    );

    let actual_up_gap =
        viewport.bounds.origin.y.0 - (up.bounds.origin.y.0 + up.bounds.size.height.0);
    let actual_down_gap =
        down.bounds.origin.y.0 - (viewport.bounds.origin.y.0 + viewport.bounds.size.height.0);
    assert_close(
        &format!("{web_name} viewport_up_gap"),
        actual_up_gap,
        expected_up_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_down_gap"),
        actual_down_gap,
        expected_down_gap,
        1.0,
    );
}

#[test]
fn web_vs_fret_select_scrollable_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_viewport_insets_match() {
    assert_select_scrollable_viewport_insets_match("select-scrollable.vp1440x240");
}

fn assert_select_scrollable_listbox_width_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_select_listbox(theme);
    let expected_w = web_listbox.rect.w;
    let expected_trigger_w = web_select_combobox(theme).rect.w;
    assert!(expected_w + 0.01 >= expected_trigger_w);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

        let entries: Vec<SelectEntry> = vec![
            SelectGroup::new(vec![
                SelectLabel::new("North America").into(),
                SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                SelectItem::new("cst", "Central Standard Time (CST)").into(),
                SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Europe & Africa").into(),
                SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                SelectItem::new("cet", "Central European Time (CET)").into(),
                SelectItem::new("eet", "Eastern European Time (EET)").into(),
                SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Asia").into(),
                SelectItem::new("msk", "Moscow Time (MSK)").into(),
                SelectItem::new("ist", "India Standard Time (IST)").into(),
                SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("Australia & Pacific").into(),
                SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            SelectGroup::new(vec![
                SelectLabel::new("South America").into(),
                SelectItem::new("art", "Argentina Time (ART)").into(),
                SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ];

        fret_ui_shadcn::Select::new(value.clone(), open.clone())
            .a11y_label("Select")
            .placeholder("Select a timezone")
            .refine_layout(fret_ui_kit::LayoutRefinement::default().w_px(Px(280.0)))
            .entries(entries)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let combobox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .unwrap_or_else(|| panic!("missing fret combobox for {web_name}"));
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox for {web_name}"));

    assert!(
        listbox.bounds.size.width.0 + 0.01 >= combobox.bounds.size.width.0,
        "{web_name} expected listbox width >= trigger width ({} >= {})",
        listbox.bounds.size.width.0,
        combobox.bounds.size.width.0
    );
    assert_close(
        &format!("{web_name} listbox_w"),
        listbox.bounds.size.width.0,
        expected_w,
        2.0,
    );
}

#[test]
fn web_vs_fret_select_scrollable_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable");
}

#[test]
fn web_vs_fret_select_scrollable_small_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x450");
}

#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_listbox_width_matches() {
    assert_select_scrollable_listbox_width_matches("select-scrollable.vp1440x240");
}

fn web_portal_first_node_by_role<'a>(theme: &'a WebGoldenTheme, role: &str) -> &'a WebNode {
    for portal in &theme.portals {
        if let Some(found) = find_first(portal, &|n| n.attrs.get("role").is_some_and(|v| v == role))
        {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) =
            find_first(wrapper, &|n| n.attrs.get("role").is_some_and(|v| v == role))
        {
            return found;
        }
    }
    panic!("missing web portal node with role={role}")
}

fn combobox_demo_open_snapshot(theme: &WebGoldenTheme) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Combobox, ComboboxItem};

        let items = vec![
            ComboboxItem::new("apple", "Apple"),
            ComboboxItem::new("banana", "Banana"),
            ComboboxItem::new("blueberry", "Blueberry"),
            ComboboxItem::new("grapes", "Grapes"),
            ComboboxItem::new("pineapple", "Pineapple"),
        ];

        Combobox::new(value.clone(), open.clone())
            .a11y_label("Select a fruit")
            .width(Px(200.0))
            .items(items)
            .into_element(cx)
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    ui.semantics_snapshot().expect("semantics snapshot").clone()
}

fn assert_combobox_demo_listbox_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_portal_first_node_by_role(theme, "listbox");
    let expected_h = web_listbox.rect.h;

    let snap = combobox_demo_open_snapshot(theme);
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret combobox listbox for {web_name}"));

    assert_close(
        &format!("{web_name} combobox listbox_h"),
        listbox.bounds.size.height.0,
        expected_h,
        2.0,
    );
}

fn assert_combobox_demo_listbox_option_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_portal_first_node_by_role(theme, "listbox");

    let expected: std::collections::BTreeSet<i32> = web_select_listbox_option_heights(web_listbox)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        expected.len() == 1,
        "{web_name} expected uniform web combobox option height; got {expected:?}"
    );

    let snap = combobox_demo_open_snapshot(theme);
    let actual: std::collections::BTreeSet<i32> = fret_listbox_option_heights_in_listbox(&snap)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        actual.len() == 1,
        "{web_name} expected uniform fret combobox option height; got {actual:?}"
    );

    let expected_h = expected.iter().next().copied().unwrap_or_default() as f32;
    let actual_h = actual.iter().next().copied().unwrap_or_default() as f32;
    assert_close(
        &format!("{web_name} combobox_option_h"),
        actual_h,
        expected_h,
        1.0,
    );
}

fn assert_combobox_demo_listbox_option_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_portal_first_node_by_role(theme, "listbox");
    let expected_inset = web_select_content_option_inset(web_listbox);

    let snap = combobox_demo_open_snapshot(theme);
    let actual_inset = fret_select_content_option_inset(&snap);
    assert_select_inset_match(web_name, actual_inset, expected_inset);
}

#[test]
fn web_vs_fret_combobox_demo_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo");
}

#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp1440x320");
}

#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp375x320");
}

#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-demo.vp1440x240");
}

#[test]
fn web_vs_fret_combobox_demo_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo");
}

#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp1440x320");
}

#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp375x320");
}

#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-demo.vp1440x240");
}

#[test]
fn web_vs_fret_combobox_demo_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo");
}

#[test]
fn web_vs_fret_combobox_demo_constrained_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp1440x320");
}

#[test]
fn web_vs_fret_combobox_demo_small_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp375x320");
}

#[test]
fn web_vs_fret_combobox_demo_tiny_viewport_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-demo.vp1440x240");
}

#[test]
fn web_vs_fret_combobox_popover_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "combobox-popover",
        Some("dialog"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonVariant, Popover, PopoverAlign, PopoverContent, PopoverSide,
            };

            Popover::new(open.clone())
                .side(PopoverSide::Right)
                .align(PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("Open")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| {
                        PopoverContent::new(Vec::new())
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_px(MetricRef::Px(Px(288.0)))
                                    .h_px(MetricRef::Px(Px(205.33334))),
                            )
                            .into_element(cx)
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_combobox_responsive_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "combobox-responsive",
        Some("dialog"),
        |cx, open| {
            use fret_ui_shadcn::{Combobox, ComboboxItem};

            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            let items = vec![
                ComboboxItem::new("nextjs", "Next.js"),
                ComboboxItem::new("sveltekit", "SvelteKit"),
                ComboboxItem::new("nuxt", "Nuxt.js"),
                ComboboxItem::new("remix", "Remix"),
                ComboboxItem::new("astro", "Astro"),
            ];

            Combobox::new(value, open.clone())
                .a11y_label("Select a framework")
                .width(Px(200.0))
                .items(items)
                .into_element(cx)
        },
        SemanticsRole::ComboBox,
        None,
        SemanticsRole::Dialog,
    );
}

#[test]
fn web_vs_fret_combobox_popover_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-popover");
}

#[test]
fn web_vs_fret_combobox_responsive_listbox_height_matches() {
    assert_combobox_demo_listbox_height_matches("combobox-responsive");
}

#[test]
fn web_vs_fret_combobox_popover_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-popover");
}

#[test]
fn web_vs_fret_combobox_responsive_listbox_option_height_matches() {
    assert_combobox_demo_listbox_option_height_matches("combobox-responsive");
}

#[test]
fn web_vs_fret_combobox_popover_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-popover");
}

#[test]
fn web_vs_fret_combobox_responsive_listbox_option_insets_match() {
    assert_combobox_demo_listbox_option_insets_match("combobox-responsive");
}

fn assert_point_anchored_overlay_placement_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_portal_role: SemanticsRole,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    open_fret_at: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        AppWindowId,
        WebPoint,
    ),
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let open_point = theme
        .open
        .as_ref()
        .map(|m| m.point)
        .unwrap_or_else(|| panic!("missing web open point for {web_name}"));

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == web_portal_role))
        .unwrap_or_else(|| panic!("missing web portal role={web_portal_role}"));
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_trigger = point_rect(open_point);
    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let content = build_frame1(cx, &open);
            vec![pad_root(cx, Px(0.0), content)]
        },
    );

    open_fret_at(&mut ui, &mut app, &mut services, window, open_point);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let build_settle = build.clone();
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let content = build_settle(cx, &open);
                vec![pad_root(cx, Px(0.0), content)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;
    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == fret_portal_role)
        .min_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let ah = a.bounds.size.height.0;
            let bw = b.bounds.size.width.0;
            let bh = b.bounds.size.height.0;

            let score_a = (aw - expected_portal_w).abs() + (ah - expected_portal_h).abs();
            let score_b = (bw - expected_portal_w).abs() + (bh - expected_portal_h).abs();
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| panic!("missing fret portal role={fret_portal_role:?}"));

    let fret_trigger = point_rect(open_point);
    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );

    if fret_portal_role == SemanticsRole::Menu {
        assert_close(
            &format!("{web_name} portal_w"),
            fret_portal.w,
            expected_portal_w,
            2.0,
        );
        assert_close(
            &format!("{web_name} portal_h"),
            fret_portal.h,
            expected_portal_h,
            2.0,
        );
    }
}

#[test]
fn web_vs_fret_context_menu_demo_overlay_placement_matches() {
    assert_point_anchored_overlay_placement_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone())
                // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
                .min_width(Px(208.0))
                // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
                .submenu_min_width(Px(176.0))
                .into_element(
                cx,
                |cx| {
                    cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(300.0));
                                layout.size.height = Length::Px(Px(150.0));
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("Right click here")],
                    )
                },
                |cx| {
                    vec![
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Back")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘[")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Forward")
                                .inset(true)
                                .disabled(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘]")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Reload")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘R")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("More Tools").inset(true).submenu(
                                vec![
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new(
                                            "Create Shortcut...",
                                        ),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Developer Tools"),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                            fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                        ),
                                    ),
                                ],
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_bookmarks,
                                "Show Bookmarks",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_full_urls,
                                "Show Full URLs",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::Label(
                            fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                            fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person)
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "pedro",
                                    "Pedro Duarte",
                                ))
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "colm",
                                    "Colm Tuite",
                                )),
                        ),
                    ]
                },
            )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    is_click: true,
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}

fn assert_context_menu_demo_constrained_overlay_placement_matches(web_name: &str) {
    assert_point_anchored_overlay_placement_matches(
        web_name,
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone())
                // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
                .min_width(Px(208.0))
                // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| {
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(300.0));
                                    layout.size.height = Length::Px(Px(150.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |cx| vec![cx.text("Right click here")],
                        )
                    },
                    |cx| {
                        vec![
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Back")
                                    .inset(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?[")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Forward")
                                    .inset(true)
                                    .disabled(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?]")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("Reload")
                                    .inset(true)
                                    .trailing(
                                        fret_ui_shadcn::ContextMenuShortcut::new("?R")
                                            .into_element(cx),
                                    ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Item(
                                fret_ui_shadcn::ContextMenuItem::new("More Tools")
                                    .inset(true)
                                    .submenu(vec![
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new(
                                                "Create Shortcut...",
                                            ),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Separator,
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new(
                                                "Developer Tools",
                                            ),
                                        ),
                                        fret_ui_shadcn::ContextMenuEntry::Separator,
                                        fret_ui_shadcn::ContextMenuEntry::Item(
                                            fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                            ),
                                        ),
                                    ]),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Separator,
                            fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                                fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                    checked_bookmarks.clone(),
                                    "Show Bookmarks",
                                ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                                fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                    checked_full_urls.clone(),
                                    "Show Full URLs",
                                ),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::Separator,
                            fret_ui_shadcn::ContextMenuEntry::Label(
                                fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                            ),
                            fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                                fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person.clone())
                                    .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                        "pedro",
                                        "Pedro Duarte",
                                    ))
                                    .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                        "colm",
                                        "Colm Tuite",
                                    )),
                            ),
                        ]
                    },
                )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    is_click: true,
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_overlay_placement_matches() {
    assert_context_menu_demo_constrained_overlay_placement_matches("context-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_constrained_overlay_placement_matches("context-menu-demo.vp1440x240");
}

fn assert_context_menu_demo_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "context-menu-item",
            "context-menu-checkbox-item",
            "context-menu-radio-item",
            "context-menu-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo.vp1440x240");
}

#[test]
fn web_vs_fret_context_menu_demo_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo");
}

#[test]
fn web_vs_fret_context_menu_demo_back_item_padding_and_shortcut_match() {
    assert_context_menu_demo_back_item_padding_and_shortcut_match_impl("context-menu-demo");
}

fn assert_context_menu_demo_back_item_padding_and_shortcut_match_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_back_item = web_portal_nodes_by_data_slot(&theme, "context-menu-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Back"))
        })
        .unwrap_or_else(|| panic!("missing web Back context-menu-item node for {web_name}"));

    let expected_pad_left = web_css_px(web_back_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Back paddingLeft for {web_name}"));
    let expected_pad_right = web_css_px(web_back_item, "paddingRight")
        .unwrap_or_else(|| panic!("missing web Back paddingRight for {web_name}"));
    let expected_gap = web_css_px(web_back_item, "gap")
        .unwrap_or_else(|| panic!("missing web Back gap for {web_name}"));
    assert_close(
        &format!("{web_name} web Back gap px"),
        expected_gap,
        8.0,
        0.1,
    );

    let web_back_shortcut = find_first(web_back_item, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == "context-menu-shortcut")
    })
    .unwrap_or_else(|| panic!("missing web Back context-menu-shortcut node for {web_name}"));
    let expected_shortcut_right_inset =
        rect_right(web_back_item.rect) - rect_right(web_back_shortcut.rect);
    assert_close(
        &format!("{web_name} web Back shortcut right inset == paddingRight"),
        expected_shortcut_right_inset,
        expected_pad_right,
        0.25,
    );

    let web_sub_trigger = web_portal_node_by_data_slot(&theme, "context-menu-sub-trigger");
    let web_sub_trigger_pad_right =
        web_css_px(web_sub_trigger, "paddingRight").unwrap_or_else(|| {
            panic!("missing web context-menu-sub-trigger paddingRight for {web_name}")
        });
    let web_sub_trigger_chevron =
        find_first(web_sub_trigger, &|n| n.tag == "svg").unwrap_or_else(|| {
            panic!("missing web context-menu-sub-trigger chevron svg for {web_name}")
        });
    let expected_chevron_right_inset =
        rect_right(web_sub_trigger.rect) - rect_right(web_sub_trigger_chevron.rect);
    assert_close(
        &format!("{web_name} web More Tools chevron right inset == paddingRight"),
        expected_chevron_right_inset,
        web_sub_trigger_pad_right,
        0.25,
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();

    let back_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id.as_deref() == Some("context-menu.back")
                && n.label.as_deref() == Some("Back")
        })
        .unwrap_or_else(|| panic!("missing fret Back MenuItem semantics for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, back_item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Back item for {web_name}"));

    let back_label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Back")
                && fret_rect_contains(back_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Back Text semantics for {web_name}"));
    let back_shortcut_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("⌘[")
                && fret_rect_contains(back_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Back shortcut Text semantics for {web_name}"));

    let actual_pad_left = back_label_text.bounds.origin.x.0 - back_item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Back paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    let back_right = back_item.bounds.origin.x.0 + back_item.bounds.size.width.0;
    let shortcut_right =
        back_shortcut_text.bounds.origin.x.0 + back_shortcut_text.bounds.size.width.0;
    let actual_shortcut_right_inset = back_right - shortcut_right;
    assert_close(
        &format!("{web_name} Back shortcut right inset"),
        actual_shortcut_right_inset,
        expected_shortcut_right_inset,
        1.0,
    );

    let more_tools_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id.as_deref() == Some("context-menu.more_tools")
                && n.label.as_deref() == Some("More Tools")
                && fret_rect_contains(menu.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret More Tools MenuItem semantics for {web_name}"));

    let more_tools_right = more_tools_item.bounds.origin.x.0 + more_tools_item.bounds.size.width.0;
    let chevron_candidates: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| fret_rect_contains(more_tools_item.bounds, n.bounds))
        .filter(|n| {
            (n.bounds.size.width.0 - 16.0).abs() <= 1.0
                && (n.bounds.size.height.0 - 16.0).abs() <= 1.0
        })
        .filter(|n| n.bounds.origin.x.0 >= more_tools_right - 48.0)
        .collect();
    let chevron = chevron_candidates
        .into_iter()
        .max_by(|a, b| {
            let right_a = a.bounds.origin.x.0 + a.bounds.size.width.0;
            let right_b = b.bounds.origin.x.0 + b.bounds.size.width.0;
            right_a.total_cmp(&right_b)
        })
        .unwrap_or_else(|| {
            let sample: Vec<_> = snap
                .nodes
                .iter()
                .filter(|n| fret_rect_contains(more_tools_item.bounds, n.bounds))
                .map(|n| (n.role, n.label.as_deref(), n.bounds))
                .take(24)
                .collect();
            panic!(
                "missing fret More Tools chevron candidate for {web_name}; sample(role,label,bounds)={sample:?}"
            )
        });

    let chevron_right = chevron.bounds.origin.x.0 + chevron.bounds.size.width.0;
    let actual_chevron_right_inset = more_tools_right - chevron_right;
    assert_close(
        &format!("{web_name} More Tools chevron right inset"),
        actual_chevron_right_inset,
        expected_chevron_right_inset,
        1.0,
    );
}

#[test]
fn web_vs_fret_context_menu_demo_checkbox_indicator_slot_inset_matches_web() {
    assert_context_menu_demo_checkbox_indicator_slot_inset_matches_web_impl("context-menu-demo");
}

fn assert_context_menu_demo_checkbox_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "context-menu-checkbox-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Show Bookmarks"))
        })
        .unwrap_or_else(|| {
            panic!("missing web Show Bookmarks context-menu-checkbox-item node for {web_name}")
        });
    let expected_pad_left = web_css_px(web_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Show Bookmarks paddingLeft for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItemCheckbox
                && n.label.as_deref() == Some("Show Bookmarks")
        })
        .unwrap_or_else(|| panic!("missing fret Show Bookmarks MenuItemCheckbox for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Show Bookmarks for {web_name}"));

    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Show Bookmarks")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Show Bookmarks Text node for {web_name}"));

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Show Bookmarks paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

#[test]
fn web_vs_fret_context_menu_demo_radio_indicator_slot_inset_matches_web() {
    assert_context_menu_demo_radio_indicator_slot_inset_matches_web_impl("context-menu-demo");
}

fn assert_context_menu_demo_radio_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "context-menu-radio-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Pedro Duarte"))
        })
        .unwrap_or_else(|| {
            panic!("missing web Pedro Duarte context-menu-radio-item node for {web_name}")
        });
    let expected_pad_left = web_css_px(web_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Pedro Duarte paddingLeft for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItemRadio && n.label.as_deref() == Some("Pedro Duarte")
        })
        .unwrap_or_else(|| panic!("missing fret Pedro Duarte MenuItemRadio for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Pedro Duarte for {web_name}"));

    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Pedro Duarte")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Pedro Duarte Text node for {web_name}"));

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Pedro Duarte paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

fn assert_context_menu_demo_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["context-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "context-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_context_menu_demo(
                cx,
                open.clone(),
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    for frame in 2..=4 {
        let request_semantics = frame == 4;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            request_semantics,
            |cx| {
                let el = build_context_menu_demo(
                    cx,
                    open.clone(),
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_context_menu_demo_constrained_scroll_state_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let open_point = theme
        .open
        .as_ref()
        .map(|m| m.point)
        .unwrap_or_else(|| panic!("missing web open point for {web_name}"));
    let labels = [
        "Back",
        "Forward",
        "Reload",
        "More Tools",
        "Show Bookmarks",
        "Show Full URLs",
        "Pedro Duarte",
        "Colm Tuite",
    ];
    let expected_first_visible_label = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(&theme, "context-menu-content"),
        &labels,
    )
    .unwrap_or_else(|| panic!("missing web first visible menu item for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{
            ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry, ContextMenuItem,
            ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItemSpec, ContextMenuShortcut,
        };

        ContextMenu::new(open.clone())
            .min_width(Px(208.0))
            .submenu_min_width(Px(176.0))
            .into_element(
                cx,
                |cx| {
                    cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(300.0));
                                layout.size.height = Length::Px(Px(150.0));
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("Right click here")],
                    )
                },
                |cx| {
                    vec![
                        ContextMenuEntry::Item(
                            ContextMenuItem::new("Back")
                                .inset(true)
                                .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                        ),
                        ContextMenuEntry::Item(
                            ContextMenuItem::new("Forward")
                                .inset(true)
                                .disabled(true)
                                .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                        ),
                        ContextMenuEntry::Item(
                            ContextMenuItem::new("Reload")
                                .inset(true)
                                .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                        ),
                        ContextMenuEntry::Item(ContextMenuItem::new("More Tools").inset(true).submenu(
                            vec![
                                ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                                ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                                ContextMenuEntry::Separator,
                                ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                    fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                )),
                            ],
                        )),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                            checked_bookmarks.clone(),
                            "Show Bookmarks",
                        )),
                        ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                            checked_full_urls.clone(),
                            "Show Full URLs",
                        )),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                        ContextMenuEntry::RadioGroup(
                            ContextMenuRadioGroup::new(radio_person.clone())
                                .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                                .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                        ),
                    ]
                },
            )
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(open_point.x), Px(open_point.y)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(open_point.x), Px(open_point.y)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let root_menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .max_by(|a, b| a.bounds.size.height.0.total_cmp(&b.bounds.size.height.0))
        .expect("fret root menu semantics");
    let first_visible =
        fret_first_visible_menu_item_label(&snap, root_menu.bounds, &labels).unwrap_or("<missing>");
    assert_eq!(
        first_visible, expected_first_visible_label,
        "{web_name}: first visible menu item label mismatch"
    );
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo.vp1440x240");
}

#[test]
fn web_vs_fret_context_menu_demo_small_viewport_scroll_state_matches() {
    assert_context_menu_demo_constrained_scroll_state_matches("context-menu-demo.vp1440x320");
}

#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_scroll_state_matches() {
    assert_context_menu_demo_constrained_scroll_state_matches("context-menu-demo.vp1440x240");
}

#[test]
fn web_vs_fret_context_menu_demo_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo");
}

fn build_context_menu_demo_submenu_snapshot(web_name: &str) -> (WebGolden, SemanticsSnapshot) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(true);
    let checked_full_urls: Model<bool> = app.models_mut().insert(false);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    let render = |cx: &mut ElementContext<'_, App>| {
        let el = build_context_menu_demo(
            cx,
            open.clone(),
            checked_bookmarks.clone(),
            checked_full_urls.clone(),
            radio_person.clone(),
        );
        vec![pad_root(cx, Px(0.0), el)]
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render,
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger_button = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("fret trigger button semantics");
    let click_point = Point::new(
        Px(trigger_button.bounds.origin.x.0 + trigger_button.bounds.size.width.0 * 0.5),
        Px(trigger_button.bounds.origin.y.0 + trigger_button.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        render,
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let mut frame: u64 = 3;

    if web_name.contains("submenu-kbd") {
        // Match the web golden extraction script behavior:
        // - `scrollIntoView({ block: "center" })` on the submenu trigger element
        // - focus the trigger and press ArrowRight
        let mut snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        for _ in 0..3 {
            let root_menu = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Menu)
                .expect("fret root menu semantics");
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools")
                })
                .expect("fret submenu trigger semantics (More Tools)");

            let root_center_y = root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5;
            let trigger_center_y = trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5;
            let dy = trigger_center_y - root_center_y;
            if dy.abs() <= 1.0 {
                break;
            }

            let wheel_pos = Point::new(
                Px(root_menu.bounds.origin.x.0 + root_menu.bounds.size.width.0 * 0.5),
                Px(root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5),
            );
            ui.dispatch_event(
                &mut app,
                &mut services,
                &Event::Pointer(PointerEvent::Wheel {
                    pointer_id: fret_core::PointerId::default(),
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-dy)),
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(frame),
                true,
                render,
            );
            frame += 1;
            snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        }

        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
            .expect("fret submenu trigger semantics (More Tools, scrolled)");
        let focus_point = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId::default(),
                position: focus_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId::default(),
                position: focus_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            render,
        );
        frame += 1;

        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let focused = fret_focused_label(&snap);
        assert_eq!(
            focused,
            Some("More Tools"),
            "{web_name}: failed to focus submenu trigger (More Tools); focused={focused:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    } else {
        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let root_menu = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Menu)
            .expect("fret root menu semantics");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
            .expect("fret submenu trigger semantics (More Tools)");
        assert!(
            fret_rect_contains(root_menu.bounds, trigger.bounds),
            "{web_name}: submenu trigger is not visible in root menu panel (expected to open submenu by hover)"
        );

        let hover_point = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: hover_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
        deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);
    }

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick as u64),
            request_semantics,
            render,
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    (web, snap)
}

fn assert_context_menu_demo_submenu_overlay_placement_matches(web_name: &str) {
    let (web, snap) = build_context_menu_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "context-menu-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "context-menu-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("fret submenu trigger semantics (final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let _root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu");
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp1440x240",
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd",
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_hover_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu",
    );
}

fn assert_context_menu_demo_submenu_constrained_menu_content_insets_match(web_name: &str) {
    let (web, snap) = build_context_menu_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_slots = ["context-menu-content", "context-menu-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "{web_name} expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(&format!("{web_name} menu[{i}] height"), *a, *e, 2.0);
    }
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_context_menu_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd-vp1440x240",
    );
}

#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches() {
    assert_tooltip_demo_overlay_placement_matches("tooltip-demo");
}

#[test]
fn web_vs_fret_tooltip_demo_overlay_placement_matches_tiny_viewport() {
    assert_tooltip_demo_overlay_placement_matches("tooltip-demo.vp1440x240");
}

fn assert_tooltip_demo_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for {web_name}");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let trigger_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(trigger_w))
                        .h_px(Px(trigger_h)),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));
            let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(content_w))
                        .h_px(Px(content_h)),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));
            let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
            vec![pad_root(cx, Px(0.0), tooltip)]
        },
    );

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("tooltip trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("Hover")
                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(trigger_w))
                            .h_px(Px(trigger_h)),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));
                let content = fret_ui_shadcn::TooltipContent::new(vec![cx.text("Add to library")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(content_w))
                            .h_px(Px(content_h)),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));
                let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content).into_element(cx);
                vec![pad_root(cx, Px(0.0), tooltip)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("tooltip trigger element id");
    let content_element = content_id_out.get().expect("tooltip content element id");

    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_element).expect("tooltip trigger bounds");
    let portal_bounds =
        bounds_for_element(&mut app, window, content_element).expect("tooltip content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "{web_name} web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );
}

#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches() {
    assert_hover_card_demo_overlay_placement_matches("hover-card-demo");
}

#[test]
fn web_vs_fret_hover_card_demo_overlay_placement_matches_tiny_viewport() {
    assert_hover_card_demo_overlay_placement_matches("hover-card-demo.vp1440x240");
}

fn assert_hover_card_demo_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = find_first(&web.themes["light"].root, &|n| n.tag == "button")
        .or_else(|| find_first(&web.themes["dark"].root, &|n| n.tag == "button"))
        .expect("web trigger (button)");
    let trigger_w = web_trigger.rect.w;
    let trigger_h = web_trigger.rect.h;

    if theme.portals.is_empty() {
        panic!("missing web portals for {web_name}");
    }
    let web_portal_leaf = &theme.portals[0];
    let web_portal = theme.portal_wrappers.get(0).unwrap_or(web_portal_leaf);
    let content_w = web_portal.rect.w;
    let content_h = web_portal.rect.h;

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let trigger_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
    let content_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let trigger_id_out = trigger_id_out.clone();
            let content_id_out = content_id_out.clone();
            let trigger = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(trigger_w))
                        .h_px(Px(trigger_h)),
                )
                .into_element(cx);
            trigger_id_out.set(Some(trigger.id));

            let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(content_w))
                        .h_px(Px(content_h)),
                )
                .into_element(cx);
            content_id_out.set(Some(content.id));

            let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx);

            vec![pad_root(cx, Px(0.0), hover_card)]
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyA,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
        .expect("hover card trigger node");
    ui.set_focus(Some(trigger_node));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let content_id_out = content_id_out.clone();
                let trigger = fret_ui_shadcn::Button::new("@nextjs")
                    .variant(fret_ui_shadcn::ButtonVariant::Link)
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(trigger_w))
                            .h_px(Px(trigger_h)),
                    )
                    .into_element(cx);
                trigger_id_out.set(Some(trigger.id));

                let content = fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")])
                    .refine_layout(
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(Px(content_w))
                            .h_px(Px(content_h)),
                    )
                    .into_element(cx);
                content_id_out.set(Some(content.id));

                let hover_card = fret_ui_shadcn::HoverCard::new(trigger, content)
                    .open_delay_frames(0)
                    .close_delay_frames(0)
                    .into_element(cx);

                vec![pad_root(cx, Px(0.0), hover_card)]
            },
        );
    }

    let trigger_element = trigger_id_out.get().expect("hover card trigger element id");
    let content_element = content_id_out.get().expect("hover card content element id");

    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_element).expect("hover card trigger bounds");
    let portal_bounds =
        bounds_for_element(&mut app, window, content_element).expect("hover card content bounds");

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "{web_name} web trigger={:?} web portal={:?} fret trigger={:?} fret portal={:?}",
            web_trigger.rect, web_portal.rect, trigger_bounds, portal_bounds
        );
    }

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_portal = WebRect {
        x: portal_bounds.origin.x.0,
        y: portal_bounds.origin.y.0,
        w: portal_bounds.size.width.0,
        h: portal_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );
}

fn shadcn_nav_menu_demo_indicator_panel<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(260.0)); // `w-[260px]`
                layout.size.height = Length::Px(Px(36.0)); // `p-2` + `text-sm` (20px line height) + `p-2`
                layout
            },
            ..Default::default()
        },
        move |cx| vec![cx.text("Home content")],
    )
}

#[test]
fn web_vs_fret_navigation_menu_demo_overlay_placement_matches() {
    let web = read_web_golden_open("navigation-menu-demo");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_desktop_panel(cx, model.clone())],
                )])
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_desktop_panel(cx, model.clone())],
                    )])
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, "home",
            )
        },
    )
    .expect("fret nav menu content id");
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    assert_close(
        "navigation-menu-demo main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo content_width",
        fret_content.w,
        web_content.rect.w,
        2.0,
    );
    assert_close(
        "navigation-menu-demo content_height",
        fret_content.h,
        web_content.rect.h,
        2.0,
    );
}

fn web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
    web_name: &str,
    open_value: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "list",
                    "List",
                    vec![shadcn_nav_menu_demo_list_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "simple",
                    "Simple",
                    vec![shadcn_nav_menu_demo_simple_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "with-icon",
                    "With Icon",
                    vec![shadcn_nav_menu_demo_with_icon_panel(cx, model.clone())],
                ),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");
    let click_point = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "list",
                        "List",
                        vec![shadcn_nav_menu_demo_list_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "simple",
                        "Simple",
                        vec![shadcn_nav_menu_demo_simple_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "with-icon",
                        "With Icon",
                        vec![shadcn_nav_menu_demo_with_icon_panel(cx, model.clone())],
                    ),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {open_value}"));
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-after-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx, root_id, open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    let label = format!("{web_name} main_gap");
    assert_close(&label, actual_gap, expected_gap, 1.0);
    let label = format!("{web_name} cross_delta");
    assert_close(&label, actual_cross, expected_cross, 1.5);
    let label = format!("{web_name} trigger_height");
    assert_close(&label, fret_trigger.h, web_trigger.rect.h, 1.0);
    let label = format!("{web_name} content_width");
    assert_close(&label, fret_content.w, web_content.rect.w, 2.0);
    let label = format!("{web_name} content_height");
    assert_close(&label, fret_content.h, web_content.rect.h, 2.0);
}

fn web_vs_fret_navigation_menu_demo_hover_switch_overlay_placement_matches(
    web_name: &str,
    initial_open_value: &str,
    hover_open_value: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .unwrap_or_else(|| {
                find_first(&theme.root, &|n| {
                    n.attrs
                        .get("data-slot")
                        .is_some_and(|v| v.as_str() == "navigation-menu-trigger")
                })
                .expect("web trigger slot=navigation-menu-trigger")
            });
    let web_content =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-content", "open")
            .expect("web content slot=navigation-menu-content state=open");

    let web_side = infer_side(web_trigger.rect, web_content.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_content.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_content.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_panel(cx, model.clone())],
                ),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(false)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-initial-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                initial_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {initial_open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");
    let click_point = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_panel(cx, model.clone())],
                    ),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let hover_trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-hover-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                hover_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {hover_open_value}"));
    let hover_bounds = bounds_for_element(&mut app, window, hover_trigger_id)
        .expect("fret nav menu trigger bounds");
    let hover_point = Point::new(
        Px(hover_bounds.origin.x.0 + hover_bounds.size.width.0 * 0.5),
        Px(hover_bounds.origin.y.0 + hover_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: hover_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(hover_point.x.0 + 1.0), hover_point.y),
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_HOVER_SWITCH")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        let selected = app.models_mut().read(&model, |v| v.clone()).ok().flatten();
        eprintln!(
            "{web_name} after hover move selected={:?}",
            selected.as_deref()
        );
    }

    // The upstream demo relies on a hover trigger + viewport size transition that is effectively
    // `duration-300` in the shadcn recipe. Our web golden is captured after a `wait=300`, so we
    // advance enough frames to reach the same steady state before asserting geometry.
    let hover_settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_300 + 2;
    for tick in 0..hover_settle_frames {
        let request_semantics = tick + 1 == hover_settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2000 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new("home", "Home", vec![cx.text("Home")]),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_panel(cx, model.clone())],
                    ),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    if debug {
        let selected = app.models_mut().read(&model, |v| v.clone()).ok().flatten();
        eprintln!(
            "{web_name} after hover settle selected={:?}",
            selected.as_deref()
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-content-query-hover-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx,
                root_id,
                hover_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu content id for {hover_open_value}"));
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-final-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                hover_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {hover_open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_content);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_content);

    let label = format!("{web_name} main_gap");
    assert_close(&label, actual_gap, expected_gap, 1.0);
    let label = format!("{web_name} cross_delta");
    assert_close(&label, actual_cross, expected_cross, 1.5);
    let label = format!("{web_name} trigger_height");
    assert_close(&label, fret_trigger.h, web_trigger.rect.h, 1.0);
    let label = format!("{web_name} content_width");
    assert_close(&label, fret_content.w, web_content.rect.w, 2.0);
    let label = format!("{web_name} content_height");
    assert_close(&label, fret_content.h, web_content.rect.h, 2.0);
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.components-vp1440x320",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_list_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list",
        "list",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_list_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.list-vp1440x320",
        "list",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_simple_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple-vp1440x320",
        "simple",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_simple_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.simple",
        "simple",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_small_viewport_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon-vp1440x320",
        "with-icon",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_with_icon_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
        "navigation-menu-demo.with-icon",
        "with-icon",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_hover_home_to_components_overlay_placement_matches() {
    web_vs_fret_navigation_menu_demo_hover_switch_overlay_placement_matches(
        "navigation-menu-demo.home-then-hover-components",
        "home",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_indicator_geometry_matches_web() {
    let web = read_web_golden_open("navigation-menu-demo-indicator");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_indicator =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-indicator", "visible")
            .expect("web indicator slot=navigation-menu-indicator state=visible");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_diamond = web_indicator
        .children
        .iter()
        .find(|n| web_css_px(n, "width").is_some_and(|v| (v - 8.0).abs() <= 0.01))
        .unwrap_or_else(|| panic!("missing web navigation-menu indicator diamond node"));
    let web_diamond_unrotated = web_unrotated_rect_for_rotated_square(web_diamond);

    let expected_track = web_indicator.rect;

    let expected_diamond = WebRect {
        x: web_diamond_unrotated.x,
        y: web_diamond_unrotated.y,
        w: web_diamond_unrotated.w,
        h: web_diamond_unrotated.h,
    };

    assert_close(
        "navigation-menu-demo-indicator web trigger_x == indicator_x",
        web_trigger.rect.x,
        expected_track.x,
        0.5,
    );
    assert_close(
        "navigation-menu-demo-indicator web trigger_w ~= indicator_w",
        web_trigger.rect.w,
        expected_track.w,
        1.0,
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("home")));
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 1..=(1 + settle_frames) {
        let request_semantics = frame == 1 + settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame as u64),
            request_semantics,
            |cx| {
                let items = vec![fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_indicator_panel(cx)],
                )];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(true)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-viewport-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let indicator_track_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-track-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_track_id(
                cx, root_id,
            )
        },
    )
    .expect("fret nav menu indicator track id");
    let indicator_track_bounds = bounds_for_element(&mut app, window, indicator_track_id)
        .expect("fret nav menu indicator track bounds");

    let indicator_diamond_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-indicator-diamond-id",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_indicator_diamond_id(
                cx, root_id,
            )
        },
    )
    .expect("fret nav menu indicator diamond id");
    let indicator_diamond_bounds = bounds_for_element(&mut app, window, indicator_diamond_id)
        .expect("fret nav menu indicator diamond bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    assert_close(
        "navigation-menu-demo-indicator track_x",
        indicator_track_bounds.origin.x.0,
        expected_track.x,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator track_y",
        indicator_track_bounds.origin.y.0,
        expected_track.y,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator track_w",
        indicator_track_bounds.size.width.0,
        expected_track.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator track_h",
        indicator_track_bounds.size.height.0,
        expected_track.h,
        0.5,
    );

    assert_close(
        "navigation-menu-demo-indicator trigger_x == track_x",
        trigger.bounds.origin.x.0,
        indicator_track_bounds.origin.x.0,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator trigger_w == track_w",
        trigger.bounds.size.width.0,
        indicator_track_bounds.size.width.0,
        1.5,
    );

    let web_gap_to_viewport = web_viewport.rect.y - (expected_track.y + expected_track.h);
    let fret_gap_to_viewport = viewport_bounds.origin.y.0
        - (indicator_track_bounds.origin.y.0 + indicator_track_bounds.size.height.0);
    assert_close(
        "navigation-menu-demo-indicator gap_to_viewport",
        fret_gap_to_viewport,
        web_gap_to_viewport,
        1.0,
    );
    assert_close(
        "navigation-menu-demo-indicator viewport_w",
        viewport_bounds.size.width.0,
        web_viewport.rect.w,
        2.0,
    );
    assert_close(
        "navigation-menu-demo-indicator viewport_h",
        viewport_bounds.size.height.0,
        web_viewport.rect.h,
        2.0,
    );

    let actual_diamond_left =
        indicator_diamond_bounds.origin.x.0 - indicator_track_bounds.origin.x.0;
    let actual_diamond_top =
        indicator_diamond_bounds.origin.y.0 - indicator_track_bounds.origin.y.0;
    assert_close(
        "navigation-menu-demo-indicator diamond_left",
        actual_diamond_left,
        expected_diamond.x - expected_track.x,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_top",
        actual_diamond_top,
        expected_diamond.y - expected_track.y,
        1.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_w",
        indicator_diamond_bounds.size.width.0,
        expected_diamond.w,
        0.5,
    );
    assert_close(
        "navigation-menu-demo-indicator diamond_h",
        indicator_diamond_bounds.size.height.0,
        expected_diamond.h,
        0.5,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_height_matches() {
    let web = read_web_golden_open("navigation-menu-demo.home-mobile");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_mobile_panel(
                        cx,
                        model.clone(),
                    )],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_MOBILE")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "nav-menu home-mobile web viewport={:?} web trigger={:?} fret viewport={:?} fret trigger={:?}",
            web_viewport.rect, web_trigger.rect, fret_viewport, fret_trigger
        );
    }

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    assert_close(
        "navigation-menu-demo.home-mobile main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo.home-mobile cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_height",
        fret_viewport.h,
        web_viewport.rect.h,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile viewport_width",
        fret_viewport.w,
        web_viewport.rect.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile trigger_height",
        fret_trigger.h,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.home-mobile",
        "home",
    );
}

fn assert_navigation_menu_demo_mobile_viewport_geometry_matches(
    web_name: &str,
    trigger_value: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_mobile_panel(
                        cx,
                        model.clone(),
                    )],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                trigger_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret nav menu trigger id (value={trigger_value})"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");
    let click_point = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-after-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                trigger_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret nav menu trigger id (value={trigger_value})"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_MOBILE")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "nav-menu {web_name} web viewport={:?} web trigger={:?} fret viewport={:?} fret trigger={:?}",
            web_viewport.rect, web_trigger.rect, fret_viewport, fret_trigger
        );
    }

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    let label = format!("{web_name} main_gap");
    assert_close(&label, actual_gap, expected_gap, 1.0);
    let label = format!("{web_name} cross_delta");
    assert_close(&label, actual_cross, expected_cross, 1.5);
    let label = format!("{web_name} viewport_height");
    assert_close(&label, fret_viewport.h, web_viewport.rect.h, 1.5);
    let label = format!("{web_name} viewport_width");
    assert_close(&label, fret_viewport.w, web_viewport.rect.w, 1.5);
    let label = format!("{web_name} trigger_height");
    assert_close(&label, fret_trigger.h, web_trigger.rect.h, 1.0);
}

fn assert_navigation_menu_demo_mobile_viewport_insets_match(web_name: &str, trigger_value: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");
    let web_content = find_first(&theme.root, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == "navigation-menu-content")
            && web_rect_contains(web_viewport.rect, n.rect)
    })
    .expect("web content slot=navigation-menu-content within viewport");

    let expected_left = web_content.rect.x - web_viewport.rect.x;
    let expected_top = web_content.rect.y - web_viewport.rect.y;
    let expected_right = rect_right(web_viewport.rect) - rect_right(web_content.rect);
    let expected_bottom = rect_bottom(web_viewport.rect) - rect_bottom(web_content.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_mobile_panel(
                        cx,
                        model.clone(),
                    )],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let label = match trigger_value {
        "home" => "Home",
        "components" => "Components",
        other => other,
    };
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("fret trigger semantics ({label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query-insets",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let content_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-content-query-insets",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_content_id(
                cx,
                root_id,
                trigger_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret nav menu content id for {trigger_value}"));
    let content_bounds =
        bounds_for_element(&mut app, window, content_id).expect("fret nav menu content bounds");

    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };
    let fret_content = WebRect {
        x: content_bounds.origin.x.0,
        y: content_bounds.origin.y.0,
        w: content_bounds.size.width.0,
        h: content_bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_MOBILE")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        eprintln!(
            "nav-menu insets {web_name} web viewport={:?} web content={:?} fret viewport={:?} fret content={:?}",
            web_viewport.rect, web_content.rect, fret_viewport, fret_content
        );
    }

    let actual_left = fret_content.x - fret_viewport.x;
    let actual_top = fret_content.y - fret_viewport.y;
    let actual_right = rect_right(fret_viewport) - rect_right(fret_content);
    let actual_bottom = rect_bottom(fret_viewport) - rect_bottom(fret_content);

    assert_close(
        &format!("{web_name} viewport_inset_left"),
        actual_left,
        expected_left,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_inset_top"),
        actual_top,
        expected_top,
        1.0,
    );
    assert_close(
        &format!("{web_name} viewport_inset_right"),
        actual_right,
        expected_right,
        2.0,
    );
    assert_close(
        &format!("{web_name} viewport_inset_bottom"),
        actual_bottom,
        expected_bottom,
        2.0,
    );
}

fn assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
    web_name: &str,
    initial_open_value: &str,
    hover_open_value: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new(
                    "components",
                    "Components",
                    vec![shadcn_nav_menu_demo_components_mobile_panel(
                        cx,
                        model.clone(),
                    )],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let root_id = root_id_out.get().expect("navigation menu root id");
    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-initial-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                initial_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {initial_open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");
    let click_point = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let hover_trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-hover-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                hover_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {hover_open_value}"));
    let hover_bounds = bounds_for_element(&mut app, window, hover_trigger_id)
        .expect("fret nav menu trigger bounds");
    let hover_point = Point::new(
        Px(hover_bounds.origin.x.0 + hover_bounds.size.width.0 * 0.5),
        Px(hover_bounds.origin.y.0 + hover_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: hover_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(hover_point.x.0 + 1.0), hover_point.y),
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let debug = std::env::var("FRET_DEBUG_NAV_MENU_HOVER_SWITCH")
        .ok()
        .is_some_and(|v| v == "1");
    if debug {
        let selected = app.models_mut().read(&model, |v| v.clone()).ok().flatten();
        eprintln!(
            "{web_name} after hover move selected={:?}",
            selected.as_deref()
        );
    }

    // The upstream demo relies on a hover trigger + viewport size transition that is effectively
    // `duration-300` in the shadcn recipe. Our web golden is captured after a `wait=300`, so we
    // advance enough frames to reach the same steady state before asserting geometry.
    let hover_settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_300 + 2;
    for tick in 0..hover_settle_frames {
        let request_semantics = tick + 1 == hover_settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2000 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "components",
                        "Components",
                        vec![shadcn_nav_menu_demo_components_mobile_panel(
                            cx,
                            model.clone(),
                        )],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    if debug {
        let selected = app.models_mut().read(&model, |v| v.clone()).ok().flatten();
        eprintln!(
            "{web_name} after hover settle selected={:?}",
            selected.as_deref()
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query-hover-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let trigger_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-trigger-query-final-open",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_trigger_id(
                cx,
                root_id,
                hover_open_value,
            )
        },
    )
    .unwrap_or_else(|| panic!("fret navigation-menu trigger id for {hover_open_value}"));
    let trigger_bounds =
        bounds_for_element(&mut app, window, trigger_id).expect("fret nav menu trigger bounds");

    let fret_trigger = WebRect {
        x: trigger_bounds.origin.x.0,
        y: trigger_bounds.origin.y.0,
        w: trigger_bounds.size.width.0,
        h: trigger_bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    let label = format!("{web_name} main_gap");
    assert_close(&label, actual_gap, expected_gap, 1.0);
    let label = format!("{web_name} cross_delta");
    assert_close(&label, actual_cross, expected_cross, 1.5);
    let label = format!("{web_name} viewport_height");
    assert_close(&label, fret_viewport.h, web_viewport.rect.h, 1.5);
    let label = format!("{web_name} viewport_width");
    assert_close(&label, fret_viewport.w, web_viewport.rect.w, 1.5);
    let label = format!("{web_name} trigger_height");
    assert_close(&label, fret_trigger.h, web_trigger.rect.h, 1.0);
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_height_matches() {
    let web = read_web_golden_open("navigation-menu-demo.home-mobile-vp375x320");
    let theme = web_theme(&web);

    let web_trigger =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-trigger", "open")
            .expect("web trigger slot=navigation-menu-trigger state=open");
    let web_viewport =
        web_find_by_data_slot_and_state(&theme.root, "navigation-menu-viewport", "open")
            .expect("web viewport slot=navigation-menu-viewport state=open");

    let web_side = infer_side(web_trigger.rect, web_viewport.rect);
    let web_align = infer_align(web_side, web_trigger.rect, web_viewport.rect);
    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_viewport.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_viewport.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let root_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let items = vec![
                fret_ui_shadcn::NavigationMenuItem::new(
                    "home",
                    "Home",
                    vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                ),
                fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
            ];

            let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                .viewport(true)
                .indicator(false)
                .items(items)
                .into_element(cx);
            root_id_out.set(Some(el.id));
            vec![pad_root(cx, Px(0.0), el)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let items = vec![
                    fret_ui_shadcn::NavigationMenuItem::new(
                        "home",
                        "Home",
                        vec![shadcn_nav_menu_demo_home_panel(cx, model.clone())],
                    ),
                    fret_ui_shadcn::NavigationMenuItem::new("components", "Components", Vec::new()),
                    fret_ui_shadcn::NavigationMenuItem::new("docs", "Docs", Vec::new()),
                ];

                let el = fret_ui_shadcn::NavigationMenu::new(model.clone())
                    .viewport(true)
                    .indicator(false)
                    .items(items)
                    .into_element(cx);
                root_id_out.set(Some(el.id));
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let root_id = root_id_out.get().expect("navigation menu root id");
    let viewport_id = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "web-vs-fret-nav-menu-viewport-query-small",
        |cx| {
            fret_ui_kit::primitives::navigation_menu::navigation_menu_viewport_panel_id(cx, root_id)
        },
    )
    .expect("fret nav menu viewport panel id");
    let viewport_bounds =
        bounds_for_element(&mut app, window, viewport_id).expect("fret nav menu viewport bounds");

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Home"))
        .expect("fret trigger semantics (Home)");

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };
    let fret_viewport = WebRect {
        x: viewport_bounds.origin.x.0,
        y: viewport_bounds.origin.y.0,
        w: viewport_bounds.size.width.0,
        h: viewport_bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_viewport);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_viewport);

    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 main_gap",
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 viewport_height",
        fret_viewport.h,
        web_viewport.rect.h,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 viewport_width",
        fret_viewport.w,
        web_viewport.rect.w,
        1.5,
    );
    assert_close(
        "navigation-menu-demo.home-mobile-vp375x320 trigger_height",
        fret_trigger.h,
        web_trigger.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.home-mobile-vp375x320",
        "home",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.components-mobile",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_small_viewport_height_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_matches(
        "navigation-menu-demo.components-mobile-vp375x320",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.components-mobile",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_components_mobile_small_viewport_insets_match() {
    assert_navigation_menu_demo_mobile_viewport_insets_match(
        "navigation-menu-demo.components-mobile-vp375x320",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_hover_to_components_viewport_geometry_matches() {
    assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
        "navigation-menu-demo.home-mobile-then-hover-components",
        "home",
        "components",
    );
}

#[test]
fn web_vs_fret_navigation_menu_demo_home_mobile_small_viewport_hover_to_components_viewport_geometry_matches()
 {
    assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
        "navigation-menu-demo.home-mobile-vp375x320-then-hover-components",
        "home",
        "components",
    );
}

#[test]
fn web_vs_fret_menubar_demo_overlay_placement_matches() {
    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let menu_label = web_trigger.text.as_deref().unwrap_or("File");

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!("menubar-demo fret Menu candidates: {}", candidates.len());
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "menubar-demo web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "menubar-demo fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close("menubar-demo main_gap", actual_gap, expected_gap, 1.0);
    assert_close(
        "menubar-demo cross_delta",
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        "menubar-demo portal_w",
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        "menubar-demo portal_h",
        fret_portal.h,
        expected_portal_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_view_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.view");
}

#[test]
fn web_vs_fret_menubar_demo_profiles_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.profiles");
}

#[test]
fn web_vs_fret_menubar_demo_view_checkbox_indicator_slot_inset_matches_web() {
    assert_menubar_demo_checkbox_indicator_slot_inset_matches_web_impl("menubar-demo.view");
}

fn assert_menubar_demo_checkbox_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "menubar-checkbox-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("Always Show Bookmarks Bar"))
        })
        .unwrap_or_else(|| {
            panic!(
                "missing web Always Show Bookmarks Bar menubar-checkbox-item node for {web_name}"
            )
        });
    let expected_pad_left = web_css_px(web_item, "paddingLeft").unwrap_or_else(|| {
        panic!("missing web Always Show Bookmarks Bar paddingLeft for {web_name}")
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let view_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("View"))
        .expect("fret menubar trigger semantics (View)");
    let click_point = Point::new(
        Px(view_trigger.bounds.origin.x.0 + view_trigger.bounds.size.width.0 * 0.5),
        Px(view_trigger.bounds.origin.y.0 + view_trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItemCheckbox
                && n.label.as_deref() == Some("Always Show Bookmarks Bar")
        })
        .unwrap_or_else(|| {
            panic!("missing fret Always Show Bookmarks Bar MenuItemCheckbox for {web_name}")
        });
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| {
            panic!("missing fret Menu containing Always Show Bookmarks Bar for {web_name}")
        });

    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Always Show Bookmarks Bar")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| {
            panic!("missing fret Always Show Bookmarks Bar Text node for {web_name}")
        });

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Always Show Bookmarks Bar paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

#[test]
fn web_vs_fret_menubar_demo_profiles_radio_indicator_slot_inset_matches_web() {
    assert_menubar_demo_radio_indicator_slot_inset_matches_web_impl("menubar-demo.profiles");
}

fn assert_menubar_demo_radio_indicator_slot_inset_matches_web_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_item = web_portal_nodes_by_data_slot(&theme, "menubar-radio-item")
        .into_iter()
        .find(|n| n.text.as_deref().is_some_and(|text| text == "Andy"))
        .unwrap_or_else(|| panic!("missing web Andy menubar-radio-item node for {web_name}"));
    let expected_pad_left = web_css_px(web_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web Andy paddingLeft for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let profiles_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Profiles"))
        .expect("fret menubar trigger semantics (Profiles)");
    let click_point = Point::new(
        Px(profiles_trigger.bounds.origin.x.0 + profiles_trigger.bounds.size.width.0 * 0.5),
        Px(profiles_trigger.bounds.origin.y.0 + profiles_trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let item = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItemRadio && n.label.as_deref() == Some("Andy"))
        .unwrap_or_else(|| panic!("missing fret Andy MenuItemRadio for {web_name}"));
    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing Andy for {web_name}"));

    let label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("Andy")
                && fret_rect_contains(item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Andy Text node for {web_name}"));

    let actual_pad_left = label_text.bounds.origin.x.0 - item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} Andy paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    assert!(fret_rect_contains(menu.bounds, item.bounds));
}

fn assert_menubar_demo_constrained_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });

    let menu_label = web_trigger.text.as_deref().unwrap_or("File");

    let web_portal_index = theme
        .portals
        .iter()
        .position(|n| n.attrs.get("role").is_some_and(|v| v == "menu"))
        .expect("web portal role=menu");
    let web_portal_leaf = &theme.portals[web_portal_index];
    let web_portal = theme
        .portal_wrappers
        .get(web_portal_index)
        .unwrap_or(web_portal_leaf);

    let web_side = find_attr_in_subtree(web_portal_leaf, "data-side")
        .or_else(|| find_attr_in_subtree(web_portal, "data-side"))
        .and_then(parse_side)
        .unwrap_or_else(|| infer_side(web_trigger.rect, web_portal.rect));
    let web_align = find_attr_in_subtree(web_portal_leaf, "data-align")
        .or_else(|| find_attr_in_subtree(web_portal, "data-align"))
        .and_then(parse_align)
        .unwrap_or_else(|| infer_align(web_side, web_trigger.rect, web_portal.rect));

    let expected_gap = rect_main_gap(web_side, web_trigger.rect, web_portal.rect);
    let expected_cross = rect_cross_delta(web_side, web_align, web_trigger.rect, web_portal.rect);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));

    let expected_portal_w = web_portal.rect.w;
    let expected_portal_h = web_portal.rect.h;

    let fret_trigger = WebRect {
        x: trigger.bounds.origin.x.0,
        y: trigger.bounds.origin.y.0,
        w: trigger.bounds.size.width.0,
        h: trigger.bounds.size.height.0,
    };

    let debug = std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
        .ok()
        .is_some_and(|v| v == "1");

    let portal = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .min_by(|a, b| {
            let rect_a = WebRect {
                x: a.bounds.origin.x.0,
                y: a.bounds.origin.y.0,
                w: a.bounds.size.width.0,
                h: a.bounds.size.height.0,
            };
            let rect_b = WebRect {
                x: b.bounds.origin.x.0,
                y: b.bounds.origin.y.0,
                w: b.bounds.size.width.0,
                h: b.bounds.size.height.0,
            };

            let score_for = |r: WebRect| {
                let gap = rect_main_gap(web_side, fret_trigger, r);
                let cross = rect_cross_delta(web_side, web_align, fret_trigger, r);
                let size = (r.w - expected_portal_w).abs() + (r.h - expected_portal_h).abs();
                (gap - expected_gap).abs() + (cross - expected_cross).abs() + 0.05 * size
            };

            let score_a = score_for(rect_a);
            let score_b = score_for(rect_b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("fret menubar portal semantics (Menu)");

    if debug {
        let candidates: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .collect();
        eprintln!("{web_name} fret Menu candidates: {}", candidates.len());
        for (idx, n) in candidates.iter().enumerate().take(8) {
            eprintln!("  [{idx}] bounds={:?} label={:?}", n.bounds, n.label);
        }
        eprintln!(
            "{web_name} web trigger={:?} web portal={:?}\n  fret trigger={:?}\n  selected portal={:?}",
            web_trigger.rect, web_portal.rect, fret_trigger, portal.bounds
        );
        eprintln!(
            "{web_name} fret trigger flags={:?} root_count={} node_count={}",
            trigger.flags,
            snap.roots.len(),
            snap.nodes.len()
        );
    }

    let fret_portal = WebRect {
        x: portal.bounds.origin.x.0,
        y: portal.bounds.origin.y.0,
        w: portal.bounds.size.width.0,
        h: portal.bounds.size.height.0,
    };

    let actual_gap = rect_main_gap(web_side, fret_trigger, fret_portal);
    let actual_cross = rect_cross_delta(web_side, web_align, fret_trigger, fret_portal);

    assert_close(
        &format!("{web_name} main_gap"),
        actual_gap,
        expected_gap,
        1.0,
    );
    assert_close(
        &format!("{web_name} cross_delta"),
        actual_cross,
        expected_cross,
        1.5,
    );
    assert_close(
        &format!("{web_name} portal_w"),
        fret_portal.w,
        expected_portal_w,
        2.0,
    );
    assert_close(
        &format!("{web_name} portal_h"),
        fret_portal.h,
        expected_portal_h,
        2.0,
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_constrained_overlay_placement_matches("menubar-demo.vp1440x240");
}

fn assert_menubar_demo_constrained_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });
    let menu_label = web_trigger.text.as_deref().unwrap_or("File");
    let expected_hs = web_portal_slot_heights(
        &theme,
        &[
            "menubar-item",
            "menubar-checkbox-item",
            "menubar-radio-item",
            "menubar-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.vp1440x240");
}

#[test]
fn web_vs_fret_menubar_demo_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo");
}

#[test]
fn web_vs_fret_menubar_demo_view_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.view");
}

#[test]
fn web_vs_fret_menubar_demo_profiles_menu_item_height_matches() {
    assert_menubar_demo_constrained_menu_item_height_matches("menubar-demo.profiles");
}

#[test]
fn web_vs_fret_menubar_demo_item_padding_and_shortcut_match() {
    assert_menubar_demo_item_padding_and_shortcut_match_impl("menubar-demo");
}

fn assert_menubar_demo_item_padding_and_shortcut_match_impl(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_new_tab_item = web_portal_nodes_by_data_slot(&theme, "menubar-item")
        .into_iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|text| text.starts_with("New Tab"))
        })
        .unwrap_or_else(|| panic!("missing web New Tab menubar-item node for {web_name}"));

    let expected_pad_left = web_css_px(web_new_tab_item, "paddingLeft")
        .unwrap_or_else(|| panic!("missing web New Tab paddingLeft for {web_name}"));
    let expected_pad_right = web_css_px(web_new_tab_item, "paddingRight")
        .unwrap_or_else(|| panic!("missing web New Tab paddingRight for {web_name}"));
    let expected_gap = web_css_px(web_new_tab_item, "gap")
        .unwrap_or_else(|| panic!("missing web New Tab gap for {web_name}"));
    assert_close(
        &format!("{web_name} web New Tab gap px"),
        expected_gap,
        8.0,
        0.1,
    );

    let web_new_tab_shortcut = find_first(web_new_tab_item, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v.as_str() == "menubar-shortcut")
    })
    .unwrap_or_else(|| panic!("missing web New Tab menubar-shortcut node for {web_name}"));
    let expected_shortcut_right_inset =
        rect_right(web_new_tab_item.rect) - rect_right(web_new_tab_shortcut.rect);
    assert_close(
        &format!("{web_name} web New Tab shortcut right inset == paddingRight"),
        expected_shortcut_right_inset,
        expected_pad_right,
        0.25,
    );

    let web_sub_trigger = web_portal_node_by_data_slot(&theme, "menubar-sub-trigger");
    let web_sub_trigger_pad_right = web_css_px(web_sub_trigger, "paddingRight")
        .unwrap_or_else(|| panic!("missing web menubar-sub-trigger paddingRight for {web_name}"));
    let web_sub_trigger_chevron = find_first(web_sub_trigger, &|n| n.tag == "svg")
        .unwrap_or_else(|| panic!("missing web menubar-sub-trigger chevron svg for {web_name}"));
    let expected_chevron_right_inset =
        rect_right(web_sub_trigger.rect) - rect_right(web_sub_trigger_chevron.rect);
    assert_close(
        &format!("{web_name} web Share chevron right inset == paddingRight"),
        expected_chevron_right_inset,
        web_sub_trigger_pad_right,
        0.25,
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let new_tab_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id.as_deref() == Some("menubar.file.new_tab")
                && n.label.as_deref() == Some("New Tab")
        })
        .unwrap_or_else(|| panic!("missing fret New Tab MenuItem semantics for {web_name}"));

    let menu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|menu| fret_rect_contains(menu.bounds, new_tab_item.bounds))
        .unwrap_or_else(|| panic!("missing fret Menu containing New Tab item for {web_name}"));

    let new_tab_label_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("New Tab")
                && fret_rect_contains(new_tab_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret New Tab Text semantics for {web_name}"));
    let new_tab_shortcut_text = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::Text
                && n.label.as_deref() == Some("⌘T")
                && fret_rect_contains(new_tab_item.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret New Tab shortcut Text semantics for {web_name}"));

    let actual_pad_left = new_tab_label_text.bounds.origin.x.0 - new_tab_item.bounds.origin.x.0;
    assert_close(
        &format!("{web_name} New Tab paddingLeft"),
        actual_pad_left,
        expected_pad_left,
        1.5,
    );

    let new_tab_right = new_tab_item.bounds.origin.x.0 + new_tab_item.bounds.size.width.0;
    let shortcut_right =
        new_tab_shortcut_text.bounds.origin.x.0 + new_tab_shortcut_text.bounds.size.width.0;
    let actual_shortcut_right_inset = new_tab_right - shortcut_right;
    assert_close(
        &format!("{web_name} New Tab shortcut right inset"),
        actual_shortcut_right_inset,
        expected_shortcut_right_inset,
        1.0,
    );

    let share_item = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem
                && n.test_id.as_deref() == Some("menubar.file.share")
                && n.label.as_deref() == Some("Share")
                && fret_rect_contains(menu.bounds, n.bounds)
        })
        .unwrap_or_else(|| panic!("missing fret Share MenuItem semantics for {web_name}"));

    let share_right = share_item.bounds.origin.x.0 + share_item.bounds.size.width.0;
    let chevron_candidates: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| fret_rect_contains(share_item.bounds, n.bounds))
        .filter(|n| {
            (n.bounds.size.width.0 - 16.0).abs() <= 1.0
                && (n.bounds.size.height.0 - 16.0).abs() <= 1.0
        })
        .filter(|n| n.bounds.origin.x.0 >= share_right - 48.0)
        .collect();
    let chevron = chevron_candidates
        .into_iter()
        .max_by(|a, b| {
            let right_a = a.bounds.origin.x.0 + a.bounds.size.width.0;
            let right_b = b.bounds.origin.x.0 + b.bounds.size.width.0;
            right_a.total_cmp(&right_b)
        })
        .unwrap_or_else(|| {
            let sample: Vec<_> = snap
                .nodes
                .iter()
                .filter(|n| fret_rect_contains(share_item.bounds, n.bounds))
                .map(|n| (n.role, n.label.as_deref(), n.bounds))
                .take(24)
                .collect();
            panic!(
                "missing fret Share chevron candidate for {web_name}; sample(role,label,bounds)={sample:?}"
            )
        });

    let chevron_right = chevron.bounds.origin.x.0 + chevron.bounds.size.width.0;
    let actual_chevron_right_inset = share_right - chevron_right;
    assert_close(
        &format!("{web_name} Share chevron right inset"),
        actual_chevron_right_inset,
        expected_chevron_right_inset,
        1.0,
    );
}

fn assert_menubar_demo_constrained_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_trigger = web_find_by_data_slot_and_state(&theme.root, "menubar-trigger", "open")
        .unwrap_or_else(|| {
            find_first(&theme.root, &|n| {
                n.attrs
                    .get("data-slot")
                    .is_some_and(|v| v.as_str() == "menubar-trigger")
            })
            .expect("web trigger slot=menubar-trigger")
        });
    let menu_label = web_trigger.text.as_deref().unwrap_or("File");
    let expected = web_menu_content_insets_for_slots(&theme, &["menubar-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "menubar-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(menu_label))
        .unwrap_or_else(|| panic!("fret menubar trigger semantics ({menu_label})"));
    let click_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn assert_menubar_demo_constrained_scroll_state_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_first_visible_label = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(&theme, "menubar-content"),
        &[
            "New Tab",
            "New Window",
            "New Incognito Window",
            "Share",
            "Print...",
        ],
    )
    .unwrap_or_else(|| panic!("missing web first visible menu item for {web_name}"));

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);

    let labels = [
        "New Tab",
        "New Window",
        "New Incognito Window",
        "Share",
        "Print...",
    ];

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            let menubar = build_menubar_demo(
                cx,
                view_bookmarks_bar.clone(),
                view_full_urls.clone(),
                profile_value.clone(),
            );
            vec![pad_root(cx, Px(0.0), menubar)]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let file_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(file_trigger.bounds.origin.x.0 + file_trigger.bounds.size.width.0 * 0.5),
        Px(file_trigger.bounds.origin.y.0 + file_trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let menubar = build_menubar_demo(
                    cx,
                    view_bookmarks_bar.clone(),
                    view_full_urls.clone(),
                    profile_value.clone(),
                );
                vec![pad_root(cx, Px(0.0), menubar)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let root_menu = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("fret root menu semantics");
    let first_visible =
        fret_first_visible_menu_item_label(&snap, root_menu.bounds, &labels).unwrap_or("<missing>");
    assert_eq!(
        first_visible, expected_first_visible_label,
        "{web_name}: first visible menu item label mismatch"
    );
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.vp1440x240");
}

#[test]
fn web_vs_fret_menubar_demo_small_viewport_scroll_state_matches() {
    assert_menubar_demo_constrained_scroll_state_matches("menubar-demo.vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_scroll_state_matches() {
    assert_menubar_demo_constrained_scroll_state_matches("menubar-demo.vp1440x240");
}

#[test]
fn web_vs_fret_menubar_demo_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo");
}

#[test]
fn web_vs_fret_menubar_demo_view_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.view");
}

#[test]
fn web_vs_fret_menubar_demo_profiles_menu_content_insets_match() {
    assert_menubar_demo_constrained_menu_content_insets_match("menubar-demo.profiles");
}

fn assert_menubar_demo_submenu_overlay_placement_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_sub_menu = web_portal_node_by_data_slot(theme, "menubar-sub-content");
    let web_sub_trigger = web_portal_node_by_data_slot(theme, "menubar-sub-trigger");

    let expected_dx = web_sub_menu.rect.x - rect_right(web_sub_trigger.rect);
    let expected_dy = web_sub_menu.rect.y - web_sub_trigger.rect.y;
    let expected_w = web_sub_menu.rect.w;
    let expected_h = web_sub_menu.rect.h;

    let (_, snap) = build_menubar_demo_submenu_snapshot(web_name);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("fret submenu trigger semantics (Share, final)");

    let menus: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .collect();
    assert!(
        menus.len() >= 2,
        "expected at least 2 menu panels after opening submenu; got {}",
        menus.len()
    );

    let _root_menu = menus
        .iter()
        .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
        .expect("root menu contains sub-trigger");
    let submenu = menus
        .iter()
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual_dx =
        submenu.bounds.origin.x.0 - (trigger.bounds.origin.x.0 + trigger.bounds.size.width.0);
    let actual_dy = submenu.bounds.origin.y.0 - trigger.bounds.origin.y.0;
    let actual_w = submenu.bounds.size.width.0;
    let actual_h = submenu.bounds.size.height.0;

    assert_close(
        &format!("{web_name} submenu dx"),
        actual_dx,
        expected_dx,
        2.0,
    );
    assert_close(
        &format!("{web_name} submenu dy"),
        actual_dy,
        expected_dy,
        2.0,
    );
    assert_close(&format!("{web_name} submenu w"), actual_w, expected_w, 2.0);
    assert_close(&format!("{web_name} submenu h"), actual_h, expected_h, 2.0);
}

#[test]
fn web_vs_fret_menubar_demo_submenu_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_hover_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_menubar_demo_submenu_overlay_placement_matches("menubar-demo.submenu-kbd-vp1440x240");
}

fn build_menubar_demo_submenu_snapshot(web_name: &str) -> (WebGolden, SemanticsSnapshot) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);
    let view_bookmarks_bar: Model<bool> = app.models_mut().insert(false);
    let view_full_urls: Model<bool> = app.models_mut().insert(true);
    let profile_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("benoit")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let render = |cx: &mut ElementContext<'_, App>| {
        let menubar = build_menubar_demo(
            cx,
            view_bookmarks_bar.clone(),
            view_full_urls.clone(),
            profile_value.clone(),
        );
        vec![pad_root(cx, Px(0.0), menubar)]
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        render,
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let file_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("fret menubar trigger semantics (File)");
    let click_point = Point::new(
        Px(file_trigger.bounds.origin.x.0 + file_trigger.bounds.size.width.0 * 0.5),
        Px(file_trigger.bounds.origin.y.0 + file_trigger.bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: click_point,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    let mut frame: u64 = 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick),
            request_semantics,
            render,
        );
    }
    frame += settle_frames;

    if web_name.contains("submenu-kbd") {
        // Match the web golden extraction script behavior:
        // - `scrollIntoView({ block: "center" })` on the submenu trigger element
        // - focus the trigger and press ArrowRight
        let mut snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        for _ in 0..3 {
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
                .expect("fret submenu trigger semantics (Share)");
            let root_menu = snap
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::Menu)
                .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
                .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Menu))
                .expect("fret root menu semantics");

            let root_center_y = root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5;
            let trigger_center_y = trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5;
            let dy = trigger_center_y - root_center_y;
            if dy.abs() <= 1.0 {
                break;
            }

            let wheel_pos = Point::new(
                Px(root_menu.bounds.origin.x.0 + root_menu.bounds.size.width.0 * 0.5),
                Px(root_menu.bounds.origin.y.0 + root_menu.bounds.size.height.0 * 0.5),
            );
            ui.dispatch_event(
                &mut app,
                &mut services,
                &Event::Pointer(PointerEvent::Wheel {
                    pointer_id: fret_core::PointerId::default(),
                    position: wheel_pos,
                    delta: Point::new(Px(0.0), Px(-dy)),
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                }),
            );
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(frame),
                true,
                render,
            );
            frame += 1;
            snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        }

        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
            .expect("fret submenu trigger semantics (Share, scrolled)");
        ui.set_focus(Some(trigger.id));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            true,
            render,
        );
        frame += 1;

        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let focused = fret_focused_label(&snap);
        assert_eq!(
            focused,
            Some("Share"),
            "{web_name}: failed to focus submenu trigger (Share); focused={focused:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    } else {
        let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
            .expect("fret submenu trigger semantics (Share)");
        let root_menu = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .find(|m| fret_rect_contains(m.bounds, trigger.bounds))
            .or_else(|| snap.nodes.iter().find(|n| n.role == SemanticsRole::Menu))
            .expect("fret root menu semantics");
        assert!(
            fret_rect_contains(root_menu.bounds, trigger.bounds),
            "{web_name}: submenu trigger is not visible in root menu panel (expected to open submenu by hover)"
        );

        let hover_point = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: hover_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
        deliver_all_timers_from_effects(&mut ui, &mut app, &mut services);
    }

    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame + tick),
            request_semantics,
            render,
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    (web, snap)
}

fn assert_menubar_demo_submenu_constrained_menu_content_insets_match(web_name: &str) {
    let (web, snap) = build_menubar_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_slots = ["menubar-content", "menubar-sub-content"];
    let expected = web_menu_content_insets_for_slots(&theme, &expected_slots);
    let expected_hs: Vec<f32> = expected_slots
        .iter()
        .map(|slot| web_portal_node_by_data_slot(&theme, slot).rect.h)
        .collect();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let mut actual_hs = fret_menu_heights(&snap);
    assert!(
        actual_hs.len() == expected_hs.len(),
        "{web_name} expected {} menus, got {}",
        expected_hs.len(),
        actual_hs.len()
    );
    let mut expected_hs = expected_hs;
    expected_hs.sort_by(|a, b| b.total_cmp(a));
    actual_hs.sort_by(|a, b| b.total_cmp(a));
    for (i, (a, e)) in actual_hs.iter().zip(expected_hs.iter()).enumerate() {
        assert_close(&format!("{web_name} menu[{i}] height"), *a, *e, 2.0);
    }
}

#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match(
        "menubar-demo.submenu-kbd-vp1440x320",
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match(
        "menubar-demo.submenu-kbd-vp1440x240",
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match("menubar-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_hover_menu_content_insets_match() {
    assert_menubar_demo_submenu_constrained_menu_content_insets_match("menubar-demo.submenu");
}

fn assert_menubar_demo_submenu_menu_item_height_matches(web_name: &str) {
    let (web, snap) = build_menubar_demo_submenu_snapshot(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(
        theme,
        &[
            "menubar-item",
            "menubar-checkbox-item",
            "menubar-radio-item",
            "menubar-sub-trigger",
        ],
    );
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web menu item rows for {web_name}"));

    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

fn assert_menubar_demo_submenu_first_visible_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let labels = ["Email link", "Messages", "Notes"];
    let expected = web_first_visible_menu_item_label(
        web_portal_node_by_data_slot(&theme, "menubar-sub-content"),
        &labels,
    )
    .unwrap_or_else(|| panic!("missing web first visible submenu item for {web_name}"));

    let (_, snap) = build_menubar_demo_submenu_snapshot(web_name);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("fret submenu trigger semantics (Share)");
    let submenu = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Menu)
        .find(|m| !fret_rect_contains(m.bounds, trigger.bounds))
        .expect("submenu menu does not contain sub-trigger");

    let actual =
        fret_first_visible_menu_item_label(&snap, submenu.bounds, &labels).unwrap_or("<missing>");
    assert_eq!(
        actual, expected,
        "{web_name}: submenu first visible item mismatch"
    );
}

#[test]
fn web_vs_fret_menubar_demo_submenu_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_hover_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_small_viewport_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd-vp1440x320");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_tiny_viewport_first_visible_matches() {
    assert_menubar_demo_submenu_first_visible_matches("menubar-demo.submenu-kbd-vp1440x240");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_menu_item_height_matches() {
    assert_menubar_demo_submenu_menu_item_height_matches("menubar-demo.submenu-kbd");
}

#[test]
fn web_vs_fret_menubar_demo_submenu_hover_menu_item_height_matches() {
    assert_menubar_demo_submenu_menu_item_height_matches("menubar-demo.submenu");
}

#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_dialog_demo_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "dialog-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_sidebar_13_dialog_overlay_center_matches() {
    use fret_ui_shadcn::{Button, ButtonSize, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "sidebar-13",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .size(ButtonSize::Sm)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(Vec::new())
                        .refine_style(fret_ui_kit::ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .max_w(fret_ui_kit::MetricRef::Px(Px(800.0)))
                                .max_h(fret_ui_kit::MetricRef::Px(Px(500.0))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_command_dialog_overlay_center_matches() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}

fn web_command_dialog_input<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    web_portal_node_by_data_slot(theme, "command-input")
}

fn web_command_dialog_listbox<'a>(theme: &'a WebGoldenTheme) -> &'a WebNode {
    web_portal_node_by_data_slot(theme, "command-list")
}

fn command_dialog_open_snapshot(theme: &WebGoldenTheme) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(theme);

    let query: Model<String> = app.models_mut().insert(String::new());
    let open: Model<bool> = app.models_mut().insert(false);

    let entries = vec![
        fret_ui_shadcn::CommandGroup::new(vec![
            fret_ui_shadcn::CommandItem::new("Calendar").on_select("command.calendar"),
            fret_ui_shadcn::CommandItem::new("Search Emoji").on_select("command.search_emoji"),
            fret_ui_shadcn::CommandItem::new("Calculator").on_select("command.calculator"),
        ])
        .heading("Suggestions")
        .into(),
        fret_ui_shadcn::CommandSeparator::new().into(),
        fret_ui_shadcn::CommandGroup::new(vec![
            fret_ui_shadcn::CommandItem::new("Profile")
                .shortcut("⌘P")
                .on_select("command.profile"),
            fret_ui_shadcn::CommandItem::new("Billing")
                .shortcut("⌘B")
                .on_select("command.billing"),
            fret_ui_shadcn::CommandItem::new("Settings")
                .shortcut("⌘S")
                .on_select("command.settings"),
        ])
        .heading("Settings")
        .into(),
    ];

    let render = |cx: &mut ElementContext<'_, App>| {
        use fret_ui_shadcn::{Button, CommandDialog};

        CommandDialog::new(open.clone(), query.clone(), Vec::new())
            .entries(entries.clone())
            .into_element(cx, |cx| Button::new("Open").into_element(cx))
    };

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = render(cx);
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                let el = render(cx);
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    ui.semantics_snapshot().expect("semantics snapshot").clone()
}

fn assert_command_dialog_input_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_input = web_command_dialog_input(theme);
    let expected_h = web_input.rect.h;

    let snap = command_dialog_open_snapshot(theme);
    let combobox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox)
        .unwrap_or_else(|| panic!("missing fret command dialog input for {web_name}"));

    assert_close(
        &format!("{web_name} command_dialog_input_h"),
        combobox.bounds.size.height.0,
        expected_h,
        2.0,
    );
}

fn assert_command_dialog_listbox_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_command_dialog_listbox(theme);
    let expected_h = web_listbox.rect.h;

    let snap = command_dialog_open_snapshot(theme);
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret command dialog listbox for {web_name}"));

    assert_close(
        &format!("{web_name} command_dialog_listbox_h"),
        listbox.bounds.size.height.0,
        expected_h,
        2.0,
    );
}

fn assert_command_dialog_listbox_option_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_command_dialog_listbox(theme);

    let expected: std::collections::BTreeSet<i32> = web_select_listbox_option_heights(web_listbox)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        expected.len() == 1,
        "{web_name} expected uniform web command item height; got {expected:?}"
    );

    let snap = command_dialog_open_snapshot(theme);
    let actual: std::collections::BTreeSet<i32> = fret_listbox_option_heights_in_listbox(&snap)
        .into_iter()
        .map(round_i32)
        .collect();
    assert!(
        actual.len() == 1,
        "{web_name} expected uniform fret command item height; got {actual:?}"
    );

    let expected_h = expected.iter().next().copied().unwrap_or_default() as f32;
    let actual_h = actual.iter().next().copied().unwrap_or_default() as f32;
    assert_close(
        &format!("{web_name} command_dialog_item_h"),
        actual_h,
        expected_h,
        1.0,
    );
}

fn assert_command_dialog_listbox_option_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let web_listbox = web_command_dialog_listbox(theme);
    let expected_inset = web_select_content_option_inset(web_listbox);

    let snap = command_dialog_open_snapshot(theme);
    let actual_inset = fret_select_content_option_inset(&snap);
    assert_select_inset_match(web_name, actual_inset, expected_inset);
}

#[test]
fn web_vs_fret_command_dialog_input_height_matches() {
    assert_command_dialog_input_height_matches("command-dialog");
}

#[test]
fn web_vs_fret_command_dialog_input_height_matches_tiny_viewport() {
    assert_command_dialog_input_height_matches("command-dialog.vp1440x240");
}

#[test]
fn web_vs_fret_command_dialog_listbox_height_matches() {
    assert_command_dialog_listbox_height_matches("command-dialog");
}

#[test]
fn web_vs_fret_command_dialog_listbox_height_matches_tiny_viewport() {
    assert_command_dialog_listbox_height_matches("command-dialog.vp1440x240");
}

#[test]
fn web_vs_fret_command_dialog_listbox_option_height_matches() {
    assert_command_dialog_listbox_option_height_matches("command-dialog");
}

#[test]
fn web_vs_fret_command_dialog_listbox_option_height_matches_tiny_viewport() {
    assert_command_dialog_listbox_option_height_matches("command-dialog.vp1440x240");
}

#[test]
fn web_vs_fret_command_dialog_listbox_option_insets_match() {
    assert_command_dialog_listbox_option_insets_match("command-dialog");
}

#[test]
fn web_vs_fret_command_dialog_listbox_option_insets_match_tiny_viewport() {
    assert_command_dialog_listbox_option_insets_match("command-dialog.vp1440x240");
}

#[test]
fn web_vs_fret_command_dialog_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    assert_centered_overlay_placement_matches(
        "command-dialog.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            CommandDialog::new(open.clone(), query, items)
                .into_element(cx, |cx| Button::new("Open").into_element(cx))
        },
    );
}

#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_alert_dialog_demo_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{AlertDialog, AlertDialogContent, Button, ButtonVariant};

    assert_centered_overlay_placement_matches(
        "alert-dialog-demo.vp1440x240",
        "alertdialog",
        SemanticsRole::AlertDialog,
        |cx, open| {
            AlertDialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Show Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    AlertDialogContent::new(vec![cx.text("Are you absolutely sure?")])
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_demo_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_top_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.top-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Top).into_element(
                cx,
                |cx| {
                    Button::new("top")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_right_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.right-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_bottom_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.bottom-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_sheet_side_left_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_viewport_anchored_overlay_placement_matches(
        "sheet-side.left-vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_drawer_demo_overlay_insets_match_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    assert_viewport_anchored_overlay_placement_matches(
        "drawer-demo.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Drawer")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
            )
        },
    );
}

#[test]
fn web_vs_fret_drawer_dialog_desktop_overlay_center_matches_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_centered_overlay_placement_matches(
        "drawer-dialog.vp1440x240",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Edit Profile")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .max_w(fret_ui_kit::MetricRef::Px(Px(425.0))),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_calendar_32_open_drawer_insets_match() {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent, DrawerHeader};
    use time::Month;

    assert_viewport_anchored_overlay_placement_matches(
        "calendar-32",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Select date")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    let month: Model<CalendarMonth> = cx
                        .app
                        .models_mut()
                        .insert(CalendarMonth::new(2025, Month::June));
                    let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                    let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                    DrawerContent::new(vec![DrawerHeader::new(vec![]).into_element(cx), calendar])
                        .into_element(cx)
                },
            )
        },
    );
}
