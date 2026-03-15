use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_icons::{IconId, ids};
use fret_runtime::{CommandId, Effect, WindowCommandGatingSnapshot};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableKeyActivation, PressableProps,
    SemanticsDecoration,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::motion::drive_tween_color_for_element;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Space, ui,
};

use crate::direction::use_direction;
use crate::{overlay_motion, rtl};

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default transition timing function: cubic-bezier(0.4, 0, 0.2, 1).
    // (Often described as `ease-in-out`-ish.)
    fret_ui_headless::easing::SHADCN_EASE.sample(t)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BreadcrumbItemKind {
    Link,
    Page,
    Ellipsis,
}

/// Separator renderer for a `Breadcrumb` list.
///
/// Upstream `BreadcrumbSeparator` accepts `children` (so the separator can be customized). We keep a
/// small typed surface that covers the upstream examples without forcing a `Fn` callback in the
/// common case.
#[derive(Debug, Clone)]
pub enum BreadcrumbSeparator {
    ChevronRight,
    Icon { icon: IconId, size: Px },
    Text(Arc<str>),
}

impl Default for BreadcrumbSeparator {
    fn default() -> Self {
        Self::ChevronRight
    }
}

/// A shadcn/ui v4-aligned breadcrumb builder.
///
/// Upstream composes `Breadcrumb` + `BreadcrumbList` + `BreadcrumbItem` + `BreadcrumbLink/Page`
/// + `BreadcrumbSeparator/Ellipsis`. In Fret we provide a compact builder surface that can render
///   the same visual/interaction result in a single declarative element tree.
#[derive(Debug, Clone, Default)]
pub struct Breadcrumb {
    items: Vec<BreadcrumbItem>,
    separator: BreadcrumbSeparator,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            separator: BreadcrumbSeparator::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn separator(mut self, separator: BreadcrumbSeparator) -> Self {
        self.separator = separator;
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        breadcrumb_with_patch(cx, self.items, self.separator, self.chrome, self.layout)
    }
}

fn breadcrumb_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Vec<BreadcrumbItem>,
    separator: BreadcrumbSeparator,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);
    let (gap, style, fg, muted, props) = {
        let theme = Theme::global(&*cx.app);

        let gap = theme
            .metric_by_key("component.breadcrumb.gap")
            // Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport,
            // so we default to the `sm` outcome (`gap-2.5`) for 1:1 geometry alignment.
            .unwrap_or_else(|| MetricRef::space(Space::N2p5).resolve(theme));

        let text_px = theme
            .metric_by_key("component.breadcrumb.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.breadcrumb.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let fg = theme.color_token("foreground");
        let muted = theme.color_token("muted-foreground");

        let mut style = typography::fixed_line_box_style(FontId::ui(), text_px, line_height);
        style.weight = FontWeight::NORMAL;

        let props = decl_style::container_props(theme, chrome, layout);
        (gap, style, fg, muted, props)
    };

    let mut children: Vec<AnyElement> = Vec::new();
    let n = items.len();
    for (i, mut item) in items.into_iter().enumerate() {
        let is_last = i + 1 == n;
        item.kind = match item.kind {
            BreadcrumbItemKind::Ellipsis => BreadcrumbItemKind::Ellipsis,
            _ if is_last => BreadcrumbItemKind::Page,
            _ => BreadcrumbItemKind::Link,
        };

        children.push(item.render(cx, &gating, &style, muted, fg));
        if !is_last {
            children.push(breadcrumb_separator(cx, &style, muted, &separator));
        }
    }

    cx.container(props, move |cx| {
        vec![
            cx.flex(
                FlexProps {
                    layout: Default::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: gap.into(),
                    padding: fret_core::Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: true,
                },
                move |_cx| children,
            )
            .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List)),
        ]
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Region)
            .label("breadcrumb"),
    )
}

#[derive(Clone)]
pub struct BreadcrumbItem {
    kind: BreadcrumbItemKind,
    label: Arc<str>,
    command: Option<CommandId>,
    on_activate: Option<OnActivate>,
    href: Option<Arc<str>>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
    disabled: bool,
    truncate: bool,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for BreadcrumbItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BreadcrumbItem")
            .field("kind", &self.kind)
            .field("label", &self.label)
            .field("command", &self.command)
            .field("on_activate", &self.on_activate.is_some())
            .field("href", &self.href)
            .field("target", &self.target)
            .field("rel", &self.rel)
            .field("disabled", &self.disabled)
            .field("truncate", &self.truncate)
            .field("layout", &self.layout)
            .finish()
    }
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            kind: BreadcrumbItemKind::Link,
            label: label.into(),
            command: None,
            on_activate: None,
            href: None,
            target: None,
            rel: None,
            disabled: false,
            truncate: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn ellipsis() -> Self {
        Self {
            kind: BreadcrumbItemKind::Ellipsis,
            label: Arc::from("…"),
            command: None,
            on_activate: None,
            href: None,
            target: None,
            rel: None,
            disabled: true,
            truncate: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Bind a stable action ID to this breadcrumb item (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn target(mut self, target: impl Into<Arc<str>>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn rel(mut self, rel: impl Into<Arc<str>>) -> Self {
        self.rel = Some(rel.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enables shadcn-aligned `truncate` behavior (single-line + ellipsis overflow).
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn render<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        gating: &WindowCommandGatingSnapshot,
        base_style: &TextStyle,
        muted: Color,
        fg: Color,
    ) -> AnyElement {
        let truncate = self.truncate;
        let layout = self.layout;
        match self.kind {
            BreadcrumbItemKind::Ellipsis => breadcrumb_ellipsis(cx, muted)
                .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::ListItem)),
            BreadcrumbItemKind::Page => {
                let label = self.label;
                breadcrumb_text(cx, label.clone(), base_style, fg, truncate, layout)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Link)
                            .label(label)
                            .disabled(true)
                            .read_only(true),
                    )
            }
            BreadcrumbItemKind::Link => {
                let disabled = self.disabled
                    || crate::command_gating::command_is_disabled_by_gating(
                        &*cx.app,
                        gating,
                        self.command.as_ref(),
                    );

                if disabled {
                    return breadcrumb_text(cx, self.label, base_style, muted, truncate, layout);
                }

                let command = self.command.clone();
                let href_for_action = self.href.clone();
                let href_for_semantics = self.href.clone();
                let target = self.target.clone();
                let rel = self.rel.clone();
                let on_activate = self.on_activate.clone();
                let should_fallback_open_url = command.is_none() && on_activate.is_none();
                let navigate_handler: Option<OnActivate> = if let Some(on_activate) = on_activate {
                    Some(on_activate)
                } else if should_fallback_open_url {
                    href_for_action
                        .clone()
                        .map(|url| open_url_on_activate(url, target.clone(), rel.clone()))
                } else {
                    None
                };

                if command.is_none() && navigate_handler.is_none() {
                    // Non-clickable link-like text (shadcn allows `<a>` without a URL).
                    return breadcrumb_link_text(
                        cx, self.label, base_style, muted, fg, false, truncate, layout,
                    );
                }

                let label = self.label.clone();
                let mut props = PressableProps::default();
                props.key_activation = PressableKeyActivation::EnterOnly;
                props.a11y.role = Some(SemanticsRole::Link);
                props.a11y.label = Some(label.clone());
                let mut element = cx.pressable(props, move |cx, st| {
                    if let Some(command) = command.clone() {
                        cx.pressable_dispatch_command_if_enabled(command);
                    }
                    if let Some(handler) = navigate_handler.clone() {
                        cx.pressable_on_activate(handler);
                    }
                    vec![breadcrumb_link_text(
                        cx,
                        label.clone(),
                        base_style,
                        muted,
                        fg,
                        st.hovered,
                        truncate,
                        layout,
                    )]
                });
                if let Some(href) = href_for_semantics {
                    element = element.attach_semantics(SemanticsDecoration::default().value(href));
                }
                element
            }
        }
    }
}

fn breadcrumb_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    base_style: &TextStyle,
    color: Color,
    truncate: bool,
    layout: LayoutRefinement,
) -> AnyElement {
    let (wrap, overflow) = if truncate {
        (TextWrap::None, TextOverflow::Ellipsis)
    } else {
        (TextWrap::Word, TextOverflow::Clip)
    };
    let mut el = ui::text(text)
        .text_size_px(base_style.size)
        .font_weight(base_style.weight)
        .text_color(ColorRef::Color(color))
        .layout(layout)
        .wrap(wrap)
        .overflow(overflow);

    if let Some(line_height) = base_style.line_height {
        el = el.line_height_px(line_height);
    }

    if let Some(letter_spacing_em) = base_style.letter_spacing_em {
        el = el.letter_spacing_em(letter_spacing_em);
    }

    el.into_element(cx)
}

fn breadcrumb_link_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    base_style: &TextStyle,
    muted: Color,
    fg: Color,
    hovered: bool,
    truncate: bool,
    layout: LayoutRefinement,
) -> AnyElement {
    let color = if hovered { fg } else { muted };
    breadcrumb_text(cx, text, base_style, color, truncate, layout)
}

fn breadcrumb_separator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    base_style: &TextStyle,
    muted: Color,
    separator: &BreadcrumbSeparator,
) -> AnyElement {
    let dir = use_direction(cx, None);
    let separator_el = match separator {
        BreadcrumbSeparator::ChevronRight => {
            // shadcn defaults to a right-pointing chevron; in RTL we mirror the visual separator
            // direction to match the reading flow.
            decl_icon::icon_with(
                cx,
                rtl::chevron_inline_end(dir),
                Some(Px(14.0)),
                Some(fret_ui_kit::ColorRef::Color(muted)),
            )
        }
        BreadcrumbSeparator::Icon { icon, size } => decl_icon::icon_with(
            cx,
            icon.clone(),
            Some(*size),
            Some(fret_ui_kit::ColorRef::Color(muted)),
        ),
        BreadcrumbSeparator::Text(text) => breadcrumb_text(
            cx,
            text.clone(),
            base_style,
            muted,
            false,
            LayoutRefinement::default(),
        ),
    };

    separator_el.attach_semantics(SemanticsDecoration::default().hidden(true))
}

fn breadcrumb_ellipsis<H: UiHost>(cx: &mut ElementContext<'_, H>, muted: Color) -> AnyElement {
    // shadcn uses a 36x36 box with a `MoreHorizontal` icon.
    // We keep the same footprint with a centered icon.
    let size = Theme::global(&*cx.app)
        .metric_by_key("component.breadcrumb.ellipsis_size")
        .unwrap_or(Px(36.0));

    let mut props = FlexProps {
        layout: Default::default(),
        direction: fret_core::Axis::Horizontal,
        gap: Px(0.0).into(),
        padding: fret_core::Edges::all(Px(0.0)).into(),
        justify: MainAlign::Center,
        align: CrossAlign::Center,
        wrap: false,
    };
    props.layout.size.width = fret_ui::element::Length::Px(size);
    props.layout.size.height = fret_ui::element::Length::Px(size);

    cx.flex(props, move |cx| {
        vec![decl_icon::icon_with(
            cx,
            ids::ui::MORE_HORIZONTAL,
            Some(Px(16.0)),
            Some(fret_ui_kit::ColorRef::Color(muted)),
        )]
    })
    .attach_semantics(SemanticsDecoration::default().hidden(true).label("More"))
}

/// Upstream-shaped Breadcrumb primitives (`Breadcrumb`/`BreadcrumbList`/`BreadcrumbItem`/...).
///
/// We keep these in a nested module for now to avoid a breaking rename of the existing
/// `fret_ui_shadcn::Breadcrumb` builder surface.
pub mod primitives {
    use super::*;

    use fret_ui::element::{ContainerProps, LayoutStyle, SizeStyle};

    fn collect_landed_children<H: UiHost, I, T>(
        cx: &mut ElementContext<'_, H>,
        children: I,
    ) -> Vec<AnyElement>
    where
        I: IntoIterator<Item = T>,
        T: IntoUiElement<H>,
    {
        children
            .into_iter()
            .map(|child| child.into_element(cx))
            .collect()
    }

    fn text_style(theme: &Theme) -> TextStyle {
        let text_px = theme
            .metric_by_key("component.breadcrumb.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.breadcrumb.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let mut style = typography::fixed_line_box_style(FontId::ui(), text_px, line_height);
        style.weight = FontWeight::NORMAL;
        style
    }

    fn colors(theme: &Theme) -> (Color, Color) {
        let fg = theme.color_token("foreground");
        let muted = theme.color_token("muted-foreground");
        (fg, muted)
    }

    #[derive(Debug, Clone, Default)]
    pub struct Breadcrumb {
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl Breadcrumb {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
            self.chrome = self.chrome.merge(chrome);
            self
        }

        pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
            self.layout = self.layout.merge(layout);
            self
        }

        #[track_caller]
        pub fn into_element<H: UiHost, I, TChild>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = TChild>,
            TChild: IntoUiElement<H>,
        {
            let props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(theme, self.chrome, self.layout)
            };
            cx.container(props, move |cx| {
                let built_children = children(cx);
                collect_landed_children(cx, built_children)
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Region)
                    .label("breadcrumb"),
            )
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct BreadcrumbList {
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl BreadcrumbList {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
            self.chrome = self.chrome.merge(chrome);
            self
        }

        pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
            self.layout = self.layout.merge(layout);
            self
        }

        #[track_caller]
        pub fn into_element<H: UiHost, I, TChild>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = TChild>,
            TChild: IntoUiElement<H>,
        {
            let (muted, style, gap, props) = {
                let theme = Theme::global(&*cx.app);
                let (_fg, muted) = colors(theme);
                let style = text_style(theme);
                let gap = theme
                    .metric_by_key("component.breadcrumb.gap")
                    // Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport,
                    // so we default to the `sm` outcome (`gap-2.5`) for 1:1 geometry alignment.
                    .unwrap_or_else(|| MetricRef::space(Space::N2p5).resolve(theme));
                let props = decl_style::container_props(theme, self.chrome, self.layout);
                (muted, style, gap, props)
            };

            cx.container(props, move |cx| {
                vec![
                    cx.flex(
                        FlexProps {
                            layout: Default::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: gap.into(),
                            padding: fret_core::Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: true,
                        },
                        move |cx| {
                            // Apply the list-level muted foreground by default; individual children
                            // (BreadcrumbLink/Page) can override it.
                            let built_children = children(cx);
                            let mut out = collect_landed_children(cx, built_children);
                            for el in &mut out {
                                // Best-effort: only text nodes support local color overrides.
                                if let fret_ui::element::ElementKind::Text(props) = &mut el.kind {
                                    if props.color.is_none() {
                                        props.color = Some(muted);
                                    }
                                    if props.style.is_none() {
                                        props.style = Some(style.clone());
                                    }
                                }
                            }
                            out
                        },
                    )
                    .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List)),
                ]
            })
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct BreadcrumbItem {
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl BreadcrumbItem {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
            self.chrome = self.chrome.merge(chrome);
            self
        }

        pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
            self.layout = self.layout.merge(layout);
            self
        }

        #[track_caller]
        pub fn into_element<H: UiHost, I, TChild>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = TChild>,
            TChild: IntoUiElement<H>,
        {
            let (item_gap, props) = {
                let theme = Theme::global(&*cx.app);
                let item_gap = theme
                    .metric_by_key("component.breadcrumb.item_gap")
                    .unwrap_or_else(|| MetricRef::space(Space::N1p5).resolve(theme));
                let props = decl_style::container_props(theme, self.chrome, self.layout);
                (item_gap, props)
            };

            cx.container(props, move |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: Default::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: item_gap.into(),
                        padding: fret_core::Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        let built_children = children(cx);
                        collect_landed_children(cx, built_children)
                    },
                )]
            })
            .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::ListItem))
        }
    }

    pub struct BreadcrumbLink {
        label: Arc<str>,
        children: Vec<AnyElement>,
        command: Option<CommandId>,
        on_activate: Option<OnActivate>,
        href: Option<Arc<str>>,
        target: Option<Arc<str>>,
        rel: Option<Arc<str>>,
        truncate: bool,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl std::fmt::Debug for BreadcrumbLink {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BreadcrumbLink")
                .field("label", &self.label)
                .field("children_len", &self.children.len())
                .field("command", &self.command)
                .field("on_activate", &self.on_activate.is_some())
                .field("href", &self.href)
                .field("target", &self.target)
                .field("rel", &self.rel)
                .field("truncate", &self.truncate)
                .field("chrome", &self.chrome)
                .field("layout", &self.layout)
                .finish()
        }
    }

    pub struct BreadcrumbLinkBuild<H, Children> {
        link: BreadcrumbLink,
        children: Children,
        _marker: PhantomData<fn() -> H>,
    }

    impl<H, Children> BreadcrumbLinkBuild<H, Children> {
        fn new(link: BreadcrumbLink, children: Children) -> Self {
            Self {
                link,
                children,
                _marker: PhantomData,
            }
        }

        /// Bind a stable action ID to this breadcrumb link (action-first authoring).
        ///
        /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
        /// through the existing command pipeline.
        pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
            self.link = self.link.action(action);
            self
        }

        pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
            self.link = self.link.on_click(command);
            self
        }

        pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
            self.link = self.link.on_activate(on_activate);
            self
        }

        pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
            self.link = self.link.href(href);
            self
        }

        pub fn target(mut self, target: impl Into<Arc<str>>) -> Self {
            self.link = self.link.target(target);
            self
        }

        pub fn rel(mut self, rel: impl Into<Arc<str>>) -> Self {
            self.link = self.link.rel(rel);
            self
        }

        pub fn truncate(mut self, truncate: bool) -> Self {
            self.link = self.link.truncate(truncate);
            self
        }

        pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
            self.link = self.link.refine_style(chrome);
            self
        }

        pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
            self.link = self.link.refine_layout(layout);
            self
        }
    }

    impl<H: UiHost, Children, I, TChild> BreadcrumbLinkBuild<H, Children>
    where
        Children: FnOnce(&mut ElementContext<'_, H>) -> I,
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>,
    {
        #[track_caller]
        pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let built_children = (self.children)(cx);
            let children = collect_landed_children(cx, built_children);
            self.link.children_raw(children).into_element(cx)
        }
    }

    impl BreadcrumbLink {
        pub fn new(label: impl Into<Arc<str>>) -> Self {
            Self {
                label: label.into(),
                children: Vec::new(),
                command: None,
                on_activate: None,
                href: None,
                target: None,
                rel: None,
                truncate: false,
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
            }
        }

        pub fn children<H: UiHost, I, TChild, Children>(
            self,
            children: Children,
        ) -> BreadcrumbLinkBuild<H, Children>
        where
            Children: FnOnce(&mut ElementContext<'_, H>) -> I,
            I: IntoIterator<Item = TChild>,
            TChild: IntoUiElement<H>,
        {
            BreadcrumbLinkBuild::new(self, children)
        }

        /// Explicit raw seam for pre-landed inline children.
        pub fn children_raw(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
            self.children = children.into_iter().collect();
            self
        }

        /// Bind a stable action ID to this breadcrumb link (action-first authoring).
        ///
        /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
        /// through the existing command pipeline.
        pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
            self.command = Some(action.into());
            self
        }

        pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
            self.command = Some(command.into());
            self
        }

        pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
            self.on_activate = Some(on_activate);
            self
        }

        pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
            self.href = Some(href.into());
            self
        }

        pub fn target(mut self, target: impl Into<Arc<str>>) -> Self {
            self.target = Some(target.into());
            self
        }

        pub fn rel(mut self, rel: impl Into<Arc<str>>) -> Self {
            self.rel = Some(rel.into());
            self
        }

        /// Enables shadcn-aligned `truncate` behavior (single-line + ellipsis overflow).
        pub fn truncate(mut self, truncate: bool) -> Self {
            self.truncate = truncate;
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

        #[track_caller]
        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let (style, fg, muted) = {
                let theme = Theme::global(&*cx.app);
                let style = text_style(theme);
                let (fg, muted) = colors(theme);
                (style, fg, muted)
            };
            let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);
            let label = self.label.clone();
            let children = self.children;
            let text_px = style.size;
            let font_weight = style.weight;
            let line_height = style.line_height;
            let letter_spacing_em = style.letter_spacing_em;
            let (wrap, overflow) = if self.truncate {
                (TextWrap::None, TextOverflow::Ellipsis)
            } else {
                (TextWrap::Word, TextOverflow::Clip)
            };

            let disabled_by_gating = crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &gating,
                self.command.as_ref(),
            );

            let chrome = self.chrome;
            let layout = self.layout;
            let command = self.command.clone();
            let href_for_action = self.href.clone();
            let href_for_semantics = self.href.clone();
            let target = self.target.clone();
            let rel = self.rel.clone();
            let on_activate = self.on_activate.clone();
            let should_fallback_open_url = command.is_none() && on_activate.is_none();

            if (command.is_some() && !disabled_by_gating)
                || href_for_action.is_some()
                || on_activate.is_some()
            {
                let mut props = PressableProps::default();
                props.key_activation = PressableKeyActivation::EnterOnly;
                props.a11y.role = Some(SemanticsRole::Link);
                props.a11y.label = Some(label.clone());

                let pressable_props = props;
                let mut element = cx.pressable_with_id_props(move |cx, st, id| {
                    cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                    if let Some(on_activate) = on_activate.clone() {
                        cx.pressable_on_activate(on_activate);
                    } else if should_fallback_open_url && let Some(href) = href_for_action.clone() {
                        cx.pressable_on_activate(open_url_on_activate(
                            href,
                            target.clone(),
                            rel.clone(),
                        ));
                    }

                    let duration = overlay_motion::shadcn_motion_duration_150(cx);
                    let target_color = if st.hovered { fg } else { muted };
                    let fg_motion = drive_tween_color_for_element(
                        cx,
                        id,
                        "breadcrumb.link.fg",
                        target_color,
                        duration,
                        tailwind_transition_ease_in_out,
                    );
                    let color = fg_motion.value;
                    let mut custom_children = if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    };
                    let children = vec![cx.container(
                        {
                            let theme = Theme::global(&*cx.app);
                            decl_style::container_props(theme, chrome.clone(), layout.clone())
                        },
                        move |cx| {
                            if let Some(children) = custom_children.take() {
                                current_color::scope_children(
                                    cx,
                                    ColorRef::Color(color),
                                    move |_cx| children,
                                )
                            } else {
                                let mut text = ui::text(label.clone())
                                    .text_size_px(text_px)
                                    .font_weight(font_weight)
                                    .text_color(ColorRef::Color(color))
                                    .wrap(wrap)
                                    .overflow(overflow);

                                if let Some(line_height) = line_height {
                                    text = text.line_height_px(line_height);
                                }

                                if let Some(letter_spacing_em) = letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }

                                vec![text.into_element(cx)]
                            }
                        },
                    )];

                    (pressable_props, children)
                });

                if let Some(href) = href_for_semantics {
                    element = element.attach_semantics(SemanticsDecoration::default().value(href));
                }

                element
            } else {
                let props = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::container_props(theme, chrome, layout)
                };
                let mut custom_children = if children.is_empty() {
                    None
                } else {
                    Some(children)
                };
                cx.container(props, move |cx| {
                    if let Some(children) = custom_children.take() {
                        current_color::scope_children(cx, ColorRef::Color(muted), move |_cx| {
                            children
                        })
                    } else {
                        let mut text = ui::text(label)
                            .text_size_px(text_px)
                            .font_weight(font_weight)
                            .text_color(ColorRef::Color(muted))
                            .wrap(wrap)
                            .overflow(overflow);

                        if let Some(line_height) = line_height {
                            text = text.line_height_px(line_height);
                        }

                        if let Some(letter_spacing_em) = letter_spacing_em {
                            text = text.letter_spacing_em(letter_spacing_em);
                        }

                        vec![text.into_element(cx)]
                    }
                })
            }
        }
    }

    #[derive(Debug)]
    pub struct BreadcrumbPage {
        label: Arc<str>,
        children: Vec<AnyElement>,
        truncate: bool,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    pub struct BreadcrumbPageBuild<H, Children> {
        page: BreadcrumbPage,
        children: Children,
        _marker: PhantomData<fn() -> H>,
    }

    impl<H, Children> BreadcrumbPageBuild<H, Children> {
        fn new(page: BreadcrumbPage, children: Children) -> Self {
            Self {
                page,
                children,
                _marker: PhantomData,
            }
        }

        pub fn truncate(mut self, truncate: bool) -> Self {
            self.page = self.page.truncate(truncate);
            self
        }

        pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
            self.page = self.page.refine_style(chrome);
            self
        }

        pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
            self.page = self.page.refine_layout(layout);
            self
        }
    }

    impl<H: UiHost, Children, I, TChild> BreadcrumbPageBuild<H, Children>
    where
        Children: FnOnce(&mut ElementContext<'_, H>) -> I,
        I: IntoIterator<Item = TChild>,
        TChild: IntoUiElement<H>,
    {
        #[track_caller]
        pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let built_children = (self.children)(cx);
            let children = collect_landed_children(cx, built_children);
            self.page.children_raw(children).into_element(cx)
        }
    }

    impl BreadcrumbPage {
        pub fn new(label: impl Into<Arc<str>>) -> Self {
            Self {
                label: label.into(),
                children: Vec::new(),
                truncate: false,
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
            }
        }

        pub fn children<H: UiHost, I, TChild, Children>(
            self,
            children: Children,
        ) -> BreadcrumbPageBuild<H, Children>
        where
            Children: FnOnce(&mut ElementContext<'_, H>) -> I,
            I: IntoIterator<Item = TChild>,
            TChild: IntoUiElement<H>,
        {
            BreadcrumbPageBuild::new(self, children)
        }

        /// Explicit raw seam for pre-landed inline children.
        pub fn children_raw(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
            self.children = children.into_iter().collect();
            self
        }

        /// Enables shadcn-aligned `truncate` behavior (single-line + ellipsis overflow).
        pub fn truncate(mut self, truncate: bool) -> Self {
            self.truncate = truncate;
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

        #[track_caller]
        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let (style, fg, props) = {
                let theme = Theme::global(&*cx.app);
                let style = text_style(theme);
                let (fg, _muted) = colors(theme);
                let props = decl_style::container_props(theme, self.chrome, self.layout);
                (style, fg, props)
            };
            let label = self.label;
            let children = self.children;
            let label_for_semantics = label.clone();
            let text_px = style.size;
            let font_weight = style.weight;
            let line_height = style.line_height;
            let letter_spacing_em = style.letter_spacing_em;
            let (wrap, overflow) = if self.truncate {
                (TextWrap::None, TextOverflow::Ellipsis)
            } else {
                (TextWrap::Word, TextOverflow::Clip)
            };
            let mut custom_children = if children.is_empty() {
                None
            } else {
                Some(children)
            };
            cx.container(props, move |cx| {
                if let Some(children) = custom_children.take() {
                    current_color::scope_children(cx, ColorRef::Color(fg), move |_cx| children)
                } else {
                    let mut text = ui::text(label.clone())
                        .text_size_px(text_px)
                        .font_weight(font_weight)
                        .text_color(ColorRef::Color(fg))
                        .wrap(wrap)
                        .overflow(overflow);

                    if let Some(line_height) = line_height {
                        text = text.line_height_px(line_height);
                    }

                    if let Some(letter_spacing_em) = letter_spacing_em {
                        text = text.letter_spacing_em(letter_spacing_em);
                    }

                    vec![text.into_element(cx)]
                }
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Link)
                    .label(label_for_semantics)
                    .disabled(true)
                    .read_only(true),
            )
        }
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    pub enum BreadcrumbSeparatorKind {
        #[default]
        ChevronRight,
        Slash,
        Icon {
            icon: IconId,
            size: Px,
        },
    }

    #[derive(Debug, Clone)]
    pub struct BreadcrumbSeparator {
        kind: BreadcrumbSeparatorKind,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl Default for BreadcrumbSeparator {
        fn default() -> Self {
            Self {
                kind: BreadcrumbSeparatorKind::default(),
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
            }
        }
    }

    impl BreadcrumbSeparator {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn kind(mut self, kind: BreadcrumbSeparatorKind) -> Self {
            self.kind = kind;
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

        #[track_caller]
        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let (muted, props) = {
                let theme = Theme::global(&*cx.app);
                let (_fg, muted) = colors(theme);
                let props = ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: fret_ui::element::Length::Px(Px(14.0)),
                            height: fret_ui::element::Length::Px(Px(14.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..decl_style::container_props(theme, self.chrome, self.layout)
                };
                (muted, props)
            };

            let dir = crate::direction::use_direction(cx, None);
            let (icon, size) = match self.kind {
                BreadcrumbSeparatorKind::ChevronRight => {
                    (crate::rtl::chevron_inline_end(dir), Px(14.0))
                }
                BreadcrumbSeparatorKind::Slash => (ids::ui::SLASH, Px(14.0)),
                BreadcrumbSeparatorKind::Icon { icon, size } => (icon, size),
            };

            // Upstream applies `[&>svg]:size-3.5` (14px) by default.
            let icon_el = decl_icon::icon_with(cx, icon, Some(size), Some(ColorRef::Color(muted)));

            // Ensure the separator is a "leaf-sized" node in layouts that scan by size.
            cx.container(props, move |_cx| vec![icon_el])
                .attach_semantics(SemanticsDecoration::default().hidden(true))
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct BreadcrumbEllipsis {
        size: Option<Px>,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl BreadcrumbEllipsis {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn size(mut self, size: Px) -> Self {
            self.size = Some(size);
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

        #[track_caller]
        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let (muted, size, wrapper_props) = {
                let theme = Theme::global(&*cx.app);
                let (_fg, muted) = colors(theme);
                let size = self.size.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.breadcrumb.ellipsis_size")
                        .unwrap_or(Px(36.0))
                });
                let wrapper_props = decl_style::container_props(theme, self.chrome, self.layout);
                (muted, size, wrapper_props)
            };

            let mut props = FlexProps {
                layout: Default::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0).into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            };
            props.layout.size.width = fret_ui::element::Length::Px(size);
            props.layout.size.height = fret_ui::element::Length::Px(size);

            let icon = decl_icon::icon_with(
                cx,
                ids::ui::MORE_HORIZONTAL,
                Some(Px(16.0)),
                Some(fret_ui_kit::ColorRef::Color(muted)),
            );

            cx.container(wrapper_props, move |cx| {
                vec![cx.flex(props, move |_cx| vec![icon])]
            })
            .attach_semantics(SemanticsDecoration::default().hidden(true).label("More"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, Size};
    use fret_ui::UiTree;
    use fret_ui::elements::GlobalElementId;
    use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: fret_core::Size::new(Px(48.0), Px(16.0)),
                    baseline: Px(12.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
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

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn breadcrumb_item_href_attaches_link_value_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let href: Arc<str> = Arc::from("https://example.com");
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-breadcrumb-href-semantics",
            |cx| {
                vec![
                    Breadcrumb::new()
                        .items([
                            BreadcrumbItem::new("Home")
                                .href(href.clone())
                                .on_activate(Arc::new(|_host, _acx, _reason| {})),
                            BreadcrumbItem::new("Breadcrumb"),
                        ])
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Link && n.label.as_deref() == Some("Home"))
            .expect("expected Home breadcrumb link semantics node");
        assert_eq!(node.value.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn breadcrumb_builder_root_and_current_page_emit_expected_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-breadcrumb-builder-semantics",
            |cx| {
                vec![
                    Breadcrumb::new()
                        .items([
                            BreadcrumbItem::new("Home").href("/"),
                            BreadcrumbItem::new("Components"),
                        ])
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::Region && n.label.as_deref() == Some("breadcrumb")
            }),
            "expected breadcrumb root region semantics"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| { n.role == SemanticsRole::List && n.parent.is_some() }),
            "expected breadcrumb list semantics"
        );
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::Link
                    && n.flags.disabled
                    && n.label.as_deref() == Some("Components")
            }),
            "expected current page to approximate upstream disabled current-page link semantics with the item label"
        );
    }

    #[test]
    fn breadcrumb_primitives_emit_list_item_and_hidden_affordance_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-breadcrumb-primitives-semantics",
            |cx| {
                vec![primitives::Breadcrumb::new().into_element(cx, |cx| {
                    vec![primitives::BreadcrumbList::new().into_element(cx, |cx| {
                        vec![
                            primitives::BreadcrumbItem::new().into_element(cx, |cx| {
                                vec![primitives::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
                            }),
                            primitives::BreadcrumbSeparator::new().into_element(cx),
                            primitives::BreadcrumbEllipsis::new().into_element(cx),
                        ]
                    })]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::List),
            "expected primitives::BreadcrumbList to emit list semantics"
        );
        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::ListItem),
            "expected primitives::BreadcrumbItem to emit listitem semantics"
        );
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::Link
                    && n.flags.disabled
                    && n.flags.read_only
                    && n.label.as_deref() == Some("Breadcrumb")
            }),
            "expected primitives::BreadcrumbPage to approximate upstream disabled current-page link semantics with a stable label"
        );
        assert!(
            snap.nodes.iter().any(|n| n.flags.hidden),
            "expected breadcrumb separator/ellipsis affordances to be hidden from semantics"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.flags.hidden && n.label.as_deref() == Some("More")),
            "expected breadcrumb ellipsis to expose the upstream fallback label while remaining hidden"
        );
    }

    #[test]
    fn breadcrumb_link_children_with_command_keep_link_semantics_without_open_url_fallback() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(120.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-breadcrumb-link-children-command",
            |cx| {
                vec![
                    primitives::BreadcrumbLink::new("Home")
                        .href("/")
                        .on_click("ui_gallery.app.open")
                        .children(|cx| [cx.text("Home")])
                        .into_element(cx)
                        .test_id("breadcrumb-link-children-command"),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let _ = app.flush_effects();

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("breadcrumb-link-children-command"))
            .expect("expected breadcrumb link semantics node");
        assert_eq!(node.role, SemanticsRole::Link);
        assert_eq!(node.label.as_deref(), Some("Home"));
        assert_eq!(node.value.as_deref(), Some("/"));

        let center = Point::new(
            Px(node.bounds.origin.x.0 + node.bounds.size.width.0 * 0.5),
            Px(node.bounds.origin.y.0 + node.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
                is_click: true,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|effect| matches!(effect, Effect::OpenUrl { .. })),
            "expected breadcrumb link command path to suppress href fallback OpenUrl effects"
        );
    }

    fn color_eq_eps(a: Color, b: Color, eps: f32) -> bool {
        (a.r - b.r).abs() <= eps
            && (a.g - b.g).abs() <= eps
            && (a.b - b.b).abs() <= eps
            && (a.a - b.a).abs() <= eps
    }

    fn find_first_text_color(el: &AnyElement) -> Option<Color> {
        match &el.kind {
            fret_ui::element::ElementKind::Text(props) => props.color,
            _ => el.children.iter().find_map(find_first_text_color),
        }
    }

    #[test]
    fn breadcrumb_link_hover_color_tweens_instead_of_snapping() {
        use fret_runtime::FrameId;

        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(120.0)),
        );

        let theme = Theme::global(&app).snapshot();
        let fg = theme.color_token("foreground");
        let muted = theme.color_token("muted-foreground");

        let link_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let color_out: Rc<Cell<Option<Color>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            link_id_out: Rc<Cell<Option<GlobalElementId>>>,
            color_out: Rc<Cell<Option<Color>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "breadcrumb-link-hover-color-tween",
                move |cx| {
                    let el = primitives::BreadcrumbLink::new("Home")
                        .href("https://example.com")
                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                        .into_element(cx);
                    link_id_out.set(Some(el.id));
                    color_out.set(find_first_text_color(&el));
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        }

        // Frame 1: baseline (not hovered).
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            link_id_out.clone(),
            color_out.clone(),
        );
        let c0 = color_out.get().expect("c0");
        assert!(
            color_eq_eps(c0, muted, 1e-6),
            "expected base link color to be muted; got c0={c0:?} muted={muted:?}"
        );

        let id = link_id_out.get().expect("link id");
        let node = fret_ui::elements::node_for_element(&mut app, window, id).expect("link node");
        let b = ui.debug_node_bounds(node).expect("link bounds");
        let center = Point::new(
            Px(b.origin.x.0 + b.size.width.0 * 0.5),
            Px(b.origin.y.0 + b.size.height.0 * 0.5),
        );

        // Hover.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover applied; color should tween.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            link_id_out.clone(),
            color_out.clone(),
        );
        let c1 = color_out.get().expect("c1");
        assert!(
            !color_eq_eps(c1, muted, 1e-6) && !color_eq_eps(c1, fg, 1e-6),
            "expected hover color to tween (intermediate), got c1={c1:?} muted={muted:?} fg={fg:?}"
        );

        // Settle to foreground.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                link_id_out.clone(),
                color_out.clone(),
            );
        }
        let cf = color_out.get().expect("cf");
        assert!(
            color_eq_eps(cf, fg, 1e-4),
            "expected hover color to settle to foreground; got cf={cf:?} fg={fg:?}"
        );
    }
}
