use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, Elements, FlexProps, GridProps, MainAlign, Overflow, PressableProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space, ui};

use crate::layout as shadcn_layout;

fn table_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.table.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.table.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn row_min_h(theme: &Theme) -> fret_core::Px {
    theme
        .metric_by_key("component.table.row_min_h")
        .unwrap_or(fret_core::Px(40.0))
}

fn border_color(theme: &Theme) -> fret_core::Color {
    theme.color_required("border")
}

fn muted_bg(theme: &Theme) -> fret_core::Color {
    theme.color_required("muted")
}

fn muted_fg(theme: &Theme) -> fret_core::Color {
    theme.color_required("muted-foreground")
}

fn foreground(theme: &Theme) -> fret_core::Color {
    theme.color_required("foreground")
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        // shadcn: `w-full caption-bottom text-sm`.
        let mut props = decl_style::container_props(theme, self.chrome, self.layout.w_full());
        props.layout.overflow = Overflow::Visible;

        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

/// shadcn/ui `TableHeader` (`thead`).
#[derive(Debug, Clone)]
pub struct TableHeader {
    children: Vec<AnyElement>,
}

impl TableHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

/// shadcn/ui `TableBody` (`tbody`).
#[derive(Debug, Clone)]
pub struct TableBody {
    children: Vec<AnyElement>,
}

impl TableBody {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

/// shadcn/ui `TableFooter` (`tfoot`).
#[derive(Debug, Clone)]
pub struct TableFooter {
    children: Vec<AnyElement>,
}

impl TableFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let mut bg = muted_bg(theme);
        bg.a *= 0.5;
        let border = border_color(theme);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));
        let mut props = decl_style::container_props(theme, chrome, LayoutRefinement::default());
        props.border = Edges {
            top: fret_core::Px(1.0),
            right: fret_core::Px(0.0),
            bottom: fret_core::Px(0.0),
            left: fret_core::Px(0.0),
        };

        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

/// shadcn/ui `TableRow` (`tr`).
///
/// This is implemented as a `Pressable` wrapper for hover/selected background parity.
#[derive(Clone)]
pub struct TableRow {
    cols: u16,
    children: Vec<AnyElement>,
    selected: bool,
    enabled: bool,
    on_click: Option<fret_runtime::CommandId>,
    on_activate: Option<OnActivate>,
    border_bottom: bool,
}

impl std::fmt::Debug for TableRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableRow")
            .field("cols", &self.cols)
            .field("selected", &self.selected)
            .field("enabled", &self.enabled)
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("border_bottom", &self.border_bottom)
            .finish_non_exhaustive()
    }
}

impl TableRow {
    pub fn new(cols: u16, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            cols: cols.max(1),
            children: children.into_iter().collect(),
            selected: false,
            enabled: true,
            on_click: None,
            on_activate: None,
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

    pub fn on_activate(mut self, handler: OnActivate) -> Self {
        self.on_activate = Some(handler);
        self
    }

    pub fn border_bottom(mut self, enabled: bool) -> Self {
        self.border_bottom = enabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let cols = self.cols;
        let selected = self.selected;
        let mut enabled = self.enabled;
        let on_click = self.on_click.clone();
        if let Some(cmd) = on_click.as_ref() {
            enabled = enabled && cx.command_is_enabled(cmd);
        }
        let on_activate = self.on_activate.clone();
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
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_add_on_activate(on_activate);
            }
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            let (props, grid_layout) = {
                let theme = Theme::global(&*cx.app);

                let mut hover_bg = muted_bg(theme);
                hover_bg.a *= 0.5;
                let selected_bg = muted_bg(theme);

                let border = border_color(theme);
                let mut chrome = ChromeRefinement::default()
                    .border_1()
                    .border_color(ColorRef::Color(border));
                if selected {
                    chrome = chrome.bg(ColorRef::Color(selected_bg));
                } else if state.hovered {
                    chrome = chrome.bg(ColorRef::Color(hover_bg));
                }

                let layout = LayoutRefinement::default().w_full();
                let mut props = decl_style::container_props(theme, chrome, layout);
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

                let grid_layout =
                    decl_style::layout_style(theme, LayoutRefinement::default().w_full());

                (props, grid_layout)
            };

            let row_children = children.clone();
            vec![cx.container(props, move |cx| {
                let grid = GridProps {
                    cols,
                    gap: fret_core::Px(0.0),
                    padding: fret_core::geometry::Edges::all(fret_core::Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    layout: grid_layout,
                    ..Default::default()
                };

                let cells = assign_grid_column_starts(row_children.clone());
                vec![cx.grid(grid, move |_cx| cells.clone())]
            })]
        })
    }
}

fn assign_grid_column_starts<I>(cells: I) -> Elements
where
    I: IntoIterator<Item = AnyElement>,
{
    let mut cells: Vec<AnyElement> = cells.into_iter().collect();

    fn grid_span(cell: &AnyElement) -> u16 {
        match &cell.kind {
            fret_ui::element::ElementKind::Container(props) => {
                props.layout.grid.column.span.unwrap_or(1).max(1)
            }
            fret_ui::element::ElementKind::Semantics(props) => {
                props.layout.grid.column.span.unwrap_or(1).max(1)
            }
            _ => 1,
        }
    }

    fn set_grid_start(mut cell: AnyElement, start: i16) -> AnyElement {
        match &mut cell.kind {
            fret_ui::element::ElementKind::Container(props) => {
                if props.layout.grid.column.start.is_none() {
                    props.layout.grid.column.start = Some(start);
                }
            }
            fret_ui::element::ElementKind::Semantics(props) => {
                if props.layout.grid.column.start.is_none() {
                    props.layout.grid.column.start = Some(start);
                }
            }
            _ => {}
        }
        cell
    }

    let mut col: i16 = 1;
    for cell in &mut cells {
        let span = grid_span(cell);
        let start = col;
        *cell = set_grid_start(cell.clone(), start);
        col = col.saturating_add(span as i16);
    }

    cells.into()
}

/// shadcn/ui `TableHead` (`th`).
#[derive(Debug, Clone)]
pub struct TableHead {
    text: Arc<str>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TableHead {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let px = Space::N2;
        let py = Space::N0;

        let style = TextStyle {
            weight: FontWeight::MEDIUM,
            ..table_text_style(theme)
        };
        let fg = foreground(theme);

        let chrome = ChromeRefinement::default().px(px).py(py).merge(self.chrome);
        let props = decl_style::container_props(
            theme,
            chrome,
            LayoutRefinement::default()
                .w_full()
                .min_h(row_min_h(theme))
                .merge(self.layout),
        );

        let text = self.text;
        let content_layout =
            decl_style::layout_style(theme, LayoutRefinement::default().w_full().h_full());
        cx.container(props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: content_layout,
                    direction: Axis::Horizontal,
                    gap: fret_core::Px(0.0),
                    padding: Edges::all(fret_core::Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let mut head_text = ui::text(cx, text.clone())
                        .text_size_px(style.size)
                        .font_weight(style.weight)
                        .text_color(ColorRef::Color(fg))
                        .nowrap();
                    if let Some(line_height) = style.line_height {
                        head_text = head_text.line_height_px(line_height);
                    }
                    if let Some(letter_spacing_em) = style.letter_spacing_em {
                        head_text = head_text.letter_spacing_em(letter_spacing_em);
                    }
                    vec![head_text.into_element(cx)]
                },
            )]
        })
    }
}

/// shadcn/ui `TableCell` (`td`).
#[derive(Debug, Clone)]
pub struct TableCell {
    child: AnyElement,
    col_span: Option<u16>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TableCell {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            col_span: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Sets `colSpan` semantics for the underlying grid-backed table layout.
    ///
    /// Note: This only affects placement within Fret's `Grid` implementation; it does not imply
    /// HTML table semantics, and column sizing remains a separate concern.
    pub fn col_span(mut self, span: u16) -> Self {
        self.col_span = Some(span.max(1));
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
        let theme = Theme::global(&*cx.app);
        let px = Space::N2;
        let py = Space::N2;

        let chrome = ChromeRefinement::default().px(px).py(py).merge(self.chrome);
        let layout = LayoutRefinement::default().w_full().merge(self.layout);
        let mut props = decl_style::container_props(theme, chrome, layout);
        if let Some(span) = self.col_span {
            props.layout.grid.column.span = Some(span);
        }
        let row_layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full());
        let wrapper_props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().flex_1().min_w_0(),
        );
        let child = self.child;
        cx.container(props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: row_layout,
                    direction: Axis::Horizontal,
                    gap: fret_core::Px(0.0),
                    padding: Edges::all(fret_core::Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| vec![cx.container(wrapper_props.clone(), move |_cx| vec![child.clone()])],
            )]
        })
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().mt(Space::N4),
        );

        let style = table_text_style(theme);
        let fg = muted_fg(theme);
        let text = self.text;

        cx.container(props, move |cx| {
            let mut caption_text = ui::text(cx, text)
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(fg))
                .wrap(TextWrap::Word)
                .overflow(TextOverflow::Clip);
            if let Some(line_height) = style.line_height {
                caption_text = caption_text.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = style.letter_spacing_em {
                caption_text = caption_text.letter_spacing_em(letter_spacing_em);
            }
            vec![caption_text.into_element(cx)]
        })
    }
}
