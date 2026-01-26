use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_icons::{IconId, ids};
use fret_runtime::{CommandId, WindowCommandGatingSnapshot};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space, ui};

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
    let theme = Theme::global(&*cx.app).clone();

    let gap = theme
        .metric_by_key("component.breadcrumb.gap")
        // Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport,
        // so we default to the `sm` outcome (`gap-2.5`) for 1:1 geometry alignment.
        .unwrap_or_else(|| MetricRef::space(Space::N2p5).resolve(&theme));

    let text_px = theme
        .metric_by_key("component.breadcrumb.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.breadcrumb.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    let fg = theme.color_required("foreground");
    let muted = theme.color_required("muted-foreground");

    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);

    let style = TextStyle {
        font: FontId::default(),
        size: text_px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
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

    cx.container(
        decl_style::container_props(&theme, chrome, layout),
        move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: Default::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap,
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: true,
                },
                move |_cx| children,
            )]
        },
    )
}

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    kind: BreadcrumbItemKind,
    label: Arc<str>,
    command: Option<CommandId>,
    disabled: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            kind: BreadcrumbItemKind::Link,
            label: label.into(),
            command: None,
            disabled: false,
        }
    }

    pub fn ellipsis() -> Self {
        Self {
            kind: BreadcrumbItemKind::Ellipsis,
            label: Arc::from("…"),
            command: None,
            disabled: true,
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        match self.kind {
            BreadcrumbItemKind::Ellipsis => breadcrumb_ellipsis(cx, muted),
            BreadcrumbItemKind::Page => breadcrumb_text(cx, self.label, base_style, fg),
            BreadcrumbItemKind::Link => {
                let disabled = self.disabled
                    || crate::command_gating::command_is_disabled_by_gating(
                        &*cx.app,
                        gating,
                        self.command.as_ref(),
                    );

                if disabled {
                    return breadcrumb_text(cx, self.label, base_style, muted);
                }

                let Some(command) = self.command else {
                    // Non-clickable link-like text (shadcn allows `<a>` without a URL).
                    return breadcrumb_link_text(cx, self.label, base_style, muted, fg, false);
                };

                cx.pressable(PressableProps::default(), move |cx, st| {
                    cx.pressable_dispatch_command_if_enabled(command.clone());
                    vec![breadcrumb_link_text(
                        cx, self.label, base_style, muted, fg, st.hovered,
                    )]
                })
            }
        }
    }
}

fn breadcrumb_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    base_style: &TextStyle,
    color: Color,
) -> AnyElement {
    let mut el = ui::text(cx, text)
        .text_size_px(base_style.size)
        .font_weight(base_style.weight)
        .text_color(ColorRef::Color(color))
        .wrap(TextWrap::Word)
        .overflow(TextOverflow::Clip);

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
) -> AnyElement {
    let color = if hovered { fg } else { muted };
    breadcrumb_text(cx, text, base_style, color)
}

fn breadcrumb_separator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    base_style: &TextStyle,
    muted: Color,
    separator: &BreadcrumbSeparator,
) -> AnyElement {
    match separator {
        BreadcrumbSeparator::ChevronRight => {
            // shadcn uses lucide `ChevronRight` at `size-3.5` (~14px).
            decl_icon::icon_with(
                cx,
                ids::ui::CHEVRON_RIGHT,
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
        BreadcrumbSeparator::Text(text) => breadcrumb_text(cx, text.clone(), base_style, muted),
    }
}

fn breadcrumb_ellipsis<H: UiHost>(cx: &mut ElementContext<'_, H>, muted: Color) -> AnyElement {
    // shadcn uses a 36x36 box with a `MoreHorizontal` icon.
    // We keep the same footprint with a centered icon.
    let theme = Theme::global(&*cx.app).clone();
    let size = theme
        .metric_by_key("component.breadcrumb.ellipsis_size")
        .unwrap_or(Px(36.0));

    let mut props = FlexProps {
        layout: Default::default(),
        direction: fret_core::Axis::Horizontal,
        gap: Px(0.0),
        padding: fret_core::Edges::all(Px(0.0)),
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
}

/// Upstream-shaped Breadcrumb primitives (`Breadcrumb`/`BreadcrumbList`/`BreadcrumbItem`/...).
///
/// We keep these in a nested module for now to avoid a breaking rename of the existing
/// `fret_ui_shadcn::Breadcrumb` builder surface.
pub mod primitives {
    use super::*;

    use fret_ui::element::{ContainerProps, LayoutStyle, SizeStyle};

    fn text_style(theme: &Theme) -> TextStyle {
        let text_px = theme
            .metric_by_key("component.breadcrumb.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.breadcrumb.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        TextStyle {
            font: FontId::default(),
            size: text_px,
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(line_height),
            letter_spacing_em: None,
        }
    }

    fn colors(theme: &Theme) -> (Color, Color) {
        let fg = theme.color_required("foreground");
        let muted = theme.color_required("muted-foreground");
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

        pub fn into_element<H: UiHost, I>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = AnyElement>,
        {
            let theme = Theme::global(&*cx.app).clone();
            cx.container(
                decl_style::container_props(&theme, self.chrome, self.layout),
                move |cx| children(cx),
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

        pub fn into_element<H: UiHost, I>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = AnyElement>,
        {
            let theme = Theme::global(&*cx.app).clone();
            let (_fg, muted) = colors(&theme);
            let style = text_style(&theme);

            let gap = theme
                .metric_by_key("component.breadcrumb.gap")
                // Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport,
                // so we default to the `sm` outcome (`gap-2.5`) for 1:1 geometry alignment.
                .unwrap_or_else(|| MetricRef::space(Space::N2p5).resolve(&theme));

            cx.container(
                decl_style::container_props(&theme, self.chrome, self.layout),
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: Default::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap,
                            padding: fret_core::Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: true,
                        },
                        move |cx| {
                            // Apply the list-level muted foreground by default; individual children
                            // (BreadcrumbLink/Page) can override it.
                            let mut out: Vec<AnyElement> = children(cx).into_iter().collect();
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
                    )]
                },
            )
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

        pub fn into_element<H: UiHost, I>(
            self,
            cx: &mut ElementContext<'_, H>,
            children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
        ) -> AnyElement
        where
            I: IntoIterator<Item = AnyElement>,
        {
            let theme = Theme::global(&*cx.app).clone();
            let item_gap = theme
                .metric_by_key("component.breadcrumb.item_gap")
                .unwrap_or_else(|| MetricRef::space(Space::N1p5).resolve(&theme));

            cx.container(
                decl_style::container_props(&theme, self.chrome, self.layout),
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: Default::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: item_gap,
                            padding: fret_core::Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| children(cx),
                    )]
                },
            )
        }
    }

    #[derive(Debug, Clone)]
    pub struct BreadcrumbLink {
        label: Arc<str>,
        command: Option<CommandId>,
        truncate: bool,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl BreadcrumbLink {
        pub fn new(label: impl Into<Arc<str>>) -> Self {
            Self {
                label: label.into(),
                command: None,
                truncate: false,
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
            }
        }

        pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
            self.command = Some(command.into());
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

        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let theme = Theme::global(&*cx.app).clone();
            let style = text_style(&theme);
            let (fg, muted) = colors(&theme);
            let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);
            let label = self.label.clone();
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

            if let Some(command) = self.command
                && !disabled_by_gating
            {
                cx.pressable(PressableProps::default(), move |cx, st| {
                    cx.pressable_dispatch_command_if_enabled(command.clone());
                    let color = if st.hovered { fg } else { muted };
                    vec![cx.container(
                        decl_style::container_props(
                            &theme,
                            self.chrome.clone(),
                            self.layout.clone(),
                        ),
                        move |cx| {
                            let mut text = ui::text(cx, label.clone())
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
                        },
                    )]
                })
            } else {
                cx.container(
                    decl_style::container_props(&theme, self.chrome, self.layout),
                    move |cx| {
                        let mut text = ui::text(cx, label)
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
                    },
                )
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct BreadcrumbPage {
        label: Arc<str>,
        truncate: bool,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl BreadcrumbPage {
        pub fn new(label: impl Into<Arc<str>>) -> Self {
            Self {
                label: label.into(),
                truncate: false,
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
            }
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

        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let theme = Theme::global(&*cx.app).clone();
            let style = text_style(&theme);
            let (fg, _muted) = colors(&theme);
            let label = self.label;
            let text_px = style.size;
            let font_weight = style.weight;
            let line_height = style.line_height;
            let letter_spacing_em = style.letter_spacing_em;
            let (wrap, overflow) = if self.truncate {
                (TextWrap::None, TextOverflow::Ellipsis)
            } else {
                (TextWrap::Word, TextOverflow::Clip)
            };
            cx.container(
                decl_style::container_props(&theme, self.chrome, self.layout),
                move |cx| {
                    let mut text = ui::text(cx, label)
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
                },
            )
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub enum BreadcrumbSeparatorKind {
        #[default]
        ChevronRight,
        Slash,
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

        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let theme = Theme::global(&*cx.app).clone();
            let (_fg, muted) = colors(&theme);

            let icon = match self.kind {
                BreadcrumbSeparatorKind::ChevronRight => ids::ui::CHEVRON_RIGHT,
                BreadcrumbSeparatorKind::Slash => ids::ui::SLASH,
            };

            // Upstream applies `[&>svg]:size-3.5` (14px).
            let icon_el = decl_icon::icon_with(
                cx,
                icon,
                Some(Px(14.0)),
                Some(fret_ui_kit::ColorRef::Color(muted)),
            );

            // Ensure the separator is a "leaf-sized" node in layouts that scan by size.
            cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: fret_ui::element::Length::Px(Px(14.0)),
                            height: fret_ui::element::Length::Px(Px(14.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..decl_style::container_props(&theme, self.chrome, self.layout)
                },
                move |_cx| vec![icon_el],
            )
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

        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let theme = Theme::global(&*cx.app).clone();
            let (_fg, muted) = colors(&theme);
            let size = self.size.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.breadcrumb.ellipsis_size")
                    .unwrap_or(Px(36.0))
            });

            let mut props = FlexProps {
                layout: Default::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
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

            cx.container(
                decl_style::container_props(&theme, self.chrome, self.layout),
                move |cx| vec![cx.flex(props, move |_cx| vec![icon])],
            )
        }
    }
}
