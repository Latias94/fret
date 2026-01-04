use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::{MetricRef, Space};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BreadcrumbItemKind {
    Link,
    Page,
    Ellipsis,
}

/// A shadcn/ui v4-aligned breadcrumb builder.
///
/// Upstream composes `Breadcrumb` + `BreadcrumbList` + `BreadcrumbItem` + `BreadcrumbLink/Page`
/// + `BreadcrumbSeparator/Ellipsis`. In Fret we provide a compact builder surface that can render
///   the same visual/interaction result in a single declarative element tree.
#[derive(Debug, Clone, Default)]
pub struct Breadcrumb {
    items: Vec<BreadcrumbItem>,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let gap = theme
            .metric_by_key("component.breadcrumb.gap")
            .unwrap_or_else(|| MetricRef::space(Space::N1p5).resolve(&theme));

        let text_px = theme
            .metric_by_key("component.breadcrumb.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.breadcrumb.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let muted = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let style = TextStyle {
            font: FontId::default(),
            size: text_px,
            weight: FontWeight::NORMAL,
            line_height: Some(line_height),
            letter_spacing_em: None,
        };

        let mut children: Vec<AnyElement> = Vec::new();
        let n = self.items.len();
        for (i, mut item) in self.items.into_iter().enumerate() {
            let is_last = i + 1 == n;
            item.kind = match item.kind {
                BreadcrumbItemKind::Ellipsis => BreadcrumbItemKind::Ellipsis,
                _ if is_last => BreadcrumbItemKind::Page,
                _ => BreadcrumbItemKind::Link,
            };

            children.push(item.render(cx, &style, muted, fg));
            if !is_last {
                children.push(breadcrumb_separator(cx, &style, muted));
            }
        }

        cx.flex(
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
        )
    }
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
            BreadcrumbItemKind::Ellipsis => breadcrumb_ellipsis(cx, base_style, muted),
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
) -> AnyElement {
    // shadcn uses lucide `ChevronRight` at size-3.5; this is a text fallback.
    breadcrumb_text(cx, Arc::from("›"), base_style, muted)
}

fn breadcrumb_ellipsis<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    base_style: &TextStyle,
    muted: Color,
) -> AnyElement {
    // shadcn uses a 36x36 box with a `MoreHorizontal` icon.
    // We keep the same footprint with a centered ellipsis glyph.
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
        vec![breadcrumb_text(cx, Arc::from("…"), base_style, muted)]
    })
}
