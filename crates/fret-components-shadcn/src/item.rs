use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableProps,
    TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemVariant {
    #[default]
    Default,
    Outline,
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemSize {
    #[default]
    Default,
    Sm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemMediaVariant {
    #[default]
    Default,
    Icon,
    Image,
}

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn item_radius(theme: &Theme) -> Px {
    MetricRef::radius(Radius::Md).resolve(theme)
}

fn item_padding(theme: &Theme, size: ItemSize) -> Edges {
    match size {
        ItemSize::Default => {
            let p = MetricRef::space(Space::N4).resolve(theme);
            Edges::all(p)
        }
        ItemSize::Sm => Edges {
            top: MetricRef::space(Space::N3).resolve(theme),
            right: MetricRef::space(Space::N4).resolve(theme),
            bottom: MetricRef::space(Space::N3).resolve(theme),
            left: MetricRef::space(Space::N4).resolve(theme),
        },
    }
}

fn item_gap(theme: &Theme, size: ItemSize) -> Px {
    match size {
        ItemSize::Default => MetricRef::space(Space::N4).resolve(theme),
        ItemSize::Sm => MetricRef::space(Space::N2p5).resolve(theme),
    }
}

fn base_item_background(theme: &Theme, variant: ItemVariant) -> Option<Color> {
    match variant {
        ItemVariant::Default => None,
        ItemVariant::Outline => None,
        ItemVariant::Muted => Some(alpha(
            theme
                .color_by_key("muted")
                .unwrap_or(theme.colors.panel_background),
            0.5,
        )),
    }
}

fn base_item_border_color(theme: &Theme, variant: ItemVariant) -> Option<Color> {
    match variant {
        ItemVariant::Default => Some(Color::TRANSPARENT),
        ItemVariant::Outline => Some(
            theme
                .color_by_key("border")
                .unwrap_or(theme.colors.panel_border),
        ),
        ItemVariant::Muted => Some(Color::TRANSPARENT),
    }
}

#[derive(Debug, Clone)]
pub struct ItemGroup {
    children: Vec<AnyElement>,
}

impl ItemGroup {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
        let children = self.children;
        cx.column(
            ColumnProps {
                layout,
                gap: Px(0.0),
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

pub fn item_group<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    ItemGroup::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct ItemSeparator;

impl ItemSeparator {
    pub fn new() -> Self {
        Self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme
            .color_by_key("border")
            .unwrap_or(theme.colors.panel_border);
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(1.0))),
        );
        cx.container(
            ContainerProps {
                layout,
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }
}

impl Default for ItemSeparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ItemMedia {
    variant: ItemMediaVariant,
    children: Vec<AnyElement>,
}

impl ItemMedia {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            variant: ItemMediaVariant::default(),
            children,
        }
    }

    pub fn variant(mut self, variant: ItemMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let (size, chrome) = match self.variant {
            ItemMediaVariant::Default => (None, ChromeRefinement::default()),
            ItemMediaVariant::Icon => {
                let bg = alpha(
                    theme
                        .color_by_key("muted")
                        .unwrap_or(theme.colors.panel_background),
                    1.0,
                );
                let border = theme
                    .color_by_key("border")
                    .unwrap_or(theme.colors.panel_border);
                let chrome = ChromeRefinement::default()
                    .rounded(Radius::Sm)
                    .border_1()
                    .bg(ColorRef::Color(bg))
                    .border_color(ColorRef::Color(border));
                (Some(MetricRef::space(Space::N8).resolve(&theme)), chrome)
            }
            ItemMediaVariant::Image => {
                let chrome = ChromeRefinement::default().rounded(Radius::Sm);
                (Some(MetricRef::space(Space::N10).resolve(&theme)), chrome)
            }
        };

        let mut layout = LayoutRefinement::default().flex_none().flex_shrink_0();
        if let Some(s) = size {
            layout = layout.w_px(MetricRef::Px(s)).h_px(MetricRef::Px(s));
        }

        let mut props = decl_style::container_props(&theme, chrome, layout);
        if self.variant == ItemMediaVariant::Image {
            props.layout.overflow = fret_ui::element::Overflow::Clip;
        }

        let children = self.children;
        cx.container(props, move |cx| {
            let inner_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            vec![cx.flex(
                FlexProps {
                    layout: inner_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Debug, Clone)]
pub struct ItemContent {
    children: Vec<AnyElement>,
}

impl ItemContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());
        let children = self.children;
        cx.column(
            ColumnProps {
                layout,
                gap,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ItemActions {
    children: Vec<AnyElement>,
}

impl ItemActions {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default().flex_none().flex_shrink_0(),
        );
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ItemHeader {
    children: Vec<AnyElement>,
}

impl ItemHeader {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::SpaceBetween,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ItemFooter {
    children: Vec<AnyElement>,
}

impl ItemFooter {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        ItemHeader::new(self.children).into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct ItemTitle {
    text: Arc<str>,
}

impl ItemTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let px = theme
            .metric_by_key("component.item.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.item.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ItemDescription {
    text: Arc<str>,
}

impl ItemDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);
        let px = theme
            .metric_by_key("component.item.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.item.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    variant: ItemVariant,
    size: ItemSize,
    on_click: Option<CommandId>,
    enabled: bool,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl Item {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            variant: ItemVariant::default(),
            size: ItemSize::default(),
            on_click: None,
            enabled: true,
            children,
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let theme_for_content = theme.clone();

        let variant = self.variant;
        let size = self.size;
        let radius = item_radius(&theme);
        let padding = item_padding(&theme, size);
        let gap = item_gap(&theme, size);

        let border_color = base_item_border_color(&theme, variant).unwrap_or(Color::TRANSPARENT);
        let base_bg = base_item_background(&theme, variant);

        let accent = theme
            .color_by_key("accent")
            .unwrap_or(theme.colors.hover_background);
        let hover_bg = alpha(accent, 0.5);
        let pressed_bg = alpha(accent, 0.7);

        let layout = decl_style::layout_style(&theme, self.layout);

        let focus_ring = decl_style::focus_ring(&theme, radius);

        let children = std::rc::Rc::new(self.children);
        let children_for_content = children.clone();
        let enabled = self.enabled && self.on_click.is_some();
        let on_click = self.on_click;

        let content = move |cx: &mut ElementCx<'_, H>, hovered: bool, pressed: bool| {
            let bg = if !enabled {
                base_bg
            } else if pressed {
                Some(pressed_bg)
            } else if hovered {
                Some(hover_bg)
            } else {
                base_bg
            };

            let inner_layout = decl_style::layout_style(
                &theme_for_content,
                LayoutRefinement::default().size_full(),
            );
            let children = children_for_content.clone();

            vec![cx.container(
                ContainerProps {
                    layout,
                    padding,
                    background: bg,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |cx| {
                    let children = children.clone();
                    vec![cx.flex(
                        FlexProps {
                            layout: inner_layout,
                            direction: fret_core::Axis::Horizontal,
                            gap,
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| (*children).clone(),
                    )]
                },
            )]
        };

        if on_click.is_some() {
            cx.pressable(
                PressableProps {
                    layout,
                    enabled,
                    on_click,
                    focus_ring: Some(focus_ring),
                },
                move |cx, st| content(cx, st.hovered, st.pressed),
            )
        } else {
            cx.container(
                ContainerProps {
                    layout,
                    padding,
                    background: base_bg,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |cx| {
                    let inner_layout =
                        decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
                    vec![cx.flex(
                        FlexProps {
                            layout: inner_layout,
                            direction: fret_core::Axis::Horizontal,
                            gap,
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| (*children).clone(),
                    )]
                },
            )
        }
    }
}
