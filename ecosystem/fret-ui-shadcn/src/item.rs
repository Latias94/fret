use std::sync::Arc;

use fret_core::{Color, Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::{CommandId, Effect};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, GridProps, MainAlign,
    PressableKeyActivation, PressableProps, SemanticsDecoration,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space, ui,
};

#[derive(Debug, Default)]
struct ItemSizeProviderState {
    current: Option<ItemSize>,
}

fn item_size_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> ItemSize {
    cx.inherited_state_where::<ItemSizeProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
        .unwrap_or_default()
}

#[track_caller]
fn with_item_size_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    size: ItemSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(ItemSizeProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(size);
        prev
    });
    let out = f(cx);
    cx.with_state(ItemSizeProviderState::default, |st| {
        st.current = prev;
    });
    out
}

/// Build an item and its parts inside a size provider.
///
/// This avoids footguns where callers construct `ItemMedia` / `ItemContent` outside the `Item`
/// subtree and accidentally miss size-dependent defaults (shadcn `group-data-[size=...]/item:*`).
#[track_caller]
pub fn item_sized<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    size: ItemSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let children = with_item_size_provider(cx, size, |cx| {
        f(cx).into_iter().collect::<Vec<AnyElement>>()
    });
    Item::new(children).size(size).into_element(cx)
}

#[derive(Debug, Clone)]
pub enum ItemRender {
    Link {
        href: Arc<str>,
        target: Option<Arc<str>>,
        rel: Option<Arc<str>>,
    },
}

fn open_url_on_activate(
    url: Arc<str>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
) -> OnActivate {
    Arc::new(move |host, _acx, _reason| {
        host.push_effect(Effect::OpenUrl {
            url: url.to_string(),
            target: target.as_ref().map(|v| v.to_string()),
            rel: rel.as_ref().map(|v| v.to_string()),
        });
    })
}

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
    Xs,
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
        // shadcn/ui v4 (`repo-ref/ui/apps/v4/registry/styles/style-*.css`):
        // - default: `gap-3.5`
        // - sm: `gap-3.5`
        // - xs: `gap-2.5`
        ItemSize::Default => MetricRef::space(Space::N3p5).resolve(theme),
        ItemSize::Sm => MetricRef::space(Space::N3p5).resolve(theme),
        ItemSize::Xs => MetricRef::space(Space::N2p5).resolve(theme),
    }
}

fn base_item_background(theme: &Theme, variant: ItemVariant) -> Option<Color> {
    match variant {
        ItemVariant::Default => None,
        ItemVariant::Outline => None,
        ItemVariant::Muted => Some(alpha(
            theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted.background")),
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
                .unwrap_or_else(|| theme.color_token("border")),
        ),
        ItemVariant::Muted => Some(Color::TRANSPARENT),
    }
}

#[derive(Debug)]
pub struct ItemGroup {
    kind: ItemGroupKind,
    size: ItemSize,
    layout: LayoutRefinement,
    gap: Option<Px>,
    children: Vec<AnyElement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemGroupKind {
    #[default]
    Column,
    Grid {
        cols: u16,
    },
}

impl ItemGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            kind: ItemGroupKind::Column,
            size: ItemSize::Default,
            layout: LayoutRefinement::default().w_full(),
            gap: None.into(),
            children,
        }
    }

    /// Sets the sizing intent for the group gap.
    ///
    /// In upstream shadcn/ui this is expressed as `has-data-[size=sm]:gap-2.5` and
    /// `has-data-[size=xs]:gap-2` in CSS. In Fret we can't infer this from descendants, so the
    /// group exposes an explicit knob.
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }

    pub fn grid(mut self, cols: u16) -> Self {
        self.kind = ItemGroupKind::Grid { cols: cols.max(1) };
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = Some(gap);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (layout, gap) = {
            let theme = Theme::global(&*cx.app);
            let layout = decl_style::layout_style(theme, self.layout);
            let gap = self.gap.unwrap_or_else(|| match self.size {
                ItemSize::Default => MetricRef::space(Space::N4).resolve(theme),
                ItemSize::Sm => MetricRef::space(Space::N2p5).resolve(theme),
                ItemSize::Xs => MetricRef::space(Space::N2).resolve(theme),
            });
            (layout, gap)
        };
        let children = self.children;

        let el = match self.kind {
            ItemGroupKind::Column => cx.column(
                ColumnProps {
                    layout,
                    gap: gap.into(),
                    ..Default::default()
                },
                move |_cx| children,
            ),
            ItemGroupKind::Grid { cols } => cx.grid(
                GridProps {
                    layout,
                    cols,
                    gap: gap.into(),
                    ..Default::default()
                },
                move |_cx| children,
            ),
        };

        el.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List))
    }
}

pub fn item_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    ItemGroup::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct ItemSeparator;

impl ItemSeparator {
    pub fn new() -> Self {
        Self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (border, mut layout, margin_y) = {
            let theme = Theme::global(&*cx.app);
            let border = theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_token("border"));
            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(1.0))),
            );
            let margin_y = MetricRef::space(Space::N2).resolve(theme);
            (border, layout, margin_y)
        };
        layout.margin.top = margin_y.into();
        layout.margin.bottom = margin_y.into();
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

#[derive(Debug)]
pub struct ItemMedia {
    variant: ItemMediaVariant,
    item_size: Option<ItemSize>,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl ItemMedia {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            variant: ItemMediaVariant::default(),
            item_size: None,
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn variant(mut self, variant: ItemMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Explicit size override for this media part.
    ///
    /// Prefer using `item_sized(...)` so all parts share the same size scope.
    pub fn item_size(mut self, size: ItemSize) -> Self {
        self.item_size = Some(size);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let variant = self.variant;
        let item_size = self.item_size.unwrap_or_else(|| item_size_in_scope(cx));
        let user_layout = self.layout;
        let children = self.children;
        let (props, inner_layout, gap) = {
            let theme = Theme::global(&*cx.app);

            let (size, chrome) = match variant {
                ItemMediaVariant::Default | ItemMediaVariant::Icon => {
                    (None, ChromeRefinement::default())
                }
                ItemMediaVariant::Image => {
                    // shadcn/ui v4:
                    // - base: `size-10`
                    // - `group-data-[size=sm]/item:size-8`
                    // - `group-data-[size=xs]/item:size-6`
                    let side = match item_size {
                        ItemSize::Default => MetricRef::space(Space::N10).resolve(theme),
                        ItemSize::Sm => MetricRef::space(Space::N8).resolve(theme),
                        ItemSize::Xs => MetricRef::space(Space::N6).resolve(theme),
                    };
                    let chrome = ChromeRefinement::default().rounded(Radius::Sm);
                    (Some(side), chrome)
                }
            };

            let mut layout = LayoutRefinement::default()
                .merge(user_layout)
                .flex_none()
                .flex_shrink_0();
            if let Some(s) = size {
                layout = layout.w_px(MetricRef::Px(s)).h_px(MetricRef::Px(s));
            }

            let mut props = decl_style::container_props(theme, chrome, layout);
            if variant == ItemMediaVariant::Image {
                props.layout.overflow = fret_ui::element::Overflow::Clip;
            }

            let inner_layout = if size.is_some() {
                decl_style::layout_style(theme, LayoutRefinement::default().size_full())
            } else {
                decl_style::layout_style(theme, LayoutRefinement::default())
            };
            let gap = MetricRef::space(Space::N2).resolve(theme);
            (props, inner_layout, gap)
        };
        cx.container(props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: inner_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Debug)]
pub struct ItemContent {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
    gap: Option<Px>,
    item_size: Option<ItemSize>,
    justify: MainAlign,
    align: CrossAlign,
}

impl ItemContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            layout: LayoutRefinement::default()
                .flex_1()
                .min_w_0()
                .overflow_hidden(),
            children,
            gap: None.into(),
            item_size: None,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }

    /// Explicit size override for this content part.
    ///
    /// Prefer using `item_sized(...)` so all parts share the same size scope.
    pub fn item_size(mut self, size: ItemSize) -> Self {
        self.item_size = Some(size);
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn justify(mut self, justify: MainAlign) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: CrossAlign) -> Self {
        self.align = align;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let size = self.item_size.unwrap_or_else(|| item_size_in_scope(cx));
            let gap = self.gap.unwrap_or_else(|| match size {
                ItemSize::Default | ItemSize::Sm => MetricRef::space(Space::N1).resolve(theme),
                ItemSize::Xs => MetricRef::space(Space::N0p5).resolve(theme),
            });
            let layout = decl_style::layout_style(theme, self.layout);
            (gap, layout)
        };
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: self.justify,
                align: self.align,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug)]
pub struct ItemActions {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl ItemActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let gap = MetricRef::space(Space::N2).resolve(theme);
            let layout = decl_style::layout_style(theme, self.layout);
            (gap, layout)
        };
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug)]
pub struct ItemHeader {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl ItemHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            layout: LayoutRefinement::default()
                .w_full()
                .basis(LengthRefinement::Fill),
            children,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let gap = MetricRef::space(Space::N2).resolve(theme);
            let layout = decl_style::layout_style(theme, self.layout);
            (gap, layout)
        };
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::SpaceBetween,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug)]
pub struct ItemFooter {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl ItemFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            layout: LayoutRefinement::default()
                .w_full()
                .basis(LengthRefinement::Fill),
            children,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let gap = MetricRef::space(Space::N2).resolve(theme);
            let layout = decl_style::layout_style(theme, self.layout);
            (gap, layout)
        };
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::SpaceBetween,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));
            let px = theme
                .metric_by_key("component.item.title_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.item.title_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

        ui::text(cx, self.text)
            .text_size_px(px)
            .fixed_line_box_px(line_height)
            .line_box_in_bounds()
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme
                .color_by_key("muted.foreground")
                .or_else(|| theme.color_by_key("muted-foreground"))
                .unwrap_or_else(|| theme.color_token("muted.foreground"));
            let px = theme
                .metric_by_key("component.item.description_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.item.description_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };
        let max_h = Px(line_height.0 * 2.0);

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .max_h(max_h)
            .overflow_hidden()
            .into_element(cx)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemStyle {
    pub background: Option<ColorRef>,
    pub border_color: Option<ColorRef>,
}

impl ItemStyle {
    pub fn background(mut self, background: ColorRef) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_color(mut self, border_color: ColorRef) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        self
    }
}

pub struct Item {
    variant: ItemVariant,
    size: ItemSize,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    enabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    render: Option<ItemRender>,
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Item")
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("enabled", &self.enabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("render", &self.render)
            .field("children_len", &self.children.len())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Item {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            variant: ItemVariant::default(),
            size: ItemSize::default(),
            on_click: None,
            on_activate: None,
            enabled: true,
            a11y_label: None,
            test_id: None,
            render: None,
            children,
            chrome: ChromeRefinement::default(),
            // shadcn/ui v4: Item root uses `w-full` by default.
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

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn render(mut self, render: ItemRender) -> Self {
        self.render = Some(render);
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

    pub fn style(mut self, style: ItemStyle) -> Self {
        if let Some(background) = style.background {
            self.chrome.background = Some(background);
        }
        if let Some(border_color) = style.border_color {
            self.chrome.border_color = Some(border_color);
        }
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let variant = self.variant;
        let size = self.size;
        let layout = self.layout;
        let (
            gap,
            border_color,
            focus_border_color,
            base_bg,
            pressable_layout,
            pressable_size,
            radius,
            focus_ring,
        ) = {
            let theme = Theme::global(&*cx.app);
            let gap = item_gap(theme, size);
            let border_color = base_item_border_color(theme, variant).unwrap_or(Color::TRANSPARENT);
            let focus_border_color = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("ring"));
            let base_bg = base_item_background(theme, variant);
            let pressable_layout = decl_style::layout_style(theme, layout.clone());
            let pressable_size = pressable_layout.size;
            let radius = item_radius(theme);
            let focus_ring = decl_style::focus_ring(theme, radius);
            (
                gap,
                border_color,
                focus_border_color,
                base_bg,
                pressable_layout,
                pressable_size,
                radius,
                focus_ring,
            )
        };

        let children = self.children;
        let enabled = self.enabled;
        let on_click = self.on_click;
        let on_activate = self.on_activate;
        let a11y_label = self.a11y_label;
        let test_id = self.test_id;
        let user_chrome = self.chrome;
        let user_bg_override = user_chrome.background.is_some();
        let user_border_override = user_chrome.border_color.is_some();
        let should_fallback_open_url = on_click.is_none() && on_activate.is_none();
        let (render_role, render_key_activation, render_on_activate) = match self.render {
            Some(ItemRender::Link { href, target, rel }) => (
                Some(SemanticsRole::Link),
                PressableKeyActivation::EnterOnly,
                should_fallback_open_url.then(|| open_url_on_activate(href, target, rel)),
            ),
            None => (None, PressableKeyActivation::EnterAndSpace, None),
        };
        let padding = match size {
            // shadcn/ui v4 (`repo-ref/ui/apps/v4/registry/styles/style-*.css`):
            // - default: `px-4 py-3.5`
            // - sm: `px-3.5 py-3`
            // - xs: `px-3 py-2.5`
            ItemSize::Default => ChromeRefinement::default().px(Space::N4).py(Space::N3p5),
            ItemSize::Sm => ChromeRefinement::default().px(Space::N3p5).py(Space::N3),
            ItemSize::Xs => ChromeRefinement::default().px(Space::N3).py(Space::N2p5),
        };

        if on_click.is_some() || on_activate.is_some() || render_role.is_some() {
            let children = children;
            control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_if_enabled_opt(on_click);
                if let Some(on_activate) = on_activate.clone() {
                    cx.pressable_on_activate(on_activate);
                } else if let Some(on_activate) = render_on_activate.clone() {
                    cx.pressable_on_activate(on_activate);
                }

                let focused = st.focused && enabled;

                let bg = base_bg;

                let mut chrome = padding.clone().merge(ChromeRefinement {
                    radius: Some(MetricRef::Px(radius)),
                    border_width: Some(MetricRef::Px(Px(1.0))),
                    ..Default::default()
                });

                if !user_bg_override {
                    chrome.background = bg.map(ColorRef::Color);
                }
                if !user_border_override {
                    chrome.border_color = Some(ColorRef::Color(if focused {
                        focus_border_color
                    } else {
                        border_color
                    }));
                }
                chrome = chrome.merge(user_chrome.clone());

                let (chrome_props, inner_layout) = {
                    let theme = Theme::global(&*cx.app);
                    let mut chrome_props =
                        decl_style::container_props(theme, chrome, LayoutRefinement::default());
                    chrome_props.layout.size = pressable_size;
                    let inner_layout =
                        decl_style::layout_style(theme, LayoutRefinement::default().w_full());
                    (chrome_props, inner_layout)
                };

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled,
                    focus_ring: Some(focus_ring),
                    key_activation: render_key_activation,
                    a11y: fret_ui::element::PressableA11y {
                        role: render_role,
                        label: a11y_label.clone(),
                        test_id: test_id.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                (pressable_props, chrome_props, move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: inner_layout,
                            direction: fret_core::Axis::Horizontal,
                            gap: gap.into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: true,
                        },
                        move |_cx| children,
                    )]
                })
            })
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

            let props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(theme, chrome, layout)
            };

            let children = children;
            cx.container(props, move |cx| {
                let inner_layout = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::layout_style(theme, LayoutRefinement::default().w_full())
                };
                vec![cx.flex(
                    FlexProps {
                        layout: inner_layout,
                        direction: fret_core::Axis::Horizontal,
                        gap: gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: true,
                    },
                    move |_cx| children,
                )]
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_ui::element::{ElementKind, PressableKeyActivation, SpacingLength};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(240.0)),
        )
    }

    fn find_element_by_test_id<'a>(el: &'a AnyElement, test_id: &str) -> Option<&'a AnyElement> {
        if el
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return Some(el);
        }
        match &el.kind {
            ElementKind::Semantics(props) => {
                if props.test_id.as_deref() == Some(test_id) {
                    return Some(el);
                }
            }
            _ => {}
        }
        for child in &el.children {
            if let Some(found) = find_element_by_test_id(child, test_id) {
                return Some(found);
            }
        }
        None
    }

    fn item_chrome_container(el: &AnyElement) -> &fret_ui::element::ContainerProps {
        match &el.kind {
            ElementKind::Pressable(_) => match el.children.first().map(|c| &c.kind) {
                Some(ElementKind::Container(props)) => props,
                other => panic!("expected chrome container child, got {other:?}"),
            },
            ElementKind::Container(props) => props,
            other => panic!("expected item root to be pressable or container, got {other:?}"),
        }
    }

    #[test]
    fn item_default_padding_matches_shadcn_registry_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = bounds();

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let content =
                ItemContent::new([ItemTitle::new("Title").into_element(cx)]).into_element(cx);
            Item::new([content])
                .on_activate(Arc::new(|_host, _cx, _reason| {}))
                .into_element(cx)
        });

        let chrome = item_chrome_container(&element);
        let theme = Theme::global(&app);
        let px = MetricRef::space(Space::N4).resolve(theme);
        let py = MetricRef::space(Space::N3p5).resolve(theme);

        assert_eq!(chrome.padding.left, SpacingLength::Px(px));
        assert_eq!(chrome.padding.right, SpacingLength::Px(px));
        assert_eq!(chrome.padding.top, SpacingLength::Px(py));
        assert_eq!(chrome.padding.bottom, SpacingLength::Px(py));
    }

    #[test]
    fn item_link_stamps_link_role_and_enter_only_key_activation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = bounds();

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let content =
                ItemContent::new([ItemTitle::new("Title").into_element(cx)]).into_element(cx);
            Item::new([content])
                .render(ItemRender::Link {
                    href: Arc::from("https://example.com"),
                    target: None,
                    rel: None,
                })
                .into_element(cx)
        });

        let ElementKind::Pressable(pressable) = &element.kind else {
            panic!("expected link item to render as pressable");
        };
        assert_eq!(pressable.a11y.role, Some(SemanticsRole::Link));
        assert_eq!(pressable.key_activation, PressableKeyActivation::EnterOnly);
    }

    #[test]
    fn item_sized_provides_size_defaults_to_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = bounds();

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            item_sized(cx, ItemSize::Xs, |cx| {
                let media = ItemMedia::new([ui::text(cx, "m").into_element(cx)])
                    .variant(ItemMediaVariant::Image)
                    .into_element(cx)
                    .test_id("media");
                let content = ItemContent::new([ui::text(cx, "c").into_element(cx)])
                    .into_element(cx)
                    .test_id("content");
                [media, content]
            })
        });

        let media = find_element_by_test_id(&element, "media").expect("media element");
        let ElementKind::Container(media_props) = &media.kind else {
            panic!("expected media to be a container");
        };
        let theme = Theme::global(&app);
        let expected_media_side = MetricRef::space(Space::N6).resolve(theme);
        assert_eq!(
            media_props.layout.size.width,
            fret_ui::element::Length::Px(expected_media_side)
        );
        assert_eq!(
            media_props.layout.size.height,
            fret_ui::element::Length::Px(expected_media_side)
        );

        let content = find_element_by_test_id(&element, "content").expect("content element");
        let ElementKind::Flex(content_props) = &content.kind else {
            panic!("expected content to be a flex element");
        };
        let expected_gap = MetricRef::space(Space::N0p5).resolve(theme);
        assert_eq!(content_props.gap, SpacingLength::Px(expected_gap));
    }

    #[test]
    fn item_group_gap_defaults_follow_size_intent() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = bounds();

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ItemGroup::new(std::iter::empty())
                .size(ItemSize::Sm)
                .into_element(cx)
        });

        let ElementKind::Column(props) = &element.kind else {
            panic!("expected item group default to be a column");
        };
        let theme = Theme::global(&app);
        let expected_gap = MetricRef::space(Space::N2p5).resolve(theme);
        assert_eq!(props.gap, SpacingLength::Px(expected_gap));
    }

    #[test]
    fn item_separator_has_vertical_margin_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = bounds();

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ItemSeparator::new().into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected separator to be a container");
        };
        let theme = Theme::global(&app);
        let expected_my = MetricRef::space(Space::N2).resolve(theme);
        assert_eq!(
            props.layout.margin.top,
            fret_ui::element::MarginEdge::Px(expected_my)
        );
        assert_eq!(
            props.layout.margin.bottom,
            fret_ui::element::MarginEdge::Px(expected_my)
        );
    }
}
