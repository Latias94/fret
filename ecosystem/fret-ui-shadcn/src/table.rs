use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, FontId, FontWeight, TextAlign, TextOverflow, TextStyle, TextWrap};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, ElementKind, FlexProps, MainAlign, Overflow, PressableProps, ScrollAxis,
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

fn apply_table_cell_text_defaults(
    mut child: AnyElement,
    text_align: Option<TextAlign>,
) -> AnyElement {
    match &mut child.kind {
        ElementKind::Text(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
            if let Some(align) = text_align {
                props.align = align;
                props.layout.size.width = fret_ui::element::Length::Fill;
            }
        }
        ElementKind::StyledText(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
            if let Some(align) = text_align {
                props.align = align;
                props.layout.size.width = fret_ui::element::Length::Fill;
            }
        }
        ElementKind::SelectableText(props) => {
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
            if let Some(align) = text_align {
                props.align = align;
                props.layout.size.width = fret_ui::element::Length::Fill;
            }
        }
        _ => {}
    }
    child
}

fn apply_table_inherited_text_style(mut child: AnyElement, style: &TextStyle) -> AnyElement {
    match &mut child.kind {
        ElementKind::Text(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            }
        }
        ElementKind::StyledText(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            }
        }
        ElementKind::SelectableText(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            }
        }
        _ => {}
    }

    child.children = child
        .children
        .into_iter()
        .map(|child| apply_table_inherited_text_style(child, style))
        .collect();
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
        let text_style = table_text_style(theme);

        // shadcn: `w-full caption-bottom text-sm`.
        let table_layout = LayoutRefinement::default().w_full().merge(self.layout);
        let mut props = decl_style::container_props(theme, self.chrome, table_layout);
        props.layout.overflow = Overflow::Visible;

        let children: Vec<AnyElement> = self
            .children
            .into_iter()
            .map(|child| apply_table_inherited_text_style(child, &text_style))
            .collect();
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
            let (props, row_layout) = {
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

                let row_layout =
                    decl_style::layout_style(theme, LayoutRefinement::default().w_full());

                (props, row_layout)
            };

            vec![cx.container(props, move |cx| {
                let row = FlexProps {
                    layout: row_layout,
                    direction: Axis::Horizontal,
                    gap: fret_core::Px(0.0).into(),
                    padding: fret_core::geometry::Edges::all(fret_core::Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                };

                let _ = cols;
                vec![cx.flex(row, move |_cx| children)]
            })]
        })
    }
}

/// shadcn/ui `TableHead` (`th`).
#[derive(Debug, Clone)]
pub struct TableHead {
    text: Arc<str>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    text_align: TextAlign,
}

impl TableHead {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            text_align: TextAlign::Start,
        }
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    pub fn text_align_end(self) -> Self {
        self.text_align(TextAlign::End)
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
        let text_align = self.text_align;
        let caller_set_flex = self.layout.flex_item.is_some();
        let layout_override = self.layout;

        let style = TextStyle {
            weight: FontWeight::MEDIUM,
            ..table_text_style(theme)
        };
        let fg = foreground(theme);

        let chrome = ChromeRefinement::default().px(px).py(py).merge(self.chrome);
        let props = decl_style::container_props(theme, chrome, {
            let mut layout = LayoutRefinement::default()
                .flex_1()
                .min_w_0()
                .min_h(row_min_h(theme))
                .merge(layout_override);
            let caller_set_width = layout
                .size
                .as_ref()
                .and_then(|s| s.width.as_ref())
                .is_some();
            if caller_set_width && !caller_set_flex {
                layout = layout.flex_none();
            }
            layout
        });

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
                        .text_align(text_align)
                        .w_full()
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
    text_align: Option<TextAlign>,
}

impl TableCell {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            col_span: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            text_align: None,
        }
    }

    /// Sets `colSpan` semantics for a flex-backed table row.
    ///
    /// This is modeled as `flex-grow = span` (only when the caller did not provide an explicit
    /// width or flex-item overrides).
    pub fn col_span(mut self, span: u16) -> Self {
        self.col_span = Some(span.max(1));
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    pub fn text_align_end(self) -> Self {
        self.text_align(TextAlign::End)
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
        let caller_set_flex = self.layout.flex_item.is_some();
        let caller_set_width = self
            .layout
            .size
            .as_ref()
            .and_then(|s| s.width.as_ref())
            .is_some();
        let layout_override = self.layout;

        let chrome = ChromeRefinement::default().px(px).py(py).merge(self.chrome);
        let mut layout = LayoutRefinement::default()
            .flex_1()
            .min_w_0()
            .merge(layout_override);
        if caller_set_width && !caller_set_flex {
            layout = layout.flex_none();
        }
        if let Some(span) = self.col_span
            && !caller_set_width
            && !caller_set_flex
        {
            layout = layout.flex_grow(span as f32);
        }

        let props = decl_style::container_props(theme, chrome, layout);
        let row_layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full());
        let wrapper_props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full().h_full().min_w_0(),
        );
        let child = apply_table_cell_text_defaults(self.child, self.text_align);
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
    use fret_ui::element::{ContainerProps, Length, Overflow, TextProps};

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
    fn table_applies_text_sm_defaults_to_unstyled_text_cells() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fn find_text<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a TextProps> {
            match &el.kind {
                ElementKind::Text(props) if props.text.as_ref() == needle => {
                    return Some(props);
                }
                _ => {}
            }
            for child in &el.children {
                if let Some(found) = find_text(child, needle) {
                    return Some(found);
                }
            }
            None
        }

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let expected = table_text_style(theme);

            let table = Table::new([TableBody::new([TableRow::new(
                1,
                [TableCell::new(cx.text("cell")).into_element(cx)],
            )
            .into_element(cx)])
            .into_element(cx)])
            .into_element(cx);

            let text = find_text(&table, "cell").expect("expected table cell text node");
            let actual = text
                .style
                .as_ref()
                .expect("expected inherited table text style");
            assert_eq!(actual.size, expected.size);
            assert_eq!(actual.line_height, expected.line_height);
            assert_eq!(actual.weight, expected.weight);
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
