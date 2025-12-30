use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space};
use fret_core::geometry::Edges;
use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, CrossAlign, GridProps, MainAlign, Overflow, PressableProps, TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

fn table_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.table.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.table.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn row_min_h(theme: &Theme) -> MetricRef {
    MetricRef::Px(
        theme
            .metric_by_key("component.table.row_min_h")
            .unwrap_or(fret_core::Px(40.0)),
    )
}

fn border_color(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border)
}

fn muted_bg(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("muted")
        .or_else(|| theme.color_by_key("muted.background"))
        .unwrap_or(theme.colors.hover_background)
}

fn muted_fg(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted)
}

fn foreground(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary)
}

/// shadcn/ui `Table` root.
///
/// Upstream wraps `<table>` in a horizontally scrollable container. Fret does not support
/// horizontal scrolling in the core Scroll primitive yet; for now, `Table` is a layout + styling
/// facade. Wrap it in a `ScrollArea` if you need clipping/scroll behavior.
#[derive(Debug, Clone)]
pub struct Table {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Table {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        // shadcn: `w-full caption-bottom text-sm`.
        let mut props = decl_style::container_props(&theme, self.chrome, self.layout.w_full());
        props.layout.overflow = Overflow::Visible;

        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `TableHeader` (`thead`).
#[derive(Debug, Clone)]
pub struct TableHeader {
    children: Vec<AnyElement>,
}

impl TableHeader {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `TableBody` (`tbody`).
#[derive(Debug, Clone)]
pub struct TableBody {
    children: Vec<AnyElement>,
}

impl TableBody {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `TableFooter` (`tfoot`).
#[derive(Debug, Clone)]
pub struct TableFooter {
    children: Vec<AnyElement>,
}

impl TableFooter {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut bg = muted_bg(&theme);
        bg.a *= 0.5;
        let border = border_color(&theme);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));
        let mut props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        props.border = Edges {
            top: fret_core::Px(1.0),
            right: fret_core::Px(0.0),
            bottom: fret_core::Px(0.0),
            left: fret_core::Px(0.0),
        };

        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `TableRow` (`tr`).
///
/// This is implemented as a `Pressable` wrapper for hover/selected background parity.
#[derive(Debug, Clone)]
pub struct TableRow {
    cols: u16,
    children: Vec<AnyElement>,
    selected: bool,
    enabled: bool,
    on_click: Option<fret_runtime::CommandId>,
    border_bottom: bool,
}

impl TableRow {
    pub fn new(cols: u16, children: Vec<AnyElement>) -> Self {
        Self {
            cols: cols.max(1),
            children,
            selected: false,
            enabled: true,
            on_click: None,
            border_bottom: true,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_click(mut self, cmd: impl Into<fret_runtime::CommandId>) -> Self {
        self.on_click = Some(cmd.into());
        self
    }

    pub fn border_bottom(mut self, enabled: bool) -> Self {
        self.border_bottom = enabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let cols = self.cols;
        let selected = self.selected;
        let enabled = self.enabled;
        let on_click = self.on_click.clone();
        let border_bottom = self.border_bottom;
        let children = self.children;

        let pressable_layout = {
            let theme = Theme::global(&*cx.app);
            decl_style::layout_style(theme, LayoutRefinement::default().w_full())
        };
        let pressable = PressableProps {
            enabled,
            layout: pressable_layout,
            ..Default::default()
        };

        cx.pressable(pressable, move |cx, state| {
            cx.pressable_dispatch_command_opt(on_click);
            let theme = Theme::global(&*cx.app).clone();

            let mut hover_bg = muted_bg(&theme);
            hover_bg.a *= 0.5;
            let selected_bg = muted_bg(&theme);

            let border = border_color(&theme);
            let mut chrome = ChromeRefinement::default()
                .border_1()
                .border_color(ColorRef::Color(border));
            if selected {
                chrome = chrome.bg(ColorRef::Color(selected_bg));
            } else if state.hovered {
                chrome = chrome.bg(ColorRef::Color(hover_bg));
            }

            let layout = LayoutRefinement::default()
                .w_full()
                .min_h(row_min_h(&theme));
            let mut props = decl_style::container_props(&theme, chrome, layout);
            props.layout.overflow = Overflow::Visible;
            props.border = if border_bottom {
                Edges {
                    top: fret_core::Px(0.0),
                    right: fret_core::Px(0.0),
                    bottom: fret_core::Px(1.0),
                    left: fret_core::Px(0.0),
                }
            } else {
                Edges::all(fret_core::Px(0.0))
            };

            let row_children = children.clone();
            vec![cx.container(props, move |cx| {
                let grid = GridProps {
                    cols,
                    gap: fret_core::Px(0.0),
                    padding: fret_core::geometry::Edges::all(fret_core::Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                    ..Default::default()
                };

                let cells = row_children.clone();
                vec![cx.grid(grid, move |_cx| cells)]
            })]
        })
    }
}

/// shadcn/ui `TableHead` (`th`).
#[derive(Debug, Clone)]
pub struct TableHead {
    text: Arc<str>,
}

impl TableHead {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let px = Space::N2;
        let py = Space::N0;

        let style = TextStyle {
            weight: FontWeight::MEDIUM,
            ..table_text_style(&theme)
        };
        let fg = foreground(&theme);

        let chrome = ChromeRefinement::default().px(px).py(py);
        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());

        let text = self.text;
        cx.container(props, move |cx| {
            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        })
    }
}

/// shadcn/ui `TableCell` (`td`).
#[derive(Debug, Clone)]
pub struct TableCell {
    child: AnyElement,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TableCell {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let px = Space::N2;
        let py = Space::N2;

        let chrome = ChromeRefinement::default().px(px).py(py).merge(self.chrome);
        let props = decl_style::container_props(&theme, chrome, self.layout);
        let child = self.child;
        cx.container(props, move |_cx| vec![child])
    }
}

/// shadcn/ui `TableCaption` (`caption`).
#[derive(Debug, Clone)]
pub struct TableCaption {
    text: Arc<str>,
}

impl TableCaption {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().mt(Space::N4),
        );

        let style = table_text_style(&theme);
        let fg = muted_fg(&theme);
        let text = self.text;

        cx.container(props, move |cx| {
            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text,
                style: Some(style),
                color: Some(fg),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            })]
        })
    }
}
