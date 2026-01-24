use std::sync::Arc;

use fret_core::{Color, Edges, Px, TextOverflow, TextWrap};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

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
                .unwrap_or_else(|| theme.color_required("muted.background")),
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
                .unwrap_or_else(|| theme.color_required("border")),
        ),
        ItemVariant::Muted => Some(Color::TRANSPARENT),
    }
}

#[derive(Debug, Clone)]
pub struct ItemGroup {
    children: Vec<AnyElement>,
}

impl ItemGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

pub fn item_group<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    ItemGroup::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct ItemSeparator;

impl ItemSeparator {
    pub fn new() -> Self {
        Self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_required("border"));
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            variant: ItemMediaVariant::default(),
            children,
        }
    }

    pub fn variant(mut self, variant: ItemMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let (size, chrome) = match self.variant {
            ItemMediaVariant::Default => (None, ChromeRefinement::default()),
            ItemMediaVariant::Icon => {
                let bg = theme
                    .color_by_key("muted")
                    .unwrap_or_else(|| theme.color_required("muted.background"));
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_required("border"));
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_required("foreground"));
        let px = theme
            .metric_by_key("component.item.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.item.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .truncate()
            .into_element(cx)
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or_else(|| theme.color_required("muted.foreground"));
        let px = theme
            .metric_by_key("component.item.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.item.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    variant: ItemVariant,
    size: ItemSize,
    on_click: Option<CommandId>,
    enabled: bool,
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Item {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            variant: ItemVariant::default(),
            size: ItemSize::default(),
            on_click: None,
            enabled: true,
            children,
            chrome: ChromeRefinement::default(),
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let variant = self.variant;
        let size = self.size;
        let gap = item_gap(&theme, size);

        let border_color = base_item_border_color(&theme, variant).unwrap_or(Color::TRANSPARENT);
        let base_bg = base_item_background(&theme, variant);

        let accent = theme
            .color_by_key("accent")
            .unwrap_or_else(|| theme.color_required("accent"));
        let hover_bg = alpha(accent, 0.5);
        let pressed_bg = alpha(accent, 0.7);

        let layout = self.layout;
        let pressable_layout = decl_style::layout_style(&theme, layout.clone());

        let radius = item_radius(&theme);
        let focus_ring = decl_style::focus_ring(&theme, radius);

        let children = std::rc::Rc::new(self.children);
        let mut enabled = self.enabled;
        let on_click = self.on_click;
        if let Some(cmd) = on_click.as_ref() {
            enabled = enabled && cx.command_is_enabled(cmd);
        }
        let user_chrome = self.chrome;
        let user_bg_override = user_chrome.background.is_some();
        let user_border_override = user_chrome.border_color.is_some();
        let padding = match size {
            ItemSize::Default => ChromeRefinement::default().px(Space::N4).py(Space::N4),
            ItemSize::Sm => ChromeRefinement::default().px(Space::N4).py(Space::N3),
        };

        if on_click.is_some() {
            cx.pressable(
                PressableProps {
                    layout: pressable_layout,
                    enabled,
                    focus_ring: Some(focus_ring),
                    ..Default::default()
                },
                move |cx, st| {
                    cx.pressable_dispatch_command_if_enabled_opt(on_click);

                    let hovered = st.hovered && enabled;
                    let pressed = st.pressed && enabled;

                    let bg = if !enabled {
                        base_bg
                    } else if pressed {
                        Some(pressed_bg)
                    } else if hovered {
                        Some(hover_bg)
                    } else {
                        base_bg
                    };

                    let mut chrome = padding.clone().merge(ChromeRefinement {
                        radius: Some(MetricRef::Px(radius)),
                        border_width: Some(MetricRef::Px(Px(1.0))),
                        ..Default::default()
                    });

                    if !user_bg_override {
                        chrome.background = bg.map(ColorRef::Color);
                    }
                    if !user_border_override {
                        chrome.border_color = Some(ColorRef::Color(border_color));
                    }
                    chrome = chrome.merge(user_chrome.clone());

                    let mut props =
                        decl_style::container_props(&theme, chrome, LayoutRefinement::default());
                    props.layout.size = pressable_layout.size;

                    let inner_layout =
                        decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

                    let children = children.clone();
                    vec![cx.container(props, move |cx| {
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
                    })]
                },
            )
        } else {
            let mut chrome = padding.merge(ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                ..Default::default()
            });

            if !user_bg_override {
                chrome.background = base_bg.map(ColorRef::Color);
            }
            if !user_border_override {
                chrome.border_color = Some(ColorRef::Color(border_color));
            }
            chrome = chrome.merge(user_chrome);

            let props = decl_style::container_props(&theme, chrome, layout);

            cx.container(props, move |cx| {
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
            })
        }
    }
}
