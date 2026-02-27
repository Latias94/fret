use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, ElementKind, Elements, FlexProps, GridProps, MainAlign, Overflow,
    PressableProps, ScrollAxis,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::scroll_area::ScrollAreaType;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space, ui};

use crate::layout as shadcn_layout;

fn table_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.table.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.table.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::NORMAL;
    style
}

fn row_min_h(theme: &Theme) -> fret_core::Px {
    theme
        .metric_by_key("component.table.row_min_h")
        .unwrap_or(fret_core::Px(40.0))
}

fn border_color(theme: &Theme) -> fret_core::Color {
    theme.color_token("border")
}

fn muted_bg(theme: &Theme) -> fret_core::Color {
    theme.color_token("muted")
}

fn muted_fg(theme: &Theme) -> fret_core::Color {
    theme.color_token("muted-foreground")
}

fn foreground(theme: &Theme) -> fret_core::Color {
    theme.color_token("foreground")
}

fn apply_table_cell_text_defaults(mut child: AnyElement) -> AnyElement {
    match &mut child.kind {
        ElementKind::Text(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::StyledText(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::SelectableText(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
        }
        _ => {}
    }
    child
}

fn apply_table_footer_inherited_style(mut child: AnyElement, style: &TextStyle) -> AnyElement {
    match &mut child.kind {
        ElementKind::Text(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            } else if let Some(existing) = props.style.as_mut()
                && existing.weight == FontWeight::NORMAL
            {
                existing.weight = style.weight;
            }
        }
        ElementKind::StyledText(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            } else if let Some(existing) = props.style.as_mut()
                && existing.weight == FontWeight::NORMAL
            {
                existing.weight = style.weight;
            }
        }
        ElementKind::SelectableText(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            } else if let Some(existing) = props.style.as_mut()
                && existing.weight == FontWeight::NORMAL
            {
                existing.weight = style.weight;
            }
        }
        _ => {}
    }

    child.children = child
        .children
        .into_iter()
        .map(|child| apply_table_footer_inherited_style(child, style))
        .collect();
    child
}

/// shadcn/ui `Table` root.
///
/// Upstream wraps `<table>` in a horizontally scrollable container (`overflow-x-auto`). We model
/// that outcome by defaulting `Table` to a horizontal [`ScrollArea`] wrapper (best-effort).
#[derive(Debug)]
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        // shadcn: `w-full caption-bottom text-sm`.
        let table_layout = LayoutRefinement::default().w_full().merge(self.layout);
        let mut props = decl_style::container_props(theme, self.chrome, table_layout);
        props.layout.overflow = Overflow::Visible;

        let children = self.children;
        let table = shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        );

        crate::ScrollArea::new([table])
            .axis(ScrollAxis::X)
            .type_(ScrollAreaType::Auto)
            .refine_layout(LayoutRefinement::default().w_full().relative())
            .into_element(cx)
    }
}

/// shadcn/ui `TableHeader` (`thead`).
#[derive(Debug)]
pub struct TableHeader {
    children: Vec<AnyElement>,
}

impl TableHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            // HTML table sections behave like block containers and fill the table width.
            LayoutRefinement::default().w_full(),
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
#[derive(Debug)]
pub struct TableBody {
    children: Vec<AnyElement>,
}

impl TableBody {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            // HTML table sections behave like block containers and fill the table width.
            LayoutRefinement::default().w_full(),
        );
        let mut children = self.children;
        if let Some(last) = children.last_mut() {
            clear_table_row_border_bottom(last);
        }
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

/// shadcn/ui `TableFooter` (`tfoot`).
#[derive(Debug)]
pub struct TableFooter {
    children: Vec<AnyElement>,
}

impl TableFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let footer_text_style = TextStyle {
            weight: FontWeight::MEDIUM,
            ..table_text_style(theme)
        };

        let mut bg = muted_bg(theme);
        bg.a *= 0.5;
        let border = border_color(theme);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));
        // HTML table sections behave like block containers and fill the table width.
        let mut props =
            decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full());
        props.border = Edges {
            top: fret_core::Px(1.0),
            right: fret_core::Px(0.0),
            bottom: fret_core::Px(0.0),
            left: fret_core::Px(0.0),
        };

        let mut children: Vec<AnyElement> = self
            .children
            .into_iter()
            .map(|child| apply_table_footer_inherited_style(child, &footer_text_style))
            .collect();
        if let Some(last) = children.last_mut() {
            clear_table_row_border_bottom(last);
        }
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

fn clear_table_row_border_bottom(el: &mut AnyElement) -> bool {
    match &mut el.kind {
        ElementKind::Container(props) => {
            if props.border.top.0 == 0.0
                && props.border.right.0 == 0.0
                && props.border.left.0 == 0.0
                && props.border.bottom.0 > 0.0
            {
                props.border.bottom = fret_core::Px(0.0);
                return true;
            }
        }
        _ => {}
    }

    for child in &mut el.children {
        if clear_table_row_border_bottom(child) {
            return true;
        }
    }
    false
}

/// shadcn/ui `TableRow` (`tr`).
///
/// This is implemented as a `Pressable` wrapper for hover/selected background parity.
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

    #[track_caller]
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

            vec![cx.container(props, move |cx| {
                let grid = GridProps {
                    cols,
                    gap: fret_core::Px(0.0).into(),
                    padding: fret_core::geometry::Edges::all(fret_core::Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    layout: grid_layout,
                    ..Default::default()
                };

                let cells = assign_grid_column_starts(children);
                vec![cx.grid(grid, move |_cx| cells)]
            })]
        })
    }
}

fn assign_grid_column_starts<I>(cells: I) -> Elements
where
    I: IntoIterator<Item = AnyElement>,
{
    let cells: Vec<AnyElement> = cells.into_iter().collect();

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
    let mut out: Vec<AnyElement> = Vec::with_capacity(cells.len());
    for cell in cells {
        let span = grid_span(&cell);
        let start = col;
        out.push(set_grid_start(cell, start));
        col = col.saturating_add(span as i16);
    }

    out.into()
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

    #[track_caller]
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
                    gap: fret_core::Px(0.0).into(),
                    padding: Edges::all(fret_core::Px(0.0)).into(),
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
#[derive(Debug)]
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

    #[track_caller]
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
        let child = apply_table_cell_text_defaults(self.child);
        cx.container(props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: row_layout,
                    direction: Axis::Horizontal,
                    gap: fret_core::Px(0.0).into(),
                    padding: Edges::all(fret_core::Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| vec![cx.container(wrapper_props, move |_cx| vec![child])],
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

    #[track_caller]
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Color, Point, Px, Rect, Size};
    use fret_ui::element::{ContainerProps, Length, Overflow};

    fn find_container_with_background(el: &AnyElement, bg: Color) -> Option<&ContainerProps> {
        match &el.kind {
            ElementKind::Container(props) => {
                if props.background == Some(bg) {
                    return Some(props);
                }
            }
            _ => {}
        }
        for child in &el.children {
            if let Some(found) = find_container_with_background(child, bg) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn table_root_defaults_to_w_full_but_allows_overrides() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        let bg = Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = Table::new([cx.text("body")])
                .refine_style(ChromeRefinement::default().bg(ColorRef::Color(bg)))
                .into_element(cx);
            let props = find_container_with_background(&el, bg).expect("table inner container");
            assert_eq!(props.layout.size.width, Length::Fill);
            assert_eq!(props.layout.overflow, Overflow::Visible);
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = Table::new([cx.text("body")])
                .refine_style(ChromeRefinement::default().bg(ColorRef::Color(bg)))
                .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                .into_element(cx);
            let props = find_container_with_background(&el, bg).expect("table inner container");
            assert_eq!(props.layout.size.width, Length::Px(Px(320.0)));
        });
    }

    #[test]
    fn table_body_clears_last_row_border_bottom() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let row1 =
                TableRow::new(1, [TableCell::new(cx.text("a")).into_element(cx)]).into_element(cx);
            let row2 =
                TableRow::new(1, [TableCell::new(cx.text("b")).into_element(cx)]).into_element(cx);

            let body = TableBody::new([row1, row2]).into_element(cx);

            fn find_row_border_container(el: &AnyElement) -> Option<&ContainerProps> {
                match &el.kind {
                    ElementKind::Container(props) => {
                        if props.border.top.0 == 0.0
                            && props.border.right.0 == 0.0
                            && props.border.left.0 == 0.0
                            && props.border.bottom.0 >= 0.0
                            && props.border_color.is_some()
                        {
                            return Some(props);
                        }
                    }
                    _ => {}
                }
                for child in &el.children {
                    if let Some(found) = find_row_border_container(child) {
                        return Some(found);
                    }
                }
                None
            }

            fn collect_pressable_borders(el: &AnyElement, out: &mut Vec<fret_core::Px>) {
                if matches!(el.kind, ElementKind::Pressable(_)) {
                    let border = find_row_border_container(el)
                        .expect("expected TableRow to contain a border container")
                        .border
                        .bottom;
                    out.push(border);
                }
                for child in &el.children {
                    collect_pressable_borders(child, out);
                }
            }

            let mut borders = Vec::new();
            collect_pressable_borders(&body, &mut borders);
            assert!(
                borders.len() >= 2,
                "expected at least two TableRow pressables"
            );

            let first_border = borders[0];
            let last_border = borders[borders.len() - 1];

            assert_eq!(
                first_border,
                Px(1.0),
                "expected non-last row to keep border-bottom"
            );
            assert_eq!(
                last_border,
                Px(0.0),
                "expected TableBody to clear the last row border-bottom (shadcn: [&_tr:last-child]:border-0)"
            );
        });
    }

    #[test]
    fn table_footer_defaults_to_font_medium_for_plain_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fn find_text_weight(el: &AnyElement, needle: &str) -> Option<FontWeight> {
            match &el.kind {
                ElementKind::Text(props) => {
                    if props.text.as_ref() == needle {
                        return props.style.as_ref().map(|s| s.weight);
                    }
                }
                _ => {}
            }

            for child in &el.children {
                if let Some(found) = find_text_weight(child, needle) {
                    return Some(found);
                }
            }
            None
        }

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let footer = TableFooter::new([TableRow::new(
                2,
                [
                    TableCell::new(ui::text(cx, "Total").into_element(cx)).into_element(cx),
                    TableCell::new(ui::text(cx, "$2,500.00").into_element(cx)).into_element(cx),
                ],
            )
            .into_element(cx)])
            .into_element(cx);

            let weight = find_text_weight(&footer, "Total").expect("find Total text weight");
            assert_eq!(weight, FontWeight::MEDIUM);
        });
    }
}
