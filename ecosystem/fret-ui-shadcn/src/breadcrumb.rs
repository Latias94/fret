use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_icons::{IconId, ids};
use fret_runtime::CommandId;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};

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

        children.push(item.render(cx, &style, muted, fg));
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
        base_style: &TextStyle,
        muted: Color,
        fg: Color,
    ) -> AnyElement {
        match self.kind {
            BreadcrumbItemKind::Ellipsis => breadcrumb_ellipsis(cx, muted),
            BreadcrumbItemKind::Page => breadcrumb_text(cx, self.label, base_style, fg),
            BreadcrumbItemKind::Link => {
                if self.disabled {
                    return breadcrumb_text(cx, self.label, base_style, muted);
                }

                let Some(command) = self.command else {
                    // Non-clickable link-like text (shadcn allows `<a>` without a URL).
                    return breadcrumb_link_text(cx, self.label, base_style, muted, fg, false);
                };

                cx.pressable(PressableProps::default(), move |cx, st| {
                    cx.pressable_dispatch_command(command.clone());
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
    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: Some(base_style.clone()),
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
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
