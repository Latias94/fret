//! Shadcn-style typography helpers.
//!
//! Upstream shadcn/ui v4 does not ship typography styles by default; the docs provide an example
//! page with utility classes. In Fret, we provide a small set of builder helpers for common
//! typographic primitives so demos can stay self-contained.
//!
//! Reference:
//! - `repo-ref/ui/apps/v4/content/docs/components/typography.mdx`

use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

fn text_props(
    text: Arc<str>,
    style: Option<TextStyle>,
    color: Option<Color>,
    wrap: TextWrap,
) -> TextProps {
    TextProps {
        layout: LayoutStyle::default(),
        text,
        style,
        color,
        wrap,
        overflow: TextOverflow::Clip,
    }
}

fn base_text_style(theme: &Theme) -> TextStyle {
    let px = theme.metric_by_key("font.size").unwrap_or(Px(14.0));
    let line_height = theme.metric_by_key("font.line_height");
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: TextSlant::Normal,
        line_height,
        letter_spacing_em: None,
    }
}

fn heading_style(theme: &Theme, px: f32, weight: FontWeight) -> TextStyle {
    TextStyle {
        size: Px(px),
        weight,
        ..base_text_style(theme)
    }
}

fn muted_color(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn container_props(
    theme: &Theme,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> ContainerProps {
    decl_style::container_props(theme, chrome, layout)
}

pub fn h1<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = heading_style(&theme, 40.0, FontWeight::EXTRA_BOLD);
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn h2<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = heading_style(&theme, 32.0, FontWeight::BOLD);
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn h3<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = heading_style(&theme, 24.0, FontWeight::BOLD);
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn h4<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = heading_style(&theme, 20.0, FontWeight::SEMIBOLD);
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn p<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(text_props(
        text.into(),
        Some(base_text_style(&theme)),
        None,
        TextWrap::Word,
    ))
}

pub fn lead<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = TextStyle {
        size: Px(18.0),
        ..base_text_style(&theme)
    };
    cx.text_props(text_props(
        text.into(),
        Some(style),
        Some(muted_color(&theme)),
        TextWrap::Word,
    ))
}

pub fn large<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = TextStyle {
        size: Px(18.0),
        weight: FontWeight::SEMIBOLD,
        ..base_text_style(&theme)
    };
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn small<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::MEDIUM,
        ..base_text_style(&theme)
    };
    cx.text_props(text_props(text.into(), Some(style), None, TextWrap::Word))
}

pub fn muted<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let style = TextStyle {
        size: Px(12.0),
        ..base_text_style(&theme)
    };
    cx.text_props(text_props(
        text.into(),
        Some(style),
        Some(muted_color(&theme)),
        TextWrap::Word,
    ))
}

pub fn inline_code<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let chrome = ChromeRefinement::default()
        .bg(ColorRef::Color(theme.color_required("muted")))
        .rounded(Radius::Sm)
        .px(Space::N2)
        .py(Space::N1);
    let layout = LayoutRefinement::default();
    let props = container_props(&theme, chrome, layout);
    cx.container(props, move |cx| {
        let style = TextStyle {
            size: Px(12.0),
            weight: FontWeight::MEDIUM,
            ..base_text_style(&theme)
        };
        [cx.text_props(text_props(text.into(), Some(style), None, TextWrap::None))]
    })
}

pub fn blockquote<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let mut layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
    layout.size.min_width = Some(Px(0.0));
    let props = ContainerProps {
        layout,
        padding: Edges {
            left: Px(16.0),
            top: Px(8.0),
            right: Px(0.0),
            bottom: Px(8.0),
        },
        background: None,
        shadow: None,
        border: Edges {
            left: Px(2.0),
            ..Edges::all(Px(0.0))
        },
        border_color: Some(theme.color_required("border")),
        corner_radii: Corners::all(Px(0.0)),
        ..Default::default()
    };
    cx.container(props, move |cx| {
        let style = TextStyle {
            size: Px(14.0),
            weight: FontWeight::MEDIUM,
            slant: TextSlant::Italic,
            ..base_text_style(&theme)
        };
        [cx.text_props(text_props(
            text.into(),
            Some(style),
            Some(muted_color(&theme)),
            TextWrap::Word,
        ))]
    })
}

pub fn list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: impl IntoIterator<Item = Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let item_style = base_text_style(&theme);
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    cx.flex(
        FlexProps {
            layout,
            direction: fret_core::Axis::Vertical,
            gap: Px(6.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
            ..Default::default()
        },
        move |cx| {
            items
                .into_iter()
                .map(|item| {
                    let mut row_layout = LayoutStyle::default();
                    row_layout.size.width = Length::Fill;
                    let item_style = item_style.clone();
                    let row = cx.flex(
                        FlexProps {
                            layout: row_layout,
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(8.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.text("•"),
                                cx.text_props(text_props(
                                    item.clone(),
                                    Some(item_style),
                                    None,
                                    TextWrap::Word,
                                )),
                            ]
                        },
                    );
                    row
                })
                .collect::<Vec<_>>()
        },
    )
}
