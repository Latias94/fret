//! AI Elements-aligned queue surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/queue.tsx`.

use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, DecorationLineStyle, Edges, FontId, FontWeight, ImageId, Px,
    SemanticsRole, StrikethroughStyle, TextOverflow, TextPaintStyle, TextSpan, TextStyle, TextWrap,
    Transform2D, ViewportFit,
};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, ImageProps, LayoutStyle, Length, MarginEdge,
    ScrollAxis, SemanticsDecoration, StyledTextProps, TextProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, LayoutRefinement, MetricRef, Radius, Space};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent, CollapsibleTrigger,
    ScrollArea,
};

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn resolve_border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_required("border"))
}

fn resolve_background(theme: &Theme) -> Color {
    theme.color_required("background")
}

fn resolve_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_required("muted.background"))
}

fn resolve_muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn rounded_xl(theme: &Theme) -> Px {
    // shadcn token model:
    // - `rounded-lg` ~= `--radius`
    // - `rounded-xl` ~= `--radius + 4px`
    let base_radius = theme.metric_required("metric.radius.lg");
    Px(base_radius.0 + 4.0)
}

fn rich_strikethrough(text: &Arc<str>, strike_color: Color) -> AttributedText {
    let span = TextSpan {
        len: text.len(),
        shaping: Default::default(),
        paint: TextPaintStyle {
            strikethrough: Some(StrikethroughStyle {
                color: Some(strike_color),
                style: DecorationLineStyle::Solid,
            }),
            ..Default::default()
        },
    };
    AttributedText::new(Arc::clone(text), Arc::<[TextSpan]>::from([span]))
}

#[derive(Clone)]
/// AI Elements-aligned `Queue` surface.
pub struct Queue {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Queue {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let bg = resolve_background(&theme);
        let border = resolve_border(&theme);
        let radius = rounded_xl(&theme);

        // queue.tsx:
        // - `flex flex-col gap-2 rounded-xl border border-border bg-background px-3 pt-2 pb-2 shadow-xs`
        let chrome = ChromeRefinement::default()
            .radius(MetricRef::Px(radius))
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .px(Space::N3)
            .pt(Space::N2)
            .pb(Space::N2)
            .merge(self.chrome);

        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.shadow = Some(decl_style::shadow_xs(&theme, radius));

        let children = self.children;
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |_cx| children,
        );

        let mut root = cx.container(props, move |_cx| vec![content]);
        if let Some(test_id) = self.test_id {
            root = root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }
        root
    }
}

#[derive(Debug, Clone)]
pub struct QueueSectionState {
    pub open: Model<bool>,
    pub is_open: bool,
}

#[derive(Clone)]
/// AI Elements-aligned collapsible section container (`QueueSection`).
pub struct QueueSection {
    open: Option<Model<bool>>,
    default_open: bool,
    disabled: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
}

impl std::fmt::Debug for QueueSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSection")
            .field("open", &"<model>")
            .field("default_open", &self.default_open)
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .field("force_mount_content", &self.force_mount_content)
            .finish()
    }
}

impl Default for QueueSection {
    fn default() -> Self {
        Self {
            open: None,
            default_open: true,
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
        }
    }
}

impl QueueSection {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open: Some(open),
            ..Default::default()
        }
    }

    pub fn uncontrolled(default_open: bool) -> Self {
        Self {
            open: None,
            default_open,
            ..Default::default()
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.force_mount_content = force_mount_content;
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, QueueSectionState) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let mut collapsible = if let Some(open) = self.open {
            Collapsible::new(open)
        } else {
            Collapsible::uncontrolled(self.default_open)
        };

        collapsible = collapsible
            .disabled(self.disabled)
            .force_mount_content(self.force_mount_content)
            .refine_style(self.chrome)
            .refine_layout(self.layout);

        collapsible.into_element_with_open_model(
            cx,
            move |cx, open, is_open| trigger(cx, QueueSectionState { open, is_open }),
            content,
        )
    }
}

#[derive(Clone)]
/// AI Elements-aligned section header trigger (`QueueSectionTrigger`).
pub struct QueueSectionTrigger {
    open: Model<bool>,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueSectionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSectionTrigger")
            .field("open", &"<model>")
            .field("a11y_label", &self.a11y_label.as_deref())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueSectionTrigger {
    pub fn new(open: Model<bool>, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            open,
            a11y_label: None,
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted = resolve_muted(&theme);
        let bg = alpha(muted, 0.4);
        let hover_bg = muted;

        let layout = self.layout;
        let children = self.children;
        let inner = cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
            let mut props = ContainerProps::default();
            props.layout = decl_style::layout_style(&theme, layout);
            props.padding = Edges::symmetric(
                MetricRef::space(Space::N3).resolve(&theme),
                MetricRef::space(Space::N2).resolve(&theme),
            );
            props.background = Some(if hovered { hover_bg } else { bg });
            props.corner_radii = Corners::all(MetricRef::radius(Radius::Sm).resolve(&theme));

            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items(Items::Center)
                    .justify_between(),
                move |_cx| children.clone(),
            );

            vec![cx.container(props, move |_cx| vec![row])]
        });

        let a11y_label = self
            .a11y_label
            .unwrap_or_else(|| Arc::<str>::from("Queue section"));
        let trigger = CollapsibleTrigger::new(self.open, vec![inner]).a11y_label(a11y_label);
        let mut el = trigger.into_element(cx, is_open);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Button)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Clone)]
/// AI Elements-aligned label content (`QueueSectionLabel`).
pub struct QueueSectionLabel {
    label: Arc<str>,
    count: Option<u32>,
    icon: Option<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueSectionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSectionLabel")
            .field("label", &self.label.as_ref())
            .field("count", &self.count)
            .field("has_icon", &self.icon.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueSectionLabel {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            count: None,
            icon: None,
            test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn icon(mut self, icon: AnyElement) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = resolve_muted_fg(&theme);
        let icon_size = Px(16.0);

        // Upstream: `group-data-[state=closed]:-rotate-90`.
        let chevron_rotation = if is_open { 0.0 } else { -90.0 };
        let center = fret_core::Point::new(Px(8.0), Px(8.0));
        let chevron_transform = Transform2D::rotation_about_degrees(chevron_rotation, center);
        let chevron = cx.visual_transform_props(
            VisualTransformProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(icon_size))
                        .h_px(MetricRef::Px(icon_size))
                        .flex_shrink_0(),
                ),
                transform: chevron_transform,
            },
            move |cx| {
                vec![decl_icon::icon_with(
                    cx,
                    fret_icons::ids::ui::CHEVRON_DOWN,
                    Some(icon_size),
                    Some(ColorRef::Color(fg)),
                )]
            },
        );

        let label_text: Arc<str> = if let Some(count) = self.count {
            Arc::<str>::from(format!("{count} {}", self.label))
        } else {
            self.label.clone()
        };

        let text = cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
            text: label_text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(
                    theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
                ),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        });

        let mut children = vec![chevron];
        if let Some(icon) = self.icon {
            children.push(icon);
        }
        children.push(text);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items_center(),
            move |_cx| children,
        );

        if let Some(test_id) = self.test_id {
            row.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            row
        }
    }
}

#[derive(Clone)]
/// AI Elements-aligned collapsible content area (`QueueSectionContent`).
pub struct QueueSectionContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueSectionContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSectionContent")
            .field("children_len", &self.children.len())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueSectionContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CollapsibleContent::new(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

#[derive(Clone)]
/// AI Elements-aligned scrollable list container (`QueueList`).
pub struct QueueList {
    children: Vec<AnyElement>,
    max_height: Px,
    viewport_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueList")
            .field("children_len", &self.children.len())
            .field("max_height", &self.max_height)
            .field("viewport_test_id", &self.viewport_test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueList {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            max_height: Px(160.0), // `max-h-40`
            viewport_test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0().mt(Space::N2),
        }
    }

    pub fn max_height_px(mut self, max_height: Px) -> Self {
        self.max_height = Px(max_height.0.max(0.0));
        self
    }

    pub fn viewport_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let max_h = self.max_height;

        let list = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0),
            move |_cx| self.children,
        )
        .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List));

        let mut viewport = ContainerProps::default();
        viewport.layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());
        viewport.padding = Edges {
            top: Px(0.0),
            right: MetricRef::space(Space::N4).resolve(&theme),
            bottom: Px(0.0),
            left: Px(0.0),
        };

        let viewport = cx.container(viewport, move |_cx| vec![list]);

        let mut scroll = ScrollArea::new(vec![viewport])
            .axis(ScrollAxis::Y)
            .refine_layout(
                self.layout
                    .merge(LayoutRefinement::default().max_h(MetricRef::Px(max_h))),
            );

        if let Some(test_id) = self.viewport_test_id {
            scroll = scroll.viewport_test_id(test_id);
        }

        scroll.into_element(cx)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QueueItemState {
    pub hovered: bool,
}

#[derive(Clone)]
/// AI Elements-aligned item wrapper (`QueueItem`).
pub struct QueueItem {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItem")
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Default for QueueItem {
    fn default() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }
}

impl QueueItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        content: impl FnOnce(&mut ElementContext<'_, H>, QueueItemState) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted = resolve_muted(&theme);

        // queue.tsx:
        // - `group flex flex-col gap-1 rounded-md px-3 py-1 text-sm transition-colors hover:bg-muted`
        let layout = self.layout;
        let test_id = self.test_id;
        let mut hover = HoverRegionProps::default();
        hover.layout = decl_style::layout_style(&theme, layout);

        let el = cx.hover_region(hover, move |cx, hovered| {
            let mut chrome = ContainerProps::default();
            chrome.padding = Edges::symmetric(
                MetricRef::space(Space::N3).resolve(&theme),
                MetricRef::space(Space::N1).resolve(&theme),
            );
            chrome.background = hovered.then_some(muted);
            chrome.corner_radii = Corners::all(MetricRef::radius(Radius::Sm).resolve(&theme));

            let children = content(cx, QueueItemState { hovered });
            let col = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N1),
                move |_cx| children,
            );
            vec![cx.container(chrome, move |_cx| vec![col])]
        });

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListItem);
        if let Some(test_id) = test_id {
            semantics = semantics.test_id(test_id);
        }
        el.attach_semantics(semantics)
    }
}

#[derive(Clone)]
/// AI Elements-aligned status indicator (`QueueItemIndicator`).
pub struct QueueItemIndicator {
    completed: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemIndicator")
            .field("completed", &self.completed)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemIndicator {
    pub fn new() -> Self {
        Self {
            completed: false,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);

        let border = if self.completed {
            alpha(muted_fg, 0.2)
        } else {
            alpha(muted_fg, 0.5)
        };
        let bg = self.completed.then_some(alpha(muted_fg, 0.1));

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(10.0)))
                .h_px(MetricRef::Px(Px(10.0)))
                .mt(Space::N0p5)
                .flex_shrink_0()
                .merge(self.layout),
        );
        props.border = Edges::all(Px(1.0));
        props.border_color = Some(border);
        props.background = bg;
        props.corner_radii = Corners::all(Px(999.0));

        let mut el = cx.container(props, |_cx| Vec::<AnyElement>::new());
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Clone)]
/// AI Elements-aligned item main content (`QueueItemContent`).
pub struct QueueItemContent {
    text: Arc<str>,
    completed: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemContent")
            .field("text", &self.text.as_ref())
            .field("completed", &self.completed)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemContent {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            completed: false,
            test_id: None,
            layout: LayoutRefinement::default()
                .min_w_0()
                .basis_0()
                .flex_grow(1.0),
        }
    }

    pub fn completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);

        let fg = if self.completed {
            alpha(muted_fg, 0.5)
        } else {
            muted_fg
        };

        let style = TextStyle {
            font: FontId::default(),
            size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(
                theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
            ),
            letter_spacing_em: None,
        };

        let el = if self.completed {
            let rich = rich_strikethrough(&self.text, fg);
            cx.styled_text_props(StyledTextProps {
                layout: decl_style::layout_style(&theme, self.layout),
                rich,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
            })
        } else {
            cx.text_props(TextProps {
                layout: decl_style::layout_style(&theme, self.layout),
                text: self.text,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
            })
        };

        if let Some(test_id) = self.test_id {
            el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Text)
                    .test_id(test_id),
            )
        } else {
            el.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Text))
        }
    }
}

#[derive(Clone)]
/// AI Elements-aligned item description (`QueueItemDescription`).
pub struct QueueItemDescription {
    text: Arc<str>,
    completed: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemDescription")
            .field("text", &self.text.as_ref())
            .field("completed", &self.completed)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            completed: false,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);

        let fg = if self.completed {
            alpha(muted_fg, 0.4)
        } else {
            muted_fg
        };

        let style = TextStyle {
            font: FontId::default(),
            size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_XS_PX),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(
                theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT),
            ),
            letter_spacing_em: None,
        };

        let mut layout = decl_style::layout_style(&theme, self.layout);
        layout.margin.left = MarginEdge::Px(MetricRef::space(Space::N6).resolve(&theme));

        let el = if self.completed {
            let rich = rich_strikethrough(&self.text, fg);
            cx.styled_text_props(StyledTextProps {
                layout,
                rich,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            })
        } else {
            cx.text_props(TextProps {
                layout,
                text: self.text,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            })
        };

        if let Some(test_id) = self.test_id {
            el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Text)
                    .test_id(test_id),
            )
        } else {
            el.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Text))
        }
    }
}

#[derive(Clone)]
/// AI Elements-aligned actions container (`QueueItemActions`).
pub struct QueueItemActions {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemActions")
            .field("children_len", &self.children.len())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N1)
                .items_center(),
            move |_cx| self.children,
        )
    }
}

pub type OnQueueItemActionActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Clone)]
/// AI Elements-aligned action button (`QueueItemAction`).
pub struct QueueItemAction {
    label: Arc<str>,
    children: Vec<AnyElement>,
    on_activate: Option<OnQueueItemActionActivate>,
    visible: bool,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemAction")
            .field("label", &self.label.as_ref())
            .field("children_len", &self.children.len())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("visible", &self.visible)
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            on_activate: None,
            visible: true,
            disabled: false,
            test_id: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            chrome: ChromeRefinement::default().rounded(Radius::Sm).p(Space::N1),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn on_activate(mut self, on_activate: OnQueueItemActionActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    /// Mirrors the upstream default `opacity-0` + `group-hover:opacity-100` outcome.
    ///
    /// When `false`, the action is not rendered (best-effort parity for Fret).
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut btn = Button::new(self.label)
            .variant(self.variant)
            .size(self.size)
            .disabled(self.disabled)
            .children(self.children)
            .refine_style(self.chrome)
            .refine_layout(self.layout);

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(Arc::new(move |host, action_cx, _reason| {
                on_activate(host, action_cx);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        }

        let btn = btn.into_element(cx);
        let opacity = if self.visible { 1.0 } else { 0.0 };
        let interactive = self.visible;

        cx.interactivity_gate(true, interactive, move |cx| {
            vec![cx.opacity(opacity, move |_cx| vec![btn])]
        })
    }
}

#[derive(Clone)]
/// AI Elements-aligned attachment row (`QueueItemAttachment`).
pub struct QueueItemAttachment {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemAttachment")
            .field("children_len", &self.children.len())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemAttachment {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().mt(Space::N1),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items(Items::Center),
            move |_cx| self.children,
        )
    }
}

#[derive(Clone)]
/// AI Elements-aligned square preview image (`QueueItemImage`).
pub struct QueueItemImage {
    image: ImageId,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemImage")
            .field("image", &self.image)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemImage {
    pub fn new(image: ImageId) -> Self {
        Self {
            image,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = resolve_border(&theme);
        let radius = MetricRef::radius(Radius::Sm).resolve(&theme);

        let mut wrapper = ContainerProps::default();
        wrapper.layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(32.0)))
                .h_px(MetricRef::Px(Px(32.0)))
                .merge(self.layout),
        );
        wrapper.border = Edges::all(Px(1.0));
        wrapper.border_color = Some(border);
        wrapper.corner_radii = Corners::all(radius);
        wrapper.snap_to_device_pixels = true;

        let mut image_layout = LayoutStyle::default();
        image_layout.size.width = Length::Fill;
        image_layout.size.height = Length::Fill;

        let img = cx.image_props(ImageProps {
            layout: image_layout,
            image: self.image,
            fit: ViewportFit::Cover,
            opacity: 1.0,
            uv: None,
        });

        let mut el = cx.container(wrapper, move |_cx| vec![img]);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Clone)]
/// AI Elements-aligned file attachment badge (`QueueItemFile`).
pub struct QueueItemFile {
    filename: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for QueueItemFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueItemFile")
            .field("filename", &self.filename.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl QueueItemFile {
    pub fn new(filename: impl Into<Arc<str>>) -> Self {
        Self {
            filename: filename.into(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = resolve_border(&theme);
        let muted = resolve_muted(&theme);
        let fg = resolve_muted_fg(&theme);

        let icon = decl_icon::icon_with(
            cx,
            fret_icons::IconId::new("lucide.paperclip"),
            Some(Px(12.0)),
            Some(ColorRef::Color(fg)),
        );

        let filename = cx.text_props(TextProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .max_w(MetricRef::Px(Px(100.0)))
                    .min_w_0(),
            ),
            text: self.filename,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_XS_PX),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(
                    theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT),
                ),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N1)
                .items_center(),
            move |_cx| vec![icon, filename],
        );

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(&theme, self.layout);
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N2).resolve(&theme),
            MetricRef::space(Space::N1).resolve(&theme),
        );
        props.border = Edges::all(Px(1.0));
        props.border_color = Some(border);
        props.background = Some(muted);
        props.corner_radii = Corners::all(MetricRef::radius(Radius::Sm).resolve(&theme));

        let mut el = cx.container(props, move |_cx| vec![row]);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}
