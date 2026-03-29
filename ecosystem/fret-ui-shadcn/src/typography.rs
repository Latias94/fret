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
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, MarginEdge,
    SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement};

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

const TRACKING_TIGHT_EM: f32 = -0.025;
const INLINE_CODE_PX_X: Px = Px(4.8);
const INLINE_CODE_PX_Y: Px = Px(3.2);
const H2_PADDING_BOTTOM: Px = Px(8.0);
const H2_BORDER_BOTTOM: Px = Px(1.0);
const BLOCKQUOTE_BORDER_INLINE_START: Px = Px(2.0);
const BLOCKQUOTE_PADDING_INLINE_START: Px = Px(24.0);
const LIST_MARGIN_INLINE_START: Px = Px(24.0);
const LIST_ITEM_GAP: Px = Px(8.0);

fn fixed_ui_style(size: Px, line_height: Px, weight: FontWeight) -> TextStyle {
    let mut style = fret_ui_kit::typography::fixed_line_box_style(FontId::ui(), size, line_height);
    style.weight = weight;
    style
}

fn fixed_monospace_style(size: Px, line_height: Px, weight: FontWeight) -> TextStyle {
    let mut style =
        fret_ui_kit::typography::fixed_line_box_style(FontId::monospace(), size, line_height);
    style.weight = weight;
    style
}

fn tracking_tight(mut style: TextStyle) -> TextStyle {
    style.letter_spacing_em = Some(TRACKING_TIGHT_EM);
    style
}

fn h1_style() -> TextStyle {
    tracking_tight(fixed_ui_style(Px(36.0), Px(40.0), FontWeight::EXTRA_BOLD))
}

fn h2_style() -> TextStyle {
    tracking_tight(fixed_ui_style(Px(30.0), Px(36.0), FontWeight::SEMIBOLD))
}

fn h3_style() -> TextStyle {
    tracking_tight(fixed_ui_style(Px(24.0), Px(32.0), FontWeight::SEMIBOLD))
}

fn h4_style() -> TextStyle {
    tracking_tight(fixed_ui_style(Px(20.0), Px(28.0), FontWeight::SEMIBOLD))
}

fn paragraph_style() -> TextStyle {
    fixed_ui_style(Px(16.0), Px(28.0), FontWeight::NORMAL)
}

fn lead_style() -> TextStyle {
    fixed_ui_style(Px(20.0), Px(28.0), FontWeight::NORMAL)
}

fn large_style() -> TextStyle {
    fixed_ui_style(Px(18.0), Px(28.0), FontWeight::SEMIBOLD)
}

fn small_style() -> TextStyle {
    fixed_ui_style(Px(14.0), Px(14.0), FontWeight::MEDIUM)
}

fn muted_style() -> TextStyle {
    fixed_ui_style(Px(14.0), Px(20.0), FontWeight::NORMAL)
}

fn inline_code_style() -> TextStyle {
    fixed_monospace_style(Px(14.0), Px(20.0), FontWeight::SEMIBOLD)
}

fn blockquote_style() -> TextStyle {
    let mut style = fixed_ui_style(Px(16.0), Px(24.0), FontWeight::NORMAL);
    style.slant = TextSlant::Italic;
    style
}

fn list_item_style() -> TextStyle {
    fixed_ui_style(Px(16.0), Px(24.0), FontWeight::NORMAL)
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
                let style = h1_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Balance))
                    .attach_semantics(heading_semantics(1))
            }
            TypographyTextKind::H2 => {
                let (props, border_color, style) = {
                    let theme = Theme::global(&*cx.app);
                    let mut layout =
                        decl_style::layout_style(theme, LayoutRefinement::default().w_full());
                    layout.size.min_width = Some(Length::Px(Px(0.0)));
                    let props = ContainerProps {
                        layout,
                        padding: Edges {
                            bottom: H2_PADDING_BOTTOM,
                            ..Edges::all(Px(0.0))
                        }
                        .into(),
                        background: None,
                        shadow: None,
                        border: Edges {
                            bottom: H2_BORDER_BOTTOM,
                            ..Edges::all(Px(0.0))
                        },
                        corner_radii: Corners::all(Px(0.0)),
                        ..Default::default()
                    };
                    (props, theme.color_token("border"), h2_style())
                };
                cx.container(
                    ContainerProps {
                        border_color: Some(border_color),
                        ..props
                    },
                    move |cx| {
                        [cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))]
                    },
                )
                .attach_semantics(heading_semantics(2))
            }
            TypographyTextKind::H3 => {
                let style = h3_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(3))
            }
            TypographyTextKind::H4 => {
                let style = h4_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
                    .attach_semantics(heading_semantics(4))
            }
            TypographyTextKind::Paragraph => {
                let style = paragraph_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
            }
            TypographyTextKind::Lead => {
                let (style, color) = {
                    let theme = Theme::global(&*cx.app);
                    (lead_style(), muted_color(theme))
                };
                cx.text_props(text_props(
                    self.text,
                    Some(style),
                    Some(color),
                    TextWrap::Word,
                ))
            }
            TypographyTextKind::Large => {
                let style = large_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))
            }
            TypographyTextKind::Small => {
                let style = small_style();
                cx.text_props(text_props(self.text, Some(style), None, TextWrap::None))
            }
            TypographyTextKind::Muted => {
                let (style, color) = {
                    let theme = Theme::global(&*cx.app);
                    (muted_style(), muted_color(theme))
                };
                cx.text_props(text_props(
                    self.text,
                    Some(style),
                    Some(color),
                    TextWrap::Word,
                ))
            }
            TypographyTextKind::InlineCode => {
                let (props, style) = {
                    let theme = Theme::global(&*cx.app);
                    let chrome =
                        ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("muted")));
                    let layout = LayoutRefinement::default();
                    let mut props = container_props(theme, chrome, layout);
                    props.padding = Edges {
                        top: INLINE_CODE_PX_Y,
                        right: INLINE_CODE_PX_X,
                        bottom: INLINE_CODE_PX_Y,
                        left: INLINE_CODE_PX_X,
                    }
                    .into();
                    props.corner_radii = Corners::all(Px(4.0));
                    (props, inline_code_style())
                };
                cx.container(props, move |cx| {
                    [cx.text_props(text_props(self.text, Some(style), None, TextWrap::None))]
                })
            }
            TypographyTextKind::Blockquote => {
                let (layout, border_color, style, dir) = {
                    let theme = Theme::global(&*cx.app);
                    let dir = crate::direction::use_direction(cx, None);
                    let mut layout =
                        decl_style::layout_style(theme, LayoutRefinement::default().w_full());
                    layout.size.min_width = Some(Length::Px(Px(0.0)));
                    let border_color = theme.color_token("border");
                    (layout, border_color, blockquote_style(), dir)
                };
                let (border_left, border_right) = crate::rtl::physical_inline_start_end(
                    dir,
                    BLOCKQUOTE_BORDER_INLINE_START,
                    Px(0.0),
                );
                let (padding_left, padding_right) = crate::rtl::physical_inline_start_end(
                    dir,
                    BLOCKQUOTE_PADDING_INLINE_START,
                    Px(0.0),
                );
                let props = ContainerProps {
                    layout,
                    padding: Edges {
                        left: padding_left,
                        right: padding_right,
                        ..Edges::all(Px(0.0))
                    }
                    .into(),
                    background: None,
                    shadow: None,
                    border: Edges {
                        left: border_left,
                        right: border_right,
                        ..Edges::all(Px(0.0))
                    },
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(0.0)),
                    ..Default::default()
                };
                cx.container(props, move |cx| {
                    [cx.text_props(text_props(self.text, Some(style), None, TextWrap::Word))]
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
        let dir = crate::direction::use_direction(cx, None);
        let item_style = list_item_style();
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        match dir {
            crate::direction::LayoutDirection::Ltr => {
                layout.margin.left = MarginEdge::Px(LIST_MARGIN_INLINE_START);
            }
            crate::direction::LayoutDirection::Rtl => {
                layout.margin.right = MarginEdge::Px(LIST_MARGIN_INLINE_START);
            }
        }
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap: LIST_ITEM_GAP.into(),
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
                                let bullet = cx.text_props(text_props(
                                    Arc::<str>::from("•"),
                                    Some(item_style.clone()),
                                    None,
                                    TextWrap::None,
                                ));
                                let label = cx.text_props(text_props(
                                    item.clone(),
                                    Some(item_style),
                                    None,
                                    TextWrap::Word,
                                ));

                                crate::rtl::vec_main_with_inline_start(dir, label, Some(bullet))
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
    use fret_ui::element::ElementKind;
    use fret_ui::elements;

    fn render(build: impl FnOnce(&mut ElementContext<'_, App>) -> AnyElement) -> AnyElement {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
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

    fn text_props_of(element: &AnyElement) -> &TextProps {
        match &element.kind {
            ElementKind::Text(props) => props,
            other => panic!("expected ElementKind::Text, got {other:?}"),
        }
    }

    fn container_props_of(element: &AnyElement) -> &ContainerProps {
        match &element.kind {
            ElementKind::Container(props) => props,
            other => panic!("expected ElementKind::Container, got {other:?}"),
        }
    }

    fn flex_props_of(element: &AnyElement) -> &FlexProps {
        match &element.kind {
            ElementKind::Flex(props) => props,
            other => panic!("expected ElementKind::Flex, got {other:?}"),
        }
    }

    #[test]
    fn typography_headings_attach_heading_semantics_levels() {
        assert_heading_level(&render(|cx| h1("Heading 1").into_element(cx)), 1);
        assert_heading_level(&render(|cx| h2("Heading 2").into_element(cx)), 2);
        assert_heading_level(&render(|cx| h3("Heading 3").into_element(cx)), 3);
        assert_heading_level(&render(|cx| h4("Heading 4").into_element(cx)), 4);
    }

    #[test]
    fn typography_helpers_match_shadcn_v4_text_metrics() {
        let h1 = render(|cx| h1("Heading 1").into_element(cx));
        let h1_props = text_props_of(&h1);
        let h1_style = h1_props.style.as_ref().expect("expected h1 text style");
        assert_eq!(h1_style.font, FontId::ui());
        assert_eq!(h1_style.size, Px(36.0));
        assert_eq!(h1_style.line_height, Some(Px(40.0)));
        assert_eq!(h1_style.weight, FontWeight::EXTRA_BOLD);
        assert_eq!(h1_style.letter_spacing_em, Some(TRACKING_TIGHT_EM));
        assert_eq!(h1_props.wrap, TextWrap::Balance);

        let h2 = render(|cx| h2("Heading 2").into_element(cx));
        let h2_container = container_props_of(&h2);
        assert_eq!(h2_container.padding.bottom, H2_PADDING_BOTTOM.into());
        assert_eq!(h2_container.border.bottom, H2_BORDER_BOTTOM);
        let h2_text = text_props_of(
            h2.children
                .first()
                .expect("expected h2 container to wrap one text child"),
        );
        let h2_style = h2_text.style.as_ref().expect("expected h2 text style");
        assert_eq!(h2_style.size, Px(30.0));
        assert_eq!(h2_style.line_height, Some(Px(36.0)));
        assert_eq!(h2_style.weight, FontWeight::SEMIBOLD);
        assert_eq!(h2_style.letter_spacing_em, Some(TRACKING_TIGHT_EM));

        let p = render(|cx| p("Paragraph").into_element(cx));
        let p_props = text_props_of(&p);
        let p_style = p_props
            .style
            .as_ref()
            .expect("expected paragraph text style");
        assert_eq!(p_style.font, FontId::ui());
        assert_eq!(p_style.size, Px(16.0));
        assert_eq!(p_style.line_height, Some(Px(28.0)));
        assert_eq!(p_props.wrap, TextWrap::Word);

        let lead = render(|cx| lead("Lead").into_element(cx));
        let lead_props = text_props_of(&lead);
        let lead_style = lead_props.style.as_ref().expect("expected lead text style");
        assert_eq!(lead_style.size, Px(20.0));
        assert_eq!(lead_style.line_height, Some(Px(28.0)));

        let large = render(|cx| large("Large").into_element(cx));
        let large_props = text_props_of(&large);
        let large_style = large_props
            .style
            .as_ref()
            .expect("expected large text style");
        assert_eq!(large_style.size, Px(18.0));
        assert_eq!(large_style.line_height, Some(Px(28.0)));
        assert_eq!(large_style.weight, FontWeight::SEMIBOLD);

        let small = render(|cx| small("Small").into_element(cx));
        let small_props = text_props_of(&small);
        let small_style = small_props
            .style
            .as_ref()
            .expect("expected small text style");
        assert_eq!(small_style.size, Px(14.0));
        assert_eq!(small_style.line_height, Some(Px(14.0)));
        assert_eq!(small_style.weight, FontWeight::MEDIUM);
        assert_eq!(small_props.wrap, TextWrap::None);
        assert_eq!(small_props.layout.size.width, Length::Auto);

        let muted = render(|cx| muted("Muted").into_element(cx));
        let muted_props = text_props_of(&muted);
        let muted_style = muted_props
            .style
            .as_ref()
            .expect("expected muted text style");
        assert_eq!(muted_style.size, Px(14.0));
        assert_eq!(muted_style.line_height, Some(Px(20.0)));
        assert_eq!(muted_style.weight, FontWeight::NORMAL);

        let inline_code = render(|cx| inline_code("code").into_element(cx));
        let inline_code_container = container_props_of(&inline_code);
        assert_eq!(inline_code_container.padding.top, INLINE_CODE_PX_Y.into());
        assert_eq!(inline_code_container.padding.right, INLINE_CODE_PX_X.into());
        assert_eq!(
            inline_code_container.padding.bottom,
            INLINE_CODE_PX_Y.into()
        );
        assert_eq!(inline_code_container.padding.left, INLINE_CODE_PX_X.into());
        let inline_code_text = text_props_of(
            inline_code
                .children
                .first()
                .expect("expected inline code container child"),
        );
        let inline_code_style = inline_code_text
            .style
            .as_ref()
            .expect("expected inline code text style");
        assert_eq!(inline_code_style.font, FontId::monospace());
        assert_eq!(inline_code_style.size, Px(14.0));
        assert_eq!(inline_code_style.line_height, Some(Px(20.0)));
        assert_eq!(inline_code_style.weight, FontWeight::SEMIBOLD);
        assert_eq!(inline_code_text.wrap, TextWrap::None);
    }

    #[test]
    fn typography_blockquote_and_list_follow_inline_start_direction() {
        let blockquote_ltr = render(|cx| blockquote("Quote").into_element(cx));
        let blockquote_ltr_props = container_props_of(&blockquote_ltr);
        assert_eq!(
            blockquote_ltr_props.border.left,
            BLOCKQUOTE_BORDER_INLINE_START
        );
        assert_eq!(blockquote_ltr_props.border.right, Px(0.0));
        assert_eq!(
            blockquote_ltr_props.padding.left,
            BLOCKQUOTE_PADDING_INLINE_START.into()
        );
        assert_eq!(blockquote_ltr_props.padding.right, Px(0.0).into());

        let blockquote_rtl = render(|cx| {
            crate::direction::with_direction_provider(
                cx,
                crate::direction::LayoutDirection::Rtl,
                |cx| blockquote("Quote").into_element(cx),
            )
        });
        let blockquote_rtl_props = container_props_of(&blockquote_rtl);
        assert_eq!(blockquote_rtl_props.border.left, Px(0.0));
        assert_eq!(
            blockquote_rtl_props.border.right,
            BLOCKQUOTE_BORDER_INLINE_START
        );
        assert_eq!(blockquote_rtl_props.padding.left, Px(0.0).into());
        assert_eq!(
            blockquote_rtl_props.padding.right,
            BLOCKQUOTE_PADDING_INLINE_START.into()
        );

        let list_ltr = render(|cx| list(["One item"]).into_element(cx));
        let list_ltr_props = flex_props_of(&list_ltr);
        assert_eq!(
            list_ltr_props.layout.margin.left,
            MarginEdge::Px(LIST_MARGIN_INLINE_START)
        );
        assert_eq!(list_ltr_props.layout.margin.right, MarginEdge::Px(Px(0.0)));
        assert_eq!(list_ltr_props.gap, LIST_ITEM_GAP.into());
        let first_ltr_row = list_ltr.children.first().expect("expected list row");
        let first_ltr_bullet = text_props_of(
            first_ltr_row
                .children
                .first()
                .expect("expected bullet at inline start"),
        );
        assert_eq!(first_ltr_bullet.text.as_ref(), "•");

        let list_rtl = render(|cx| {
            crate::direction::with_direction_provider(
                cx,
                crate::direction::LayoutDirection::Rtl,
                |cx| list(["عنصر واحد"]).into_element(cx),
            )
        });
        let list_rtl_props = flex_props_of(&list_rtl);
        assert_eq!(list_rtl_props.layout.margin.left, MarginEdge::Px(Px(0.0)));
        assert_eq!(
            list_rtl_props.layout.margin.right,
            MarginEdge::Px(LIST_MARGIN_INLINE_START)
        );
        let first_rtl_row = list_rtl.children.first().expect("expected rtl list row");
        let last_rtl_bullet = text_props_of(
            first_rtl_row
                .children
                .last()
                .expect("expected bullet at rtl inline start"),
        );
        assert_eq!(last_rtl_bullet.text.as_ref(), "•");
    }
}
