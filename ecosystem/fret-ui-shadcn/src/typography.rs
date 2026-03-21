//! Shadcn-style typography helpers.
//!
//! Upstream shadcn/ui v4 does not ship typography styles by default; the docs provide an example
//! page with utility classes. In Fret, we provide a small set of builder helpers for common
//! typographic primitives so demos can stay self-contained.
//!
//! Reference:
//! - `repo-ref/ui/apps/v4/content/docs/components/base/typography.mdx`

use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow,
    TextSlant, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, Radius, Space};

fn text_props(
    text: Arc<str>,
    style: Option<TextStyle>,
    color: Option<Color>,
    wrap: TextWrap,
) -> TextProps {
    let mut layout = LayoutStyle::default();
    // Typography helpers are intended for long-form / block-like content (shadcn docs parity).
    // Default to full-width layout for wrapped text so headings/paragraphs don't shrink-wrap to
    // min-content widths under intrinsic sizing probes.
    if !matches!(wrap, TextWrap::None) {
        layout.size.width = Length::Fill;
    }
    TextProps {
        layout,
        text,
        style,
        color,
        wrap,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

fn base_text_style(theme: &Theme) -> TextStyle {
    let px = theme.metric_by_key("font.size").unwrap_or(Px(14.0));
    let line_height = scaled_line_height(theme, px);
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        line_height,
        ..Default::default()
    }
}

fn scaled_line_height(theme: &Theme, size: Px) -> Option<Px> {
    let base_size = theme.metric_by_key("font.size").unwrap_or(Px(14.0)).0;
    if base_size <= 0.0 {
        return None;
    }
    let base_line_height = theme.metric_by_key("font.line_height")?.0;
    if base_line_height <= 0.0 {
        return None;
    }

    let ratio = base_line_height / base_size;
    Some(Px((size.0 * ratio).max(size.0)))
}

fn heading_style(theme: &Theme, px: f32, weight: FontWeight) -> TextStyle {
    TextStyle {
        size: Px(px),
        weight,
        line_height: scaled_line_height(theme, Px(px)),
        ..base_text_style(theme)
    }
}

fn muted_color(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn container_props(
    theme: &Theme,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> ContainerProps {
    decl_style::container_props(theme, chrome, layout)
}

fn heading_semantics(level: u32) -> SemanticsDecoration {
    SemanticsDecoration::default()
        .role(SemanticsRole::Heading)
        .level(level)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypographyTextKind {
    H1,
    H2,
    H3,
    H4,
    Paragraph,
    Lead,
    Large,
    Small,
    Muted,
    InlineCode,
    Blockquote,
}

#[derive(Debug, Clone)]
struct TypographyText {
    kind: TypographyTextKind,
    text: Arc<str>,
}

impl TypographyText {
    fn new(kind: TypographyTextKind, text: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    #[track_caller]
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self.kind {
            TypographyTextKind::H1 => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    heading_style(theme, 40.0, FontWeight::EXTRA_BOLD)
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(1))
            }
            TypographyTextKind::H2 => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    heading_style(theme, 32.0, FontWeight::BOLD)
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(2))
            }
            TypographyTextKind::H3 => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    heading_style(theme, 24.0, FontWeight::BOLD)
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(3))
            }
            TypographyTextKind::H4 => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    heading_style(theme, 20.0, FontWeight::SEMIBOLD)
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(4))
            }
            TypographyTextKind::Paragraph => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    base_text_style(theme)
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
            }
            TypographyTextKind::Lead => {
                let (style, color) = {
                    let theme = Theme::global(&*cx.app);
                    let style = TextStyle {
                        size: Px(18.0),
                        line_height: scaled_line_height(theme, Px(18.0)),
                        ..base_text_style(theme)
                    };
                    (style, muted_color(theme))
                };
                cx.text_props(text_props(
                    self.text,
                    Some(style),
                    Some(color),
                    TextWrap::Word,
                ))
            }
            TypographyTextKind::Large => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    TextStyle {
                        size: Px(18.0),
                        weight: FontWeight::SEMIBOLD,
                        line_height: scaled_line_height(theme, Px(18.0)),
                        ..base_text_style(theme)
                    }
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
            }
            TypographyTextKind::Small => {
                let style = {
                    let theme = Theme::global(&*cx.app);
                    TextStyle {
                        size: Px(12.0),
                        weight: FontWeight::MEDIUM,
                        line_height: scaled_line_height(theme, Px(12.0)),
                        ..base_text_style(theme)
                    }
                };
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
            }
            TypographyTextKind::Muted => {
                let (style, color) = {
                    let theme = Theme::global(&*cx.app);
                    let style = TextStyle {
                        size: Px(12.0),
                        line_height: scaled_line_height(theme, Px(12.0)),
                        ..base_text_style(theme)
                    };
                    (style, muted_color(theme))
                };
                cx.text_props(text_props(
                    self.text,
                    Some(style),
                    Some(color),
                    TextWrap::Word,
                ))
            }
            TypographyTextKind::InlineCode => {
                let (props, base_style, line_height) = {
                    let theme = Theme::global(&*cx.app);
                    let chrome = ChromeRefinement::default()
                        .bg(ColorRef::Color(theme.color_token("muted")))
                        .rounded(Radius::Sm)
                        .px(Space::N2)
                        .py(Space::N1);
                    let layout = LayoutRefinement::default();
                    let props = container_props(theme, chrome, layout);
                    let base_style = base_text_style(theme);
                    let line_height = scaled_line_height(theme, Px(12.0));
                    (props, base_style, line_height)
                };
                cx.container(props, move |cx| {
                    let style = TextStyle {
                        size: Px(12.0),
                        weight: FontWeight::MEDIUM,
                        line_height,
                        ..base_style.clone()
                    };
                    [cx.text_props(text_props(self.text, Some(style), None, TextWrap::None))]
                })
            }
            TypographyTextKind::Blockquote => {
                let (layout, border_color, muted, base_style) = {
                    let theme = Theme::global(&*cx.app);
                    let mut layout =
                        decl_style::layout_style(theme, LayoutRefinement::default().w_full());
                    layout.size.min_width = Some(Length::Px(Px(0.0)));
                    let border_color = theme.color_token("border");
                    let muted = muted_color(theme);
                    let base_style = base_text_style(theme);
                    (layout, border_color, muted, base_style)
                };
                let props = ContainerProps {
                    layout,
                    padding: Edges {
                        left: Px(16.0),
                        top: Px(8.0),
                        right: Px(0.0),
                        bottom: Px(8.0),
                    }
                    .into(),
                    background: None,
                    shadow: None,
                    border: Edges {
                        left: Px(2.0),
                        ..Edges::all(Px(0.0))
                    },
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(0.0)),
                    ..Default::default()
                };
                cx.container(props, move |cx| {
                    let style = TextStyle {
                        size: Px(14.0),
                        weight: FontWeight::MEDIUM,
                        slant: TextSlant::Italic,
                        ..base_style.clone()
                    };
                    [cx.text_props(text_props(
                        self.text,
                        Some(style),
                        Some(muted),
                        TextWrap::Word,
                    ))]
                })
            }
        }
    }
}

fret_ui_kit::ui_component_passthrough!(TypographyText);

#[derive(Debug, Clone)]
struct TypographyList {
    items: Vec<Arc<str>>,
}

impl TypographyList {
    fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<str>>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }

    #[track_caller]
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let item_style = {
            let theme = Theme::global(&*cx.app);
            base_text_style(theme)
        };
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap: Px(6.0).into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| {
                self.items
                    .into_iter()
                    .map(|item| {
                        let mut row_layout = LayoutStyle::default();
                        row_layout.size.width = Length::Fill;
                        let item_style = item_style.clone();
                        cx.flex(
                            FlexProps {
                                layout: row_layout,
                                direction: fret_core::Axis::Horizontal,
                                gap: Px(8.0).into(),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Start,
                                wrap: false,
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
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
    }
}

fret_ui_kit::ui_component_passthrough!(TypographyList);

pub fn h1<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::H1, text)
}

pub fn h2<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::H2, text)
}

pub fn h3<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::H3, text)
}

pub fn h4<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::H4, text)
}

pub fn p<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Paragraph, text)
}

pub fn lead<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Lead, text)
}

pub fn large<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Large, text)
}

pub fn small<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Small, text)
}

pub fn muted<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Muted, text)
}

pub fn inline_code<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::InlineCode, text)
}

pub fn blockquote<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    TypographyText::new(TypographyTextKind::Blockquote, text)
}

pub fn list<H: UiHost, I, T>(items: I) -> impl IntoUiElement<H> + use<H, I, T>
where
    I: IntoIterator<Item = T>,
    T: Into<Arc<str>>,
{
    TypographyList::new(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size as CoreSize};
    use fret_ui::elements;

    fn render(build: impl FnOnce(&mut ElementContext<'_, App>) -> AnyElement) -> AnyElement {
        let mut app = App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(200.0)),
        );
        elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "typography-heading-semantics",
            build,
        )
    }

    fn assert_heading_level(element: &AnyElement, expected_level: u32) {
        let decoration = element
            .semantics_decoration
            .as_ref()
            .expect("expected heading semantics decoration");
        assert_eq!(decoration.role, Some(SemanticsRole::Heading));
        assert_eq!(decoration.level, Some(expected_level));
    }

    #[test]
    fn typography_headings_attach_heading_semantics_levels() {
        assert_heading_level(&render(|cx| h1("Heading 1").into_element(cx)), 1);
        assert_heading_level(&render(|cx| h2("Heading 2").into_element(cx)), 2);
        assert_heading_level(&render(|cx| h3("Heading 3").into_element(cx)), 3);
        assert_heading_level(&render(|cx| h4("Heading 4").into_element(cx)), 4);
    }
}
